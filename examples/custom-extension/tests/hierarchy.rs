//! FIX-H01-G: independent portable callback, ownership, atomicity and snapshot contracts.
use chart_core::{ChartResult, DiagnosticCode, hierarchy::*};
use serde_json::{Value, json};
fn registered(mode: &str) -> Value {
    json!({"operation":{"id":"example.hierarchy","version":"1"},"parameters":{"mode":mode}})
}
fn input() -> String {
    json!({"version":1,"identity":"9007199254740993","input":{"Children":{"root":{"key":"9007199254740994","data":{"key":"9007199254740994","name":"R","value":5,"children":[{"key":"9007199254740995","name":"A","value":2,"children":[{"key":"9007199254740996","name":"C","value":4},{"key":"9007199254740997","name":"D","value":6}]},{"key":"9007199254740998","name":"B","value":3}]}},"accessor":registered("Children")}}}).to_string()
}
fn root() -> NodeHandle {
    NodeHandle {
        hierarchy: HierarchyId::new(9007199254740993),
        node: HierarchyNodeId::new(9007199254740994),
    }
}
#[test]
fn registered_operations_snapshots_history_and_atomic_failures() -> ChartResult<()> {
    let registry = chart_extension_example::registry()?;
    let mut session = HierarchySession::from_json(&input(), &registry)?;
    session.apply_json(&json!({"Sum":{"Registered":registered("Value")}}).to_string())?;
    let initial = session.query(HierarchyQuery::Nodes)?;
    assert_eq!(initial[0]["value"], 20.);
    let names = |v: &Value| {
        v.as_array()
            .unwrap()
            .iter()
            .map(|n| n["data"]["name"].as_str().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&initial), ["R", "A", "B", "C", "D"]);
    let result = session.query_json(
        &json!({"FindRegistered":{"root":root(),"predicate":registered("FindC")}}).to_string(),
    )?;
    assert_eq!(
        serde_json::from_str::<NodeHandle>(&result)
            .unwrap()
            .node
            .get(),
        9007199254740996
    );
    session
        .apply_json(&json!({"Sort":{"Registered":registered("DescendingValue")}}).to_string())?;
    assert_eq!(
        names(&session.query(HierarchyQuery::Nodes)?),
        ["R", "A", "B", "D", "C"]
    );
    session.apply_json(&json!({"Layout":{"Tree":{"options":{"mode":{"Extent":[8,4]}},"separation":registered("DepthSeparation")}}}).to_string())?;
    assert!(session.layout().is_some());
    let retained = session.clone();
    let retained_json = retained.to_json()?;
    let layout = json!({"Layout":{"Treemap":{"options":{"size":[80,40],"tile":{"Resquarify":1.618033988749895}},"history":true,"padding":{"Registered":registered("DepthPadding")},"tiler":null}}});
    session.apply_json(&layout.to_string())?;
    assert_eq!(
        session.query(HierarchyQuery::Configuration)?["history_members"],
        4
    );
    let snapshot = session.to_json()?;
    let mut restored = HierarchySession::from_snapshot_json(&snapshot, &registry)?;
    assert_eq!(
        restored.query(HierarchyQuery::Nodes)?,
        session.query(HierarchyQuery::Nodes)?
    );
    assert_eq!(restored.to_json()?, snapshot);
    for owner in [&mut session, &mut restored] {
        owner.apply(HierarchyChange::Count)?;
        owner.apply_json(&layout.to_string())?;
    }
    assert_eq!(restored.to_json()?, session.to_json()?);
    assert_eq!(retained.to_json()?, retained_json);
    let before = session.to_json()?;
    for bad in [
        json!({"Layout":{"Treemap":{"options":{"tile":"Custom"},"history":true,"padding":null,"tiler":registered("InvalidTile")}}}),
        json!({"Sum":{"Constant":-1}}),
        json!({"Replace":{"version":2,"identity":"1","input":{"Rows":[]}}}),
        json!({"Sum":{"Registered":{"operation":{"id":"missing","version":"1"},"parameters":null}}}),
        json!({"Sum":{"Registered":{"operation":{"id":"example.native_hierarchy","version":"1"},"parameters":{"mode":"Value"}}}}),
    ] {
        assert!(session.apply_json(&bad.to_string()).is_err(), "{bad}");
        assert_eq!(session.to_json()?, before);
    }
    session.apply(HierarchyChange::ResetHistory)?;
    assert_eq!(
        session.query(HierarchyQuery::Configuration)?["history_members"],
        0
    );
    assert_eq!(
        HierarchySession::from_snapshot_json(&session.to_json()?, &registry)?
            .query(HierarchyQuery::Nodes)?,
        session.query(HierarchyQuery::Nodes)?
    );
    let copy = session.copy_subtree(root(), HierarchyId::new(3))?;
    assert_eq!(
        copy.query(HierarchyQuery::Configuration)?["history_members"],
        0
    );
    assert!(copy.layout().is_none());
    assert!(copy.query(HierarchyQuery::Node(root())).is_err());
    Ok(())
}
#[test]
fn custom_pack_tiler_and_untrusted_snapshot_validation() -> ChartResult<()> {
    let registry = chart_extension_example::registry()?;
    let mut session = HierarchySession::from_json(&input(), &registry)?;
    session.apply(HierarchyChange::Count)?;
    let rectangles = session.tile_json(&json!({"version":1,"parent":root(),"bounds":[0,0,8,4],"tiler":"Custom","history":false,"operation":registered("EqualTile")}).to_string())?;
    let rectangles: Value = serde_json::from_str(&rectangles).unwrap();
    assert_eq!(rectangles[0][1], json!([0., 0., 4., 4.]));
    assert_eq!(rectangles[1][1], json!([4., 0., 8., 4.]));
    session.apply_json(&json!({"Layout":{"Pack":{"options":{"size":[80,40],"radius":"Explicit"},"radius":{"Registered":registered("Value")},"padding":{"Registered":registered("DepthPadding")}}}}).to_string())?;
    let nodes = session.query(HierarchyQuery::Nodes)?;
    let c = nodes
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["data"]["name"] == "C")
        .unwrap();
    assert_eq!(c["geometry"]["Circle"]["r"], 4.);
    let snapshot = session.to_json()?;
    let mut corrupt: Value = serde_json::from_str(&snapshot).unwrap();
    corrupt["nodes"][0]["children"] = json!([]);
    assert!(HierarchySession::from_snapshot_json(&corrupt.to_string(), &registry).is_err());
    let mut corrupt: Value = serde_json::from_str(&snapshot).unwrap();
    corrupt["nodes"][0]["geometry"]["Circle"]["r"] = json!(-1);
    assert!(HierarchySession::from_snapshot_json(&corrupt.to_string(), &registry).is_err());
    let too_large = format!(
        "{}{}",
        " ".repeat(chart_core::portable::MAX_INPUT_BYTES),
        input()
    );
    assert_eq!(
        HierarchySession::from_json(&too_large, &registry)
            .err()
            .unwrap()
            .code,
        DiagnosticCode::ResourceLimit
    );
    let enclosure = PackingRequest::execute_json(
        r#"{"version":1,"siblings":false,"circles":[{"x":0,"y":0,"r":1},{"x":4,"y":0,"r":1}]}"#,
    )?;
    assert_eq!(
        serde_json::from_str::<Value>(&enclosure).unwrap(),
        json!({"x":2.,"y":0.,"r":3.})
    );
    Ok(())
}

