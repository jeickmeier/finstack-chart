//! HIR-07: nine hierarchy projections through the native adapter.
#[path = "../../common/hierarchy_fixtures.rs"]
mod fixtures;
#[path = "common/native_probe.rs"]
mod native_probe;
use gpui::{prelude::*, *};
use gpui_charts::{ChartInput, ChartView, NativeFont};
struct Gallery {
    charts: Vec<Entity<ChartView>>,
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0xffffff))
            .child(native_probe::begin())
            .children(self.charts.chunks(3).map(|row| {
                div().flex().children(
                    row.iter()
                        .map(|chart| div().w(px(380.)).h(px(270.)).child(chart.clone())),
                )
            }))
            .child(native_probe::end(&self.charts))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let figures = fixtures::figures()?;
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    gpui_platform::application().run(move |cx| {
        let font = NativeFont::from_bytes(bytes, "Noto Sans", cx).expect("supplied font");
        let inputs: Vec<_> = figures
            .iter()
            .map(|(_, p)| ChartInput::from_plot(p, font.clone()).expect("shared hierarchy"))
            .collect();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1140.), px(820.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Hierarchy Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Gallery {
                    charts: inputs
                        .into_iter()
                        .map(|input| cx.new(|cx| ChartView::new(input, cx)))
                        .collect(),
                })
            },
        )
        .expect("native window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
    Ok(())
}
