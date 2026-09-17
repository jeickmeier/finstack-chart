//! Typed materialization/recipe reuse validates schema, budgets and portable boundaries.
use chart_core::{
    Revision,
    data::DataLimits,
    grammar::OperationRef,
    plot::{AuthoringSelection, autoplot},
};
#[test]
fn materialization_and_composable_dispatch_share_existing_compiler() {
    let registry = chart_extension_example::registry().unwrap();
    let op = OperationRef::new(
        chart_extension_example::recipe_dispatch::XY,
        Revision::new(1),
    );
    let payload = serde_json::json!({"x":[0.,1.,2.],"y":[2.,4.,3.]});
    let data = registry
        .materialize(&op, &payload, DataLimits::default(), true)
        .unwrap();
    let selection = AuthoringSelection {
        operation: op.clone(),
        parameters: serde_json::json!({"line":true}),
    };
    let plot = autoplot(data.clone(), selection.clone(), registry.clone(), true)
        .unwrap()
        .title(chart_core::plot::title("Typed recipe"))
        .build()
        .unwrap();
    assert_eq!(plot.chart().unwrap().prepare().unwrap().layers().len(), 2);
    let copy = autoplot(data.clone(), selection, registry.clone(), true)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(
        plot.chart().unwrap().prepare().unwrap().layers()[0].marks(),
        copy.chart().unwrap().prepare().unwrap().layers()[0].marks()
    );
    let bad = AuthoringSelection {
        operation: op.clone(),
        parameters: serde_json::json!({"line":true,"ignored":1}),
    };
    assert!(autoplot(data, bad, registry.clone(), true).is_err());
    assert!(
        registry
            .materialize(
                &op,
                &payload,
                DataLimits {
                    max_batch_rows: 2,
                    ..Default::default()
                },
                true
            )
            .is_err()
    );
    assert!(
        registry
            .materialize(
                &op,
                &serde_json::json!({"x":[0.],"y":[]}),
                DataLimits::default(),
                true
            )
            .is_err()
    );
    let native = OperationRef::new(
        chart_extension_example::recipe_dispatch::NATIVE_XY,
        Revision::new(1),
    );
    assert!(
        registry
            .materialize(&native, &payload, DataLimits::default(), true)
            .is_err()
    );
    assert!(
        registry
            .materialize(&native, &payload, DataLimits::default(), false)
            .is_ok()
    );
}
