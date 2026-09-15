//! FIX-GG05: complete default sampled ramps and source-normalized key positions.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision,
    color::Paint,
    layout::{LaidOutChart, LayoutRequest, layout},
    prelude::*,
    scene::{Color, GradientDirection, PathCommand, Primitive},
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
};
use serde_json::Value as Json;
#[path = "../../../examples/common/ggplot_colorsteps_boundary_fixtures.rs"]
mod boundary_fixtures;
#[path = "../../../examples/common/ggplot_colorbar_fixtures.rs"]
mod fixtures;
#[path = "../../../examples/common/ggplot_colorsteps_fixtures.rs"]
mod steps_fixtures;
use fixtures::{author, scale};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, request: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(request.text.len() as f64 * 5., 8., 2.)
    }
}
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-layout.json"
    ))
    .unwrap()
}
fn paint(value: &Json) -> Color {
    Paint::from_css(value.as_str().unwrap()).unwrap().resolve()
}
fn request(height: f64) -> LayoutRequest {
    let mut r = LayoutRequest::new(
        Rect::new(0., 0., 600., height).unwrap(),
        Units::Points,
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
fn draw(p: &Plot, r: &LayoutRequest) -> ChartResult<LaidOutChart> {
    layout(p.chart().unwrap().prepare().unwrap(), r, &Metrics)
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 3e-12, "{a} != {b}");
}
#[test]
fn source_ramps_and_key_positions_paint_in_single_collected_and_local_facets() {
    let fixture = fixture();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 3);
    for case in fixture["cases"].as_array().unwrap() {
        let colors = case["result"]["decor_colors"].as_array().unwrap();
        assert_eq!(colors.len(), 300);
        for channel in ["color", "fill", "stroke"] {
            for facet in ["single", "collected", "local"] {
                let p = author(case["palette"].as_str().unwrap(), channel, facet, false);
                let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
                let frame = draw(&p, &request(360.)).unwrap();
                let bars = if facet == "local" { 2 } else { 1 };
                let gradients = frame
                    .scene()
                    .items()
                    .iter()
                    .filter_map(|item| {
                        if let Primitive::SampledGradientRectangle {
                            bounds,
                            direction,
                            colors,
                            ..
                        } = &item.primitive
                        {
                            Some((*bounds, *direction, colors))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                assert_eq!(gradients.len(), bars, "{channel} {facet} {case}");
                assert_eq!(frame.scene().wire_version(), 19);
                for (bounds, direction, samples) in gradients {
                    assert_eq!(direction, GradientDirection::Vertical);
                    assert_eq!(samples.len(), 300);
                    for (a, b) in samples.iter().rev().zip(colors) {
                        assert_eq!(*a, paint(b));
                    }
                    let bottom = bounds.max_y();
                    let height = bounds.height();
                    let x = bounds.max_x() + request(360.).label_gap;
                    for (label, value) in case["result"]["labels"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .zip(case["result"]["values"].as_array().unwrap())
                    {
                        let actual = frame
                            .scene()
                            .items()
                            .iter()
                            .find_map(|item| match &item.primitive {
                                Primitive::Text { origin, text, .. }
                                    if text == label.as_str().unwrap()
                                        && (origin.x() - x).abs() < 1e-10 =>
                                {
                                    Some(origin.y() - 3.)
                                }
                                _ => None,
                            })
                            .expect("every source key is painted next to its bar");
                        close((bottom - actual) / height, value.as_f64().unwrap());
                    }
                }
            }
        }
    }
}
#[test]
fn hidden_guides_and_small_destinations_keep_marks_and_report_pressure() {
    let visible = author("asymmetric", "fill", "single", false);
    let hidden = visible
        .edit()
        .scale(color_mapped("v", scale("asymmetric", true)))
        .build()
        .unwrap();
    let a = visible.chart().unwrap().prepare().unwrap();
    let b = hidden.chart().unwrap().prepare().unwrap();
    assert_eq!(a.layers()[0].marks(), b.layers()[0].marks());
    let frame = draw(&hidden, &request(360.)).unwrap();
    assert!(
        !frame
            .scene()
            .items()
            .iter()
            .any(|item| matches!(item.primitive, Primitive::SampledGradientRectangle { .. }))
    );
    let small = draw(&visible, &request(80.)).unwrap();
    assert!(
        small
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("Legend pressure"))
    );
    let mut limited = request(360.);
    limited.limits.max_path_commands = 100;
    assert!(draw(&visible, &limited).is_err());
}

#[test]
fn authored_raster_sample_counts_match_reference_keys_and_paint() {
    use chart_core::{interpolate::Number, scales::GgplotColorbarOptions};
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-boundaries.json"
    ))
    .unwrap();
    let fractional: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-fractional.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fractional["cases"].as_array().unwrap())
        .filter(|c| {
            c["display"] == "raster" && c["reverse"] == false && c["direction"] == "vertical"
        })
    {
        let palette = case["palette"].as_str().unwrap();
        for facet in ["single", "collected", "local"] {
            let p = author(palette, "fill", facet, false)
                .edit()
                .scale(color_mapped(
                    "v",
                    scale(palette, false).with_colorbar_options(GgplotColorbarOptions {
                        nbin: Some(Number(case["nbin"].as_f64().unwrap())),
                        ..Default::default()
                    }),
                ))
                .build()
                .unwrap();
            let wire = p.to_json().unwrap();
            assert_eq!(serde_json::from_str::<Json>(&wire).unwrap()["version"], 64);
            let mut stale: Json = serde_json::from_str(&wire).unwrap();
            stale["version"] = 63.into();
            assert!(Plot::from_json(&stale.to_string()).is_err());
            let p = Plot::from_json(&wire).unwrap();
            let frame = draw(&p, &request(360.)).unwrap();
            let colors = case["result"]["decor_colors"].as_array().unwrap();
            let expected: Vec<_> = colors.iter().rev().map(paint).collect();
            let bars: Vec<_> = frame
                .scene()
                .items()
                .iter()
                .filter_map(|item| match &item.primitive {
                    Primitive::SampledGradientRectangle { bounds, colors, .. } => {
                        assert_eq!(colors, &expected);
                        Some(*bounds)
                    }
                    Primitive::Rectangle { bounds, fill }
                        if expected.len() == 1
                            && bounds.width() == 1.5 * request(360.).font_size
                            && bounds.height() == 10. * request(360.).font_size =>
                    {
                        assert_eq!(*fill, expected[0]);
                        Some(*bounds)
                    }
                    _ => None,
                })
                .collect();
            assert_eq!(bars.len(), if facet == "local" { 2 } else { 1 });
            for bounds in bars {
                let x = bounds.max_x() + request(360.).label_gap;
                let labels: Vec<_> = frame
                    .scene()
                    .items()
                    .iter()
                    .filter_map(|item| match &item.primitive {
                        Primitive::Text { origin, text, .. } if (origin.x() - x).abs() < 1e-10 => {
                            Some((text, origin.y() - 3.))
                        }
                        _ => None,
                    })
                    .collect();
                let positions = case["result"]["values"].as_array().unwrap();
                if case["nbin"] == 0 {
                    assert!(labels.is_empty());
                } else {
                    assert_eq!(labels.len(), positions.len());
                    for ((label, center), (expected_label, position)) in labels.iter().zip(
                        case["result"]["labels"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .zip(positions),
                    ) {
                        assert_eq!(label.as_str(), expected_label.as_str().unwrap());
                        close(
                            (bounds.max_y() - center) / bounds.height(),
                            position.as_f64().unwrap(),
                        );
                    }
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 81);
}

#[test]
fn raster_sampling_is_demanded_by_keys_including_empty_populations() {
    use chart_core::{
        interpolate::Number,
        scales::{
            GgplotColorbarOptions, GgplotContinuousGuide, GgplotGuideLabels, GgplotScaleGuide,
        },
    };
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-demand.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 60);
    for case in cases {
        let empty = case["population"] == "empty";
        let values = if empty { vec![] } else { vec![-2., 0., 3., 8.] };
        let data = Data::columns().column("v", values).build().unwrap();
        let guide = if case["hidden"] == true {
            GgplotScaleGuide::Hidden
        } else {
            GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
                breaks: Some(if case["selection"] == "no_breaks" {
                    vec![]
                } else {
                    [-2., 0., 3., 8.].map(Number).to_vec()
                }),
                labels: if case["selection"] == "no_labels" {
                    GgplotGuideLabels::Hidden
                } else {
                    GgplotGuideLabels::Automatic
                },
                ..Default::default()
            })
        };
        let descriptor = scale("ordinary", false)
            .with_guide(guide)
            .unwrap()
            .with_colorbar_options(GgplotColorbarOptions {
                nbin: Some(Number(case["nbin"].as_f64().unwrap_or(f64::NAN))),
                ..Default::default()
            });
        let p = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x(1.).y(1.).fill("v").fill_scale("v"))
            .scale(color_mapped("v", descriptor))
            .layer(points())
            .build()
            .unwrap();
        let mut chart = p.chart().unwrap();
        let result = chart.prepare();
        if case["result"]["error"].is_string() {
            assert!(result.is_err(), "{case}");
        } else {
            let prepared = result.unwrap();
            let frame = layout(prepared, &request(360.), &Metrics).unwrap();
            let bars = frame
                .scene()
                .items()
                .iter()
                .filter(|item| match &item.primitive {
                    Primitive::SampledGradientRectangle { .. } => true,
                    Primitive::Rectangle { bounds, .. } => {
                        bounds.width() == 1.5 * request(360.).font_size
                            && bounds.height() == 10. * request(360.).font_size
                    }
                    _ => false,
                })
                .count();
            assert_eq!(
                bars,
                case["result"]["guide_count"].as_u64().unwrap() as usize,
                "{case}"
            );
        }
    }
    let p = author("ordinary", "fill", "single", false)
        .edit()
        .scale(color_mapped(
            "v",
            scale("ordinary", false).with_colorbar_options(GgplotColorbarOptions {
                nbin: Some(Number(f64::MAX)),
                ..Default::default()
            }),
        ))
        .build()
        .unwrap();
    assert!(p.chart().unwrap().prepare().is_err());
}

