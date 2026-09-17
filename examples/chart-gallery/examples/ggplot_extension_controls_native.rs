//! FIX-GG16 native recipe and target proof.
#[path = "../../common/ggplot_extension_controls.rs"]
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
        let cell_width = if self.charts.len() <= 2 { 720. } else { 360. };
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
                            .w(px(cell_width))
                            .h(px(344.))
                            .child(div().h(px(26.)).px(px(10.)).child(name.clone()))
                            .child(div().w(px(cell_width)).h(px(320.)).child(chart.clone()))
                    })),
            )
            .child(native_probe::end(&charts))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = std::env::args()
        .nth(1)
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);
    let plots = (start..(start + 8).min(fixtures::CASES))
        .map(|mode| Ok((format!("Extension {mode}"), fixtures::author(mode)?)))
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
                    title: Some("GGplot Extension Controls".into()),
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
