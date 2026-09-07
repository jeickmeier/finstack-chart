use super::*;
use crate::composition::{Anchor, ScaleValue};
use crate::grammar::SelectionPolicy;
use crate::layout::LaidOutChart;
use crate::{Limits, SceneStamp};
use std::{collections::VecDeque, sync::Arc};

#[derive(Clone, Debug)]
struct Command {
    before: BTreeMap<String, Option<Annotation>>,
    after: BTreeMap<String, Option<Annotation>>,
}
#[derive(Clone, Debug)]
struct LinkRecord {
    revision: Revision,
    viewport: Option<Viewport>,
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
    history: VecDeque<Command>,
    redo: Vec<Command>,
    links: BTreeMap<String, LinkRecord>,
    last_gesture: Revision,
    disposed: bool,
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
            history: VecDeque::new(),
            redo: Vec::new(),
            links: BTreeMap::new(),
            last_gesture: Revision::INITIAL,
            disposed: false,
        }
    }
    /// Current immutable semantic state for preparation and application observation.
    pub fn state(&self) -> &ChartState {
        &self.state
    }
    /// Acknowledge the scene actually submitted by the host. Active gestures keep their own basis.
    pub fn present(&mut self, scene: Arc<LaidOutChart>) {
        if !self.disposed {
            self.presented = Some(scene);
        }
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
    /// Bounded undo/redo command counts; transient actions never add commands.
    pub fn history_lengths(&self) -> (usize, usize) {
        (self.history.len(), self.redo.len())
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
        let changed = *before != next.state;
        let viewport_changed = before.viewport() != next.state.viewport();
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
                origin: request.origin.clone(),
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
        let epoch = chart.prepared().source().get()?.epoch();
        let mut available = BTreeSet::new();
        for (i, (item, values)) in chart
            .scene()
            .items()
            .iter()
            .zip(chart.targets())
            .enumerate()
        {
            if selectable
                && chart
                    .interactions()
                    .get(&i)
                    .is_some_and(|v| v.selection == SelectionPolicy::Disabled)
            {
                continue;
            }
            let Some(layer) = item.layer else {
                continue;
            };
            for target in values {
                available.insert(MarkTarget {
                    epoch,
                    layer,
                    panel: chart.item_panels()[i].clone(),
                    identity: target.into(),
                });
            }
        }
        if targets.iter().any(|t| !available.contains(t)) {
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
                    GesturePreview::Viewport(v) => *v == committed.viewport(),
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
                        if *viewport != last.viewport || *selection != last.selection {
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
                if let Some(v) = viewport {
                    v.validate()?;
                    self.state.durable.viewport = *v;
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
                && state.viewport() != self.state.viewport())
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
        self.state = state;
        self.history.clear();
        self.redo.clear();
        if self.state.follow() != FollowMode::FreezePresentation {
            self.frozen = None;
        }
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
        self.gesture_basis = None;
        self.history.clear();
        self.redo.clear();
        self.links.clear();
        self.disposed = true;
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
                    ScaleValue::Timestamp { .. } => true,
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
