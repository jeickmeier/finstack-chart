//! FIX-GG03: independent aesthetic training, precedence, dimensions and identity.
use chart_core::plot::shape_line;
use chart_core::{
    ChartResult, Point, Rect, ResourceId, Revision,
    grammar::{Compiler, NumericAesthetic as A, PaintAesthetic},
    inspection::{InspectionMode, Inspector},
    layout::{AxisScale, LaidOutChart, LayoutRequest, layout},
    prelude::*,
    scales::{Bounds, ContinuousDomain},
    scene::Primitive,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    state::ChartState,
};
use std::sync::Arc;
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn frame(plot: &Plot) -> Arc<LaidOutChart> {
    frame_units(plot, Units::LogicalPixels)
}
fn frame_units(plot: &Plot, units: Units) -> Arc<LaidOutChart> {
    let prepared = Compiler::new()
        .prepare(
            plot.definition(),
            &plot.source(),
            &ChartState::default(),
            plot.compile_limits(),
        )
        .unwrap();
    let mut request = LayoutRequest::new(
        Rect::new(0., 0., 400., 200.).unwrap(),
        units,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    request.padding = 0.;
    for a in &mut request.axes {
        a.visible = false;
        a.scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 4.).unwrap()));
    }
    Arc::new(layout(Arc::new(prepared), &request, &Metrics).unwrap())
}

