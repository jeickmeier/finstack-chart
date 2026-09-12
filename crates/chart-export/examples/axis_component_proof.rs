//! AXIS-05 Rust primary publication with addressable components and retained styles.
#[path = "../../../examples/common/axis_component_fixtures.rs"]
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
    assert_eq!(plot.definition().wire_version(), 14);
    let loaded = chart_core::plot::Plot::from_json_with_extensions(
        &wire,
        chart_extension_example::registry()?,
    )?;
    assert_eq!(loaded.to_json()?, wire);
    std::fs::write(out.join("components.plot.json"), wire)?;
    for (name, text) in [("text", TextMode::Preserve), ("outline", TextMode::Outline)] {
        for dpi in [300, 600] {
            let frame = output
                .request(
                    &loaded,
                    export_options(PageSize::points(600., 400.)?)
                        .dpi(dpi)
                        .text(text)
                        .basis(CaptureBasis::Current),
                )?
                .prepare()?;
            assert_eq!(frame.scene().wire_version(), 14);
            let prefix = format!("{name}-{dpi}");
            std::fs::write(
                out.join(format!("{prefix}.guides.json")),
                frame.guides_json()?,
            )?;
            std::fs::write(
                out.join(format!("{prefix}.scene.json")),
                frame.scene_json()?,
            )?;
            for (extension, format) in [
                ("svg", Format::Svg),
                ("pdf", Format::Pdf),
                ("png", Format::Png),
            ] {
                std::fs::write(
                    out.join(format!("{prefix}.{extension}")),
                    frame.export(format)?.bytes,
                )?;
            }
        }
    }
    {
        let p = fixtures::typography()?;
        let frame = output
            .request(
                &p,
                export_options(PageSize::points(400., 300.)?)
                    .dpi(72)
                    .basis(CaptureBasis::Current),
            )?
            .prepare()?;
        std::fs::write(out.join("typography.plot.json"), p.to_json()?)?;
        std::fs::write(out.join("typography.scene.json"), frame.scene_json()?)?;
        for (extension, format) in [
            ("svg", Format::Svg),
            ("pdf", Format::Pdf),
            ("png", Format::Png),
        ] {
            std::fs::write(
                out.join(format!("typography.{extension}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!(
        "PASS AX04: v14 round trip, retained component roles and text/outline SVG/PDF/PNG at 300/600 DPI."
    );
    Ok(())
}
