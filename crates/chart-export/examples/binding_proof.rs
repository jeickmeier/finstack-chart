//! Native execution of the same versioned fixture consumed by Python and real WASM.
use chart_export::portable::PortableChart;
use std::{fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    let fixture = root.join("fixtures/bindings");
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/wp-09/native".into()),
    );
    fs::create_dir_all(&output)?;
    let mut chart = PortableChart::new(
        &fs::read_to_string(fixture.join("chart.json"))?,
        &fs::read_to_string(fixture.join("data.json"))?,
        &fs::read_to_string(fixture.join("profile.json"))?,
        fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
    )?;
    fs::write(output.join("initial.json"), chart.semantics()?)?;
    fs::write(output.join("definition.json"), chart.definition()?)?;
    let transaction = fs::read_to_string(fixture.join("correction.json"))?;
    fs::write(
        output.join("transaction.json"),
        chart.transaction(&transaction)?,
    )?;
    fs::write(output.join("replay.json"), chart.transaction(&transaction)?)?;
    fs::write(
        output.join("action.json"),
        chart.action(&fs::read_to_string(fixture.join("action.json"))?)?,
    )?;
    fs::write(output.join("final.json"), chart.semantics()?)?;
    fs::write(output.join("scene.json"), chart.scene()?)?;
    fs::write(output.join("state.json"), chart.state()?)?;
    fs::write(output.join("chart.svg"), chart.export("svg")?)?;
    fs::write(output.join("chart.pdf"), chart.export("pdf")?)?;
    fs::write(output.join("chart.png"), chart.export("png")?)?;
    let mut cases: Vec<serde_json::Value> = serde_json::from_str(&fs::read_to_string(
        root.join("fixtures/statistics/portable-cases.json"),
    )?)?;
    cases.extend(serde_json::from_str::<Vec<serde_json::Value>>(
        &fs::read_to_string(root.join("fixtures/families/portable-cases.json"))?,
    )?);
    let mut statistics = serde_json::Map::new();
    let mut scenes = serde_json::Map::new();
    for case in cases {
        let name = case["name"].as_str().ok_or("case name")?;
        let mut chart = PortableChart::new(
            &case["chart"].to_string(),
            &case["data"].to_string(),
            &fs::read_to_string(fixture.join("profile.json"))?,
            fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
        )?;
        statistics.insert(name.into(), serde_json::from_str(&chart.semantics()?)?);
        scenes.insert(name.into(), serde_json::from_str(&chart.scene()?)?);
        fs::write(
            output.join(format!("statistics-{name}.svg")),
            chart.export("svg")?,
        )?;
        fs::write(
            output.join(format!("statistics-{name}.png")),
            chart.export("png")?,
        )?;
        if name.starts_with("family-") {
            fs::write(
                output.join(format!("statistics-{name}.pdf")),
                chart.export("pdf")?,
            )?;
        }
        chart.dispose();
    }
    fs::write(
        output.join("statistics.json"),
        serde_json::to_string_pretty(&statistics)?,
    )?;
    fs::write(
        output.join("statistics-scenes.json"),
        serde_json::to_string_pretty(&scenes)?,
    )?;
    chart.dispose();
    assert_eq!(
        chart.scene().unwrap_err().code,
        chart_core::DiagnosticCode::DisposedHandle
    );
    chart.dispose();
    println!(
        "PASS native shared portable fixture, transaction replay, action and controlled disposal"
    );
    Ok(())
}
