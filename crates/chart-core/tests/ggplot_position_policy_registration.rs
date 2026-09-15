//! FIX-GG04: registered label vectors against pinned source and guide-key results.
use chart_core::{
    ChartResult, Rect, ResourceId, Revision,
    composition::ScaleValue,
    grammar::{CustomGuideFormatter, ExtensionDescriptor, ExtensionRegistry, GuideLabelsInput},
    layout::*,
    prelude::*,
    services::*,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
struct Formatter(Arc<Mutex<Vec<Value>>>, bool);
impl CustomGuideFormatter for Formatter {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.scale_labels", Revision::new(1), self.1)
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn format_labels(&self, input: GuideLabelsInput<'_>) -> ChartResult<Vec<Option<String>>> {
        let values = input
            .values
            .iter()
            .map(|v| match v {
                ScaleValue::Number(v) if v.is_nan() => Value::Null,
                ScaleValue::Number(v) if v.is_infinite() => {
                    json!(if *v > 0. { "Infinity" } else { "-Infinity" })
                }
                ScaleValue::Number(v) => json!(v),
                ScaleValue::Category(v) => json!(v),
                ScaleValue::MissingCategory => Value::Null,
                _ => panic!("unexpected timestamp"),
            })
            .collect::<Vec<_>>();
        self.0
            .lock()
            .unwrap()
            .push(json!({"values":values,"names":input.names.unwrap_or(&[])}));
        let mut labels = if input.values.is_empty() {
            vec![Some("/0".into())]
        } else {
            (1..=input.values.len())
                .map(|i| Some(format!("{i}/{}", input.values.len())))
                .collect::<Vec<_>>()
        };
        match input.parameters.as_str().unwrap() {
            "missing" => {
                for (i, label) in labels.iter_mut().enumerate() {
                    if i % 2 == 1 {
                        *label = None;
                    }
                }
            }
            "short" => labels.truncate(1),
            "empty" => labels.clear(),
            "indexed" | "named" => {}
            _ => panic!("unexpected mode"),
        }
        Ok(labels)
    }
}

struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}
fn registered() -> chart_core::scales::GgplotGuideLabels {
    chart_core::scales::GgplotGuideLabels::Registered {
        operation: chart_core::grammar::OperationRef::new("test.scale_labels", Revision::new(1)),
        parameters: json!("indexed"),
    }
}
fn authored(
    registry: Arc<ExtensionRegistry>,
    discrete: bool,
    override_labels: bool,
) -> ChartResult<Plot> {
    use chart_core::scales::*;
    let data = Data::columns()
        .column("x", vec![1., 2.])
        .column("category", categorical(["a", "b"]))
        .build()?;
    let mut axis = if discrete {
        x_axis()
            .scale(scale_band())
            .discrete_policy(Some(GgplotDiscretePosition {
                guide: GgplotDiscreteGuide {
                    labels: registered(),
                    ..Default::default()
                },
                ..Default::default()
            }))
    } else {
        x_axis().scale(scale_binned(GgplotBinnedPosition {
            bins: GgplotBinnedPolicy {
                limits: Some([Some(1.0.into()), Some(2.0.into())]),
                breaks: GgplotBreaks::Explicit(vec![1.0.into(), 2.0.into()]),
                ..Default::default()
            },
            labels: registered(),
            ..Default::default()
        }))
    };
    if override_labels {
        axis = axis.tick_format(Some(GuideFormatter::Labels(vec![
            "left".into(),
            "right".into(),
        ])));
    }
    plot(data)
        .extensions(registry)
        .profile(Profile::Ggplot2_4_0_3)
        .aes(aes().x(if discrete { "category" } else { "x" }).y(1.))
        .layer(points())
        .x_axis(axis)
        .build()
}
#[test]
fn positional_policy_registrations_are_portable_versioned_and_deferred() {
    for discrete in [false, true] {
        for portable in [false, true] {
            let calls = Arc::new(Mutex::new(vec![]));
            let mut registry = ExtensionRegistry::new();
            registry
                .register_guide_formatter(Arc::new(Formatter(calls.clone(), portable)))
                .unwrap();
            let registry = Arc::new(registry);
            let plot = authored(registry.clone(), discrete, false).unwrap();
            assert_eq!(plot.definition().wire_version(), 32);
            let prepared = plot.chart().unwrap().prepare().unwrap();
            assert!(calls.lock().unwrap().is_empty());
            if portable {
                let wire = plot.to_json().unwrap();
                assert!(Plot::from_json(&wire).is_err());
                let restored = Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
                assert_eq!(restored.to_json().unwrap(), wire);
                let mut downgraded: Value = serde_json::from_str(&wire).unwrap();
                downgraded["version"] = json!(31);
                assert!(
                    Plot::from_json_with_extensions(&downgraded.to_string(), registry.clone())
                        .is_err()
                );
            } else {
                assert_eq!(
                    plot.to_json().unwrap_err().code,
                    chart_core::DiagnosticCode::UnsupportedCapability
                );
            }
            assert!(authored(Arc::new(ExtensionRegistry::new()), discrete, false).is_err());
            assert!(calls.lock().unwrap().is_empty());
            let request = |units| {
                LayoutRequest::new(
                    Rect::new(0., 0., 640., 360.).unwrap(),
                    units,
                    ResourceDescriptor {
                        id: ResourceId::new(1),
                        revision: Revision::INITIAL,
                        kind: ResourceKind::Font,
                        byte_len: 1,
                    },
                )
            };
            layout(prepared.clone(), &request(Units::LogicalPixels), &Metrics).unwrap();
            assert!(!calls.lock().unwrap().is_empty());
            calls.lock().unwrap().clear();
            if !portable {
                assert_eq!(
                    layout(prepared, &request(Units::Points), &Metrics)
                        .unwrap_err()
                        .code,
                    chart_core::DiagnosticCode::UnsupportedCapability
                );
                assert!(calls.lock().unwrap().is_empty());
            }
            let overridden = authored(registry, discrete, true)
                .unwrap()
                .chart()
                .unwrap()
                .prepare()
                .unwrap();
            let frame = layout(overridden, &request(Units::LogicalPixels), &Metrics).unwrap();
            assert!(calls.lock().unwrap().is_empty());
            assert_eq!(
                frame.guides()[&chart_core::GuideId::new(0)]
                    .ticks
                    .iter()
                    .map(|t| t.label.as_str())
                    .collect::<Vec<_>>(),
                ["left", "right"]
            );
        }
    }
}
