//! Execute public hierarchy request sequences without any oracle/layout calculations.
use chart_core::{ChartResult, Diagnostic, DiagnosticCode, hierarchy::*, portable};
use serde_json::{Value, json};
use std::collections::BTreeMap;
fn run(steps: &[Value]) -> ChartResult<Value> {
    let registry = chart_extension_example::registry()?;
    let mut sessions = BTreeMap::<String, HierarchySession>::new();
    let mut out = serde_json::Map::new();
    for step in steps {
        let name = step["session"].as_str().unwrap();
        let data = step["data"].to_string();
        let result = match step["action"].as_str().unwrap() {
            "construct" => {
                sessions.insert(name.into(), HierarchySession::from_json(&data, &registry)?);
                None
            }
            "apply" => {
                sessions.get_mut(name).unwrap().apply_json(&data)?;
                None
            }
            "query" => Some(sessions[name].query(portable::decode(&data)?)?),
            "tile" => Some(
                serde_json::from_str::<Value>(&sessions.get_mut(name).unwrap().tile_json(&data)?)
                    .unwrap(),
            ),
            "packing" => Some(portable::decode::<PackingRequest>(&data)?.execute()?),
            "snapshot" => Some(portable::decode::<Value>(&sessions[name].to_json()?)?),
            "restore" => {
                let snapshot = sessions[name].to_json()?;
                sessions.insert(
                    name.into(),
                    HierarchySession::from_snapshot_json(&snapshot, &registry)?,
                );
                None
            }
            "clone" => {
                sessions.insert(
                    step["target"].as_str().unwrap().into(),
                    sessions[name].clone(),
                );
                None
            }
            "copy_subtree" => {
                let copy = sessions[name].copy_subtree(
                    portable::decode(&step["data"]["node"].to_string())?,
                    portable::decode(&step["data"]["identity"].to_string())?,
                )?;
                sessions.insert(step["target"].as_str().unwrap().into(), copy);
                None
            }
            _ => {
                return Err(Diagnostic::error(
                    DiagnosticCode::Validation,
                    "Unknown proof operation.",
                    "Use the committed request generator.",
                ));
            }
        };
        if let Some(result) = result {
            out.insert(step["out"].as_str().unwrap().into(), result);
        }
    }
    Ok(Value::Object(out))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let input: Vec<Value> = serde_json::from_slice(&std::fs::read(&args[1])?)?;
    let output = input.iter().map(|case| {
        let result = run(case["steps"].as_array().unwrap());
        json!({"id":case["id"],"result":match result { Ok(value) => value, Err(error) => json!({"error":error.message,"code":error.code}) }})
    }).collect::<Vec<_>>();
    std::fs::write(&args[2], serde_json::to_vec(&output)?)?;
    println!("{} Rust hierarchy sequences executed", output.len());
    Ok(())
}
