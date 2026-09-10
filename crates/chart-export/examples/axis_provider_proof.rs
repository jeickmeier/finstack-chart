//! AXIS-01 actual registered provider publication via shared primary authoring.
#[path = "../../../examples/common/axis_provider_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let plot = fixtures::figure()?;
    let wire = plot.to_json()?;
    assert_eq!(plot.definition().wire_version(), 10);
    assert_eq!(
        chart_core::plot::Plot::from_json_with_extensions(
            &wire,
            chart_extension_example::registry()?
        )?
        .to_json()?,
        wire
    );
    let request = output.request(
        &plot,
        export_options(PageSize::points(600., 400.)?)
            .dpi(72)
            .basis(CaptureBasis::Current),
    )?;
    drop(plot);
    let frame = request.prepare()?;
    std::fs::write(out.join("provider.plot.json"), wire)?;
    std::fs::write(out.join("provider.scene.json"), frame.scene_json()?)?;
    for (name, format) in [
        ("svg", Format::Svg),
        ("pdf", Format::Pdf),
        ("png", Format::Png),
    ] {
        std::fs::write(
            out.join(format!("provider.{name}")),
            frame.export(format)?.bytes,
        )?;
    }
    println!(
        "PASS Rust provider: v10 registered round trip, immutable request, SVG/PDF/PNG publication."
    );
    Ok(())
}
