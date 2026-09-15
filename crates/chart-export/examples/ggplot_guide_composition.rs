//! FIX-GG05 independently authored Rust guide publications.
#[path = "../../../examples/common/ggplot_guide_composition.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for mode in 0..10 {
        let original = fixtures::author(mode)?;
        let p = chart_core::prelude::Plot::from_json(&original.to_json()?)?;
        let request = output.request(&p, export_options(PageSize::points(600., 360.)?).dpi(144))?;
        let frame = request.prepare()?;
        std::fs::write(out.join(format!("guide-{mode}.plot.json")), p.to_json()?)?;
        std::fs::write(
            out.join(format!("guide-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("guide-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!("PASS Rust guide composition: ten independent authors, 30 publications.");
    Ok(())
}
