//! FIX-S01 native sector, hole and custom sink using the accepted path foundation.
#[path = "../../common/shape_foundation_fixtures.rs"]
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
            .child(div().w(px(540.)).h(px(280.)).child(self.chart.clone()))
            .child(native_probe::end(std::slice::from_ref(&self.chart)))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let plot = fixtures::figure()?;
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    gpui_platform::application().run(move |cx| {
        let font = NativeFont::from_bytes(bytes, "Noto Sans", cx).expect("supplied font");
        let input = ChartInput::from_plot(&plot, font).expect("primary retained path");
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(540.), px(305.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Shape Foundation Proof".into()),
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
