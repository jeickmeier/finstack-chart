use chart_export::portable::PortableChart;
use serde_json::{Value, json};
use std::{fs, path::Path};
pub fn run(root: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let case: Value = serde_json::from_str(&fs::read_to_string(
        root.join("fixtures/streaming/replay.json"),
    )?)?;
    let mut chart = PortableChart::new(
        &case["chart"].to_string(),
        &case["data"].to_string(),
        &fs::read_to_string(root.join("fixtures/interaction/profile.json"))?,
        fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
    )?;
    let mut stamp = serde_json::from_str::<Value>(&chart.present()?)?["stamp"].clone();
    let mut trace = vec![];
    for step in case["steps"].as_array().unwrap() {
        println!("stream {}", step["name"]);
        let result = if let Some(transaction) = step.get("transaction") {
            chart.transaction(&transaction.to_string())
        } else if let Some(operation) = step.get("stream") {
            chart.stream(&json!({"version":1,"operation":operation}).to_string())
        } else {
            let state: Value = serde_json::from_str(&chart.state()?)?;
            chart.dispatch(&json!({"definition_revision":state["definition_revision"],"expected_state":state["state_revision"],"scene":stamp,"origin":"Control","action":step["action"]}).to_string())
        };
        let result = match result {
            Ok(value) => {
                assert!(
                    step.get("error").is_none(),
                    "Expected error at {}",
                    step["name"]
                );
                serde_json::from_str::<Value>(&value)?
            }
            Err(e) => {
                assert_eq!(e.code.as_str(), step["error"].as_str().unwrap());
                json!({"error":e.code.as_str()})
            }
        };
        let semantics: Value = serde_json::from_str(&chart.semantics()?)?;
        let state: Value = serde_json::from_str(&chart.state()?)?;
        if step["present"] == true {
            stamp = serde_json::from_str::<Value>(&chart.present()?)?["stamp"].clone();
            let name = step["name"].as_str().unwrap();
            fs::write(
                output.join(format!("stream-{name}.svg")),
                chart.export("svg")?,
            )?;
            fs::write(
                output.join(format!("stream-{name}.png")),
                chart.export("png")?,
            )?;
        }
        trace
            .push(json!({"name":step["name"],"result":result,"semantics":semantics,"state":state}));
    }
    fs::write(
        output.join("stream-trace.json"),
        serde_json::to_vec_pretty(&trace)?,
    )?;
    chart.dispose();
    Ok(())
}
