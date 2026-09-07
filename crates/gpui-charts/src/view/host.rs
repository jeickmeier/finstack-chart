use super::*;
use chart_core::state::StateEvent;
use std::collections::BTreeSet;

/// Application operations whose implementation remains explicitly host-owned.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostCommand {
    /// Copy a semantic data alternative through the host clipboard service.
    Copy,
    /// Export the captured scene through the host's selected destination/profile.
    Export,
    /// Open a replaceable context menu for the captured input location/targets.
    ContextMenu,
}
/// Replaceable native control locations. Their bodies have no implicit publication representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlSlot {
    /// Top-left chart toolbar.
    Toolbar,
    /// Bottom-left series/guide controls; actions still use the shared reducer.
    Legend,
    /// Pointer context menu, dismissed by Escape or a chart pointer action.
    ContextMenu,
}
/// Native body factory; no per-source-row GPUI entity is required.
pub type ControlBuilder =
    Rc<dyn Fn(&ChartState, Option<&Inspector>, &mut Window, &mut App) -> AnyElement>;
/// Immutable actual scene and semantic targets captured when a host operation was requested.
#[derive(Clone, Debug)]
pub struct HostContext {
    /// Exact acknowledged scene; source and geometry remain valid for the handler's lifetime.
    pub scene: Arc<chart_core::layout::LaidOutChart>,
    /// Current stable selection without population filtering.
    pub selection: Vec<MarkTarget>,
    /// Current focus identity.
    pub focus: Option<MarkTarget>,
    /// Optional scene-local context-menu location.
    pub position: Option<Point>,
}
/// Subscribe once to common changes and explicitly enabled host-operation requests.
#[derive(Clone, Debug)]
pub enum ChartHostEvent {
    /// One effective common reducer event; linked origin is preserved for echo suppression.
    StateChanged(StateEvent),
    /// Explicit capability dispatch with retained presented resources.
    Requested {
        /// Operation the host enabled and handles.
        command: HostCommand,
        /// Coherent immutable input basis.
        context: HostContext,
    },
}
impl gpui::EventEmitter<ChartHostEvent> for ChartView {}
#[derive(Default)]
pub(super) struct HostState {
    enabled: BTreeSet<HostCommand>,
    toolbar: Option<ControlBuilder>,
    legend: Option<ControlBuilder>,
    menu: Option<ControlBuilder>,
    pub(super) menu_at: Option<Point>,
    summary: Option<String>,
}
impl ChartView {
    /// Declare only operations for which the host has installed an event handler.
    pub fn set_host_commands(&mut self, commands: &[HostCommand], cx: &mut Context<Self>) {
        self.host.enabled = commands.iter().copied().collect();
        cx.notify();
    }
    /// Runtime capability check; copy/export are never advertised before the host enables them.
    pub fn supports_host_command(&self, command: HostCommand) -> bool {
        self.host.enabled.contains(&command)
    }
    /// Replace or remove a bounded native control factory without changing core actions.
    pub fn set_control_builder(
        &mut self,
        slot: ControlSlot,
        builder: Option<ControlBuilder>,
        cx: &mut Context<Self>,
    ) {
        match slot {
            ControlSlot::Toolbar => self.host.toolbar = builder,
            ControlSlot::Legend => self.host.legend = builder,
            ControlSlot::ContextMenu => {
                if builder.is_some() {
                    self.host.enabled.insert(HostCommand::ContextMenu);
                } else {
                    self.host.enabled.remove(&HostCommand::ContextMenu);
                    self.host.menu_at = None;
                }
                self.host.menu = builder;
            }
        }
        cx.notify();
    }
    /// Override the automatic chart summary with meaningful application context, bounded to 8192 bytes.
    pub fn set_accessible_summary(
        &mut self,
        summary: Option<String>,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if summary.as_ref().is_some_and(|s| s.len() > 8192) {
            return Err(Diagnostic::error(
                chart_core::DiagnosticCode::ResourceLimit,
                "Chart summary exceeds its text budget.",
                "Use a concise summary and the paged data alternative.",
            ));
        }
        self.host.summary = summary;
        cx.notify();
        Ok(())
    }
    /// Emit one explicit host request. Missing capabilities return a recoverable diagnostic.
    pub fn request_host_command(
        &mut self,
        command: HostCommand,
        position: Option<Point>,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if !self.supports_host_command(command) {
            return Err(Diagnostic::error(
                chart_core::DiagnosticCode::UnsupportedCapability,
                "Host operation has no installed capability.",
                "Install a handler and enable that operation before presenting its control.",
            ));
        }
        let scene = self
            .inspector
            .as_ref()
            .ok_or_else(|| {
                Diagnostic::error(
                    chart_core::DiagnosticCode::Validation,
                    "Host operation requires a presented scene.",
                    "Wait for the first successful paint.",
                )
            })?
            .presented()
            .clone();
        if command == HostCommand::ContextMenu {
            self.host.menu_at = position.or(Some(Point::new(12., 12.)?));
            cx.notify();
        }
        cx.emit(ChartHostEvent::Requested {
            command,
            context: HostContext {
                scene,
                selection: self.state().selection().iter().cloned().collect(),
                focus: self.state().focus().cloned(),
                position,
            },
        });
        Ok(())
    }
    pub(super) fn host_elements(&self, window: &mut Window, cx: &mut App) -> Vec<AnyElement> {
        let mut elements = vec![];
        for (builder, top) in [(&self.host.toolbar, true), (&self.host.legend, false)] {
            if let Some(builder) = builder {
                let body = builder(self.state(), self.inspector.as_ref(), window, cx);
                let container = div()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                    .on_mouse_move(|_, _, cx| cx.stop_propagation())
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .absolute()
                    .left(px(8.))
                    .max_w(px(480.))
                    .p_2()
                    .bg(gpui::rgba(0xfffffff0))
                    .child(body);
                elements.push(if top {
                    container.top(px(8.)).into_any_element()
                } else {
                    container.bottom(px(8.)).into_any_element()
                });
            }
        }
        if let (Some(builder), Some(at)) = (&self.host.menu, self.host.menu_at) {
            let body = builder(self.state(), self.inspector.as_ref(), window, cx);
            elements.push(
                div()
                    .id("chart-context-menu")
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_mouse_down(MouseButton::Right, |_, _, cx| cx.stop_propagation())
                    .on_mouse_move(|_, _, cx| cx.stop_propagation())
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
                    .role(Role::Menu)
                    .aria_label("Chart actions")
                    .absolute()
                    .left(px(at.x().max(0.) as f32))
                    .top(px(at.y().max(0.) as f32))
                    .max_w(px(360.))
                    .p_3()
                    .bg(rgb(0xffffff))
                    .border_1()
                    .border_color(rgb(0xaabbcc))
                    .child(body)
                    .into_any_element(),
            );
        }
        elements
    }
    pub(super) fn accessibility_elements(
        &self,
        cx: &Context<Self>,
    ) -> (String, Option<AnyElement>) {
        let Some(inspector) = &self.inspector else {
            return (
                self.host
                    .summary
                    .clone()
                    .unwrap_or_else(|| "Chart awaiting presentation".into()),
                None,
            );
        };
        let automatic = format!(
            "Interactive chart with {} visible targets and {} selected. Arrow keys inspect; Space toggles selection; Escape cancels.",
            inspector.semantic_targets().len(),
            self.state().selection().len()
        );
        let summary = self.host.summary.clone().unwrap_or(automatic);
        if let Some((index, tool)) = self.input.focused_edit.and_then(|index| {
            self.input
                .annotation_tools
                .get(index)
                .map(|tool| (index, tool))
        }) {
            let annotations = inspector
                .presented()
                .prepared()
                .state()
                .annotations(inspector.presented().prepared().definition());
            if let Some(annotation) = annotations.iter().find(|a| a.id == tool.id) {
                let anchor = if tool.part == chart_core::editing::AnnotationPart::Callout {
                    annotation.callout.as_ref()
                } else {
                    Some(&annotation.anchor)
                };
                if let Some(anchor) = anchor {
                    let p = inspector
                        .presented()
                        .project_anchor(anchor)
                        .ok()
                        .flatten()
                        .map(|v| v.0);
                    let weak = cx.entity().downgrade();
                    let node=div().id(gpui::SharedString::from(format!("annotation-handle-{}-{index}",tool.id))).role(Role::Button)
                        .aria_label(format!("{} annotation handle. {}",tool.id,anchor_description(anchor)))
                        .aria_description("Arrow keys move by the configured snap step. Escape leaves the handle; undo restores the previous edit.")
                        .aria_active_descendant().focusable()
                        .on_a11y_action(gpui::AccessibleAction::Click,move|_,window,cx|{let _=weak.update(cx,|this,cx|{window.focus(&this.focus,cx);this.focus_annotation(index,cx)});})
                        .absolute().left(px(p.map_or(0.,|p|p.x()) as f32)).top(px(p.map_or(0.,|p|p.y()) as f32)).w(px(1.)).h(px(1.)).into_any_element();
                    return (summary, Some(node));
                }
            }
        }
        let focused = self
            .state()
            .focus()
            .and_then(|t| inspector.target(t).ok().flatten())
            .and_then(|h| {
                inspector
                    .describe_target(h, self.state())
                    .ok()
                    .map(|d| (h, d))
            });
        let child = focused.map(|(hit, description)| {
            let label = format!(
                "{}. {}",
                description.description,
                description
                    .cells
                    .iter()
                    .map(|c| format!("{}: {}", c.field, c.value))
                    .collect::<Vec<_>>()
                    .join("; ")
            );
            let id = gpui::SharedString::from(format!(
                "chart-target-{}",
                serde_identity(&description.target)
            ));
            let target = description.target.clone();
            let weak = cx.entity().downgrade();
            div()
                .id(id)
                .role(Role::ListBoxOption)
                .on_a11y_action(gpui::AccessibleAction::Click, move |_, window, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        window.focus(&this.focus, cx);
                        let request = this.action_request(
                            ChartAction::Select {
                                change: chart_core::state::SelectionChange::Toggle,
                                targets: vec![target.clone()],
                            },
                            ActionOrigin::Control,
                        );
                        this.dispatch_action(request, cx)
                    });
                })
                .aria_label(label)
                .aria_selected(description.selected)
                .aria_active_descendant()
                .absolute()
                .left(px(hit.position.x() as f32))
                .top(px(hit.position.y() as f32))
                .w(px(1.))
                .h(px(1.))
                .into_any_element()
        });
        (summary, child)
    }
}
fn serde_identity(target: &MarkTarget) -> String {
    // Stable identity within the mounted process; no visible source data is encoded in node IDs.
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    format!("{target:?}").hash(&mut h);
    format!("{:016x}", h.finish())
}

fn anchor_description(anchor: &chart_core::composition::Anchor) -> String {
    use chart_core::composition::{Anchor, ScaleValue};
    let value = |v: &ScaleValue| match v {
        ScaleValue::Number(v) => v.to_string(),
        ScaleValue::Category(v) => v.clone(),
        ScaleValue::Timestamp { value, unit } => format!("{value} {unit:?}"),
    };
    match anchor {
        Anchor::Data { x, y, .. } => format!("x {}, y {}", value(x), value(y)),
        Anchor::Figure { x, y } | Anchor::Panel { x, y, .. } | Anchor::Output { x, y } => {
            format!("x {x}, y {y}")
        }
    }
}
