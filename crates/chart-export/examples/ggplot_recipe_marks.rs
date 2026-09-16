//! FIX-GG07 independently authored Rust recipe mark publications.
#[path = "../../../examples/common/ggplot_recipe_marks.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for mode in 0..5 {
        let original = fixtures::author(mode)?;
        let p = chart_core::prelude::Plot::from_json(&original.to_json()?)?;
        let frame = output
            .request(&p, export_options(PageSize::points(600., 360.)?))?
            .prepare()?;
        std::fs::write(out.join(format!("mark-{mode}.plot.json")), p.to_json()?)?;
        std::fs::write(
            out.join(format!("mark-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("mark-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!("PASS Rust five independent recipe mark authors, 15 publications.");
    Ok(())
}
