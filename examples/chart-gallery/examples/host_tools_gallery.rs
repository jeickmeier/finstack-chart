//! WP-17 two linked charts, a bounded accessible table, replaceable controls and annotation tools.
use chart_core::composition::Anchor;
use chart_core::{
    composition::*, editing::*, grammar::ScaleBindings, inspection::Inspector, linking::*,
    portable::Session, services::*, state::*, typography::RichText, *,
};
use chart_export::{
    FigureSnapshot, FontResource, FontResources, Format, PageSize, PublicationProfile,
};
use gpui::{prelude::*, *};
use gpui_charts::*;
use std::{rc::Rc, sync::Arc};
const FONT: &[u8] = include_bytes!("../../../fixtures/capability/fonts/NotoSans-Regular.ttf");
fn font_descriptor() -> ResourceDescriptor {
    ResourceDescriptor {
        id: ResourceId::new(1),
        revision: Revision::new(1),
        kind: ResourceKind::Font,
        byte_len: FONT.len() as u64,
    }
}
fn annotation(id: &str, y: f64) -> Annotation {
    let anchor = |x| Anchor::Data {
        panel: None,
        scales: ScaleBindings::default(),
        x: ScaleValue::Number(x),
        y: ScaleValue::Number(y),
    };
    Annotation {
        id: id.into(),
        anchor: anchor(0.5),
        callout: Some(anchor(3.5)),
        text: RichText::plain(id),
        offset: [0., -22.],
        priority: 0,
        collision: Collision::Keep,
        connector_origin: ConnectorOrigin::Anchor,
        overflow: false,
    }
}
fn input(layer: u64, font: NativeFont) -> ChartInput {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/interaction/cases.json")).unwrap();
    let c = &cases[0];
    let session = Session::new(&c["chart"].to_string(), &c["data"].to_string()).unwrap();
    let mut definition = session.definition().clone();
    definition.layers[0].id = LayerId::new(layer);
    for a in &mut definition.axes {
        a.visible = true;
    }
    definition.figure = Some(FigureComposition {
        annotations: vec![annotation("Threshold", 3.), annotation("Range", 1.)],
        ..FigureComposition::default()
    });
    ChartInput::new(definition, session.source(), font).unwrap()
}
fn host_controls(chart: &Entity<ChartView>, layer: u64, cx: &mut App) {
    chart.update(cx,|chart,cx|{
        chart.set_host_commands(&[HostCommand::Copy,HostCommand::Export,HostCommand::ContextMenu],cx);
        chart.set_accessible_summary(Some(format!("Linked observation chart {}. Five source observations. Arrow keys inspect values; Space selects; the data table provides the same observations.",layer)),cx).unwrap();
    });
    let weak = chart.downgrade();
    let toolbar: ControlBuilder = Rc::new(move |_, _, _, _| {
        let mut bar = div().flex().gap_2();
        for (name, command) in [("Copy", HostCommand::Copy), ("Export", HostCommand::Export)] {
            let target = weak.clone();
            bar = bar.child(
                div()
                    .id(name)
                    .role(Role::Button)
                    .aria_label(name)
                    .px_2()
                    .py_1()
                    .bg(rgb(0xe7eff9))
                    .child(name)
                    .on_click(move |_, _, cx| {
                        let _ = target.update(cx, |chart, cx| {
                            chart.request_host_command(command, None, cx)
                        });
                    }),
            );
        }
        bar.into_any_element()
    });
    let weak = chart.downgrade();
    let legend: ControlBuilder = Rc::new(move |state, _, _, _| {
        let visible = state.is_visible(LayerId::new(layer));
        let target = weak.clone();
        div()
            .id("series-toggle")
            .role(Role::Button)
            .aria_label(if visible {
                "Hide observations"
            } else {
                "Show observations"
            })
            .child(if visible {
                "● Observations"
            } else {
                "○ Observations hidden"
            })
            .on_click(move |_, _, cx| {
                let _ = target.update(cx, |chart, cx| {
                    chart.dispatch_chart(
                        ChartAction::SetLayerVisible {
                            layer: LayerId::new(layer),
                            visible: !visible,
                        },
                        cx,
                    )
                });
            })
            .into_any_element()
    });
    let weak = chart.downgrade();
    let menu: ControlBuilder = Rc::new(move |_, _, _, _| {
        let target = weak.clone();
        div()
            .id("menu-copy")
            .role(Role::MenuItem)
            .aria_label("Copy selected observations")
            .child("Copy selected observations")
            .on_click(move |_, _, cx| {
                let _ = target.update(cx, |chart, cx| {
                    chart.request_host_command(HostCommand::Copy, None, cx)
                });
            })
            .into_any_element()
    });
    chart.update(cx, |chart, cx| {
        chart.set_control_builder(ControlSlot::Toolbar, Some(toolbar), cx);
        chart.set_control_builder(ControlSlot::Legend, Some(legend), cx);
        chart.set_control_builder(ControlSlot::ContextMenu, Some(menu), cx);
        chart.set_tooltip(
            Rc::new(|i, _, _| {
                let text = i
                    .hits()
                    .first()
                    .and_then(|h| i.describe_target(h, i.presented().prepared().state()).ok())
                    .map(|d| {
                        format!(
                            "{}\n{}",
                            d.description,
                            d.cells
                                .iter()
                                .map(|v| format!("{}: {}", v.field, v.value))
                                .collect::<Vec<_>>()
                                .join(" · ")
                        )
                    })
                    .unwrap_or_default();
                div().child(text).into_any_element()
            }),
            cx,
        );
    });
}
struct Gallery {
    charts: [Entity<ChartView>; 2],
    pending: [Option<StateEvent>; 2],
    status: String,
    linked: u64,
    _subscriptions: Vec<Subscription>,
}
impl Gallery {
    fn event(&mut self, index: usize, event: &ChartHostEvent, cx: &mut Context<Self>) {
        match event {
            ChartHostEvent::StateChanged(event) => {
                if event.durable && !matches!(event.origin, ActionOrigin::Linked(_)) {
                    self.pending[index] = Some(event.clone());
                }
            }
            ChartHostEvent::Requested { command, context } => {
                self.status = match command {
                    HostCommand::ContextMenu => "Context menu uses the captured chart scene".into(),
                    HostCommand::Copy => {
                        let i = Inspector::new(context.scene.clone(), 10., 32).unwrap();
                        let rows = context
                            .selection
                            .iter()
                            .filter_map(|t| i.target(t).ok().flatten())
                            .filter_map(|h| {
                                i.describe_target(h, context.scene.prepared().state()).ok()
                            })
                            .map(|d| {
                                format!(
                                    "{}\t{}",
                                    d.description,
                                    d.cells
                                        .iter()
                                        .map(|c| format!("{}: {}", c.field, c.value))
                                        .collect::<Vec<_>>()
                                        .join("\t")
                                )
                            })
                            .collect::<Vec<_>>();
                        let text = if rows.is_empty() {
                            "No selected observations".into()
                        } else {
                            rows.join("\n")
                        };
                        cx.write_to_clipboard(ClipboardItem::new_string(text));
                        format!("Copied {} selected observations", rows.len())
                    }
                    HostCommand::Export => {
                        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                            let fonts = FontResources::new(vec![FontResource::new(
                                font_descriptor(),
                                Arc::from(FONT),
                            )?])?;
                            let profile = PublicationProfile::new(
                                PageSize::points(480., 300.)?,
                                font_descriptor(),
                            )?;
                            let prepared = context.scene.prepared();
                            let snapshot = FigureSnapshot::capture(
                                prepared.definition(),
                                prepared.source().clone(),
                                prepared.state(),
                                fonts,
                                profile,
                            )?;
                            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                                .join("../../artifacts/wp-17");
                            std::fs::create_dir_all(&dir)?;
                            std::fs::write(
                                dir.join("native-host.svg"),
                                snapshot.export(Format::Svg)?.bytes,
                            )?;
                            std::fs::write(
                                dir.join("native-host.png"),
                                snapshot.export(Format::Png)?.bytes,
                            )?;
                            Ok(())
                        })();
                        result.err().map_or_else(
                            || {
                                "Exported captured chart to artifacts/wp-17/native-host.svg and PNG"
                                    .into()
                            },
                            |e| e.to_string(),
                        )
                    }
                };
            }
        }
        cx.notify();
    }
    fn links(&mut self, cx: &mut Context<Self>) {
        for index in 0..2 {
            let Some(event) = self.pending[index].clone() else {
                continue;
            };
            let source = self.charts[index].read(cx);
            let Some(i) = source.inspector() else {
                continue;
            };
            let result = LinkMessage::from_event(
                if index == 0 { "left" } else { "right" },
                &event,
                i,
                source.state(),
                &[ScaleId::new(0)],
                None,
                true,
            );
            let message = match result {
                Ok(Some(m)) => m,
                Ok(None) => {
                    self.pending[index] = None;
                    continue;
                }
                Err(e) if e.code == chart_core::DiagnosticCode::Superseded => continue,
                Err(e) => {
                    self.status = e.message;
                    self.pending[index] = None;
                    continue;
                }
            };
            self.pending[index] = None;
            let other = 1 - index;
            let target = self.charts[other].read(cx);
            let Some(i) = target.inspector() else {
                continue;
            };
            let update = message.resolve(
                i,
                &[AxisLink {
                    source: ScaleId::new(0),
                    destination: ScaleId::new(0),
                }],
                None,
                MissingMatch::ReportAndOmit,
            );
            match update {
                Ok(update) => {
                    let r = self.charts[other].update(cx, |chart, cx| {
                        let request = chart.action_request(update.action, update.origin);
                        chart.dispatch_action(request, cx)
                    });
                    match r {
                        Ok(r) => {
                            if r.outcome.changed {
                                self.linked += 1;
                            }
                        }
                        Err(e) => self.status = e.message,
                    }
                }
                Err(e) => self.status = e.message,
            }
        }
    }
}
impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.links(cx);
        let left = self.charts[0].read(cx);
        let page = left
            .inspector()
            .and_then(|i| i.accessible_page(left.state(), 0, 16).ok());
        let annotation_values = left
            .inspector()
            .map(|i| {
                left.state()
                    .annotations(i.presented().prepared().definition())
                    .iter()
                    .map(|a| {
                        let value = |anchor: &Anchor| match anchor {
                            Anchor::Data {
                                x: ScaleValue::Number(x),
                                y: ScaleValue::Number(y),
                                ..
                            } => Some((*x, *y)),
                            _ => None,
                        };
                        let (x, y) = value(&a.anchor).unwrap_or_default();
                        if a.id == "Threshold" {
                            format!("Threshold {y:.2}")
                        } else {
                            format!(
                                "Range {x:.2}–{:.2}",
                                a.callout.as_ref().and_then(value).unwrap_or_default().0
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" · ")
            })
            .unwrap_or_default();
        let detail = format!(
            "Left selected {} · Right selected {} · Linked changes {} · Active edit {}",
            left.state().selection().len(),
            self.charts[1].read(cx).state().selection().len(),
            self.linked,
            left.state().active_gesture().is_some()
        );
        let mut buttons = div().flex().gap_2();
        for (label, index) in [
            ("Edit threshold", 0),
            ("Edit range start", 1),
            ("Edit range end", 2),
        ] {
            buttons=buttons.child(div().id(label).role(Role::Button).aria_label(label).p_2().bg(rgb(0xffffff)).child(label).on_click(cx.listener(move|this,_,window,cx|{
                let focus=this.charts[0].read(cx).focus_handle(cx);window.focus(&focus,cx);
                let r=this.charts[0].update(cx,|chart,cx|chart.focus_annotation(index,cx));this.status=r.err().map_or_else(||format!("{label}: arrows move one snap step; Shift moves ten; Escape leaves handle"),|e|e.message);cx.notify();
            })));
        }
        for (label, action) in [
            ("Undo", ChartAction::Undo),
            ("Redo", ChartAction::Redo),
            ("Reset", ChartAction::Reset),
        ] {
            buttons = buttons.child(
                div()
                    .id(label)
                    .role(Role::Button)
                    .aria_label(label)
                    .p_2()
                    .bg(rgb(0xffffff))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let r = this.charts[0]
                            .update(cx, |chart, cx| chart.dispatch_chart(action.clone(), cx));
                        this.status = r.err().map_or_else(|| label.into(), |e| e.message);
                        cx.notify();
                    })),
            );
        }
        buttons = buttons.child(
            div()
                .id("lose-focus")
                .role(Role::Button)
                .aria_label("Lose focus")
                .p_2()
                .bg(rgb(0xffffff))
                .child("Lose focus")
                .on_click(|_, window, cx| window.blur(cx)),
        );
        buttons = buttons.child(
            div()
                .id("preview-threshold")
                .role(Role::Button)
                .aria_label("Preview threshold")
                .p_2()
                .bg(rgb(0xffffff))
                .child("Preview threshold")
                .on_click(cx.listener(|this, _, window, cx| {
                    let focus = this.charts[0].read(cx).focus_handle(cx);
                    window.focus(&focus, cx);
                    let result = this.charts[0].update(cx, |chart, cx| -> ChartResult<()> {
                        let Some(inspector) = chart.inspector() else {
                            return Ok(());
                        };
                        let scene = inspector.presented().clone();
                        let editor = chart_core::editing::AnnotationEditor::new(
                            scene.clone(),
                            "Threshold",
                            AnnotationPart::Translate,
                            EditConstraints {
                                horizontal: false,
                                vertical: true,
                                y: Some(ValueConstraint::Number {
                                    bounds: Some([0.5, 3.5]),
                                    step: Some(0.5),
                                    origin: 0.,
                                }),
                                ..EditConstraints::default()
                            },
                        )?;
                        let annotation = editor.nudge(scene.scene().stamp(), false, true, 1)?;
                        let id = chart.next_gesture_id()?;
                        chart.dispatch_chart(
                            ChartAction::BeginGesture {
                                id,
                                kind: GestureKind::Annotation("Threshold".into()),
                            },
                            cx,
                        )?;
                        chart.dispatch_chart(
                            ChartAction::PreviewGesture {
                                id,
                                preview: GesturePreview::Annotation(Box::new(annotation)),
                            },
                            cx,
                        )?;
                        Ok(())
                    });
                    this.status = result.err().map_or_else(
                        || "Preview only: Escape or Lose focus cancels".into(),
                        |e| e.message,
                    );
                    cx.notify();
                })),
        );
        let mut table = div()
            .id("linked-data-table")
            .role(Role::Table)
            .aria_label("Accessible observation table linked to both charts")
            .aria_column_count(3)
            .flex()
            .flex_col()
            .gap_1();
        if let Some(page) = page {
            table = table.aria_row_count(page.total).child(page.summary);
            for (index, row) in page.targets.into_iter().enumerate() {
                let target = row.target.clone();
                let label = format!(
                    "{}. {}",
                    row.description,
                    row.cells
                        .iter()
                        .map(|c| format!("{} {}", c.field, c.value))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let mut line = div()
                    .id(("source-row", index))
                    .role(Role::Row)
                    .aria_row_index(index + 1)
                    .aria_selected(row.selected)
                    .aria_label(label)
                    .flex()
                    .gap_3()
                    .p_1()
                    .bg(rgb(if row.selected { 0xc6dcf6 } else { 0xffffff }));
                line = line.child(
                    div()
                        .id("identity")
                        .role(Role::Cell)
                        .aria_label(row.description.clone())
                        .w(px(430.))
                        .child(row.description),
                );
                for (j, cell) in row.cells.into_iter().enumerate() {
                    line = line.child(
                        div()
                            .id(("value", j))
                            .role(Role::Cell)
                            .aria_label(format!("{}: {}", cell.field, cell.value))
                            .child(cell.value),
                    );
                }
                table = table.child(line.on_click(cx.listener(move |this, _, _, cx| {
                    let r = this.charts[0].update(cx, |chart, cx| {
                        chart.dispatch_chart(
                            ChartAction::Select {
                                change: SelectionChange::Toggle,
                                targets: vec![target.clone()],
                            },
                            cx,
                        )
                    });
                    if let Err(e) = r {
                        this.status = e.message;
                    }
                    cx.notify();
                })));
            }
        }
        div().size_full().flex().flex_col().p_4().gap_2().bg(rgb(0xecf1f8)).text_color(rgb(0x202b3c)).font_family("Noto Sans")
            .child(div().text_xl().child("Linked views and annotation editing"))
            .child("Drag blue handles · threshold snaps by 0.5 · range endpoints snap by 0.25 · arrows/Space inspect/select · Escape cancels · right-click opens host menu")
            .child(buttons).child(detail).child(self.status.clone()).child(annotation_values)
            .child(div().flex().gap_3().flex_1().min_h_0().child(div().flex_1().h_full().bg(rgb(0xffffff)).child(self.charts[0].clone())).child(div().flex_1().h_full().bg(rgb(0xffffff)).child(self.charts[1].clone())))
            .child(table)
    }
}
fn main() {
    gpui_platform::application().run(|cx: &mut App| {
        let font = NativeFont::load(font_descriptor(), Arc::from(FONT), "Noto Sans", cx).unwrap();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1280.), px(900.)),
                    cx,
                ))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Finstack Host Tools Proof".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    let charts = [
                        cx.new(|cx| ChartView::new(input(1, font.clone()), cx)),
                        cx.new(|cx| ChartView::new(input(99, font.clone()), cx)),
                    ];
                    for (i, chart) in charts.iter().enumerate() {
                        host_controls(chart, if i == 0 { 1 } else { 99 }, cx);
                    }
                    charts[0]
                        .update(cx, |chart, cx| {
                            chart.set_annotation_tools(
                                vec![
                                    NativeAnnotationTool {
                                        id: "Threshold".into(),
                                        part: AnnotationPart::Translate,
                                        constraints: EditConstraints {
                                            horizontal: false,
                                            vertical: true,
                                            y: Some(ValueConstraint::Number {
                                                bounds: Some([0.5, 3.5]),
                                                step: Some(0.5),
                                                origin: 0.,
                                            }),
                                            ..EditConstraints::default()
                                        },
                                    },
                                    NativeAnnotationTool {
                                        id: "Range".into(),
                                        part: AnnotationPart::Anchor,
                                        constraints: EditConstraints {
                                            horizontal: true,
                                            vertical: false,
                                            x: Some(ValueConstraint::Number {
                                                bounds: Some([0., 4.]),
                                                step: Some(0.25),
                                                origin: 0.,
                                            }),
                                            preserve_order: Some(true),
                                            ..EditConstraints::default()
                                        },
                                    },
                                    NativeAnnotationTool {
                                        id: "Range".into(),
                                        part: AnnotationPart::Callout,
                                        constraints: EditConstraints {
                                            horizontal: true,
                                            vertical: false,
                                            x: Some(ValueConstraint::Number {
                                                bounds: Some([0., 4.]),
                                                step: Some(0.25),
                                                origin: 0.,
                                            }),
                                            preserve_order: Some(true),
                                            ..EditConstraints::default()
                                        },
                                    },
                                ],
                                cx,
                            )
                        })
                        .unwrap();
                    let mut subscriptions = vec![];
                    for (i, chart) in charts.iter().enumerate() {
                        subscriptions.push(
                            cx.subscribe(chart, move |this: &mut Gallery, _, event, cx| {
                                this.event(i, event, cx)
                            }),
                        );
                        subscriptions.push(cx.observe(chart, |_, _, cx| cx.notify()));
                    }
                    Gallery {
                        charts,
                        pending: [None, None],
                        status: "Ready".into(),
                        linked: 0,
                        _subscriptions: subscriptions,
                    }
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
