//! HIR-07: primary hierarchy recipes rendered through immutable publication.
#[path = "../../../examples/common/hierarchy_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for (name, plot) in fixtures::figures()? {
        let wire = plot.to_json()?;
        assert_eq!(plot.definition().wire_version(), 15);
        let loaded = chart_core::plot::Plot::from_json(&wire)?;
        assert_eq!(loaded.to_json()?, wire);
        std::fs::write(out.join(format!("{name}.plot.json")), wire)?;
        for (mode, text) in [("text", TextMode::Preserve), ("outline", TextMode::Outline)] {
            let frame = output
                .request(
                    &loaded,
                    export_options(PageSize::points(500., 360.)?)
                        .dpi(144)
                        .text(text),
                )?
                .prepare()?;
            std::fs::write(
                out.join(format!("{name}-{mode}.scene.json")),
                frame.scene_json()?,
            )?;
            for (ext, format) in [
                ("svg", Format::Svg),
                ("pdf", Format::Pdf),
                ("png", Format::Png),
            ] {
                std::fs::write(
                    out.join(format!("{name}-{mode}.{ext}")),
                    frame.export(format)?.bytes,
                )?;
            }
        }
    }
    println!(
        "PASS HIR-07: nine primary projections, v15 exact round trips, text/outline SVG/PDF/PNG."
    );
    Ok(())
}
