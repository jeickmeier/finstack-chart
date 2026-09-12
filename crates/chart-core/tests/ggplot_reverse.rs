//! FIX-GG04: decreasing positional transform through primary authoring and layout.
use chart_core::{
    Rect, ResourceId, Revision, ScaleId,
    grammar::PreparedGeometry,
    layout::*,
    prelude::*,
    scales::{Bounds, GgplotExpansion, ScaleTransform},
    services::*,
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn near(got: f64, expected: f64) {
    assert!(
        (got - expected).abs() <= 2e-13 * expected.abs().max(1.),
        "{got} != {expected}"
    );
}
#[test]
fn reverse_primary_axes_match_r_panels_and_transformed_population() {
    check_panels(
        include_str!("../../../fixtures/parity/ggplot2/reverse-position.json"),
        true,
        16,
    );
}
#[test]
fn sqrt_primary_axes_match_r_domain_clipping_and_expansion() {
    check_panels(
        include_str!("../../../fixtures/parity/ggplot2/sqrt-position.json"),
        false,
        20,
    );
}
fn check_panels(input: &str, reverse: bool, count: usize) {
    let fixture: serde_json::Value = serde_json::from_str(input).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), count);
    for case in cases {
        let list = |name: &str| {
            case[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>()
        };
        let x = list("x");
        let mut scale = if reverse {
            scale_reverse()
        } else {
            scale_sqrt()
        };
        if case["explicit"] == true {
            scale = scale.domain(
                x.iter()
                    .copied()
                    .filter(|v| reverse || *v >= 0.)
                    .fold(f64::INFINITY, f64::min),
                x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            );
        }
        let figure = plot(
            Data::columns()
                .column("x", x.clone())
                .column("y", [1., 2., 3.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale)
                .expansion(Some(GgplotExpansion {
                    mult: list("mult").try_into().unwrap(),
                    add: list("add").try_into().unwrap(),
                }))
                .guide_geometry(Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
        let wire = figure.to_json().unwrap();
        assert_eq!(figure.definition().wire_version(), 17);
        assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
        let prepared = figure.chart().unwrap().prepare().unwrap();
        let values: Vec<_> = case["transformed"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_f64())
            .collect();
        assert_eq!(prepared.layers()[0].marks().len(), values.len());
        for (mark, value) in prepared.layers()[0].marks().iter().zip(values) {
            let PreparedGeometry::Point(p) = mark.geometry else {
                panic!()
            };
            near(p.x(), value);
        }
        let request = LayoutRequest::new(
            Rect::new(0., 0., 500., 300.).unwrap(),
            Units::LogicalPixels,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        let frame = layout(prepared, &request, &Metrics).unwrap();
        let axis = &frame.axes()[&ScaleId::new(0)];
        let ResolvedScale::Nonlinear(scale) = &axis.scale else {
            panic!()
        };
        for (v, e) in [
            scale.transformed_viewport().start(),
            scale.transformed_viewport().end(),
        ]
        .into_iter()
        .zip(list("expanded"))
        {
            near(v, e);
        }
        for ((v, expected), transformed) in x
            .iter()
            .zip(case["projected"].as_array().unwrap())
            .zip(case["transformed"].as_array().unwrap())
        {
            let Some(expected) = expected.as_f64() else {
                assert_eq!(scale.map(*v).unwrap(), None);
                continue;
            };
            let transformed = transformed.as_f64().unwrap();
            let pixel = scale.map(*v).unwrap().unwrap();
            near(
                (pixel - scale.range().start()) / (scale.range().end() - scale.range().start()),
                expected,
            );
            near(scale.map_transformed(transformed).unwrap().unwrap(), pixel);
            near(scale.invert(pixel).unwrap(), *v);
        }
        assert_eq!(axis.ticks.len(), list("breaks").len(), "{case}");
        for ((tick, value), label) in axis
            .ticks
            .iter()
            .zip(list("breaks"))
            .zip(case["labels"].as_array().unwrap())
        {
            let chart_core::composition::ScaleValue::Number(v) = tick.value else {
                panic!()
            };
            near(v, value);
            assert_eq!(tick.label, label.as_str().unwrap());
        }
    }
}
#[test]
fn reverse_population_limits_and_numeric_edges_are_checked() {
    use chart_core::grammar::{ScaleOob, ScaleProjection};
    let mut policy = ScaleProjection {
        binned: None,
        id: ScaleId::new(0),
        timestamp: None,
        transform: Some(ScaleTransform::Reverse),
        limits: Some(Bounds::new(2., 8.).unwrap()),
        outside: ScaleOob::Censor,
    };
    policy.validate().unwrap();
    assert_eq!(policy.project(1.), None);
    assert_eq!(policy.project(4.), Some(-4.));
    assert_eq!(policy.project(9.), None);
    policy.outside = ScaleOob::Squish;
    assert_eq!(policy.project(1.), Some(-2.));
    assert_eq!(policy.project(9.), Some(-8.));
    policy.outside = ScaleOob::Keep;
    assert_eq!(policy.project(9.), Some(-9.));
    assert_eq!(policy.project(f64::NAN), None);
    for v in [
        0.,
        -0.,
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        f64::MAX,
        -f64::MAX,
    ] {
        let t = ScaleTransform::Reverse;
        assert_eq!(
            t.inverse(t.forward(v).unwrap().unwrap()).unwrap().to_bits(),
            v.to_bits()
        );
    }
    assert!(ScaleTransform::Reverse.forward(f64::INFINITY).is_err());
    assert!(ScaleTransform::Reverse.inverse(f64::NAN).is_err());
}

#[test]
fn reverse_explicit_histogram_edges_retain_existing_left_closed_policy() {
    use chart_core::grammar::PreparedRows;
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/reverse-position.json"
    ))
    .unwrap();
    let case = fixture["histograms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["closed"] == "left")
        .unwrap();
    let list = |name: &str| {
        case[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect::<Vec<_>>()
    };
    let figure = plot(Data::columns().column("x", list("x")).build().unwrap())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x"))
        .layer(histogram().breaks(list("breaks")))
        .x_axis(x_axis().scale(scale_reverse()))
        .build()
        .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    let PreparedRows::Binned(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    assert_eq!(
        rows.iter().map(|r| r.count as f64).collect::<Vec<_>>(),
        list("count")
    );
    assert_eq!(
        rows.iter().map(|r| r.start).collect::<Vec<_>>(),
        list("start")
    );
    assert_eq!(rows.iter().map(|r| r.end).collect::<Vec<_>>(), list("end"));
}

#[test]
fn explicit_nested_reverse_mappings_require_v17_without_an_axis() {
    use chart_core::grammar::{ChartDefinition, Numeric, ScaleOob, ScaleProjection};
    let reverse = Numeric::Scaled {
        input: Box::new(Numeric::Literal(4.)),
        scale: Box::new(ScaleProjection {
            binned: None,
            id: ScaleId::new(0),
            timestamp: None,
            transform: Some(ScaleTransform::Reverse),
            limits: None,
            outside: ScaleOob::Keep,
        }),
    };
    let mut definition = ChartDefinition::new(Revision::INITIAL);
    definition.mappings.x = Some(Numeric::Scaled {
        input: Box::new(reverse),
        scale: Box::new(ScaleProjection {
            binned: None,
            id: ScaleId::new(1),
            timestamp: None,
            transform: None,
            limits: None,
            outside: ScaleOob::Keep,
        }),
    });
    assert_eq!(definition.wire_version(), 17);
    let mut envelope = chart_core::portable::ChartEnvelope {
        version: 17,
        definition,
    };
    envelope.validate().unwrap();
    envelope.version = 3;
    assert!(envelope.validate().is_err());
}

#[test]
fn sqrt_inverse_domain_and_coordinate_only_eligibility_are_explicit() {
    use chart_core::scales::{ContinuousDomain, NonlinearScale, OutsidePolicy};
    let transform = ScaleTransform::Sqrt;
    assert_eq!(transform.forward(-1.).unwrap(), None);
    assert_eq!(transform.forward(9.).unwrap(), Some(3.));
    assert_eq!(transform.inverse(3.).unwrap(), 9.);
    assert!(transform.inverse(-1.).is_err());
    assert!(transform.inverse(f64::MAX).is_err());
    assert!(transform.inverse(f64::MIN_POSITIVE).is_err());
    assert!(transform.forward(f64::NAN).is_err());
    assert!(
        NonlinearScale::resolve(
            None,
            ContinuousDomain::explicit(Bounds::new(-1., 4.).unwrap()),
            transform,
            Bounds::new(0., 100.).unwrap(),
            None,
            OutsidePolicy::Extend
        )
        .is_err()
    );
    let figure = plot(
        Data::columns()
            .column("x", [-1., 0., 4.])
            .column("y", [1., 2., 3.])
            .build()
            .unwrap(),
    )
    .profile(Profile::Ggplot2_4_0_3)
    .aes(aes().x("x").y("y"))
    .layer(points())
    .x_axis(x_axis().coordinate_scale(scale_sqrt()))
    .build()
    .unwrap();
    let prepared = figure.chart().unwrap().prepare().unwrap();
    assert_eq!(prepared.layers()[0].marks().len(), 3);
    let request = LayoutRequest::new(
        Rect::new(0., 0., 500., 300.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    let frame = layout(prepared, &request, &Metrics).unwrap();
    let ResolvedScale::Nonlinear(scale) = &frame.axes()[&ScaleId::new(0)].scale else {
        panic!()
    };
    assert_eq!(scale.map(-1.).unwrap(), None);
    assert_eq!(
        scale.transformed_viewport(),
        Bounds::new(-0.1, 2.1).unwrap()
    );
    let zero = scale.map(0.).unwrap().unwrap();
    assert_eq!(scale.invert(zero).unwrap(), 0.);
    assert!(scale.invert(zero - 1.).is_err());
}

#[test]
fn sqrt_inverse_at_clamped_nonzero_boundary_preserves_source_value() {
    use chart_core::scales::{ContinuousDomain, NonlinearScale, OutsidePolicy};
    let scale = NonlinearScale::resolve(
        None,
        ContinuousDomain::explicit(Bounds::new(4., 9.).unwrap()),
        ScaleTransform::Sqrt,
        Bounds::new(0., 100.).unwrap(),
        None,
        OutsidePolicy::Clamp,
    )
    .unwrap();
    assert_eq!(scale.map(0.).unwrap(), Some(0.));
    assert_eq!(scale.invert(0.).unwrap(), 4.);
}
