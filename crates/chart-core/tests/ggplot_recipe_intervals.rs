//! FIX-GG07 shared interval emissions, scale stages, provenance, reference lines and arrows.
use chart_core::{
    Point, Rect, ResourceId, Revision,
    grammar::*,
    layout::{LayoutRequest, layout},
    prelude::*,
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn request() -> LayoutRequest {
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 480., 320.).unwrap(),
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
fn intervals_orientation_and_source_contract() {
    let d = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [2., -1., 0.])
        .column("lo", [1., -2., 0.])
        .column("hi", [3., 0., 0.])
        .build()
        .unwrap();
    for horizontal in [false, true] {
        for (kind, n) in [
            (IntervalKind::LineRange, 1),
            (IntervalKind::PointRange, 2),
            (IntervalKind::ErrorBar, 3),
            (IntervalKind::Crossbar, 2),
        ] {
            let l = rule()
                .recipe(BuiltinRecipe::Interval(IntervalRecipe {
                    kind,
                    width: Some(0.4),
                    ..Default::default()
                }))
                .recipe_value(RecipeAesthetic::Lower, "lo")
                .recipe_value(RecipeAesthetic::Upper, "hi")
                .orientation(if horizontal {
                    Orientation::Horizontal
                } else {
                    Orientation::Vertical
                });
            let p = plot(d.clone())
                .profile(Profile::Ggplot2_4_0_3)
                .aes(if horizontal {
                    aes().x("y").y("x")
                } else {
                    aes().x("x").y("y")
                })
                .layer(l)
                .build()
                .unwrap();
            let q = Plot::from_json(&p.to_json().unwrap()).unwrap();
            let prepared = q.chart().unwrap().prepare().unwrap();
            let marks = prepared.layers()[0].marks();
            assert_eq!(marks.len(), 3 * n);
            for row in 0..3 {
                let a = &marks[row * n];
                for b in &marks[row * n..(row + 1) * n] {
                    assert_eq!(a.targets, b.targets);
                }
            }
            let first = &marks[0].geometry;
            let (from, to) = match first {
                PreparedGeometry::Rule { from, to } | PreparedGeometry::Rectangle { from, to } => {
                    (*from, *to)
                }
                _ => panic!("interval"),
            };
            let expected = if kind == IntervalKind::Crossbar {
                [Point::new(0.8, 1.).unwrap(), Point::new(1.2, 3.).unwrap()]
            } else {
                [Point::new(1., 1.).unwrap(), Point::new(1., 3.).unwrap()]
            };
            for (a, b) in [from, to].into_iter().zip(expected) {
                assert_eq!(
                    a,
                    if horizontal {
                        Point::new(b.y(), b.x()).unwrap()
                    } else {
                        b
                    }
                );
            }
        }
    }
}
#[test]
fn logarithmic_interval_bounds_share_y_projection() {
    let d = Data::columns()
        .column("x", [1.])
        .column("y", [10.])
        .column("lo", [1.])
        .column("hi", [100.])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            linerange()
                .recipe_value(RecipeAesthetic::Lower, "lo")
                .recipe_value(RecipeAesthetic::Upper, "hi"),
        )
        .y_axis(y_axis().scale(scale_log(10.)))
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedGeometry::Rule { from, to } = prepared.layers()[0].marks()[0].geometry else {
        panic!("rule")
    };
    assert_eq!((from.y(), to.y()), (0., 2.));
}
#[test]
fn reference_equations_do_not_train_domains_and_arrows_keep_targets() {
    let d = Data::columns()
        .column("x", [1., 4.])
        .column("y", [2., 5.])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .layer(abline(0.5, 1.))
        .layer(
            segment()
                .aes(aes().x(1.).y(2.).x2(3.).y2(4.))
                .recipe(BuiltinRecipe::Segment {
                    arrow: Some(ArrowSpec {
                        length_mm: 3.,
                        ends: ArrowEnds::Both,
                        ..Default::default()
                    }),
                }),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    assert!(prepared.layers()[1].domains().x.is_none());
    assert!(prepared.layers()[1].domains().y.is_none());
    let frame = layout(prepared, &request(), &Metrics).unwrap();
    assert!(
        frame
            .scene()
            .items()
            .iter()
            .filter(|i| matches!(i.primitive, Primitive::Path { .. }))
            .count()
            >= 2
    );
}
#[test]
fn statistical_bounds_and_blank_training_remain_shared() {
    let d = Data::columns()
        .column("x", [1., 1., 2., 2.])
        .column("y", [1., 3., 10., 30.])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(
            pointrange()
                .stat(
                    summary()
                        .x("x")
                        .y("y")
                        .summary_helper(SummaryHelper::default()),
                )
                .recipe_stat_value(RecipeAesthetic::Lower, StatField::Lower)
                .recipe_stat_value(RecipeAesthetic::Upper, StatField::Upper),
        )
        .layer(blank().aes(aes().x(20.)))
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    assert_eq!(prepared.layers()[0].marks().len(), 4);
    assert!(prepared.layers()[1].marks().is_empty());
    assert_eq!(prepared.layers()[1].domains().x.unwrap().maximum, 20.);
}
#[test]
fn horizontal_literal_interval_bounds_share_transform() {
    let d = Data::columns()
        .column("x", [10.])
        .column("y", [1.])
        .build()
        .unwrap();
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            linerange()
                .orientation(Orientation::Horizontal)
                .recipe_value(RecipeAesthetic::Lower, 1.)
                .recipe_value(RecipeAesthetic::Upper, 100.),
        )
        .x_axis(x_axis().scale(scale_log(10.)))
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedGeometry::Rule { from, to } = prepared.layers()[0].marks()[0].geometry else {
        panic!("rule")
    };
    assert_eq!((from.x(), to.x()), (0., 2.));
}
#[test]
fn crossbar_default_is_unfilled_and_explicit_fill_is_preserved() {
    for profile in [Profile::LibraryV1, Profile::Ggplot2_4_0_3] {
        for filled in [false, true] {
            let d = Data::columns()
                .column("x", [1.])
                .column("y", [2.])
                .build()
                .unwrap();
            let mut l = crossbar()
                .recipe_value(RecipeAesthetic::Lower, 1.)
                .recipe_value(RecipeAesthetic::Upper, 3.);
            if filled {
                l = l.fill(rgb(255, 0, 0));
            }
            let p = plot(d)
                .profile(profile)
                .aes(aes().x("x").y("y"))
                .layer(l)
                .build()
                .unwrap();
            let frame =
                layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
            let (fill, stroke) = frame
                .scene()
                .items()
                .iter()
                .find_map(|i| match &i.primitive {
                    Primitive::ShapePath { fill, stroke, .. } => Some((fill, stroke)),
                    _ => None,
                })
                .unwrap();
            assert_eq!(fill.is_some(), filled);
            assert!(stroke.is_some());
        }
    }
}
#[test]
fn source_interval_component_controls_and_alpha() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/interval-paint-controls.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 10);
    let d = Data::columns()
        .column("x", [1.])
        .column("y", [2.])
        .build()
        .unwrap();
    let spec = IntervalRecipe {
        kind: IntervalKind::Crossbar,
        middle: IntervalStroke {
            color: Some(rgb(255, 0, 0).into()),
            linewidth: Some(2.),
            line_type: Some(LineType::Dashed),
        },
        box_style: IntervalStroke {
            color: Some(rgb(0, 0, 255).into()),
            linewidth: Some(0.25),
            line_type: Some(LineType::Dotted),
        },
        ..Default::default()
    };
    let p = plot(d.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            crossbar()
                .recipe(BuiltinRecipe::Interval(spec))
                .recipe_value(RecipeAesthetic::Lower, 1.)
                .recipe_value(RecipeAesthetic::Upper, 3.)
                .fill(rgb(255, 215, 0))
                .alpha(0.2),
        )
        .build()
        .unwrap();
    let f = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let mut widths = vec![];
    for item in f.scene().items().iter().filter(|i| i.layer.is_some()) {
        match &item.primitive {
            Primitive::ShapePath {
                stroke: Some(stroke),
                fill,
                ..
            } => {
                widths.push(stroke.width);
                assert_eq!(stroke.color.alpha, 255);
                if let Some(fill) = fill {
                    assert_eq!(fill.alpha, 51);
                }
            }
            Primitive::DashedPath { stroke, .. } => {
                widths.push(stroke.width);
                assert_eq!(stroke.color.alpha, 255);
            }
            _ => {}
        }
    }
    assert_eq!(widths.len(), 2);
    assert!((widths[0] - 0.7113188976377953 * 0.75).abs() < 1e-12);
    assert!((widths[1] - 5.690551181102363 * 0.75).abs() < 1e-12);
    let point = IntervalPoint {
        size: Some(1.),
        stroke: Some(2.),
        shape: Some(21),
        fill: Some(rgb(255, 215, 0).into()),
    };
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            pointrange()
                .recipe(BuiltinRecipe::Interval(IntervalRecipe {
                    kind: IntervalKind::PointRange,
                    point,
                    ..Default::default()
                }))
                .recipe_value(RecipeAesthetic::Lower, 1.)
                .recipe_value(RecipeAesthetic::Upper, 3.)
                .linewidth(0.25),
        )
        .build()
        .unwrap();
    let f = layout(p.chart().unwrap().prepare().unwrap(), &request(), &Metrics).unwrap();
    let point = f
        .scene()
        .items()
        .iter()
        .find_map(|i| match &i.primitive {
            Primitive::ShapePath {
                geometry,
                fill: Some(fill),
                stroke: Some(stroke),
                ..
            } if fill.red == 255 && fill.green == 215 => Some((geometry, stroke)),
            _ => None,
        })
        .unwrap();
    assert!((point.1.width - 3.7795275590551185 * 0.75).abs() < 1e-12);
    let bounds = point.0.bounds(0.01, 10000).unwrap().unwrap();
    // Path bounds conservatively include the 0.01 flattening tolerance on each side.
    assert!((bounds.width() - 15.160629921259844 * 0.75).abs() <= 0.020001);
}
#[test]
fn reference_constructor_dedup_and_reversed_interval_source_contract() {
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/reference-line-controls.json"
    ))
    .unwrap();
    for c in source["cases"].as_array().unwrap().iter().take(3) {
        assert_eq!(c["data"][0].as_array().unwrap().len(), 1);
    }
    let d = Data::columns()
        .column("x", [1., 2., 3.])
        .column("y", [1., 2., 3.])
        .column("v", [1., 1., 2.])
        .build()
        .unwrap();
    for l in [hline(0.5), vline(0.5), abline(1., 0.5)] {
        let p = plot(d.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(l.alpha(0.5))
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        assert_eq!(marks.len(), 1);
        assert_eq!(marks[0].targets.len(), 3);
        let f = layout(prepared, &request(), &Metrics).unwrap();
        assert_eq!(
            f.scene()
                .items()
                .iter()
                .filter(|i| i.layer.is_some())
                .count(),
            1
        );
    }
    let p = plot(d.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .layer(hline(0.).recipe_value(RecipeAesthetic::Intercept, "v"))
        .build()
        .unwrap();
    let q = p.chart().unwrap().prepare().unwrap();
    assert_eq!(q.layers()[0].marks().len(), 2);
    for (kind, n) in [
        (IntervalKind::LineRange, 1),
        (IntervalKind::PointRange, 2),
        (IntervalKind::ErrorBar, 3),
        (IntervalKind::Crossbar, 2),
    ] {
        let p = plot(d.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(
                rule()
                    .recipe(BuiltinRecipe::Interval(IntervalRecipe {
                        kind,
                        ..Default::default()
                    }))
                    .recipe_value(RecipeAesthetic::Lower, 2.)
                    .recipe_value(RecipeAesthetic::Upper, 0.),
            )
            .build()
            .unwrap();
        let q = p.chart().unwrap().prepare().unwrap();
        assert_eq!(q.layers()[0].marks().len(), 3 * n);
        let f = layout(q, &request(), &Metrics).unwrap();
        assert!(f.scene().items().iter().any(|i| i.layer.is_some()));
    }
}
#[test]
fn crossbar_keeps_embedded_colour_alpha_when_row_alpha_is_reset() {
    let d = Data::columns()
        .column("x", [1.])
        .column("y", [1.])
        .build()
        .unwrap();
    let paint = chart_core::scene::Color {
        red: 255,
        green: 0,
        blue: 0,
        alpha: 128,
    };
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            crossbar()
                .recipe_value(RecipeAesthetic::Lower, 0.)
                .recipe_value(RecipeAesthetic::Upper, 2.)
                .color(paint)
                .fill(paint)
                .alpha(0.2),
        )
        .build()
        .unwrap();
    let q = p.chart().unwrap().prepare().unwrap();
    assert_eq!(q.layers()[0].marks()[0].style.color.alpha, 128);
    assert_eq!(q.layers()[0].marks()[0].style.fill.unwrap().alpha, 51);
    assert_eq!(q.layers()[0].marks()[1].style.color.alpha, 128);
}
#[test]
fn mapped_missing_recipe_controls_omit_ink_and_retain_training() {
    let d = Data::columns()
        .column("x", [1., 100.])
        .column("y", [1., 20.])
        .column("w", [Some(0.5), None])
        .column("intercept", [Some(1.), None])
        .build()
        .unwrap();
    let p = plot(d.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            errorbar()
                .recipe_value(RecipeAesthetic::Width, "w")
                .recipe_value(RecipeAesthetic::Lower, 0.)
                .recipe_value(RecipeAesthetic::Upper, 2.),
        )
        .build()
        .unwrap();
    let q = p.chart().unwrap().prepare().unwrap();
    assert_eq!(q.layers()[0].marks().len(), 3);
    assert_eq!(q.layers()[0].domains().x.unwrap().maximum, 100.);
    assert_eq!(q.layers()[0].domains().y.unwrap().maximum, 20.);
    let p = plot(d.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .layer(hline(5.).recipe_value(RecipeAesthetic::Intercept, "intercept"))
        .build()
        .unwrap();
    let q = p.chart().unwrap().prepare().unwrap();
    assert_eq!(q.layers()[0].marks().len(), 1);
    assert_eq!(q.layers()[0].marks()[0].targets.len(), 1);
    let p = plot(d)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(hline(5.))
        .build()
        .unwrap();
    let q = p.chart().unwrap().prepare().unwrap();
    assert_eq!(q.layers()[0].marks().len(), 1);
    assert_eq!(q.layers()[0].marks()[0].targets.len(), 2);
}
