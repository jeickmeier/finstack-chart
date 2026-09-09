//! FIX-P01–06: standalone Rust results and primary path publication for actual host comparison.
#[path = "../../../examples/common/path_fixtures.rs"]
mod fixtures;
use chart_core::{
    Diagnostic,
    path::{Path, PathOp},
};
use chart_export::*;
use serde_json::{Value, json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&dir)?;
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/parity/d3-path/cases.json"))?;
    let mut records = vec![];
    for case in corpus["cases"].as_array().ok_or("case list")? {
        let mut path = Path::with_digits(case["digits"].as_f64())?;
        let operations: Vec<PathOp> = serde_json::from_value(case["operations"].clone())?;
        let mut observations = vec![];
        for operation in operations {
            let error = path.apply(&operation).err().map(|e: Diagnostic| e.code);
            observations.push(json!({"error":error,"result":serde_json::from_str::<Value>(&path.result_json()?)?}));
        }
        records.push(json!({"id":case["id"],"observations":observations}));
    }
    std::fs::write(dir.join("paths.json"), serde_json::to_vec_pretty(&records)?)?;
    let plot = fixtures::figure()?;
    std::fs::write(dir.join("figure.plot.json"), plot.to_json()?)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let cases = fixtures::cases();
    for dpi in [300, 600] {
        let request = output.request(
            &plot,
            export_options(PageSize::points(cases.width, cases.height)?).dpi(dpi),
        )?;
        let snapshot = request.prepare()?;
        assert_eq!(snapshot.scene().wire_version(), 2);
        let scene: Value = serde_json::from_str(&snapshot.scene_json()?)?;
        for (i, item) in snapshot.scene().items().iter().enumerate() {
            if matches!(
                item.primitive,
                chart_core::scene::Primitive::VectorPath { .. }
            ) {
                assert_eq!(scene["targets"][i], json!([]));
            }
        }
        std::fs::write(
            dir.join(format!("figure-{dpi}.scene.json")),
            serde_json::to_vec_pretty(&scene)?,
        )?;
        for (format, extension) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            let artifact = snapshot.export(format)?;
            std::fs::write(
                dir.join(format!("figure-{dpi}.{extension}")),
                artifact.bytes,
            )?;
        }
    }
    println!(
        "PASS FIX-P01–06 Rust: 86 operation sequences; retained scenes; SVG/PDF/PNG at 300/600 DPI."
    );
    Ok(())
}
