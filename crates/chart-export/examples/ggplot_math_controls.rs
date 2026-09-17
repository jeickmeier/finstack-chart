//! FIX-GG14 independently authored Rust text/raster publications.
#[path = "../../../examples/common/ggplot_math_controls.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let mut output =
        Output::new(include_bytes!("../../../fixtures/math/fonts/DejaVuSerif.ttf").as_slice())?;
    let fonts = chart_core::typography::MathFonts {
        regular: Some(output.primary_font()),
        italic: Some(output.register_font(
            include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Italic.ttf").as_slice(),
        )?),
        bold: Some(output.register_font(
            include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Bold.ttf").as_slice(),
        )?),
        bold_italic: Some(output.register_font(
            include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-BoldItalic.ttf").as_slice(),
        )?),
        symbol: Some(output.register_font(
            include_bytes!("../../../fixtures/math/fonts/DejaVuMathTeXGyre.ttf").as_slice(),
        )?),
    };
    let fallback = output
        .register_font(include_bytes!("../../../fixtures/math/fonts/DejaVuSans.ttf").as_slice())?;
    for mode in 0..fixtures::CASES {
        let original = fixtures::author(mode, fonts.clone(), fallback)?;
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
        std::fs::write(out.join(format!("math-{mode}.plot.json")), p.to_json()?)?;
        std::fs::write(
            out.join(format!("math-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("math-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!("PASS Rust math controls: nine authors, 27 publications.");
    Ok(())
}
