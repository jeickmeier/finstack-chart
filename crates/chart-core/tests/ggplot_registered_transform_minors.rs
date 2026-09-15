//! FIX-GG04 independent reference chart outcomes for registered pointwise transforms.
use chart_core::grammar::{
    CustomTransformFactory, ExtensionDescriptor, ExtensionRegistry, OperationRef,
    PointwiseTransform, TransformOperation, TransformSelection,
};
use serde_json::Value;
use std::sync::Arc;
type Trace = Arc<
    std::sync::Mutex<
        Vec<(
            Vec<chart_core::interpolate::Number>,
            [chart_core::interpolate::Number; 2],
        )>,
    >,
>;
struct Factory(Trace);
struct Kernel {
    trace: Trace,
    cubic: bool,
    mode: String,
}
impl PointwiseTransform for Kernel {
    fn forward(&self, x: f64) -> f64 {
        if self.cubic { x * x * x } else { 2. * x + 3. }
    }
    fn inverse(&self, x: f64) -> f64 {
        if self.cubic {
            if x == 0. {
                x
            } else {
                x.signum() * x.abs().powf(1. / 3.)
            }
        } else {
            (x - 3.) / 2.
        }
    }
    fn domain(&self) -> [chart_core::interpolate::Number; 2] {
        [f64::NEG_INFINITY, f64::INFINITY].map(chart_core::interpolate::Number)
    }
    fn monotone_on(&self, _: [f64; 2]) -> bool {
        true
    }
    fn minor_breaks(
        &self,
        major: &[chart_core::interpolate::Number],
        limits: [chart_core::interpolate::Number; 2],
        n: usize,
    ) -> ChartResult<Option<Vec<chart_core::interpolate::Number>>> {
        assert_eq!(n, 2);
        if self.mode != "default" {
            self.trace.lock().unwrap().push((major.to_vec(), limits));
        }
        if self.mode == "error" {
            return Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::NumericalDomain,
                format!("Minor callback failure: {limits:?}"),
                "Use valid callback parameters.",
            ));
        }
        Ok(match self.mode.as_str() {
            "oversized" => Some(vec![0.; 4097]),
            "fixed" => Some(vec![-1., 0., 1.]),
            "limits" => Some(vec![
                limits[0].0,
                (limits[0].0 + limits[1].0) / 2.,
                limits[1].0,
            ]),
            _ => None,
        }
        .map(|v| v.into_iter().map(chart_core::interpolate::Number).collect()))
    }
}
impl CustomTransformFactory for Factory {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.reference_transform", Revision::new(1), true)
    }
    fn validate(&self, p: &Value) -> ChartResult<()> {
        if matches!(p["family"].as_str(), Some("affine" | "cubic")) && p["mode"].is_string() {
            Ok(())
        } else {
            Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Invalid reference transform.",
                "Use a captured configuration.",
            ))
        }
    }
    fn compile(&self, p: &Value) -> ChartResult<Arc<dyn PointwiseTransform>> {
        Ok(Arc::new(Kernel {
            trace: self.0.clone(),
            cubic: p["family"] == "cubic",
            mode: p["mode"].as_str().unwrap().into(),
        }))
    }
}
struct ExplicitMinor;
impl chart_core::grammar::CustomScaleBreaks for ExplicitMinor {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("test.explicit_minor", Revision::new(1), true)
    }
    fn validate(&self, _: &Value) -> ChartResult<()> {
        Ok(())
    }
    fn evaluate(
        &self,
        _: chart_core::grammar::ScaleBreaksInput<'_>,
    ) -> ChartResult<chart_core::grammar::ScaleBreaksOutput> {
        Ok(chart_core::grammar::ScaleBreaksOutput {
            values: Some(
                vec![-1.5, 0.5]
                    .into_iter()
                    .map(|v| ScaleKey::Number(chart_core::interpolate::Number(v)))
                    .collect(),
            ),
            ..Default::default()
        })
    }
}
fn registry(trace: Trace) -> Arc<ExtensionRegistry> {
    let mut r = ExtensionRegistry::new();
    r.register_transform(Arc::new(Factory(trace))).unwrap();
    r.register_scale_breaks(Arc::new(ExplicitMinor)).unwrap();
    Arc::new(r)
}
fn transform(c: &Value) -> GgplotTransform {
    GgplotTransform::Registered {
        selection: TransformSelection::new(TransformOperation {
            operation: OperationRef {
                id: "test.reference_transform".into(),
                version: Revision::new(1),
            },
            parameters: serde_json::json!({"family":c["family"],"mode":c["mode"]}),
        })
        .into(),
    }
}
fn number(v: &Value) -> f64 {
    v.as_f64()
        .unwrap_or_else(|| match v["number"].as_str().unwrap() {
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            "NA" | "NaN" => f64::NAN,
            s => panic!("Unknown number {s}"),
        })
}
fn close(actual: f64, expected: f64, context: &str) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "{context}: {actual} expected NaN");
    } else if expected.is_infinite() {
        assert_eq!(actual, expected, "{context}");
    } else {
        assert!(
            (actual - expected).abs() <= 4e-14 * expected.abs().max(1.),
            "{context}: actual={actual:.17e}, expected={expected:.17e}"
        );
    }
}
use chart_core::{
    ChartResult, GuideId, Rect, ResourceId, Revision, layout::*, prelude::*, scales::*, services::*,
};
struct Metrics;
impl TextMeasurer for Metrics {
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        TextMetrics::new(r.text.len() as f64 * 5., 8., 2.)
    }
}