#[test]
fn constant_limits_keep_unique_zero_count_sample_and_source_keys() {
    use chart_core::{
        interpolate::Number,
        scales::{
            GgplotColorbarOptions, GgplotContinuousGuide, GgplotOob, GgplotScaleGuide,
            GgplotScalePolicy,
        },
    };
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-constant.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 12);
    for case in cases {
        let descriptor = scale(case["palette"].as_str().unwrap(), false)
            .with_ggplot(GgplotScalePolicy::Continuous {
                limits: Some([Some(Number(3.)), Some(Number(3.))]),
                empty_population: false,
                nonfinite_population: false,
                oob: GgplotOob::Censor,
            })
            .unwrap()
            .with_guide(GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
                breaks: Some(vec![Number(3.)]),
                ..Default::default()
            }))
            .unwrap()
            .with_colorbar_options(GgplotColorbarOptions {
                nbin: Some(Number(case["nbin"].as_f64().unwrap())),
                ..Default::default()
            });
        let p = author(case["palette"].as_str().unwrap(), "color", "single", false)
            .edit()
            .scale(color_mapped("v", descriptor))
            .build()
            .unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let samples = &prepared.layers()[0].color_legend().unwrap().colorbar;
        let expected = case["result"]["decor_colors"].as_array().unwrap();
        assert_eq!(samples.len(), expected.len(), "{case}");
        for (sample, color) in samples.iter().zip(expected) {
            assert_eq!(sample.value, Number(3.));
            assert_eq!(sample.color, paint(color));
        }
        let frame = draw(&p, &request(360.)).unwrap();
        let ticks = frame
            .scene()
            .items()
            .iter()
            .filter(
                |i| matches!(&i.primitive, Primitive::Path { commands, .. } if commands.len()==4),
            )
            .count();
        assert_eq!(ticks, usize::from(case["nbin"] != 0));
    }
}