fn prepared(p: &Plot) -> chart_core::grammar::PreparedChart {
    Compiler::new()
        .prepare(
            p.definition(),
            &p.source(),
            &ChartState::default(),
            p.compile_limits(),
        )
        .unwrap()
}
fn data() -> Data {
    Data::columns()
        .keys([9007199254741001, 9007199254741003])
        .column("x", [1., 3.])
        .column("y", [2., 2.])
        .column("fill", categorical(["A", "B"]))
        .column("stroke", categorical(["B", "A"]))
        .column("area", [1., 4.])
        .column("alpha", [0.5, 1.])
        .build()
        .unwrap()
}
#[test]
fn independent_paint_training_constant_precedence_and_roundtrip() {
    let red = rgb(255, 0, 0);
    let blue = rgb(0, 0, 255);
    let green = rgb(0, 128, 0);
    let black = rgb(0, 0, 0);
    let p = plot(data())
        .aes(
            aes()
                .x("x")
                .y("y")
                .fill("fill")
                .fill_scale("inside")
                .stroke("stroke")
                .stroke_scale("outside"),
        )
        .scale(
            color_discrete("inside")
                .domain(["A", "B"])
                .palette(vec![red, blue]),
        )
        .scale(
            color_discrete("outside")
                .domain(["A", "B"])
                .palette(vec![black, green]),
        )
        .layer(
            points()
                .name("points")
                .radius(5.)
                .linewidth(2.)
                .shape_value(A::Opacity, "alpha"),
        )
        .build()
        .unwrap();
    let pre = prepared(&p);
    let marks = pre.layers()[0].marks();
    assert_eq!(
        marks[0].style.fill.unwrap(),
        chart_core::scene::Color { alpha: 128, ..red }
    );
    assert_eq!(marks[1].style.fill.unwrap(), blue);
    assert_eq!(
        marks[0].style.stroke.unwrap(),
        chart_core::scene::Color {
            alpha: 128,
            ..green
        }
    );
    assert_eq!(marks[1].style.stroke.unwrap(), black);
    assert_eq!(marks[0].style.radius, 5.);
    assert_eq!(marks[0].style.stroke_width, 2.);
    assert_eq!(pre.layers()[0].paint_legends().len(), 2);
    assert_ne!(
        pre.layers()[0].paint_legends()[&PaintAesthetic::Fill].id,
        pre.layers()[0].paint_legends()[&PaintAesthetic::Stroke].id
    );
    let wire = p.to_json().unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&wire).unwrap()["version"],
        16
    );
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    let edited = p
        .edit()
        .layer("points", points().fill(black))
        .build()
        .unwrap();
    assert!(
        !prepared(&edited).layers()[0]
            .paint_legends()
            .contains_key(&PaintAesthetic::Fill)
    );
    assert!(
        prepared(&edited).layers()[0]
            .marks()
            .iter()
            .all(|m| m.style.fill == Some(black))
    );
    assert_eq!(p.to_json().unwrap(), wire);
    let changed = p
        .edit()
        .scale(
            color_discrete("inside")
                .domain(["A", "B"])
                .palette(vec![green, black]),
        )
        .build()
        .unwrap();
    assert_eq!(
        prepared(&changed).layers()[0].marks()[1].style.fill,
        Some(black)
    );
}
#[test]
fn area_size_has_fourfold_area_legacy_radius_has_sixteenfold_area() {
    let make = |layer| {
        plot(data())
            .aes(aes().x("x").y("y"))
            .layer(layer)
            .build()
            .unwrap()
    };
    let p = make(points().linewidth(3.).shape_value(A::AreaSize, "area"));
    let pre = prepared(&p);
    let m = pre.layers()[0].marks();
    assert!((m[1].style.radius.powi(2) / m[0].style.radius.powi(2) - 4.).abs() < 1e-12);
    assert!(m.iter().all(|m| m.style.stroke_width == 3.));
    let old = make(points().aes(aes().size("area")));
    let pre = prepared(&old);
    let m = pre.layers()[0].marks();
    assert_eq!(m[1].style.radius.powi(2) / m[0].style.radius.powi(2), 16.);
}
#[test]
fn outlined_points_retain_one_exact_source_anchor_and_outline_hit() {
    let p = plot(data())
        .aes(aes().x("x").y("y"))
        .layer(
            points()
                .radius(5.)
                .linewidth(2.)
                .fill(rgb(255, 0, 0))
                .stroke(rgb(0, 0, 0)),
        )
        .build()
        .unwrap();
    let f = frame(&p);
    let inspector = Inspector::new(f.clone(), 0.001, 128).unwrap();
    let (i, center) = f
        .scene()
        .items()
        .iter()
        .enumerate()
        .find_map(|(i, item)| match &item.primitive {
            Primitive::ShapePath {
                anchors,
                fill,
                stroke,
                ..
            } => {
                assert!(fill.is_some() && stroke.is_some());
                assert_eq!(anchors.len(), 1);
                Some((i, anchors[0]))
            }
            _ => None,
        })
        .unwrap();
    let result = inspector.query(
        Point::new(center.x() + 5.5, center.y()).unwrap(),
        InspectionMode::Containment,
    );
    assert_eq!(result.hits.len(), 1);
    assert_eq!(result.hits[0].target, f.targets()[i][0]);
    assert!(
        inspector
            .query(
                Point::new(center.x() + 6.1, center.y()).unwrap(),
                InspectionMode::Containment
            )
            .hits
            .is_empty()
    );
}

#[test]
fn alpha_replaces_embedded_alpha_while_legacy_opacity_multiplies() {
    let color = chart_core::scene::Color {
        red: 200,
        green: 20,
        blue: 10,
        alpha: 64,
    };
    let make = |layer| {
        plot(data())
            .aes(aes().x("x").y("y"))
            .layer(layer)
            .build()
            .unwrap()
    };
    let alpha = make(points().fill(color).shape_value(A::Alpha, 0.5));
    let opacity = make(points().fill(color).shape_value(A::Opacity, 0.5));
    assert_eq!(
        prepared(&alpha).layers()[0].marks()[0]
            .style
            .fill
            .unwrap()
            .alpha,
        128
    );
    assert_eq!(
        prepared(&opacity).layers()[0].marks()[0]
            .style
            .fill
            .unwrap()
            .alpha,
        32
    );
    let constant = make(points().fill(color).alpha(0.25).shape_value(A::Alpha, 0.5));
    assert_eq!(
        prepared(&constant).layers()[0].marks()[0]
            .style
            .fill
            .unwrap()
            .alpha,
        64
    );
    assert!(make_result_invalid_alpha());
}
fn make_result_invalid_alpha() -> bool {
    plot(data())
        .aes(aes().x("x").y("y"))
        .layer(points().alpha(1.1))
        .build()
        .is_err()
}

