//! AXIS-05 native independent tick selection and formatter qualification.
#[path = "../../common/axis_component_fixtures.rs"]
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
            .child(div().w(px(680.)).h(px(420.)).child(self.charts[0].clone()))
            .child(div().w(px(680.)).h(px(270.)).child(self.charts[1].clone()))
            .child(native_probe::end(&self.charts))
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let plot = fixtures::figure()?;
    let typography = fixtures::typography()?;
    let bytes =
        include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice();
    gpui_platform::application().run(move |cx| {
        let font = NativeFont::from_bytes(bytes, "Noto Sans", cx).expect("supplied font");
        let input = ChartInput::from_plot(&plot, font.clone()).expect("shared positional guides");
        let second = ChartInput::from_plot(&typography, font).expect("shared shaped labels");
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(680.), px(720.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Axis Component Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Gallery {
                    charts: vec![
                        cx.new(|cx| ChartView::new(input, cx)),
                        cx.new(|cx| ChartView::new(second, cx)),
                    ],
                })
            },
        )
        .expect("native window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
    Ok(())
}
