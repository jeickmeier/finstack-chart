//! FIX-AUTH05 independent runtime replay through primary builders.
#[path = "../../../examples/common/authoring_fixtures.rs"]
#[allow(dead_code)]
mod authors;
#[path = "support/authoring_replay.rs"]
mod replay;
#[test]
fn primary_runtime_replays_existing_action_and_input_expectations() {
    let output = chart_export::Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    let (actions, inputs) = replay::run(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        &output,
    );
    assert_eq!(actions.as_array().unwrap().len(), 23);
    assert_eq!(inputs.as_array().unwrap().len(), 47);
    assert_eq!(
        replay::run_stream(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
            &output
        )
        .as_array()
        .unwrap()
        .len(),
        70
    );
}
