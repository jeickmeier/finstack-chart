//! FIX-21 retained physical publication for perceptual color, overlap and grayscale.
#[path = "../../../examples/common/chromatic_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let options = export_options(PageSize::points(720., 420.)?).dpi(144);
    for (name, plot) in fixtures::figures()? {
        let json = plot.to_json()?;
        assert_eq!(plot.definition().wire_version(), 6);
        assert_eq!(chart_core::plot::Plot::from_json(&json)?.to_json()?, json);
        std::fs::write(out.join(format!("{name}.plot.json")), json)?;
        let request = output.request(&plot, options.clone())?;
        drop(plot);
        let frame = request.prepare()?;
        std::fs::write(out.join(format!("{name}.scene.json")), frame.scene_json()?)?;
        for (format, extension) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("{name}.{extension}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!("PASS FIX-21: five v6 chromatic figures and immutable SVG/PDF/PNG publication.");
    Ok(())
}
