//! Primary component gallery; shares explicit builder recipes with the host proofs.
#[path = "../../common/authoring_fixtures.rs"]
mod authoring_fixtures;
use chart_core::prelude::Plot;
#[cfg(feature = "kit")]
use chart_core::prelude::theme;
use gpui::{
    Bounds, Context, Entity, IntoElement, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_charts::{ChartInput, ChartView, NativeFont};

struct Case {
    name: String,
    plot: Plot,
}
struct Gallery {
    cases: Vec<Case>,
    font: NativeFont,
    selected: usize,
    chart: Entity<ChartView>,
}
impl Gallery {
    fn mount(case: &Case, font: &NativeFont, cx: &mut Context<Self>) -> Entity<ChartView> {
        #[cfg(feature = "kit")]
        let input = if case.name == "composition-kit-host" {
            use gpui_kit::component::ActiveTheme;
            gpui_charts_kit::chart_input(&case.plot, font.clone(), cx.theme()).expect("Kit input")
        } else {
            ChartInput::from_plot(&case.plot, font.clone()).expect("native input")
        };
        #[cfg(not(feature = "kit"))]
        let input = ChartInput::from_plot(&case.plot, font.clone()).expect("native input");
        let chart = cx.new(|cx| ChartView::new(input, cx));
        #[cfg(feature = "kit")]
        if case.name == "composition-kit-host" {
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
    let facets = std::env::args().any(|arg| arg == "--facets")
        || std::env::current_exe()?
            .file_stem()
            .is_some_and(|name| name == "facet_gallery");
    let selected = std::env::args()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    gpui_platform::application().run(move |cx| {
        #[cfg(feature = "kit")]
        gpui_kit::init(cx);
        let mut font = NativeFont::from_bytes(
            include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf").as_slice(),
            "Noto Sans",
            cx,
        )
        .expect("fixture font");
        let plots = if composition {
            let bold = NativeFont::from_bytes(
                include_bytes!("../../../fixtures/composition/fonts/NotoSans-Bold.ttf").as_slice(),
                "Noto Sans",
                cx,
            )
            .expect("bold face");
            let arabic = NativeFont::from_bytes(
                include_bytes!("../../../fixtures/composition/fonts/NotoSansArabic-Regular.ttf")
                    .as_slice(),
                "Noto Sans Arabic",
                cx,
            )
            .expect("Arabic face");
            font = font
                .with_face(&bold)
                .expect("bold bank")
                .with_face(&arabic)
                .expect("Arabic bank");
            authoring_fixtures::composition_cases(bold.descriptor(), arabic.descriptor())
        } else {
            authoring_fixtures::cases()
                .into_iter()
                .filter(|(name, _)| name.starts_with(if facets { "facet-" } else { "family-" }))
                .collect()
        };
        let cases: Vec<Case> = plots
            .into_iter()
            .map(|(name, plot)| Case { name, plot })
            .collect();
        #[cfg(feature = "kit")]
        let cases = {
            let mut cases = cases;
            if composition {
                cases.push(Case {
                    name: "composition-kit-host".into(),
                    plot: cases[0]
                        .plot
                        .edit()
                        .theme(theme())
                        .build()
                        .expect("inherited Kit theme"),
                });
            }
            cases
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
