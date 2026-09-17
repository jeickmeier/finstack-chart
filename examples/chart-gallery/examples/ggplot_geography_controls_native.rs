//! FIX-GG15 native recipe and target proof.
#[path = "../../common/ggplot_geography_controls.rs"]
mod fixtures;
#[path = "common/native_probe.rs"]
mod native_probe;
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
                            .w(px(480.))
                            .h(px(344.))
                            .child(div().h(px(26.)).px(px(10.)).child(name.clone()))
                            .child(div().w(px(480.)).h(px(320.)).child(chart.clone()))
                    })),
            )
            .child(native_probe::end(&charts))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let modes = std::env::var("GG15_MODES")
        .ok()
        .map(|v| {
            v.split(',')
                .map(str::parse::<usize>)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_else(|| vec![0, 1, 2, 3, 4, 5]);
    let plots = modes
        .into_iter()
        .map(|mode| Ok((format!("Geography {mode}"), fixtures::author(mode)?)))
        .collect::<chart_core::ChartResult<Vec<_>>>()?;
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
                    size(px(1440.), px(720.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("GGplot Geography Controls".into()),
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
