mod input;
use crate::native::{NativeFont, NativeFrame};
use chart_core::data::{SnapshotHandle, StoreSnapshot};
use chart_core::grammar::{ChartDefinition, CompileLimits, Compiler, PreparedChart};
use chart_core::inspection::{InputOrigin, InspectionAction, Inspector};
use chart_core::layout::LayoutRequest;
use chart_core::services::Units;
use chart_core::state::{
    ActionOrigin, ActionReducer, ActionRequest, ChartAction, ChartState, DispatchOutcome,
    MarkTarget,
};
use chart_core::{ChartResult, Diagnostic, Point, Rect, Revision};
use gpui::{
    AnyElement, App, Bounds, Context, FocusHandle, IntoElement, MouseButton, Pixels, Render, Role,
    Window, canvas, div, prelude::*, px, rgb,
};
pub use input::NativeDragTool;
use std::{rc::Rc, sync::Arc};

/// Caller-owned native tooltip body. The inspector pins the exact source snapshot;
/// aggregated targets remain explicit. Native elements have no implicit export representation.
pub type TooltipBuilder = Rc<dyn Fn(&Inspector, &mut Window, &mut App) -> AnyElement>;
/// Observable retained-work counts; these are not performance timings.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NativeMetrics {
    /// Layout/paint-data preparation attempts (hover-only redraws reuse the frame).
    pub layout_attempts: u64,
    /// Successful native paint submissions.
    pub paints: u64,
}
/// Validated, owned input for a native chart mount. Construct before creating the GPUI entity.
pub struct ChartInput {
    definition: ChartDefinition,
    source: SnapshotHandle<StoreSnapshot>,
    state: ChartState,
    compiler: Compiler,
    prepared: Arc<PreparedChart>,
    font: NativeFont,
    painters: Rc<crate::NativePainterRegistry>,
    request: LayoutRequest,
}
impl ChartInput {
    /// Compile once with recoverable errors, retaining the supplied immutable data snapshot.
    pub fn new(
        definition: ChartDefinition,
        source: SnapshotHandle<StoreSnapshot>,
        font: NativeFont,
    ) -> ChartResult<Self> {
        Self::with_extensions(
            definition,
            source,
            font,
            Arc::new(chart_core::grammar::ExtensionRegistry::new()),
        )
    }
    /// Compile with explicitly supplied versioned native/portable stat and geometry extensions.
    pub fn with_extensions(
        definition: ChartDefinition,
        source: SnapshotHandle<StoreSnapshot>,
        font: NativeFont,
        extensions: Arc<chart_core::grammar::ExtensionRegistry>,
    ) -> ChartResult<Self> {
        let state = ChartState::default();
        let mut compiler = Compiler::with_extensions(extensions);
        let prepared =
            Arc::new(compiler.prepare(&definition, &source, &state, CompileLimits::default())?);
        let request = LayoutRequest::new(
            Rect::new(0., 0., 400., 240.)?,
            Units::LogicalPixels,
            font.descriptor(),
        );
        Ok(Self {
            definition,
            source,
            state,
            compiler,
            prepared,
            font,
            painters: Rc::new(crate::NativePainterRegistry::new()),
            request,
        })
    }
    /// Retain exact host-only painter implementations for this mount and its prepared frames.
    pub fn with_native_painters(mut self, painters: Rc<crate::NativePainterRegistry>) -> Self {
        self.painters = painters;
        self
    }
}

