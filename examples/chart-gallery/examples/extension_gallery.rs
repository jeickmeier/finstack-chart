//! FIX-17 custom stat/geom and explicit host-native painter, using only public APIs.
use chart_core::plot::{Data, column, x_axis};
use chart_core::{
    grammar::{OperationRef, SemanticValue},
    *,
};
use chart_extension_example as fixture;
use gpui::{prelude::*, size, *};
use gpui_charts::*;
use std::rc::Rc;
struct NativeBar;
struct PaintedBar(PaintQuad);
impl PreparedNativePaint for PaintedBar {
    fn paint(&self, w: &mut Window, _: &mut App) {
        w.paint_quad(self.0.clone());
    }
}
impl NativePainter for NativeBar {
    fn operation(&self) -> OperationRef {
        OperationRef::new(fixture::PAINTER, Revision::new(1))
    }
    fn prepare(
        &self,
        b: Bounds<Pixels>,
        p: &serde_json::Value,
        c: chart_core::scene::Color,
    ) -> ChartResult<Rc<dyn PreparedNativePaint>> {
        let radius = p["radius"]
            .as_f64()
            .filter(|v| v.is_finite() && *v >= 0. && *v <= 32.)
            .ok_or_else(|| {
                Diagnostic::error(
                    DiagnosticCode::Validation,
                    "Native radius must be 0..32 logical pixels.",
                    "Use valid native painter parameters.",
                )
            })?;
        let color = rgba(
            (u32::from(c.red) << 24)
                | (u32::from(c.green) << 16)
                | (u32::from(c.blue) << 8)
                | u32::from(c.alpha),
        );
        Ok(Rc::new(PaintedBar(
            fill(
                b,
                gpui::linear_gradient(
                    180.,
                    gpui::linear_color_stop(color, 0.),
                    gpui::linear_color_stop(rgb(0x8d5dd9), 1.),
                )
                .color_space(ColorSpace::Srgb),
            )
            .corner_radii(px(radius as f32)),
        )))
    }
}
struct Gallery {
    font: NativeFont,
    painters: Rc<NativePainterRegistry>,
    chart: Entity<ChartView>,
    native: bool,
}
impl Gallery {
    fn mount(
        native: bool,
        font: &NativeFont,
        painters: Rc<NativePainterRegistry>,
        cx: &mut Context<Self>,
    ) -> Entity<ChartView> {
        let data = Data::columns()
            .column(
                "x",
                column(vec![0., 0.25, 0.75, 1., 1.5, 2., 999.])
                    .validity(vec![true, true, true, true, true, true, false]),
            )
            .keys(9007199254743001..=9007199254743007)
            .build()
            .expect("data");
        let plot = fixture::authoring::density_plot(data, "x", vec![0., 1., 2.], native)
            .expect("extension plot")
            .edit()
            .x_axis(x_axis().ticks(vec![
                (0.0.into(), "Lower".into()),
                (1.0.into(), "Boundary".into()),
                (2.0.into(), "Upper".into()),
            ]))
            .build()
            .expect("guide");
        let input = ChartInput::from_plot(&plot, font.clone())
            .expect("native extension")
            .with_native_painters(painters);
        cx.new(|cx| {
            let mut chart = ChartView::new(input, cx);
            chart.set_tooltip(
                Rc::new(|i, _, _| {
                    let values = i
                        .hits()
                        .iter()
                        .flat_map(|h| h.values.iter())
                        .map(|(k, v)| {
                            let value = match v {
                                SemanticValue::Number(v) => v.to_string(),
                                SemanticValue::Text(v) => v.clone(),
                                SemanticValue::Signed(v) => v.to_string(),
                                SemanticValue::Unsigned(v) => v.to_string(),
                            };
                            format!("{k}: {value}")
                        })
                        .collect::<Vec<_>>()
                        .join(" · ");
                    div()
                        .p_2()
                        .bg(rgb(0xeeeeff))
                        .text_color(rgb(0x202040))
                        .child(values)
                        .into_any_element()
                }),
                cx,
            );
            chart
        })
    }
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut controls = div().flex().gap_3();
        for (native, label) in [
            (false, "Portable custom geometry"),
            (true, "Native painter · export unsupported"),
        ] {
            controls = controls.child(
                div()
                    .id(if native { "native" } else { "portable" })
                    .p_3()
                    .rounded_md()
                    .bg(rgb(if self.native == native {
                        0xd8e9ff
                    } else {
                        0xffffff
                    }))
                    .cursor_pointer()
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.native = native;
                        this.chart = Self::mount(native, &this.font, this.painters.clone(), cx);
                        cx.notify();
                    })),
            );
        }
        div()
            .size_full()
            .flex()
            .flex_col()
            .p_5()
            .gap_3()
            .bg(rgb(0xf4f7fa))
            .text_color(rgb(0x203b4c))
            .font_family("Noto Sans")
            .child(
                div()
                    .text_xl()
                    .child("Custom density histogram · public extension API"),
            )
            .child("Counts 3 + 3 · densities 0.5 · custom guides, hit regions and keyboard order")
            .child(controls)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .bg(rgb(0xffffff))
                    .child(self.chart.clone()),
            )
    }
}
fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        let font = NativeFont::from_bytes(
            include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
            "Noto Sans",
            cx,
        )
        .expect("supplied font");
        let mut painters = NativePainterRegistry::new();
        painters
            .register(Rc::new(NativeBar))
            .expect("native registry");
        let painters = Rc::new(painters);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1000.), px(670.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Finstack Extension Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    let chart = Gallery::mount(false, &font, painters.clone(), cx);
                    Gallery {
                        font,
                        painters,
                        chart,
                        native: false,
                    }
                })
            },
        )
        .expect("window");
        cx.activate(true);
    });
}