#[test]
fn physical_units_and_line_patterns_share_rendered_stroke_and_hit_geometry() {
    use chart_core::grammar::{AestheticUnits, LineType};
    let p = plot(data())
        .aes(aes().x("x").y("y"))
        .layer(
            points()
                .radius(1.)
                .linewidth(0.2)
                .aesthetic_units(AestheticUnits::Millimeters)
                .stroke(rgb(0, 0, 0)),
        )
        .build()
        .unwrap();
    let f = frame(&p);
    let (geometry, stroke) = f
        .scene()
        .items()
        .iter()
        .find_map(|item| match &item.primitive {
            Primitive::ShapePath {
                geometry,
                stroke: Some(stroke),
                ..
            } => Some((geometry, stroke)),
            _ => None,
        })
        .unwrap();
    assert!((stroke.width - 0.2 * 96. / 25.4).abs() < 1e-12);
    assert!(
        geometry
            .to_svg(chart_core::path::Precision::Unrounded, 4096)
            .unwrap()
            .contains("3.779527")
    );
    let p = plot(data())
        .aes(aes().x("x").y("y"))
        .layer(shape_line().linewidth(2.).line_type(LineType::Dashed))
        .build()
        .unwrap();
    let f = frame(&p);
    let inspector = Inspector::new(f.clone(), 0.001, 128).unwrap();
    let (start, pattern) = f
        .scene()
        .items()
        .iter()
        .find_map(|item| match &item.primitive {
            Primitive::ShapePath {
                anchors, dashes, ..
            } => Some((anchors[0], dashes)),
            _ => None,
        })
        .unwrap();
    assert_eq!(pattern, &[8., 8.]);
    assert_eq!(
        inspector
            .query(
                Point::new(start.x() + 4., start.y()).unwrap(),
                InspectionMode::Containment
            )
            .hits
            .len(),
        1
    );
    assert!(
        inspector
            .query(
                Point::new(start.x() + 12., start.y()).unwrap(),
                InspectionMode::Containment
            )
            .hits
            .is_empty()
    );
}

#[test]
fn text_channels_use_the_same_scale_engine_and_retain_typed_values() {
    use chart_core::{
        grammar::ValueAesthetic as V,
        interpolate::{Number, Value},
        scales::{ScaleConstructor, ScaleOptions, ScaleTraining},
    };
    let family = ScaleConstructor::Ordinal
        .create(ScaleOptions {
            range: Some(vec![
                Value::Text("Noto Sans".into()),
                Value::Text("Noto Serif".into()),
            ]),
            ..Default::default()
        })
        .unwrap()
        .mapped(ScaleTraining::Eligible)
        .unwrap();
    let size = ScaleConstructor::Linear
        .create(ScaleOptions::default())
        .unwrap()
        .mapped(ScaleTraining::Authored)
        .unwrap();
    let p = plot(data())
        .aes(aes().x("x").y("y"))
        .layer(
            points()
                .value_scale(V::FontFamily, "fill", family)
                .value_scale(V::TextSize, "area", size)
                .aesthetic_value(V::FontFace, Value::Text("bold".into()))
                .aesthetic_value(V::HJust, Value::Number(Number(0.5))),
        )
        .build()
        .unwrap();
    let pre = prepared(&p);
    let marks = pre.layers()[0].marks();
    assert_eq!(
        marks[0].aesthetics[&V::FontFamily],
        Value::Text("Noto Sans".into())
    );
    assert_eq!(
        marks[1].aesthetics[&V::FontFamily],
        Value::Text("Noto Serif".into())
    );
    assert_eq!(marks[1].aesthetics[&V::TextSize], Value::Number(Number(4.)));
    assert_eq!(
        marks[0].aesthetics[&V::FontFace],
        Value::Text("bold".into())
    );
    assert_eq!(
        Plot::from_json(&p.to_json().unwrap())
            .unwrap()
            .to_json()
            .unwrap(),
        p.to_json().unwrap()
    );
}

