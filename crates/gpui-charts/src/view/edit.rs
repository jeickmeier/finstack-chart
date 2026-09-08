use super::*;
use chart_core::editing::{AnnotationEditor, AnnotationPart, EditConstraints};
use chart_core::state::{CancelReason, GestureKind, GesturePreview};

/// Explicit editable annotation handle supplied by the host. Later handles win overlap ties.
#[derive(Clone, Debug)]
pub struct NativeAnnotationTool {
    /// Existing authored/committed annotation identity.
    pub id: String,
    /// Anchor, callout endpoint, or joint threshold movement.
    pub part: AnnotationPart,
    /// Typed snapping, movement and range-order policy.
    pub constraints: EditConstraints,
}
impl NativeAnnotationTool {
    /// Begin an explicit native edit handle for one authored annotation.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            part: AnnotationPart::Anchor,
            constraints: EditConstraints::default(),
        }
    }
    /// Choose anchor, callout endpoint or joint threshold movement.
    pub fn part(mut self, part: AnnotationPart) -> Self {
        self.part = part;
        self
    }
    /// Apply the shared exact snapping and movement constraints.
    pub fn constraints(mut self, constraints: EditConstraints) -> Self {
        self.constraints = constraints;
        self
    }
}
pub(super) fn validate_tools(tools: &[NativeAnnotationTool]) -> ChartResult<()> {
    if tools.len() > 256 || tools.iter().any(|t| t.id.is_empty() || t.id.len() > 256) {
        return Err(Diagnostic::error(
            chart_core::DiagnosticCode::ResourceLimit,
            "Annotation tools exceed identity/count bounds.",
            "Use at most 256 bounded authored annotation handles.",
        ));
    }
    Ok(())
}
pub(super) struct EditDrag {
    editor: AnnotationEditor,
    id: Revision,
    start: Point,
    origin: gpui::Point<Pixels>,
    moved: bool,
}
impl ChartView {
    /// Install at most 256 replaceable edit handles; this does not enable source-data editing.
    pub fn set_annotation_tools(
        &mut self,
        tools: Vec<NativeAnnotationTool>,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        validate_tools(&tools)?;
        self.cancel_input(CancelReason::Explicit, cx)?;
        self.input.annotation_tools = tools;
        self.input.focused_edit = None;
        cx.notify();
        Ok(())
    }
    /// Give one explicit handle keyboard ownership; arrows edit in destination units, Escape leaves it.
    pub fn focus_annotation(&mut self, index: usize, cx: &mut Context<Self>) -> ChartResult<()> {
        let tool = self.input.annotation_tools.get(index).ok_or_else(|| {
            Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Annotation handle is absent.",
                "Use an installed tool index.",
            )
        })?;
        let chart = self
            .inspector
            .as_ref()
            .ok_or_else(|| {
                Diagnostic::error(
                    chart_core::DiagnosticCode::Validation,
                    "Annotation handle needs a presented chart.",
                    "Wait for initial presentation.",
                )
            })?
            .presented()
            .clone();
        AnnotationEditor::new(chart, &tool.id, tool.part, tool.constraints.clone())?;
        self.cancel_input(CancelReason::Explicit, cx)?;
        self.dispatch_inspection(
            InspectionAction::Clear,
            chart_core::inspection::InputOrigin::Keyboard,
            cx,
        )?;
        self.input.focused_edit = Some(index);
        cx.notify();
        Ok(())
    }
    fn handle_position(&self, tool: &NativeAnnotationTool) -> ChartResult<Option<Point>> {
        let Some(frame) = &self.frame else {
            return Ok(None);
        };
        let annotations = frame
            .chart
            .prepared()
            .state()
            .annotations(frame.chart.prepared().definition());
        let Some(a) = annotations.iter().find(|a| a.id == tool.id) else {
            return Ok(None);
        };
        let anchor = if tool.part == AnnotationPart::Callout {
            a.callout.as_ref()
        } else {
            Some(&a.anchor)
        };
        let Some(anchor) = anchor else {
            return Ok(None);
        };
        Ok(frame.chart.project_anchor(anchor)?.and_then(|(p, clip)| {
            (p.x() >= clip.origin().x()
                && p.x() <= clip.max_x()
                && p.y() >= clip.origin().y()
                && p.y() <= clip.max_y())
            .then_some(p)
        }))
    }
    pub(super) fn edit_down(
        &mut self,
        event: &gpui::MouseDownEvent,
        cx: &mut Context<Self>,
    ) -> ChartResult<bool> {
        let Some(frame) = &self.frame else {
            return Ok(false);
        };
        let origin = frame.bounds.origin;
        let p = Point::new(
            f64::from(f32::from(event.position.x - origin.x)),
            f64::from(f32::from(event.position.y - origin.y)),
        )?;
        let mut chosen = None;
        for (i, tool) in self.input.annotation_tools.iter().enumerate().rev() {
            if self
                .handle_position(tool)?
                .is_some_and(|h| (h.x() - p.x()).hypot(h.y() - p.y()) <= 8.)
            {
                chosen = Some(i);
                break;
            }
        }
        let Some(index) = chosen else {
            self.input.focused_edit = None;
            return Ok(false);
        };
        let tool = &self.input.annotation_tools[index];
        let editor = AnnotationEditor::new(
            frame.chart.clone(),
            &tool.id,
            tool.part,
            tool.constraints.clone(),
        )?;
        let name = tool.id.clone();
        self.dispatch_inspection(
            InspectionAction::Clear,
            chart_core::inspection::InputOrigin::Pointer,
            cx,
        )?;
        let id = self.next_gesture_id()?;
        let request = self.action_request(
            ChartAction::BeginGesture {
                id,
                kind: GestureKind::Annotation(name),
            },
            ActionOrigin::Pointer,
        );
        self.dispatch_action(request, cx)?;
        self.input.edit = Some(EditDrag {
            editor,
            id,
            start: p,
            origin,
            moved: false,
        });
        self.input.focused_edit = Some(index);
        cx.stop_propagation();
        Ok(true)
    }
    pub(super) fn edit_move(
        &mut self,
        event: &gpui::MouseMoveEvent,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if event.pressed_button != Some(MouseButton::Left) {
            return self.cancel_input(CancelReason::CaptureLost, cx);
        }
        let edit = self.input.edit.as_mut().expect("active edit");
        let dx = f64::from(f32::from(event.position.x - edit.origin.x)) - edit.start.x();
        let dy = f64::from(f32::from(event.position.y - edit.origin.y)) - edit.start.y();
        if !edit.moved && dx.hypot(dy) < 3. {
            return Ok(());
        }
        let a = edit
            .editor
            .preview(edit.editor.presented().scene().stamp(), dx, dy)?;
        edit.moved = true;
        let id = edit.id;
        let request = self.action_request(
            ChartAction::PreviewGesture {
                id,
                preview: GesturePreview::Annotation(Box::new(a)),
            },
            ActionOrigin::Pointer,
        );
        self.dispatch_action(request, cx)?;
        cx.stop_propagation();
        Ok(())
    }
    pub(super) fn edit_up(
        &mut self,
        event: &gpui::MouseUpEvent,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        self.edit_move(
            &gpui::MouseMoveEvent {
                position: event.position,
                pressed_button: Some(MouseButton::Left),
                modifiers: event.modifiers,
            },
            cx,
        )?;
        let edit = self.input.edit.as_ref().expect("active edit");
        let action = if edit.moved {
            ChartAction::CommitGesture { id: edit.id }
        } else {
            ChartAction::CancelGesture(CancelReason::Explicit)
        };
        let request = self.action_request(action, ActionOrigin::Pointer);
        self.dispatch_action(request, cx)?;
        self.input.edit = None;
        cx.stop_propagation();
        Ok(())
    }
    pub(super) fn edit_key(
        &mut self,
        event: &gpui::KeyDownEvent,
        cx: &mut Context<Self>,
    ) -> ChartResult<bool> {
        let Some(index) = self.input.focused_edit else {
            return Ok(false);
        };
        if event.keystroke.key == "escape" {
            self.input.focused_edit = None;
            cx.notify();
            return Ok(true);
        }
        let steps = if event.keystroke.modifiers.shift {
            10
        } else {
            1
        };
        let step = 1.;
        let (dx, dy) = match event.keystroke.key.as_str() {
            "left" => (-step, 0.),
            "right" => (step, 0.),
            "up" => (0., -step),
            "down" => (0., step),
            _ => return Ok(false),
        };
        let tool = &self.input.annotation_tools[index];
        let Some(frame) = &self.frame else {
            return Ok(false);
        };
        let editor = AnnotationEditor::new(
            frame.chart.clone(),
            &tool.id,
            tool.part,
            tool.constraints.clone(),
        )?;
        let a = editor.nudge(
            frame.chart.scene().stamp(),
            dx != 0.,
            dx > 0. || dy > 0.,
            steps,
        )?;
        let id = self.next_gesture_id()?;
        let name = tool.id.clone();
        for action in [
            ChartAction::BeginGesture {
                id,
                kind: GestureKind::Annotation(name),
            },
            ChartAction::PreviewGesture {
                id,
                preview: GesturePreview::Annotation(Box::new(a)),
            },
            ChartAction::CommitGesture { id },
        ] {
            let r = self.action_request(action, ActionOrigin::Keyboard);
            if let Err(e) = self.dispatch_action(r, cx) {
                let _ = self.cancel_input(CancelReason::Explicit, cx);
                return Err(e);
            }
        }
        Ok(true)
    }
    pub(super) fn paint_edit_handles(&self, window: &mut Window) -> ChartResult<()> {
        let Some(frame) = &self.frame else {
            return Ok(());
        };
        for (i, tool) in self.input.annotation_tools.iter().enumerate() {
            if let Some(p) = self.handle_position(tool)? {
                let bounds = gpui::Bounds::new(
                    gpui::point(
                        frame.bounds.origin.x + px(p.x() as f32 - 4.),
                        frame.bounds.origin.y + px(p.y() as f32 - 4.),
                    ),
                    gpui::size(px(8.), px(8.)),
                );
                window.paint_quad(gpui::fill(
                    bounds,
                    if self.input.focused_edit == Some(i) {
                        rgb(0xc87121)
                    } else {
                        rgb(0x2167c8)
                    },
                ));
            }
        }
        Ok(())
    }
}
