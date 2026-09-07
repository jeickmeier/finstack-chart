//! Required family gallery consuming the identical portable fixture catalog as host proofs.
use chart_core::portable::{ChartEnvelope, DataEnvelope, Session, decode, encode};
use chart_core::services::{ResourceDescriptor, ResourceKind};
use chart_core::{ResourceId, Revision};
use gpui::{
    Bounds, Context, Entity, IntoElement, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_charts::{ChartInput, ChartView, NativeFont};
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct Case {
    name: String,
    chart: ChartEnvelope,
    data: DataEnvelope,
}
struct Gallery {
    cases: Vec<Case>,
    font: NativeFont,
    selected: usize,
    chart: Entity<ChartView>,
}
impl Gallery {
    fn mount(case: &Case, font: &NativeFont, cx: &mut Context<Self>) -> Entity<ChartView> {
        let session = Session::new(
            &encode(&case.chart).expect("fixture"),
            &encode(&case.data).expect("fixture"),
        )
        .expect("valid family fixture");
        let input = ChartInput::new(session.definition().clone(), session.source(), font.clone())
            .expect("native input");
        cx.new(|cx| ChartView::new(input, cx))
    }
    fn new(cases: Vec<Case>, font: NativeFont, selected: usize, cx: &mut Context<Self>) -> Self {
        let selected = selected.min(cases.len() - 1);
        let chart = Self::mount(&cases[selected], &font, cx);
        Self {
            cases,
            font,
            selected,
            chart,
        }
    }
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut buttons = div().flex().flex_wrap().gap_2();
        for (index, case) in self.cases.iter().enumerate() {
            buttons = buttons.child(
                div()
                    .id(index)
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .cursor_pointer()
                    .bg(rgb(if index == self.selected {
                        0xd4e8f3
                    } else {
                        0xffffff
                    }))
                    .child(
                        case.name
                            .trim_start_matches("family-")
                            .trim_start_matches("facet-")
                            .to_owned(),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected = index;
                        this.chart = Self::mount(&this.cases[index], &this.font, cx);
                        eprintln!("family selected: {}", this.cases[index].name);
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
                div().text_xl().child(
                    if self
                        .cases
                        .first()
                        .is_some_and(|c| c.name.starts_with("facet-"))
                    {
                        "Facets and shared layout"
                    } else {
                        "Chart families"
                    },
                ),
            )
            .child(
                div().child(
                    "Portable fixtures · Native vector rendering · Shared scales and semantics",
                ),
            )
            .child(buttons)
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<Case> = if std::env::args().any(|arg| arg == "--facets")
        || std::env::current_exe()?
            .file_stem()
            .is_some_and(|name| name == "facet_gallery")
    {
        decode(include_str!("../../../fixtures/facets/portable-cases.json"))?
    } else {
        decode(include_str!(
            "../../../fixtures/families/portable-cases.json"
        ))?
    };
    let selected = std::env::args()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    gpui_platform::application().run(move |cx| {
        let bytes: Arc<[u8]> = Arc::from(
            include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
        );
        let font = NativeFont::load(
            ResourceDescriptor {
                id: ResourceId::new(0),
                revision: Revision::INITIAL,
                kind: ResourceKind::Font,
                byte_len: bytes.len() as u64,
            },
            bytes,
            "Noto Sans",
            cx,
        )
        .expect("fixture font");
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1050.), px(700.)),
                    cx,
                ))),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Finstack Chart Families".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| Gallery::new(cases, font, selected, cx)),
        )
        .expect("native window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
    Ok(())
}
