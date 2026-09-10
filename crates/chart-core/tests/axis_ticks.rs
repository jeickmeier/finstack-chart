//! WP-AX02 / FIX-19: independent guide selection and formatting over shared scales.
use chart_core::{
    ChartResult, DiagnosticCode, GuideId, Rect, ResourceId, Revision,
    composition::ScaleValue,
    data::TimeUnit,
    grammar::{
        Compiler, CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideFormatInput,
        OperationRef, PreparedChart,
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
    if let Some(arguments) = config.get("arguments") {
        let a = arguments.as_array().unwrap();
        style.tick_arguments = Some(GuideTickArguments {
            count: a.first().and_then(Json::as_f64),
            specifier: a.get(1).and_then(Json::as_str).map(Into::into),
            interval: None,
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
