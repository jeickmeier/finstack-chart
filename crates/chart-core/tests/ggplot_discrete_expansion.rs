//! FIX-GG04: reference discrete expansion and real categorical axis projection.
use chart_core::{
    Rect, ResourceId, Revision, ScaleId, layout::*, prelude::*, scales::*, services::*,
};
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/discrete_expansion.json"
    ))
    .unwrap()
}
fn pair(value: &serde_json::Value) -> [f64; 2] {
    [value[0].as_f64().unwrap(), value[1].as_f64().unwrap()]
}
fn expansion(case: &serde_json::Value) -> GgplotExpansion {
    GgplotExpansion {
        mult: pair(&case["mult"]),
        add: pair(&case["add"]),
    }
}
#[test]
fn discrete_limits_match_reference_with_continuous_extents() {
    let f = fixture();
    let cases = f["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 36);
    for case in cases {
        let continuous = case["continuous"]
            .as_array()
            .filter(|v| !v.is_empty())
            .map(|_| {
                let [a, b] = pair(&case["continuous"]);
                Bounds::new(a, b).unwrap()
            });
        let view = expansion(case)
            .discrete_viewport(case["count"].as_u64().unwrap() as usize, continuous)
            .unwrap();
        for (got, expected) in [view.start(), view.end()]
            .into_iter()
            .zip(pair(&case["expanded"]))
        {
            assert!(
                (got - expected).abs() < 2e-14,
                "{case}: {got} != {expected}"
            );
        }
    }
}
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
#[test]
fn categorical_axes_match_reference_projection_and_keep_lookup() {
    let f = fixture();
    let panels = f["panels"].as_array().unwrap();
    assert_eq!(panels.len(), 9);
    for case in panels {
        let labels: Vec<_> = case["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let data = Data::columns()
            .column("x", categorical(labels.clone()))
            .column("y", vec![1.; labels.len()])
            .build()
            .unwrap();
        let plot = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .build()
            .unwrap();
        let prepared = plot.chart().unwrap().prepare().unwrap();
        for family in [
            AxisScale::Auto,
            AxisScale::Band(Default::default()),
            AxisScale::Point(Default::default()),
        ] {
            for reversed in [false, true] {
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
                request.axes[0].scale = family.clone();
                // Guide presentation is explicit: legacy automatic thinning can omit
                // endpoint labels with zero expansion; its profile defaults belong to GG-05.
                request.axes[0].geometry = Some(GuideGeometry {
                    labels: Some(GuideLabelPolicy::Preserve),
                    ..Default::default()
                });
                let policy = expansion(case);
                // Exercise default profile selection as well as explicit asymmetric/zero policies.
                if policy != GgplotExpansion::discrete_default() {
                    request.axes[0].expansion = Some(policy);
                }
                request.axes[0].range = Some(if reversed {
                    Bounds::new(400., 0.).unwrap()
                } else {
                    Bounds::new(0., 400.).unwrap()
                });
                let frame = layout(prepared.clone(), &request, &Metrics).unwrap();
                let axis = &frame.axes()[&ScaleId::new(0)];
                for (i, label) in labels.iter().enumerate() {
                    let expected = case["projected"][i].as_f64().unwrap();
                    let expected = if reversed { 1. - expected } else { expected } * 400.;
                    let position = match &axis.scale {
                        ResolvedScale::Band(s) => {
                            let p = s.center(label).unwrap().unwrap();
                            assert_eq!(s.category_at(p).unwrap(), Some(*label));
                            assert!(s.bandwidth().is_finite());
                            p
                        }
                        ResolvedScale::Point(s) => {
                            let p = s.center(label).unwrap().unwrap();
                            assert_eq!(s.category_at(p).unwrap(), Some(*label));
                            p
                        }
                        _ => panic!(),
                    };
                    assert!(
                        (position - expected).abs() < 2e-12,
                        "{case}: {position} != {expected}"
                    );
                    let tick = axis
                        .ticks
                        .iter()
                        .find(|tick| tick.label == *label)
                        .unwrap_or_else(|| {
                            panic!("missing {label}; case {case}; ticks {:?}", axis.ticks)
                        });
                    assert!((tick.position - expected).abs() < 2e-12);
                }
            }
        }
    }
}
