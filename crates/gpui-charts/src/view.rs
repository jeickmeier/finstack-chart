mod edit;
mod host;
mod input;
mod scheduling;
use crate::native::{NativeFont, NativeFrame};
use chart_core::data::{SnapshotHandle, StoreSnapshot};
use chart_core::grammar::{ChartDefinition, CompileLimits, Compiler, PreparedChart};
use chart_core::inspection::{InputOrigin, InspectionAction, Inspector};
use chart_core::layout::LayoutRequest;
use chart_core::services::Units;
use chart_core::state::{
    ActionOrigin, ActionRequest, ChartAction, ChartState, DispatchOutcome, MarkTarget,
};
use chart_core::{ChartResult, Diagnostic, Point, Rect, Revision};
pub use edit::NativeAnnotationTool;
use gpui::{
    AnyElement, App, Bounds, Context, FocusHandle, IntoElement, MouseButton, Pixels, Render, Role,
    Window, canvas, div, prelude::*, px, rgb,
};
pub use host::{ChartHostEvent, ControlBuilder, ControlSlot, HostCommand, HostContext};
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
    chart: chart_core::runtime::Chart,
    prepared: Arc<PreparedChart>,
    font: NativeFont,
    painters: Rc<crate::NativePainterRegistry>,
    request: LayoutRequest,
    tooltip: Option<TooltipBuilder>,
    host: host::HostState,
    input: input::InputState,
    density: chart_core::dense::DensityOptions,
}
impl ChartInput {
    /// Mount the same immutable primary plot used by headless publication.
    /// Native preparation retains the plot's source and exact extension registrations.
    pub fn from_plot(plot: &chart_core::plot::Plot, font: NativeFont) -> ChartResult<Self> {
        Self::from_chart(plot.chart()?, font)
    }
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
        Self::from_chart(
            chart_core::runtime::Chart::from_external(definition, source, extensions)?,
            font,
        )
    }
    /// Adopt one typed runtime, retaining its ingestion, replay, reducer and compiler ownership.
    pub fn from_chart(
        mut chart: chart_core::runtime::Chart,
        font: NativeFont,
    ) -> ChartResult<Self> {
        let prepared = chart.prepare()?;
        let request = LayoutRequest::new(
            Rect::new(0., 0., 400., 240.)?,
            Units::LogicalPixels,
            font.descriptor(),
        );
        Ok(Self {
            chart,
            prepared,
            font,
            painters: Rc::new(crate::NativePainterRegistry::new()),
            request,
            tooltip: None,
            host: Default::default(),
            input: Default::default(),
            density: Default::default(),
        })
    }
    /// Configure shared destination layout before mounting, retaining its supplied font/units.
    pub fn layout(mut self, options: chart_core::plot::LayoutOptions) -> Self {
        options.apply(&mut self.request);
        self
    }
    /// Retain exact host-only painter implementations for this mount and its prepared frames.
    pub fn with_native_painters(mut self, painters: Rc<crate::NativePainterRegistry>) -> Self {
        self.painters = painters;
        self
    }
    /// Configure the native inspection body before mounting.
    pub fn tooltip(mut self, builder: TooltipBuilder) -> Self {
        self.tooltip = Some(builder);
        self
    }
    /// Configure a replaceable native toolbar, legend control or context menu.
    pub fn control(mut self, slot: ControlSlot, builder: ControlBuilder) -> Self {
        self.host.control(slot, Some(builder));
        self
    }
    /// Enable operations handled by the application's ChartHostEvent subscription.
    pub fn host_commands(mut self, commands: &[HostCommand]) -> Self {
        self.host.commands(commands);
        self
    }
    /// Supply bounded meaningful accessibility context before the first frame.
    pub fn accessible_summary(mut self, summary: impl Into<String>) -> ChartResult<Self> {
        self.host.summary(Some(summary.into()))?;
        Ok(self)
    }
    /// Choose the initial shared navigation/selection drag tool.
    pub fn drag_tool(mut self, tool: NativeDragTool) -> Self {
        self.input.tool = tool;
        self
    }
    /// Enable or disable built-in native input bindings for an application-owned adapter.
    pub fn default_bindings(mut self, enabled: bool) -> Self {
        self.input.disabled = !enabled;
        self
    }
    /// Install bounded authored annotation edit handles.
    pub fn annotation_tools(mut self, tools: Vec<NativeAnnotationTool>) -> ChartResult<Self> {
        edit::validate_tools(&tools)?;
        self.input.annotation_tools = tools;
        Ok(self)
    }
    /// Configure source-preserving screen reduction before mounting.
    pub fn render(mut self, options: chart_core::plot::RenderOptions) -> ChartResult<Self> {
        self.density = options.build(&self.chart)?;
        Ok(self)
    }
}

