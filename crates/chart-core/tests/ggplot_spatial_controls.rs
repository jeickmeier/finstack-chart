//! FIX-GG11 actual generated geometry, defaults and replay.
use chart_core::plot::density2d;
#[path = "../../../examples/common/ggplot_spatial_controls.rs"]
mod authors;
#[test]
fn spatial_authors_emit_real_geometry_and_roundtrip() {
    for mode in 0..authors::CASES {
        let p = authors::author(mode).unwrap_or_else(|e| panic!("author {mode}: {e:?}"));
        let prepared = p
            .chart()
            .unwrap_or_else(|e| panic!("chart {mode}: {e:?}"))
            .prepare()
            .unwrap_or_else(|e| panic!("prepare {mode}: {e:?}"));
        assert!(
            !prepared.layers()[0].marks().is_empty(),
            "no geometry mode {mode}"
        );
        let wire = p.to_json().unwrap();
        assert!(
            serde_json::from_str::<serde_json::Value>(&wire).unwrap()["version"]
                .as_u64()
                .unwrap()
                >= 79
        );
        let restored = chart_core::prelude::Plot::from_json(&wire).unwrap();
        let replay = restored.chart().unwrap().prepare().unwrap();
        assert_eq!(
            prepared.layers()[0].marks(),
            replay.layers()[0].marks(),
            "replay mode {mode}"
        );
    }
}

#[test]
fn filled_levels_train_ordered_colors_and_generated_groups() {
    use chart_core::grammar::*;
    let p = authors::author(7).unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let layer = &prepared.layers()[0];
    let PreparedRows::Statistical(rows) = &layer.table().rows() else {
        panic!("stat rows");
    };
    let OutputSchema::Statistical { fields, .. } = &layer.table().schema() else {
        panic!("stat schema");
    };
    let level = fields.iter().find(|f| f.field == StatField::Level).unwrap();
    assert_eq!(
        level.space,
        ValueSpace::Categorical {
            categories: vec!["(0.5, 2]".into(), "(2.0, 4]".into()]
        }
    );
    let group = fields.iter().find(|f| f.field == StatField::Group).unwrap();
    let ValueSpace::Categorical { categories } = &group.space else {
        panic!("group catalog");
    };
    assert!(rows.iter().all(|r| categories.contains(&r.group.label())));
    assert!(rows.iter().all(|r| r.value(&StatField::LevelMid)
        == Some(
            0.5 * (r.value(&StatField::LevelLow).unwrap()
                + r.value(&StatField::LevelHigh).unwrap())
        )));
    assert!(rows.iter().all(|r| r.value(&StatField::NormalizedLevel)
        == Some(r.value(&StatField::LevelHigh).unwrap() / 4.)));
    let fills = layer
        .marks()
        .iter()
        .map(|m| m.style.fill)
        .collect::<Vec<_>>();
    assert!(fills.len() >= 2);
    assert!(
        fills.windows(2).any(|p| p[0] != p[1]),
        "ordinal levels must map to distinct actual paints"
    );
}
#[test]
fn all_missing_density_is_empty_and_named_spatial_palette_is_retained() {
    use chart_core::prelude::*;
    let data = Data::columns()
        .column("x", [None::<f64>, None])
        .column("y", [1., 2.])
        .build()
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(density2d())
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    assert!(prepared.layers()[0].marks().is_empty());
    let data = Data::columns()
        .column("x", [0., 1., 2., 2.])
        .column("y", [0., 1., 2., 2.])
        .build()
        .unwrap();
    let p = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .scale(color_continuous("fill:count", 0., 2.).palette(vec![
            chart_core::theme::rgb(
                255, 0, 0
            );
            2
        ]))
        .layer(bin2d())
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    assert!(prepared.layers()[0].marks().iter().all(|m| {
        m.style
            .fill
            .as_ref()
            .is_some_and(|p| p.red == 255 && p.green == 0)
    }));
}

#[test]
fn source_spatial_linewidth_uses_reference_physical_units() {
    use chart_core::{
        ChartResult, Rect, ResourceId, Revision,
        layout::{LayoutRequest, layout},
        scene::Primitive,
        services::{
            ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units,
        },
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
        }
    }
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/spatial-statistics-controls.json"
    ))
    .unwrap();
    let width = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "ellipse-norm")
        .unwrap()["built"]["linewidth"][0]
        .as_f64()
        .unwrap();
    for mode in [4, 6, 10, 11, 12] {
        for units in [Units::Points, Units::LogicalPixels] {
            let prepared = authors::author(mode)
                .unwrap()
                .chart()
                .unwrap()
                .prepare()
                .unwrap();
            let request = LayoutRequest::new(
                Rect::new(0., 0., 600., 360.).unwrap(),
                units,
                ResourceDescriptor {
                    id: ResourceId::new(1),
                    revision: Revision::INITIAL,
                    kind: ResourceKind::Font,
                    byte_len: 1,
                },
            );
            let frame = layout(prepared, &request, &Metrics).unwrap();
            let widths = frame
                .scene()
                .items()
                .iter()
                .filter(|i| i.layer.is_some())
                .filter_map(|i| match &i.primitive {
                    Primitive::ShapePath {
                        stroke: Some(s), ..
                    } => Some(s.width),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert!(!widths.is_empty());
            let expected = width * 72.27 / 25.4 * if units == Units::Points { 0.75 } else { 1. };
            assert!(
                widths.iter().all(|w| (w - expected).abs() < 1e-12),
                "mode {mode}: {widths:?} expected {expected}"
            );
        }
    }
}

#[test]
fn generated_continuous_fills_prepare_the_reference_colorbar() {
    use chart_core::grammar::PaintAesthetic;
    for mode in [0, 1, 2, 3, 14] {
        let prepared = authors::author(mode)
            .unwrap()
            .chart()
            .unwrap()
            .prepare()
            .unwrap();
        let legend = &prepared.layers()[0].paint_legends()[&PaintAesthetic::Fill];
        assert_eq!(legend.colorbar.len(), 300, "mode {mode}");
        assert!(legend.colorbar.windows(2).any(|p| p[0].color != p[1].color));
    }
}
