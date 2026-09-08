//! FIX-AUTH02/04: one primary plot published through a reusable supplied-font destination.
use chart_core::prelude::*;
use chart_export::{Output, PageSize};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let destination = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/authoring".into());
    std::fs::create_dir_all(&destination)?;
    let data = Data::columns()
        .column("time", [1., 2., 3., 4., 5.])
        .column("value", [10., 12., 11., 14., 13.])
        .build()?;
    let plot = plot(data)
        .aes(aes().x("time").y("value"))
        .layer(line().size(1.5))
        .layer(points().size(3.))
        .title(title("Prices"))
        .subtitle(subtitle(
            "One plot, shared native and publication semantics",
        ))
        .x_axis(x_axis().label("Time"))
        .y_axis(y_axis().label("USD"))
        .caption(caption("Authored from ordinary named columns"))
        .source_note(source_note("Source: independent example values"))
        .theme(theme().preset(NamedTheme::Editorial))
        .build()?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let size = PageSize::millimeters(180., 120.)?;
    for (extension, bytes) in [
        ("svg", output.svg(&plot, size)?),
        ("pdf", output.pdf(&plot, size)?),
        ("png", output.png(&plot, size)?),
    ] {
        let path =
            std::path::Path::new(&destination).join(format!("primary-authoring.{extension}"));
        std::fs::write(&path, bytes)?;
        println!("{}", path.display());
    }
    Ok(())
}