#[test]
fn ggplot_solid_line_segments_take_their_start_rows_style_after_sorting() {
    use chart_core::grammar::{LineType, PreparedGeometry};
    let d = Data::columns()
        .keys([30, 10, 20])
        .column("x", [3., 1., 2.])
        .column("y", [2., 1., 3.])
        .column("width", [3., 1., 2.])
        .build()
        .unwrap();
    let make = |profile, layer| {
        plot(d.clone())
            .profile(profile)
            .aes(aes().x("x").y("y").group_all())
            .layer(layer)
            .build()
    };
    let p = make(
        Profile::Ggplot2_4_0_3,
        line().shape_value(A::StrokeWidth, "width"),
    )
    .unwrap();
    let pre = prepared(&p);
    let marks = pre.layers()[0].marks();
    assert_eq!(marks.len(), 2);
    assert_eq!(
        marks
            .iter()
            .map(|m| m.style.stroke_width)
            .collect::<Vec<_>>(),
        [1., 2.]
    );
    for (mark, x) in marks.iter().zip([1., 2.]) {
        let PreparedGeometry::LineRun(points) = &mark.geometry else {
            panic!()
        };
        assert_eq!(points[0].x(), x);
        assert_eq!(points[1].x(), x + 1.);
        assert_eq!(mark.targets.len(), 2);
    }
    let old = make(
        Profile::LibraryV1,
        line().shape_value(A::StrokeWidth, "width"),
    )
    .unwrap();
    assert!(old.chart().unwrap().prepare().is_err());
    let dashed = make(
        Profile::Ggplot2_4_0_3,
        line()
            .shape_value(A::StrokeWidth, "width")
            .line_type(LineType::Dashed),
    )
    .unwrap();
    assert!(dashed.chart().unwrap().prepare().is_err());
}

#[test]
fn new_after_scale_channels_read_one_snapshot_and_preserve_width_defaults() {
    use chart_core::grammar::AfterScaleAesthetic as S;
    let p = plot(data())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            points().linewidth(2.).after_scale(
                scale_aes()
                    .fill(after_scale_expr(S::Color))
                    .stroke(after_scale_expr(S::Fill))
                    .alpha(after_scale_expr(S::Alpha) * 0.5)
                    .linewidth(after_scale_expr(S::LineWidth) * 2.),
            ),
        )
        .build()
        .unwrap();
    let pre = prepared(&p);
    let style = pre.layers()[0].marks()[0].style;
    assert_eq!(style.stroke_width, 4.);
    assert_eq!(style.fill.unwrap().alpha, 128);
    assert_eq!(style.fill, style.stroke);
    assert_eq!(
        style.radius,
        chart_core::theme::GeometryTheme::<chart_core::scene::Color>::default().point_size
    );
}

#[test]
fn ggplot_constant_width_suppresses_mapping_training_but_retains_authored_wire() {
    let p = plot(data())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y").group_all())
        .layer(line().linewidth(2.).shape_value(A::StrokeWidth, "area"))
        .build()
        .unwrap();
    assert!(
        p.definition().layers[0]
            .numeric_scales
            .contains_key(&A::StrokeWidth)
    );
    let pre = prepared(&p);
    assert_eq!(pre.layers()[0].marks().len(), 1);
    assert_eq!(pre.layers()[0].marks()[0].style.stroke_width, 2.);
    assert!(
        !pre.layers()[0]
            .numeric_scales()
            .contains_key(&A::StrokeWidth)
    );
}

