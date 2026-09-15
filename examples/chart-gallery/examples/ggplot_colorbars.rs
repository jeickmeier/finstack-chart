//! FIX-GG05: actual native complete sampled colorbars and facet collection.
#[path = "../../common/ggplot_colorsteps_boundary_fixtures.rs"]
mod boundary_fixtures;
#[path = "../../common/ggplot_colorbar_fixtures.rs"]
mod fixtures;
#[path = "common/native_probe.rs"]
mod native_probe;
#[path = "../../common/ggplot_colorsteps_fixtures.rs"]
mod steps_fixtures;
use gpui::{prelude::*, *};
use gpui_charts::{ChartInput, ChartView, NativeFont};
struct Gallery {
    charts: Vec<(String, Entity<ChartView>)>,
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let charts = self
            .charts
            .iter()
            .map(|(_, chart)| chart.clone())
            .collect::<Vec<_>>();
        div()
            .size_full()
            .bg(rgb(0xffffff))
            .child(native_probe::begin())
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .children(self.charts.iter().map(|(name, chart)| {
                        div()
                            .w(px(600.))
                            .h(px(386.))
                            .child(div().h(px(26.)).px(px(10.)).child(name.clone()))
                            .child(div().w(px(600.)).h(px(360.)).child(chart.clone()))
                    })),
            )
            .child(native_probe::end(&charts))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sampling = std::env::args().any(|arg| arg == "--sampling");
    let presentation = std::env::args().any(|arg| arg == "--presentation");
    let display = std::env::args().any(|arg| arg == "--display");
    let alpha = std::env::args().any(|arg| arg == "--alpha");
    let boundaries = std::env::args().any(|arg| arg == "--steps-boundaries");
    let step_controls = std::env::args().any(|arg| arg == "--steps-controls");
    let steps = step_controls || boundaries || std::env::args().any(|arg| arg == "--steps");
    let cases = if steps {
        vec![
            ("asymmetric", "fill", "single", Some(3.)),
            ("asymmetric", "color", "collected", Some(3.)),
            ("asymmetric", "stroke", "local", Some(3.)),
            ("asymmetric", "fill", "single", Some(3.)),
        ]
    } else if alpha {
        vec![
            ("transparent", "fill", "single", Some(5.)),
            ("transparent", "color", "collected", Some(5.)),
            ("transparent", "stroke", "local", Some(5.)),
            ("asymmetric", "fill", "single", Some(5.)),
        ]
    } else if display {
        vec![
            ("asymmetric", "fill", "single", Some(15.)),
            ("discontinuous", "color", "collected", Some(5.)),
            ("asymmetric", "stroke", "local", Some(2.5)),
            ("ordinary", "fill", "single", Some(300.)),
        ]
    } else if presentation {
        vec![
            ("asymmetric", "fill", "single", Some(5.)),
            ("discontinuous", "color", "collected", Some(5.)),
            ("ordinary", "stroke", "local", Some(5.)),
            ("asymmetric", "fill", "single", Some(5.)),
        ]
    } else if sampling {
        vec![
            ("asymmetric", "fill", "single", Some(0.)),
            ("discontinuous", "color", "collected", Some(1.)),
            ("ordinary", "stroke", "local", Some(2.5)),
            ("asymmetric", "fill", "single", Some(5.)),
        ]
    } else {
        vec![
            ("asymmetric", "fill", "single", None),
            ("discontinuous", "color", "collected", None),
            ("ordinary", "stroke", "local", None),
        ]
    };
    let plots = cases
        .into_iter()
        .enumerate()
        .map(|(index, (palette, channel, facet, nbin))| {
            let plot = fixtures::author(palette, channel, facet, false);
            let plot = if let Some(n) = nbin {
                use chart_core::{
                    interpolate::Number,
                    scales::{
                        GgplotColorbarOptions, GgplotContinuousGuide, GgplotGuideLabels,
                        GgplotScaleGuide,
                    },
                    scene::GradientDirection,
                };
                let mut scale = fixtures::scale(palette, false);
                let mut options = GgplotColorbarOptions {
                    nbin: Some(Number(n)),
                    ..Default::default()
                };
                if steps {
                    scale = steps_fixtures::steps_scale(
                        if index % 2 == 0 {
                            "continuous"
                        } else {
                            "binned"
                        },
                        index >= 2,
                        false,
                    );
                    if index == 3 {
                        scale = scale
                            .with_guide(GgplotScaleGuide::Binned(GgplotGuideLabels::Automatic))
                            .expect("default binned guide");
                    }
                    options.nbin = None;
                    options.direction = Some(if index % 2 == 0 {
                        GradientDirection::Horizontal
                    } else {
                        GradientDirection::Vertical
                    });
                    options.reverse = index == 2;
                    if step_controls {
                        options.even_steps = index == 1;
                        options.show_limits = index != 0;
                    }
                }
                if boundaries {
                    let source: serde_json::Value = serde_json::from_str(include_str!(
                        "../../../fixtures/parity/ggplot2/colorsteps-boundaries.json"
                    ))
                    .expect("boundary source");
                    let mode = ["duplicates", "nonfinite", "unsorted", "endpoint_first"][index];
                    let family = if index % 2 == 0 {
                        "continuous"
                    } else {
                        "binned"
                    };
                    let case = source["cases"]
                        .as_array()
                        .expect("cases")
                        .iter()
                        .find(|c| {
                            c["family"] == family
                                && c["mode"] == mode
                                && c["population"] == "ordinary"
                        })
                        .expect("case");
                    scale = boundary_fixtures::boundary_scale(case).expect("boundary scale");
                }
                if alpha {
                    options.alpha =
                        [None, Some(Number(0.5)), Some(Number(1.)), Some(Number(0.))][index];
                    options.display = match index {
                        0 | 3 => chart_core::scales::GgplotColorbarDisplay::Raster,
                        1 => chart_core::scales::GgplotColorbarDisplay::Gradient,
                        _ => chart_core::scales::GgplotColorbarDisplay::Rectangles,
                    };
                    options.direction = Some(if index % 2 == 0 {
                        GradientDirection::Horizontal
                    } else {
                        GradientDirection::Vertical
                    });
                    options.reverse = index == 1 || index == 2;
                }
                if display {
                    options.display = if index < 2 {
                        chart_core::scales::GgplotColorbarDisplay::Gradient
                    } else {
                        chart_core::scales::GgplotColorbarDisplay::Rectangles
                    };
                    options.direction = Some(if index % 2 == 0 {
                        GradientDirection::Horizontal
                    } else {
                        GradientDirection::Vertical
                    });
                    options.reverse = index == 1 || index == 2;
                }
                if presentation {
                    options.direction = Some(if index == 1 {
                        GradientDirection::Vertical
                    } else {
                        GradientDirection::Horizontal
                    });
                    options.reverse = matches!(index, 1 | 2);
                    options.draw_lower_limit = index != 3;
                    options.draw_upper_limit = index != 3;
                    if index == 2 {
                        scale = scale
                            .with_guide(GgplotScaleGuide::Colorbar(GgplotContinuousGuide {
                                breaks: Some([-2., 0., 3., 8.].map(Number).to_vec()),
                                labels: GgplotGuideLabels::Hidden,
                                ..Default::default()
                            }))
                            .expect("hidden colorbar labels");
                    }
                }
                plot.edit()
                    .scale(chart_core::prelude::color_mapped(
                        "v",
                        scale.with_colorbar_options(options),
                    ))
                    .build()
                    .expect("sampled colorbar plot")
            } else {
                plot
            };
            (
                format!(
                    "{palette} / {channel} / {facet}{}",
                    if boundaries {
                        [
                            " / continuous duplicate cuts H",
                            " / binned infinite edge cells V",
                            " / continuous unsorted cuts H reversed",
                            " / binned upper endpoint first V",
                        ][index]
                            .into()
                    } else if steps {
                        [
                            " / continuous steps H",
                            " / binned steps V",
                            " / continuous steps H reversed, endpoint keys",
                            " / default binned steps V, endpoint keys",
                        ][index]
                            .into()
                    } else if alpha {
                        [
                            " / raster H, palette alpha",
                            " / gradient V reversed, alpha=.5",
                            " / rectangles H reversed, alpha=1",
                            " / raster V, alpha=0",
                        ][index]
                            .into()
                    } else if display {
                        match index {
                            0 => " / gradient H".into(),
                            1 => " / gradient V reversed".into(),
                            2 => " / rectangles H reversed, nbin=2.5".into(),
                            _ => " / rectangles V, nbin=300".into(),
                        }
                    } else if presentation {
                        match index {
                            0 => " / H".into(),
                            1 => " / V reversed".into(),
                            2 => " / H reversed, labels hidden".into(),
                            _ => " / H, end ticks hidden".into(),
                        }
                    } else {
                        nbin.map_or(String::new(), |n| format!(" / nbin={n}"))
                    }
                ),
                plot,
            )
        })
        .collect::<Vec<_>>();
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    gpui_platform::application().run(move |cx| {
        let font = NativeFont::from_bytes(bytes, "Noto Sans", cx).expect("supplied font");
        let inputs = plots
            .iter()
            .map(|(name, plot)| {
                (
                    name.to_owned(),
                    ChartInput::from_plot(plot, font.clone()).expect("scale input"),
                )
            })
            .collect::<Vec<_>>();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.), px(800.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("GGplot Colorbar Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Gallery {
                    charts: inputs
                        .into_iter()
                        .map(|(name, input)| (name, cx.new(|cx| ChartView::new(input, cx))))
                        .collect(),
                })
            },
        )
        .expect("native color window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
    Ok(())
}
