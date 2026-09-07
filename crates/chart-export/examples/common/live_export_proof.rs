use chart_core::ChartResult;
use chart_export::portable::PortableChart;
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};
pub fn run(root: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let fixture: Value = serde_json::from_str(&fs::read_to_string(
        root.join("fixtures/live-export/replay.json"),
    )?)?;
    let mut chart = PortableChart::new(
        &fixture["chart"].to_string(),
        &fixture["data"].to_string(),
        &fs::read_to_string(root.join("fixtures/live-export/profile.json"))?,
        fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
    )?;
    let mut stamp = serde_json::from_str::<Value>(&chart.present()?)?["stamp"].clone();
    let mut jobs = BTreeMap::<String, String>::new();
    let mut outputs = vec![];
    let mut trace = vec![];
    let mut disposed = false;
    for step in fixture["steps"].as_array().unwrap() {
        let name = step["name"].as_str().unwrap();
        let result: ChartResult<Value> = (|| {
            let input=match step["kind"].as_str().unwrap(){
   "action"=>{let state:Value=serde_json::from_str(&chart.state()?).unwrap();chart.dispatch(&json!({"definition_revision":state["definition_revision"],"expected_state":state["state_revision"],"origin":"Control","scene":stamp,"action":step["action"]}).to_string())?},
   "transaction"=>chart.transaction(&step["transaction"].to_string())?,
   "present"=>{let scene:Value=serde_json::from_str(&chart.present()?).unwrap();stamp=scene["stamp"].clone();return Ok(json!({"stamp":stamp}));},
   "begin"=>{let result=chart.export_control(&json!({"version":1,"operation":{"Begin":step["options"]}}).to_string())?;let parsed:Value=serde_json::from_str(&result).unwrap();jobs.insert(name.to_owned(),parsed["job"].as_str().unwrap().to_owned());result},
   "cancel"=>chart.export_control(&json!({"version":1,"operation":{"Cancel":{"job":jobs[step["job"].as_str().unwrap()]}}}).to_string())?,
   "export"=>{let bytes=chart.export_job(&jobs[step["job"].as_str().unwrap()])?;let len=bytes.len();outputs.push((name.to_owned(),bytes));return Ok(json!({"bytes":len}));},
   "status"=>chart.export_control("{\"version\":1,\"operation\":\"Status\"}")?,
   "dispose"=>{chart.dispose();disposed=true;return Ok(json!({"disposed":true}));},
   _=>panic!("unknown live-export step"),
  };
            Ok(serde_json::from_str(&input).unwrap())
        })();
        let result = match result {
            Ok(result) => {
                assert!(step.get("error").is_none(), "{name}");
                result
            }
            Err(e) => {
                assert_eq!(
                    e.code.as_str(),
                    step["error"].as_str().unwrap_or("unexpected error"),
                    "{name}: {}",
                    e.message
                );
                json!({"error":e.code.as_str()})
            }
        };
        let status = if disposed {
            Value::Null
        } else {
            serde_json::from_str(
                &chart.export_control("{\"version\":1,\"operation\":\"Status\"}")?,
            )?
        };
        let live = if disposed {
            Value::Null
        } else {
            let semantics: Value = serde_json::from_str(&chart.semantics()?)?;
            json!({"store":semantics["store_revision"],"state":serde_json::from_str::<Value>(&chart.state()?)?})
        };
        trace.push(json!({"name":name,"result":result,"status":status,"live":live}));
    }
    for (name, bytes) in outputs {
        assert!(String::from_utf8_lossy(&bytes).contains("<svg"));
        fs::write(output.join(format!("live-{name}.svg")), bytes)?;
    }
    fs::write(
        output.join("live-export.json"),
        serde_json::to_vec_pretty(&trace)?,
    )?;
    println!(
        "PASS Rust FIX-14 held captures, atomic dual-dataset updates, bounded/canceled exports and owned bytes after disposal"
    );
    Ok(())
}
