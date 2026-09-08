use super::*;
use chart_core::grammar::PanelKey;
use chart_core::inspection::SelectionRegion;
use chart_core::navigation::{Navigation, NavigationBoundary, Navigator};
use chart_core::state::{CancelReason, GestureKind, GesturePreview, SelectionChange};
use chart_core::{DiagnosticCode, ScaleId};
use gpui::{MouseDownEvent, MouseMoveEvent, MouseUpEvent, ScrollWheelEvent};
use std::collections::BTreeSet;

/// Remappable primary drag behavior. Modifier selection operations are captured at gesture start.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NativeDragTool {
    /// Pan both primary dimensions; a click selects one target.
    #[default]
    Pan,
    /// Zoom to a dragged rectangle.
    ZoomRegion,
    /// Select centers/intersections in a rectangle.
    Rectangle,
    /// Select an x range.
    XRange,
    /// Select a y range.
    YRange,
    /// Select with a bounded closed lasso.
    Lasso,
}
struct Drag {
    id: Revision,
    start: Point,
    current: Point,
    origin: gpui::Point<Pixels>,
    tool: NativeDragTool,
    inspector: Inspector,
    axes: Vec<ScaleId>,
    panel: Option<PanelKey>,
    change: SelectionChange,
    selection: BTreeSet<MarkTarget>,
    points: Vec<Point>,
    moved: bool,
}
/// Native gesture ownership is bounded and retains the same presented basis as the reducer.
#[derive(Default)]
pub(super) struct InputState {
    drag: Option<Drag>,
    pub(super) edit: Option<super::edit::EditDrag>,
    pub(super) annotation_tools: Vec<NativeAnnotationTool>,
    pub(super) focused_edit: Option<usize>,
    pub(super) tool: NativeDragTool,
    pub(super) disabled: bool,
    pub(super) focused: bool,
    pub(super) subscriptions: Vec<gpui::Subscription>,
}
fn local(position: gpui::Point<Pixels>, origin: gpui::Point<Pixels>) -> ChartResult<Point> {
    Point::new(
        f64::from(f32::from(position.x - origin.x)),
        f64::from(f32::from(position.y - origin.y)),
    )
}
fn inside(rect: Rect, p: Point) -> bool {
    p.x() >= rect.origin().x()
        && p.x() <= rect.max_x()
        && p.y() >= rect.origin().y()
        && p.y() <= rect.max_y()
}
fn panel_axes(
    chart: &chart_core::layout::LaidOutChart,
    p: Point,
) -> Option<(Option<PanelKey>, Vec<ScaleId>)> {
    let (panel, chart) = if chart.panels().is_empty() {
        (None, chart)
    } else {
        let panel = chart
            .panels()
            .iter()
            .find(|v| v.chart.plot().is_some_and(|r| inside(r, p)))?;
        (Some(panel.key.clone()), panel.chart.as_ref())
    };
    if !chart.plot().is_some_and(|r| inside(r, p)) {
        return None;
    }
    let ids = chart
        .axes()
        .iter()
        .filter(|(_, a)| !matches!(a.scale, chart_core::layout::ResolvedScale::Secondary { .. }))
        .map(|(id, _)| *id)
        .collect();
    Some((panel, ids))
}
impl ChartView {
    /// Remap drag behavior from a host toolbar; cancels an active gesture before replacing its tool.
    pub fn set_drag_tool(
        &mut self,
        tool: NativeDragTool,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        self.cancel_input(CancelReason::Explicit, cx)?;
        self.input.tool = tool;
        cx.notify();
        Ok(())
    }
    /// Disable the default pointer/keyboard bindings when a host supplies its own mapping.
    pub fn set_default_bindings(
        &mut self,
        enabled: bool,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if !enabled {
            self.cancel_input(CancelReason::Explicit, cx)?;
        }
        self.input.disabled = !enabled;
        Ok(())
    }
    pub(super) fn has_drag(&self) -> bool {
        self.input.drag.is_some() || self.input.edit.is_some()
    }
    pub(super) fn release_input(&mut self) {
        self.input.drag = None;
        self.input.edit = None;
    }
    pub(super) fn cancel_input(
        &mut self,
        reason: CancelReason,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if self.state().active_gesture().is_some() {
            let r = self.action_request(ChartAction::CancelGesture(reason), ActionOrigin::Pointer);
            self.dispatch_action(r, cx)?;
        }
        self.input.drag = None;
        self.input.edit = None;
        Ok(())
    }
    pub(super) fn pointer_down(
        &mut self,
        event: &MouseDownEvent,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if self.input.disabled || self.state().active_gesture().is_some() {
            return Ok(());
        }
        if self.edit_down(event, cx)? {
            return Ok(());
        }
        let Some(inspector) = self.inspector.clone() else {
            return Ok(());
        };
        let Some(frame) = &self.frame else {
            return Ok(());
        };
        let origin = frame.bounds.origin;
        let start = local(event.position, origin)?;
        let Some((panel, axes)) = panel_axes(inspector.presented(), start) else {
            return Ok(());
        };
        let tool = self.input.tool;
        let kind = if matches!(tool, NativeDragTool::Pan | NativeDragTool::ZoomRegion) {
            GestureKind::Viewport
        } else {
            GestureKind::Selection
        };
        let id = self.chart.reducer().next_gesture_id()?;
        let change = if event.modifiers.platform || event.modifiers.control {
            SelectionChange::Toggle
        } else if event.modifiers.shift {
            SelectionChange::Add
        } else {
            SelectionChange::Replace
        };
        let selection = self.state().selection().clone();
        let request = self.action_request(
            ChartAction::BeginGesture { id, kind },
            ActionOrigin::Pointer,
        );
        self.dispatch_action(request, cx)?;
        self.input.drag = Some(Drag {
            id,
            start,
            current: start,
            origin,
            tool,
            inspector,
            axes,
            panel,
            change,
            selection,
            points: vec![start],
            moved: false,
        });
        cx.stop_propagation();
        Ok(())
    }
    pub(super) fn pointer_move(
        &mut self,
        event: &MouseMoveEvent,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if !self.has_drag() {
            return Ok(());
        }
        if self.input.edit.is_some() {
            return self.edit_move(event, cx);
        }
        if event.pressed_button != Some(MouseButton::Left) {
            return self.cancel_input(CancelReason::CaptureLost, cx);
        }
        let drag = self.input.drag.as_mut().expect("active input");
        let p = local(event.position, drag.origin)?;
        if !drag.moved && (p.x() - drag.start.x()).hypot(p.y() - drag.start.y()) < 3. {
            return Ok(());
        }
        drag.moved = true;
        drag.current = p;
        let stamp = drag.inspector.presented().scene().stamp();
        let preview = if matches!(drag.tool, NativeDragTool::Pan | NativeDragTool::ZoomRegion) {
            let action = if drag.tool == NativeDragTool::Pan {
                Navigation::Pan {
                    dx: p.x() - drag.start.x(),
                    dy: p.y() - drag.start.y(),
                }
            } else {
                Navigation::Region {
                    from: drag.start,
                    to: p,
                }
            };
            GesturePreview::AxisWindows(
                Navigator::new(drag.inspector.presented().clone()).navigate(
                    stamp,
                    &drag.axes,
                    drag.panel.as_ref(),
                    action,
                    NavigationBoundary::Extend,
                )?,
            )
        } else {
            let region = match drag.tool {
                NativeDragTool::XRange => SelectionRegion::XRange(drag.start.x(), p.x()),
                NativeDragTool::YRange => SelectionRegion::YRange(drag.start.y(), p.y()),
                NativeDragTool::Lasso => {
                    if drag
                        .points
                        .last()
                        .is_none_or(|last| (p.x() - last.x()).hypot(p.y() - last.y()) >= 2.)
                    {
                        if drag.points.len() == 4096 {
                            return Err(Diagnostic::error(
                                DiagnosticCode::ResourceLimit,
                                "Lasso exceeds 4096 vertices.",
                                "Release or cancel the gesture.",
                            ));
                        }
                        drag.points.push(p);
                    }
                    if drag.points.len() < 3 {
                        return Ok(());
                    }
                    SelectionRegion::Lasso(drag.points.clone())
                }
                _ => SelectionRegion::Rectangle(Rect::new(
                    drag.start.x().min(p.x()),
                    drag.start.y().min(p.y()),
                    (p.x() - drag.start.x()).abs(),
                    (p.y() - drag.start.y()).abs(),
                )?),
            };
            let targets = drag.inspector.select(stamp, &region, 4096)?;
            let mut selection = if drag.change == SelectionChange::Replace {
                BTreeSet::new()
            } else {
                drag.selection.clone()
            };
            for target in targets {
                if drag.change != SelectionChange::Toggle || !selection.remove(&target) {
                    selection.insert(target);
                }
            }
            GesturePreview::Selection(selection.into_iter().collect())
        };
        let id = drag.id;
        let request = self.action_request(
            ChartAction::PreviewGesture { id, preview },
            ActionOrigin::Pointer,
        );
        self.dispatch_action(request, cx)?;
        cx.stop_propagation();
        Ok(())
    }
    pub(super) fn pointer_up(
        &mut self,
        event: &MouseUpEvent,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if event.button != MouseButton::Left || !self.has_drag() {
            return Ok(());
        }
        if self.input.edit.is_some() {
            return self.edit_up(event, cx);
        }
        self.pointer_move(
            &MouseMoveEvent {
                position: event.position,
                pressed_button: Some(MouseButton::Left),
                modifiers: event.modifiers,
            },
            cx,
        )?;
        let drag = self.input.drag.as_ref().expect("active input");
        let id = drag.id;
        if drag.moved {
            let request =
                self.action_request(ChartAction::CommitGesture { id }, ActionOrigin::Pointer);
            self.dispatch_action(request, cx)?;
        } else {
            let targets = drag.inspector.select(
                drag.inspector.presented().scene().stamp(),
                &SelectionRegion::Point(drag.start),
                4096,
            )?;
            let change = drag.change;
            self.cancel_input(CancelReason::Explicit, cx)?;
            let request = self.action_request(
                ChartAction::Select { change, targets },
                ActionOrigin::Pointer,
            );
            self.dispatch_action(request, cx)?;
        }
        self.input.drag = None;
        self.input.edit = None;
        cx.stop_propagation();
        Ok(())
    }
    pub(super) fn scroll_input(
        &mut self,
        event: &ScrollWheelEvent,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        if self.input.disabled
            || !self.focus.is_focused(window)
            || self.state().active_gesture().is_some()
        {
            return Ok(());
        }
        let Some(frame) = &self.frame else {
            return Ok(());
        };
        let anchor = local(event.position, frame.bounds.origin)?;
        let Some((panel, axes)) = panel_axes(&frame.chart, anchor) else {
            return Ok(());
        };
        let delta = event.delta.pixel_delta(px(24.));
        let factor = (f64::from(f32::from(delta.y)) * 0.005).clamp(-3., 3.).exp();
        let windows = Navigator::new(frame.chart.clone()).navigate(
            frame.chart.scene().stamp(),
            &axes,
            panel.as_ref(),
            Navigation::Zoom { anchor, factor },
            NavigationBoundary::Extend,
        )?;
        let request =
            self.action_request(ChartAction::SetAxisWindows(windows), ActionOrigin::Pointer);
        self.dispatch_action(request, cx)?;
        cx.stop_propagation();
        Ok(())
    }
}

