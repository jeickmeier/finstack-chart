use chart_export::portable::PortableChart;
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn subset(actual: &Value, expected: &Value) {
    match expected {
        Value::Object(values) => {
            for (key, value) in values {
                subset(&actual[key], value);
            }
        }
        Value::Array(values) => {
            assert_eq!(actual.as_array().expect("array").len(), values.len());
            for (index, value) in values.iter().enumerate() {
                subset(&actual[index], value);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

pub fn run(root: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut chart = PortableChart::new(
        &fs::read_to_string(root.join("fixtures/actions/chart.json"))?,
        &fs::read_to_string(root.join("fixtures/bindings/data.json"))?,
        &fs::read_to_string(root.join("fixtures/bindings/profile.json"))?,
        fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
    )?;
    let scene: Value = serde_json::from_str(&chart.present()?)?;
    let mut stamp = scene["stamp"].clone();
    let steps: Vec<Value> = serde_json::from_str(&fs::read_to_string(
        root.join("fixtures/actions/trace.json"),
    )?)?;
    let mut trace = vec![];
    for step in steps {
        let state: Value = serde_json::from_str(&chart.state()?)?;
        let request = json!({"definition_revision":state["definition_revision"],"expected_state":step.get("expected_state").unwrap_or(&state["state_revision"]),"origin":step.get("origin").unwrap_or(&json!("Control")),"scene":stamp,"action":step["action"]});
        let result = match chart.dispatch(&request.to_string()) {
            Ok(value) => {
                assert!(step.get("error").is_none(), "{}", step["name"]);
                serde_json::from_str::<Value>(&value)?
            }
            Err(error) => {
                let value = serde_json::to_value(error)?;
                assert_eq!(value["code"], step["error"], "{}", step["name"]);
                json!({"error":value["code"]})
            }
        };
        let after: Value = serde_json::from_str(&chart.state()?)?;
        if let Some(expected) = step.get("expect") {
            subset(&result, expected);
        }
        if let Some(expected) = step.get("state") {
            subset(&after, expected);
        }
        if step.get("error").is_some() {
            assert_eq!(after, state);
        }
        if let Some(name) = step["export"].as_str() {
            fs::write(
                output.join(format!("actions-{name}.svg")),
                chart.export("svg")?,
            )?;
            fs::write(
                output.join(format!("actions-{name}.png")),
                chart.export("png")?,
            )?;
        }
        if step["present"] == true {
            stamp = serde_json::from_str::<Value>(&chart.present()?)?["stamp"].clone();
        }
        trace.push(json!({"name":step["name"],"result":result,"state":after}));
    }
    fs::write(
        output.join("actions-state-trace.json"),
        serde_json::to_string_pretty(&trace)?,
    )?;
    chart.dispose();
    println!(
        "PASS WP-15 deterministic shared action trace: {} transitions",
        trace.len()
    );
    Ok(())
}
