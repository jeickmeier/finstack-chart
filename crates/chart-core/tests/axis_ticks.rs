//! WP-AX02 / FIX-19: independent guide selection and formatting over shared scales.
use chart_core::{
    ChartResult, DiagnosticCode, GuideId, Rect, ResourceId, Revision,
    composition::ScaleValue,
    data::TimeUnit,
    grammar::{
        Compiler, CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideFormatInput,
        GuideLabelsInput, OperationRef, PreparedChart,
    },
    layout::{
        AxisScale, AxisSide, GuideFormatter, GuideProfile, GuideStyle, LayoutRequest, layout,
    },
    plot::{Plot, categorical, timestamps},
    prelude::*,
    scales::*,
    services::{ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units},
    state::ChartState,
};
use serde_json::{Value as Json, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, request: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(request.text.chars().count() as f64 * 5., 8., 2.)
    }
}
struct Formatter {
    calls: Arc<AtomicUsize>,
    portable: bool,
}
impl CustomGuideFormatter for Formatter {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.guide-format", Revision::new(1), self.portable)
    }
    fn validate(&self, p: &Json) -> ChartResult<()> {
        assert!(matches!(
            p.as_str(),
            Some("Blank" | "Same" | "Indexed" | "Context")
        ));
        Ok(())
    }
    fn format(&self, input: GuideFormatInput<'_>) -> ChartResult<String> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        assert_eq!(&input.values[input.index], input.value);
        let value = match input.value {
            ScaleValue::Number(n) => n.to_string(),
            ScaleValue::Category(s) => s.clone(),
            ScaleValue::Timestamp { value, .. } => value.to_string(),
            ScaleValue::MissingCategory => "NA".into(),
        };
        Ok(match input.parameters.as_str().unwrap() {
            "Blank" => String::new(),
            "Same" => "same".into(),
            "Context" => format!("{}:{}:{value}", input.values.len(), input.index),
            _ => format!("{}:{value}", input.index),
        })
    }
}
fn registry(portable: bool) -> (Arc<ExtensionRegistry>, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut registry = ExtensionRegistry::new();
    registry
        .register_guide_formatter(Arc::new(Formatter {
            calls: calls.clone(),
            portable,
        }))
        .unwrap();
    (Arc::new(registry), calls)
}
fn registered(mode: &str) -> GuideFormatter {
    GuideFormatter::Registered {
        operation: OperationRef::new("test.guide-format", Revision::new(1)),
        parameters: json!(mode),
    }
}
fn prepare(p: &Plot) -> Arc<PreparedChart> {
    Arc::new(
        Compiler::with_extensions(p.extensions().clone())
            .prepare(
                p.definition(),
                &p.source(),
                &ChartState::default(),
                p.compile_limits(),
            )
            .unwrap(),
    )
}
fn request() -> LayoutRequest {
    LayoutRequest::new(
        Rect::new(0., 0., 600., 400.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    )
}
fn numeric_plot(registry: Arc<ExtensionRegistry>) -> Plot {
    plot(
        Data::columns()
            .column("x", vec![0., 0.5, 1.])
            .column("y", vec![1., 2., 3.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .extensions(registry)
    .build()
    .unwrap()
}
fn configured_request() -> LayoutRequest {
    let mut r = request();
    r.axes[0].scale = AxisScale::Linear(ContinuousDomain::explicit(Bounds::new(0., 1.).unwrap()));
    r.axes[0].range = Some(Bounds::new(100., 500.).unwrap());
    r.axes[0].profile = GuideProfile::D3_3_0_0;
    r
}
fn labels(prepared: Arc<PreparedChart>, r: &LayoutRequest) -> Vec<(ScaleValue, String)> {
    layout(prepared, r, &Metrics).unwrap().guides()[&GuideId::new(0)]
        .ticks
        .iter()
        .map(|tick| (tick.value.clone(), tick.label.clone()))
        .collect()
}

#[test]
fn count_is_a_hint_and_explicit_values_preserve_order_occurrences_and_empty_labels() {
    let p = prepare(&numeric_plot(Arc::default()));
    let mut r = configured_request();
    let ticks = labels(p.clone(), &r);
    assert_eq!(ticks.len(), 11);
    assert_eq!(
        ticks.iter().map(|t| t.0.clone()).collect::<Vec<_>>(),
        (0..=10)
            .map(|n| ScaleValue::Number(n as f64 / 10.))
            .collect::<Vec<_>>()
    );
    assert_eq!(ticks.first().unwrap().1, "0.0");
    r.axes[0].tick_arguments = Some(GuideTickArguments {
        count: Some(0.),
        ..Default::default()
    });
    assert!(labels(p.clone(), &r).is_empty());
    r.axes[0].tick_values = Some(vec![
        0.8.into(),
        0.2.into(),
        0.8.into(),
        0.0.into(),
        1.0.into(),
    ]);
    r.axes[0].tick_format = Some(GuideFormatter::Labels(vec![
        "".into(),
        "same".into(),
        "same".into(),
        "".into(),
        "same".into(),
    ]));
    let explicit = labels(p.clone(), &r);
    assert_eq!(
        explicit.iter().map(|t| t.0.clone()).collect::<Vec<_>>(),
        r.axes[0].tick_values.clone().unwrap()
    );
    assert_eq!(
        explicit.iter().map(|t| t.1.as_str()).collect::<Vec<_>>(),
        ["", "same", "same", "", "same"]
    );
    r.axes[0].tick_format = None;
    r.axes[0].tick_values = Some(vec![]);
    assert!(labels(p.clone(), &r).is_empty());
    r.axes[0].tick_arguments = None;
    r.axes[0].tick_values = None;
    assert_eq!(labels(p, &r), ticks);
}

#[test]
fn explicit_time_values_bypass_an_unbounded_automatic_interval_and_keep_exact_source_units() {
    let base = 9_007_199_254_740_993i64;
    let p = plot(
        Data::columns()
            .column(
                "x",
                timestamps(
                    vec![base, base + 1_000_000_000],
                    TimeUnit::Nanoseconds,
                    "UTC",
                ),
            )
            .column("y", vec![0., 1.])
            .build()
            .unwrap(),
    )
    .aes(aes().x("x").y("y"))
    .layer(points())
    .build()
    .unwrap();
    let mut r = request();
    r.max_ticks = 3;
    r.target_ticks = 2;
    r.axes[0].profile = GuideProfile::D3_3_0_0;
    r.axes[0].tick_arguments = Some(GuideTickArguments {
        interval: Some(CalendarInterval::new(CalendarUnit::Millisecond)),
        ..Default::default()
    });
    r.axes[0].tick_values = Some(vec![]);
    let p = prepare(&p);
    assert!(labels(p.clone(), &r).is_empty());
    r.axes[0].tick_values = Some(vec![ScaleValue::Timestamp {
        value: base + 1,
        unit: TimeUnit::Nanoseconds,
    }]);
    assert_eq!(
        labels(p.clone(), &r)[0].0,
        r.axes[0].tick_values.as_ref().unwrap()[0]
    );
    r.axes[0].tick_values = None;
    assert_eq!(
        layout(p, &r, &Metrics).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
}

#[test]
fn registered_formatter_gets_full_semantic_context_and_failures_precede_callbacks() {
    let (registry, calls) = registry(true);
    let p = prepare(&numeric_plot(registry));
    let mut r = configured_request();
    r.axes[0].tick_values = Some(vec![0.8.into(), 0.2.into(), 0.8.into()]);
    r.axes[0].tick_format = Some(registered("Context"));
    let ticks = labels(p.clone(), &r);
    assert_eq!(
        ticks.iter().map(|t| t.1.as_str()).collect::<Vec<_>>(),
        ["3:0:0.8", "3:1:0.2", "3:2:0.8"]
    );
    assert!(calls.load(Ordering::Relaxed) >= 3);
    calls.store(0, Ordering::Relaxed);
    r.axes[0].tick_values = Some(vec![ScaleValue::Number(f64::NAN)]);
    assert!(layout(p.clone(), &r, &Metrics).is_err());
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    r.axes[0].tick_values = Some(vec![0.0.into(); r.max_ticks + 1]);
    assert_eq!(
        layout(p, &r, &Metrics).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[test]
fn vector_formatter_preserves_order_missing_labels_and_checks_output() {
    struct VectorFormatter;
    impl CustomGuideFormatter for VectorFormatter {
        fn descriptor(&self) -> ExtensionDescriptor {
            ExtensionDescriptor::batch("test.guide-format", Revision::new(1), true)
        }
        fn validate(&self, _: &Json) -> ChartResult<()> {
            Ok(())
        }
        fn format_labels(&self, input: GuideLabelsInput<'_>) -> ChartResult<Vec<Option<String>>> {
            assert_eq!(input.values, &[0.8.into(), 0.2.into(), 0.8.into()]);
            assert!(input.names.is_none());
            Ok(match input.parameters.as_str().unwrap() {
                "Short" => vec![Some("short".into())],
                "Long" => vec![Some("x".repeat(input.limits.max_text_bytes + 1)); 3],
                _ => vec![Some("first".into()), None, Some("last".into())],
            })
        }
    }
    let mut registry = ExtensionRegistry::new();
    registry
        .register_guide_formatter(Arc::new(VectorFormatter))
        .unwrap();
    let p = prepare(&numeric_plot(Arc::new(registry)));
    let mut r = configured_request();
    r.axes[0].tick_values = Some(vec![0.8.into(), 0.2.into(), 0.8.into()]);
    r.axes[0].tick_format = Some(registered("Vector"));
    let ticks = labels(p.clone(), &r);
    assert_eq!(
        ticks.iter().map(|t| t.1.as_str()).collect::<Vec<_>>(),
        ["first", "", "last"]
    );
    r.axes[0].tick_format = Some(registered("Short"));
    assert_eq!(
        layout(p.clone(), &r, &Metrics).unwrap_err().code,
        DiagnosticCode::SchemaConflict
    );
    r.axes[0].tick_format = Some(registered("Long"));
    assert_eq!(
        layout(p, &r, &Metrics).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
}

#[test]
fn primary_controls_are_versioned_independent_resettable_and_portability_checked() {
    for portable in [true, false] {
        let (registry, _) = registry(portable);
        let p = plot(
            Data::columns()
                .column("x", vec![0., 1.])
                .column("y", vec![0., 1.])
                .build()
                .unwrap(),
        )
        .aes(aes().x("x").y("y"))
        .layer(points())
        .extensions(registry.clone())
        .axis(
            x_axis()
                .name("x")
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_arguments(Some(GuideTickArguments {
                    count: Some(5.),
                    specifier: Some(".1%".into()),
                    interval: None,
                    seconds: None,
                    time_width: None,
                    width: None,
                }))
                .tick_values(Some(vec![0.0.into(), 1.0.into()]))
                .tick_format(Some(registered("Same"))),
        )
        .guide(
            axis_guide("top", "x")
                .side(AxisSide::Top)
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_values(Some(vec![]))
                .tick_values(None)
                .tick_format(Some(registered("Same")))
                .tick_format(None),
        )
        .build()
        .unwrap();
        assert_eq!(p.definition().wire_version(), 11);
        let top = &p.definition().guides[0];
        assert!(top.tick_values.is_none() && top.tick_format.is_none());
        let encoded = p.to_json();
        if portable {
            let encoded = encoded.unwrap();
            assert!(Plot::from_json(&encoded).is_err());
            assert_eq!(
                Plot::from_json_with_extensions(&encoded, registry)
                    .unwrap()
                    .definition(),
                p.definition()
            );
            let mut old: Json = serde_json::from_str(&encoded).unwrap();
            old["version"] = json!(10);
            assert!(
                Plot::from_json_with_extensions(&old.to_string(), p.extensions().clone()).is_err()
            );
        } else {
            assert_eq!(
                encoded.unwrap_err().code,
                DiagnosticCode::UnsupportedCapability
            );
        }
    }
}

fn apply_config(style: &mut GuideStyle, config: &Json) {
    for (key, field) in [
        ("tickSizeInner", "inner"),
        ("tickSizeOuter", "outer"),
        ("tickPadding", "padding"),
        ("offset", "offset"),
        ("tickSize", "both"),
    ] {
        if let Some(value) = config[key].as_f64() {
            let g = style.geometry.get_or_insert_with(Default::default);
            match field {
                "inner" => g.inner = Some(value),
                "outer" => g.outer = Some(value),
                "padding" => g.padding = Some(value),
                "offset" => g.offset = Some(value),
                _ => {
                    g.inner = Some(value);
                    g.outer = Some(value);
                }
            }
        }
    }
    if let Some(arguments) = config.get("arguments") {
        let a = arguments.as_array().unwrap();
        style.tick_arguments = Some(GuideTickArguments {
            count: a.first().and_then(Json::as_f64),
            specifier: a.get(1).and_then(Json::as_str).map(Into::into),
            interval: None,
            seconds: None,
            time_width: None,
            width: None,
        });
    }
    if let Some(interval) = config.get("interval") {
        let unit = match interval[0].as_str().unwrap() {
            "utcDay" => CalendarUnit::Day,
            "utcMinute" => CalendarUnit::Minute,
            "utcYear" => CalendarUnit::Year,
            other => panic!("{other}"),
        };
        style.tick_arguments = Some(GuideTickArguments {
            interval: Some(CalendarInterval {
                unit,
                step: interval[1].as_u64().unwrap() as u32,
            }),
            ..Default::default()
        });
    }
    if let Some(values) = config.get("values") {
        style.tick_values = values.as_array().map(|values| {
            values
                .iter()
                .map(|n| ScaleValue::Number(n.as_f64().unwrap()))
                .collect()
        });
    }
    if let Some(formatter) = config.get("formatter") {
        style.tick_format = formatter.as_str().map(registered);
    }
}

#[test]
fn pinned_axis_reference_preserves_all_selected_values_and_labels_in_both_device_profiles() {
    let reference: Json =
        serde_json::from_str(include_str!("../../../fixtures/axes/reference.json")).unwrap();
    let (registry, _) = registry(true);
    let mut case_count = 0;
    let mut state_count = 0;
    for profile in reference["profiles"].as_array().unwrap() {
        for case in profile["cases"].as_array().unwrap() {
            let id = case["id"].as_str().unwrap();
            let scale = &case["scale"];
            let kind = scale["kind"].as_str().unwrap();
            let domain = scale["domain"].as_array().unwrap();
            let side: AxisSide = serde_json::from_value(case["side"].clone()).unwrap();
            let horizontal = matches!(side, AxisSide::Top | AxisSide::Bottom);
            let axis_index = usize::from(!horizontal);
            let mut r = request();
            r.device_scale = profile["device_scale"].as_f64();
            r.axes[1 - axis_index].visible = false;
            let values = match kind {
                "Band" | "Point" => categorical(domain.iter().map(|s| s.as_str().unwrap())),
                "Utc" => timestamps(
                    domain.iter().map(|n| n.as_i64().unwrap()).collect(),
                    TimeUnit::Milliseconds,
                    "UTC",
                ),
                _ => domain
                    .iter()
                    .map(|n| n.as_f64().unwrap())
                    .collect::<Vec<_>>()
                    .into(),
            };
            let p = plot(
                Data::columns()
                    .column("value", values)
                    .column("other", vec![1.; domain.len()])
                    .build()
                    .unwrap(),
            )
            .aes(if horizontal {
                aes().x("value").y("other")
            } else {
                aes().x("other").y("value")
            })
            .layer(points())
            .extensions(registry.clone())
            .build()
            .unwrap();
            let a = &mut r.axes[axis_index];
            a.side = side;
            a.profile = GuideProfile::D3_3_0_0;
            if let Some(range) = scale.get("range") {
                a.range = Some(
                    Bounds::new(range[0].as_f64().unwrap(), range[1].as_f64().unwrap()).unwrap(),
                );
            }
            // Standalone D3 identity uses its domain as its range; the chart adapter
            // explicitly places that coordinate range instead of fitting the plot span.
            if kind == "Identity" {
                a.range = Some(
                    Bounds::new(domain[0].as_f64().unwrap(), domain[1].as_f64().unwrap()).unwrap(),
                );
            }
            let padding = scale["padding"].as_f64().unwrap_or(0.);
            let align = scale["align"].as_f64().unwrap_or(0.5);
            let round = scale["round"].as_bool().unwrap_or(false);
            a.scale = match kind {
                "Band" => AxisScale::D3Band(BandSpec {
                    domain: Some(domain.iter().map(|s| s.as_str().unwrap().into()).collect()),
                    padding_inner: padding,
                    padding_outer: padding,
                    align,
                    round,
                }),
                "Point" => AxisScale::D3Point(PointSpec {
                    domain: Some(domain.iter().map(|s| s.as_str().unwrap().into()).collect()),
                    padding,
                    align,
                    round,
                }),
                "Utc" => AxisScale::Utc {
                    domain: Some(TimeBounds {
                        start: domain[0].as_i64().unwrap(),
                        end: domain[1].as_i64().unwrap(),
                    }),
                    interval: None,
                },
                _ => {
                    let family = match kind {
                        "Linear" => NumericFamily::Linear,
                        "Log" => NumericFamily::Log {
                            base: scale["base"].as_f64().unwrap_or(10.),
                        },
                        "Symlog" => NumericFamily::Symlog {
                            constant: scale["constant"].as_f64().unwrap_or(1.),
                        },
                        "Pow" => NumericFamily::Pow {
                            exponent: scale["exponent"].as_f64().unwrap_or(1.),
                        },
                        "Sqrt" => NumericFamily::Pow { exponent: 0.5 },
                        "Identity" => NumericFamily::Identity,
                        other => panic!("{other}"),
                    };
                    let mut s = NumericScaleSpec::d3(family);
                    s.domain = domain.iter().map(|n| n.as_f64().unwrap().into()).collect();
                    AxisScale::Numeric(s)
                }
            };
            apply_config(&mut a.guide, &case["config"]);
            let prepared = prepare(&p);
            for (state_index, expected) in case["states"].as_array().unwrap().iter().enumerate() {
                if state_index == 1 {
                    apply_config(&mut r.axes[axis_index].guide, &case["config"]["reset"]);
                }
                let scene = layout(prepared.clone(), &r, &Metrics)
                    .unwrap_or_else(|e| panic!("{id}: {e:?}"));
                assert_axis_geometry(&scene, axis_index, expected, id);
                let ticks = &scene.guides()[&GuideId::new(axis_index as u64)].ticks;
                let expected = expected["ticks"].as_array().unwrap();
                assert_eq!(ticks.len(), expected.len(), "{id} state {state_index}");
                for (tick, expected) in ticks.iter().zip(expected) {
                    let value = match kind {
                        "Band" | "Point" => {
                            ScaleValue::Category(expected["value"].as_str().unwrap().into())
                        }
                        "Utc" => ScaleValue::Timestamp {
                            value: expected["value"]["Timestamp"].as_i64().unwrap(),
                            unit: TimeUnit::Milliseconds,
                        },
                        _ => ScaleValue::Number(expected["value"].as_f64().unwrap()),
                    };
                    assert_eq!(tick.value, value, "{id} state {state_index}");
                    assert_eq!(
                        tick.label,
                        expected["text"]["label"].as_str().unwrap(),
                        "{id} state {state_index} {value:?}"
                    );
                }
                state_count += 1;
            }
            case_count += 1;
        }
    }
    assert_eq!(case_count, 372);
    assert_eq!(state_count, 376);
}

// FIX-19-F: compare the committed browser's coordinates, not a second copy of core formulas.
fn assert_axis_geometry(
    chart: &chart_core::layout::LaidOutChart,
    axis: usize,
    expected: &Json,
    id: &str,
) {
    use chart_core::scene::{PathCommand, Primitive};
    let guide = &chart.guides()[&GuideId::new(axis as u64)];
    let plot = chart.plot().unwrap();
    let side = guide.spec.side;
    let horizontal = side.horizontal();
    let origin = match side {
        AxisSide::Bottom => (0., plot.max_y()),
        AxisSide::Top => (0., plot.origin().y()),
        AxisSide::Left => (plot.origin().x(), 0.),
        AxisSide::Right => (plot.max_x(), 0.),
    };
    let close = |actual: f64, expected: f64| {
        assert!(
            (actual - expected).abs() <= 1e-9,
            "{id}: {actual} != {expected}"
        )
    };
    let commands = chart
        .scene()
        .items()
        .iter()
        .find_map(|item| match &item.primitive {
            Primitive::Path { commands, .. } if item.layer.is_none() => Some(commands),
            _ => None,
        })
        .unwrap();
    let source = expected["domain"][0]["attributes"]["d"].as_str().unwrap();
    let mut x = 0.;
    let mut y = 0.;
    let mut vertices = vec![];
    let bytes = source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let op = bytes[i] as char;
        i += 1;
        let start = i;
        while i < bytes.len() && !matches!(bytes[i], b'M' | b'H' | b'V') {
            i += 1;
        }
        let values: Vec<f64> = source[start..i]
            .split(',')
            .map(|v| v.parse().unwrap())
            .collect();
        match op {
            'M' => {
                x = values[0];
                y = values[1];
            }
            'H' => x = values[0],
            'V' => y = values[0],
            _ => panic!("{source}"),
        }
        vertices.push((x + origin.0, y + origin.1));
    }
    assert_eq!(commands.len(), vertices.len(), "{id}");
    for (command, (x, y)) in commands.iter().zip(vertices) {
        let point = match command {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => p,
            _ => panic!("domain curve"),
        };
        close(point.x(), x);
        close(point.y(), y);
    }
    let rules: Vec<_> = chart
        .scene()
        .items()
        .iter()
        .filter_map(|item| match item.primitive {
            Primitive::Rule { from, to, .. } if item.layer.is_none() => Some((from, to)),
            _ => None,
        })
        .collect();
    assert_eq!(rules.len(), guide.ticks.len(), "{id}");
    let mut text = chart
        .scene()
        .items()
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Text {
                origin,
                text,
                font_size,
                ..
            } if item.layer.is_none() => Some((origin, text, font_size)),
            _ => None,
        });
    for ((tick, expected), (from, to)) in guide
        .ticks
        .iter()
        .zip(expected["ticks"].as_array().unwrap())
        .zip(rules)
    {
        let transform = expected["attributes"]["transform"]
            .as_str()
            .unwrap()
            .trim_start_matches("translate(")
            .trim_end_matches(')');
        let xy: Vec<f64> = transform.split(',').map(|v| v.parse().unwrap()).collect();
        close(tick.position, xy[usize::from(!horizontal)]);
        close(from.x(), origin.0 + xy[0]);
        close(from.y(), origin.1 + xy[1]);
        let line = &expected["line"];
        close(
            to.x(),
            from.x() + line["x2"].as_str().unwrap_or("0").parse::<f64>().unwrap(),
        );
        close(
            to.y(),
            from.y() + line["y2"].as_str().unwrap_or("0").parse::<f64>().unwrap(),
        );
        if !tick.label.is_empty() {
            let (painted, label, size) = text.next().unwrap();
            assert_eq!(label, &tick.label);
            close(*size, 10.);
            let a = &expected["text"]["attributes"];
            let anchor_x =
                origin.0 + xy[0] + a["x"].as_str().unwrap_or("0").parse::<f64>().unwrap();
            let anchor_y =
                origin.1 + xy[1] + a["y"].as_str().unwrap_or("0").parse::<f64>().unwrap();
            let width = label.chars().count() as f64 * 5.;
            close(
                painted.x(),
                anchor_x
                    - match side {
                        AxisSide::Bottom | AxisSide::Top => width / 2.,
                        AxisSide::Left => width,
                        AxisSide::Right => 0.,
                    },
            );
            close(
                painted.y(),
                anchor_y
                    + a["dy"]
                        .as_str()
                        .unwrap()
                        .trim_end_matches("em")
                        .parse::<f64>()
                        .unwrap()
                        * 10.,
            );
        }
    }
}

#[test]
fn geometry_signed_controls_clipping_policy_reset_and_resource_failures_are_independent() {
    use chart_core::layout::{GuideGeometry, GuideLabelPolicy, GuideOverflow};
    use chart_core::scene::Primitive;
    let p = prepare(&numeric_plot(Arc::default()));
    let mut r = configured_request();
    r.axes[1].visible = false;
    r.axes[0].tick_values = Some(vec![0.5.into(), 0.5.into(), 0.5.into()]);
    r.axes[0].tick_format = Some(GuideFormatter::Labels(vec!["same".into(); 3]));
    r.axes[0].geometry = Some(GuideGeometry {
        inner: Some(-40.),
        outer: Some(-6.),
        padding: Some(-3.),
        offset: Some(0.),
        overflow: GuideOverflow::Clip,
        clip_ticks: true,
        ..Default::default()
    });
    let full = layout(p.clone(), &r, &Metrics).unwrap();
    assert_eq!(full.guides()[&GuideId::new(0)].ticks.len(), 3);
    let rules: Vec<_> = full
        .scene()
        .items()
        .iter()
        .filter(|i| matches!(i.primitive, Primitive::Rule { .. }))
        .collect();
    assert_eq!(rules.len(), 3);
    assert!(rules.iter().all(|i| i.clip == full.plot()));
    let text_count = |c: &chart_core::layout::LaidOutChart| {
        c.scene()
            .items()
            .iter()
            .filter(|i| matches!(i.primitive, Primitive::Text { .. }))
            .count()
    };
    assert_eq!(text_count(&full), 3);
    r.axes[0].geometry.as_mut().unwrap().labels = Some(GuideLabelPolicy::HideLabels);
    let hidden = layout(p.clone(), &r, &Metrics).unwrap();
    assert_eq!(hidden.guides()[&GuideId::new(0)].ticks.len(), 3);
    assert_eq!(text_count(&hidden), 1);
    r.axes[0].geometry.as_mut().unwrap().labels = Some(GuideLabelPolicy::ThinTicks);
    assert_eq!(
        layout(p.clone(), &r, &Metrics).unwrap().guides()[&GuideId::new(0)]
            .ticks
            .len(),
        1
    );
    r.axes[0].geometry.as_mut().unwrap().offset = Some(f64::NAN);
    assert_eq!(
        layout(p.clone(), &r, &Metrics).unwrap_err().code,
        DiagnosticCode::Validation
    );
    r.axes[0].geometry = None;
    r.limits.max_path_commands = 3;
    assert_eq!(
        layout(p.clone(), &r, &Metrics).unwrap_err().code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(full.guides()[&GuideId::new(0)].ticks.len(), 3);
    let authored = numeric_plot(Arc::default())
        .edit()
        .x_axis(
            x_axis()
                .tick_size(-9.)
                .tick_size_inner(-40.)
                .tick_padding(-3.)
                .tick_offset(Some(0.)),
        )
        .build()
        .unwrap();
    let wire = authored.to_json().unwrap();
    assert_eq!(serde_json::from_str::<Json>(&wire).unwrap()["version"], 13);
    assert_eq!(Plot::from_json(&wire).unwrap().to_json().unwrap(), wire);
    let reset = authored
        .edit()
        .x_axis(x_axis().guide_geometry(None))
        .build()
        .unwrap();
    assert!(reset.definition().axes.iter().all(|a| a.geometry.is_none()));
}

#[test]
fn signed_geometry_survives_shared_facets_resize_and_translation_without_mark_changes() {
    use chart_core::{
        layout::GuideGeometry,
        scene::{PathCommand, Primitive},
    };
    let data = Data::columns()
        .column("x", [0., 1., 0., 1.])
        .column("y", [0., 1., 1., 0.])
        .column("panel", categorical(["A", "A", "B", "B"]))
        .build()
        .unwrap();
    let p = plot(data)
        .aes(aes().x("x").y("y"))
        .layer(points())
        .facet(facet_wrap("panel").columns(2))
        .x_axis(
            x_axis()
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_values(Some(vec![0f64.into(), 1f64.into()]))
                .guide_geometry(Some(GuideGeometry {
                    inner: Some(-40.),
                    outer: Some(6.),
                    offset: Some(0.),
                    ..Default::default()
                })),
        )
        .guide(
            axis_guide("top", "x")
                .side(AxisSide::Top)
                .translate(7., -11.)
                .guide_profile(GuideProfile::D3_3_0_0)
                .tick_offset(Some(0.))
                .tick_values(Some(vec![0f64.into(), 1f64.into()])),
        )
        .build()
        .unwrap();
    let prepared = prepare(&p);
    for width in [600., 900.] {
        let mut r = request();
        r.bounds = Rect::new(0., 0., width, 400.).unwrap();
        let c = layout(prepared.clone(), &r, &Metrics).unwrap();
        assert_eq!(c.panels().len(), 2);
        let scopes: std::collections::BTreeSet<_> = c
            .scene()
            .items()
            .iter()
            .filter_map(|i| i.guide.as_ref())
            .map(|g| g.scope.clone())
            .collect();
        assert_eq!(scopes.len(), 2);
        assert!(
            scopes
                .iter()
                .all(|scope| scope.len() == 1 && scope[0].starts_with("panel:"))
        );
        for panel in c.panels() {
            let bottom = panel
                .chart
                .guides()
                .values()
                .find(|g| g.spec.side == AxisSide::Bottom)
                .unwrap();
            let top = panel
                .chart
                .guides()
                .values()
                .find(|g| g.spec.side == AxisSide::Top)
                .unwrap();
            for (a, b) in bottom.ticks.iter().zip(&top.ticks) {
                assert_eq!(a.value, b.value);
                assert!((b.position - a.position - 7.).abs() < 1e-9);
            }
            let bounds = panel.chart.plot().unwrap();
            let paths: Vec<_> = panel
                .chart
                .scene()
                .items()
                .iter()
                .filter_map(|i| {
                    if let Primitive::Path { commands, .. } = &i.primitive {
                        Some(commands)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(paths.len(), 2);
            assert!(paths.iter().any(|commands| matches!(commands[0],PathCommand::MoveTo(p) if (p.x()-bounds.origin().x()).abs()<1e-9 && (p.y()-bounds.max_y()-6.).abs()<1e-9)));
            let points: Vec<_> = panel
                .chart
                .scene()
                .items()
                .iter()
                .filter_map(|i| {
                    if let Primitive::Point { center, .. } = i.primitive {
                        Some(center)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(points.len(), 2);
            assert!((points[0].x() - bottom.ticks[0].position).abs() < 1e-9);
            assert!((points[1].x() - bottom.ticks[1].position).abs() < 1e-9);
        }
    }
}
