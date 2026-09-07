//! Actual destination fonts, saved state and frozen publications through the public adapter.
use chart_core::{DiagnosticCode, SceneStamp, state::*};
use chart_export::portable::PortableChart;
use serde_json::{Value, json};
const CHART: &str = include_str!("../../../fixtures/actions/chart.json");
const DATA: &str = include_str!("../../../fixtures/bindings/data.json");
const PROFILE: &str = include_str!("../../../fixtures/bindings/profile.json");
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
fn chart() -> PortableChart {
    PortableChart::new(CHART, DATA, PROFILE, FONT.to_vec()).unwrap()
}
fn state(c: &PortableChart) -> Value {
    serde_json::from_str(&c.state().unwrap()).unwrap()
}
fn send(c: &mut PortableChart, action: ChartAction, stamp: SceneStamp) -> Value {
    let s = state(c);
    serde_json::from_str(&c.dispatch(&json!({"definition_revision":s["definition_revision"],"expected_state":s["state_revision"],"origin":"Control","scene":stamp,"action":action}).to_string()).unwrap()).unwrap()
}
#[path = "../examples/common/actions_trace.rs"]
mod trace;
#[test]
fn shared_trace_executes_real_font_annotation_preview_cancel_commit_and_undo() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = std::env::temp_dir().join(format!("chart-actions-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    trace::run(&root, &dir).unwrap();
    let svg = |name| std::fs::read(dir.join(format!("actions-{name}.svg"))).unwrap();
    // Scene revision metadata differs; compare actual XML primitive content independently below.
    let geometry = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .unwrap()
            .lines()
            .filter(|line| !line.contains("<metadata>"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(geometry(svg("cancel")), geometry(svg("undo")));
    assert_eq!(geometry(svg("preview")), geometry(svg("commit")));
    assert_eq!(geometry(svg("commit")), geometry(svg("redo")));
    assert_ne!(geometry(svg("cancel")), geometry(svg("commit")));
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn frozen_exports_keep_the_presented_data_while_ingestion_advances_and_resume_releases_it() {
    let mut c = chart();
    let first = c.capture().unwrap();
    let stamp = first.layout().scene().stamp();
    let weak = std::sync::Arc::downgrade(first.layout());
    drop(first);
    // capture returns an independent immutable publication; acknowledge the owned presentation.
    let shown: Value = serde_json::from_str(&c.present().unwrap()).unwrap();
    let original = c.export("svg").unwrap();
    send(
        &mut c,
        ChartAction::SetFollow(FollowMode::FreezePresentation),
        stamp,
    );
    let frozen = c.capture().unwrap();
    let frozen_weak = std::sync::Arc::downgrade(frozen.layout());
    drop(frozen);
    c.transaction(include_str!("../../../fixtures/bindings/correction.json"))
        .unwrap();
    assert_eq!(c.export("svg").unwrap(), original);
    assert_eq!(
        serde_json::from_str::<Value>(&c.present().unwrap()).unwrap(),
        shown
    );
    assert_eq!(
        serde_json::from_str::<Value>(&c.semantics().unwrap()).unwrap()["store_revision"],
        "1"
    );
    send(&mut c, ChartAction::ResumeLatest, stamp);
    assert_ne!(c.export("svg").unwrap(), original);
    c.present().unwrap();
    assert!(frozen_weak.upgrade().is_none());
    assert!(weak.upgrade().is_none());
}
#[test]
fn portable_controlled_replacements_check_component_fences_and_keep_hover_ephemeral() {
    let mut c = chart();
    let stamp = c.capture().unwrap().scene().stamp();
    let target = serde_json::from_value(json!({"epoch":"9007199254746001","layer":"9007199254742003","panel":null,"identity":{"Source":{"dataset":"9007199254741001","key":"9007199254743001"}}})).unwrap();
    send(&mut c, ChartAction::SetHover(vec![target]), stamp);
    let before = c.state().unwrap();
    c.restore_state(&before, "1").unwrap();
    assert_eq!(c.state().unwrap(), before);
    let mut replacement = state(&c);
    replacement["state_revision"] = json!("2");
    replacement["interaction"]["legend_visible"] = json!(false);
    assert_eq!(
        c.restore_state(&replacement.to_string(), "1")
            .unwrap_err()
            .code,
        DiagnosticCode::RevisionConflict
    );
    replacement["interaction"]["revisions"]["visibility"] = json!("1");
    replacement["interaction"]["revisions"]["durable"] = json!("1");
    c.restore_state(&replacement.to_string(), "1").unwrap();
    assert_eq!(state(&c)["interaction"]["revisions"]["hover"], "1");
    send(
        &mut c,
        ChartAction::BeginGesture {
            id: chart_core::Revision::new(1),
            kind: GestureKind::Viewport,
        },
        stamp,
    );
    send(
        &mut c,
        ChartAction::PreviewGesture {
            id: chart_core::Revision::new(1),
            preview: GesturePreview::Viewport(Viewport {
                x: Some((0.5, 1.5)),
                y: None,
            }),
        },
        stamp,
    );
    let saved = state(&c);
    assert_eq!(saved["viewport"]["x"], Value::Null);
    assert!(saved.get("active").is_none());
    let revision = saved["state_revision"].as_str().unwrap();
    assert!(c.restore_state(&saved.to_string(), revision).is_err());
    send(
        &mut c,
        ChartAction::CancelGesture(CancelReason::Explicit),
        stamp,
    );
    let saved = state(&c);
    c.restore_state(
        &saved.to_string(),
        saved["state_revision"].as_str().unwrap(),
    )
    .unwrap();
}