#[test]
fn independent_per_side_padding_accessors_and_clamped_ratio_readback() -> ChartResult<()> {
    let registry = chart_extension_example::registry()?;
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/hierarchy/padding-accessors.json"
    ))
    .unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 72);
    for case in fixture["cases"].as_array().unwrap() {
        let mut session = HierarchySession::from_json(&case["input"].to_string(), &registry)?;
        session.apply_json(r#"{"Sum":{"Field":"value"}}"#)?;
        session.apply_json(&json!({"Layout":case["layout"]}).to_string())?;
        let actual = session.query(HierarchyQuery::Nodes)?;
        for (a, b) in actual
            .as_array()
            .unwrap()
            .iter()
            .zip(case["expected"].as_array().unwrap())
        {
            assert_eq!(a["handle"]["node"], b["key"]);
            assert_eq!(a["value"].as_f64(), b["value"].as_f64());
            for key in ["x0", "y0", "x1", "y1"] {
                let a = a["geometry"]["Rectangle"][key].as_f64().unwrap();
                let b = b["geometry"]["Rectangle"][key].as_f64().unwrap();
                assert!(
                    (a - b).abs() <= 1e-10 + 1e-12 * b.abs(),
                    "{} {key}",
                    case["id"]
                );
            }
        }
        let config = session.query(HierarchyQuery::Configuration)?;
        assert_eq!(
            config["layout"]["Treemap"]["padding_sides"],
            case["layout"]["Treemap"]["padding_sides"]
        );
        let saved = session.to_json()?;
        assert_eq!(
            HierarchySession::from_snapshot_json(&saved, &registry)?.to_json()?,
            saved
        );
    }
    let mut session =
        HierarchySession::from_json(&fixture["cases"][0]["input"].to_string(), &registry)?;
    session.apply_json(r#"{"Sum":{"Field":"value"}}"#)?;
    session.apply_json(
        r#"{"Layout":{"Treemap":{"options":{"tile":{"Squarify":0}},"history":false}}}"#,
    )?;
    assert_eq!(
        session.query(HierarchyQuery::Configuration)?["effective_ratio"],
        1.
    );
    Ok(())
}
