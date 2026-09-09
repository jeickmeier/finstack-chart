//! FIX-S01 sector, hole and custom sink proof through the accepted path foundation.
#[path = "../../../examples/common/shape_foundation_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let plot = fixtures::figure()?;
    std::fs::write(out.join("figure.plot.json"), plot.to_json()?)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for dpi in [300, 600] {
        let request = output.request(
            &plot,
            export_options(PageSize::points(540., 280.)?).dpi(dpi),
        )?;
        let frame = request.prepare()?;
        let scene: serde_json::Value = serde_json::from_str(&frame.scene_json()?)?;
        assert_eq!(
            scene["items"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|i| i["primitive"].get("VectorPath").is_some())
                .count(),
            3
        );
        for (i, item) in scene["items"].as_array().unwrap().iter().enumerate() {
            if item["primitive"].get("VectorPath").is_some() {
                assert_eq!(scene["targets"][i], serde_json::json!([]));
            }
        }
        std::fs::write(
            out.join(format!("figure-{dpi}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("figure-{dpi}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!(
        "PASS FIX-S01: circular sector, winding hole and external-sink Bezier use shared paths at 300/600 DPI; generator implementations remain separate work."
    );
    Ok(())
}
