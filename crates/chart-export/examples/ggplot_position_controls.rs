//! FIX-GG06 independently authored Rust position publications.
#[path = "../../../examples/common/ggplot_position_controls.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for mode in 0..8 {
        let original = fixtures::author(mode)?;
        let plot = chart_core::prelude::Plot::from_json(&original.to_json()?)?;
        let request = output.request(&plot, export_options(PageSize::points(480., 320.)?))?;
        let frame = request.prepare()?;
        std::fs::write(
            out.join(format!("position-{mode}.plot.json")),
            plot.to_json()?,
        )?;
        std::fs::write(
            out.join(format!("position-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("position-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
        println!("{}", fixtures::name(mode));
    }
    println!("PASS Rust GG06 positions: 8 authors, roundtrips, 24 publications.");
    Ok(())
}
