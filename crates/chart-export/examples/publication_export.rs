//! Real headless chart export through the public WP-08 snapshot API.
#[path = "../../../fixtures/publication/support.rs"]
mod fixture;
use chart_core::state::ChartState;
use chart_export::{FigureSnapshot, Format, TextMode};
use std::{fs, path::PathBuf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/wp-08".into()),
    );
    fs::create_dir_all(&output)?;
    let source = fixture::store().snapshot();
    let definition = fixture::definition();
    let fonts = fixture::fonts();
    for mode in [TextMode::Preserve, TextMode::Outline] {
        let mut profile = fixture::profile();
        profile.text = mode;
        let snapshot = FigureSnapshot::capture(
            &definition,
            source.clone(),
            &ChartState::default(),
            fonts.clone(),
            profile,
        )?;
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
        let mut profile = fixture::profile();
        profile.dpi = dpi;
        let snapshot = FigureSnapshot::capture(
            &definition,
            source.clone(),
            &ChartState::default(),
            fonts.clone(),
            profile,
        )?;
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
