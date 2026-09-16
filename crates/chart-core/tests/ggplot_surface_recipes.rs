//! FIX-GG07: compound contours distinguish parity fill from winding fill.
use chart_core::{
    Point,
    path::{FlatSubpath, FlattenedPath},
    scene::FillRule,
};
#[test]
fn compound_fill_preserves_nonzero_and_supports_same_winding_holes() {
    let square = |lo, hi| FlatSubpath {
        points: [(lo, lo), (hi, lo), (hi, hi), (lo, hi)]
            .map(|(x, y)| Point::new(x, y).unwrap())
            .to_vec(),
        closed: true,
    };
    let flat = FlattenedPath {
        subpaths: vec![square(0., 10.), square(3., 7.)],
        max_error: 0.01,
    };
    let hole = Point::new(5., 5.).unwrap();
    assert!(flat.contains(hole, true, None));
    assert!(!flat.contains_with_rule(hole, true, None, FillRule::EvenOdd));
    assert!(flat.contains_with_rule(Point::new(1., 5.).unwrap(), true, None, FillRule::EvenOdd));
    assert!(!flat.contains_with_rule(Point::new(12., 5.).unwrap(), true, None, FillRule::EvenOdd));
    assert!(flat.contains_with_rule(
        Point::new(3., 5.).unwrap(),
        false,
        Some(1.),
        FillRule::EvenOdd
    ));
    let mut opposite = flat.clone();
    opposite.subpaths[1].points.reverse();
    assert!(!opposite.contains(hole, true, None));
    assert!(!opposite.contains_with_rule(hole, true, None, FillRule::EvenOdd));
}

