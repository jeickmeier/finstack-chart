//! FIX-GG04: automatic and explicit default constructors obey theme selection.
use chart_core::{
    ChartResult, Revision,
    grammar::*,
    interpolate::{Number, Value},
    prelude::*,
    scales::*,
};
use serde_json::{Value as Json, json};
use std::sync::Arc;
struct Constant;

#[test]
fn ordinal_paint_defaults_bypass_theme_for_20_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/default-ordinal-theme-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 20);
    for t in cases {
        let inputs = t["inputs"].as_array().unwrap();
        let data = Data::columns()
            .column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>())
            .column(
                "v",
                inputs
                    .iter()
                    .map(|v| v.as_str().map(str::to_owned))
                    .collect::<Vec<_>>(),
            )
            .build()
            .unwrap();
        let channel = t["channel"].as_str().unwrap();
        let ColorScale::Mapped { scale, .. } = ggplot_color_ordinal().unwrap() else {
            unreachable!()
        };
        assert!(scale.palette_theme_aesthetics.is_empty());
        let mapping = if channel == "fill" {
            aes().x("x").y(1.).fill("v").fill_scale("v")
        } else {
            aes().x("x").y(1.).color("v").color_scale("v")
        };
        let mut registry = ExtensionRegistry::default();
        registry.register_scale_palette(Arc::new(Constant)).unwrap();
        let mut draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry))
            .aes(mapping)
            .scale(color_mapped(
                "v",
                scale.with_guide(GgplotScaleGuide::Hidden).unwrap(),
            ))
            .layer(points().aesthetic_value(ValueAesthetic::Shape, Value::Number(Number(21.))));
        if t["theme_mode"] == "supplied" {
            draft = draft.theme(theme().scale_palettes(std::collections::BTreeMap::from([(
                format!("palette.{channel}.discrete"),
                ScalePaletteOperation {
                    operation: OperationRef {
                        id: "test.default_palette".into(),
                        version: Revision::new(1),
                    },
                    parameters: json!({"channel":channel}),
                },
            )])));
        }
        let p = draft.build().unwrap();
        let wire = p.to_json().unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        let wanted = t["result"]["mapped"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| channel == "fill" || !v.is_null())
            .collect::<Vec<_>>();
        assert_eq!(marks.len(), wanted.len(), "{t}");
        assert_eq!(
            marks.len(),
            t["result"]["point_count"].as_u64().unwrap() as usize
        );
        for (m, v) in marks.iter().zip(wanted) {
            let actual = if channel == "fill" {
                m.style.fill
            } else {
                Some(m.style.color)
            };
            let expected = Some(v.as_str().map_or(
                chart_core::scene::Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                },
                |v| chart_core::color::parse_r(v).unwrap().resolve(),
            ));
            assert_eq!(actual, expected, "{t}");
        }
        assert_eq!(p.to_json().unwrap(), wire);
    }
}
impl CustomScalePalette for Constant {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.default_palette", Revision::new(1), true)
    }
    fn validate(&self, _: &Json) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let n = match input.domain {
            ScalePaletteDomain::Count(n) => n,
            ScalePaletteDomain::Normalized(v) => v.len(),
        };
        let channel = input.parameters["channel"].as_str().unwrap();
        let value = match channel {
            "colour" | "fill" => Value::Text("#ff0000".into()),
            "alpha" => Value::Number(Number(0.3)),
            _ => Value::Number(Number(3.)),
        };
        Ok(ScalePaletteOutput {
            values: Some(vec![value; n]),
            names: None,
        })
    }
}
fn kind(channel: &str, route: &str) -> GgplotNumericPalette {
    match (channel, route) {
        (_, "area") => GgplotNumericPalette::Area,
        (_, "radius") => GgplotNumericPalette::Radius,
        ("size", _) => GgplotNumericPalette::Size,
        ("alpha", _) => GgplotNumericPalette::Alpha,
        _ => GgplotNumericPalette::Linewidth,
    }
}
#[test]
fn default_constructors_match_56_reference_draws() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/default-palette-selection.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 56);
    for t in cases {
        assert!(t["result"].get("error").is_none(), "{t}");
        let channel = t["channel"].as_str().unwrap();
        let route = t["route"].as_str().unwrap();
        let discrete = t["family"] == "discrete";
        let inputs = t["inputs"].as_array().unwrap();
        let data =
            Data::columns().column("x", (0..inputs.len()).map(|i| i as f64).collect::<Vec<_>>());
        let data = if discrete {
            data.column(
                "v",
                inputs
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect::<Vec<_>>(),
            )
        } else {
            data.column(
                "v",
                inputs
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect::<Vec<_>>(),
            )
        }
        .build()
        .unwrap();
        let mut mapping = aes().x("x").y(1.);
        mapping = match channel {
            "colour" => mapping.color("v"),
            "fill" => mapping.fill("v"),
            "size" => mapping.size("v"),
            "alpha" => mapping.alpha("v"),
            _ => mapping.linewidth("v"),
        };
        let mut layer = points().name("marks");
        let mut registry = ExtensionRegistry::default();
        registry.register_scale_palette(Arc::new(Constant)).unwrap();
        let mut draft = plot(data)
            .profile(Profile::Ggplot2_4_0_3)
            .extensions(Arc::new(registry));
        if route != "automatic" {
            if matches!(channel, "colour" | "fill") {
                let ColorScale::Mapped { mut scale, .. } = ggplot_color_default(!discrete).unwrap()
                else {
                    unreachable!()
                };
                if channel == "fill" {
                    scale = scale.with_theme_palette(vec!["fill".into()]).unwrap();
                    mapping = mapping.fill_scale("v");
                } else {
                    mapping = mapping.color_scale("v");
                }
                draft = draft.scale(color_mapped("v", scale));
            } else {
                let mut scale = if discrete {
                    ggplot_numeric_ordinal(kind(channel, route))
                } else {
                    ggplot_numeric_default(kind(channel, route))
                }
                .unwrap();
                if route == "range" {
                    if let Some(GgplotScalePolicy::Discrete {
                        palette: GgplotDiscretePalette::NumericRange { range, .. },
                        ..
                    }) = scale.ggplot.as_deref_mut()
                    {
                        *range = [Number(0.2), Number(0.8)];
                    } else if let ScaleFunctionSpec::Interpolated(s) = &mut scale.function
                        && let ScaleRangeFunction::Interpolate(
                            chart_core::interpolate::InterpolationSpec::PowerRange {
                                range, ..
                            },
                        ) = &mut s.output
                    {
                        *range = [Number(0.2), Number(0.8)];
                    }
                    scale = scale.with_theme_palette(vec![]).unwrap();
                }
                layer = layer.numeric_scale(
                    match channel {
                        "size" => NumericAesthetic::Size,
                        "alpha" => NumericAesthetic::Alpha,
                        _ => NumericAesthetic::StrokeWidth,
                    },
                    "v",
                    scale,
                );
            }
        }
        if t["theme_mode"] == "supplied" {
            draft = draft.theme(
                theme().scale_palettes(
                    [(
                        format!(
                            "palette.{channel}.{}",
                            if discrete { "discrete" } else { "continuous" }
                        ),
                        ScalePaletteOperation {
                            operation: OperationRef {
                                id: "test.default_palette".into(),
                                version: Revision::new(1),
                            },
                            parameters: json!({"channel":channel}),
                        },
                    )]
                    .into(),
                ),
            );
        }
        let p = draft.aes(mapping).layer(layer).build().unwrap();
        let wire = p.to_json().unwrap();
        let prepared = p.chart().unwrap().prepare().unwrap();
        let marks = prepared.layers()[0].marks();
        let wanted = t["result"]["mapped"].as_array().unwrap();
        assert_eq!(marks.len(), wanted.len(), "{t}");
        for (index, (m, v)) in marks.iter().zip(wanted).enumerate() {
            if matches!(channel, "colour" | "fill") {
                let actual = if channel == "fill" {
                    m.style.fill.unwrap()
                } else {
                    m.style.color
                };
                assert_eq!(
                    actual,
                    chart_core::color::parse_r(v.as_str().unwrap())
                        .unwrap()
                        .resolve(),
                    "{t}"
                );
            } else {
                let expected = v.as_f64().unwrap();
                let (actual, expected) = match channel {
                    "size" => (m.style.radius, expected),
                    "alpha" => (
                        f64::from(m.style.color.alpha),
                        f64::from(
                            chart_core::color::parse_r(
                                t["result"]["point_colours"][index].as_str().unwrap(),
                            )
                            .unwrap()
                            .resolve()
                            .alpha,
                        ),
                    ),
                    _ => (m.style.stroke_width, expected),
                };
                assert!(
                    (actual - expected).abs() < 2e-12,
                    "{actual} != {expected}: {t}"
                );
            }
        }
        assert_eq!(p.to_json().unwrap(), wire);
    }
}