fn presentation_bars(
    frame: &LaidOutChart,
    direction: GradientDirection,
    colors: &[Color],
) -> Vec<Rect> {
    frame
        .scene()
        .items()
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::SampledGradientRectangle {
                bounds,
                direction: actual,
                colors: actual_colors,
                ..
            } => {
                assert_eq!(*actual, direction);
                assert_eq!(actual_colors, colors);
                Some(*bounds)
            }
            Primitive::Rectangle { bounds, fill }
                if colors.len() == 1
                    && ((bounds.width() / bounds.height() - 20. / 3.).abs() < 1e-10
                        || (bounds.height() / bounds.width() - 20. / 3.).abs() < 1e-10) =>
            {
                assert_eq!(*fill, colors[0]);
                Some(*bounds)
            }
            _ => None,
        })
        .collect()
}
fn presentation_ticks(frame: &LaidOutChart, bar: Rect, direction: GradientDirection) -> Vec<f64> {
    frame
        .scene()
        .items()
        .iter()
        .filter_map(|item| {
            let Primitive::Path { commands, .. } = &item.primitive else {
                return None;
            };
            if commands.len() != 4 {
                return None;
            }
            let (PathCommand::MoveTo(a), PathCommand::MoveTo(b)) = (&commands[0], &commands[2])
            else {
                return None;
            };
            let position = match direction {
                GradientDirection::Vertical
                    if a.x() == bar.origin().x() && b.x() > a.x() && b.x() < bar.max_x() =>
                {
                    (bar.max_y() - a.y()) / bar.height()
                }
                GradientDirection::Horizontal
                    if a.y() == bar.origin().y() && b.y() > a.y() && b.y() < bar.max_y() =>
                {
                    (a.x() - bar.origin().x()) / bar.width()
                }
                _ => return None,
            };
            ((-1e-12..=1. + 1e-12).contains(&position)).then_some(position)
        })
        .collect()
}
fn presentation_labels(
    frame: &LaidOutChart,
    bar: Rect,
    direction: GradientDirection,
) -> Vec<(String, f64)> {
    frame
        .scene()
        .items()
        .iter()
        .filter_map(|item| {
            let Primitive::Text { origin, text, .. } = &item.primitive else {
                return None;
            };
            if text.parse::<f64>().is_err() {
                return None;
            }
            let position = match direction {
                GradientDirection::Vertical
                    if (origin.x() - bar.max_x() - request(360.).label_gap).abs() < 1e-10 =>
                {
                    (bar.max_y() - (origin.y() - 3.)) / bar.height()
                }
                GradientDirection::Horizontal
                    if (origin.y() - bar.max_y() - request(360.).label_gap - 8.).abs() < 1e-10 =>
                {
                    (origin.x() + text.len() as f64 * 2.5 - bar.origin().x()) / bar.width()
                }
                _ => return None,
            };
            ((-1e-12..=1. + 1e-12).contains(&position)).then_some((text.clone(), position))
        })
        .collect()
}

#[test]
fn raster_direction_and_reversal_match_reference_without_changing_marks() {
    use chart_core::{interpolate::Number, scales::GgplotColorbarOptions};
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-boundaries.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["display"] == "raster")
    {
        let direction = if case["direction"] == "horizontal" {
            GradientDirection::Horizontal
        } else {
            GradientDirection::Vertical
        };
        let palette = case["palette"].as_str().unwrap();
        let mut colors: Vec<_> = case["result"]["decor_colors"]
            .as_array()
            .unwrap()
            .iter()
            .map(paint)
            .collect();
        if direction == GradientDirection::Vertical {
            colors.reverse();
        }
        for channel in ["color", "fill", "stroke"] {
            for facet in ["single", "collected", "local"] {
                let original = author(palette, channel, facet, false);
                let p = original
                    .edit()
                    .scale(color_mapped(
                        "v",
                        scale(palette, false).with_colorbar_options(GgplotColorbarOptions {
                            nbin: Some(Number(case["nbin"].as_f64().unwrap())),
                            direction: Some(direction),
                            reverse: case["reverse"].as_bool().unwrap(),
                            ..Default::default()
                        }),
                    ))
                    .build()
                    .unwrap();
                let wire = p.to_json().unwrap();
                let mut stale: Json = serde_json::from_str(&wire).unwrap();
                assert_eq!(stale["version"], 65);
                stale["version"] = 64.into();
                assert!(Plot::from_json(&stale.to_string()).is_err());
                let p = Plot::from_json(&wire).unwrap();
                let a = original.chart().unwrap().prepare().unwrap();
                let b = p.chart().unwrap().prepare().unwrap();
                for (a, b) in a.layers().iter().zip(b.layers()) {
                    assert_eq!(a.marks(), b.marks());
                }
                let frame = draw(&p, &request(360.)).unwrap();
                let bars = presentation_bars(&frame, direction, &colors);
                assert_eq!(bars.len(), if facet == "local" { 2 } else { 1 });
                for bar in bars {
                    close(
                        if direction == GradientDirection::Horizontal {
                            bar.width()
                        } else {
                            bar.height()
                        },
                        10. * request(360.).font_size,
                    );
                    let ticks = presentation_ticks(&frame, bar, direction);
                    let expected: Vec<_> = case["result"]["values"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter_map(Json::as_f64)
                        .collect();
                    assert_eq!(ticks.len(), expected.len());
                    for (a, b) in ticks.iter().zip(&expected) {
                        close(*a, *b);
                    }
                    let labels = presentation_labels(&frame, bar, direction);
                    assert_eq!(labels.len(), expected.len());
                    for ((label, position), (text, expected)) in labels.iter().zip(
                        case["result"]["labels"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .zip(&expected),
                    ) {
                        assert_eq!(label, text.as_str().unwrap());
                        close(*position, *expected);
                    }
                }
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 432);
}

#[test]
fn reference_tick_limits_are_independent_of_labels_and_nonfinite_positions() {
    use chart_core::{
        interpolate::Number,
        scales::{
            GgplotColorbarOptions, GgplotContinuousGuide, GgplotGuideLabels, GgplotScaleGuide,
        },
    };
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-presentation.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 128);
    for case in cases {
        let direction = if case["direction"] == "horizontal" {
            GradientDirection::Horizontal
        } else {
            GradientDirection::Vertical
        };
        let guide = GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
            breaks: Some([-2., 0., 3., 8.].map(Number).to_vec()),
            labels: if case["labels"] == "hidden" {
                GgplotGuideLabels::Hidden
            } else {
                GgplotGuideLabels::Automatic
            },
            ..Default::default()
        });
        let descriptor = scale("ordinary", false)
            .with_guide(guide)
            .unwrap()
            .with_colorbar_options(GgplotColorbarOptions {
                nbin: Some(Number(case["nbin"].as_f64().unwrap())),
                direction: Some(direction),
                reverse: case["reverse"].as_bool().unwrap(),
                draw_lower_limit: case["lower"].as_bool().unwrap(),
                draw_upper_limit: case["upper"].as_bool().unwrap(),
                ..Default::default()
            });
        let p = author("ordinary", "color", "single", false)
            .edit()
            .scale(color_mapped("v", descriptor))
            .build()
            .unwrap();
        let frame = draw(&p, &request(360.)).unwrap();
        let mut colors: Vec<_> = case["result"]["decor_colors"]
            .as_array()
            .unwrap()
            .iter()
            .map(paint)
            .collect();
        if direction == GradientDirection::Vertical {
            colors.reverse();
        }
        let bars = presentation_bars(&frame, direction, &colors);
        assert_eq!(bars.len(), 1, "{case}");
        let bar = bars[0];
        let ticks = presentation_ticks(&frame, bar, direction);
        let expected: Vec<_> = case["result"]["tick_positions"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Json::as_f64)
            .collect();
        assert_eq!(ticks.len(), expected.len(), "{case}");
        for (a, b) in ticks.iter().zip(expected) {
            close(*a, b);
        }
        let labels = presentation_labels(&frame, bar, direction);
        let expected: Vec<_> = case["result"]["keys"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["result"]["labels"].as_array().unwrap())
            .filter_map(|(v, label)| v.as_f64().map(|v| (label.as_str().unwrap(), v)))
            .collect();
        assert_eq!(labels.len(), expected.len(), "{case}");
        for ((label, position), (text, expected)) in labels.iter().zip(expected) {
            assert_eq!(label, text);
            close(*position, expected);
        }
    }
}

