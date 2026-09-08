//! Primary headless publication with supplied fonts and explicit physical dimensions.
use chart_core::prelude::*;
use chart_export::{Format, Output, PageSize, TextMode, export_options};
use std::{fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "target/authoring/publication".into()),
    );
    fs::create_dir_all(&output)?;
    let data = Data::rows([
        (1u64, 0., Some(1.)),
        (2, 0.5, Some(3.)),
        (3, 1., None),
        (4, 1.5, Some(2.)),
        (5, 2., Some(4.)),
    ])
    .field("x", |row| row.1)
    .field("value", |row| row.2)
    .keys(|row| row.0)
    .build()?;
    let plot = plot(data)
        .aes(aes().x("x").y("value"))
        .layer(line())
        .layer(points())
        .title(title("Captured publication — café Ω"))
        .build()?;
    let destination = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let options = export_options(PageSize::millimeters(180., 120.)?)
        .layout(layout_options().padding(30.).font_size(10.));
    for mode in [TextMode::Preserve, TextMode::Outline] {
        let snapshot = destination
            .request(&plot, options.clone().text(mode))?
            .prepare()?;
        let name = if mode == TextMode::Preserve {
            "text"
        } else {
            "outline"
        };
        for (format, extension) in [(Format::Svg, "svg"), (Format::Pdf, "pdf")] {
            let artifact = snapshot.export(format)?;
            fs::write(
                output.join(format!("publication-{name}.{extension}")),
                &artifact.bytes,
            )?;
            println!(
                "{format:?} {mode:?}: {} bytes; stamp={:?}; diagnostics={:?}",
                artifact.bytes.len(),
                artifact.metadata.stamp,
                artifact.diagnostics
            );
        }
        if mode == TextMode::Preserve {
            fs::write(
                output.join("publication-preview.svg"),
                snapshot.preview_svg()?,
            )?;
            fs::write(
                output.join("metadata.txt"),
                format!("{:#?}\n", snapshot.metadata()),
            )?;
            fs::write(
                output.join("geometry.txt"),
                format!(
                    "plot={:?}\nscene={:#?}\n",
                    snapshot.layout().plot(),
                    snapshot.scene()
                ),
            )?;
        }
    }
    for dpi in [300, 600] {
        let snapshot = destination
            .request(&plot, options.clone().dpi(dpi))?
            .prepare()?;
        let artifact = snapshot.export(Format::Png)?;
        fs::write(
            output.join(format!("publication-{dpi}.png")),
            artifact.bytes,
        )?;
        println!(
            "PNG {dpi} DPI: {:?}",
            snapshot.metadata().profile.raster_dimensions()?
        );
    }
    Ok(())
}
