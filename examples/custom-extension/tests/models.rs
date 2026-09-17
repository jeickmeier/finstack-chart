//! External application model contract, independent closed-form expected predictions.
use chart_core::{DiagnosticCode, Revision, grammar::*, prelude::*};
use chart_extension_example::models::*;
use std::sync::Arc;
fn author(
    registry: Arc<ExtensionRegistry>,
    operation: &str,
    parameters: serde_json::Value,
    se: bool,
) -> ChartResult<Plot> {
    plot(
        Data::columns()
            .column("x", [0., 1., 2.])
            .column("y", [2., 5., 10.])
            .column("w", [1., 2., 1.])
            .build()?,
    )
    .extensions(registry)
    .layer(
        smooth().stat(
            model_stat(ModelOptions {
                method: ModelMethod::Registered {
                    operation: OperationRef::new(operation, Revision::new(1)),
                    parameters,
                },
                xseq: Some(vec![-1., 0., 1., 2., 3.]),
                level: 0.8,
                se,
                ..Default::default()
            })
            .x("x")
            .y("y")
            .weight("w"),
        ),
    )
    .build()
}
#[test]
fn weighted_model_consumes_grid_options_and_retains_registry_across_prepare_and_replay() {
    let registry = chart_extension_example::registry().unwrap();
    let weak = Arc::downgrade(&registry);
    let plot = author(
        registry,
        PRESCRIBED_SLOPE,
        serde_json::json!({"slope":2.,"envelope":1.25}),
        true,
    )
    .unwrap();
    let wire = plot.to_json().unwrap();
    assert!(Plot::from_json(&wire).is_err());
    assert!(weak.upgrade().is_some());
    let restored =
        Plot::from_json_with_extensions(&wire, chart_extension_example::registry().unwrap())
            .unwrap();
    for p in [&plot, &restored, &plot] {
        let prepared = p.chart().unwrap().prepare().unwrap();
        let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
            panic!("generated model rows")
        };
        assert_eq!(rows.len(), 5);
        for (i, row) in rows.iter().enumerate() {
            assert_eq!(row.value(&StatField::X), Some(i as f64 - 1.));
            let expected = 1.5 + 2. * i as f64;
            assert_eq!(row.value(&StatField::Y), Some(expected));
            assert_eq!(row.value(&StatField::Lower), Some(expected - 1.));
            assert_eq!(row.value(&StatField::Upper), Some(expected + 1.));
        }
    }
    let p = author(
        chart_extension_example::registry().unwrap(),
        PRESCRIBED_SLOPE,
        serde_json::json!({"slope":2.,"envelope":1.25}),
        false,
    )
    .unwrap();
    let prepared = p.chart().unwrap().prepare().unwrap();
    let PreparedRows::Statistical(rows) = prepared.layers()[0].table().rows() else {
        panic!()
    };
    assert!(
        rows.iter()
            .all(|r| r.value(&StatField::Lower).is_none() && r.value(&StatField::Upper).is_none())
    );
}
#[test]
fn model_schema_and_portability_are_explicit() {
    for parameters in [
        serde_json::json!({"slope":2.}),
        serde_json::json!({"slope":2.,"envelope":-1.}),
        serde_json::json!({"slope":2.,"envelope":1.,"ignored":true}),
    ] {
        assert!(
            author(
                chart_extension_example::registry().unwrap(),
                PRESCRIBED_SLOPE,
                parameters,
                true
            )
            .is_err()
        );
    }
    let native = author(
        chart_extension_example::registry().unwrap(),
        NATIVE_PRESCRIBED_SLOPE,
        serde_json::json!({"slope":2.,"envelope":1.25}),
        true,
    )
    .unwrap();
    assert!(native.chart().unwrap().prepare().is_ok());
    assert_eq!(
        native.to_json().unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
}
struct Malformed(u8);
impl CustomModel for Malformed {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.malformed", Revision::new(1), true)
    }
    fn validate(&self, _: &serde_json::Value) -> ChartResult<()> {
        Ok(())
    }
    fn predict(
        &self,
        input: ModelInput<'_>,
        _: &serde_json::Value,
    ) -> ChartResult<Vec<ModelEstimate>> {
        let mut values = input
            .grid
            .iter()
            .map(|_| ModelEstimate {
                mean: Some(1.),
                standard_error: Some(0.),
                lower: Some(0.),
                upper: Some(2.),
            })
            .collect::<Vec<_>>();
        match self.0 {
            0 => {
                values.pop();
            }
            1 => values[0].mean = Some(f64::NAN),
            2 => values[0].standard_error = Some(-1.),
            _ => values[0].lower = Some(3.),
        }
        Ok(values)
    }
}
#[test]
fn malformed_registered_predictions_fail_before_geometry() {
    for case in 0..4 {
        let mut registry = ExtensionRegistry::new();
        registry.register_model(Arc::new(Malformed(case))).unwrap();
        let p = author(
            Arc::new(registry),
            "example.malformed",
            serde_json::Value::Null,
            true,
        )
        .unwrap();
        assert_eq!(
            p.chart().unwrap().prepare().unwrap_err().code,
            DiagnosticCode::SchemaConflict
        );
    }
}
