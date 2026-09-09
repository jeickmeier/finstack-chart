//! CLR-04 actual Rust authoring and immutable publication through retained paint descriptors.
#[path = "../../../examples/common/paint_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let background = chart_core::color::hsl(210., 0.1, 0.97);
    let options = export_options(PageSize::points(440., 340.)?)
        .dpi(96)
        .background(background);
    for (name, plot) in fixtures::figures()? {
        assert_eq!(plot.definition().wire_version(), 4);
        let wire = plot.to_json()?;
        assert_eq!(chart_core::plot::Plot::from_json(&wire)?.to_json()?, wire);
        std::fs::write(out.join(format!("{name}.plot.json")), wire)?;
        let request = output.request(&plot, options.clone())?;
        let frame = request.prepare()?;
        assert_eq!(
            frame.metadata().profile.background.value(),
            chart_core::color::ColorValue::from(background)
        );
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
    println!(
        "PASS CLR-04 Rust: four public paint figures, version-four round trips and retained output background."
    );
    Ok(())
}
