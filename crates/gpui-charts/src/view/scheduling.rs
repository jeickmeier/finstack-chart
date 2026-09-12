use super::*;
use chart_core::scheduling::{
    CompatibilityStamp, CompletionOutcome, JobToken, PreparationScheduler, SchedulerMetrics,
    SubmitOutcome,
};
use gpui::Task;

struct Work {
    definition: ChartDefinition,
    source: SnapshotHandle<StoreSnapshot>,
    state: ChartState,
    limits: CompileLimits,
}
pub(super) struct Scheduling {
    queue: PreparationScheduler<Work>,
    compiler: Option<Compiler>,
    task: Option<Task<()>>,
    latest: Option<SnapshotHandle<StoreSnapshot>>,
    generation: Revision,
    pub prepared_token: Option<JobToken>,
    pub window: Option<gpui::AnyWindowHandle>,
    frame_requested: bool,
}
impl Scheduling {
    pub fn new(extensions: Arc<chart_core::grammar::ExtensionRegistry>) -> Self {
        Self {
            queue: PreparationScheduler::new(),
            compiler: Some(Compiler::with_extensions(extensions)),
            task: None,
            latest: None,
            generation: Revision::INITIAL,
            prepared_token: None,
            window: None,
            frame_requested: false,
        }
    }
}
impl ChartView {
    /// Accept a coherent committed snapshot for bounded background numeric preparation.
    /// New data replaces only the pending request; an active compatible build continues.
    /// This acknowledges preparation acceptance, not validation, commitment or painting.
    /// The synchronous `set_data` path remains available for small atomic replacements.
    pub fn queue_data(
        &mut self,
        source: SnapshotHandle<StoreSnapshot>,
        cx: &mut Context<Self>,
    ) -> ChartResult<SubmitOutcome> {
        let reconciliation = self.chart.accept_source(source)?;
        cx.emit(ChartHostEvent::DataReconciled(reconciliation));
        self.queue_current(cx)
    }
    /// Queue the latest committed source of this owned or external chart.
    pub fn queue_current(&mut self, cx: &mut Context<Self>) -> ChartResult<SubmitOutcome> {
        let source = self.chart.source();
        let snapshot = source.get()?;
        let generation = self.scheduling.generation;
        let key = CompatibilityStamp {
            epoch: snapshot.epoch(),
            definition: generation,
            viewport: generation,
            layout: generation,
            resources: generation,
            presentation: generation,
        };
        let outcome = self.scheduling.queue.submit(
            key,
            snapshot.revision(),
            Work {
                definition: self.chart.definition().clone(),
                source: source.clone(),
                state: self.chart.state().clone(),
                limits: self.chart.compile_limits(),
            },
        )?;
        self.scheduling.latest = Some(source);
        self.start_preparation(cx);
        cx.notify();
        Ok(outcome)
    }
    /// Bounded active/pending work and latest committed-to-painted revision lag.
    pub fn scheduling_metrics(&self) -> &SchedulerMetrics {
        self.scheduling.queue.metrics()
    }
    /// Permanently close background preparation and release pending/cache ownership.
    /// An already running synchronous core calculation releases its snapshot on completion.
    pub fn dispose_preparation(&mut self, cx: &mut Context<Self>) {
        self.guide_animation.running = None;
        self.scheduling.queue.dispose();
        self.scheduling.latest = None;
        self.scheduling.compiler = None;
        self.scheduling.prepared_token = None;
        cx.notify();
    }
    pub(super) fn reset_preparation(
        &mut self,
        resubmit: bool,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        self.scheduling.generation = self.scheduling.generation.checked_next()?;
        self.scheduling.queue.invalidate();
        self.scheduling.prepared_token = None;
        if resubmit {
            if self.scheduling.latest.is_some() {
                self.queue_current(cx)?;
            }
        } else {
            self.scheduling.latest = None;
        }
        Ok(())
    }
    fn start_preparation(&mut self, cx: &mut Context<Self>) {
        let Some(job) = self.scheduling.queue.start() else {
            return;
        };
        let mut compiler = self
            .scheduling
            .compiler
            .take()
            .unwrap_or_else(|| Compiler::with_extensions(self.chart.extensions().clone()));
        let task = cx.background_spawn(async move {
            let result = compiler.prepare(
                &job.input.definition,
                &job.input.source,
                &job.input.state,
                job.input.limits,
            );
            (job.token, compiler, result)
        });
        self.scheduling.task = Some(cx.spawn(async move |entity, cx| {
            let (token, mut compiler, result) = task.await;
            let _ = entity.update(cx, |this, cx| {
                let outcome = this.scheduling.queue.complete(token, result.is_ok());
                if outcome == CompletionOutcome::Ready {
                    let result = result
                        .and_then(|prepared| this.install_preparation(prepared, &mut compiler, cx));
                    if let Err(e) = result {
                        this.last_error = Some(e);
                    } else {
                        this.scheduling.prepared_token = Some(token);
                    }
                } else if outcome == CompletionOutcome::Failed {
                    this.last_error = result.err();
                }
                this.scheduling.task = None;
                if !this.scheduling.queue.metrics().disposed {
                    this.scheduling.compiler = Some(compiler);
                    this.start_preparation(cx);
                }
                cx.notify();
                if matches!(
                    outcome,
                    CompletionOutcome::Ready | CompletionOutcome::Failed
                ) {
                    this.request_presentation_frame(cx);
                }
            });
        }));
    }
    // Entity notifications can leave an idle platform frame source asleep. Keep at most
    // one explicit wake outstanding; the callback owns only a weak chart reference.
    fn request_presentation_frame(&mut self, cx: &mut Context<Self>) {
        if self.scheduling.frame_requested || self.scheduling.queue.metrics().disposed {
            return;
        }
        let Some(window) = self.scheduling.window else {
            return;
        };
        let weak = cx.entity().downgrade();
        self.scheduling.frame_requested = true;
        if window
            .update(cx, |_, window, _| {
                window.refresh();
                window.on_next_frame(move |_, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        this.scheduling.frame_requested = false;
                        if !this.scheduling.queue.metrics().disposed {
                            cx.notify();
                        }
                    });
                });
            })
            .is_err()
        {
            self.scheduling.frame_requested = false;
        }
    }
    fn install_preparation(
        &mut self,
        prepared: PreparedChart,
        compiler: &mut Compiler,
        cx: &mut Context<Self>,
    ) -> ChartResult<()> {
        self.prepared = self.chart.admit_preparation(prepared, compiler)?;
        if self.state().active_gesture().is_none() {
            self.release_input();
        }
        self.last_error = None;
        if let Some(reconciliation) = self.chart.reconciliation() {
            cx.emit(ChartHostEvent::DataReconciled(reconciliation.clone()));
        }
        Ok(())
    }
    pub(super) fn acknowledge_preparation(&mut self, token: Option<JobToken>) {
        if let Some(token) = token {
            self.scheduling.queue.present(token);
        }
    }
}