#[test]
fn transform_minor_reference_plots() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/registered-transform-minors.json"
    ))
    .unwrap();
    for (index, c, callback) in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .flat_map(|(i, c)| {
            std::iter::once((i, c, false))
                .chain((c["override"] == "explicit").then_some((i, c, true)))
        })
    {
        let trace = Trace::default();
        let registry = registry(trace.clone());
        let config = &fixture["configurations"][c["configuration"].as_u64().unwrap() as usize];
        let mut t = transform(config);
        if config["composed"] == true {
            t = GgplotTransform::Compose {
                transforms: vec![t, GgplotTransform::Reverse],
            };
        }
        let input = c["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .map(number)
            .collect::<Vec<_>>();
        let data = Data::columns()
            .column("y", vec![1.; input.len()])
            .column("x", input)
            .build()
            .unwrap();
        let minor = match c["override"].as_str().unwrap() {
            "hidden" => Some(MinorBreaks::Hidden),
            "explicit" if callback => Some(MinorBreaks::Registered(
                chart_core::grammar::ScaleBreaksOperation {
                    operation: OperationRef {
                        id: "test.explicit_minor".into(),
                        version: Revision::new(1),
                    },
                    parameters: Value::Null,
                },
            )),
            "explicit" => Some(MinorBreaks::Numeric(
                vec![-1.5, 0.5]
                    .into_iter()
                    .map(chart_core::interpolate::Number)
                    .collect(),
            )),
            _ => None,
        };
        let p = plot(data)
            .extensions(registry.clone())
            .profile(Profile::Ggplot2_4_0_3)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .x_axis(
                x_axis()
                    .scale(chart_core::plot::scale_transform(ScaleTransform::Ggplot {
                        transform: t.clone(),
                    }))
                    .range(100., 540.)
                    .minor_breaks(minor)
                    .tick_arguments(c["count"].as_f64().map(|count| GuideTickArguments {
                        count: Some(count),
                        ..Default::default()
                    }))
                    .guide_geometry(Some(GuideGeometry {
                        labels: Some(GuideLabelPolicy::Preserve),
                        ..Default::default()
                    })),
            )
            .build()
            .unwrap();
        let wire = p.to_json().unwrap();
        let restored =
            chart_core::plot::Plot::from_json_with_extensions(&wire, registry.clone()).unwrap();
        let frame = layout(
            restored.chart().unwrap().prepare().unwrap(),
            &LayoutRequest::new(
                Rect::new(0., 0., 640., 360.).unwrap(),
                Units::LogicalPixels,
                ResourceDescriptor {
                    id: ResourceId::new(1),
                    revision: Revision::INITIAL,
                    kind: ResourceKind::Font,
                    byte_len: 1,
                },
            ),
            &Metrics,
        )
        .unwrap();
        let calls = trace.lock().unwrap();
        let expected_calls = c["result"]["build_calls"].as_array().unwrap();
        assert_eq!(
            calls.is_empty(),
            expected_calls.is_empty(),
            "callback execution case {index}"
        );
        if let Some((major, bounds)) = calls.first() {
            let expected = &expected_calls[0];
            assert_eq!(
                major.len(),
                expected["major"].as_array().unwrap().len(),
                "callback major case {index}"
            );
            for (a, e) in major.iter().zip(expected["major"].as_array().unwrap()) {
                close(a.0, number(e), &format!("callback major case {index}"));
            }
            for (a, e) in bounds.iter().zip(expected["limits"].as_array().unwrap()) {
                close(a.0, number(e), &format!("callback limits case {index}"));
            }
        }
        let actual = &frame.guides()[&GuideId::new(0)].minor_ticks;
        let expected = c["result"]["minor_positions"].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "case {index}: {c}");
        let resolved = registry.resolve_transform(&t).unwrap();
        for (a, e) in actual.iter().zip(c["result"]["minor"].as_array().unwrap()) {
            let Some(chart_core::composition::ScaleValue::Number(raw)) = a.value else {
                panic!("case {index}: expected finite raw minor");
            };
            close(
                resolved.forward(raw),
                number(e),
                &format!("minor value case {index}"),
            );
        }
        for (a, e) in actual.iter().zip(expected) {
            close(
                (a.position - 100.) / 440.,
                number(e),
                &format!("case {index}"),
            );
        }
    }
}

