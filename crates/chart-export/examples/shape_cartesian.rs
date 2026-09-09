//! Actual all-curve and general-area immutable publication.
#[path = "../../../examples/common/shape_cartesian_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let plot = fixtures::figure(None)?;
    std::fs::write(out.join("figure.plot.json"), plot.to_json()?)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for dpi in [300, 600] {
        let request = output.request(
            &plot,
            export_options(PageSize::points(680., 640.)?).dpi(dpi),
        )?;
        let frame = request.prepare()?;
        let scene: serde_json::Value = serde_json::from_str(&frame.scene_json()?)?;
        assert_eq!(
            scene["items"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|i| i["primitive"].get("ShapePath").is_some())
                .count(),
            39
        );
        for (i, item) in scene["items"].as_array().unwrap().iter().enumerate() {
            if let Some(shape) = item["primitive"].get("ShapePath") {
                assert_eq!(
                    shape["anchors"].as_array().unwrap().len(),
                    scene["targets"][i].as_array().unwrap().len()
                );
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
        "PASS Cartesian publication: 20 line curves, 19 general areas, source anchors and 300/600 DPI outputs."
    );
    Ok(())
}
