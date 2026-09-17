//! FIX-GG14 native recipe and target proof.
#[path = "../../common/ggplot_math_controls.rs"]
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
    let modes = std::env::var("GG14_MATH_MODES")
        .unwrap_or_else(|_| "0,1,2,3,4,5".into())
        .split(',')
        .map(str::parse::<usize>)
        .collect::<Result<Vec<_>, _>>()?;
    if modes.iter().any(|mode| *mode >= fixtures::CASES) {
        return Err("Invalid math gallery mode".into());
    }
    gpui_platform::application().run(move |cx| {
        let mut font = NativeFont::from_bytes(
            include_bytes!("../../../fixtures/math/fonts/DejaVuSerif.ttf").as_slice(),
            "DejaVu Serif",
            cx,
        )
        .expect("regular face");
        let italic = NativeFont::from_bytes(
            include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Italic.ttf").as_slice(),
            "DejaVu Serif",
            cx,
        )
        .expect("italic face");
        let bold = NativeFont::from_bytes(
            include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-Bold.ttf").as_slice(),
            "DejaVu Serif",
            cx,
        )
        .expect("bold face");
        let bold_italic = NativeFont::from_bytes(
            include_bytes!("../../../fixtures/math/fonts/DejaVuSerif-BoldItalic.ttf").as_slice(),
            "DejaVu Serif",
            cx,
        )
        .expect("bold italic face");
        let symbol = NativeFont::from_bytes(
            include_bytes!("../../../fixtures/math/fonts/DejaVuMathTeXGyre.ttf").as_slice(),
            "DejaVu Math TeX Gyre",
            cx,
        )
        .expect("symbol face");
        let fallback = NativeFont::from_bytes(
            include_bytes!("../../../fixtures/math/fonts/DejaVuSans.ttf").as_slice(),
            "DejaVu Sans",
            cx,
        )
        .expect("fallback face");
        let fonts = chart_core::typography::MathFonts {
            regular: Some(font.descriptor()),
            italic: Some(italic.descriptor()),
            bold: Some(bold.descriptor()),
            bold_italic: Some(bold_italic.descriptor()),
            symbol: Some(symbol.descriptor()),
        };
        for face in [&italic, &bold, &bold_italic, &symbol, &fallback] {
            font = font.with_face(face).expect("explicit font bank");
        }
        let plots = modes
            .iter()
            .copied()
            .map(|mode| {
                (
                    format!("Math {mode}"),
                    fixtures::author(mode, fonts.clone(), fallback.descriptor())
                        .expect("math author"),
                )
            })
            .collect::<Vec<_>>();
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
                    title: Some("GGplot Math Controls".into()),
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
