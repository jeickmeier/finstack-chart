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
        let chart = cx.new(|cx| ChartView::new(input, cx));
        #[cfg(feature = "kit")]
        if case.name == "composition-kit-host" {
            use gpui_kit::component::ActiveTheme;
            let theme = cx.theme().clone();
            chart
                .update(cx, |view, cx| {
                    gpui_charts_kit::apply_theme(view, &theme, cx)
                })
                .expect("Kit host tokens");
            // A bounded initial view makes the Kit reset control observable in this example.
            chart
                .update(cx, |view, cx| {
                    view.dispatch_chart(
                        chart_core::state::ChartAction::SetViewport(chart_core::state::Viewport {
                            x: Some((0., 1.)),
                            y: None,
                        }),
                        cx,
                    )
                })
                .expect("example viewport");
        }
        chart
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
                            .trim_start_matches("composition-")
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
        #[cfg(feature = "kit")]
        if self
            .cases
            .first()
            .is_some_and(|c| c.name.starts_with("composition-"))
        {
            buttons = buttons.child(gpui_charts_kit::reset_button(self.chart.clone()));
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
                    } else if self
                        .cases
                        .first()
                        .is_some_and(|c| c.name.starts_with("composition-"))
                    {
                        "Themes and publication composition"
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
    let composition = std::env::args().any(|arg| arg == "--composition")
        || std::env::current_exe()?
            .file_stem()
            .is_some_and(|name| name == "composition_gallery");
    let cases: Vec<Case> = if composition {
        decode(include_str!(
            "../../../fixtures/composition/portable-cases.json"
        ))?
    } else if std::env::args().any(|arg| arg == "--facets")
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
    #[cfg(feature = "kit")]
    let cases = {
        let mut cases = cases;
        if composition {
            let mut case: Case = decode(include_str!(
                "../../../fixtures/composition/portable-cases.json"
            ))
            .map(|mut v: Vec<Case>| v.remove(0))?;
            case.name = "composition-kit-host".into();
            case.chart.definition.theme.as_mut().expect("theme").named = None;
            cases.push(case);
        }
        cases
    };
    let selected = std::env::args()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    gpui_platform::application().run(move |cx| {
        #[cfg(feature = "kit")]
        gpui_kit::init(cx);
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
        let font = if composition {
            let mut font = font;
            for (id, bytes, family) in [
                (
                    9007199254747002_u64,
                    include_bytes!("../../../fixtures/composition/fonts/NotoSans-Bold.ttf")
                        .as_slice(),
                    "Noto Sans",
                ),
                (
                    9007199254747003_u64,
                    include_bytes!(
                        "../../../fixtures/composition/fonts/NotoSansArabic-Regular.ttf"
                    )
                    .as_slice(),
                    "Noto Sans Arabic",
                ),
            ] {
                let face = NativeFont::load(
                    ResourceDescriptor {
                        id: ResourceId::new(id),
                        revision: Revision::new(1),
                        kind: ResourceKind::Font,
                        byte_len: bytes.len() as u64,
                    },
                    Arc::from(bytes),
                    family,
                    cx,
                )
                .expect("rich fixture face");
                font = font.with_face(&face).expect("rich face bank");
            }
            font
        } else {
            font
        };
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
            |window, cx| {
                let view = cx.new(|cx| Gallery::new(cases, font, selected, cx));
                #[cfg(feature = "kit")]
                let view = cx.new(|cx| gpui_kit::component::Root::new(view, window, cx));
                #[cfg(not(feature = "kit"))]
                let _ = window;
                view
            },
        )
        .expect("native window");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
    Ok(())
}
