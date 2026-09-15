//! FIX-GG04: secondary break and label callbacks through the shared guide engine.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision, composition::ScaleValue, data::TimeUnit, grammar::*,
    interpolate::Number, layout::*, prelude::*, scales::*, services::*,
};
use serde_json::{Value as Json, json};
use std::sync::{Arc, Mutex};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn factor(t: &Json, unit: TimeUnit) -> f64 {
    (match unit {
        TimeUnit::Seconds => 1.,
        TimeUnit::Milliseconds => 1000.,
        TimeUnit::Microseconds => 1000000.,
        TimeUnit::Nanoseconds => 1000000000.,
    }) * if t["family"] == "date" { 86400. } else { 1. }
}
fn data(t: &Json, unit: TimeUnit) -> ChartResult<Data> {
    let v = t["inputs"].as_array().unwrap();
    let d = Data::columns().column("y", vec![1.; v.len()]);
    match t["family"].as_str().unwrap() {
        "discrete" => d
            .column(
                "x",
                categorical(v.iter().map(|v| v.as_str().unwrap_or("")))
                    .validity(v.iter().map(|v| !v.is_null()).collect()),
            )
            .build(),
        "date" | "datetime" => {
            let factor = factor(t, unit);
            d.column(
                "x",
                timestamps(
                    v.iter()
                        .map(|v| (v.as_f64().unwrap_or(0.) * factor) as i64)
                        .collect::<Vec<_>>(),
                    unit,
                    "UTC",
                )
                .validity(v.iter().map(|v| !v.is_null()).collect()),
            )
            .build()
        }
        _ => d
            .column("x", v.iter().map(Json::as_f64).collect::<Vec<_>>())
            .build(),
    }
}
fn build(t: &Json, unit: TimeUnit, registry: Arc<ExtensionRegistry>) -> ChartResult<Plot> {
    let mut second = x_axis().name("secondary").side(AxisSide::Top);
    second = match t["conversion"].as_str().unwrap() {
        "affine" => second.secondary("x", 2., 3.),
        "square" => second.secondary_transform(
            "x",
            NumericScaleSpec {
                domain: vec![Number(0.), Number(20.)],
                range: vec![Number(0.), Number(400.)],
                ..NumericScaleSpec::d3(NumericFamily::Ggplot {
                    transform: GgplotTransform::Registered {
                        selection: Box::new(TransformSelection::new(TransformOperation {
                            operation: OperationRef::new(
                                "example.scale_transform",
                                Revision::new(1),
                            ),
                            parameters: json!({"family":"square","custom":false}),
                        })),
                    },
                })
            },
        ),
        "shift" => second.secondary("x", 1., 2.),
        _ => second.secondary("x", 1., 0.),
    };
    let mode = match t["mode"].as_str().unwrap() {
        "typed_empty" => "empty",
        "empty" if t["family"] == "date" || t["family"] == "datetime" => "untyped_empty",
        x => x,
    };
    second = second
        .breaks_function(Some(ScaleBreaksOperation {
            operation: OperationRef::new("example.breaks_limits", Revision::new(1)),
            parameters: json!(mode),
        }))
        .guide_geometry(Some(GuideGeometry {
            labels: Some(GuideLabelPolicy::Preserve),
            ..Default::default()
        }));
    if t["label_mode"] == "function" {
        second = second.tick_format(Some(GuideFormatter::Registered {
            operation: OperationRef::new("example.scale_labels", Revision::new(1)),
            parameters: json!("indexed"),
        }));
    }
    let scale = match t["family"].as_str().unwrap() {
        "date" => scale_date(),
        "datetime" => scale_utc(),
        "discrete" => scale_band(),
        _ => scale_linear(),
    };
    plot(data(t, unit)?)
        .profile(Profile::Ggplot2_4_0_3)
        .extensions(registry)
        .aes(
            aes()
                .x(if t["family"] == "date" || t["family"] == "datetime" {
                    Mapping::Timestamp {
                        field: "x".into(),
                        origin: 0,
                    }
                } else {
                    Mapping::Field("x".into())
                })
                .y("y"),
        )
        .layer(points())
        .x_axis(
            x_axis()
                .scale(scale)
                .range(0., 100.)
                .expansion((t["expand"] == false).then_some(GgplotExpansion {
                    mult: [0.; 2],
                    add: [0.; 2],
                })),
        )
        .axis(second)
        .build()
}
fn check(
    t: &Json,
    r: &LayoutRequest,
    unit: TimeUnit,
    registry: Arc<ExtensionRegistry>,
) -> Result<(), String> {
    let attempt = (|| -> ChartResult<_> {
        let p = build(t, unit, registry.clone())?;
        assert_eq!(p.definition().wire_version(), 62);
        let wire = p.to_json()?;
        assert!(Plot::from_json(&wire).is_err());
        let restored = Plot::from_json_with_extensions(&wire, registry.clone())?;
        assert_eq!(restored.to_json()?, wire);
        let mut downgraded: Json = serde_json::from_str(&wire).unwrap();
        downgraded["version"] = 61.into();
        assert!(
            Plot::from_json_with_extensions(&downgraded.to_string(), registry.clone()).is_err()
        );
        let id = restored.axis("secondary")?.id();
        let f = layout(restored.chart()?.prepare()?, r, &Metrics)?;
        Ok((f, id))
    })();
    let e = &t["result"];
    if e["error"].is_string() {
        return if attempt.is_err() {
            Ok(())
        } else {
            Err("expected source rejection".into())
        };
    }
    let (f, id) = attempt.map_err(|e| format!("{e:?}"))?;
    let a = &f.axes()[&id];
    let positions = e["positions"].as_array().unwrap();
    if a.ticks.len() != positions.len() {
        return Err(format!(
            "tick count {} vs {}",
            a.ticks.len(),
            positions.len()
        ));
    }
    for (i, tick) in a.ticks.iter().enumerate() {
        let expected = positions[i].as_f64().unwrap();
        if !tick.position.is_finite() || (tick.position / 100. - expected).abs() > 2e-12 {
            return Err(format!("position {} vs {expected}", tick.position / 100.));
        }
        if tick.label != e["labels"][i].as_str().unwrap_or("NA") {
            return Err(format!("label {:?} vs {}", tick.label, e["labels"][i]));
        }
        let actual = match (&tick.value, &a.space) {
            (ScaleValue::Timestamp { value, .. }, _) => *value as f64 / factor(t, unit),
            (ScaleValue::Number(v), ValueSpace::Timestamp { origin, .. }) => {
                (*v + *origin as f64) / factor(t, unit)
            }
            (ScaleValue::Number(v), _) => *v,
            _ => return Err("unexpected key type".into()),
        };
        if let Some(expected) = e["user_values"][i].as_f64()
            && (!actual.is_finite() || (actual - expected).abs() > 2e-12)
        {
            return Err(format!("value {actual} vs {expected}"));
        }
    }
    Ok(())
}
fn encoded(v: f64) -> Json {
    if v.is_nan() {
        Json::Null
    } else if v.is_infinite() {
        json!(if v > 0. { "Infinity" } else { "-Infinity" })
    } else {
        json!(v)
    }
}
fn metadata(
    values: impl Iterator<Item = f64>,
    names: Option<&[String]>,
    context: Option<GuideTemporalContext<'_>>,
) -> Json {
    let (class, zone) = if let Some(t) = context {
        (
            if t.normalization.date {
                vec!["Date"]
            } else {
                vec!["POSIXct", "POSIXt"]
            },
            if t.normalization.date {
                vec![]
            } else {
                vec![match t.zone {
                    CalendarZone::Utc => "UTC",
                    CalendarZone::Local(r) => &r.zone,
                }]
            },
        )
    } else {
        (vec!["numeric"], vec![])
    };
    json!({"values":values.map(|v|encoded(context.map_or(v,|t|{ let n=t.normalization; let f=(match n.unit {TimeUnit::Seconds=>1.,TimeUnit::Milliseconds=>1000.,TimeUnit::Microseconds=>1000000.,TimeUnit::Nanoseconds=>1000000000.}) * if n.date {86400.}else{1.}; n.origin as f64/f+v/f }))).collect::<Vec<_>>(),"names":names.unwrap_or(&[]),"classes":class,"timezone":zone})
}
struct LoggedBreaks(Arc<Mutex<Vec<Json>>>);
impl CustomScaleBreaks for LoggedBreaks {
    fn descriptor(&self) -> ExtensionDescriptor {
        chart_extension_example::scale_breaks::Breaks("limits").descriptor()
    }
    fn validate(&self, p: &Json) -> ChartResult<()> {
        chart_extension_example::scale_breaks::Breaks("limits").validate(p)
    }
    fn evaluate(&self, input: ScaleBreaksInput<'_>) -> ChartResult<ScaleBreaksOutput> {
        assert_eq!(input.count, None);
        assert_eq!(input.count_argument, None);
        self.0.lock().unwrap().push(metadata(
            input.domain.iter().map(|v| match v {
                ScaleKey::Number(n) => n.0,
                _ => panic!("numeric limits"),
            }),
            None,
            input.temporal,
        ));
        chart_extension_example::scale_breaks::Breaks("limits").evaluate(input)
    }
}
struct LoggedLabels(Arc<Mutex<Vec<Json>>>);
impl CustomGuideFormatter for LoggedLabels {
    fn descriptor(&self) -> ExtensionDescriptor {
        chart_extension_example::scale_labels::Formatter.descriptor()
    }
    fn validate(&self, p: &Json) -> ChartResult<()> {
        chart_extension_example::scale_labels::Formatter.validate(p)
    }
    fn format_labels(&self, input: GuideLabelsInput<'_>) -> ChartResult<Vec<Option<String>>> {
        self.0.lock().unwrap().push(metadata(
            input.values.iter().map(|v| match v {
                ScaleValue::Number(n) => *n,
                _ => panic!("numeric labels"),
            }),
            input.names,
            input.temporal,
        ));
        chart_extension_example::scale_labels::Formatter.format_labels(input)
    }
}
fn recorded_match(actual: &[Json], expected: &Json) -> Result<(), String> {
    let expected = expected.as_array().unwrap();
    if actual.is_empty() != expected.is_empty() {
        return Err(format!("callback presence {actual:?} vs {expected:?}"));
    }
    for a in actual {
        let agrees = expected.iter().any(|e| {
            // The source's empty tzone attribute inherits the explicitly pinned UTC resource.
            let zone = e["timezone"]
                .as_array()
                .unwrap()
                .iter()
                .map(|z| if z == "" { json!("UTC") } else { z.clone() })
                .collect::<Vec<_>>();
            a["classes"] == e["classes"]
                && a["names"] == e["names"]
                && a["timezone"] == json!(zone)
                && a["values"].as_array().unwrap().len() == e["values"].as_array().unwrap().len()
                && a["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(e["values"].as_array().unwrap())
                    .all(|(a, e)| match (a.as_f64(), e.as_f64()) {
                        (Some(a), Some(e)) => (a - e).abs() < 2e-12,
                        _ => a == e,
                    })
        });
        if !agrees {
            return Err(format!("callback {a} vs {expected:?}"));
        }
    }
    Ok(())
}
#[test]
fn secondary_guide_functions_match_reference() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/secondary-guide-functions.json"
    ))
    .unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 292);
    let r = LayoutRequest::new(
        Rect::new(0., 0., 640., 360.).unwrap(),
        Units::LogicalPixels,
        ResourceDescriptor {
            id: ResourceId::new(1),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        },
    );
    let mut errors = vec![];
    let mut count = 0;
    for t in cases {
        for unit in [
            TimeUnit::Seconds,
            TimeUnit::Milliseconds,
            TimeUnit::Microseconds,
            TimeUnit::Nanoseconds,
        ] {
            if t["family"] != "date" && t["family"] != "datetime" && unit != TimeUnit::Milliseconds
            {
                continue;
            }
            count += 1;
            let breaks = Arc::new(Mutex::new(vec![]));
            let labels = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::new();
            registry
                .register_scale_breaks(Arc::new(LoggedBreaks(breaks.clone())))
                .unwrap();
            registry
                .register_guide_formatter(Arc::new(LoggedLabels(labels.clone())))
                .unwrap();
            registry
                .register_transform(Arc::new(
                    chart_extension_example::scale_transforms::Transform { portable: true },
                ))
                .unwrap();
            let result = check(t, &r, unit, Arc::new(registry))
                .and_then(|()| recorded_match(&breaks.lock().unwrap(), &t["calls"]))
                .and_then(|()| recorded_match(&labels.lock().unwrap(), &t["label_calls"]));
            if let Err(e) = result {
                errors.push(format!(
                    "{} {} {} {} {} {} {unit:?}: {e}",
                    t["family"],
                    t["conversion"],
                    t["population"],
                    t["mode"],
                    t["label_mode"],
                    t["expand"]
                ));
            }
        }
    }
    assert_eq!(count, 778);
    assert!(
        errors.is_empty(),
        "{} failures:\n{}",
        errors.len(),
        errors.join("\n")
    );
}
