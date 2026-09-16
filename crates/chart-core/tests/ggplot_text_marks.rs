//! FIX-GG08: row text shares provenance, shaping and bounded portable geometry.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision,
    grammar::{TextGeom, TextSizeUnit},
    layout::{LayoutRequest, layout},
    prelude::*,
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    typography::{ShapeRequest, ShapedRun},
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest) -> ChartResult<TextMetrics> {
        TextMetrics::new(
            r.text.chars().count() as f64 * r.font_size * 0.5,
            r.font_size * 0.8,
            r.font_size * 0.2,
        )
    }
    fn shape(&self, r: ShapeRequest<'_>) -> ChartResult<ShapedRun> {
        let size = r.font_size * r.run.size;
        Ok(ShapedRun {
            text: r.run.text.clone(),
            font: r.run.font.unwrap_or(*r.default_font),
            font_size: size,
            language: r.run.language.clone(),
            direction: r.run.direction,
            tabular: r.run.tabular,
            metrics: TextMetrics::new(
                r.run.text.chars().count() as f64 * size * 0.5,
                size * 0.8,
                size * 0.2,
            )?,
            glyphs: vec![],
            outlines: vec![],
            used_fallback: false,
        })
    }
}
fn request(units: Units) -> LayoutRequest {
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 800., 600.).unwrap(),
        units,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    for axis in &mut r.axes {
        axis.visible = false;
    }
    r
}
fn author(labels: Vec<&str>, options: TextGeom) -> Plot {
    let n = labels.len();
    let data = Data::columns()
        .column("x", vec![1.; n])
        .column("y", vec![1.; n])
        .column("label", labels)
        .build()
        .unwrap();
    plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points().text_geom(options).text_label("label"))
        .build()
        .unwrap()
}
#[test]
fn source_rows_unicode_empty_and_wire_roundtrip_are_not_fixed_annotations() {
    let mut labels = vec!["row"; 300];
    labels[0] = "";
    labels[1] = "éλ";
    let p = author(labels, TextGeom::default());
    let restored = Plot::from_json(&p.to_json().unwrap()).unwrap();
    let prepared = restored.chart().unwrap().prepare().unwrap();
    assert_eq!(prepared.layers()[0].marks().len(), 300);
    let scene = layout(prepared, &request(Units::Points), &Metrics).unwrap();
    let texts: Vec<_> = scene
        .scene()
        .items()
        .iter()
        .filter_map(|i| match &i.primitive {
            Primitive::GlyphRun { run, .. } if i.layer.is_some() => Some(run.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(texts.len(), 299);
    assert_eq!(texts[0], "éλ");
    assert_eq!(scene.interactions().len(), 299);
    let anchor = scene
        .interactions()
        .values()
        .next()
        .unwrap()
        .hit
        .anchor()
        .unwrap();
    let inspector =
        chart_core::inspection::Inspector::new(std::sync::Arc::new(scene.clone()), 4., 16).unwrap();
    assert!(
        !inspector
            .query(anchor, chart_core::inspection::InspectionMode::Auto)
            .hits
            .is_empty()
    );
    assert!(
        scene
            .scene()
            .items()
            .iter()
            .all(|i| !matches!(i.primitive, Primitive::Point { .. }))
    );
}
#[test]
fn rotated_boxes_and_overlap_use_whole_label_and_physical_sizes() {
    let options = TextGeom {
        size: 10.,
        units: TextSizeUnit::Points,
        angle: 90.,
        fill: Some(chart_core::scene::Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        }),
        check_overlap: true,
        ..Default::default()
    };
    let p = author(vec!["abcd", "later"], options);
    for units in [Units::Points, Units::LogicalPixels] {
        let scene = layout(
            p.chart().unwrap().prepare().unwrap(),
            &request(units),
            &Metrics,
        )
        .unwrap();
        let items: Vec<_> = scene
            .scene()
            .items()
            .iter()
            .filter(|i| i.layer.is_some())
            .collect();
        assert_eq!(items.len(), 2);
        let Primitive::GlyphRun { rotation, run, .. } = &items[1].primitive else {
            panic!("text")
        };
        assert_eq!(*rotation, 90.);
        assert_eq!(run.text, "abcd");
        let expected = if units == Units::Points {
            10.
        } else {
            40. / 3.
        };
        assert!((run.font_size - expected).abs() < 1e-12);
        let Primitive::VectorPath { geometry, .. } = &items[0].primitive else {
            panic!("box")
        };
        let bounds = geometry.bounds(0.01, 1000).unwrap().unwrap();
        assert!(
            (bounds.width() - (1.5 * expected + 0.02)).abs() < 1e-10,
            "width={} expected={}",
            bounds.width(),
            1.5 * expected
        );
        assert!((bounds.height() - (2.5 * expected + 0.02)).abs() < 1e-10);
    }
}
#[test]
fn data_labels_honor_text_budget_and_invalid_controls_fail() {
    let p = author(vec!["12345678"; 20], TextGeom::default());
    let mut r = request(Units::Points);
    r.limits.max_text_bytes = 100;
    assert!(layout(p.chart().unwrap().prepare().unwrap(), &r, &Metrics).is_err());
    let data = Data::columns().column("x", vec![1.]).build().unwrap();
    assert!(
        plot(data)
            .aes(aes().x("x").y(1.))
            .layer(points().text_geom(TextGeom {
                padding: [-1., 0.],
                ..Default::default()
            }))
            .build()
            .is_err()
    );
}
#[test]
fn generated_count_labels_retain_statistical_targets() {
    use chart_core::grammar::StatField;
    let data = Data::columns()
        .column("x", vec!["A", "A", "B"])
        .build()
        .unwrap();
    let p = plot(data)
        .aes(aes().x("x"))
        .layer(
            points()
                .stat(count().group("x"))
                .after_stat(stat_aes().x(StatField::Group).y(StatField::Count))
                .text_geom(TextGeom::default())
                .text_stat_label(StatField::Count),
        )
        .build()
        .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    assert_eq!(prepared.layers()[0].marks().len(), 2);
    let scene = layout(prepared, &request(Units::Points), &Metrics).unwrap();
    let labels: Vec<_> = scene
        .scene()
        .items()
        .iter()
        .filter_map(|i| match &i.primitive {
            Primitive::GlyphRun { run, .. } if i.layer.is_some() => Some(run.text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(labels, vec!["2", "1"]);
}
#[test]
fn reference_units_and_built_text_rows_match_pinned_capture() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/text-marks.json"
    ))
    .unwrap();
    for (key, unit) in [
        ("mm", TextSizeUnit::Millimeters),
        ("pt", TextSizeUnit::Points),
        ("cm", TextSizeUnit::Centimeters),
        ("in", TextSizeUnit::Inches),
        ("pc", TextSizeUnit::Picas),
    ] {
        assert!(
            (unit.factor(Units::Points) - reference["units"][key].as_f64().unwrap()).abs() < 1e-12
        );
    }
    for case in reference["cases"].as_array().unwrap() {
        let labels = case["labels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let p = author(
            labels,
            TextGeom {
                angle: case["angle"].as_f64().unwrap(),
                hjust: case["hjust"].as_f64().unwrap(),
                vjust: case["vjust"].as_f64().unwrap(),
                ..Default::default()
            },
        );
        assert_eq!(
            p.chart().unwrap().prepare().unwrap().layers()[0]
                .marks()
                .len(),
            case["rows"].as_u64().unwrap() as usize
        );
    }
}
#[test]
fn raster_owns_one_portable_primitive_and_retains_alpha_and_interpolation() {
    use chart_core::grammar::{AestheticUnits, AnnotationContent, RasterAnnotation, RowAnnotation};
    let pixels = vec![
        chart_core::scene::Color {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 128,
        },
        chart_core::scene::Color {
            red: 0,
            green: 0,
            blue: 255,
            alpha: 255,
        },
    ];
    for interpolate in [false, true] {
        let data = Data::columns().column("x", vec![1.]).build().unwrap();
        let p = plot(data)
            .aes(aes().x("x").y(1.))
            .layer(points().annotation(RowAnnotation {
                units: AestheticUnits::Points,
                content: AnnotationContent::Raster {
                    raster: RasterAnnotation {
                        width: 2,
                        height: 1,
                        pixels: pixels.clone(),
                    },
                    bounds: [-20., -10., 40., 20.],
                    interpolate,
                },
            }))
            .build()
            .unwrap();
        let restored = Plot::from_json(&p.to_json().unwrap()).unwrap();
        let scene = layout(
            restored.chart().unwrap().prepare().unwrap(),
            &request(Units::Points),
            &Metrics,
        )
        .unwrap();
        assert_eq!(scene.scene().wire_version(), 20);
        let raster = scene
            .scene()
            .items()
            .iter()
            .find_map(|i| {
                if let Primitive::RasterImage {
                    raster,
                    interpolate: mode,
                    bounds,
                    ..
                } = &i.primitive
                {
                    Some((raster, mode, bounds))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(raster.0.pixels, pixels);
        assert_eq!(*raster.1, interpolate);
        assert_eq!(raster.2.width(), 40.);
    }
}
