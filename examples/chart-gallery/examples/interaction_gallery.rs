//! WP-16 actual native pointer/keyboard producers over the shared presented-scene engine.
use chart_core::{portable::Session, services::*, state::*, *};
use gpui::{prelude::*, *};
use gpui_charts::*;
use std::sync::Arc;
struct Gallery {
    chart: Entity<ChartView>,
    font: NativeFont,
    case: usize,
    tool: NativeDragTool,
    status: String,
    _observe: Subscription,
}
fn input(case: usize, font: NativeFont) -> ChartInput {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/interaction/cases.json"))
            .expect("cases");
    let c = &cases[case];
    let session = Session::new(&c["chart"].to_string(), &c["data"].to_string()).expect("session");
    let mut definition = session.definition().clone();
    for axis in &mut definition.axes {
        axis.visible = true;
    }
    ChartInput::new(definition, session.source(), font).expect("input")
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chart = self.chart.read(cx);
        let state = chart.state();
        let selection = match state.active_gesture().and_then(|g| g.preview.as_ref()) {
            Some(GesturePreview::Selection(v)) => v.len(),
            _ => state.selection().len(),
        };
        let windows = state
            .axis_windows()
            .iter()
            .map(|(id, w)| {
                format!(
                    "{}: {}",
                    id.get(),
                    match w {
                        AxisWindow::Numeric(a, b) => format!("{a:.2}..{b:.2}"),
                        AxisWindow::Timestamp(a, b) => format!("{a}..{b}"),
                        AxisWindow::Category { first, last } => format!("{first}..{last}"),
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let detail = format!(
            "Selected: {selection} · Gesture: {} · Windows: {} · Layouts: {} · State: {}",
            state.active_gesture().is_some(),
            windows,
            chart.metrics().layout_attempts,
            state.revision().get()
        );
        let mut controls = div().flex().gap_2();
        for (label, tool) in [
            ("Pan", NativeDragTool::Pan),
            ("Rectangle", NativeDragTool::Rectangle),
            ("Lasso", NativeDragTool::Lasso),
            ("X range", NativeDragTool::XRange),
            ("Y range", NativeDragTool::YRange),
            ("Region zoom", NativeDragTool::ZoomRegion),
        ] {
            controls = controls.child(
                div()
                    .id(label)
                    .role(Role::Button)
                    .px_3()
                    .py_2()
                    .bg(rgb(if self.tool == tool {
                        0xbed6f4
                    } else {
                        0xffffff
                    }))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let result = this
                            .chart
                            .update(cx, |chart, cx| chart.set_drag_tool(tool, cx));
                        this.tool = tool;
                        this.status = result
                            .err()
                            .map_or_else(|| format!("Tool: {tool:?}"), |e| e.message);
                        cx.notify();
                    })),
            );
        }
        controls = controls
            .child(
                div()
                    .id("reset")
                    .role(Role::Button)
                    .px_3()
                    .py_2()
                    .bg(rgb(0xffffff))
                    .child("Reset")
                    .on_click(cx.listener(|this, _, _, cx| {
                        let result = this
                            .chart
                            .update(cx, |chart, cx| chart.dispatch_chart(ChartAction::Reset, cx));
                        this.status = result.err().map_or_else(|| "Reset".into(), |e| e.message);
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .id("blur")
                    .role(Role::Button)
                    .px_3()
                    .py_2()
                    .bg(rgb(0xffffff))
                    .child("Lose focus")
                    .on_click(|_, window, cx| window.blur(cx)),
            );
        controls = controls.child(
            div()
                .id("preview-pan")
                .role(Role::Button)
                .px_3()
                .py_2()
                .bg(rgb(0xffffff))
                .child("Preview pan")
                .on_click(cx.listener(|this, _, window, cx| {
                    let focus = this.chart.read(cx).focus_handle(cx);
                    window.focus(&focus, cx);
                    let result = this.chart.update(cx, |chart, cx| -> ChartResult<()> {
                        use chart_core::navigation::{Navigation, NavigationBoundary, Navigator};
                        let Some(inspector) = chart.inspector() else {
                            return Ok(());
                        };
                        let scene = inspector.presented().clone();
                        let windows = Navigator::new(scene.clone()).navigate(
                            scene.scene().stamp(),
                            &[ScaleId::new(0)],
                            None,
                            Navigation::Pan { dx: 60., dy: 0. },
                            NavigationBoundary::Extend,
                        )?;
                        let id = chart.next_gesture_id()?;
                        chart.dispatch_chart(
                            ChartAction::BeginGesture {
                                id,
                                kind: GestureKind::Viewport,
                            },
                            cx,
                        )?;
                        chart.dispatch_chart(
                            ChartAction::PreviewGesture {
                                id,
                                preview: GesturePreview::AxisWindows(windows),
                            },
                            cx,
                        )?;
                        Ok(())
                    });
                    this.status = result.err().map_or_else(
                        || "Preview active: Escape, Lose focus or switch apps to cancel".into(),
                        |e| e.message,
                    );
                    cx.notify();
                })),
        );
        let mut cases = div().flex().gap_2();
        for (index, label) in [
            (0, "Scatter"),
            (1, "Bars"),
            (2, "Line gaps"),
            (3, "Categories"),
            (4, "UTC"),
        ] {
            cases = cases.child(
                div()
                    .id(label)
                    .role(Role::Button)
                    .px_3()
                    .py_2()
                    .bg(rgb(if self.case == index {
                        0xbed6f4
                    } else {
                        0xffffff
                    }))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.chart =
                            cx.new(|cx| ChartView::new(input(index, this.font.clone()), cx));
                        this.case = index;
                        this.tool = NativeDragTool::Pan;
                        this._observe = cx.observe(&this.chart, |_, _, cx| cx.notify());
                        this.status = label.into();
                        cx.notify();
                    })),
            );
        }
        div().size_full().flex().flex_col().p_4().gap_3().bg(rgb(0xecf1f8)).text_color(rgb(0x202b3c)).font_family("Noto Sans")
            .child(div().text_xl().child("Presented-scene interaction"))
            .child("Click to focus/select · wheel zooms at pointer · drag uses chosen tool · Shift adds · Command toggles · arrows inspect · Space selects · Escape cancels · Home resets")
            .child(cases).child(controls).child(detail).child(self.status.clone())
            .child(div().flex_1().min_h_0().w_full().bg(rgb(0xffffff)).child(self.chart.clone()))
    }
}
fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        let bytes: Arc<[u8]> =
            include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf")
                .as_slice()
                .into();
        let font = NativeFont::load(
            ResourceDescriptor {
                id: ResourceId::new(1),
                revision: Revision::new(1),
                kind: ResourceKind::Font,
                byte_len: bytes.len() as u64,
            },
            bytes,
            "Noto Sans",
            cx,
        )
        .expect("font");
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1200.), px(760.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Finstack Interaction Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    let chart = cx.new(|cx| ChartView::new(input(0, font.clone()), cx));
                    let observe = cx.observe(&chart, |_, _, cx| cx.notify());
                    Gallery {
                        chart,
                        font,
                        case: 0,
                        tool: NativeDragTool::Pan,
                        status: "Ready".into(),
                        _observe: observe,
                    }
                })
            },
        )
        .expect("window");
        cx.activate(true);
    });
}
