//! FIX-GG05: complete sampled ramp publication through the common layout engine.
#[path = "../../../examples/common/ggplot_colorsteps_boundary_fixtures.rs"]
mod boundary_fixtures;
#[path = "../../../examples/common/ggplot_colorbar_fixtures.rs"]
mod fixtures;
#[path = "../../../examples/common/ggplot_colorsteps_fixtures.rs"]
mod steps_fixtures;
use chart_export::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::path::PathBuf::from(std::env::args().nth(1).ok_or("Supply output directory")?);
    std::fs::create_dir_all(&out)?;
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )?;
    if std::env::args().any(|arg| arg == "--steps-boundaries") {
        return steps_boundaries_proof(&output, &out);
    }
    if std::env::args().any(|arg| arg == "--steps") {
        return steps_proof(&output, &out);
    }
    if std::env::args().any(|arg| arg == "--orientation") {
        return presentation_proof(&output, &out, true);
    }
    if std::env::args().any(|arg| arg == "--presentation") {
        return presentation_proof(&output, &out, false);
    }
    if std::env::args().any(|arg| arg == "--display" || arg == "--alpha") {
        return display_proof(&output, &out);
    }
    let sampling = std::env::args().any(|arg| arg == "--sampling");
    let constant = std::env::args().any(|arg| arg == "--constant");
    let counts: &[Option<f64>] = if constant {
        &[Some(0.), Some(1.), Some(2.5), Some(300.)]
    } else if sampling {
        &[Some(0.), Some(1.), Some(2.5), Some(5.)]
    } else {
        &[None]
    };
    for nbin in counts {
        for (i, palette) in ["ordinary", "asymmetric", "discontinuous"]
            .into_iter()
            .enumerate()
        {
            let facets: &[&str] = if constant {
                &["single"]
            } else {
                &["single", "collected", "local"]
            };
            for (j, facet) in facets.iter().enumerate() {
                let channel = if constant {
                    "color"
                } else {
                    ["color", "fill", "stroke"][(i + j) % 3]
                };
                let name = format!(
                    "{palette}-{channel}-{facet}{}",
                    nbin.map_or(String::new(), |n| format!("-nbin-{n}"))
                );
                let plot = fixtures::author(palette, channel, facet, false);
                let plot = if let Some(n) = nbin {
                    let scale = fixtures::scale(palette, false);
                    let scale = if constant {
                        use chart_core::{
                            interpolate::Number,
                            scales::{
                                GgplotContinuousGuide, GgplotOob, GgplotScaleGuide,
                                GgplotScalePolicy,
                            },
                        };
                        scale
                            .with_ggplot(GgplotScalePolicy::Continuous {
                                limits: Some([Some(Number(3.)), Some(Number(3.))]),
                                empty_population: false,
                                nonfinite_population: false,
                                oob: GgplotOob::Censor,
                            })?
                            .with_guide(GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
                                breaks: Some(vec![Number(3.)]),
                                ..Default::default()
                            }))?
                    } else {
                        scale
                    };
                    plot.edit()
                        .scale(chart_core::prelude::color_mapped(
                            "v",
                            scale.with_colorbar_options(
                                chart_core::scales::GgplotColorbarOptions {
                                    nbin: Some(chart_core::interpolate::Number(*n)),
                                    ..Default::default()
                                },
                            ),
                        ))
                        .build()?
                } else {
                    plot
                };
                publish(&output, &plot, &out, &name)?;
            }
        }
    }
    println!(
        "PASS Rust GG-05: {} SVG/PDF/PNG triplets.",
        counts.len() * if constant { 3 } else { 9 }
    );
    Ok(())
}