#[test]
fn horizontal_endpoint_labels_fit_and_tiny_destinations_report_pressure() {
    use chart_core::scales::GgplotColorbarOptions;
    let p = author("ordinary", "color", "single", false)
        .edit()
        .scale(color_mapped(
            "v",
            scale("ordinary", false).with_colorbar_options(GgplotColorbarOptions {
                direction: Some(GradientDirection::Horizontal),
                ..Default::default()
            }),
        ))
        .build()
        .unwrap();
    let frame = draw(&p, &request(360.)).unwrap();
    let (bar, clip) = frame
        .scene()
        .items()
        .iter()
        .find_map(|item| match item.primitive {
            Primitive::SampledGradientRectangle {
                bounds,
                direction: GradientDirection::Horizontal,
                ..
            } => Some((bounds, item.clip.unwrap())),
            _ => None,
        })
        .unwrap();
    assert!(bar.origin().x() > clip.origin().x());
    let mut labels = 0;
    for item in frame.scene().items() {
        if let Primitive::Text { origin, text, .. } = &item.primitive
            && item.clip == Some(clip)
            && text.parse::<f64>().is_ok()
        {
            assert!(origin.x() >= clip.origin().x());
            assert!(origin.x() + text.len() as f64 * 5. <= clip.max_x());
            labels += 1;
        }
    }
    assert_eq!(labels, 4);
    for width in [80., 10.] {
        let mut small = request(360.);
        small.bounds = Rect::new(0., 0., width, 360.).unwrap();
        let frame = draw(&p, &small).unwrap();
        assert!(
            frame
                .diagnostics()
                .iter()
                .any(|d| d.message.contains("Legend pressure"))
        );
        if width == 10. {
            assert!(
                !frame.scene().items().iter().any(|item| matches!(
                    item.primitive,
                    Primitive::SampledGradientRectangle { .. }
                ))
            );
        }
    }
}

