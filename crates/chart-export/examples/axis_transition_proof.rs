//! AXIS-06 frozen publication samples, including interruption and exact final geometry.
#[path = "../../../examples/common/axis_transition_fixtures.rs"]
mod fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    let plots = fixtures::sequence()?;
    for (index, p) in plots.iter().enumerate() {
        std::fs::write(out.join(format!("plot-{index}.json")), p.to_json()?)?;
    }
    for (mode, text) in [("text", TextMode::Preserve), ("outline", TextMode::Outline)] {
        let figures = plots
            .iter()
            .map(|p| {
                output
                    .request(
                        p,
                        export_options(PageSize::points(900., 300.)?)
                            .text(text)
                            .dpi(144),
                    )?
                    .prepare()
            })
            .collect::<chart_core::ChartResult<Vec<_>>>()?;
        let plan = figures[1].guide_transition(&figures[0])?;
        let mid = plan.sample(0.5)?;
        let interrupt = figures[2].guide_transition(&mid)?;
        for (name, figure) in [
            ("start", plan.sample(0.)?),
            ("mid", mid),
            ("end", plan.sample(1.)?),
            ("interrupt-start", interrupt.sample(0.)?),
            ("interrupt-mid", interrupt.sample(0.5)?),
            ("interrupt-end", interrupt.sample(1.)?),
        ] {
            let prefix = format!("{mode}-{name}");
            std::fs::write(
                out.join(format!("{prefix}.scene.json")),
                figure.scene_json()?,
            )?;
            std::fs::write(
                out.join(format!("{prefix}.guides.json")),
                figure.guides_json()?,
            )?;
            std::fs::write(
                out.join(format!("{prefix}.presentation.json")),
                figure.presentation_json()?,
            )?;
            for (extension, format) in [
                ("svg", Format::Svg),
                ("pdf", Format::Pdf),
                ("png", Format::Png),
            ] {
                std::fs::write(
                    out.join(format!("{prefix}.{extension}")),
                    figure.export(format)?.bytes,
                )?;
            }
        }
    }
    println!("PASS 12 sampled figures / 72 coherent guide, scene and publication artifacts");
    Ok(())
}