/// One retained chart entity with bounded pane/overlay elements and one cached native frame.
/// Datasets are supplied as immutable snapshots, never recreated inside Render.
pub struct ChartView {
    definition: ChartDefinition,
    source: SnapshotHandle<StoreSnapshot>,
    reducer: ActionReducer,
    compiler: Compiler,
    prepared: Arc<PreparedChart>,
    font: NativeFont,
    painters: Rc<crate::NativePainterRegistry>,
    request: LayoutRequest,
    focus: FocusHandle,
    frame: Option<Rc<NativeFrame>>,
    cached: Option<Rc<NativeFrame>>,
    attempted: Option<(Bounds<Pixels>, Revision, usize)>,
    inspector: Option<Inspector>,
    tooltip: Option<TooltipBuilder>,
    last_error: Option<Diagnostic>,
    metrics: NativeMetrics,
    input: input::InputState,
}
impl ChartView {
    /// Mount an already validated input; no source preparation or fallible work is hidden here.
    pub fn new(input: ChartInput, cx: &mut Context<Self>) -> Self {
        let ChartInput {
            definition,
            source,
            state,
            compiler,
            prepared,
            font,
            painters,
            request,
        } = input;
        Self {
            definition,
            source,
            reducer: ActionReducer::new(state),
            compiler,
            prepared,
            font,
            painters,
            request,
            focus: cx.focus_handle(),
            frame: None,
            cached: None,
            attempted: None,
            inspector: None,
            tooltip: None,
            last_error: None,
            metrics: NativeMetrics::default(),
            input: input::InputState::default(),
        }
    }
    /// Change the native-only inspection body, with no data/stat/layout invalidation.
    pub fn set_tooltip(&mut self, builder: TooltipBuilder, cx: &mut Context<Self>) {
        self.tooltip = Some(builder);
        cx.notify();
    }
    /// Replace source data atomically after grammar validation. A failure preserves inputs/frame.
    pub fn set_data(
        &mut self,
        source: SnapshotHandle<StoreSnapshot>,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        let prepared = self
            .compiler
            .prepare(
                &self.definition,
                &source,
                self.reducer.state(),
                CompileLimits::default(),
            )
            .inspect_err(|e| {
                self.last_error = Some(e.clone());
                cx.notify();
            })?;
        self.source = source;
        self.prepared = Arc::new(prepared);
        self.last_error = None;
        cx.notify();
        Ok(())
    }
    /// Replace the authoring definition only after successful preparation.
    pub fn set_definition(
        &mut self,
        definition: ChartDefinition,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        let mut next = self.reducer.clone();
        if next.state().active_gesture().is_some() {
            let request = next.request(
                &self.definition,
                ChartAction::CancelGesture(chart_core::state::CancelReason::TargetRemoved),
                ActionOrigin::Programmatic,
            );
            next.dispatch(&self.definition, request)?;
        }
        let prepared = self
            .compiler
            .prepare(
                &definition,
                &self.source,
                next.state(),
                CompileLimits::default(),
            )
            .inspect_err(|e| {
                self.last_error = Some(e.clone());
                cx.notify();
            })?;
        self.reducer = next;
        if self.state().active_gesture().is_none() {
            self.release_input();
        }
        self.definition = definition;
        self.prepared = Arc::new(prepared);
        self.last_error = None;
        cx.notify();
        Ok(())
    }
    /// Set destination policies. Bounds follow the actual element; owner revision is advanced.
    /// Font/layout failures retain the previously painted frame with an explicit diagnostic.
    pub fn set_layout(
        &mut self,
        mut request: LayoutRequest,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        request.revision = self.request.revision.checked_next()?;
        self.request = request;
        self.attempted = None;
        cx.notify();
        Ok(())
    }
    /// Current destination policy template (actual bounds are supplied in prepaint).
    pub fn layout_request(&self) -> &LayoutRequest {
        &self.request
    }
    /// Dispatch caller controls/programmatic actions through the retained core reducer.
    pub fn dispatch_chart(
        &mut self,
        action: ChartAction,
        cx: &mut Context<Self>,
    ) -> ChartResult<bool> {
        let request = self
            .reducer
            .request(&self.definition, action, ActionOrigin::Programmatic);
        Ok(self.dispatch_action(request, cx)?.outcome.changed)
    }
    /// Current semantic state, including distinct component revisions.
    pub fn state(&self) -> &ChartState {
        self.reducer.state()
    }
    /// Next valid identity shared by native gestures and host editing controls.
    pub fn next_gesture_id(&self) -> ChartResult<Revision> {
        self.reducer.next_gesture_id()
    }
    /// Prepare a control/input request using current state and the presented or pinned basis.
    pub fn action_request(&self, action: ChartAction, origin: ActionOrigin) -> ActionRequest {
        self.reducer.request(&self.definition, action, origin)
    }
    /// Exact pinned axes/source for host gesture coordinate conversion, never pending geometry.
    pub fn gesture_basis(&self) -> Option<&Arc<chart_core::layout::LaidOutChart>> {
        self.reducer.gesture_basis()
    }
    /// Apply a current controlled response atomically with required preparation.
    pub fn accept_controlled(
        &mut self,
        expected: Revision,
        state: ChartState,
        cx: &mut Context<Self>,
    ) -> ChartResult<bool> {
        let mut next = self.reducer.clone();
        if !next.accept_controlled(&self.definition, expected, state)? {
            return Ok(false);
        }
        let prepared = self.compiler.prepare(
            &self.definition,
            &self.source,
            next.state(),
            CompileLimits::default(),
        )?;
        self.reducer = next;
        if self.state().active_gesture().is_none() {
            self.release_input();
        }
        self.prepared = Arc::new(prepared);
        cx.notify();
        Ok(true)
    }
    /// Full origin/scene/revision-fenced path. Failed preparation leaves reducer/history intact.
    pub fn dispatch_action(
        &mut self,
        request: ActionRequest,
        cx: &mut Context<Self>,
    ) -> ChartResult<DispatchOutcome> {
        let mut next = self.reducer.clone();
        let result = next.dispatch(&self.definition, request)?;
        if result
            .event
            .as_ref()
            .is_some_and(|e| e.presentation_changed)
        {
            let prepared = self.compiler.prepare(
                &self.definition,
                &self.source,
                next.state(),
                CompileLimits::default(),
            )?;
            self.prepared = Arc::new(prepared);
        }
        self.reducer = next;
        if self.state().active_gesture().is_none() {
            self.release_input();
        }
        if result.outcome.changed {
            cx.notify();
        }
        Ok(result)
    }
    /// Resolve native input using the exact presented inspector, then dispatch semantic actions.
    pub fn dispatch_inspection(
        &mut self,
        action: InspectionAction,
        origin: InputOrigin,
        cx: &mut Context<Self>,
    ) -> ChartResult<bool> {
        let Some(mut inspector) = self.inspector.clone() else {
            return Ok(false);
        };
        let stamp = inspector.presented().scene().stamp();
        let inspected = inspector.dispatch(stamp, action, origin)?;
        let epoch = inspector.presented().prepared().source().get()?.epoch();
        let targets: Vec<_> = inspector
            .hits()
            .iter()
            .map(|h| MarkTarget::from_inspected(h, epoch))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let semantic = match action {
            InspectionAction::Hover(_) => ChartAction::SetHover(targets),
            InspectionAction::StepFocus { .. } => ChartAction::SetFocus(targets.into_iter().next()),
            InspectionAction::Clear => ChartAction::ClearInspection,
        };
        let origin = match origin {
            InputOrigin::Pointer => ActionOrigin::Pointer,
            InputOrigin::Keyboard => ActionOrigin::Keyboard,
            InputOrigin::Programmatic => ActionOrigin::Programmatic,
        };
        let mut request = self.reducer.request(&self.definition, semantic, origin);
        request.scene = Some(stamp);
        let outcome = self.dispatch_action(request, cx)?;
        self.inspector = Some(inspector);
        if inspected.changed {
            cx.notify();
        }
        Ok(outcome.outcome.changed || inspected.changed)
    }
    /// Last successfully painted inspection snapshot; never a pending preparation.
    pub fn inspector(&self) -> Option<&Inspector> {
        self.inspector.as_ref()
    }
    /// Recoverable preparation/paint diagnostic, if any.
    pub fn diagnostic(&self) -> Option<&Diagnostic> {
        self.last_error.as_ref()
    }
    /// Retained-work counters for host validation.
    pub fn metrics(&self) -> NativeMetrics {
        self.metrics
    }
    fn prepaint(&mut self, bounds: Bounds<Pixels>, window: &Window) -> Option<Rc<NativeFrame>> {
        if self.reducer.frozen_scene().is_some() {
            return self.frame.clone();
        }
        let key = (
            bounds,
            self.request.revision,
            Arc::as_ptr(&self.prepared) as usize,
        );
        if self.attempted.as_ref() == Some(&key) {
            return self.cached.clone();
        }
        self.attempted = Some(key);
        self.metrics.layout_attempts = self.metrics.layout_attempts.saturating_add(1);
        let revision = self.request.revision.checked_next();
        let result = revision.and_then(|revision| {
            self.request.revision = revision;
            self.attempted = Some((bounds, revision, Arc::as_ptr(&self.prepared) as usize));
            NativeFrame::prepare(
                self.prepared.clone(),
                self.request.clone(),
                &self.font,
                &self.painters,
                bounds,
                window,
            )
        });
        match result {
            Ok(frame) => {
                self.last_error = None;
                self.cached = Some(Rc::new(frame));
                self.cached.clone()
            }
            Err(e) => {
                self.last_error = Some(e);
                if self.frame.as_ref().is_some_and(|f| f.bounds != bounds) {
                    self.inspector = None;
                }
                self.cached = self.frame.clone();
                self.cached.clone()
            }
        }
    }
}
impl Render for ChartView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus.contains_focused(window, cx);
        if self.input.focused && !focused {
            if let Err(e) = self.cancel_input(chart_core::state::CancelReason::FocusLost, cx) {
                self.last_error = Some(e);
            }
            let _ = self.dispatch_inspection(InspectionAction::Clear, InputOrigin::Keyboard, cx);
        }
        self.input.focused = focused;
        let mut tokens = self
            .definition
            .theme
            .as_ref()
            .and_then(|t| t.resolve(&self.request.host_theme).ok())
            .unwrap_or_else(|| self.request.host_theme.clone());
        tokens.overlay(&self.request.output_theme);
        let focus_color = chart_core::theme::paint_color(
            tokens.focus.unwrap_or(chart_core::theme::rgb(59, 130, 196)),
            tokens.color_mode,
        );
        let focus_color = gpui::rgba(
            (u32::from(focus_color.red) << 24)
                | (u32::from(focus_color.green) << 16)
                | (u32::from(focus_color.blue) << 8)
                | u32::from(focus_color.alpha),
        );
        if self.input.subscriptions.is_empty() {
            let focus = self.focus.clone();
            self.input
                .subscriptions
                .push(cx.on_focus_out(&focus, window, |this, _, _, cx| {
                    if let Err(e) =
                        this.cancel_input(chart_core::state::CancelReason::FocusLost, cx)
                    {
                        this.last_error = Some(e);
                    }
                    let _ = this.dispatch_inspection(
                        InspectionAction::Clear,
                        InputOrigin::Keyboard,
                        cx,
                    );
                }));
            self.input.subscriptions.push(cx.observe_window_activation(
                window,
                |this, window, cx| {
                    if !window.is_window_active()
                        && let Err(e) =
                            this.cancel_input(chart_core::state::CancelReason::CaptureLost, cx)
                    {
                        this.last_error = Some(e);
                    }
                },
            ));
        }
        let prepaint = cx.entity().downgrade();
        let paint = prepaint.clone();
        let diagnostic = self.last_error.as_ref().map(|e| {
            div()
                .id("chart-error")
                .role(Role::Status)
                .absolute()
                .bottom_0()
                .left_0()
                .p_2()
                .bg(rgb(0xffeeee))
                .text_color(rgb(0x9b3030))
                .child(format!("Chart update unavailable: {}", e.message))
                .into_any_element()
        });
        let chart_canvas = canvas(
            move |bounds, window, cx| {
                prepaint
                    .update(cx, |this, cx| {
                        let before = this.last_error.clone();
                        let frame = this.prepaint(bounds, window);
                        if this.last_error != before {
                            let weak = cx.entity().downgrade();
                            window.on_next_frame(move |_, cx| {
                                let _ = weak.update(cx, |_, cx| cx.notify());
                            });
                        }
                        let tooltip = frame
                            .as_ref()
                            .and_then(|frame| {
                                this.inspector.as_ref().filter(|i| {
                                    !i.hits().is_empty() && Arc::ptr_eq(i.presented(), &frame.chart)
                                })
                            })
                            .and_then(|i| this.tooltip.as_ref().map(|f| f(i, window, cx)))
                            .map(|body| {
                                div()
                                    .p_3()
                                    .bg(rgb(0xf0f5fa))
                                    .rounded_md()
                                    .child(body)
                                    .into_any_element()
                            });
                        (frame, tooltip)
                    })
                    .ok()
                    .map(|(frame, mut tooltip)| {
                        if let Some(body) = &mut tooltip {
                            let available = gpui::size(
                                gpui::AvailableSpace::Definite(
                                    (bounds.size.width - px(24.)).max(px(0.)).min(px(360.)),
                                ),
                                gpui::AvailableSpace::MinContent,
                            );
                            let size = body.layout_as_root(available, window, cx);
                            body.prepaint_at(
                                gpui::point(
                                    bounds.origin.x
                                        + (bounds.size.width - size.width - px(12.)).max(px(0.)),
                                    bounds.origin.y + px(12.),
                                ),
                                window,
                                cx,
                            );
                        }
                        (frame, tooltip)
                    })
            },
            move |_,
                  prepared: Option<(Option<Rc<NativeFrame>>, Option<AnyElement>)>,
                  window,
                  cx| {
                if let Some((Some(frame), mut tooltip)) = prepared {
                    let result = frame.paint(window, cx);
                    let painted = result.is_ok();
                    let _ = paint.update(cx, |this, cx| match result {
                        Ok(()) => {
                            if this
                                .frame
                                .as_ref()
                                .is_none_or(|old| !Rc::ptr_eq(old, &frame))
                            {
                                this.reducer.present(frame.chart.clone());
                                let mut inspector =
                                    Inspector::new(frame.chart.clone(), 10., 32).ok();
                                if let Some(focus) = this.state().focus().cloned() {
                                    let restored = inspector
                                        .as_mut()
                                        .is_some_and(|i| i.restore_focus(&focus).unwrap_or(false));
                                    if !restored {
                                        let request = this.action_request(
                                            ChartAction::SetFocus(None),
                                            ActionOrigin::Programmatic,
                                        );
                                        let _ = this.dispatch_action(request, cx);
                                    }
                                }
                                if !this.state().hover().is_empty() {
                                    let request = this.action_request(
                                        ChartAction::SetHover(vec![]),
                                        ActionOrigin::Programmatic,
                                    );
                                    let _ = this.dispatch_action(request, cx);
                                }
                                this.inspector = inspector;
                                this.frame = Some(frame);
                                let weak = cx.entity().downgrade();
                                window.on_next_frame(move |_, cx| {
                                    let _ = weak.update(cx, |_, cx| cx.notify());
                                });
                            }
                            this.metrics.paints = this.metrics.paints.saturating_add(1);
                        }
                        Err(e) => {
                            this.last_error = Some(e);
                            this.inspector = None;
                        }
                    });
                    let _ = paint.update(cx, |this, _| {
                        if let Err(e) = this.paint_input(window) {
                            this.last_error = Some(e);
                        }
                    });
                    let move_owner = paint.clone();
                    window.on_mouse_event(move |event: &gpui::MouseMoveEvent, phase, _, cx| {
                        if phase == gpui::DispatchPhase::Capture {
                            let _ = move_owner.update(cx, |this, cx| {
                                if let Err(e) = this.pointer_move(event, cx) {
                                    this.last_error = Some(e);
                                    let _ = this.cancel_input(
                                        chart_core::state::CancelReason::Explicit,
                                        cx,
                                    );
                                }
                            });
                        }
                    });
                    let up_owner = paint.clone();
                    window.on_mouse_event(move |event: &gpui::MouseUpEvent, phase, _, cx| {
                        if phase == gpui::DispatchPhase::Capture {
                            let _ = up_owner.update(cx, |this, cx| {
                                if let Err(e) = this.pointer_up(event, cx) {
                                    this.last_error = Some(e);
                                    let _ = this.cancel_input(
                                        chart_core::state::CancelReason::Explicit,
                                        cx,
                                    );
                                }
                            });
                        }
                    });
                    if painted && let Some(body) = &mut tooltip {
                        body.paint(window, cx);
                    }
                }
            },
        )
        .size_full();
        div()
            .id("native-chart")
            .relative()
            .size_full()
            .overflow_hidden()
            .border_1()
            .border_color(if self.focus.is_focused(window) { focus_color } else { gpui::rgba(0x00000000) })
            .track_focus(&self.focus)
            .hover_listener_mode(gpui::HoverListenerMode::InputModalityIndependent)
            .role(Role::Image)
            .aria_label("Interactive chart. Click to focus; arrow keys inspect visible observations; Escape clears inspection.")
            .on_mouse_down(MouseButton::Left, cx.listener(|this, event, window, cx| {
                window.focus(&this.focus, cx);
                if let Err(e)=this.pointer_down(event,cx){this.last_error=Some(e);}
                cx.notify();
            }))
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if this.has_drag() || this.input.disabled {return;}
                let Some(frame) = &this.frame else { return };
                let local = Point::new(
                    f64::from(f32::from(event.position.x - frame.bounds.origin.x)),
                    f64::from(f32::from(event.position.y - frame.bounds.origin.y)),
                );
                if let Ok(p) = local
                    && let Err(e) = this.dispatch_inspection(
                        InspectionAction::Hover(Some(p)), InputOrigin::Pointer, cx,
                    )
                {
                    this.last_error = Some(e);
                }
            }))
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                if !hovered && !this.has_drag() && !this.inspector.as_ref().is_some_and(Inspector::has_keyboard_focus) {
                    let _ = this.dispatch_inspection(InspectionAction::Clear, InputOrigin::Pointer, cx);
                }
            }))
            .on_scroll_wheel(cx.listener(|this,event,window,cx| {
                if let Err(e)=this.scroll_input(event,window,cx){this.last_error=Some(e);cx.notify();}
            }))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                if this.input.disabled{return;}
                if event.keystroke.key == "escape" && this.state().active_gesture().is_some() {
                    if let Err(e)=this.cancel_input(chart_core::state::CancelReason::Explicit,cx){this.last_error=Some(e);}
                    cx.stop_propagation(); return;
                }
                if this.state().active_gesture().is_some(){return;}
                if event.keystroke.key=="home" {
                    let r=this.action_request(ChartAction::Reset,ActionOrigin::Keyboard);
                    if let Err(e)=this.dispatch_action(r,cx){this.last_error=Some(e);}
                    cx.stop_propagation();return;
                }
                if event.keystroke.key=="space" {
                    if let Some(focus)=this.state().focus().cloned() {
                        let r=this.action_request(ChartAction::Select {change:chart_core::state::SelectionChange::Toggle,targets:vec![focus]},ActionOrigin::Keyboard);
                        if let Err(e)=this.dispatch_action(r,cx){this.last_error=Some(e);}
                    }
                    cx.stop_propagation();return;
                }
                let action = match event.keystroke.key.as_str() {
                    "right" | "down" => Some(InspectionAction::StepFocus { forward: true }),
                    "left" | "up" => Some(InspectionAction::StepFocus { forward: false }),
                    "escape" => Some(InspectionAction::Clear),
                    _ => None,
                };
                if let Some(action) = action {
                    if let Err(e) = this.dispatch_inspection(action, InputOrigin::Keyboard, cx) {
                        this.last_error = Some(e);
                    }
                    cx.stop_propagation();
                }
            }))
            .child(chart_canvas)
            .children(diagnostic)
    }
}

impl gpui::Focusable for ChartView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