#[test]
fn gradient_and_rectangle_displays_match_source_geometry_and_keep_mark_mapping() {
    use chart_core::{interpolate::Number, scales::*, scene::SampledGradientMode};
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-display.json"
    ))
    .unwrap();
    let boundaries: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-boundaries.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in source["cases"].as_array().unwrap().iter().chain(
        boundaries["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["display"] != "raster"),
    ) {
        let display = if case["display"] == "gradient" {
            GgplotColorbarDisplay::Gradient
        } else {
            GgplotColorbarDisplay::Rectangles
        };
        let direction = if case["direction"] == "horizontal" {
            GradientDirection::Horizontal
        } else {
            GradientDirection::Vertical
        };
        let horizontal = direction == GradientDirection::Horizontal;
        let palette = case["palette"].as_str().unwrap();
        let constant = case["constant"] == true;
        let expected: Vec<_> = case["result"]["decor_colors"]
            .as_array()
            .unwrap()
            .iter()
            .map(paint)
            .collect();
        for facet in ["single", "collected", "local"] {
            let mut descriptor = scale(palette, false);
            if constant {
                descriptor = descriptor
                    .with_ggplot(GgplotScalePolicy::Continuous {
                        limits: Some([Some(Number(2.)), Some(Number(2.))]),
                        empty_population: false,
                        nonfinite_population: false,
                        oob: GgplotOob::Censor,
                    })
                    .unwrap()
                    .with_guide(GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
                        breaks: Some(vec![Number(2.)]),
                        ..Default::default()
                    }))
                    .unwrap();
            }
            let original = author(palette, "color", facet, false)
                .edit()
                .scale(color_mapped("v", descriptor.clone()))
                .build()
                .unwrap();
            let p = original
                .edit()
                .scale(color_mapped(
                    "v",
                    descriptor.with_colorbar_options(GgplotColorbarOptions {
                        nbin: case["nbin"].as_f64().map(Number),
                        display,
                        direction: Some(direction),
                        reverse: case["reverse"].as_bool().unwrap(),
                        ..Default::default()
                    }),
                ))
                .build()
                .unwrap();
            let wire = p.to_json().unwrap();
            let mut stale: Json = serde_json::from_str(&wire).unwrap();
            assert_eq!(stale["version"], 66);
            stale["version"] = 65.into();
            assert!(Plot::from_json(&stale.to_string()).is_err());
            let p = Plot::from_json(&wire).unwrap();
            let prepared = p.chart().unwrap().prepare().unwrap();
            let before = original.chart().unwrap().prepare().unwrap();
            for (a, b) in before.layers().iter().zip(prepared.layers()) {
                assert_eq!(a.marks(), b.marks());
            }
            let samples = &prepared.layers()[0].color_legend().unwrap().colorbar;
            assert_eq!(
                samples.iter().map(|s| s.color).collect::<Vec<_>>(),
                expected
            );
            let frame = draw(&p, &request(360.)).unwrap();
            let colors: Vec<_> = if horizontal {
                expected.clone()
            } else {
                expected.iter().rev().copied().collect()
            };
            let colors = if constant && display == GgplotColorbarDisplay::Gradient {
                &colors[..1]
            } else {
                &colors[..]
            };
            let bars = presentation_bars(&frame, direction, colors);
            for item in frame.scene().items() {
                if let Primitive::SampledGradientRectangle { mode, colors, .. } = &item.primitive {
                    assert_eq!(frame.scene().wire_version(), 19);
                    if display == GgplotColorbarDisplay::Gradient {
                        assert_eq!(*mode, SampledGradientMode::Endpoints);
                        if let Some(stops) = case["result"]["bar"]["gradient"]["stops"].as_array() {
                            for ((position, _), expected) in mode.stops(colors).zip(stops) {
                                close(position, expected.as_f64().unwrap());
                            }
                        }
                    } else {
                        assert_eq!(*mode, SampledGradientMode::Steps);
                        let stops: Vec<_> = mode.stops(colors).collect();
                        assert_eq!(stops.len(), 2 * colors.len());
                        for (i, pair) in stops.chunks(2).enumerate() {
                            close(pair[0].0, i as f64 / colors.len() as f64);
                            close(pair[1].0, (i + 1) as f64 / colors.len() as f64);
                            assert_eq!(pair[0].1, colors[i]);
                            assert_eq!(pair[1].1, colors[i]);
                            if let Some(extent) = case["result"]["bar"]
                                [if horizontal { "width" } else { "height" }]
                            .as_array()
                            {
                                close(pair[1].0 - pair[0].0, extent[0].as_f64().unwrap());
                            }
                        }
                    }
                }
            }
            assert_eq!(bars.len(), if facet == "local" { 2 } else { 1 });
            let key_values = case["result"]["keys"]
                .as_array()
                .or_else(|| case["result"]["values"].as_array())
                .unwrap();
            let expected_keys: Vec<_> = key_values
                .iter()
                .filter_map(Json::as_f64)
                .filter(|v| v.is_finite())
                .collect();
            for bar in bars {
                let actual = presentation_ticks(&frame, bar, direction);
                assert_eq!(actual.len(), expected_keys.len(), "{case}");
                for (a, b) in actual.iter().zip(&expected_keys) {
                    close(*a, *b);
                }
                let labels = presentation_labels(&frame, bar, direction);
                assert_eq!(labels.len(), expected_keys.len());
                for ((_, a), b) in labels.iter().zip(&expected_keys) {
                    close(*a, *b);
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 528);
}

#[test]
fn colorbar_alpha_matches_source_without_changing_marks_or_keys() {
    use chart_core::{interpolate::Number, scales::*};
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-alpha.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in source["cases"].as_array().unwrap() {
        let palette = case["palette"].as_str().unwrap();
        let direction = if case["direction"] == "horizontal" {
            GradientDirection::Horizontal
        } else {
            GradientDirection::Vertical
        };
        let expected: Vec<_> = case["result"]["decor_colors"]
            .as_array()
            .unwrap()
            .iter()
            .map(paint)
            .collect();
        for facet in ["single", "collected", "local"] {
            let original = author(palette, "color", facet, false);
            let options = GgplotColorbarOptions {
                nbin: Some(Number(5.)),
                alpha: case["alpha"].as_f64().map(Number),
                display: match case["display"].as_str().unwrap() {
                    "gradient" => GgplotColorbarDisplay::Gradient,
                    "rectangles" => GgplotColorbarDisplay::Rectangles,
                    _ => GgplotColorbarDisplay::Raster,
                },
                direction: Some(direction),
                reverse: case["reverse"].as_bool().unwrap(),
                ..Default::default()
            };
            let p = original
                .edit()
                .scale(color_mapped(
                    "v",
                    scale(palette, false).with_colorbar_options(options),
                ))
                .build()
                .unwrap();
            let wire = p.to_json().unwrap();
            let mut stale: Json = serde_json::from_str(&wire).unwrap();
            if !case["alpha"].is_null() {
                assert_eq!(stale["version"], 67);
                stale["version"] = 66.into();
                assert!(Plot::from_json(&stale.to_string()).is_err());
            }
            let p = Plot::from_json(&wire).unwrap();
            let prepared = p.chart().unwrap().prepare().unwrap();
            let before = original.chart().unwrap().prepare().unwrap();
            for (a, b) in before.layers().iter().zip(prepared.layers()) {
                assert_eq!(a.marks(), b.marks());
            }
            assert_eq!(
                prepared.layers()[0]
                    .color_legend()
                    .unwrap()
                    .colorbar
                    .iter()
                    .map(|s| s.color)
                    .collect::<Vec<_>>(),
                expected,
                "{case}"
            );
            let frame = draw(&p, &request(360.)).unwrap();
            let colors = if direction == GradientDirection::Horizontal {
                expected.clone()
            } else {
                expected.iter().rev().copied().collect()
            };
            let bars = presentation_bars(&frame, direction, &colors);
            assert_eq!(bars.len(), if facet == "local" { 2 } else { 1 });
            for bar in bars {
                let ticks = presentation_ticks(&frame, bar, direction);
                let keys = case["result"]["keys"].as_array().unwrap();
                assert_eq!(ticks.len(), keys.len());
                for (a, b) in ticks.iter().zip(keys) {
                    close(*a, b.as_f64().unwrap());
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 288);
    for case in source["constructor_boundaries"].as_array().unwrap() {
        let alpha = match case["alpha"].as_str() {
            Some("Infinity") => f64::INFINITY,
            Some("-Infinity") => f64::NEG_INFINITY,
            Some("NaN") => f64::NAN,
            _ => case["alpha"].as_f64().unwrap(),
        };
        let hidden = case["hidden"].as_bool().unwrap();
        let result = author("ordinary", "color", "single", hidden)
            .edit()
            .scale(color_mapped(
                "v",
                scale("ordinary", hidden).with_colorbar_options(GgplotColorbarOptions {
                    alpha: Some(Number(alpha)),
                    ..Default::default()
                }),
            ))
            .build();
        assert_eq!(result.is_ok(), case["result"]["accepted"] == true, "{case}");
        if let Ok(p) = result {
            let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
            let prepared = p.chart().unwrap().prepare().unwrap();
            if !hidden {
                assert!(
                    prepared.layers()[0]
                        .color_legend()
                        .unwrap()
                        .colorbar
                        .iter()
                        .all(|s| s.color.alpha == 0)
                );
            }
        }
    }
}

#[test]
fn colorbar_alpha_preserves_missing_identity_and_replaces_transparent_paint() {
    use chart_core::{
        grammar::*,
        interpolate::{Number, Value},
        scales::*,
    };
    use std::sync::Arc;
    struct MissingPalette;
    impl CustomScalePalette for MissingPalette {
        fn descriptor(&self) -> ExtensionDescriptor {
            ExtensionDescriptor::batch("alpha.missing", Revision::new(1), true)
        }
        fn validate(&self, _: &Json) -> ChartResult<()> {
            Ok(())
        }
        fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
            let ScalePaletteDomain::Normalized(values) = input.domain else {
                panic!("normalized colors")
            };
            Ok(ScalePaletteOutput {
                values: Some(
                    values
                        .iter()
                        .map(|v| {
                            if v.0 < 0.25 {
                                Value::Missing
                            } else if v.0 < 0.75 {
                                Value::Text("#00000000".into())
                            } else {
                                Value::Text("grey50".into())
                            }
                        })
                        .collect(),
                ),
                names: None,
            })
        }
    }
    let mut registry = ExtensionRegistry::new();
    registry
        .register_scale_palette(Arc::new(MissingPalette))
        .unwrap();
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-alpha.json"
    ))
    .unwrap();
    for case in fixture["missing"].as_array().unwrap() {
        let alpha = match case["alpha"].as_str() {
            Some("Infinity") => Some(Number(f64::INFINITY)),
            Some("-Infinity") => Some(Number(f64::NEG_INFINITY)),
            _ => case["alpha"].as_f64().map(Number),
        };
        let mut descriptor = scale("ordinary", false)
            .with_palette_function(ScalePaletteOperation {
                operation: OperationRef {
                    id: "alpha.missing".into(),
                    version: Revision::new(1),
                },
                parameters: Json::Null,
            })
            .unwrap()
            .with_colorbar_options(GgplotColorbarOptions {
                alpha,
                nbin: Some(Number(3.)),
                ..Default::default()
            });
        descriptor.missing_paint_is_na = true;
        let mapped = MappedScale::new_with_registry(descriptor, &registry).unwrap();
        let legend = mapped
            .legend(
                chart_core::ScaleId::new(1),
                paint(&Json::String("#7f7f7f".into())),
            )
            .unwrap();
        let expected: Vec<_> = case["colors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                if v.is_null() {
                    Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,
                    }
                } else {
                    paint(v)
                }
            })
            .collect();
        assert_eq!(
            legend.colorbar.iter().map(|s| s.color).collect::<Vec<_>>(),
            expected,
            "{case}"
        );
    }
}