#[test]
fn value_channel_reductions_specialize_over_source_rows() {
    use chart_core::{
        grammar::{ExpressionReduce, ValueAesthetic as V},
        interpolate::{Number, Value},
        scales::{ScaleConstructor, ScaleOptions, ScaleTraining},
    };
    let scale = ScaleConstructor::Linear
        .create(ScaleOptions::default())
        .unwrap()
        .mapped(ScaleTraining::Authored)
        .unwrap();
    let p = plot(data())
        .aes(aes().x("x").y("y"))
        .layer(points().value_scale(
            V::TextSize,
            source_expr("area").reduce(ExpressionReduce::Sum, true),
            scale,
        ))
        .build()
        .unwrap();
    let pre = prepared(&p);
    assert_eq!(pre.layers()[0].marks().len(), 2);
    for mark in pre.layers()[0].marks() {
        assert_eq!(mark.aesthetics[&V::TextSize], Value::Number(Number(5.)));
    }
}

#[test]
fn presentation_theme_preserves_resolved_alpha_width_and_line_type() {
    use chart_core::grammar::LineType;
    let p = plot(data())
        .aes(aes().x("x").y("y"))
        .layer(points().alpha(0.25))
        .theme(theme().style(style().mark(rgb(255, 0, 0))))
        .build()
        .unwrap();
    let f = frame(&p);
    assert!(f.scene().items().iter().any(|i| matches!(
        i.primitive, Primitive::Point { fill, .. } if fill.alpha == 64
    )));
    let layer = shape_line()
        .shape_value(A::StrokeWidth, 2.)
        .line_type(LineType::Dashed);
    let handle = layer.handle().unwrap();
    let p = plot(data())
        .aes(aes().x("x").y("y"))
        .layer(layer)
        .theme(theme().layer(handle, style().stroke_width(9.).dashes(vec![1., 1.])))
        .build()
        .unwrap();
    let f = frame(&p);
    assert!(f.scene().items().iter().any(|i| matches!(
        &i.primitive, Primitive::ShapePath { stroke: Some(stroke), dashes, .. }
            if stroke.width == 2. && dashes == &[8., 8.]
    )));
}

