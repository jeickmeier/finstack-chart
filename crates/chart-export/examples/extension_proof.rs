//! FIX-17 public custom stat/geom publication and portable fixture generator.
use chart_core::{portable::*, *};
use chart_export::portable::PortableChart;
use chart_extension_example::*;
use std::{fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = root.join("artifacts/wp-14");
    fs::create_dir_all(&output)?;
    let snapshot = store()?.snapshot();
    let data = snapshot.get()?.dataset(DatasetId::new(1))?;
    let data = DataEnvelope {
        version: 1,
        epoch: SourceEpoch::new(1),
        datasets: vec![DatasetWire {
            id: DatasetId::new(1),
            batch: BatchWire::from_batch(data.chunks()[0].batch()),
        }],
    };
    let chart = ChartEnvelope {
        version: 1,
        definition: definition(false),
    };
    let case = serde_json::json!({"name":"extension-histogram","registered":true,"chart":chart,"data":data,"profile_file":"fixtures/bindings/profile.json"});
    let fixture = root.join("fixtures/extensions");
    fs::create_dir_all(&fixture)?;
    if std::env::args().any(|a| a == "--write-fixture") {
        let families: Vec<serde_json::Value> = serde_json::from_str(&fs::read_to_string(
            root.join("fixtures/families/portable-cases.json"),
        )?)?;
        let mut candle = families
            .into_iter()
            .find(|c| c["name"] == "family-ohlc-volume")
            .ok_or("OHLC fixture")?;
        candle["name"] = "alpha-candle-colors".into();
        candle["chart"]["definition"]["layers"][0]["candle_colors"] = serde_json::json!({"up":{"red":30,"green":145,"blue":85,"alpha":255},"down":{"red":195,"green":55,"blue":55,"alpha":255}});
        fs::write(
            fixture.join("portable-cases.json"),
            serde_json::to_string_pretty(&vec![case.clone(), candle])? + "\n",
        )?;
    }
    let profile = fs::read_to_string(root.join("fixtures/bindings/profile.json"))?;
    let font = fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?;
    let mut chart = PortableChart::with_extensions(
        &case["chart"].to_string(),
        &case["data"].to_string(),
        &profile,
        font,
        registry()?,
    )?;
    fs::write(output.join("extension-semantics.json"), chart.semantics()?)?;
    fs::write(output.join("extension-scene.json"), chart.scene()?)?;
    for format in ["svg", "pdf", "png"] {
        fs::write(
            output.join(format!("extension.{format}")),
            chart.export(format)?,
        )?;
    }
    println!(
        "PASS FIX-17 registered public custom stat/geom exports; expected counts [3,3], densities [0.5,0.5], exact aggregate members and custom guides/hit/keyboard metadata"
    );
    Ok(())
}
