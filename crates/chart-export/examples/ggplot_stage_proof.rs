//! FIX-GG02: actual Rust authors and immutable profile-aware publication.
#[path = "../../../examples/common/ggplot_stage_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let options = export_options(PageSize::points(420., 280.)?).dpi(96);
    for (name, plot) in fixtures::figures()? {
        std::fs::write(out.join(format!("{name}.plot.json")), plot.to_json()?)?;
        let mut chart = chart_export::host::Runtime::new(&plot)?;
        std::fs::write(
            out.join(format!("{name}.semantics.json")),
            chart.semantics()?,
        )?;
        let request = output.request(&plot, options.clone())?;
        let frame = request.prepare()?;
        assert_eq!(
            frame.metadata().definition.profile(),
            chart_core::prelude::Profile::Ggplot2_4_0_3
        );
        std::fs::write(out.join(format!("{name}.scene.json")), frame.scene_json()?)?;
        for (format, ext) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("{name}.{ext}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!(
        "PASS FIX-GG02 Rust: 13 primary stage figures with retained semantic profile and SVG/PDF/PNG publication."
    );
    Ok(())
}
