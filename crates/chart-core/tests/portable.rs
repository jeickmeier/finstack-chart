//! Minimal FIX-15/16 portable semantics with independent expected values and negative contracts.
use chart_core::{data::*, grammar::*, portable::*, *};
use serde_json::{Value, json};
const CHART: &str = include_str!("../../../fixtures/bindings/chart.json");
const DATA: &str = include_str!("../../../fixtures/bindings/data.json");
const TX: &str = include_str!("../../../fixtures/bindings/correction.json");
const ACTION: &str = include_str!("../../../fixtures/bindings/action.json");
fn session() -> Session {
    Session::new(CHART, DATA).unwrap()
}
fn counts(s: &mut Session) -> Vec<u64> {
    let p = s.prepare().unwrap();
    let PreparedRows::Binned(rows) = p.layers()[0].table().rows() else {
        panic!("bin rows")
    };
    rows.iter().map(|r| r.count).collect()
}
#[test]
fn fix15_round_trip_correction_viewport_and_aggregate_members() {
    let mut s = session();
    assert_eq!(counts(&mut s), vec![2, 1]);
    let round = Session::new(&s.chart_json().unwrap(), DATA).unwrap();
    assert_eq!(round.definition(), s.definition());
    assert!(matches!(
        s.apply_transaction(TX).unwrap(),
        chart_core::transaction::CommitOutcome::Applied(_)
    ));
    assert_eq!(counts(&mut s), vec![1, 2]);
    assert!(matches!(
        s.apply_transaction(TX).unwrap(),
        chart_core::transaction::CommitOutcome::AlreadyApplied(_)
    ));
    s.apply_action(ACTION).unwrap();
    assert_eq!(counts(&mut s), vec![1, 2]);
    let prepared = s.prepare().unwrap();
    let PreparedRows::Binned(rows) = prepared.layers()[0].table().rows() else {
        panic!("bins")
    };
    let chart_core::provenance::Target::Aggregate { members, .. } = &rows[1].target else {
        panic!("aggregate")
    };
    assert_eq!(
        members.as_ref(),
        [RowKey::new(9007199254743002), RowKey::new(9007199254743004)]
    );
    let state = s.state_json().unwrap();
    let mut restored = session();
    restored.restore_state(&state, Revision::INITIAL).unwrap();
    assert_eq!(restored.state_json().unwrap(), state);
    assert_eq!(
        s.apply_action(ACTION).unwrap_err().code,
        DiagnosticCode::RevisionConflict
    );
    assert_eq!(s.state_json().unwrap(), state);
}
#[test]
fn fix16_all_64_bits_null_payload_and_formatted_columns_round_trip() {
    let envelope: DataEnvelope = decode(DATA).unwrap();
    let batch = envelope.datasets[0].batch.clone().into_batch().unwrap();
    assert_eq!(BatchWire::from_batch(&batch).into_batch().unwrap(), batch);
    assert_eq!(batch.keys()[0].get(), 9007199254743001);
    assert_eq!(
        batch.columns()[2].values(),
        &ColumnValues::Timestamp(vec![
            1712345678901234567,
            1712345678901234568,
            1712345678901234569,
            1712345678901234570
        ])
    );
    assert!(!batch.columns()[0].validity()[2]);
    assert_eq!(batch.columns()[0].formatted(0), Some("0.2500"));
    let values = ColumnValues::Float64(vec![
        -0.,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::from_bits(0x7ff8000000000055),
    ]);
    let restored: ColumnValues = decode(&encode(&values).unwrap()).unwrap();
    assert_eq!(restored, values);
    for input in [
        "9007199254740993",
        "\"01\"",
        "\"-1\"",
        "\"18446744073709551616\"",
    ] {
        assert_eq!(
            decode::<RowKey>(input).unwrap_err().code,
            DiagnosticCode::Validation
        );
    }
}
#[test]
fn strict_versions_required_fields_unknown_operations_and_duplicate_keys_reject() {
    let mut chart: Value = serde_json::from_str(CHART).unwrap();
    chart["version"] = json!(2);
    assert_eq!(
        Session::new(&chart.to_string(), DATA).err().unwrap().code,
        DiagnosticCode::UnsupportedCapability
    );
    chart["version"] = json!(1);
    chart["definition"]["layers"][0]["statistic"]["operation"]["id"] = json!("native.closure");
    assert_eq!(
        Session::new(&chart.to_string(), DATA).err().unwrap().code,
        DiagnosticCode::UnsupportedCapability
    );
    let missing = CHART.replacen("\"version\": 1,", "", 1);
    assert_eq!(
        Session::new(&missing, DATA).err().unwrap().code,
        DiagnosticCode::Validation
    );
    let duplicate = CHART.replacen("\"version\": 1,", "\"version\": 1, \"version\": 1,", 1);
    assert_eq!(
        Session::new(&duplicate, DATA).err().unwrap().code,
        DiagnosticCode::Validation
    );
    let unknown = CHART.replacen("\"version\": 1,", "\"version\": 1, \"widget\": {},", 1);
    assert_eq!(
        Session::new(&unknown, DATA).err().unwrap().code,
        DiagnosticCode::Validation
    );
}
#[test]
fn bounded_decoding_and_failed_transactions_leave_data_and_state_unchanged() {
    assert_eq!(
        decode::<Value>(&" ".repeat(MAX_INPUT_BYTES + 1))
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    assert_eq!(
        decode::<Value>(&format!("{}0{}", "[".repeat(33), "]".repeat(33)))
            .unwrap_err()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let mut s = session();
    let before = s.semantics_json().unwrap();
    let mut tx: Value = serde_json::from_str(TX).unwrap();
    tx["operations"][0]["mutation"]["UpsertByKey"]["columns"][0]["validity"] = json!([]);
    assert!(s.apply_transaction(&tx.to_string()).is_err());
    assert_eq!(s.semantics_json().unwrap(), before);
    tx = serde_json::from_str(TX).unwrap();
    tx["expected"][0]["revision"] = json!("99");
    assert!(matches!(
        s.apply_transaction(&tx.to_string()).unwrap(),
        chart_core::transaction::CommitOutcome::Conflict(_)
    ));
    assert_eq!(s.semantics_json().unwrap(), before);
}
#[test]
fn native_accessors_fail_serialization_instead_of_disappearing() {
    let rows = TypedRows::snapshot(
        DatasetId::new(1),
        Revision::INITIAL,
        vec![RowKey::new(1)],
        vec![42.],
        10,
    )
    .unwrap();
    let builder = TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(1)).float(
        FieldId::new(1),
        "value",
        |v| Some(*v),
    );
    assert_eq!(
        builder.to_portable_spec().unwrap_err().code,
        DiagnosticCode::UnsupportedCapability
    );
    assert!(builder.finish(DataLimits::default()).is_ok());
}

#[test]
fn state_restore_cannot_reuse_revision_for_changed_content() {
    let mut s = session();
    s.apply_action(ACTION).unwrap();
    let before = s.state_json().unwrap();
    let mut snapshot: Value = serde_json::from_str(&before).unwrap();
    snapshot["viewport"]["x"] = json!([0.0, 2.0]);
    assert_eq!(
        s.restore_state(&snapshot.to_string(), Revision::new(1))
            .unwrap_err()
            .code,
        DiagnosticCode::RevisionConflict
    );
    snapshot["state_revision"] = json!("2");
    assert_eq!(
        s.restore_state(&snapshot.to_string(), Revision::new(1))
            .unwrap_err()
            .code,
        DiagnosticCode::RevisionConflict
    );
    assert_eq!(s.state_json().unwrap(), before);
    snapshot["viewport_revision"] = json!("2");
    s.restore_state(&snapshot.to_string(), Revision::new(1))
        .unwrap();
    assert_eq!(s.state().viewport().x, Some((0., 2.)));
}
