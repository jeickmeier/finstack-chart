//! ITP-07 explicit sampled frames, registered ramps and retained publication.
#[path = "../../../examples/common/interpolation_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for t in [0., 0.5, 1.] {
        let out = out.join(format!("frame-{t}"));
        std::fs::create_dir_all(&out)?;
        let plot = fixtures::figure(t, false)?;
        let wire = plot.to_json()?;
        assert_eq!(plot.definition().wire_version(), 12);
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
        std::fs::write(out.join("interpolation.plot.json"), wire)?;
        std::fs::write(out.join("interpolation.guides.json"), frame.guides_json()?)?;
        std::fs::write(out.join("interpolation.scene.json"), frame.scene_json()?)?;
        for (name, format) in [
            ("svg", Format::Svg),
            ("pdf", Format::Pdf),
            ("png", Format::Png),
        ] {
            std::fs::write(
                out.join(format!("interpolation.{name}")),
                frame.export(format)?.bytes,
            )?;
        }
        let second = output.request(&loaded, options)?.prepare()?;
        assert_eq!(frame.guides_json()?, second.guides_json()?);
        assert_eq!(
            frame.export(Format::Png)?.bytes,
            second.export(Format::Png)?.bytes
        );
    }
    let plot = fixtures::figure(0.5, false)?;
    let session = chart_core::portable::Session::from_runtime(plot.chart()?)?;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&session.chart_json()?)?["version"],
        12
    );
    println!(
        "PASS Rust IP06: three v12 registered ramp/transform/zoom frames, retained SVG/PDF/PNG."
    );
    Ok(())
}
