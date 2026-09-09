//! Actual sparse stack bars/areas and five offsets in all themes.
#[path = "../../../examples/common/shape_stack_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    for preset in [
        chart_core::theme::NamedTheme::Editorial,
        chart_core::theme::NamedTheme::Terminal,
        chart_core::theme::NamedTheme::Grayscale,
    ] {
        let out = out.join(format!("{preset:?}"));
        std::fs::create_dir_all(&out)?;
        let plot = fixtures::figure(preset)?;
        std::fs::write(out.join("figure.plot.json"), plot.to_json()?)?;
        let output = Output::new(
            include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
        )?;
        for dpi in [300, 600] {
            let request = output.request(
                &plot,
                export_options(PageSize::points(600., 740.)?).dpi(dpi),
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
                15
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
    }
    println!(
        "PASS stack publication: five offsets, InsideOut order, bars/areas, sparse samples, three themes and 300/600 DPI outputs."
    );
    Ok(())
}
