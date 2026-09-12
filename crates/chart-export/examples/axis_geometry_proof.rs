//! AXIS-04 Rust primary publication and exact retained guide records.
#[path = "../../../examples/common/axis_geometry_fixtures.rs"]
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
    assert_eq!(plot.definition().wire_version(), 13);
    let loaded = chart_core::plot::Plot::from_json_with_extensions(
        &wire,
        chart_extension_example::registry()?,
    )?;
    assert_eq!(loaded.to_json()?, wire);
    let options = export_options(PageSize::points(600., 400.)?)
        .dpi(72)
        .basis(CaptureBasis::Current);
    let request = output.request(&plot, options.clone())?;
    drop(plot);
    let frame = request.prepare()?;
    std::fs::write(out.join("geometry.plot.json"), wire)?;
    std::fs::write(out.join("geometry.guides.json"), frame.guides_json()?)?;
    std::fs::write(out.join("geometry.scene.json"), frame.scene_json()?)?;
    for (name, format) in [
        ("svg", Format::Svg),
        ("pdf", Format::Pdf),
        ("png", Format::Png),
    ] {
        std::fs::write(
            out.join(format!("geometry.{name}")),
            frame.export(format)?.bytes,
        )?;
    }
    let second = output.request(&loaded, options)?.prepare()?;
    assert_eq!(frame.guides_json()?, second.guides_json()?);
    assert_eq!(
        frame.export(Format::Png)?.bytes,
        second.export(Format::Png)?.bytes
    );
    println!(
        "PASS Rust AX03: v13 primary round trip, independent labels/values, retained guides and SVG/PDF/PNG."
    );
    Ok(())
}
