//! WP-15 controls exercise the same window-independent reducer as Python and WASM.
use chart_core::plot::{Data, aes, labels, plot, points};
use chart_core::{
    composition::{Anchor, Annotation, Collision},
    state::{FollowMode, *},
    typography::RichText,
    *,
};
use gpui::{prelude::*, *};
use gpui_charts::*;

fn annotation(x: f64) -> Annotation {
    Annotation {
        id: "threshold".into(),
        anchor: Anchor::Figure { x, y: 0.1 },
        text: RichText::plain("Threshold"),
        offset: [0., 0.],
        priority: 0,
        collision: Collision::Keep,
        callout: None,
        connector_origin: chart_core::composition::ConnectorOrigin::Label,
        overflow: false,
    }
}
struct Gallery {
    chart: Entity<ChartView>,
    next_gesture: u64,
    status: String,
}
impl Gallery {
    fn act(&mut self, label: &str, cx: &mut Context<Self>) {
        let actions = match label {
            "Preview edit" => {
                self.next_gesture = match self.chart.read(cx).next_gesture_id() {
                    Ok(id) => id.get(),
                    Err(e) => {
                        self.status = e.message;
                        return;
                    }
                };
                vec![
                    ChartAction::BeginGesture {
                        id: Revision::new(self.next_gesture),
                        kind: GestureKind::Annotation("threshold".into()),
                    },
                    ChartAction::PreviewGesture {
                        id: Revision::new(self.next_gesture),
                        preview: GesturePreview::Annotation(annotation(0.7).into()),
                    },
                ]
            }
            "Commit" => vec![ChartAction::CommitGesture {
                id: Revision::new(self.next_gesture),
            }],
            "Cancel" => vec![ChartAction::CancelGesture(CancelReason::Explicit)],
            "Undo" => vec![ChartAction::Undo],
            "Redo" => vec![ChartAction::Redo],
            "Freeze" => vec![ChartAction::SetFollow(FollowMode::FreezePresentation)],
            "Resume" => vec![ChartAction::ResumeLatest],
            "Zoom" => vec![ChartAction::SetViewport(Viewport {
                x: Some((0.5, 1.5)),
                y: None,
            })],
            _ => vec![ChartAction::Reset],
        };
        for action in actions {
            let result = self.chart.update(cx, |chart, cx| {
                let request = chart.action_request(action, ActionOrigin::Control);
                chart.dispatch_action(request, cx)
            });
            match result {
                Ok(outcome) => {
                    self.status = outcome.event.map_or_else(
                        || "No effective change".into(),
                        |event| {
                            format!(
                                "{} · {} · revision {}",
                                label,
                                if event.durable {
                                    "Committed change"
                                } else {
                                    "Transient change"
                                },
                                event.revision.get()
                            )
                        },
                    )
                }
                Err(error) => {
                    self.status = error.message;
                    break;
                }
            }
        }
        cx.notify();
    }
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.chart.read(cx).state();
        let committed = state
            .interaction_snapshot()
            .annotations
            .get("threshold")
            .and_then(|a| a.as_ref())
            .map_or(0.1, |a| match a.anchor {
                Anchor::Figure { x, .. } => x,
                _ => 0.1,
            });
        let detail = format!(
            "Committed label x: {committed:.1} · Preview: {} · Follow: {:?} · State revision: {}",
            state.active_gesture().is_some(),
            state.follow(),
            state.revision().get()
        );
        let mut controls = div().flex().gap_2();
        for label in [
            "Preview edit",
            "Commit",
            "Cancel",
            "Undo",
            "Redo",
            "Freeze",
            "Resume",
            "Zoom",
            "Reset",
        ] {
            controls = controls.child(
                div()
                    .id(label)
                    .role(Role::Button)
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(0xffffff))
                    .cursor_pointer()
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| this.act(label, cx))),
            );
        }
        div().size_full().flex().flex_col().gap_3().p_4().bg(rgb(0xecf1f8)).text_color(rgb(0x202b3c)).font_family("Noto Sans")
            .child(div().text_xl().child("Shared actions · gesture and state ownership"))
            .child("Preview moves the label; cancellation restores it. Commit creates one undo command.")
            .child(controls).child(detail).child(self.status.clone())
            .child(div().flex_1().min_h_0().w_full().bg(rgb(0xffffff)).child(self.chart.clone()))
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
        let data = Data::columns()
            .column("x", [0., 1., 2., 3.])
            .column("y", [1., 3., 2., 4.])
            .build()
            .expect("data");
        let plot = plot(data)
            .aes(aes().x("x").y("y"))
            .layer(points())
            .layer(
                labels()
                    .id("threshold")
                    .figure_at(0.3, 0.1)
                    .text("Threshold"),
            )
            .build()
            .expect("plot");
        let input = ChartInput::from_plot(&plot, font).expect("native input");
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1120.), px(700.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Finstack Actions Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| Gallery {
                    chart: cx.new(|cx| ChartView::new(input, cx)),
                    next_gesture: 0,
                    status: "Ready".into(),
                })
            },
        )
        .expect("window");
        cx.activate(true);
    });
}
