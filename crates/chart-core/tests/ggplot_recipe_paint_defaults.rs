//! FIX-GG07: default paints and device units from pinned ggplot2 4.0.3 build/draw output.
#[path = "../../../examples/common/ggplot_interval_recipes.rs"]
mod intervals;
#[path = "../../../examples/common/ggplot_recipe_marks.rs"]
mod marks;
use chart_core::{
    ChartResult, Rect, ResourceId, Revision,
    layout::{LaidOutChart, LayoutRequest, layout},
    prelude::*,
    scene::{Color, Primitive},
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(p: Plot, units: Units) -> LaidOutChart {
    let prepared = p.chart().unwrap().prepare().unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 600., 360.).unwrap(),
        units,
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
    layout(prepared, &request, &Metrics).unwrap()
}
fn widths(f: &LaidOutChart) -> Vec<f64> {
    f.scene()
        .items()
        .iter()
        .filter(|i| i.layer.is_some())
        .filter_map(|i| match &i.primitive {
            Primitive::Rule { stroke, .. }
            | Primitive::Path { stroke, .. }
            | Primitive::DashedPath { stroke, .. } => Some(stroke.width),
            Primitive::ShapePath {
                stroke: Some(s), ..
            }
            | Primitive::VectorPath {
                stroke: Some(s), ..
            } => Some(s.width),
            _ => None,
        })
        .collect()
}
#[test]
fn default_line_recipes_share_reference_line_unit_in_both_destinations() {
    // Each named fixture has colour=black, linewidth=0.5, linetype=1, alpha=NULL.
    // gg_par converts to .pt=72.27/25.4, then devices use 1/96-inch line units.
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/primitive-recipes.json"
    ))
    .unwrap();
    for units in [Units::LogicalPixels, Units::Points] {
        let device = if units == Units::Points {
            72. / 96.
        } else {
            1.
        };
        for (scope, mode, name) in [
            (0, 0, "linerange-vertical"),
            (0, 2, "errorbar-vertical"),
            (0, 8, "step-hv-FALSE"),
            (0, 11, "segment-both-closed"),
            (0, 12, "reference-lines-linear"),
            (0, 13, "reference-lines-linear"),
            (0, 14, "reference-lines-linear"),
            (1, 2, "rug-bltr"),
            (1, 3, "curve-0.4"),
            (1, 4, "spokes-signed"),
        ] {
            let case = reference["cases"]
                .as_array()
                .unwrap()
                .iter()
                .find(|c| c["name"] == name)
                .unwrap();
            let linewidth = case["data"]
                .as_array()
                .unwrap()
                .iter()
                .find_map(|rows| rows[0]["linewidth"].as_f64())
                .unwrap();
            let f = frame(
                if scope == 0 {
                    intervals::author(mode)
                } else {
                    marks::author(mode)
                }
                .unwrap(),
                units,
            );
            let widths = widths(&f);
            assert!(!widths.is_empty(), "{name}");
            for width in widths {
                assert!(
                    (width - linewidth * 72.27 / 25.4 * device).abs() < 1e-12,
                    "{name} mode{mode} {units:?}: actual{width}"
                );
            }
            for item in f.scene().items().iter().filter(|i| i.layer.is_some()) {
                let color = match &item.primitive {
                    Primitive::Rule { stroke, .. }
                    | Primitive::Path { stroke, .. }
                    | Primitive::DashedPath { stroke, .. } => Some(stroke.color),
                    Primitive::ShapePath {
                        stroke: Some(s), ..
                    }
                    | Primitive::VectorPath {
                        stroke: Some(s), ..
                    } => Some(s.color),
                    _ => None,
                };
                if let Some(c) = color {
                    assert_eq!(
                        c,
                        Color {
                            red: 0,
                            green: 0,
                            blue: 0,
                            alpha: 255
                        },
                        "{name}"
                    );
                }
            }
        }
    }
}
#[test]
fn crossbar_default_is_unfilled_and_pointrange_point_has_reference_fattened_size() {
    for units in [Units::LogicalPixels, Units::Points] {
        let f = frame(intervals::author(3).unwrap(), units);
        let outlines = f
            .scene()
            .items()
            .iter()
            .filter(|i| i.layer.is_some())
            .filter_map(|i| {
                if let Primitive::ShapePath { fill, stroke, .. } = &i.primitive {
                    Some((fill, stroke))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(outlines.len(), 3);
        for (fill, stroke) in outlines {
            assert!(fill.is_none_or(|c| c.alpha == 0));
            assert!(stroke.is_some());
        }
        let center_widths = f
            .scene()
            .items()
            .iter()
            .filter(|i| i.layer.is_some())
            .filter_map(|i| {
                if let Primitive::Rule { stroke, .. } = &i.primitive {
                    Some(stroke.width)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(center_widths.len(), 3);
        for width in center_widths {
            let expected = 3.556594488188977
                * if units == Units::Points {
                    72. / 96.
                } else {
                    1.
                };
            assert!((width - expected).abs() < 1e-12);
        }
        let f = frame(intervals::author(1).unwrap(), units);
        let radii = f
            .scene()
            .items()
            .iter()
            .filter_map(|i| {
                if i.layer.is_some() {
                    if let Primitive::Point { radius, .. } = i.primitive {
                        Some(radius)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(radii.len(), 3);
        // Pinned draw: fontsize=7.580314960629922 points, R glyph radius=3/8 fontsize.
        let expected = 7.580314960629922
            * 0.375
            * if units == Units::Points {
                1.
            } else {
                96. / 72.
            };
        for radius in radii {
            assert!(
                (radius - expected).abs() < 1e-12,
                "pointrange radius{radius},expected{expected}"
            );
        }
    }
}
#[test]
fn column_reference_default_fill_is_grey_and_count_max_size_matches_reference() {
    let f = frame(marks::author(1).unwrap(), Units::LogicalPixels);
    let fills = f
        .scene()
        .items()
        .iter()
        .filter(|i| i.layer.is_some())
        .filter_map(|i| {
            if let Primitive::Rectangle { fill, .. } = i.primitive {
                Some(fill)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert!(!fills.is_empty());
    for fill in fills {
        assert_eq!(
            fill,
            Color {
                red: 89,
                green: 89,
                blue: 89,
                alpha: 255
            }
        );
    }
    // Weighted count uses the ordinary size scale (maximum6mm), shape19, stroke0.5mm.
    let f = frame(marks::author(0).unwrap(), Units::LogicalPixels);
    let max = f
        .scene()
        .items()
        .iter()
        .filter(|i| i.layer.is_some())
        .filter_map(|i| match &i.primitive {
            Primitive::Point { radius, .. } => Some(*radius),
            Primitive::ShapePath { geometry, .. } => geometry
                .bounds(0.01, 10000)
                .unwrap()
                .map(|b| b.width() / 2.),
            _ => None,
        })
        .reduce(f64::max)
        .unwrap();
    let expected = (6. * 0.37640625 + 0.5 * 0.25) * 96. / 25.4;
    assert!(
        (max - expected).abs() < 1e-10,
        "count radius{max},expected{expected}"
    );
}
#[test]
fn recipe_kind_preserves_physical_defaults_and_explicit_units_independent_of_base_geom() {
    use chart_core::grammar::{
        AestheticUnits, BuiltinRecipe, CountRecipe, ReferenceKind, ReferenceRecipe,
    };
    for units in [Units::LogicalPixels, Units::Points] {
        for base in [points(), rule()] {
            let data = Data::columns()
                .column("x", [0., 1.])
                .column("y", [0., 1.])
                .build()
                .unwrap();
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y"))
                .layer(
                    base.recipe(BuiltinRecipe::Reference(ReferenceRecipe {
                        kind: ReferenceKind::Horizontal,
                        slope: 0.,
                        intercept: 1.,
                        arrow: None,
                    }))
                    .linewidth(2.)
                    .aesthetic_units(AestheticUnits::Points),
                )
                .x_axis(x_axis().scale(scale_linear().domain(0., 2.)))
                .y_axis(y_axis().scale(scale_linear().domain(0., 2.)))
                .build()
                .unwrap();
            let f = frame(p, units);
            for width in widths(&f) {
                assert!(
                    (width
                        - 2. * if units == Units::Points {
                            1.
                        } else {
                            96. / 72.
                        })
                    .abs()
                        < 1e-12
                );
            }
        }
    }
    let mut all = vec![];
    for base in [points(), rule()] {
        let data = Data::columns()
            .column("x", [1., 1., 2., 3.])
            .column("y", [1., 1., -1., 2.])
            .build()
            .unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(
                base.recipe(BuiltinRecipe::Count(CountRecipe::default()))
                    .stat(count().sum_count().x("x").y("y")),
            )
            .build()
            .unwrap();
        let f = frame(p, Units::LogicalPixels);
        let radii = f
            .scene()
            .items()
            .iter()
            .filter(|i| i.layer.is_some())
            .filter_map(|i| match &i.primitive {
                Primitive::Point { radius, .. } => Some(*radius),
                Primitive::ShapePath { geometry, .. } => geometry
                    .bounds(0.01, 10000)
                    .unwrap()
                    .map(|b| b.width() / 2.),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(!radii.is_empty());
        all.push(radii);
    }
    assert_eq!(all[0], all[1]);
}

#[test]
fn portable_round_caps_use_authored_point_units_in_both_destinations() {
    use chart_core::grammar::AestheticUnits;
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/stroke-controls.json"
    ))
    .unwrap();
    assert_eq!(source["grid_defaults"]["linemitre"], 10);
    assert_eq!(source["cases"].as_array().unwrap().len(), 9);
    for units in [Units::LogicalPixels, Units::Points] {
        let p = plot(
            Data::columns()
                .column("x", [1., 2.])
                .column("y", [1., 1.])
                .build()
                .unwrap(),
        )
        .aes(aes().x("x").y("y"))
        .layer(
            line()
                .linewidth(12.)
                .aesthetic_units(AestheticUnits::Points)
                .lineend(LineEnd::Round),
        )
        .x_axis(x_axis().scale(scale_linear().domain(0., 3.)))
        .y_axis(y_axis().scale(scale_linear().domain(0., 2.)))
        .build()
        .unwrap();
        let f = frame(Plot::from_json(&p.to_json().unwrap()).unwrap(), units);
        let (geometry, anchors) = f
            .scene()
            .items()
            .iter()
            .find_map(|i| match &i.primitive {
                Primitive::ShapePath {
                    geometry,
                    anchors,
                    stroke: None,
                    ..
                } if i.layer.is_some() => Some((geometry, anchors)),
                _ => None,
            })
            .unwrap();
        let points = geometry
            .flatten(0.00001, 10000)
            .unwrap()
            .subpaths
            .into_iter()
            .flat_map(|s| s.points)
            .collect::<Vec<_>>();
        let xmin = points.iter().map(|p| p.x()).reduce(f64::min).unwrap();
        let ymin = points.iter().map(|p| p.y()).reduce(f64::min).unwrap();
        let ymax = points.iter().map(|p| p.y()).reduce(f64::max).unwrap();
        let radius = if units == Units::Points { 6. } else { 8. };
        // Portable round geometry has the documented <=0.01 destination error.
        assert!((anchors[0].x() - xmin - radius).abs() < 0.01);
        assert!((ymax - ymin - radius * 2.).abs() < 0.02);
        let index = f
            .scene()
            .items()
            .iter()
            .position(|i| i.layer.is_some())
            .unwrap();
        let inspector = chart_core::inspection::Inspector::new(f.clone().into(), 0.01, 32).unwrap();
        let hits = inspector
            .query(
                chart_core::Point::new(anchors[0].x() - radius * 0.5, anchors[0].y()).unwrap(),
                chart_core::inspection::InspectionMode::Containment,
            )
            .hits;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].target, f.targets()[index][0]);
        assert!(
            inspector
                .query(
                    chart_core::Point::new(anchors[0].x() - radius * 1.5, anchors[0].y()).unwrap(),
                    chart_core::inspection::InspectionMode::Containment
                )
                .hits
                .is_empty()
        );
    }
}
