//! FIX-S06 native area and stroked symbols using the shared chart engine.
#[path = "../../common/shape_symbol_fixtures.rs"]
mod fixtures;
#[path = "common/native_probe.rs"]
mod native_probe;
use gpui::{prelude::*, *};
use gpui_charts::{ChartInput, ChartView, NativeFont};
struct Gallery {
    chart: Entity<ChartView>,
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xffffff))
            .child(native_probe::begin())
            .child(div().w(px(600.)).h(px(740.)).child(self.chart.clone()))
            .child(native_probe::end(std::slice::from_ref(&self.chart)))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let preset = match std::env::args().nth(1).as_deref() {
        Some("Terminal") => chart_core::theme::NamedTheme::Terminal,
        Some("Grayscale") => chart_core::theme::NamedTheme::Grayscale,
        _ => chart_core::theme::NamedTheme::Editorial,
    };
    let plot = fixtures::figure(preset)?;
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    gpui_platform::application().run(move |cx| {
        let font = NativeFont::from_bytes(bytes, "Noto Sans", cx).expect("supplied font");
        let input = ChartInput::from_plot(&plot, font).expect("primary retained path");
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(600.), px(765.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Symbol Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Gallery {
                    chart: cx.new(|cx| ChartView::new(input, cx)),
                })
            },
        )
        .expect("native window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
    Ok(())
}