#[test]
fn temporal_paint_bypasses_theme_while_numeric_fallbacks_use_it() {
    use chart_core::data::TimeUnit;
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/default-temporal-theme-palettes.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 60);
    for t in cases {
        for (unit, multiplier) in [
            (TimeUnit::Seconds, 1_i64),
            (TimeUnit::Milliseconds, 1000),
            (TimeUnit::Microseconds, 1_000_000),
            (TimeUnit::Nanoseconds, 1_000_000_000),
        ] {
            let values = t["inputs"].as_array().unwrap();
            let channel = t["channel"].as_str().unwrap();
            let stamps = values
                .iter()
                .map(|v| {
                    v.as_i64().unwrap_or(0)
                        * multiplier
                        * if t["kind"] == "date" { 86400 } else { 1 }
                })
                .collect::<Vec<_>>();
            let d = Data::columns()
                .column("x", (0..values.len()).map(|i| i as f64).collect::<Vec<_>>())
                .column(
                    "end",
                    (0..values.len())
                        .map(|i| i as f64 + 0.5)
                        .collect::<Vec<_>>(),
                )
                .column(
                    "v",
                    timestamps(stamps, unit, "UTC")
                        .validity(values.iter().map(|v| !v.is_null()).collect()),
                )
                .build()
                .unwrap();
            let mapping = aes().x("x").y(1.);
            let mapping = match channel {
                "colour" => mapping.color("v"),
                "fill" => mapping.fill("v"),
                "size" => mapping.size("v"),
                "alpha" => mapping.alpha("v"),
                _ => mapping.x2("end").y2(1.).linewidth("v"),
            };
            let mut registry = ExtensionRegistry::new();
            registry.register_scale_palette(Arc::new(Constant)).unwrap();
            let mut draft = plot(d)
                .profile(Profile::Ggplot2_4_0_3)
                .extensions(Arc::new(registry))
                .aes(mapping)
                .layer(if channel == "linewidth" {
                    rule()
                } else {
                    points()
                });
            if t["theme_mode"] == "supplied" {
                draft = draft.theme(
                    theme().scale_palettes(
                        [(
                            format!("palette.{channel}.continuous"),
                            ScalePaletteOperation {
                                operation: OperationRef {
                                    id: "test.default_palette".into(),
                                    version: Revision::new(1),
                                },
                                parameters: json!({"channel":channel}),
                            },
                        )]
                        .into(),
                    ),
                );
            }
            let p = draft.build().unwrap();
            let pre = p.chart().unwrap().prepare().unwrap();
            let marks = pre.layers()[0].marks();
            let wanted = t["result"]["mapped"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|v| !v.is_null() || !matches!(channel, "size" | "linewidth"))
                .collect::<Vec<_>>();
            assert_eq!(marks.len(), wanted.len(), "{t}");
            for (index, (m, v)) in marks.iter().zip(wanted).enumerate() {
                if channel == "linewidth" && v.is_null() {
                    let encoding =
                        &pre.layers()[0].numeric_scales()[&NumericAesthetic::StrokeWidth];
                    assert_eq!(
                        MappedScale::new(encoding.scale.clone())
                            .unwrap()
                            .numeric(None)
                            .unwrap(),
                        Value::Missing
                    );
                    assert_eq!(
                        m.style.color,
                        chart_core::color::parse_r(
                            t["result"]["mark_colours"][index].as_str().unwrap()
                        )
                        .unwrap()
                        .resolve()
                    );
                    continue;
                }
                if matches!(channel, "colour" | "fill") {
                    assert_eq!(
                        if channel == "fill" {
                            m.style.fill.unwrap()
                        } else {
                            m.style.color
                        },
                        chart_core::color::parse_r(v.as_str().unwrap())
                            .unwrap()
                            .resolve(),
                        "{t}"
                    );
                } else {
                    let (actual, expected) = match channel {
                        "size" => (m.style.radius, v.as_f64().unwrap()),
                        "alpha" => (
                            f64::from(m.style.color.alpha),
                            f64::from(
                                chart_core::color::parse_r(
                                    t["result"]["mark_colours"][index].as_str().unwrap(),
                                )
                                .unwrap()
                                .resolve()
                                .alpha,
                            ),
                        ),
                        _ => (m.style.stroke_width, v.as_f64().unwrap()),
                    };
                    assert!(
                        (actual - expected).abs() < 2e-12,
                        "{t}: {actual} != {expected}"
                    );
                }
            }
        }
    }
}