fn publish(
    output: &Output,
    plot: &chart_core::prelude::Plot,
    out: &std::path::Path,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = output.request(plot, export_options(PageSize::points(600., 360.)?).dpi(144))?;
    let frame = request.prepare()?;
    std::fs::write(out.join(format!("{name}.plot.json")), plot.to_json()?)?;
    std::fs::write(out.join(format!("{name}.scene.json")), frame.scene_json()?)?;
    for (format, suffix) in [
        (Format::Svg, "svg"),
        (Format::Pdf, "pdf"),
        (Format::Png, "png"),
    ] {
        std::fs::write(
            out.join(format!("{name}.{suffix}")),
            frame.export(format)?.bytes,
        )?;
    }
    Ok(())
}
fn presentation_proof(
    output: &Output,
    out: &std::path::Path,
    orientation: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use chart_core::{
        interpolate::Number,
        prelude::color_mapped,
        scales::{
            GgplotColorbarOptions, GgplotContinuousGuide, GgplotGuideLabels, GgplotScaleGuide,
        },
        scene::GradientDirection,
    };
    let fixture: serde_json::Value = serde_json::from_str(if orientation {
        include_str!("../../../fixtures/parity/ggplot2/colorbar-boundaries.json")
    } else {
        include_str!("../../../fixtures/parity/ggplot2/colorbar-presentation.json")
    })?;
    let mut count = 0;
    for (index, case) in fixture["cases"]
        .as_array()
        .ok_or("source cases")?
        .iter()
        .filter(|c| c["display"] == "raster")
        .enumerate()
    {
        let nbin = case["nbin"].as_f64().ok_or("source sample count")?;
        if !orientation
            && !(nbin == 5.
                || nbin > 0.
                    && case["lower"] == true
                    && case["upper"] == true
                    && case["labels"] == "automatic")
        {
            continue;
        }
        let palette = case["palette"].as_str().ok_or("source palette")?;
        let channel = if orientation {
            ["color", "fill", "stroke"][index % 3]
        } else {
            "color"
        };
        let facet = ["single", "collected", "local"][if orientation {
            index / 3 % 3
        } else {
            index % 3
        }];
        let mut scale = fixtures::scale(palette, false);
        if case["labels"] == "hidden" {
            scale = scale.with_guide(GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
                breaks: Some([-2., 0., 3., 8.].map(Number).to_vec()),
                labels: GgplotGuideLabels::Hidden,
                ..Default::default()
            }))?;
        }
        let scale = scale.with_colorbar_options(GgplotColorbarOptions {
            nbin: Some(Number(nbin)),
            direction: Some(if case["direction"] == "horizontal" {
                GradientDirection::Horizontal
            } else {
                GradientDirection::Vertical
            }),
            reverse: case["reverse"].as_bool().ok_or("source reversal")?,
            draw_lower_limit: case["lower"].as_bool().unwrap_or(true),
            draw_upper_limit: case["upper"].as_bool().unwrap_or(true),
            ..Default::default()
        });
        let plot = fixtures::author(palette, channel, facet, false)
            .edit()
            .scale(color_mapped("v", scale))
            .build()?;
        let name = format!(
            "{}-{index:03}",
            if orientation {
                "orientation"
            } else {
                "presentation"
            }
        );
        publish(output, &plot, out, &name)?;
        count += 1;
    }
    assert_eq!(count, if orientation { 48 } else { 40 });
    println!("PASS Rust GG-05: {count} presentation SVG/PDF/PNG triplets.");
    Ok(())
}

fn display_proof(output: &Output, out: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use chart_core::{
        interpolate::Number, prelude::color_mapped, scales::*, scene::GradientDirection,
    };
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-display.json"
    ))?;
    let boundaries: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-boundaries.json"
    ))?;
    let alpha = std::env::args().any(|arg| arg == "--alpha");
    let alpha_source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorbar-alpha.json"
    ))?;
    let cases = source["cases"]
        .as_array()
        .ok_or("source display cases")?
        .iter()
        .chain(
            boundaries["cases"]
                .as_array()
                .ok_or("source boundaries")?
                .iter()
                .filter(|c| c["display"] != "raster"),
        );
    let mut count = 0;
    let cases: Vec<_> = if alpha {
        alpha_source["cases"]
            .as_array()
            .ok_or("alpha cases")?
            .iter()
            .collect()
    } else {
        cases.collect()
    };
    for (index, case) in cases.into_iter().enumerate() {
        if if alpha {
            index % 4 != (index / 4) % 4
        } else {
            !(index < 80 && (case["nbin"].is_null() || case["nbin"] == 2.5)
                || index >= 80 && case["palette"] == "discontinuous" && case["nbin"] == 5)
        } {
            continue;
        }
        let palette = case["palette"].as_str().ok_or("source palette")?;
        let channel = ["color", "fill", "stroke"][index % 3];
        let facet = ["single", "collected", "local"][index / 3 % 3];
        let constant = case["constant"] == true;
        let mut descriptor = fixtures::scale(palette, false);
        if constant {
            descriptor = descriptor
                .with_ggplot(GgplotScalePolicy::Continuous {
                    limits: Some([Some(Number(2.)), Some(Number(2.))]),
                    empty_population: false,
                    nonfinite_population: false,
                    oob: GgplotOob::Censor,
                })?
                .with_guide(GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
                    breaks: Some(vec![Number(2.)]),
                    ..Default::default()
                }))?;
        }
        descriptor = descriptor.with_colorbar_options(GgplotColorbarOptions {
            nbin: case["nbin"].as_f64().map(Number),
            alpha: case["alpha"].as_f64().map(Number),
            display: if case["display"] == "raster" {
                GgplotColorbarDisplay::Raster
            } else if case["display"] == "gradient" {
                GgplotColorbarDisplay::Gradient
            } else {
                GgplotColorbarDisplay::Rectangles
            },
            direction: Some(if case["direction"] == "horizontal" {
                GradientDirection::Horizontal
            } else {
                GradientDirection::Vertical
            }),
            reverse: case["reverse"].as_bool().ok_or("source reversal")?,
            ..Default::default()
        });
        let values = if constant {
            [2.; 8]
        } else {
            [-2., 0., 3., 8., -2., 0., 3., 8.]
        };
        let plot = fixtures::author_values(palette, channel, facet, false, values)
            .edit()
            .scale(color_mapped("v", descriptor))
            .build()?;
        publish(
            output,
            &plot,
            out,
            &format!("{}-{index:03}", if alpha { "alpha" } else { "display" }),
        )?;
        count += 1;
    }
    assert_eq!(count, if alpha { 24 } else { 40 });
    println!(
        "PASS primary Rust display publications: {} files.",
        count * 3
    );
    Ok(())
}