#[test]
fn default_even_colorsteps_paint_source_cells_and_keys() {
    use chart_core::{scales::*, scene::SampledGradientMode};
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorsteps-layout.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in source["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["even_steps"] == true && c["show_limits"] == false)
    {
        let direction = if case["direction"] == "horizontal" {
            GradientDirection::Horizontal
        } else {
            GradientDirection::Vertical
        };
        let mut cells: Vec<_> = case["result"]["decor"]["colour"]
            .as_array()
            .unwrap()
            .iter()
            .zip(case["result"]["decor"]["min"].as_array().unwrap())
            .zip(case["result"]["decor"]["max"].as_array().unwrap())
            .map(|((color, a), b)| (a.as_f64().unwrap().min(b.as_f64().unwrap()), paint(color)))
            .collect();
        cells.sort_by(|a, b| a.0.total_cmp(&b.0));
        let expected: Vec<_> = cells.into_iter().map(|(_, color)| color).collect();
        for facet in ["single", "collected", "local"] {
            let mut descriptor = steps_fixtures::steps_scale(
                case["family"].as_str().unwrap(),
                case["endpoints"].as_bool().unwrap(),
                false,
            )
            .with_colorbar_options(GgplotColorbarOptions {
                direction: Some(direction),
                reverse: case["reverse"].as_bool().unwrap(),
                ..Default::default()
            });
            if case["guide_kind"] == "default" {
                descriptor = descriptor
                    .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))
                    .unwrap();
            }
            let p = author("asymmetric", "color", facet, false)
                .edit()
                .scale(color_mapped("v", descriptor))
                .build()
                .unwrap();
            let p = Plot::from_json(&p.to_json().unwrap()).unwrap();
            let frame = draw(&p, &request(360.)).unwrap();
            let colors = if direction == GradientDirection::Horizontal {
                expected.clone()
            } else {
                expected.iter().rev().copied().collect()
            };
            let bars = presentation_bars(&frame, direction, &colors);
            assert_eq!(bars.len(), if facet == "local" { 2 } else { 1 }, "{case}");
            for item in frame.scene().items() {
                if let Primitive::SampledGradientRectangle { mode, .. } = &item.primitive {
                    assert_eq!(*mode, SampledGradientMode::Steps);
                }
            }
            for bar in bars {
                let keys = case["result"]["key"][".value"].as_array().unwrap();
                let ticks = presentation_ticks(&frame, bar, direction);
                assert_eq!(ticks.len(), keys.len());
                for (a, b) in ticks.iter().zip(keys) {
                    close(*a, b.as_f64().unwrap());
                }
                let labels = presentation_labels(&frame, bar, direction);
                assert_eq!(labels.len(), keys.len());
                for ((a, position), (label, key)) in labels.iter().zip(
                    case["result"]["key"][".label"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .zip(keys),
                ) {
                    assert_eq!(a, label.as_str().unwrap());
                    close(*position, key.as_f64().unwrap());
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 54);
}

#[test]
fn even_colorsteps_boundary_cells_keys_and_rejections_match_source() {
    use chart_core::scales::*;
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorsteps-boundaries.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in source["cases"].as_array().unwrap() {
        // The rejected binned NULL input has no authored enum variant.
        if case["family"] == "binned" && case["mode"] == "null" {
            continue;
        }
        let result = boundary_fixtures::boundary_scale(case).and_then(|s| {
            MappedScale::new(s.clone()).and_then(|m| {
                m.legend(
                    chart_core::ScaleId::new(1),
                    paint(&Json::String("#7f7f7f".into())),
                )
                .map(|l| (s, l))
            })
        });
        if case["result"]["error"].is_string() {
            if let Ok((descriptor, _)) = result {
                let p = fixtures::author_values("asymmetric", "color", "single", false, [2.; 8])
                    .edit()
                    .scale(color_mapped("v", descriptor))
                    .build()
                    .unwrap();
                assert!(
                    draw(&p, &request(360.)).is_err(),
                    "expected source draw rejection: {case}"
                );
            }
        } else {
            let (descriptor, legend) = result.unwrap_or_else(|e| panic!("{case}: {e}"));
            let colors: Vec<_> = case["result"]["decor"]["colour"]
                .as_array()
                .map(|v| v.iter().map(paint).collect())
                .unwrap_or_default();
            assert_eq!(
                legend
                    .colorsteps
                    .iter()
                    .map(|s| s.color)
                    .collect::<Vec<_>>(),
                colors,
                "{case}"
            );
            // Exercise the shared layout for ordinary data. Empty/constant semantics are
            // checked above against the scale's actual training population.
            if case["population"] == "ordinary" && !colors.is_empty() {
                let p = author("asymmetric", "color", "single", false)
                    .edit()
                    .scale(color_mapped("v", descriptor))
                    .build()
                    .unwrap();
                let frame = draw(&p, &request(360.)).unwrap();
                let bars = presentation_bars(
                    &frame,
                    GradientDirection::Vertical,
                    &colors.iter().rev().copied().collect::<Vec<_>>(),
                );
                assert_eq!(bars.len(), 1, "{case}");
                let keys = case["result"]["key"][".value"].as_array().unwrap();
                let ticks = presentation_ticks(&frame, bars[0], GradientDirection::Vertical);
                assert_eq!(ticks.len(), keys.len(), "{case}");
                for (a, b) in ticks.iter().zip(keys) {
                    close(*a, b.as_f64().unwrap());
                }
                let labels = presentation_labels(&frame, bars[0], GradientDirection::Vertical);
                assert_eq!(labels.len(), keys.len(), "{case}");
                for ((label, position), (expected, key)) in labels.iter().zip(
                    case["result"]["key"][".label"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .zip(keys),
                ) {
                    assert_eq!(label, expected.as_str().unwrap(), "{case}");
                    close(*position, key.as_f64().unwrap());
                }
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 57);
}

#[test]
fn default_binned_boundary_cells_reuse_palette_and_source_positions() {
    use chart_core::scales::*;
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorsteps-default-boundaries.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in source["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["population"] != "constant" && c["mode"] != "null")
    {
        let expected: Vec<_> = case["result"]["decor"]["colour"]
            .as_array()
            .map(|v| v.iter().map(paint).collect())
            .unwrap_or_default();
        for implicit in [false, true] {
            let mut descriptor = boundary_fixtures::boundary_scale(case)
                .unwrap()
                .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))
                .unwrap();
            if implicit {
                descriptor.guide = None;
            }
            let legend = MappedScale::for_colors(descriptor.clone())
                .unwrap()
                .legend(
                    chart_core::ScaleId::new(1),
                    paint(&Json::String("#7f7f7f".into())),
                )
                .unwrap();
            assert_eq!(
                legend
                    .colorsteps
                    .iter()
                    .map(|c| c.color)
                    .collect::<Vec<_>>(),
                expected,
                "{case}"
            );
            if case["population"] == "ordinary" && !expected.is_empty() {
                for facet in ["single", "collected", "local"] {
                    let p = author("asymmetric", "color", facet, false)
                        .edit()
                        .scale(color_mapped("v", descriptor.clone()))
                        .build()
                        .unwrap();
                    let frame = draw(&p, &request(360.)).unwrap();
                    let bars = presentation_bars(
                        &frame,
                        GradientDirection::Vertical,
                        &expected.iter().rev().copied().collect::<Vec<_>>(),
                    );
                    assert_eq!(bars.len(), if facet == "local" { 2 } else { 1 }, "{case}");
                    for bar in bars {
                        let keys = case["result"]["key"][".value"].as_array().unwrap();
                        let ticks = presentation_ticks(&frame, bar, GradientDirection::Vertical);
                        assert_eq!(ticks.len(), keys.len(), "{case}");
                        for (a, b) in ticks.iter().zip(keys) {
                            close(*a, b.as_f64().unwrap());
                        }
                        let labels = presentation_labels(&frame, bar, GradientDirection::Vertical);
                        assert_eq!(labels.len(), keys.len(), "{case}");
                        for ((label, pos), (want, key)) in labels.iter().zip(
                            case["result"]["key"][".label"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .zip(keys),
                        ) {
                            assert_eq!(label, want.as_str().unwrap());
                            close(*pos, key.as_f64().unwrap());
                        }
                    }
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 40);
}

#[test]
fn stepped_widths_and_limit_labels_match_source() {
    use chart_core::scales::*;
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorsteps-controls.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in source["cases"].as_array().unwrap().iter().filter(|c| {
        c["population"] != "constant" && !(c["family"] == "binned" && c["mode"] == "null")
    }) {
        let descriptor = boundary_fixtures::boundary_scale(case)
            .unwrap()
            .with_colorbar_options(GgplotColorbarOptions {
                even_steps: case["even_steps"].as_bool().unwrap(),
                show_limits: case["show_limits"].as_bool().unwrap(),
                ..Default::default()
            });
        let legend = MappedScale::for_colors(descriptor.clone())
            .unwrap()
            .legend(
                chart_core::ScaleId::new(1),
                paint(&Json::String("#7f7f7f".into())),
            )
            .unwrap();
        let expected: Vec<_> = case["result"]["decor"]["colour"]
            .as_array()
            .map(|v| v.iter().map(paint).collect())
            .unwrap_or_default();
        assert_eq!(
            legend
                .colorsteps
                .iter()
                .map(|s| s.color)
                .collect::<Vec<_>>(),
            expected,
            "{case}"
        );
        let actual: Vec<_> = legend
            .numeric_breaks
            .iter()
            .zip(&legend.colorstep_positions)
            .filter_map(|(key, p)| p.map(|p| (key.label.as_deref(), p.0)))
            .collect();
        let keys = case["result"]["key"][".value"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let labels = case["result"]["key"][".label"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(actual.len(), keys.len(), "{case}");
        for ((label, pos), (key, want)) in actual.iter().zip(keys.iter().zip(&labels)) {
            assert_eq!(*label, want.as_str(), "{case}");
            close(*pos, key.as_f64().unwrap());
        }
        if case["population"] == "ordinary"
            && !expected.is_empty()
            && case["result"]["bar"].is_object()
        {
            let p = author("asymmetric", "color", "single", false)
                .edit()
                .scale(color_mapped("v", descriptor))
                .build()
                .unwrap();
            let frame = draw(&p, &request(360.)).unwrap();
            if !case["even_steps"].as_bool().unwrap() {
                let cells: Vec<_> = frame
                    .scene()
                    .items()
                    .iter()
                    .filter_map(|i| match &i.primitive {
                        Primitive::Rectangle { bounds, fill }
                            if i.layer.is_none() && expected.contains(fill) =>
                        {
                            Some((*bounds, *fill))
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(cells.len(), expected.len(), "{case}");
                for ((bounds, color), (height, want)) in cells.iter().zip(
                    case["result"]["bar"]["height"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .zip(&expected),
                ) {
                    assert_eq!(color, want);
                    close(bounds.height() / 120., height.as_f64().unwrap());
                }
            }
        }
        checked += 1;
    }
    assert_eq!(checked, 152);
}

#[test]
fn stepped_control_constant_draw_outcomes_match_source() {
    use chart_core::scales::*;
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorsteps-controls.json"
    ))
    .unwrap();
    for case in source["cases"].as_array().unwrap().iter().filter(|c| {
        c["population"] == "constant" && !(c["family"] == "binned" && c["mode"] == "null")
    }) {
        let descriptor = boundary_fixtures::boundary_scale(case)
            .unwrap()
            .with_colorbar_options(GgplotColorbarOptions {
                even_steps: case["even_steps"].as_bool().unwrap(),
                show_limits: case["show_limits"].as_bool().unwrap(),
                ..Default::default()
            });
        let result = fixtures::author_values("asymmetric", "color", "single", false, [2.; 8])
            .edit()
            .scale(color_mapped("v", descriptor))
            .build()
            .and_then(|p| p.chart())
            .and_then(|mut c| c.prepare())
            .and_then(|p| layout(p, &request(360.), &Metrics));
        assert_eq!(
            result.is_err(),
            case["result"]["error"].is_string(),
            "{case}; actual error: {:?}",
            result.err()
        );
    }
}

#[test]
fn default_binned_constant_draw_outcomes_match_source() {
    use chart_core::scales::*;
    let source: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorsteps-default-boundaries.json"
    ))
    .unwrap();
    let mut checked = 0;
    for case in source["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["population"] == "constant" && c["mode"] != "null")
    {
        for implicit in [false, true] {
            let mut descriptor = boundary_fixtures::boundary_scale(case)
                .unwrap()
                .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))
                .unwrap();
            if implicit {
                descriptor.guide = None;
            }
            let result = fixtures::author_values("asymmetric", "color", "single", false, [2.; 8])
                .edit()
                .scale(color_mapped("v", descriptor))
                .build()
                .and_then(|p| p.chart())
                .and_then(|mut c| c.prepare())
                .and_then(|p| layout(p, &request(360.), &Metrics));
            assert_eq!(
                result.is_err(),
                case["result"]["error"].is_string(),
                "{case}: {:?}",
                result.err()
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 20);
}
