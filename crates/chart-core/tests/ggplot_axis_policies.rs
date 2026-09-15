//! FIX-GG05: shared multi-aesthetic key composition, placement and immutable controls.
use chart_core::{
    Rect, ResourceId, Revision,
    layout::{LayoutRequest, layout},
    prelude::*,
    scene::{GuideRole, Primitive},
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn shape(
        &self,
        r: chart_core::typography::ShapeRequest<'_>,
    ) -> ChartResult<chart_core::typography::ShapedRun> {
        Ok(chart_core::typography::ShapedRun {
            text: r.run.text.clone(),
            font: *r.default_font,
            font_size: r.font_size,
            language: r.run.language.clone(),
            direction: r.run.direction,
            tabular: r.run.tabular,
            metrics: TextMetrics::new(r.run.text.len() as f64 * 5., 8., 2.)?,
            glyphs: vec![],
            outlines: vec![],
            used_fallback: false,
        })
    }
    fn measure(&self, r: TextRequest) -> chart_core::ChartResult<TextMetrics> {
        TextMetrics::new(r.text.chars().count() as f64 * 6., 8., 2.)
    }
}
fn request() -> LayoutRequest {
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 600., 360.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for a in &mut r.axes {
        a.visible = false;
    }
    r
}

#[test]
fn dodge_overlap_caps_and_minor_ticks_preserve_selection() {
    use chart_core::{
        composition::ScaleValue,
        interpolate::Number,
        layout::{AxisCap, GgplotAxisOptions, MinorBreaks},
    };
    for dodge in [1, 2, 3] {
        for overlap in [false, true] {
            let data = Data::columns().column("x", vec![0., 10.]).build().unwrap();
            let p = plot(data)
                .profile(chart_core::grammar::Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(points())
                .axis(
                    x_axis()
                        .scale(scale_linear().domain(0., 10.))
                        .ticks((1..10).map(|i| {
                            (
                                ScaleValue::Number(i as f64),
                                format!("long label number {i}"),
                            )
                        }))
                        .minor_breaks(Some(MinorBreaks::Numeric(vec![Number(1.5), Number(4.5)])))
                        .ggplot_axis(Some(GgplotAxisOptions {
                            n_dodge: dodge,
                            check_overlap: overlap,
                            cap: AxisCap::Both,
                            minor_ticks: true,
                            ..Default::default()
                        })),
                )
                .axis(y_axis().visible(false))
                .build()
                .unwrap();
            let wire = p.to_json().unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&wire).unwrap()["version"],
                70
            );
            let restored = Plot::from_json(&wire).unwrap();
            let frame = layout(
                restored.chart().unwrap().prepare().unwrap(),
                &request(),
                &Metrics,
            )
            .unwrap();
            let items = frame.scene().items();
            let majors: Vec<_> = items
                .iter()
                .filter(|i| {
                    i.guide
                        .as_ref()
                        .is_some_and(|g| g.role == GuideRole::Line && g.scope.is_empty())
                })
                .collect();
            let minors: Vec<_> = items
                .iter()
                .filter(|i| {
                    i.guide
                        .as_ref()
                        .is_some_and(|g| g.role == GuideRole::Line && g.scope == ["minor"])
                })
                .collect();
            assert_eq!(majors.len(), 9);
            assert_eq!(minors.len(), 2);
            let labels: Vec<_> = items
                .iter()
                .filter(|i| i.guide.as_ref().is_some_and(|g| g.role == GuideRole::Label))
                .collect();
            if !overlap {
                assert_eq!(labels.len(), 9);
            } else {
                assert!(labels.len() <= 9);
                assert!(
                    labels
                        .iter()
                        .any(|i| i.guide.as_ref().unwrap().index == Some(0))
                );
                assert!(
                    labels
                        .iter()
                        .any(|i| i.guide.as_ref().unwrap().index == Some(8))
                );
            }
            let mut rows: Vec<_> = labels
                .iter()
                .filter_map(|i| match &i.primitive {
                    Primitive::Text { origin, .. } => Some(origin.y()),
                    _ => None,
                })
                .collect();
            rows.sort_by(f64::total_cmp);
            rows.dedup();
            assert_eq!(rows.len(), dodge);
            let domain = items
                .iter()
                .find(|i| {
                    i.guide
                        .as_ref()
                        .is_some_and(|g| g.role == GuideRole::Domain)
                })
                .unwrap();
            let Primitive::Rule { from, to, .. } = domain.primitive else {
                panic!("domain rule")
            };
            let Primitive::Rule { from: first, .. } = majors[0].primitive else {
                panic!("tick")
            };
            let Primitive::Rule { from: last, .. } = majors[8].primitive else {
                panic!("tick")
            };
            assert_eq!(from.x(), first.x());
            assert_eq!(to.x(), last.x());
        }
    }
}

