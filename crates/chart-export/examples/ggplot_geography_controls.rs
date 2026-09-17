//! FIX-GG15 independently authored Rust geography publications.
#[path = "../../../examples/common/ggplot_geography_controls.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for mode in 0..19 {
        let original = fixtures::author(mode)?;
        let p = chart_core::prelude::Plot::from_json(&original.to_json()?)?;
        let request = output.request(
            &p,
            export_options(PageSize::points(600., 360.)?).dpi(if mode == 12 { 300 } else { 144 }),
        )?;
        let frame = request.prepare()?;
        let original_frame = output
            .request(
                &original,
                export_options(PageSize::points(600., 360.)?).dpi(if mode == 12 {
                    300
                } else {
                    144
                }),
            )?
            .prepare()?;
        assert_eq!(frame.scene_json()?, original_frame.scene_json()?);
        std::fs::write(
            out.join(format!("geography-{mode}.plot.json")),
            p.to_json()?,
        )?;
        std::fs::write(
            out.join(format!("geography-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("geography-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!("PASS Rust geography controls: 19 authors, 57 publications.");
    Ok(())
}
