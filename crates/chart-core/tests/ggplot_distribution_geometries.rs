//! FIX-GG09: source distribution recipes retain controls, orientation and provenance.
#[path = "../../../examples/common/ggplot_distribution_geometries.rs"]
mod fixtures;
use chart_core::{
    grammar::{PreparedDistribution, PreparedGeometry, PreparedRecipe},
    prelude::*,
    provenance::Target,
};
#[test]
fn box_summary_components_and_original_outlier_target_match_reference() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/distribution-geometry-controls.json"
    ))
    .unwrap();
    let data = &reference["cases"][0]["built"];
    let p = fixtures::author(0).unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let marks = prepared.layers()[0].marks();
    assert_eq!(marks.len(), 9);
    let outlier = marks
        .iter()
        .find_map(|m| {
            if let PreparedGeometry::Recipe(r) = &m.geometry
                && let PreparedRecipe::Distribution(PreparedDistribution::Outlier {
                    center, ..
                }) = r.as_ref()
            {
                return Some((m, *center));
            }
            None
        })
        .unwrap();
    assert_eq!(outlier.1.y(), data["outliers"][0].as_f64().unwrap());
    assert!(matches!(outlier.0.targets[0],Target::Source(s)if s.key==chart_core::RowKey::new(10)));
    let whiskers = marks
        .iter()
        .filter_map(|m| {
            if let PreparedGeometry::Rule { from, to } = &m.geometry {
                (from.x() == to.x()).then_some((*from, *to))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(whiskers.len(), 4);
    assert_eq!(whiskers[0].0.y(), data["lower"][0].as_f64().unwrap());
    assert_eq!(whiskers[0].1.y(), data["ymin"][0].as_f64().unwrap());
}
#[test]
fn notches_violin_quantiles_and_dot_stack_counts_survive_replay() {
    for mode in 0..8 {
        let p = fixtures::author(mode).unwrap();
        let replay = Plot::from_json(&p.to_json().unwrap()).unwrap();
        let prepared = replay.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        assert!(!marks.is_empty(), "mode {mode}");
        if mode >= 5 {
            assert_eq!(marks.len(), 8);
        }
    }
}

#[test]
fn distribution_extent_and_orientation_controls_match_source_setup() {
    use chart_core::grammar::PreparedSurface;
    let p = fixtures::author(2).unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let widths = prepared.layers()[0]
        .marks()
        .iter()
        .filter_map(|m| {
            if let PreparedGeometry::Recipe(r) = &m.geometry
                && let PreparedRecipe::Surface(PreparedSurface::Polygon { contours, .. }) =
                    r.as_ref()
            {
                let lo = contours[0].iter().map(|p| p.x()).reduce(f64::min).unwrap();
                let hi = contours[0].iter().map(|p| p.x()).reduce(f64::max).unwrap();
                return Some(hi - lo);
            }
            None
        })
        .collect::<Vec<_>>();
    assert_eq!(widths.len(), 2);
    assert!((widths[0] - 0.75).abs() < 1e-12);
    assert!((widths[1] - 0.75 * (5_f64 / 8.).sqrt()).abs() < 1e-12);
    let p = fixtures::author(3).unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let center = prepared.layers()[0]
        .marks()
        .iter()
        .find_map(|m| {
            if let PreparedGeometry::Recipe(r) = &m.geometry
                && let PreparedRecipe::Distribution(PreparedDistribution::Outlier {
                    center, ..
                }) = r.as_ref()
            {
                return Some(*center);
            }
            None
        })
        .unwrap();
    assert_eq!((center.x(), center.y()), (20., 1.));
    for mode in [5, 7] {
        let p = fixtures::author(mode).unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let domain = prepared.layers()[0].domains().y.unwrap();
        assert_eq!(
            (domain.minimum, domain.maximum),
            if mode == 5 { (0., 1.) } else { (-0.5, 0.5) }
        );
    }
}
#[test]
fn source_outliers_follow_summary_nudge_once_and_keep_original_target() {
    use chart_core::grammar::*;
    let data = Data::columns()
        .keys([42])
        .column("x", [1.])
        .column("y", [2.])
        .build()
        .unwrap();
    let p = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(
            rule()
                .recipe(BuiltinRecipe::Boxplot(Box::new(BoxplotRecipe {
                    source_outliers: vec![SourceBoxOutliers {
                        row: chart_core::RowKey::new(42),
                        values: vec![10.],
                    }],
                    ..Default::default()
                })))
                .recipe_value(RecipeAesthetic::Lower, 1.)
                .recipe_value(RecipeAesthetic::Upper, 3.)
                .recipe_value(RecipeAesthetic::Middle, 2.)
                .recipe_value(RecipeAesthetic::WhiskerLower, 0.)
                .recipe_value(RecipeAesthetic::WhiskerUpper, 4.)
                .position(nudge(2., 3.)),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let mark = &prepared.layers()[0].marks()[0];
    let PreparedGeometry::Recipe(r) = &mark.geometry else {
        panic!()
    };
    let PreparedRecipe::Distribution(PreparedDistribution::Outlier { center, .. }) = r.as_ref()
    else {
        panic!()
    };
    assert_eq!((center.x(), center.y()), (3., 13.));
    assert!(matches!(mark.targets[0],Target::Source(s)if s.key==chart_core::RowKey::new(42)));
}

#[test]
fn box_component_alpha_and_physical_units_match_pinned_draw_contract() {
    use chart_core::{grammar::*, scene::Color};
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/distribution-geometry-controls.json"
    ))
    .unwrap();
    let case = source["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["name"] == "box-components")
        .unwrap();
    fn leaves<'a>(v: &'a serde_json::Value, out: &mut Vec<&'a serde_json::Value>) {
        if v.get("gp").is_some_and(|g| g.get("lwd").is_some()) {
            out.push(v);
        }
        if let Some(children) = v["children"].as_array() {
            for c in children {
                leaves(c, out);
            }
        }
    }
    let mut gp = vec![];
    for d in case["draw"].as_array().unwrap() {
        leaves(d, &mut gp);
    }
    let paint = |r, g, b| {
        Some(
            Color {
                red: r,
                green: g,
                blue: b,
                alpha: 255,
            }
            .into(),
        )
    };
    let spec = BoxplotRecipe {
        source_outliers: vec![SourceBoxOutliers {
            row: chart_core::RowKey::new(1),
            values: vec![20.],
        }],
        staple_width: 0.5,
        whisker: IntervalStroke {
            color: paint(255, 0, 0),
            linewidth: Some(1.),
            line_type: Some(LineType::Dashed),
        },
        staple: IntervalStroke {
            color: paint(0, 0, 255),
            linewidth: Some(0.25),
            ..Default::default()
        },
        median: IntervalStroke {
            color: paint(0, 255, 0),
            linewidth: Some(2.),
            ..Default::default()
        },
        box_style: IntervalStroke {
            color: paint(160, 32, 240),
            linewidth: Some(0.75),
            ..Default::default()
        },
        outlier: IntervalPoint {
            size: Some(3.),
            stroke: Some(1.),
            shape: Some(21),
            fill: paint(0, 0, 255),
        },
        outlier_color: paint(255, 165, 0),
        outlier_alpha: Some(0.6),
        ..Default::default()
    };
    let data = Data::columns()
        .keys([1])
        .column("x", [1.])
        .column("y", [2.])
        .build()
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            rule()
                .recipe(BuiltinRecipe::Boxplot(Box::new(spec)))
                .recipe_value(RecipeAesthetic::Lower, 1.)
                .recipe_value(RecipeAesthetic::Upper, 3.)
                .recipe_value(RecipeAesthetic::Middle, 2.)
                .recipe_value(RecipeAesthetic::WhiskerLower, 0.)
                .recipe_value(RecipeAesthetic::WhiskerUpper, 4.)
                .fill(Color {
                    red: 255,
                    green: 215,
                    blue: 0,
                    alpha: 255,
                })
                .alpha(0.2),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let marks = prepared.layers()[0].marks();
    assert_eq!(marks.len(), 7);
    for (mark, leaf) in [(1, 1), (3, 2), (5, 3), (6, 4)] {
        let expected = gp[leaf]["gp"]["lwd"]
            .as_f64()
            .unwrap_or_else(|| gp[leaf]["gp"]["lwd"][0].as_f64().unwrap())
            * 72.
            / 96.;
        assert!((marks[mark].style.stroke_width - expected).abs() < 1e-12);
        assert_eq!(marks[mark].style.color.alpha, 255);
        assert_eq!(marks[mark].style.alpha, None);
    }
    assert_eq!(marks[5].style.fill.unwrap().alpha, 51);
    assert_eq!(marks[0].style.alpha, Some(0.6));
}

#[test]
fn derived_distribution_factories_emit_components_and_source_outlier_identity() {
    for mode in 8..11 {
        let p = fixtures::author(mode).unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        assert!(!marks.is_empty(), "derived mode {mode}");
        if mode == 8 {
            let outlier=marks.iter().find(|m| matches!(&m.geometry, PreparedGeometry::Recipe(r) if matches!(r.as_ref(), PreparedRecipe::Distribution(PreparedDistribution::Outlier{..})))).unwrap();
            assert!(matches!(outlier.targets[0], Target::Source { .. }));
            assert!(marks.iter().any(|m| {
                m.targets
                    .iter()
                    .any(|t| matches!(t, Target::Aggregate { .. }))
            }));
        }
        if mode == 10 {
            assert_eq!(marks.len(), 8);
        }
        let replay = Plot::from_json(&p.to_json().unwrap()).unwrap();
        assert_eq!(
            replay.chart().unwrap().prepare().unwrap().layers()[0].marks(),
            marks
        );
    }
}

#[test]
fn signed_and_zero_dot_widths_follow_physical_stack_contract() {
    use chart_core::{
        Rect, ResourceId, Revision,
        grammar::*,
        layout::{LayoutRequest, layout},
        scene::Primitive,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let mut centers = Vec::new();
    for bw in [0.5, -0.5, 0.] {
        let data = Data::columns().column("x", [1.]).build().unwrap();
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y(0.))
            .layer(
                rule()
                    .recipe(BuiltinRecipe::Dotplot(DotplotRecipe::default()))
                    .recipe_value(RecipeAesthetic::BinWidth, bw)
                    .recipe_value(RecipeAesthetic::Count, 1.),
            )
            .build()
            .unwrap();
        let mut request = LayoutRequest::new(
            Rect::new(0., 0., 600., 360.).unwrap(),
            Units::Points,
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
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap();
        let dots = frame
            .scene()
            .items()
            .iter()
            .filter_map(|i| {
                if let Primitive::ShapePath {
                    geometry, anchors, ..
                } = &i.primitive
                {
                    Some((geometry, anchors))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if bw == 0. {
            assert!(dots.is_empty());
            continue;
        }
        assert_eq!(dots.len(), 1);
        // PathGeometry::bounds is a control-hull bound, not the actual cubic extrema.
        let flat = dots[0].0.flatten(0.001, 10000).unwrap();
        let pts = &flat.subpaths[0].points;
        let width = pts.iter().map(|p| p.x()).reduce(f64::max).unwrap()
            - pts.iter().map(|p| p.x()).reduce(f64::min).unwrap();
        let height = pts.iter().map(|p| p.y()).reduce(f64::max).unwrap()
            - pts.iter().map(|p| p.y()).reduce(f64::min).unwrap();
        assert!((width - height).abs() < 0.01);
        centers.push((dots[0].1[0].y(), width));
    }
    assert!((centers[0].1 - centers[1].1).abs() < 1e-9);
    assert!(
        centers[1].0 > centers[0].0,
        "negative width reverses stack displacement"
    );
}

#[test]
fn box_outlier_destination_paint_uses_symbol_color_and_point_stroke() {
    use chart_core::{
        Rect, ResourceId, Revision,
        layout::{LayoutRequest, layout},
        scene::Primitive,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let p = fixtures::author(0).unwrap();
    let request = LayoutRequest::new(
        Rect::new(0., 0., 600., 360.).unwrap(),
        Units::Points,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    let frame = layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap();
    let (fill, stroke) = frame
        .scene()
        .items()
        .iter()
        .find_map(|i| {
            if let Primitive::ShapePath {
                fill,
                stroke,
                anchors,
                ..
            } = &i.primitive
            {
                if anchors.len() == 1 && anchors[0].y() < 100. {
                    Some((fill, stroke))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    let fill = fill.unwrap();
    let stroke = stroke.unwrap();
    assert_eq!((fill.red, fill.green, fill.blue), (51, 51, 51));
    assert!((stroke.width - 0.5 * 72. / 25.4 / 2.).abs() < 1e-12);
}

#[test]
fn density_default_and_boundary_controls_match_source_ribbon_contract() {
    use chart_core::{
        Rect, ResourceId, Revision,
        grammar::*,
        layout::{LayoutRequest, layout},
        scene::Primitive,
        services::*,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> chart_core::ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    for (outline, lines) in [
        (AreaOutline::Upper, 1),
        (AreaOutline::Lower, 1),
        (AreaOutline::Both, 2),
        (AreaOutline::Full, 1),
    ] {
        let p = plot(
            Data::columns()
                .column("x", [0., 1., 2.])
                .column("y", [1., 2., 1.])
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            area()
                .recipe(BuiltinRecipe::Density(DensityRecipe { outline }))
                .fill(chart_core::scene::Color {
                    red: 255,
                    green: 215,
                    blue: 0,
                    alpha: 255,
                })
                .alpha(0.2),
        )
        .build()
        .unwrap();
        let request = LayoutRequest::new(
            Rect::new(0., 0., 600., 360.).unwrap(),
            Units::Points,
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: 1,
            },
        );
        let frame = layout(p.chart().unwrap().prepare().unwrap(), &request, &Metrics).unwrap();
        let paths = frame
            .scene()
            .items()
            .iter()
            .filter_map(|i| {
                if let Primitive::ShapePath {
                    fill,
                    stroke,
                    geometry,
                    ..
                } = &i.primitive
                {
                    Some((fill, stroke, geometry))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let outline_path = |fill: &Option<chart_core::scene::Color>,
                            stroke: &Option<chart_core::scene::Stroke>| {
            stroke.is_some() || fill.is_some_and(|c| c.red == 0 && c.green == 0 && c.blue == 0)
        };
        assert_eq!(
            paths.iter().filter(|(f, s, _)| outline_path(f, s)).count(),
            lines
        );
        assert_eq!(
            paths
                .iter()
                .filter(|(f, _, _)| f.is_some_and(|c| c.red == 255 && c.green == 215))
                .count(),
            1
        );
        for (fill, stroke, geometry) in paths {
            if outline_path(fill, stroke) {
                assert_eq!(stroke.map(|s| s.color).or(*fill).unwrap().alpha, 255);
                let flat = geometry.flatten(0.01, 10000).unwrap();
                let ys = flat
                    .subpaths
                    .iter()
                    .flat_map(|s| s.points.iter().map(|p| p.y()))
                    .collect::<Vec<_>>();
                let span = ys.iter().copied().reduce(f64::max).unwrap()
                    - ys.iter().copied().reduce(f64::min).unwrap();
                if outline == AreaOutline::Upper {
                    assert!(
                        span > 10.,
                        "expanded upper outline must follow estimated y values"
                    );
                }
                if outline == AreaOutline::Lower {
                    assert!(
                        span < 2.,
                        "expanded lower outline stays within its stroke width of baseline"
                    );
                }
            } else if let Some(fill) = fill {
                assert_eq!(fill.alpha, 51);
            }
        }
    }
    let p = chart_core::plot::plot(Data::columns().column("x", [0., 1., 2.]).build().unwrap())
        .profile(Profile::Ggplot2_4_0_3)
        .layer(
            chart_core::plot::density()
                .stat(chart_core::plot::density_stat().input("x"))
                .alpha(0.2),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    assert!(
        prepared.layers()[0]
            .marks()
            .iter()
            .all(|m| m.style.fill.is_none())
    );
}
