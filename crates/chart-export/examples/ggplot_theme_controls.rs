//! FIX-GG14 independently authored Rust text/raster publications.
#[path = "../../../examples/common/ggplot_theme_controls.rs"]
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
        std::fs::write(out.join(format!("theme-{mode}.plot.json")), p.to_json()?)?;
        std::fs::write(
            out.join(format!("theme-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("theme-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
        if mode == 14 {
            for dpi in [300, 600] {
                for (text, label) in [
                    (TextMode::Preserve, "preserve"),
                    (TextMode::Outline, "outline"),
                ] {
                    let variant = output
                        .request(
                            &p,
                            export_options(PageSize::points(600., 360.)?)
                                .dpi(dpi)
                                .text(text),
                        )?
                        .prepare()?;
                    assert_eq!(variant.scene_json()?, frame.scene_json()?);
                    for (format, suffix) in [
                        (Format::Svg, "svg"),
                        (Format::Pdf, "pdf"),
                        (Format::Png, "png"),
                    ] {
                        std::fs::write(
                            out.join(format!("theme-{mode}.{label}-{dpi}.{suffix}")),
                            variant.export(format)?.bytes,
                        )?;
                    }
                }
            }
        }
    }
    println!(
        "PASS Rust theme controls: sixteen authors, 60 publications including text/outline at300/600dpi."
    );
    Ok(())
}
