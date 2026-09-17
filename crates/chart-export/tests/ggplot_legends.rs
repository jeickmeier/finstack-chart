//! FIX-GG01: shared legend acceptance through primary authoring and real publication.
use chart_core::{prelude::*, scene::Primitive, state::ChartAction, theme::rgb};
use chart_export::*;

fn authored(case: &str, faceted: bool, collect: bool) -> Plot {
    let data = Data::columns()
        .column("x", [1., 2.])
        .column("y", [2., 4.])
        .column("series", ["Alpha", "Beta"])
        .column("panel", ["One", "One"])
        .build()
        .unwrap();
    let mut scale = color_discrete("series").palette(vec![rgb(0, 80, 180), rgb(200, 20, 40)]);
    if case == "empty" {
        scale = scale.domain(Vec::<String>::new());
    }
    let mut builder = plot(data)
        .aes(aes().x("x").y("y").color("series"))
        .scale(scale)
        .layer(points())
        .legend(if case == "untitled" {
            legend().scale("series").untitled()
        } else {
            legend().scale("series").title("Series")
        });
    if case == "shared" {
        builder = builder.layer(points());
    } else if case == "incompatible" {
        builder = builder
            .scale(color_discrete("other").palette(vec![rgb(200, 20, 40), rgb(0, 80, 180)]))
            .layer(points().aes(aes().color("series").color_scale("other")))
            .legend(legend().scale("other").title("Other"));
    }
    if faceted {
        builder = builder.facet(facet_wrap("panel").collect_guides(collect));
    }
    let plot = builder.build().unwrap();
    if case == "edited-untitled" {
        plot.edit()
            .legend(legend().scale("series").untitled())
            .build()
            .unwrap()
    } else {
        plot
    }
}

fn text(item: &chart_core::scene::SceneItem) -> Option<&str> {
    match &item.primitive {
        Primitive::Text { text, .. } => Some(text),
        Primitive::GlyphRun { run, .. } => Some(&run.text),
        _ => None,
    }
}

#[test]
fn fix_gg01_primary_legend_matrix() {
    let output = Output::new(
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
    )
    .unwrap();
    for (faceted, collect) in [(false, true), (true, true), (true, false)] {
        for case in [
            "two-entry",
            "empty",
            "hidden",
            "tight",
            "shared",
            "incompatible",
            "untitled",
            "edited-untitled",
        ] {
            let name = format!("{case}-facet{faceted}-collect{collect}");
            let plot =
                Plot::from_json(&authored(case, faceted, collect).to_json().unwrap()).unwrap();
            let mut chart = plot.chart().unwrap();
            if case == "hidden" {
                chart.act(ChartAction::SetLegendVisible(false)).unwrap();
            }
            let (width, height) = if case == "tight" {
                (130., 65.)
            } else {
                (500., 300.)
            };
            let options = export_options(PageSize::points(width, height).unwrap())
                .dpi(96)
                .basis(CaptureBasis::Current);
            let snapshot = output
                .live_request(&chart, options)
                .unwrap()
                .prepare()
                .unwrap();
            let scene = snapshot.layout().scene();
            let count = |expected| {
                scene
                    .items()
                    .iter()
                    .filter(|i| text(i) == Some(expected))
                    .count()
            };
            let visible = !matches!(case, "hidden" | "empty");
            if case != "tight" {
                assert_eq!(
                    count("Series"),
                    usize::from(visible && !case.ends_with("untitled")),
                    "{name}"
                );
                assert_eq!(
                    count("Color"),
                    0,
                    "{name}: an omitted title must stay omitted"
                );
                let copies = if case == "incompatible" {
                    2
                } else {
                    usize::from(visible)
                };
                assert_eq!(count("Alpha"), copies, "{name}");
                assert_eq!(count("Beta"), copies, "{name}");
                assert_eq!(
                    count("Other"),
                    usize::from(case == "incompatible"),
                    "{name}"
                );
            } else {
                assert!(
                    snapshot.layout().diagnostics().iter().any(|d| {
                        d.code == DiagnosticCode::LayoutPressure
                            && d.message.contains("Legend pressure")
                    }),
                    "{name}: clipping/omission must be reported"
                );
            }
            let blue = rgb(0, 80, 180);
            let red = rgb(200, 20, 40);
            let swatches: Vec<_> = scene
                .items()
                .iter()
                .enumerate()
                .filter_map(|(index, i)| match i.primitive {
                    Primitive::Rectangle { bounds, fill }
                        if i.layer.is_none() && [blue, red].contains(&fill) =>
                    {
                        assert!(
                            snapshot.layout().targets()[index].is_empty(),
                            "{name}: guide acquired a source target"
                        );
                        let clip = i.clip.expect("guide clip");
                        assert!(
                            bounds.origin().x() >= clip.origin().x()
                                && bounds.max_x() <= clip.max_x()
                        );
                        assert!(
                            bounds.origin().y() >= clip.origin().y()
                                && bounds.max_y() <= clip.max_y()
                        );
                        Some(fill)
                    }
                    _ => None,
                })
                .collect();
            if case != "tight" {
                let expected = if case == "incompatible" {
                    vec![blue, red, red, blue]
                } else if visible {
                    vec![blue, red]
                } else {
                    vec![]
                };
                assert_eq!(
                    swatches, expected,
                    "{name}: shared guides deduplicate; incompatible palettes remain separate"
                );
                let points: Vec<_> = scene
                    .items()
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| {
                        i.layer.is_some() && matches!(i.primitive, Primitive::Point { .. })
                    })
                    .collect();
                assert_eq!(
                    points.len(),
                    if matches!(case, "shared" | "incompatible") {
                        4
                    } else {
                        2
                    },
                    "{name}"
                );
                for (index, _) in points {
                    assert!(
                        !snapshot.layout().targets()[index].is_empty(),
                        "{name}: source provenance missing"
                    );
                }
            }
            assert_eq!(scene.items().len(), snapshot.layout().targets().len());
            // Set explicitly when retaining the same tested inputs for host and visual evidence.
            let directory = std::env::var_os("GG_LEGEND_ARTIFACTS").map(std::path::PathBuf::from);
            if let Some(directory) = &directory {
                std::fs::create_dir_all(directory).unwrap();
                std::fs::write(
                    directory.join(format!("{name}.plot.json")),
                    plot.to_json().unwrap(),
                )
                .unwrap();
                std::fs::write(
                    directory.join(format!("{name}.scene.json")),
                    snapshot.scene_json().unwrap(),
                )
                .unwrap();
            }
            for (format, extension) in [
                (Format::Svg, "svg"),
                (Format::Pdf, "pdf"),
                (Format::Png, "png"),
            ] {
                let artifact = snapshot.export(format).unwrap();
                match format {
                    Format::Svg => assert!(
                        std::str::from_utf8(&artifact.bytes)
                            .unwrap()
                            .contains("<svg")
                    ),
                    Format::Pdf => assert!(artifact.bytes.starts_with(b"%PDF-")),
                    Format::Png => {
                        let mut reader = png::Decoder::new(artifact.bytes.as_slice())
                            .read_info()
                            .unwrap();
                        let mut pixels = vec![0; reader.output_buffer_size()];
                        reader.next_frame(&mut pixels).unwrap();
                    }
                    _ => unreachable!("This legend matrix enumerates SVG, PDF and PNG above."),
                }
                if let Some(directory) = &directory {
                    artifact
                        .save(directory.join(format!("{name}.{extension}")))
                        .unwrap();
                }
            }
        }
    }
}