#[test]
fn manual_paints_alpha_and_linewidth_match_pinned_ggplot_builds() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/aesthetics.json"
    ))
    .unwrap();
    let columns = |id: &str| {
        reference["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap()["layers"][0]["columns"]
            .clone()
    };
    let data = Data::columns()
        .column("x", [1., 2., 3., 4.])
        .column("y", [1., 2., 1., 2.])
        .column("a", categorical(["A", "B", "A", "B"]))
        .column("b", categorical(["B", "A", "A", "B"]))
        .column("alpha", [0.2, 0.4, 0.6, 0.8])
        .column("width", [0.5, 1., 1.5, 2.])
        .build()
        .unwrap();
    let author = |layer| {
        plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(
                aes()
                    .x("x")
                    .y("y")
                    .fill("a")
                    .fill_scale("inside")
                    .stroke("b")
                    .stroke_scale("outside"),
            )
            .scale(
                color_discrete("inside")
                    .domain(["A", "B"])
                    .palette(vec![rgb(255, 0, 0), rgb(0, 0, 255)]),
            )
            .scale(
                color_discrete("outside")
                    .domain(["A", "B"])
                    .palette(vec![rgb(0, 0, 0), rgb(0, 128, 0)]),
            )
            .layer(layer)
            .build()
            .unwrap()
    };
    for (id, layer, channels) in [
        ("independent_paints", points(), vec!["fill", "colour"]),
        (
            "constant_fill",
            points().fill(rgb(18, 52, 86)),
            vec!["fill"],
        ),
        (
            "constant_stroke",
            points().stroke(rgb(18, 52, 86)),
            vec!["colour"],
        ),
    ] {
        let p = author(layer);
        let pre = prepared(&p);
        let expected = columns(id);
        for channel in channels {
            for (mark, expected) in pre.layers()[0]
                .marks()
                .iter()
                .zip(expected[channel].as_array().unwrap())
            {
                let actual = if channel == "fill" {
                    mark.style.fill
                } else {
                    mark.style.stroke
                }
                .unwrap();
                let expected = chart_core::color::Paint::from_css(expected.as_str().unwrap())
                    .unwrap()
                    .resolve();
                assert_eq!(actual, expected, "{id}/{channel}");
            }
        }
    }
    let color = chart_core::scene::Color {
        red: 255,
        green: 0,
        blue: 0,
        alpha: 128,
    };
    let p = author(points().fill(color).shape_value(A::Alpha, "alpha"));
    let pre = prepared(&p);
    for (mark, expected) in pre.layers()[0]
        .marks()
        .iter()
        .zip(columns("embedded_alpha")["alpha"].as_array().unwrap())
    {
        assert_eq!(
            mark.style.fill.unwrap().alpha,
            (expected.as_f64().unwrap() * 255.).round_ties_even() as u8
        );
    }
    for (id, layer) in [
        ("linewidth", line().shape_value(A::StrokeWidth, "width")),
        (
            "constant_linewidth",
            line().shape_value(A::StrokeWidth, "width").linewidth(2.),
        ),
    ] {
        let p = plot(data.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y").group_all())
            .layer(layer)
            .build()
            .unwrap();
        let pre = prepared(&p);
        for (mark, expected) in pre.layers()[0]
            .marks()
            .iter()
            .zip(columns(id)["linewidth"].as_array().unwrap())
        {
            assert_eq!(mark.style.stroke_width, expected.as_f64().unwrap(), "{id}");
        }
    }
}

#[test]
fn reference_point_sizes_include_zero_and_match_r_graphics_parameters() {
    use chart_core::{
        grammar::{PreparedGeometry, ValueAesthetic},
        interpolate::{Number, Value},
    };
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/point-sizes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 9);
    for case in cases {
        let sizes = case["size"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect::<Vec<_>>();
        let p = plot(
            Data::columns()
                .column(
                    "x",
                    (0..sizes.len())
                        .map(|i| 0.25 + 3.5 * i as f64 / (sizes.len() - 1) as f64)
                        .collect::<Vec<_>>(),
                )
                .column("y", vec![2.; sizes.len()])
                .column("size", sizes.clone())
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(
            points()
                .linewidth(case["stroke"].as_f64().unwrap())
                .shape_value(A::Size, "size")
                .aesthetic_value(
                    ValueAesthetic::Shape,
                    Value::Number(Number(case["shape"].as_f64().unwrap())),
                ),
        )
        .build()
        .unwrap();
        let pre = prepared(&p);
        let layer = &pre.layers()[0];
        assert_eq!(layer.invalid_geometry(), 0);
        assert_eq!(layer.marks().len(), case["rows"].as_u64().unwrap() as usize);
        for (i, mark) in layer.marks().iter().enumerate() {
            assert_eq!(mark.style.radius, sizes[i]);
            let PreparedGeometry::ShapePath { geometry, .. } = &mark.geometry else {
                panic!()
            };
            let expected_points = case["fontsize"][i].as_f64().unwrap() * 0.375;
            if expected_points <= 0. {
                assert!(geometry.commands().is_empty());
            } else {
                let radius = geometry
                    .commands()
                    .iter()
                    .find_map(|c| match c {
                        chart_core::path::Command::Arc { radius, .. } => Some(*radius),
                        _ => None,
                    })
                    .unwrap();
                assert!(
                    (radius * 72. / 25.4 - expected_points).abs() < 1e-10,
                    "{case}"
                );
            }
        }
        let f = frame(&p);
        for item in f.scene().items().iter().filter(|item| item.layer.is_some()) {
            if let Primitive::ShapePath {
                stroke: Some(stroke),
                ..
            } = &item.primitive
            {
                let expected_lwd = case["lwd"].as_array().unwrap()[0].as_f64().unwrap();
                let expected_lwd = if expected_lwd == 0. {
                    case["pdf_widths"][0].as_f64().unwrap() * 96. / 72.
                } else {
                    expected_lwd
                };
                assert!(
                    (stroke.width - expected_lwd).abs() < 1e-10,
                    "{case}: width {} expected {}",
                    stroke.width,
                    expected_lwd
                );
            }
        }
        let json = p.to_json().unwrap();
        assert_eq!(Plot::from_json(&json).unwrap().to_json().unwrap(), json);
    }
}

#[test]
fn zero_constant_point_size_retains_row_and_legacy_validation() {
    let data = Data::columns()
        .column("x", [1.])
        .column("y", [1.])
        .build()
        .unwrap();
    let p = plot(data.clone())
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points().size(0.).linewidth(0.5))
        .build()
        .unwrap();
    let before = prepared(&p);
    assert_eq!(before.layers()[0].marks().len(), 1);
    assert_eq!(before.layers()[0].invalid_geometry(), 0);
    assert_eq!(before.layers()[0].marks()[0].style.radius, 0.);
    let mut canonical = p.definition().clone();
    canonical.layers[0].grammar = None;
    let direct = Compiler::new()
        .prepare(
            &canonical,
            &p.source(),
            &ChartState::default(),
            p.compile_limits(),
        )
        .unwrap();
    assert_eq!(
        direct.layers()[0].marks()[0].style.units,
        Some(chart_core::grammar::AestheticUnits::Millimeters)
    );

    assert!(
        plot(data.clone())
            .aes(aes().x("x").y("y"))
            .layer(points().radius(0.))
            .build()
            .is_err()
    );
    let negative = plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y("y"))
        .layer(points().size(-1.).linewidth(2.))
        .build()
        .unwrap();
    assert_eq!(prepared(&negative).layers()[0].marks().len(), 1);
}