#[test]
fn transform_minor_errors_budgets_and_zero_range_precedence() {
    for mode in ["error", "oversized"] {
        for (hidden, collapsed) in [(false, false), (true, false), (false, true)] {
            let r = registry(Trace::default());
            let data = Data::columns()
                .column("x", if collapsed { vec![0.] } else { vec![] })
                .build()
                .unwrap();
            let p = plot(data)
                .extensions(r)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(points())
                .x_axis(
                    x_axis()
                        .scale(chart_core::plot::scale_transform(ScaleTransform::Ggplot {
                            transform: transform(
                                &serde_json::json!({"family":"affine", "mode":mode}),
                            ),
                        }))
                        .minor_breaks(hidden.then_some(MinorBreaks::Hidden))
                        .expansion(collapsed.then_some(GgplotExpansion {
                            mult: [0., 0.],
                            add: [0., 0.],
                        })),
                )
                .build()
                .unwrap();
            let result = layout(
                p.chart().unwrap().prepare().unwrap(),
                &LayoutRequest::new(
                    Rect::new(0., 0., 640., 360.).unwrap(),
                    Units::LogicalPixels,
                    ResourceDescriptor {
                        id: ResourceId::new(1),
                        revision: Revision::INITIAL,
                        kind: ResourceKind::Font,
                        byte_len: 1,
                    },
                ),
                &Metrics,
            );
            if hidden || collapsed {
                assert!(
                    result.is_ok(),
                    "{mode} hidden={hidden} collapsed={collapsed}: {:?}",
                    result.as_ref().err()
                );
            } else {
                let e = result.expect_err("default minor failure must propagate on empty data");
                assert_eq!(
                    e.code,
                    if mode == "error" {
                        chart_core::DiagnosticCode::NumericalDomain
                    } else {
                        chart_core::DiagnosticCode::ResourceLimit
                    }
                );
            }
        }
    }
}
