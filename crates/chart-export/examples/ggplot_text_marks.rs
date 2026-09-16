//! FIX-GG08 independently authored Rust text/raster publications.
#[path = "../../../examples/common/ggplot_text_marks.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for mode in 0..7 {
        let original = fixtures::author(mode)?;
        let p = chart_core::prelude::Plot::from_json(&original.to_json()?)?;
        let request = output.request(&p, export_options(PageSize::points(600., 360.)?).dpi(144))?;
        let frame = request.prepare()?;
        std::fs::write(out.join(format!("text-{mode}.plot.json")), p.to_json()?)?;
        std::fs::write(
            out.join(format!("text-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("text-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
        if mode < 4 {
            let outlined = output
                .request(
                    &p,
                    export_options(PageSize::points(600., 360.)?)
                        .dpi(144)
                        .text(TextMode::Outline),
                )?
                .prepare()?;
            for (format, suffix) in [(Format::Svg, "svg"), (Format::Pdf, "pdf")] {
                std::fs::write(
                    out.join(format!("text-{mode}.outline.{suffix}")),
                    outlined.export(format)?.bytes,
                )?;
            }
        }
    }
    println!(
        "PASS Rust text marks: seven independent authors, 21 standard and 8 outline publications."
    );
    Ok(())
}