#[test]
fn logarithmic_ticks_and_stacks_share_the_primary_scale() {
    use chart_core::layout::{AxisSide, GgplotAxisOptions, LogTickOptions};
    let data = Data::columns().column("x", vec![1., 100.]).build().unwrap();
    let p = plot(data)
        .profile(chart_core::grammar::Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(1.))
        .layer(points())
        .axis(
            x_axis()
                .scale(scale_log(10.).domain(1., 100.))
                .label("near")
                .ggplot_axis(Some(GgplotAxisOptions {
                    stack_order: Some(0),
                    stack_spacing: 7.,
                    ..Default::default()
                })),
        )
        .guide(
            axis_guide("outer", "x")
                .side(AxisSide::Bottom)
                .label("far")
                .ggplot_axis(Some(GgplotAxisOptions {
                    stack_order: Some(1),
                    logticks: Some(LogTickOptions {
                        expanded: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                })),
        )
        .build()
        .unwrap();
    let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let log: Vec<_> = frame
        .scene()
        .items()
        .iter()
        .filter(|i| i.guide.as_ref().is_some_and(|g| g.scope == ["logtick"]))
        .collect();
    assert_eq!(log.len(), 19);
    let plot = frame.plot().unwrap();
    for item in log {
        let Primitive::Rule { from, to, .. } = item.primitive else {
            panic!("rule")
        };
        assert!(from.y() > plot.max_y());
        assert!(to.y() > from.y());
        assert!(from.x() >= plot.origin().x() && from.x() <= plot.max_x());
    }
    let guides = frame.guides();
    let near = guides
        .values()
        .find(|g| {
            g.spec
                .ggplot_axis
                .as_ref()
                .is_some_and(|o| o.stack_order == Some(0))
        })
        .unwrap();
    let far = guides
        .values()
        .find(|g| {
            g.spec
                .ggplot_axis
                .as_ref()
                .is_some_and(|o| o.stack_order == Some(1))
        })
        .unwrap();
    assert!(far.spec.translation[1] > near.spec.translation[1]);
}

#[test]
fn signed_and_prescaled_log_ticks_preserve_source_coordinates() {
    use chart_core::{
        composition::ScaleValue,
        layout::{GgplotAxisOptions, LogTickOptions},
    };
    for (bounds, base, expected) in [
        ([-10., 10.], None, vec![-10., -5., -1., 0., 1., 5., 10.]),
        ([0., 2.], Some(10.), vec![0., 1., 2.]),
    ] {
        let data = Data::columns()
            .column("x", bounds.to_vec())
            .build()
            .unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(1.))
            .layer(points())
            .axis(
                x_axis()
                    .scale(scale_linear().domain(bounds[0], bounds[1]))
                    .ggplot_axis(Some(GgplotAxisOptions {
                        logticks: Some(LogTickOptions {
                            prescale_base: base,
                            negative_small: Some(1.),
                            expanded: false,
                            ..Default::default()
                        }),
                        ..Default::default()
                    })),
            )
            .axis(y_axis().visible(false))
            .build()
            .unwrap();
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
        let values: Vec<_> = frame
            .scene()
            .items()
            .iter()
            .filter_map(|i| i.guide.as_ref())
            .filter(|g| g.scope == ["logtick"])
            .map(|g| match g.tick.as_ref().unwrap().value {
                ScaleValue::Number(v) => v,
                _ => panic!("numeric identity"),
            })
            .collect();
        for value in expected {
            assert!(
                values.iter().any(|v| (v - value).abs() < 1e-12),
                "{value}: {values:?}"
            );
        }
        assert!(values.iter().all(|v| *v >= bounds[0] && *v <= bounds[1]));
    }
}
