//! FIX-GG11 independently authored Rust text/raster publications.
#[path = "../../../examples/common/ggplot_spatial_controls.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for mode in 0..fixtures::CASES {
        let original = fixtures::author(mode)?;
        let p = chart_core::prelude::Plot::from_json(&original.to_json()?)?;
        let request = output.request(&p, export_options(PageSize::points(600., 360.)?).dpi(144))?;
        let frame = request.prepare()?;
        let original_frame = output
            .request(
                &original,
                export_options(PageSize::points(600., 360.)?).dpi(144),
            )?
            .prepare()?;
        assert_eq!(frame.scene_json()?, original_frame.scene_json()?);
        std::fs::write(out.join(format!("spatial-{mode}.plot.json")), p.to_json()?)?;
        std::fs::write(
            out.join(format!("spatial-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("spatial-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!("PASS Rust spatial controls: sixteen authors, 48 publications.");
    Ok(())
}
