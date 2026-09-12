//! AXIS-02 Rust primary publication and exact retained guide records.
#[path = "../../../examples/common/axis_tick_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let authored = fixtures::figure()?;
    // Fix authored fixture identities independently of earlier process allocations.
    let mut descriptor: serde_json::Value = serde_json::from_str(&authored.to_json()?)?;
    for (index, name) in ["top", "lower"].into_iter().enumerate() {
        let id = (1026 + index).to_string();
        descriptor["definition"]["guides"][index]["id"] = id.clone().into();
        descriptor["guides"][name] = id.into();
    }
    let plot = chart_core::plot::Plot::from_json_with_extensions(
        &descriptor.to_string(),
        chart_extension_example::registry()?,
    )?;
    let wire = plot.to_json()?;
    assert_eq!(plot.definition().wire_version(), 11);
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
    std::fs::write(out.join("ticks.plot.json"), wire)?;
    std::fs::write(out.join("ticks.guides.json"), frame.guides_json()?)?;
    std::fs::write(out.join("ticks.scene.json"), frame.scene_json()?)?;
    for (name, format) in [
        ("svg", Format::Svg),
        ("pdf", Format::Pdf),
        ("png", Format::Png),
    ] {
        std::fs::write(
            out.join(format!("ticks.{name}")),
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
        "PASS Rust AX02: v11 primary round trip, independent labels/values, retained guides and SVG/PDF/PNG."
    );
    Ok(())
}