impl ChartView {
    pub(super) fn paint_input(&self, window: &mut Window) -> ChartResult<()> {
        let (Some(frame), Some(inspector)) = (&self.frame, &self.inspector) else {
            return Ok(());
        };
        let selected = match self
            .state()
            .active_gesture()
            .and_then(|g| g.preview.as_ref())
        {
            Some(GesturePreview::Selection(v)) => v.clone(),
            _ => self.state().selection().iter().cloned().collect(),
        };
        let mut inspection: Vec<_> = self.state().hover().iter().cloned().collect();
        inspection.extend(self.state().focus().cloned());
        let selected = inspector.highlights(&selected)?;
        let inspection = inspector.highlights(&inspection)?;
        let destination = |r: Rect| {
            gpui::Bounds::new(
                gpui::point(
                    frame.bounds.origin.x + px(r.origin().x() as f32),
                    frame.bounds.origin.y + px(r.origin().y() as f32),
                ),
                gpui::size(px(r.width() as f32), px(r.height() as f32)),
            )
        };
        window.with_content_mask(
            Some(gpui::ContentMask {
                bounds: frame.bounds,
            }),
            |window| {
                for (rects, color) in [(&selected, 0x2167c8), (&inspection, 0xc87121)] {
                    for rect in rects {
                        window.paint_quad(gpui::outline(
                            destination(*rect),
                            rgb(color),
                            gpui::BorderStyle::Solid,
                        ));
                    }
                }
                if let Some(drag) = &self.input.drag
                    && drag.moved
                    && drag.tool != NativeDragTool::Pan
                {
                    let points = if drag.tool == NativeDragTool::Lasso {
                        drag.points.clone()
                    } else {
                        let bounds = drag
                            .inspector
                            .presented()
                            .plot()
                            .unwrap_or(drag.inspector.presented().scene().bounds());
                        let (a, b) = match drag.tool {
                            NativeDragTool::XRange => (
                                Point::new(drag.start.x(), bounds.origin().y())?,
                                Point::new(drag.current.x(), bounds.max_y())?,
                            ),
                            NativeDragTool::YRange => (
                                Point::new(bounds.origin().x(), drag.start.y())?,
                                Point::new(bounds.max_x(), drag.current.y())?,
                            ),
                            _ => (drag.start, drag.current),
                        };
                        vec![a, Point::new(b.x(), a.y())?, b, Point::new(a.x(), b.y())?]
                    };
                    if let Some(first) = points.first() {
                        let point = |p: Point| {
                            gpui::point(
                                drag.origin.x + px(p.x() as f32),
                                drag.origin.y + px(p.y() as f32),
                            )
                        };
                        let mut path = gpui::PathBuilder::stroke(px(1.5));
                        path.move_to(point(*first));
                        for p in &points[1..] {
                            path.line_to(point(*p));
                        }
                        path.close();
                        if let Ok(path) = path.build() {
                            window.paint_path(path, rgb(0x2167c8));
                        }
                    }
                }
                Ok(())
            },
        )
    }
}
