use chart_export::portable::PortableChart;
use serde_json::{Value, json};
use std::{fs, path::Path};
fn subset(actual: &Value, expected: &Value) {
    match expected {
        Value::Object(values) => {
            for (k, v) in values {
                subset(&actual[k], v)
            }
        }
        Value::Array(values) => {
            assert_eq!(actual.as_array().unwrap().len(), values.len());
            for (a, b) in actual.as_array().unwrap().iter().zip(values) {
                subset(a, b)
            }
        }
        Value::Number(b) => {
            let a = actual.as_f64().unwrap();
            let b = b.as_f64().unwrap();
            assert!((a - b).abs() <= 1e-12 * b.abs().max(1.), "{a} != {b}");
        }
        _ => assert_eq!(actual, expected),
    }
}
fn dispatch(
    chart: &mut PortableChart,
    stamp: &Value,
    action: &Value,
    origin: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let state: Value = serde_json::from_str(&chart.state()?)?;
    Ok(serde_json::from_str(&chart.dispatch(&json!({"definition_revision":state["definition_revision"],"expected_state":state["state_revision"],"scene":stamp,"origin":origin,"action":action}).to_string())?)?)
}
pub fn run(root: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut cases: Vec<Value> = vec![];
    for fixture in [
        "fixtures/interaction/cases.json",
        "fixtures/host-tools/cases.json",
    ] {
        cases.extend(serde_json::from_str::<Vec<Value>>(&fs::read_to_string(
            root.join(fixture),
        )?)?);
    }
    let mut trace = vec![];
    for case in cases {
        let mut chart = PortableChart::new(
            &case["chart"].to_string(),
            &case["data"].to_string(),
            &fs::read_to_string(root.join("fixtures/interaction/profile.json"))?,
            fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
        )?;
        let mut stamp = serde_json::from_str::<Value>(&chart.present()?)?["stamp"].clone();
        let mut basis = None;
        let initial: Value = serde_json::from_str(&chart.semantics()?)?;
        for step in case["queries"].as_array().unwrap() {
            println!("input {} / {}", case["name"], step["name"]);
            let before: Value = serde_json::from_str(&chart.state()?)?;
            let mut result = json!(null);
            if let Some(query) = step.get("query") {
                let mut query_stamp = if step["gesture"] == true {
                    basis.as_ref().unwrap_or(&stamp).clone()
                } else {
                    stamp.clone()
                };
                if step["stale"] == true {
                    query_stamp["layout"] = json!("999999");
                }
                result = match chart.query(
                    &json!({"scene":query_stamp,"gesture":step["gesture"]==true,"query":query})
                        .to_string(),
                ) {
                    Ok(value) => {
                        assert!(step.get("error").is_none());
                        serde_json::from_str(&value)?
                    }
                    Err(error) => {
                        let error = serde_json::to_value(error)?;
                        assert_eq!(error["code"], step["error"]);
                        json!({"error":error["code"]})
                    }
                };
                assert_eq!(
                    serde_json::from_str::<Value>(&chart.state()?)?,
                    before,
                    "query must be read-only"
                );
                if let Some(expected) = step.get("expect") {
                    subset(&result, expected);
                }
            }
            let action=step.get("action").cloned().or_else(||match step["apply"].as_str(){
                Some("annotation_preview")=>Some(json!({"PreviewGesture":{"id":step["id"],"preview":{"Annotation":result["annotation"]}}})),
                Some("linked")=>Some(result["action"].clone()),
                Some("windows")=>Some(json!({"SetAxisWindows":result["windows"]})),
                Some("preview")=>Some(json!({"PreviewGesture":{"id":step["id"],"preview":{"AxisWindows":result["windows"]}}})),
                Some("targets")=>Some(json!({"Select":{"change":step.get("change").unwrap_or(&json!("Replace")),"targets":result["targets"]}})),
                _=>None,
            });
            if let Some(action) = action {
                if action.get("BeginGesture").is_some() {
                    basis = Some(stamp.clone());
                }
                let origin = if step["apply"] == "linked" {
                    result["origin"].clone()
                } else {
                    json!("Pointer")
                };
                dispatch(
                    &mut chart,
                    basis.as_ref().unwrap_or(&stamp),
                    &action,
                    &origin,
                )?;
                if action.get("CancelGesture").is_some() || action.get("CommitGesture").is_some() {
                    basis = None;
                }
            }
            let state: Value = serde_json::from_str(&chart.state()?)?;
            if let Some(expected) = step.get("state") {
                subset(&state, expected);
            }
            if step["present"] == true {
                stamp = serde_json::from_str::<Value>(&chart.present()?)?["stamp"].clone();
                let name = format!(
                    "{}-{}",
                    case["name"].as_str().unwrap(),
                    step["name"].as_str().unwrap()
                );
                fs::write(output.join(format!("{name}.svg")), chart.export("svg")?)?;
                fs::write(output.join(format!("{name}.png")), chart.export("png")?)?;
            }
            let semantic: Value = serde_json::from_str(&chart.semantics()?)?;
            assert_eq!(semantic["datasets"], initial["datasets"]);
            for (a, b) in semantic["layers"]
                .as_array()
                .unwrap()
                .iter()
                .zip(initial["layers"].as_array().unwrap())
            {
                assert_eq!(a["rows"], b["rows"]);
                assert_eq!(a["domains"], b["domains"]);
            }
            trace.push(
                json!({"case":case["name"],"name":step["name"],"result":result,"state":state}),
            );
        }
    }
    fs::write(
        output.join("input-trace.json"),
        serde_json::to_string_pretty(&trace)?,
    )?;
    println!(
        "PASS WP-16/17 shared input queries/actions: {} steps",
        trace.len()
    );
    Ok(())
}
