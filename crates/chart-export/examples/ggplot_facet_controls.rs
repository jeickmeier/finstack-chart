//! Independent GG12 Rust authors, replay, and SVG/PDF/PNG publication.
#[path = "../../../examples/common/ggplot_facet_controls.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    for mode in 0..fixtures::CASES {
        let p = fixtures::author(mode)?;
        let restored = chart_core::prelude::Plot::from_json(&p.to_json()?)?;
        let frame = output
            .request(
                &p,
                export_options(PageSize::points(800., 560.)?)
                    .dpi(144)
                    .layout(chart_core::prelude::layout_options().minimum_plot((0.1, 0.1))),
            )?
            .prepare()?;
        let replay = output
            .request(
                &restored,
                export_options(PageSize::points(800., 560.)?)
                    .dpi(144)
                    .layout(chart_core::prelude::layout_options().minimum_plot((0.1, 0.1))),
            )?
            .prepare()?;
        assert_eq!(frame.scene_json()?, replay.scene_json()?);
        std::fs::write(out.join(format!("facet-{mode}.plot.json")), p.to_json()?)?;
        std::fs::write(
            out.join(format!("facet-{mode}.scene.json")),
            frame.scene_json()?,
        )?;
        for (format, suffix) in [
            (Format::Svg, "svg"),
            (Format::Pdf, "pdf"),
            (Format::Png, "png"),
        ] {
            std::fs::write(
                out.join(format!("facet-{mode}.{suffix}")),
                frame.export(format)?.bytes,
            )?;
        }
    }
    println!(
        "PASS Rust GG12 facets: seventeen authors, original/replay equality, 51 publications."
    );
    Ok(())
}
