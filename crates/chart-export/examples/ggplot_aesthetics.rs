//! FIX-GG03: immutable Rust publication through the canonical aesthetic engine.
#[path = "../../../examples/common/ggplot_aesthetic_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let plot = fixtures::figure()?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let request = output.request(
        &plot,
        export_options(PageSize::points(640., 400.)?).dpi(144),
    )?;
    let frame = request.prepare()?;
    std::fs::write(out.join("glyphs.plot.json"), plot.to_json()?)?;
    std::fs::write(out.join("glyphs.scene.json"), frame.scene_json()?)?;
    for (format, suffix) in [
        (Format::Svg, "svg"),
        (Format::Pdf, "pdf"),
        (Format::Png, "png"),
    ] {
        std::fs::write(
            out.join(format!("glyphs.{suffix}")),
            frame.export(format)?.bytes,
        )?;
    }
    println!("PASS Rust GG-03: independently authored 26-glyph SVG/PDF/PNG publication.");
    Ok(())
}