/// Thread-transferable immutable presentation capture. Contains no GPUI window, painter,
/// text-system or entity handles. The exporter supplies its own publication measurement.
#[derive(Clone)]
pub struct PresentedCapture {
    /// Exact raw laid-out source underlying the last painted (possibly dense/frozen) scene.
    pub chart: Arc<chart_core::layout::LaidOutChart>,
    /// Painted geometry state plus the inspection overlays acknowledged by that paint.
    pub state: ChartState,
    /// Captured native layout/theme/resource policy; publication must choose point dimensions.
    pub layout: LayoutRequest,
    /// Exact supplied font resources, independent of the native text registry.
    pub fonts: Vec<(chart_core::services::ResourceDescriptor, Arc<[u8]>)>,
    /// Exact versioned portable extension implementations for publication preparation.
    pub extensions: Arc<chart_core::grammar::ExtensionRegistry>,
}
/// One retained chart entity with bounded pane/overlay elements and one cached native frame.
/// Datasets are supplied as immutable snapshots, never recreated inside Render.
pub struct ChartView {
    chart: chart_core::runtime::Chart,
    prepared: Arc<PreparedChart>,
    font: NativeFont,
    painters: Rc<crate::NativePainterRegistry>,
    request: LayoutRequest,
    focus: FocusHandle,
    frame: Option<Rc<NativeFrame>>,
    painted_state: Option<ChartState>,
    cached: Option<Rc<NativeFrame>>,
    attempted: Option<(Bounds<Pixels>, Revision, usize)>,
    inspector: Option<Inspector>,
    tooltip: Option<TooltipBuilder>,
    last_error: Option<Diagnostic>,
    metrics: NativeMetrics,
    density: chart_core::dense::DensityOptions,
    scheduling: scheduling::Scheduling,
    input: input::InputState,
    host: host::HostState,
}
impl ChartView {
    /// Mount an already validated input; no source preparation or fallible work is hidden here.
    pub fn new(input: ChartInput, cx: &mut Context<Self>) -> Self {
        let ChartInput {
            chart,
            prepared,
            font,
            painters,
            request,
            tooltip,
            host,
            input,
            density,
        } = input;
        let scheduling = scheduling::Scheduling::new(chart.extensions().clone());
        Self {
            scheduling,
            chart,
            prepared,
            font,
            painters,
            request,
            focus: cx.focus_handle(),
            frame: None,
            painted_state: None,
            cached: None,
            attempted: None,
            inspector: None,
            tooltip,
            last_error: None,
            metrics: NativeMetrics::default(),
            density,
            input,
            host,
        }
    }
    /// Explicit destination-only reduction; the current source/inspection values stay exact.
    pub fn set_density(
        &mut self,
        options: chart_core::dense::DensityOptions,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        options.validate()?;
        self.density = options;
        self.attempted = None;
        cx.notify();
        Ok(())
    }
    /// Configure screen-density reduction through the shared typed options.
    pub fn set_render_options(
        &mut self,
        options: chart_core::plot::RenderOptions,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        self.set_density(options.build(&self.chart)?, cx)
    }
    /// Actual last-presented raw/prepared/rendered work counts.
    pub fn density_metrics(&self) -> Option<&chart_core::dense::DensityMetrics> {
        self.frame.as_ref().map(|f| &f.density)
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
        let prepared = self.chart.accept_source_prepared(source).inspect_err(|e| {
            self.last_error = Some(e.clone());
            cx.notify();
        })?;
        self.reset_preparation(false, cx)?;
        if self.state().active_gesture().is_none() {
            self.release_input();
        }
        self.prepared = prepared;
        self.last_error = None;
        if let Some(reconciliation) = self.chart.reconciliation() {
            cx.emit(ChartHostEvent::DataReconciled(reconciliation.clone()));
        }
        cx.notify();
        Ok(())
    }
    /// Replace the authoring definition only after successful preparation.
    pub fn set_definition(
        &mut self,
        definition: ChartDefinition,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        self.prepared = self
            .chart
            .replace_definition_prepared(definition)
            .inspect_err(|e| {
                self.last_error = Some(e.clone());
                cx.notify();
            })?;
        if self.state().active_gesture().is_none() {
            self.release_input();
        }
        self.reset_preparation(true, cx)?;
        self.last_error = None;
        cx.notify();
        Ok(())
    }
    /// Read the same typed runtime used by owned ingestion, capture and host adapters.
    pub fn chart(&self) -> &chart_core::runtime::Chart {
        &self.chart
    }
    /// Capture a primary linked-view message from this retained chart's acknowledged scene.
    pub fn capture_link(
        &mut self,
        link: &chart_core::plot::LinkBuilder,
        event: &chart_core::state::StateEvent,
    ) -> ChartResult<Option<chart_core::linking::LinkMessage>> {
        link.capture(&mut self.chart, event)
    }
    /// Resolve a primary linked-view message; callers dispatch the returned typed action.
    pub fn resolve_link(
        &mut self,
        link: &chart_core::plot::LinkBuilder,
        message: &chart_core::linking::LinkMessage,
    ) -> ChartResult<chart_core::linking::LinkedUpdate> {
        link.resolve(&mut self.chart, message)
    }
    /// Apply an immutable definition edit while retaining the latest owned or external data.
    pub fn apply_plot(
        &mut self,
        plot: &chart_core::plot::Plot,
        expected: Revision,
        cx: &mut Context<Self>,
    ) -> ChartResult<bool> {
        let changed = self.chart.apply_plot(plot, expected)?;
        if changed {
            if self.state().active_gesture().is_none() {
                self.release_input();
            }
            if let Err(error) = self
                .reset_preparation(false, cx)
                .and_then(|()| self.queue_current(cx))
            {
                self.last_error = Some(error);
            }
            cx.notify();
        }
        Ok(changed)
    }
    /// Commit through this mount's owned runtime and enqueue its latest snapshot for preparation.
    pub fn commit(
        &mut self,
        transaction: chart_core::transaction::Transaction,
        cx: &mut Context<Self>,
    ) -> ChartResult<chart_core::transaction::CommitOutcome> {
        let outcome = self.chart.apply_transaction(transaction)?;
        if matches!(outcome, chart_core::transaction::CommitOutcome::Applied(_)) {
            self.schedule_committed(cx);
        }
        Ok(outcome)
    }
    /// Configure the owned queue; acceptance and synchronous commitment remain separate.
    pub fn stream(&mut self, options: chart_core::plot::StreamOptions) -> ChartResult<()> {
        self.chart.stream(options)
    }
    /// Enqueue a revision-fenced transaction without claiming it has committed or painted.
    pub fn enqueue(
        &mut self,
        transaction: chart_core::transaction::Transaction,
    ) -> ChartResult<chart_core::ingestion::EnqueueOutcome> {
        self.chart.enqueue(transaction)
    }
    /// Commit one queued transaction and schedule an applied result without changing its receipt.
    pub fn commit_next(
        &mut self,
        cx: &mut Context<Self>,
    ) -> ChartResult<
        Option<(
            chart_core::transaction::TransactionId,
            chart_core::transaction::CommitOutcome,
        )>,
    > {
        let outcome = self.chart.commit_next()?;
        if matches!(
            outcome,
            Some((_, chart_core::transaction::CommitOutcome::Applied(_)))
        ) {
            self.schedule_committed(cx);
        }
        Ok(outcome)
    }
    /// Advance the owned epoch and retain explicit conflict outcomes for previously queued updates.
    pub fn reset_epoch(
        &mut self,
        cx: &mut Context<Self>,
    ) -> ChartResult<chart_core::state::Reconciliation> {
        let reconciliation = self.chart.reset_epoch()?;
        self.release_input();
        if let Err(error) = self.reset_preparation(false, cx) {
            self.last_error = Some(error);
        }
        self.schedule_committed(cx);
        Ok(reconciliation)
    }
    fn schedule_committed(&mut self, cx: &mut Context<Self>) {
        if let Some(reconciliation) = self.chart.reconciliation() {
            cx.emit(ChartHostEvent::DataReconciled(reconciliation.clone()));
        }
        if let Err(error) = self.queue_current(cx) {
            self.last_error = Some(error);
            cx.notify();
        }
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
        self.reset_preparation(true, cx)?;
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
        let request = self.chart.request(action, ActionOrigin::Programmatic);
        Ok(self.dispatch_action(request, cx)?.outcome.changed)
    }
    /// Current semantic state, including distinct component revisions.
    pub fn state(&self) -> &ChartState {
        self.chart.reducer().state()
    }
    /// Next valid identity shared by native gestures and host editing controls.
    pub fn next_gesture_id(&self) -> ChartResult<Revision> {
        self.chart.reducer().next_gesture_id()
    }
    /// Prepare a control/input request using current state and the presented or pinned basis.
    pub fn action_request(&self, action: ChartAction, origin: ActionOrigin) -> ActionRequest {
        self.chart.request(action, origin)
    }
    /// Exact pinned axes/source for host gesture coordinate conversion, never pending geometry.
    pub fn gesture_basis(&self) -> Option<&Arc<chart_core::layout::LaidOutChart>> {
        self.chart.reducer().gesture_basis()
    }
    /// Apply a current controlled response atomically with required preparation.
    pub fn accept_controlled(
        &mut self,
        expected: Revision,
        state: ChartState,
        cx: &mut Context<Self>,
    ) -> ChartResult<bool> {
        let Some(prepared) = self.chart.accept_controlled_prepared(expected, state)? else {
            return Ok(false);
        };
        if self.state().active_gesture().is_none() {
            self.release_input();
        }
        self.prepared = prepared;
        self.reset_preparation(true, cx)?;
        cx.notify();
        Ok(true)
    }
    /// Full origin/scene/revision-fenced path. Failed preparation leaves reducer/history intact.
    pub fn dispatch_action(
        &mut self,
        request: ActionRequest,
        cx: &mut Context<Self>,
    ) -> ChartResult<DispatchOutcome> {
        let (result, prepared) = self.chart.dispatch_prepared(request)?;
        if let Some(prepared) = prepared {
            self.prepared = prepared;
        }
        if result
            .event
            .as_ref()
            .is_some_and(|e| e.presentation_changed)
        {
            self.reset_preparation(true, cx)?;
        }
        if self.state().active_gesture().is_none() {
            self.release_input();
        }
        if result.outcome.changed {
            cx.notify();
        }
        if let Some(event) = &result.event {
            cx.emit(ChartHostEvent::StateChanged(event.clone()));
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
        let mut request = self.chart.request(semantic, origin);
        request.scene = Some(stamp);
        let outcome = self.dispatch_action(request, cx)?;
        self.inspector = Some(inspector);
        if inspected.changed {
            cx.notify();
        }
        Ok(outcome.outcome.changed || inspected.changed)
    }
    /// Capture the last coherent painted source, geometry state, inspection and resources.
    /// Pending data/annotation/theme changes cannot enter this capture before they are painted.
    pub fn capture_presented(&self) -> ChartResult<PresentedCapture> {
        let frame = self
            .frame
            .as_ref()
            .filter(|_| self.inspector.is_some() && self.painted_state.is_some())
            .ok_or_else(|| {
                crate::native::error(
                    chart_core::DiagnosticCode::Validation,
                    "No coherent painted chart is available for capture.",
                )
            })?;
        let painted = self
            .painted_state
            .as_ref()
            .unwrap_or(frame.chart.prepared().state());
        Ok(PresentedCapture {
            chart: frame.chart.clone(),
            state: frame
                .chart
                .prepared()
                .state()
                .with_painted_inspection(painted),
            layout: frame.request.clone(),
            fonts: frame.fonts.clone(),
            extensions: self.chart.extensions().clone(),
        })
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
        if self.chart.reducer().frozen_scene().is_some() {
            let frame = self.frame.clone()?;
            if frame.bounds == bounds {
                return Some(frame);
            }
            if let Some(cached) = &self.cached
                && cached.bounds == bounds
                && Arc::ptr_eq(cached.chart.prepared(), frame.chart.prepared())
            {
                return Some(cached.clone());
            }
            let key = (
                bounds,
                self.request.revision,
                Arc::as_ptr(frame.chart.prepared()) as usize,
            );
            if self.attempted.as_ref() == Some(&key) {
                return Some(frame);
            }
            self.attempted = Some(key);
            self.metrics.layout_attempts = self.metrics.layout_attempts.saturating_add(1);
            match self
                .request
                .revision
                .max(frame.request.revision)
                .checked_next()
                .and_then(|revision| frame.reproject(bounds, revision, window))
            {
                Ok(next) => {
                    self.request.revision = next.request.revision;
                    self.last_error = None;
                    self.cached = Some(Rc::new(next));
                    return self.cached.clone();
                }
                Err(e) => {
                    self.last_error = Some(e);
                    self.inspector = None;
                    return Some(frame);
                }
            }
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
                &self.density,
                bounds,
                window,
            )
        });
        match result {
            Ok(mut frame) => {
                frame.job = self.scheduling.prepared_token;
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
        self.scheduling.window = Some(window.window_handle());
        let focused = self.focus.contains_focused(window, cx);
        if self.input.focused && !focused {
            if let Err(e) = self.cancel_input(chart_core::state::CancelReason::FocusLost, cx) {
                self.last_error = Some(e);
            }
            let _ = self.dispatch_inspection(InspectionAction::Clear, InputOrigin::Keyboard, cx);
        }
        self.input.focused = focused;
        let mut tokens = self
            .chart
            .definition()
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
                    if window.is_window_active() {
                        window.refresh();
                        cx.notify();
                    }
                    if !window.is_window_active()
                        && let Err(e) =
                            this.cancel_input(chart_core::state::CancelReason::CaptureLost, cx)
                    {
                        this.last_error = Some(e);
                    }
                },
            ));
        }
        let controls = self.host_elements(window, cx);
        let (accessible_summary, accessible_focus) = self.accessibility_elements(cx);
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
                                if this.chart.reducer().frozen_scene().is_none() {
                                    this.acknowledge_preparation(frame.job);
                                }
                                let painted_state = this.state().clone();
                                if let Err(e) = this.chart.acknowledge_paint_with_layout(
                                    frame.chart.clone(),
                                    &painted_state,
                                    &frame.request,
                                ) {
                                    this.last_error = Some(e);
                                    this.inspector = None;
                                    return;
                                }
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
                        if let Err(e) = this
                            .paint_input(window)
                            .and_then(|()| this.paint_edit_handles(window))
                        {
                            this.last_error = Some(e);
                            this.painted_state = None;
                        } else if painted {
                            this.painted_state = Some(this.state().clone());
                            if let Some(frame) = &this.frame {
                                let painted_state = this.state().clone();
                                if let Err(e) = this.chart.acknowledge_paint_with_layout(
                                    frame.chart.clone(),
                                    &painted_state,
                                    &frame.request,
                                ) {
                                    this.last_error = Some(e);
                                }
                            }
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
            .border_color(if self.focus.is_focused(window) {
                focus_color
            } else {
                gpui::rgba(0x00000000)
            })
            .focusable()
            .track_focus(&self.focus)
            .hover_listener_mode(gpui::HoverListenerMode::InputModalityIndependent)
            .role(Role::Group)
            .aria_label(accessible_summary)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event, window, cx| {
                    this.host.menu_at = None;
                    window.focus(&this.focus, cx);
                    if let Err(e) = this.pointer_down(event, cx) {
                        this.last_error = Some(e);
                    }
                    cx.notify();
                }),
            )
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                    if !this.supports_host_command(HostCommand::ContextMenu) {
                        return;
                    }
                    window.focus(&this.focus, cx);
                    let p = this.frame.as_ref().and_then(|f| {
                        Point::new(
                            f64::from(f32::from(event.position.x - f.bounds.origin.x)),
                            f64::from(f32::from(event.position.y - f.bounds.origin.y)),
                        )
                        .ok()
                    });
                    if let Err(e) = this.request_host_command(HostCommand::ContextMenu, p, cx) {
                        this.last_error = Some(e);
                    }
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if this.has_drag() || this.input.disabled {
                    return;
                }
                let Some(frame) = &this.frame else { return };
                let local = Point::new(
                    f64::from(f32::from(event.position.x - frame.bounds.origin.x)),
                    f64::from(f32::from(event.position.y - frame.bounds.origin.y)),
                );
                if let Ok(p) = local
                    && let Err(e) = this.dispatch_inspection(
                        InspectionAction::Hover(Some(p)),
                        InputOrigin::Pointer,
                        cx,
                    )
                {
                    this.last_error = Some(e);
                }
            }))
            .on_hover(cx.listener(|this, hovered: &bool, _, cx| {
                if !hovered
                    && !this.has_drag()
                    && !this
                        .inspector
                        .as_ref()
                        .is_some_and(Inspector::has_keyboard_focus)
                {
                    let _ =
                        this.dispatch_inspection(InspectionAction::Clear, InputOrigin::Pointer, cx);
                }
            }))
            .on_scroll_wheel(cx.listener(|this, event, window, cx| {
                if let Err(e) = this.scroll_input(event, window, cx) {
                    this.last_error = Some(e);
                    cx.notify();
                }
            }))
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if this.input.disabled || !this.focus.is_focused(window) {
                    return;
                }
                if event.keystroke.key == "escape" && this.host.menu_at.take().is_some() {
                    cx.notify();
                    cx.stop_propagation();
                    return;
                }
                if event.keystroke.modifiers.platform || event.keystroke.modifiers.control {
                    let command = match event.keystroke.key.as_str() {
                        "c" => Some(HostCommand::Copy),
                        "e" => Some(HostCommand::Export),
                        _ => None,
                    };
                    if let Some(command) = command {
                        if this.supports_host_command(command) {
                            if let Err(e) = this.request_host_command(command, None, cx) {
                                this.last_error = Some(e);
                            }
                            cx.stop_propagation();
                        }
                        return;
                    }
                    if event.keystroke.key == "z" && this.state().active_gesture().is_none() {
                        let action = if event.keystroke.modifiers.shift {
                            ChartAction::Redo
                        } else {
                            ChartAction::Undo
                        };
                        let request = this.action_request(action, ActionOrigin::Keyboard);
                        if let Err(e) = this.dispatch_action(request, cx) {
                            this.last_error = Some(e);
                        }
                        cx.stop_propagation();
                        return;
                    }
                }
                if event.keystroke.key == "escape" && this.state().active_gesture().is_some() {
                    if let Err(e) = this.cancel_input(chart_core::state::CancelReason::Explicit, cx)
                    {
                        this.last_error = Some(e);
                    }
                    cx.stop_propagation();
                    return;
                }
                if this.state().active_gesture().is_some() {
                    return;
                }
                match this.edit_key(event, cx) {
                    Ok(true) => {
                        cx.stop_propagation();
                        return;
                    }
                    Err(e) => {
                        this.last_error = Some(e);
                        cx.notify();
                        return;
                    }
                    _ => {}
                }
                if event.keystroke.key == "home" {
                    let r = this.action_request(ChartAction::Reset, ActionOrigin::Keyboard);
                    if let Err(e) = this.dispatch_action(r, cx) {
                        this.last_error = Some(e);
                    }
                    cx.stop_propagation();
                    return;
                }
                if event.keystroke.key == "space" {
                    if let Some(focus) = this.state().focus().cloned() {
                        let r = this.action_request(
                            ChartAction::Select {
                                change: chart_core::state::SelectionChange::Toggle,
                                targets: vec![focus],
                            },
                            ActionOrigin::Keyboard,
                        );
                        if let Err(e) = this.dispatch_action(r, cx) {
                            this.last_error = Some(e);
                        }
                    }
                    cx.stop_propagation();
                    return;
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
            .children(accessible_focus)
            .children(controls)
            .children(diagnostic)
    }
}

impl gpui::Focusable for ChartView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}