#[path = "../../../examples/common/ggplot_surface_recipes.rs"]
mod fixtures;
use chart_core::{
    ChartResult, Rect, ResourceId, Revision,
    grammar::PreparedGeometry,
    inspection::{InspectionMode, Inspector},
    layout::{LaidOutChart, LayoutRequest, layout},
    prelude::*,
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(mode: usize) -> Arc<LaidOutChart> {
    let original = fixtures::author(mode).unwrap();
    let plot = Plot::from_json(&original.to_json().unwrap()).unwrap();
    let prepared = plot.chart().unwrap().prepare().unwrap();
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
    Arc::new(layout(prepared, &request, &Metrics).unwrap())
}
#[test]
fn polygon_reference_holes_hit_only_when_fill_rule_paints_the_interior() {
    for mode in 0..4 {
        let f = frame(mode);
        let item = f
            .scene()
            .items()
            .iter()
            .find(|i| matches!(i.primitive, Primitive::ShapePath { .. }))
            .unwrap();
        let Primitive::ShapePath {
            geometry, anchors, ..
        } = &item.primitive
        else {
            unreachable!()
        };
        assert_eq!(anchors.len(), 8);
        let b = geometry.bounds(0.01, 1000).unwrap().unwrap();
        let center = Point::new(
            b.origin().x() + b.width() / 2.,
            b.origin().y() + b.height() / 2.,
        )
        .unwrap();
        let inspector = Inspector::new(f.clone(), 0.5, 32).unwrap();
        assert_eq!(
            !inspector
                .query(center, InspectionMode::Auto)
                .hits
                .is_empty(),
            mode == 2,
            "mode {mode}"
        );
        if mode < 2 {
            assert_eq!(f.scene().wire_version(), 21);
        } else {
            assert!(f.scene().wire_version() < 21);
        }
    }
}
#[test]
fn tile_extents_match_pinned_resolution_and_explicit_sizes_before_layout() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/primitive-recipes.json"
    ))
    .unwrap();
    for mode in [4, 5] {
        let plot = fixtures::author(mode).unwrap();
        let prepared = plot.chart().unwrap().prepare().unwrap();
        let case = reference["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == if mode == 4 { "tile-FALSE" } else { "tile-TRUE" })
            .unwrap();
        let expected = &case["data"][0][0];
        let PreparedGeometry::Rectangle { from, to } = &prepared.layers()[0].marks()[0].geometry
        else {
            panic!("tile rectangle")
        };
        // Explicit fixture width/height are checked independently below; source recipe uses 0.8/0.6.
        if mode == 4 {
            assert_eq!(from.x(), expected["xmin"].as_f64().unwrap());
            assert_eq!(to.x(), expected["xmax"].as_f64().unwrap());
        }
        assert!((to.x() - from.x() - if mode == 4 { 1. } else { 0.8 }).abs() < 1e-12);
        assert!((to.y() - from.y() - if mode == 4 { 1. } else { 0.6 }).abs() < 1e-12);
    }
    let generated = fixtures::author(8)
        .unwrap()
        .chart()
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(generated.layers()[0].marks().len(), 2);
}
#[test]
fn raster_missing_cell_alpha_and_original_cell_targets_survive_plot_replay() {
    for mode in [6, 7] {
        let f = frame(mode);
        let (index, raster, cells, interpolate) = f
            .scene()
            .items()
            .iter()
            .enumerate()
            .find_map(|(i, s)| {
                if let Primitive::RasterImage {
                    raster,
                    cells,
                    interpolate,
                    ..
                } = &s.primitive
                {
                    Some((i, raster, cells, *interpolate))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!((raster.width, raster.height), (3, 2));
        assert_eq!(interpolate, mode == 7);
        assert_eq!(raster.pixels.iter().filter(|c| c.alpha == 0).count(), 1);
        assert_eq!(cells.len(), 5);
        assert_eq!(f.scene().wire_version(), 21);
        let inspector = Inspector::new(f.clone(), 0.1, 32).unwrap();
        for (cell, target) in cells.iter().zip(&f.targets()[index]) {
            let p = Point::new(
                cell.origin().x() + cell.width() / 2.,
                cell.origin().y() + cell.height() / 2.,
            )
            .unwrap();
            let hits = inspector.query(p, InspectionMode::Auto).hits;
            assert_eq!(hits.len(), 1);
            assert_eq!(&hits[0].target, target);
        }
        let first = cells[0];
        let missing = Point::new(
            first.max_x() + first.width() / 2.,
            first.origin().y() + first.height() / 2.,
        )
        .unwrap();
        assert!(
            inspector
                .query(missing, InspectionMode::Auto)
                .hits
                .is_empty()
        );
    }
}
#[test]
fn raster_justification_and_irregular_index_truncation_match_pinned_reference() {
    use chart_core::grammar::{BuiltinRecipe, PreparedRecipe, PreparedSurface, RasterRecipe};
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/surface-controls.json"
    ))
    .unwrap();
    for case in reference["cases"].as_array().unwrap() {
        let data = Data::columns()
            .column("x", [0., 1., 2.8])
            .column("y", [1., 1., 1.])
            .build()
            .unwrap();
        let p = plot(data)
            .aes(aes().x("x").y("y"))
            .layer(points().recipe(BuiltinRecipe::Raster(RasterRecipe {
                hjust: case["hjust"].as_f64().unwrap(),
                vjust: case["vjust"].as_f64().unwrap(),
                interpolate: false,
            })))
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let PreparedGeometry::Recipe(recipe) = &prepared.layers()[0].marks()[0].geometry else {
            panic!()
        };
        let PreparedRecipe::Surface(PreparedSurface::Raster {
            cells,
            width,
            height,
            ..
        }) = recipe.as_ref()
        else {
            panic!()
        };
        assert_eq!((*width, *height), (3, 1));
        for (i, cell) in cells.iter().enumerate() {
            let expected = &case["data"][i];
            assert!((cell.corners[0].x() - expected["xmin"].as_f64().unwrap()).abs() < 1e-12);
            assert!((cell.corners[1].y() - expected["ymax"].as_f64().unwrap()).abs() < 1e-12);
            assert_eq!(cell.grid, [i, 0]);
        }
    }
}
#[test]
fn tile_mapped_dimensions_and_nudge_apply_once_and_statistics_remain_composable() {
    use chart_core::grammar::{BuiltinRecipe, RecipeAesthetic, StatField, TileRecipe};
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("y", [2., 3.])
        .column("w", [0.4, 0.8])
        .column("h", [0.6, 1.2])
        .build()
        .unwrap();
    let p = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(
            points()
                .recipe(BuiltinRecipe::Tile(TileRecipe::default()))
                .recipe_value(RecipeAesthetic::Width, "w")
                .recipe_value(RecipeAesthetic::Height, "h")
                .position(nudge(1., 2.)),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedGeometry::Rectangle { from, to } = prepared.layers()[0].marks()[0].geometry else {
        panic!()
    };
    assert!((from.x() - 1.8).abs() < 1e-12);
    assert!((to.x() - 2.2).abs() < 1e-12);
    assert!((from.y() - 3.7).abs() < 1e-12);
    assert!((to.y() - 4.3).abs() < 1e-12);
    for recipe in [
        BuiltinRecipe::Polygon(Default::default()),
        BuiltinRecipe::Raster(Default::default()),
    ] {
        let p = plot(
            Data::columns()
                .column("g", ["A", "A", "B", "C", "C", "C"])
                .build()
                .unwrap(),
        )
        .layer(
            points()
                .stat(count().group("g"))
                .after_stat(stat_aes().x(StatField::Group).y(StatField::Count))
                .recipe(recipe),
        )
        .build()
        .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        assert_eq!(
            prepared.layers()[0]
                .marks()
                .iter()
                .map(|m| m.targets.len())
                .sum::<usize>(),
            3
        );
    }
}
#[test]
fn raster_log_scale_transforms_before_pixel_extents() {
    use chart_core::grammar::{BuiltinRecipe, PreparedRecipe, PreparedSurface};
    let data = Data::columns()
        .column("x", [10., 100., 1000.])
        .column("y", [1., 1., 1.])
        .build()
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .x_axis(x_axis().scale(scale_log(10.)))
        .layer(points().recipe(BuiltinRecipe::Raster(Default::default())))
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedGeometry::Recipe(recipe) = &prepared.layers()[0].marks()[0].geometry else {
        panic!()
    };
    let PreparedRecipe::Surface(PreparedSurface::Raster { cells, .. }) = recipe.as_ref() else {
        panic!()
    };
    for (i, c) in cells.iter().enumerate() {
        assert!((c.corners[0].x() - (i as f64 + 0.5)).abs() < 1e-12);
        assert!((c.corners[1].x() - (i as f64 + 1.5)).abs() < 1e-12);
    }
}

#[test]
fn explicitly_missing_tile_dimensions_do_not_fall_back_to_defaults() {
    use chart_core::grammar::{BuiltinRecipe, RecipeAesthetic, TileRecipe};
    for channel in [RecipeAesthetic::Width, RecipeAesthetic::Height] {
        let data = Data::columns()
            .column("x", [1., 100.])
            .column("y", [1., 2.])
            .column("size", [Some(0.8), None])
            .build()
            .unwrap();
        let p = plot(data)
            .aes(aes().x("x").y("y"))
            .layer(
                points()
                    .recipe(BuiltinRecipe::Tile(TileRecipe {
                        width: Some(4.),
                        height: Some(4.),
                    }))
                    .recipe_value(channel, "size"),
            )
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        assert_eq!(prepared.layers()[0].marks().len(), 1);
        assert_eq!(prepared.layers()[0].domains().x.unwrap().maximum, 100.);
    }
}