#[test]
fn implicit_reference_circles_match_device_radii_including_empty_glyphs() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/point-sizes.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["shape"] == 16)
    {
        let sizes: Vec<_> = case["size"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let p = plot(
            Data::columns()
                .column("x", vec![2.; sizes.len()])
                .column("size", sizes)
                .build()
                .unwrap(),
        )
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x("x").y(2.))
        .layer(
            points()
                .linewidth(case["stroke"].as_f64().unwrap())
                .shape_value(A::Size, "size"),
        )
        .build()
        .unwrap();
        for units in [Units::Points, Units::LogicalPixels] {
            let f = frame_units(&p, units);
            let actual: Vec<_> = f
                .scene()
                .items()
                .iter()
                .filter(|i| i.layer.is_some())
                .filter_map(|i| match i.primitive {
                    Primitive::Point { radius, .. } => Some(radius),
                    _ => None,
                })
                .collect();
            let expected: Vec<_> = case["fontsize"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() * 0.375)
                .filter(|v| *v > 0.)
                .collect();
            assert_eq!(actual.len(), expected.len(), "{case}");
            for (actual, expected) in actual.into_iter().zip(expected) {
                let points = actual
                    * if units == Units::Points {
                        1.
                    } else {
                        72. / 96.
                    };
                assert!(
                    (points - expected).abs() < 1e-10,
                    "{case}: {points} != {expected}"
                );
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 3);
}

#[test]
fn retained_empty_reference_glyphs_are_not_an_empty_population() {
    use chart_core::{
        interpolate::{Number, Value},
        layout::LayoutStatus,
    };
    for empty in [false, true] {
        for explicit in [false, true] {
            let xs = if empty { vec![] } else { vec![1., 2.] };
            let data = Data::columns()
                .column("size", vec![-2.; xs.len()])
                .column("x", xs)
                .build()
                .unwrap();
            let mut layer = points().linewidth(0.5).shape_value(A::Size, "size");
            if explicit {
                layer = layer.aesthetic_value(
                    chart_core::grammar::ValueAesthetic::Shape,
                    Value::Number(Number(19.)),
                );
            }
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(2.))
                .layer(layer)
                .build()
                .unwrap();
            let f = frame(&p);
            assert_eq!(
                f.status(),
                if empty {
                    LayoutStatus::NoData
                } else {
                    LayoutStatus::Ready
                }
            );
            assert_eq!(
                f.scene().items().iter().any(
                    |i| matches!(&i.primitive, Primitive::Text { text, .. } if text == "No data")
                ),
                empty
            );
            assert!(!f.scene().items().iter().any(|i| i.layer.is_some()));
        }
    }
}

