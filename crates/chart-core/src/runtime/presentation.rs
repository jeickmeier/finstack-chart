use super::*;

impl Chart {
    /// Compile and atomically accept a synchronous external-source view update.
    /// The external store remains committed independently if preparation rejects it.
    pub fn accept_source_prepared(
        &mut self,
        source: SnapshotHandle<StoreSnapshot>,
    ) -> ChartResult<Arc<PreparedChart>> {
        self.active()?;
        self.validate_external_source(&source)?;
        let mut next = self.reducer.clone();
        next.reconcile_source(&self.definition, &source)?;
        let mut prepared =
            self.compiler
                .prepare(&self.definition, &source, next.state(), self.limits)?;
        let reconciliation = next.reconcile_prepared(&prepared)?;
        if prepared.state() != next.state() {
            prepared =
                self.compiler
                    .prepare(&self.definition, &source, next.state(), self.limits)?;
        }
        self.source = Source::External(source);
        self.reducer = next;
        self.reconciliation = Some(reconciliation);
        self.prune_inspectors();
        Ok(Arc::new(prepared))
    }
    /// Compatibility replacement for a caller-owned, already revisioned definition.
    /// All work succeeds before publishing its definition/reducer; primary edits use apply_plot.
    pub fn replace_definition_prepared(
        &mut self,
        definition: ChartDefinition,
    ) -> ChartResult<Arc<PreparedChart>> {
        self.active()?;
        self.compiler
            .validate(&definition, &self.source(), self.limits)?;
        let mut next = self.reducer.clone();
        if next.state().active_gesture().is_some() {
            let request = next.request(
                &self.definition,
                ChartAction::CancelGesture(CancelReason::TargetRemoved),
                ActionOrigin::Programmatic,
            );
            next.dispatch(&self.definition, request)?;
        }
        let prepared =
            self.compiler
                .prepare(&definition, &self.source(), next.state(), self.limits)?;
        self.definition = definition;
        self.reducer = next;
        self.prune_inspectors();
        Ok(Arc::new(prepared))
    }
    /// Dispatch an action, preparing changed presentation before publishing reducer/history.
    /// Hover/focus-only actions return no new geometry and never run a statistic.
    pub fn dispatch_prepared(
        &mut self,
        request: ActionRequest,
    ) -> ChartResult<(DispatchOutcome, Option<Arc<PreparedChart>>)> {
        self.active()?;
        let mut next = self.reducer.clone();
        let result = next.dispatch(&self.definition, request)?;
        let prepared = if result
            .event
            .as_ref()
            .is_some_and(|e| e.presentation_changed)
        {
            Some(Arc::new(self.compiler.prepare(
                &self.definition,
                &self.source(),
                next.state(),
                self.limits,
            )?))
        } else {
            None
        };
        self.reducer = next;
        self.prune_inspectors();
        Ok((result, prepared))
    }
    /// Apply a controlled response atomically with required preparation and existing fences.
    pub fn accept_controlled_prepared(
        &mut self,
        expected: Revision,
        state: ChartState,
    ) -> ChartResult<Option<Arc<PreparedChart>>> {
        self.active()?;
        let mut next = self.reducer.clone();
        if !next.accept_controlled(&self.definition, expected, state)? {
            return Ok(None);
        }
        let prepared =
            self.compiler
                .prepare(&self.definition, &self.source(), next.state(), self.limits)?;
        self.reducer = next;
        self.prune_inspectors();
        Ok(Some(Arc::new(prepared)))
    }
    /// Reconcile a host worker's admitted output while retaining its independent compiler cache.
    /// Compatible older source results may be presented while newer work waits; they never
    /// replace the current committed source or regress its reconciled selection/follow state.
    pub fn admit_preparation(
        &mut self,
        mut prepared: PreparedChart,
        compiler: &mut Compiler,
    ) -> ChartResult<Arc<PreparedChart>> {
        self.active()?;
        let source = prepared.source().clone();
        let current = self.source();
        let (before, now) = (source.get()?, current.get()?);
        if prepared.definition() != &self.definition
            || before.epoch() != now.epoch()
            || before.revision() > now.revision()
            || (before.revision() == now.revision() && !std::ptr::eq(before, now))
            || !Arc::ptr_eq(compiler.extensions(), self.extensions())
        {
            return Err(error(
                DiagnosticCode::Superseded,
                "Worker output does not match this chart's current definition/source/registry.",
            ));
        }
        let mut next = self.reducer.clone();
        let reconciliation = if std::ptr::eq(before, now) {
            Some(next.reconcile_prepared(&prepared)?)
        } else {
            None
        };
        if prepared.state() != next.state() {
            prepared = compiler.prepare(&self.definition, &source, next.state(), self.limits)?;
        }
        self.reducer = next;
        if let Some(reconciliation) = reconciliation {
            self.reconciliation = Some(reconciliation);
        }
        self.prune_inspectors();
        Ok(Arc::new(prepared))
    }
    /// Current numeric-work budgets, copied into destination-owned worker jobs.
    pub fn compile_limits(&self) -> CompileLimits {
        self.limits
    }
    /// Record an actually painted frame and its inspection overlay state, including frozen
    /// reprojection. Pending definition/data/view state cannot enter this capture.
    pub fn acknowledge_paint(
        &mut self,
        scene: Arc<LaidOutChart>,
        painted: &ChartState,
    ) -> ChartResult<()> {
        self.active()?;
        let state = scene.prepared().state().with_painted_inspection(painted);
        let changed = self
            .reducer
            .presented()
            .is_none_or(|old| !Arc::ptr_eq(old, &scene));
        if let Some(frozen) = self.reducer.frozen_scene() {
            // Repainting the exact frozen frame updates only its painted overlays.
            // A different layout still requires the reducer's strict reprojection checks.
            if !Arc::ptr_eq(frozen, &scene) {
                self.reducer.present_frozen(scene)?;
            }
        } else {
            self.reducer.present(scene);
        }
        if changed {
            self.painted_layout = None;
        }
        self.painted_state = Some(state);
        self.prune_inspectors();
        Ok(())
    }
    /// Exact geometry/inspection state recorded by the last acknowledged paint.
    pub fn painted_state(&self) -> Option<&ChartState> {
        self.painted_state.as_ref()
    }
    /// Acknowledge the actual destination policy together with the scene and painted overlays.
    pub fn acknowledge_paint_with_layout(
        &mut self,
        scene: Arc<LaidOutChart>,
        painted: &ChartState,
        layout: &crate::layout::LayoutRequest,
    ) -> ChartResult<()> {
        self.acknowledge_paint(scene, painted)?;
        if self.painted_layout.is_none() {
            self.painted_layout = Some(layout.clone());
        }
        Ok(())
    }
    /// Original destination policy, when the host acknowledged it with the last paint.
    pub fn painted_layout(&self) -> Option<&crate::layout::LayoutRequest> {
        self.painted_layout.as_ref()
    }
}
