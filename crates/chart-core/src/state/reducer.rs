use super::*;
use crate::composition::{Anchor, ScaleValue};
use crate::grammar::SelectionPolicy;
use crate::layout::LaidOutChart;
use crate::{Limits, SceneStamp};
use std::{
    collections::VecDeque,
    sync::{Arc, Weak},
};

#[derive(Clone, Debug)]
struct Command {
    before: BTreeMap<String, Option<Annotation>>,
    after: BTreeMap<String, Option<Annotation>>,
}
#[derive(Clone, Debug)]
struct LinkRecord {
    revision: Revision,
    viewport: Option<Viewport>,
    windows: Option<AxisWindows>,
    selection: Option<Vec<MarkTarget>>,
}
/// Retained synchronous reducer. Only this owner retains history and at most one gesture basis
/// plus one frozen scene; cloning semantic ChartState never retains these runtime resources.
#[derive(Clone, Debug)]
pub struct ActionReducer {
    state: ChartState,
    presented: Option<Arc<LaidOutChart>>,
    gesture_basis: Option<Arc<LaidOutChart>>,
    frozen: Option<Arc<LaidOutChart>>,
    pinned: Option<Arc<LaidOutChart>>,
    current_source: Option<crate::data::SnapshotHandle<crate::data::StoreSnapshot>>,
    followed_source: Option<(crate::SourceEpoch, Revision)>,
    history: VecDeque<Command>,
    redo: Vec<Command>,
    links: BTreeMap<String, LinkRecord>,
    last_gesture: Revision,
    disposed: bool,
    target_indexes: Vec<TargetIndex>,
}
#[derive(Clone, Debug)]
struct TargetIndex {
    scene: Weak<LaidOutChart>,
    targets: Arc<BTreeMap<MarkTarget, (bool, bool)>>,
}
impl Default for ActionReducer {
    fn default() -> Self {
        Self::new(ChartState::default())
    }
}
impl ActionReducer {
    /// Own committed semantic state; no presented scene is inferred from pending preparation.
    pub fn new(mut state: ChartState) -> Self {
        state.active = None;
        Self {
            state,
            presented: None,
            gesture_basis: None,
            frozen: None,
            pinned: None,
            current_source: None,
            followed_source: None,
            history: VecDeque::new(),
            redo: Vec::new(),
            links: BTreeMap::new(),
            last_gesture: Revision::INITIAL,
            disposed: false,
            target_indexes: vec![],
        }
    }
    /// Current immutable semantic state for preparation and application observation.
    pub fn state(&self) -> &ChartState {
        &self.state
    }
    /// Acknowledge the scene actually submitted by the host. Active gestures keep their own basis.
    pub fn present(&mut self, scene: Arc<LaidOutChart>) {
        if !self.disposed {
            if !self
                .target_indexes
                .iter()
                .any(|i| i.scene.ptr_eq(&Arc::downgrade(&scene)))
            {
                let mut targets = BTreeMap::new();
                if let Ok(source) = scene.prepared().source().get() {
                    for target in scene.prepared().semantic_targets() {
                        let entry = targets
                            .entry(MarkTarget {
                                epoch: source.epoch(),
                                layer: target.layer,
                                panel: target.panel.cloned(),
                                identity: target.target.into(),
                            })
                            .or_insert((false, false));
                        entry.1 |= target.selectable;
                    }
                    for (i, (item, values)) in scene
                        .scene()
                        .items()
                        .iter()
                        .zip(scene.targets())
                        .enumerate()
                    {
                        let Some(layer) = item.layer else { continue };
                        let selectable = scene
                            .interactions()
                            .get(&i)
                            .is_none_or(|v| v.selection != SelectionPolicy::Disabled);
                        for target in values {
                            let entry = targets
                                .entry(MarkTarget {
                                    epoch: source.epoch(),
                                    layer,
                                    panel: scene.item_panels()[i].clone(),
                                    identity: target.into(),
                                })
                                .or_insert((false, false));
                            entry.0 = true;
                            entry.1 |= selectable;
                        }
                    }
                }
                self.target_indexes.push(TargetIndex {
                    scene: Arc::downgrade(&scene),
                    targets: Arc::new(targets),
                });
            }
            self.presented = Some(scene);
            self.prune_indexes();
        }
    }
    fn prune_indexes(&mut self) {
        self.target_indexes.retain(|i| {
            [
                &self.presented,
                &self.gesture_basis,
                &self.frozen,
                &self.pinned,
            ]
            .into_iter()
            .flatten()
            .any(|s| i.scene.ptr_eq(&Arc::downgrade(s)))
        });
    }
    /// Currently acknowledged visible scene (freeze, when active, takes precedence).
    pub fn presented(&self) -> Option<&Arc<LaidOutChart>> {
        self.frozen.as_ref().or(self.presented.as_ref())
    }
    /// Exact immutable gesture basis; later data/layout results cannot silently move it.
    pub fn gesture_basis(&self) -> Option<&Arc<LaidOutChart>> {
        self.gesture_basis.as_ref()
    }
    /// Explicit frozen scene retained independently of source ingestion.
    pub fn frozen_scene(&self) -> Option<&Arc<LaidOutChart>> {
        self.frozen.as_ref()
    }
    /// Acknowledge a resized frozen figure after successful host painting. Only layout may
    /// change: the exact retained prepared object must survive, and layout revisions advance.
    /// Active gestures/pins retain their original basis independently of this new projection.
    pub fn present_frozen(&mut self, scene: Arc<LaidOutChart>) -> ChartResult<()> {
        let old = self.frozen.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::RevisionConflict,
                "No frozen presentation is active.",
            )
        })?;
        if self.disposed
            || !Arc::ptr_eq(old.prepared(), scene.prepared())
            || scene.scene().stamp().layout <= old.scene().stamp().layout
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Frozen reprojection must retain the exact prepared input and advance layout.",
            ));
        }
        self.frozen = Some(scene.clone());
        self.present(scene);
        Ok(())
    }
    /// At most one original pinned scene; explicitly unpin to release historical resources.
    pub fn pinned_scene(&self) -> Option<&Arc<LaidOutChart>> {
        self.pinned.as_ref()
    }
    /// Whether the pinned observation/model describes an older coherent source snapshot.
    pub fn pinned_is_historical(&self) -> bool {
        match (&self.pinned, &self.current_source) {
            (Some(scene), Some(current)) => {
                match (scene.prepared().source().get(), current.get()) {
                    (Ok(old), Ok(new)) => {
                        old.epoch() != new.epoch() || old.revision() != new.revision()
                    }
                    _ => true,
                }
            }
            _ => false,
        }
    }
    /// Resolve the original pinned observation, with an explicit historical label.
    pub fn describe_pinned(&self) -> ChartResult<Option<PinnedDescription>> {
        let (Some(scene), Some(target)) = (&self.pinned, self.state.pinned()) else {
            return Ok(None);
        };
        let inspector = crate::inspection::Inspector::new(scene.clone(), 10., 32)?;
        let hit = inspector.target(target)?.ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                "Pinned target is absent from its original scene.",
            )
        })?;
        Ok(Some(PinnedDescription {
            historical: self.pinned_is_historical(),
            scene: scene.scene().stamp(),
            target: inspector.describe_target(hit, &self.state)?,
        }))
    }
    /// Prune evicted source identities immediately on acceptance, preserving original pinned
    /// values and frozen/gesture scenes. Does not acknowledge a pending scene as presented.
    pub fn reconcile_source(
        &mut self,
        definition: &ChartDefinition,
        source: &crate::data::SnapshotHandle<crate::data::StoreSnapshot>,
    ) -> ChartResult<Reconciliation> {
        let snapshot = source.get()?;
        let valid = |t: &MarkTarget| {
            t.epoch == snapshot.epoch()
                && match &t.identity {
                    TargetIdentity::Source { dataset, key } => snapshot
                        .dataset(*dataset)
                        .is_ok_and(|d| d.row(*key).is_some()),
                    TargetIdentity::Aggregate { dataset, .. } => snapshot.dataset(*dataset).is_ok(),
                    TargetIdentity::HierarchyNode { dataset, node, .. } => {
                        snapshot.dataset(*dataset).is_ok_and(|data| match node {
                            crate::hierarchy::HierarchyTargetKey::Source(key) => {
                                data.row(*key).is_some()
                            }
                            crate::hierarchy::HierarchyTargetKey::Synthetic(_) => true,
                        })
                    }
                    TargetIdentity::Derived { datasets, .. } => {
                        datasets.iter().all(|id| snapshot.dataset(*id).is_ok())
                    }
                }
        };
        self.reconcile_with(definition, source, valid)
    }
    /// Reconcile group/model/mark availability after exact preparation; hidden and clipped
    /// prepared targets remain valid, while disappeared groups/facets are explicitly removed.
    pub fn reconcile_prepared(
        &mut self,
        prepared: &crate::grammar::PreparedChart,
    ) -> ChartResult<Reconciliation> {
        let source = prepared.source();
        let epoch = source.get()?.epoch();
        let available: BTreeSet<_> = prepared
            .semantic_targets()
            .filter(|t| t.selectable)
            .map(|t| MarkTarget {
                epoch,
                layer: t.layer,
                panel: t.panel.cloned(),
                identity: t.target.into(),
            })
            .collect();
        let mut next = self.clone();
        let result =
            next.reconcile_with(prepared.definition(), source, |t| available.contains(t))?;
        let stamp = (source.get()?.epoch(), source.get()?.revision());
        if next.state.follow() == FollowMode::FollowLatest && next.followed_source != Some(stamp) {
            super::follow::advance(&mut next.state, prepared)?;
        }
        next.followed_source = Some(stamp);
        // Publish all removal/follow changes as one revisioned transition.
        // reconcile_with ran on the candidate; reset its counters before the combined publish.
        next.state.revision = self.state.revision;
        next.state.viewport_revision = self.state.viewport_revision;
        next.state.revisions = self.state.revisions.clone();
        let transition = self.publish(
            prepared.definition(),
            next,
            ActionOrigin::Programmatic,
            result.transition.event.and_then(|e| e.cancellation),
        )?;
        Ok(Reconciliation {
            transition,
            ..result
        })
    }
    fn reconcile_with(
        &mut self,
        definition: &ChartDefinition,
        source: &crate::data::SnapshotHandle<crate::data::StoreSnapshot>,
        valid: impl Fn(&MarkTarget) -> bool,
    ) -> ChartResult<Reconciliation> {
        if self.disposed {
            return Err(error(
                DiagnosticCode::DisposedHandle,
                "Cannot reconcile a disposed reducer.",
            ));
        }
        if let Some(old) = &self.current_source {
            let (old, new) = (old.get()?, source.get()?);
            if new.epoch() < old.epoch()
                || (new.epoch() == old.epoch() && new.revision() < old.revision())
            {
                return Err(error(
                    DiagnosticCode::RevisionConflict,
                    "Reconciliation cannot regress accepted source revisions.",
                ));
            }
        }
        let mut next = self.clone();
        let removed_selection: Vec<_> = self
            .state
            .durable
            .selection
            .iter()
            .filter(|t| !valid(t))
            .cloned()
            .collect();
        next.state.durable.selection.retain(&valid);
        next.state.hover.retain(&valid);
        if next.state.focus.as_ref().is_some_and(|t| !valid(t)) {
            next.state.focus = None;
        }
        let cancel = next.state.active.as_ref().is_some_and(|g| matches!(g.kind,GestureKind::Selection) && (!removed_selection.is_empty() || matches!(&g.preview, Some(GesturePreview::Selection(v)) if v.iter().any(|t| !valid(t)))));
        if cancel {
            next.state.active = None;
            next.gesture_basis = None;
        }
        next.current_source = Some(source.clone());
        let historical = next.pinned_is_historical();
        let transition = self.publish(
            definition,
            next,
            ActionOrigin::Programmatic,
            cancel.then_some(CancelReason::TargetRemoved),
        )?;
        Ok(Reconciliation {
            removed_selection,
            store_revision: source.get()?.revision(),
            pinned_historical: historical,
            transition,
        })
    }
    /// Bounded undo/redo command counts; transient actions never add commands.
    pub fn history_lengths(&self) -> (usize, usize) {
        (self.history.len(), self.redo.len())
    }
    /// Allocate no state; return the next valid gesture identity across all input producers.
    pub fn next_gesture_id(&self) -> ChartResult<Revision> {
        self.last_gesture.checked_next()
    }
    /// Build a request from current fences; callers may retain it to detect stale responses.
    pub fn request(
        &self,
        definition: &ChartDefinition,
        action: ChartAction,
        origin: ActionOrigin,
    ) -> ActionRequest {
        ActionRequest {
            definition_revision: definition.revision,
            expected_state: self.state.revision(),
            origin,
            scene: self
                .gesture_basis
                .as_ref()
                .or(self.presented())
                .map(|p| p.scene().stamp()),
            action,
        }
    }
    /// Validate and publish one atomic effective transition. Errors leave values/history/handles intact.
    pub fn dispatch(
        &mut self,
        definition: &ChartDefinition,
        request: ActionRequest,
    ) -> ChartResult<DispatchOutcome> {
        if self.disposed {
            return Err(error(
                DiagnosticCode::DisposedHandle,
                "This action reducer has been disposed.",
            ));
        }
        if request.definition_revision != definition.revision
            || request.expected_state != self.state.revision
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Action has a stale definition or state revision.",
            ));
        }
        validate_action_bounds(&request.action, self.state.configuration.max_targets)?;
        let before = &self.state;
        let mut next = self.clone();
        let mut cancellation = None;
        let mut record = true;
        next.reduce(definition, &request, &mut cancellation, &mut record)?;
        if record && before.durable.annotations != next.state.durable.annotations {
            next.redo.clear();
            if next.state.configuration.history_capacity > 0 {
                next.history.push_back(Command {
                    before: before.durable.annotations.clone(),
                    after: next.state.durable.annotations.clone(),
                });
            }
        }
        while next.history.len() > next.state.configuration.history_capacity {
            next.history.pop_front();
        }
        while next.redo.len() > next.state.configuration.history_capacity {
            next.redo.remove(0);
        }
        self.publish(definition, next, request.origin, cancellation)
    }
    fn publish(
        &mut self,
        definition: &ChartDefinition,
        mut next: Self,
        origin: ActionOrigin,
        cancellation: Option<CancelReason>,
    ) -> ChartResult<DispatchOutcome> {
        let before = &self.state;
        if next.state.durable.pinned.is_none() {
            next.pinned = None;
        }
        let changed = *before != next.state;
        let viewport_changed = before.viewport() != next.state.viewport()
            || before.axis_windows() != next.state.axis_windows();
        let durable_changed = before.durable != next.state.durable
            || before.configuration != next.state.configuration;
        let presentation_changed = viewport_changed
            || before.durable.hidden != next.state.durable.hidden
            || before.durable.legend_visible != next.state.durable.legend_visible
            || before.annotations(definition) != next.state.annotations(definition)
            || before.durable.follow != next.state.durable.follow;
        let event = if changed {
            next.state.revision = before.revision.checked_next()?;
            if viewport_changed {
                next.state.viewport_revision = before.viewport_revision.checked_next()?;
            }
            let a = before;
            let b = &mut next.state;
            macro_rules! bump {
                ($field:ident, $condition:expr) => {
                    if $condition {
                        b.revisions.$field = a.revisions.$field.checked_next()?;
                    }
                };
            }
            bump!(durable, durable_changed);
            bump!(follow, a.durable.follow != b.durable.follow);
            bump!(hover, a.hover != b.hover);
            bump!(focus, a.focus != b.focus);
            bump!(pin, a.durable.pinned != b.durable.pinned);
            bump!(selection, a.durable.selection != b.durable.selection);
            bump!(
                visibility,
                a.durable.hidden != b.durable.hidden
                    || a.durable.legend_visible != b.durable.legend_visible
            );
            bump!(annotations, a.durable.annotations != b.durable.annotations);
            bump!(gesture, a.active != b.active);
            bump!(configuration, a.configuration != b.configuration);
            Some(StateEvent {
                revision: b.revision,
                origin,
                durable: durable_changed,
                presentation_changed,
                cancellation,
            })
        } else {
            None
        };
        let result = DispatchOutcome {
            outcome: ActionOutcome {
                changed,
                revision: next.state.revision,
                viewport_changed,
            },
            event,
        };
        next.prune_indexes();
        *self = next;
        Ok(result)
    }
    fn basis(&self, stamp: Option<SceneStamp>, gesture: bool) -> ChartResult<&Arc<LaidOutChart>> {
        let basis = if gesture {
            self.gesture_basis.as_ref()
        } else {
            self.presented()
        }
        .ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "This action requires an acknowledged presented scene.",
            )
        })?;
        if Some(basis.scene().stamp()) != stamp {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Input scene differs from the presented or pinned gesture scene.",
            ));
        }
        Ok(basis)
    }
    fn check_targets(
        &self,
        targets: &[MarkTarget],
        stamp: Option<SceneStamp>,
        gesture: bool,
        selectable: bool,
    ) -> ChartResult<()> {
        if targets.is_empty() {
            return Ok(());
        }
        let chart = self.basis(stamp, gesture)?;
        chart.prepared().source().get()?;
        if selectable && let Some(source) = &self.current_source {
            let source = source.get()?;
            if targets.iter().any(|t| t.epoch != source.epoch() || matches!(t.identity, TargetIdentity::Source { dataset, key } | TargetIdentity::HierarchyNode { dataset, node: crate::hierarchy::HierarchyTargetKey::Source(key), .. } if !source.dataset(dataset).is_ok_and(|d| d.row(key).is_some()))) {
                return Err(error(DiagnosticCode::Validation, "Selection target was evicted from current data; historical inspection does not restore an active selection."));
            }
        }
        // The exact Arc identity prevents equal revision stamps from borrowing another scene's targets.
        let available = self
            .target_indexes
            .iter()
            .find(|i| i.scene.ptr_eq(&Arc::downgrade(chart)));
        if targets.iter().any(|t| {
            !available
                .and_then(|i| i.targets.get(t))
                .is_some_and(|(presented, allowed)| if selectable { *allowed } else { *presented })
        }) {
            return Err(error(
                DiagnosticCode::Validation,
                "Action targets must exist with the requested capability in the presented scene.",
            ));
        }
        Ok(())
    }
    fn reduce(
        &mut self,
        d: &ChartDefinition,
        r: &ActionRequest,
        cancellation: &mut Option<CancelReason>,
        record: &mut bool,
    ) -> ChartResult<()> {
        use ChartAction::*;
        if matches!(r.origin, ActionOrigin::Linked(_)) != matches!(r.action, Synchronize { .. }) {
            return Err(error(
                DiagnosticCode::Validation,
                "Linked origins and synchronization actions must be paired.",
            ));
        }
        if self.state.active.is_some()
            && !matches!(
                r.action,
                PreviewGesture { .. }
                    | CommitGesture { .. }
                    | CancelGesture(_)
                    | SetHover(_)
                    | SetFocus(_)
                    | ClearInspection
                    | Reset
            )
        {
            return Err(error(
                DiagnosticCode::Validation,
                "One active gesture owns editing; commit or cancel it before another durable action.",
            ));
        }
        match &r.action {
            SetViewport(v) => {
                v.validate()?;
                self.state.durable.viewport = *v;
                self.state.durable.windows.remove(&crate::ScaleId::new(0));
                self.state.durable.windows.remove(&crate::ScaleId::new(1));
                if self.state.configuration.manual_view_enters_history {
                    self.state.durable.follow = FollowMode::InspectHistory;
                    self.frozen = None;
                }
            }
            SetAxisWindows(w) => {
                windows::validate_windows(w)?;
                self.state.durable.windows = w.clone();
                if self.state.configuration.manual_view_enters_history {
                    self.state.durable.follow = FollowMode::InspectHistory;
                    self.frozen = None;
                }
            }
            SetLayerVisible { layer, visible } => {
                if !d.layers.iter().any(|l| l.id == *layer) {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Visibility names an absent layer.",
                    ));
                }
                if *visible {
                    self.state.durable.hidden.remove(layer);
                } else {
                    self.state.durable.hidden.insert(*layer);
                }
            }
            SetLegendVisible(v) => self.state.durable.legend_visible = *v,
            SetFollow(mode) => self.follow(*mode, r.scene)?,
            ResumeLatest => {
                self.follow(FollowMode::FollowLatest, r.scene)?;
            }
            ClearInspection => {
                self.state.hover.clear();
                self.state.focus = None;
            }
            SetHover(targets) => {
                self.check_targets(targets, r.scene, false, false)?;
                self.state.hover = targets.iter().cloned().collect();
            }
            SetFocus(target) => {
                self.check_targets(target.as_slice(), r.scene, false, false)?;
                self.state.focus = target.clone();
            }
            SetPinned(target) => {
                self.check_targets(target.as_slice(), r.scene, false, false)?;
                if target != &self.state.durable.pinned {
                    self.pinned = if target.is_some() {
                        Some(self.basis(r.scene, false)?.clone())
                    } else {
                        None
                    };
                }
                self.state.durable.pinned = target.clone();
            }
            Select { change, targets } => {
                if !matches!(change, SelectionChange::Clear | SelectionChange::Remove) {
                    self.check_targets(targets, r.scene, false, true)?;
                }
                let selection = &mut self.state.durable.selection;
                match change {
                    SelectionChange::Replace => *selection = targets.iter().cloned().collect(),
                    SelectionChange::Add => selection.extend(targets.iter().cloned()),
                    SelectionChange::Toggle => {
                        for t in targets {
                            if !selection.remove(t) {
                                selection.insert(t.clone());
                            }
                        }
                    }
                    SelectionChange::Remove => {
                        for t in targets {
                            selection.remove(t);
                        }
                    }
                    SelectionChange::Clear => {
                        if !targets.is_empty() {
                            return Err(error(
                                DiagnosticCode::Validation,
                                "Clear selection takes no targets.",
                            ));
                        }
                        selection.clear();
                    }
                }
                if selection.len() > self.state.configuration.max_targets {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Selection exceeds its target budget.",
                    ));
                }
            }
            SetAnnotation(a) => self.annotation(d, a)?,
            RemoveAnnotation(id) => {
                if !self.state.annotations(d).iter().any(|a| &a.id == id) {
                    return Ok(());
                }
                self.state.durable.annotations.insert(id.clone(), None);
            }
            Configure(c) => {
                validate_config(c)?;
                if self.state.durable.selection.len() > c.max_targets
                    || self.state.hover.len() > c.max_targets
                {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Configuration is smaller than retained target state.",
                    ));
                }
                self.state.configuration = c.clone();
            }
            BeginGesture { id, kind } => {
                if *id <= self.last_gesture {
                    return Err(error(
                        DiagnosticCode::RevisionConflict,
                        "Gesture IDs must be positive and strictly increasing.",
                    ));
                }
                let basis = self.basis(r.scene, false)?.clone();
                if basis.scene().stamp().definition != d.revision {
                    return Err(error(
                        DiagnosticCode::RevisionConflict,
                        "Gesture basis uses a different definition.",
                    ));
                }
                if let GestureKind::Annotation(id) = kind
                    && !self.state.annotations(d).iter().any(|a| &a.id == id)
                {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Edited annotation is absent.",
                    ));
                }
                self.state.active = Some(ActiveGesture {
                    id: *id,
                    kind: kind.clone(),
                    basis: basis.scene().stamp(),
                    preview: None,
                });
                self.gesture_basis = Some(basis);
                self.last_gesture = *id;
            }
            PreviewGesture { id, preview } => {
                self.check_gesture(*id, r.scene)?;
                let kind = &self.state.active.as_ref().expect("checked gesture").kind;
                match (kind, preview) {
                    (GestureKind::Viewport, GesturePreview::Viewport(v)) => v.validate()?,
                    (GestureKind::Viewport, GesturePreview::AxisWindows(w)) => {
                        windows::validate_windows(w)?
                    }
                    (GestureKind::Selection, GesturePreview::Selection(targets)) => {
                        self.check_targets(targets, r.scene, true, true)?
                    }
                    (GestureKind::Annotation(id), GesturePreview::Annotation(a)) if id == &a.id => {
                        validate_annotation(d, a)?
                    }
                    _ => {
                        return Err(error(
                            DiagnosticCode::Validation,
                            "Preview does not belong to the active gesture owner.",
                        ));
                    }
                }
                let mut committed = self.state.clone();
                committed.active = None;
                let unchanged = match preview {
                    GesturePreview::Viewport(v) => {
                        *v == committed.viewport()
                            && !committed
                                .durable
                                .windows
                                .contains_key(&crate::ScaleId::new(0))
                            && !committed
                                .durable
                                .windows
                                .contains_key(&crate::ScaleId::new(1))
                    }
                    GesturePreview::AxisWindows(w) => w == committed.axis_windows().as_ref(),
                    GesturePreview::Selection(v) => {
                        v.iter().cloned().collect::<BTreeSet<_>>() == committed.durable.selection
                    }
                    GesturePreview::Annotation(a) => {
                        committed.annotations(d).iter().any(|old| old == a.as_ref())
                    }
                };
                self.state.active.as_mut().expect("checked gesture").preview = if unchanged {
                    None
                } else {
                    Some(preview.clone())
                };
            }
            CommitGesture { id } => {
                self.check_gesture(*id, r.scene)?;
                let preview = self.state.active.take().expect("checked gesture").preview;
                match preview {
                    Some(GesturePreview::Viewport(v)) => {
                        self.state.durable.viewport = v;
                        self.state.durable.windows.remove(&crate::ScaleId::new(0));
                        self.state.durable.windows.remove(&crate::ScaleId::new(1));
                        if self.state.configuration.manual_view_enters_history {
                            self.state.durable.follow = FollowMode::InspectHistory;
                            self.frozen = None;
                        }
                    }
                    Some(GesturePreview::AxisWindows(w)) => {
                        self.state.durable.windows = w;
                        if self.state.configuration.manual_view_enters_history {
                            self.state.durable.follow = FollowMode::InspectHistory;
                            self.frozen = None;
                        }
                    }
                    Some(GesturePreview::Selection(v)) => {
                        self.state.durable.selection = v.into_iter().collect()
                    }
                    Some(GesturePreview::Annotation(a)) => self.annotation(d, &a)?,
                    None => {}
                }
                self.gesture_basis = None;
            }
            CancelGesture(reason) => {
                if self.state.active.take().is_some() {
                    *cancellation = Some(*reason);
                }
                self.gesture_basis = None;
            }
            Undo => {
                *record = false;
                if let Some(command) = self.history.pop_back() {
                    self.state.durable.annotations = command.before.clone();
                    self.redo.push(command);
                }
            }
            Redo => {
                *record = false;
                if let Some(command) = self.redo.pop() {
                    self.state.durable.annotations = command.after.clone();
                    self.history.push_back(command);
                }
            }
            Synchronize {
                revision,
                viewport,
                windows,
                selection,
            } => {
                *record = false;
                let ActionOrigin::Linked(source) = &r.origin else {
                    unreachable!("checked linked origin")
                };
                if source.is_empty() || source.len() > 128 || *revision == Revision::INITIAL {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Synchronization requires a bounded source and positive revision.",
                    ));
                }
                if let Some(last) = self.links.get(source) {
                    if *revision < last.revision {
                        return Ok(());
                    }
                    if *revision == last.revision {
                        if *viewport != last.viewport
                            || *windows != last.windows
                            || *selection != last.selection
                        {
                            return Err(error(
                                DiagnosticCode::TransactionReuse,
                                "A synchronization revision was reused for different values.",
                            ));
                        }
                        return Ok(());
                    }
                } else if self.links.len() >= 64 {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "At most 64 linked sources are retained.",
                    ));
                }
                if viewport.is_some() && windows.is_some() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Choose legacy or typed linked windows, not both.",
                    ));
                }
                if let Some(v) = windows {
                    windows::validate_windows(v)?;
                    self.state.durable.windows.extend(v.clone());
                    windows::validate_windows(&self.state.durable.windows)?;
                }
                if let Some(v) = viewport {
                    v.validate()?;
                    self.state.durable.viewport = *v;
                    self.state.durable.windows.remove(&crate::ScaleId::new(0));
                    self.state.durable.windows.remove(&crate::ScaleId::new(1));
                }
                if let Some(v) = selection {
                    self.check_targets(v, r.scene, false, true)?;
                    self.state.durable.selection = v.iter().cloned().collect();
                }
                self.links.insert(
                    source.clone(),
                    LinkRecord {
                        revision: *revision,
                        viewport: *viewport,
                        windows: windows.clone(),
                        selection: selection.clone(),
                    },
                );
            }
            Reset => {
                if self.state.active.take().is_some() {
                    *cancellation = Some(CancelReason::Explicit);
                }
                self.state.durable = DurableState::default();
                self.state.hover.clear();
                self.state.focus = None;
                self.gesture_basis = None;
                self.frozen = None;
            }
        }
        Ok(())
    }
    fn follow(&mut self, mode: FollowMode, stamp: Option<SceneStamp>) -> ChartResult<()> {
        if mode == FollowMode::FreezePresentation && self.frozen.is_none() {
            self.frozen = Some(self.basis(stamp, false)?.clone());
        }
        if mode != FollowMode::FreezePresentation {
            self.frozen = None;
        }
        self.state.durable.follow = mode;
        Ok(())
    }
    fn check_gesture(&self, id: Revision, stamp: Option<SceneStamp>) -> ChartResult<()> {
        if self.state.active.as_ref().is_none_or(|g| g.id != id) {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Gesture ID is not the active owner.",
            ));
        }
        self.basis(stamp, true)?;
        Ok(())
    }
    fn annotation(&mut self, d: &ChartDefinition, a: &Annotation) -> ChartResult<()> {
        validate_annotation(d, a)?;
        if self.state.annotations(d).iter().any(|old| old == a) {
            return Ok(());
        }
        self.state
            .durable
            .annotations
            .insert(a.id.clone(), Some(a.clone()));
        if self.state.durable.annotations.len() > 256 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Annotation overrides exceed 256 identities.",
            ));
        }
        self.state
            .figure(d)
            .expect("annotation inserted")
            .validate(Limits::default())
    }
    /// Accept a controlled response only against the latest action revision. The host may
    /// acknowledge the exact proposal, or send a strictly newer valid durable replacement.
    pub fn accept_controlled(
        &mut self,
        definition: &ChartDefinition,
        expected: Revision,
        state: ChartState,
    ) -> ChartResult<bool> {
        if self.disposed {
            return Err(error(
                DiagnosticCode::DisposedHandle,
                "This action reducer has been disposed.",
            ));
        }
        if expected != self.state.revision
            || state.revision < expected
            || (state.revision == expected && state != self.state)
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Controlled response is stale or reuses a revision for different state.",
            ));
        }
        if state == self.state {
            return Ok(false);
        }
        if self.state.active.is_some() || state.active.is_some() {
            return Err(error(
                DiagnosticCode::Validation,
                "Controlled replacements must wait for gesture commit or cancellation.",
            ));
        }
        validate_state(definition, &state)?;
        if state.hover != self.state.hover || state.focus != self.state.focus {
            return Err(error(
                DiagnosticCode::Validation,
                "Controlled replacements cannot inject ephemeral inspection state.",
            ));
        }
        let old = &self.state;
        macro_rules! fence {
            ($name:ident, $same:expr) => {
                if state.revisions.$name < old.revisions.$name
                    || (state.revisions.$name == old.revisions.$name && !$same)
                {
                    return Err(error(
                        DiagnosticCode::RevisionConflict,
                        "Controlled response reuses/regresses a component revision.",
                    ));
                }
            };
        }
        fence!(
            durable,
            state.durable == old.durable && state.configuration == old.configuration
        );
        fence!(follow, state.durable.follow == old.durable.follow);
        fence!(hover, state.hover == old.hover);
        fence!(focus, state.focus == old.focus);
        fence!(pin, state.durable.pinned == old.durable.pinned);
        fence!(selection, state.durable.selection == old.durable.selection);
        fence!(
            visibility,
            state.durable.hidden == old.durable.hidden
                && state.durable.legend_visible == old.durable.legend_visible
        );
        fence!(
            annotations,
            state.durable.annotations == old.durable.annotations
        );
        fence!(gesture, state.active == old.active);
        fence!(configuration, state.configuration == old.configuration);
        if state.viewport_revision < self.state.viewport_revision
            || (state.viewport_revision == self.state.viewport_revision
                && (state.viewport() != self.state.viewport()
                    || state.axis_windows() != self.state.axis_windows()))
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Controlled response regresses the viewport revision.",
            ));
        }
        if state.follow() == FollowMode::FreezePresentation && self.frozen.is_none() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "A controlled freeze requires an existing captured scene; dispatch freeze explicitly.",
            ));
        }
        if state.pinned() != self.state.pinned() {
            self.check_targets(
                state.durable.pinned.as_slice(),
                self.presented().map(|s| s.scene().stamp()),
                false,
                false,
            )?;
            self.pinned = if state.pinned().is_some() {
                self.presented().cloned()
            } else {
                None
            };
        }
        self.state = state;
        self.history.clear();
        self.redo.clear();
        if self.state.follow() != FollowMode::FreezePresentation {
            self.frozen = None;
        }
        self.prune_indexes();
        Ok(true)
    }
    /// Release all scene handles and transient state on owner disposal. Durable state is retained.
    pub fn dispose(&mut self, definition: &ChartDefinition) -> ChartResult<DispatchOutcome> {
        let request = self.request(
            definition,
            ChartAction::CancelGesture(CancelReason::Disposed),
            ActionOrigin::Programmatic,
        );
        let result = self.dispatch(definition, request);
        self.state.active = None;
        self.state.hover.clear();
        self.state.focus = None;
        self.presented = None;
        self.frozen = None;
        self.pinned = None;
        self.current_source = None;
        self.followed_source = None;
        self.gesture_basis = None;
        self.history.clear();
        self.redo.clear();
        self.links.clear();
        self.disposed = true;
        self.target_indexes.clear();
        result
    }
}
fn validate_config(c: &InteractionConfig) -> ChartResult<()> {
    if c.max_targets == 0 || c.max_targets > 4096 || c.history_capacity > 128 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Interaction target/history capacities exceed supported bounds.",
        ));
    }
    Ok(())
}
fn validate_action_bounds(action: &ChartAction, maximum: usize) -> ChartResult<()> {
    use ChartAction::*;
    let targets = match action {
        SetHover(v)
        | Select { targets: v, .. }
        | PreviewGesture {
            preview: GesturePreview::Selection(v),
            ..
        } => v.as_slice(),
        SetFocus(v) | SetPinned(v) => v.as_slice(),
        Synchronize {
            selection: Some(v), ..
        } => v.as_slice(),
        _ => &[],
    };
    validate_targets(targets, maximum)?;
    if let Configure(c) = action {
        validate_config(c)?;
    }
    Ok(())
}
fn validate_targets(targets: &[MarkTarget], maximum: usize) -> ChartResult<()> {
    if targets.len() > maximum {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Action exceeds the semantic target budget.",
        ));
    }
    for t in targets {
        let mut bytes = match &t.identity {
            TargetIdentity::Source { .. } => 0,
            TargetIdentity::HierarchyNode { node, .. } => match node {
                crate::hierarchy::HierarchyTargetKey::Source(_) => 0,
                crate::hierarchy::HierarchyTargetKey::Synthetic(path) => path.len(),
            },
            TargetIdentity::Aggregate { group, .. } => group.len(),
            TargetIdentity::Derived {
                model, datasets, ..
            } => {
                if datasets.len() > 64 {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Target input scope exceeds 64 datasets.",
                    ));
                }
                model.len()
            }
        };
        if let Some(panel) = &t.panel {
            if panel.values.len() > 64 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Target panel exceeds 64 fields.",
                ));
            }
            for value in &panel.values {
                if let crate::grammar::GroupValue::Text(text) = value {
                    bytes = bytes.saturating_add(text.len());
                }
            }
        }
        if bytes > 8192 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Target identity exceeds its metadata budget.",
            ));
        }
    }
    if targets.iter().collect::<BTreeSet<_>>().len() != targets.len() {
        return Err(error(
            DiagnosticCode::Validation,
            "Selection/inspection action targets must be distinct.",
        ));
    }
    Ok(())
}
fn validate_annotation(d: &ChartDefinition, a: &Annotation) -> ChartResult<()> {
    fn anchor(a: &Anchor, d: &ChartDefinition) -> ChartResult<()> {
        if let Anchor::Data {
            panel: Some(panel), ..
        }
        | Anchor::Panel {
            panel: Some(panel), ..
        } = a
            && (panel.values.len() > 64
                || panel
                    .values
                    .iter()
                    .filter_map(|v| match v {
                        crate::grammar::GroupValue::Text(s) => Some(s.len()),
                        _ => None,
                    })
                    .fold(0usize, usize::saturating_add)
                    > 8192)
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Annotation panel exceeds its metadata budget.",
            ));
        }
        let valid = match a {
            Anchor::Output { x, y } | Anchor::Figure { x, y } | Anchor::Panel { x, y, .. } => {
                x.is_finite() && y.is_finite()
            }
            Anchor::Data { scales, x, y, .. } => {
                let value = |v: &ScaleValue| match v {
                    ScaleValue::Number(v) => v.is_finite(),
                    ScaleValue::Category(s) => s.len() <= 8192,
                    ScaleValue::Timestamp { .. } | ScaleValue::MissingCategory => true,
                };
                let ids: BTreeSet<_> = if d.axes.is_empty() {
                    [crate::ScaleId::new(0), crate::ScaleId::new(1)]
                        .into_iter()
                        .collect()
                } else {
                    d.axes.iter().map(|a| a.id).collect()
                };
                ids.contains(&scales.x) && ids.contains(&scales.y) && value(x) && value(y)
            }
        };
        if !valid {
            return Err(error(
                DiagnosticCode::Validation,
                "Annotation requires finite explicit values and current named scales.",
            ));
        }
        Ok(())
    }
    a.text.validate(Limits::default())?;
    if a.id.len() > 256 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Annotation identity exceeds 256 bytes.",
        ));
    }
    anchor(&a.anchor, d)?;
    if let Some(p) = &a.callout {
        anchor(p, d)?;
    }
    FigureComposition {
        annotations: vec![a.clone()],
        ..Default::default()
    }
    .validate(Limits::default())
}
pub(super) fn validate_state(d: &ChartDefinition, s: &ChartState) -> ChartResult<()> {
    s.durable.viewport.validate()?;
    windows::validate_windows(&s.durable.windows)?;
    validate_config(&s.configuration)?;
    let r = &s.revisions;
    if [
        r.durable,
        r.follow,
        r.hover,
        r.focus,
        r.pin,
        r.selection,
        r.visibility,
        r.annotations,
        r.gesture,
        r.configuration,
    ]
    .into_iter()
    .any(|v| v > s.revision)
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Component revisions exceed the total state revision.",
        ));
    }
    validate_targets(s.durable.pinned.as_slice(), 1)?;
    validate_targets(
        &s.durable.selection.iter().cloned().collect::<Vec<_>>(),
        s.configuration.max_targets,
    )?;
    if s.viewport_revision > s.revision
        || s.durable
            .hidden
            .iter()
            .any(|id| !d.layers.iter().any(|l| l.id == *id))
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Controlled state has invalid revisions or layer identities.",
        ));
    }
    for (id, a) in &s.durable.annotations {
        if id.is_empty() || id.len() > 256 || id.chars().any(char::is_control) {
            return Err(error(
                DiagnosticCode::Validation,
                "Invalid annotation override identity.",
            ));
        }
        if let Some(a) = a {
            if id != &a.id {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Annotation map identity disagrees with its value.",
                ));
            }
            validate_annotation(d, a)?;
        }
    }
    if s.durable.annotations.len() > 256 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Too many annotation overrides.",
        ));
    }
    if let Some(f) = s.figure(d) {
        f.validate(Limits::default())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disposal_is_terminal_even_when_cancellation_cannot_advance_a_revision() {
        let d = ChartDefinition::new(Revision::INITIAL);
        let mut reducer = ActionReducer::default();
        reducer.state.revision = Revision::new(u64::MAX);
        reducer.state.active = Some(ActiveGesture {
            id: Revision::new(1),
            kind: GestureKind::Viewport,
            basis: SceneStamp {
                definition: Revision::INITIAL,
                store: Revision::INITIAL,
                layout: Revision::INITIAL,
                state: Revision::INITIAL,
                viewport: Revision::INITIAL,
            },
            preview: Some(GesturePreview::Viewport(Viewport {
                x: Some((1., 2.)),
                y: None,
            })),
        });
        assert_eq!(
            reducer.dispose(&d).unwrap_err().code,
            DiagnosticCode::RevisionOverflow
        );
        assert!(reducer.state.active.is_none());
        assert_eq!(reducer.state.viewport(), Viewport::default());
        assert_eq!(
            reducer
                .accept_controlled(&d, reducer.state.revision(), reducer.state.clone())
                .unwrap_err()
                .code,
            DiagnosticCode::DisposedHandle
        );
        let request = reducer.request(&d, ChartAction::Reset, ActionOrigin::Control);
        assert_eq!(
            reducer.dispatch(&d, request).unwrap_err().code,
            DiagnosticCode::DisposedHandle
        );
    }
}