fn steps_proof(output: &Output, out: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use chart_core::{prelude::color_mapped, scales::*, scene::GradientDirection};
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorsteps-layout.json"
    ))?;
    let mut count = 0;
    for (index, case) in source["cases"]
        .as_array()
        .ok_or("step cases")?
        .iter()
        .filter(|c| c["even_steps"] == true && c["show_limits"] == false)
        .enumerate()
    {
        let mut descriptor = steps_fixtures::steps_scale(
            case["family"].as_str().ok_or("family")?,
            case["endpoints"].as_bool().ok_or("endpoints")?,
            false,
        );
        if case["guide_kind"] == "default" {
            descriptor =
                descriptor.with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))?;
        }
        descriptor = descriptor.with_colorbar_options(GgplotColorbarOptions {
            direction: Some(if case["direction"] == "horizontal" {
                GradientDirection::Horizontal
            } else {
                GradientDirection::Vertical
            }),
            reverse: case["reverse"].as_bool().ok_or("reverse")?,
            ..Default::default()
        });
        let channel = ["color", "fill", "stroke"][index % 3];
        let facet = ["single", "collected", "local"][index / 3 % 3];
        let plot = fixtures::author("asymmetric", channel, facet, false)
            .edit()
            .scale(color_mapped("v", descriptor))
            .build()?;
        publish(output, &plot, out, &format!("steps-{index:03}"))?;
        count += 1;
    }
    assert_eq!(count, 18);
    println!(
        "PASS primary Rust stepped guide publications: {} files.",
        count * 3
    );
    Ok(())
}

fn steps_boundaries_proof(
    output: &Output,
    out: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use chart_core::{prelude::color_mapped, scales::*, scene::GradientDirection};
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/parity/ggplot2/colorsteps-boundaries.json"
    ))?;
    let mut count = 0;
    for (index, case) in source["cases"]
        .as_array()
        .ok_or("cases")?
        .iter()
        .filter(|c| !(c["family"] == "binned" && c["mode"] == "null"))
        .enumerate()
    {
        if case["population"] != "ordinary" || case["result"]["bar"].is_null() {
            continue;
        }
        let descriptor =
            boundary_fixtures::boundary_scale(case)?.with_colorbar_options(GgplotColorbarOptions {
                direction: Some(GradientDirection::Vertical),
                ..Default::default()
            });
        let plot = fixtures::author(
            "asymmetric",
            ["color", "fill", "stroke"][index % 3],
            "single",
            false,
        )
        .edit()
        .scale(color_mapped("v", descriptor))
        .build()?;
        publish(output, &plot, out, &format!("step-boundaries-{index:03}"))?;
        count += 1;
    }
    assert_eq!(count, 16);
    println!(
        "PASS primary Rust stepped boundary publications: {} files.",
        count * 3
    );
    Ok(())
}
