//! WP-13 physical composition artifacts from the same portable fixture used by all hosts.
use chart_export::portable::PortableChart;
use std::{fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = std::env::args()
        .nth(1)
        .map_or_else(|| root.join("artifacts/wp-13"), PathBuf::from);
    fs::create_dir_all(&output)?;
    let cases: Vec<serde_json::Value> = serde_json::from_str(&fs::read_to_string(
        root.join("fixtures/composition/portable-cases.json"),
    )?)?;
    for case in cases {
        let name = case["name"].as_str().ok_or("Missing name")?;
        let profile =
            fs::read_to_string(root.join(case["profile_file"].as_str().ok_or("Missing profile")?))?;
        let mut chart = PortableChart::new(
            &case["chart"].to_string(),
            &case["data"].to_string(),
            &profile,
            fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
        )?;
        let figure = chart.capture()?;
        fs::write(
            output.join(format!("{name}-semantics.json")),
            chart.semantics()?,
        )?;
        fs::write(output.join(format!("{name}-scene.json")), chart.scene()?)?;
        for format in ["svg", "pdf", "png"] {
            fs::write(
                output.join(format!("{name}.{format}")),
                chart.export(format)?,
            )?;
        }
        fs::write(
            output.join(format!("{name}-preview.svg")),
            figure.preview_svg()?,
        )?;
        let mut dpi: serde_json::Value = serde_json::from_str(&profile)?;
        dpi["dpi"] = 600.into();
        let large = PortableChart::new(
            &case["chart"].to_string(),
            &case["data"].to_string(),
            &dpi.to_string(),
            fs::read(root.join("fixtures/capability/fonts/NotoSans-Regular.ttf"))?,
        )?;
        fs::write(output.join(format!("{name}-600.png")), large.export("png")?)?;
        println!(
            "{name}: panels={}, insets={}, items={}, diagnostics={:?}",
            figure.layout().panels().len(),
            figure.layout().insets().len(),
            figure.scene().items().len(),
            figure.layout().diagnostics()
        );
    }
    Ok(())
}