#[test]
fn reference_linewidths_match_r_device_units_and_zero_hairlines() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/line-widths.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    let mut checked = 0;
    for case in cases {
        let kind = case["kind"].as_str().unwrap();
        // Polygon geometry is owned by GG-07; its independently recorded device
        // widths remain in the fixture without claiming that geometry exists.
        if kind == "polygon" {
            continue;
        }
        let width = case["linewidth"].as_f64().unwrap();
        for mapped in [false, true] {
            let data = Data::columns()
                .column("x", [1., 2., 3.])
                .column("y", [1., 2., 1.])
                .column("xmax", [1.2, 2.2, 3.2])
                .column("ymax", [1.5, 2.5, 1.5])
                .column("width", [width; 3])
                .build()
                .unwrap();
            let layer = match kind {
                "segment" => rule(),
                "rect" => rectangle(),
                "line" => line(),
                "path" => line().order(chart_core::grammar::LineOrder::Authored),
                _ => unreachable!(),
            }
            .stroke(rgb(0, 0, 0));
            let layer = if mapped {
                layer.shape_value(A::StrokeWidth, "width")
            } else {
                layer.linewidth(width)
            };
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y("y").x2("xmax").y2("ymax"))
                .layer(layer)
                .build()
                .unwrap();
            let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
            for units in [Units::LogicalPixels, Units::Points] {
                let f = frame_units(&p, units);
                let widths: Vec<_> = f
                    .scene()
                    .items()
                    .iter()
                    .filter(|i| i.layer.is_some())
                    .filter_map(|i| match &i.primitive {
                        Primitive::Rule { stroke, .. }
                        | Primitive::Path { stroke, .. }
                        | Primitive::DashedPath { stroke, .. } => Some(stroke.width),
                        Primitive::ShapePath { stroke, .. }
                        | Primitive::VectorPath { stroke, .. } => stroke.map(|s| s.width),
                        _ => None,
                    })
                    .collect();
                assert!(!widths.is_empty(), "{case} mapped={mapped}");
                let expected = if width == 0. {
                    case["pdf_widths"][0].as_f64().unwrap()
                        * if units == Units::LogicalPixels {
                            96. / 72.
                        } else {
                            1.
                        }
                } else {
                    case["lwd"][0].as_f64().unwrap()
                        * if units == Units::Points {
                            72. / 96.
                        } else {
                            1.
                        }
                };
                for got in widths {
                    assert!(
                        (got - expected).abs() < 2e-12,
                        "{case} {units:?}: {got} != {expected}"
                    );
                }
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 96);
    for profile in [Profile::LibraryV1, Profile::Ggplot2_4_0_3] {
        let p = plot(data())
            .profile(profile)
            .aes(aes().x("x").y(1.).x2("x").y2(2.))
            .layer(
                rule()
                    .linewidth(2.)
                    .aesthetic_units(chart_core::grammar::AestheticUnits::Destination),
            )
            .build()
            .unwrap();
        for item in frame(&p)
            .scene()
            .items()
            .iter()
            .filter(|i| i.layer.is_some())
        {
            if let Primitive::Rule { stroke, .. } = item.primitive {
                assert_eq!(stroke.width, 2.);
            }
        }
    }
}
