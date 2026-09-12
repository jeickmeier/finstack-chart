//! FIX-GG04: finite expansion policies against pinned R values.
use chart_core::scales::{Bounds, GgplotExpansion};

#[test]
fn expansion_matches_r_reference() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/expansion.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 120);
    for case in cases {
        let pair = |name: &str| {
            [
                case[name][0].as_f64().unwrap(),
                case[name][1].as_f64().unwrap(),
            ]
        };
        let [a, b] = pair("domain");
        let output = GgplotExpansion {
            mult: pair("mult"),
            add: pair("add"),
        }
        .expand(Bounds::new(a, b).unwrap())
        .unwrap();
        for (got, expected) in [output.start(), output.end()]
            .into_iter()
            .zip(pair("expanded"))
        {
            assert!(
                (got - expected).abs() <= 2e-14 * expected.abs().max(f64::MIN_POSITIVE),
                "{case}: {got} != {expected}"
            );
        }
    }
}

#[test]
fn expansion_rejects_nonfinite_parameters_and_overflow() {
    let limits = Bounds::new(0., 1.).unwrap();
    assert!(
        GgplotExpansion {
            mult: [f64::NAN, 0.],
            add: [0.; 2]
        }
        .expand(limits)
        .is_err()
    );
    assert!(
        GgplotExpansion {
            mult: [0.; 2],
            add: [0., f64::INFINITY]
        }
        .expand(limits)
        .is_err()
    );
    assert!(
        GgplotExpansion::default()
            .expand(Bounds::new(-f64::MAX, f64::MAX).unwrap())
            .is_err()
    );
}

#[test]
fn automatic_numeric_axes_match_ggplot_panel_expansion() {
    use chart_core::{Rect, ResourceId, Revision, ScaleId, layout::*, prelude::*, services::*};
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/expansion.json"
    ))
    .unwrap();
    assert_eq!(fixture["panels"].as_array().unwrap().len(), 32);
    for case in fixture["panels"].as_array().unwrap() {
        let x = [
            case["domain"][0].as_f64().unwrap(),
            case["domain"][1].as_f64().unwrap(),
        ];
        let data = Data::columns()
            .column("x", x)
            .column("y", [0., 1.])
            .build()
            .unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let mut request = LayoutRequest::new(
            Rect::new(0., 0., 400., 200.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        for a in &mut request.axes {
            a.visible = false;
        }
        request.axes[0].visible = true;
        let domain = if case["explicit"] == true {
            chart_core::scales::ContinuousDomain::explicit(Bounds::new(x[0], x[1]).unwrap())
        } else {
            Default::default()
        };
        let log = case["kind"] == "log";
        if log {
            request.axes[0].scale = AxisScale::Nonlinear {
                transform: chart_core::scales::ScaleTransform::Log { base: 10. },
                domain,
            };
        } else if case["kind"] == "linear" {
            request.axes[0].scale = AxisScale::Linear(domain);
        }
        if case["custom"] == true {
            request.axes[0].expansion = Some(GgplotExpansion {
                mult: [
                    case["mult"][0].as_f64().unwrap(),
                    case["mult"][1].as_f64().unwrap(),
                ],
                add: [
                    case["add"][0].as_f64().unwrap(),
                    case["add"][1].as_f64().unwrap(),
                ],
            });
            let encoded = serde_json::to_string(&request.axes[0]).unwrap();
            let decoded: AxisSpec = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, request.axes[0]);
            let mut definition = p.definition().clone();
            definition.axes.push(decoded);
            assert_eq!(definition.wire_version(), 17);
        }
        let view = |frame: &LaidOutChart| match &frame.axes()[&ScaleId::new(0)].scale {
            ResolvedScale::Linear(scale) => scale.viewport(),
            ResolvedScale::Nonlinear(scale) => scale.viewport(),
            _ => panic!(),
        };
        let frame = layout(prepared.clone(), &request, &Metrics).unwrap();
        let ticks = &frame.axes()[&ScaleId::new(0)].ticks;
        let expected_breaks = case["breaks"].as_array().unwrap();
        let expected_labels = case["labels"].as_array().unwrap();
        assert_eq!(ticks.len(), expected_breaks.len(), "{case}");
        for (i, tick) in ticks.iter().enumerate() {
            let chart_core::composition::ScaleValue::Number(value) = tick.value else {
                panic!()
            };
            let expected = expected_breaks[i].as_f64().unwrap();
            assert!(
                (value - expected).abs() <= 2e-12 * expected.abs().max(1.),
                "{case}: {value} != {expected}"
            );
            assert_eq!(tick.label, expected_labels[i].as_str().unwrap(), "{case}");
        }
        for (i, got) in [view(&frame).start(), view(&frame).end()]
            .into_iter()
            .enumerate()
        {
            let got = if log { got.log10() } else { got };
            let expected = case["expanded"][i].as_f64().unwrap();
            assert!(
                (got - expected).abs() <= 2e-14 * expected.abs().max(1.),
                "{case}: {got} != {expected}"
            );
        }
        if let Some(projected) = case["projected"].as_array() {
            assert_eq!(projected, &vec![serde_json::json!(0.5); 2]);
            match &frame.axes()[&ScaleId::new(0)].scale {
                ResolvedScale::Linear(scale) => {
                    let midpoint = (scale.range().start() + scale.range().end()) / 2.;
                    assert_eq!(scale.map(10.).unwrap(), Some(midpoint));
                    assert_eq!(scale.invert(midpoint).unwrap(), 10.);
                    assert_eq!(scale.ticks(5, 64).unwrap().len(), 1);
                }
                ResolvedScale::Nonlinear(scale) => {
                    let midpoint = (scale.range().start() + scale.range().end()) / 2.;
                    assert_eq!(scale.map(10.).unwrap(), Some(midpoint));
                    assert_eq!(scale.map_transformed(1.).unwrap(), Some(midpoint));
                    assert_eq!(scale.invert(midpoint).unwrap(), 10.);
                    assert_eq!(scale.ticks(5, 64).unwrap().len(), 1);
                }
                _ => panic!(),
            }
        }
        // An explicit navigation window remains exact after automatic training.
        request.axes[0].viewport = Some(Bounds::new(2., 4.).unwrap());
        let frame = layout(prepared.clone(), &request, &Metrics).unwrap();
        assert_eq!(view(&frame), Bounds::new(2., 4.).unwrap());
        if case["kind"] == "auto" && x == [0., 1.] {
            request.axes[0].viewport = None;
            request.axes[0].profile = GuideProfile::D3_3_0_0;
            let frame = layout(prepared, &request, &Metrics).unwrap();
            let ticks = &frame.axes()[&ScaleId::new(0)].ticks;
            assert_eq!(ticks.len(), 11);
            for (i, tick) in ticks.iter().enumerate() {
                assert_eq!(
                    tick.value,
                    chart_core::composition::ScaleValue::Number(i as f64 / 10.)
                );
                assert_eq!(tick.label, format!("{:.1}", i as f64 / 10.));
            }
        }
    }
}
