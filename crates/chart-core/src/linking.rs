//! Linked views exchange explicit scale meanings and provenance identities, never pixels.
use crate::data::{DatasetVersion, TimestampType};
use crate::grammar::{PanelKey, ValueSpace};
use crate::inspection::{InspectedTarget, Inspector};
use crate::layout::{LaidOutChart, ResolvedAxis};
use crate::navigation::{Navigation, NavigationBoundary, Navigator};
use crate::provenance::Target;
use crate::state::{
    ActionOrigin, AxisWindow, AxisWindows, ChartAction, ChartState, MarkTarget, StateEvent,
    TargetIdentity,
};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Revision, ScaleId, SourceEpoch};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Semantic identity independent of a receiving chart's layer/panel choices.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct LinkedTarget {
    /// Explicit source universe shared by the application.
    pub epoch: SourceEpoch,
    /// Source, aggregate or derived identity; these kinds never substitute for each other.
    pub identity: TargetIdentity,
    /// Exact aggregate/model inputs; source-row identity deliberately survives corrections.
    pub inputs: Vec<DatasetVersion>,
}
impl LinkedTarget {
    /// Preserve model/aggregate revisions while removing presentation-specific layer/panel IDs.
    pub fn from_hit(hit: &InspectedTarget, epoch: SourceEpoch) -> Self {
        Self::from_target(&hit.target, epoch)
    }
    fn from_target(target: &Target, epoch: SourceEpoch) -> Self {
        Self {
            epoch,
            identity: target.into(),
            inputs: match target {
                Target::Source(_) => vec![],
                Target::Aggregate { input, .. } => vec![*input],
                Target::Derived { inputs, .. } => inputs.clone(),
            },
        }
    }
}
/// Portable calculation-space compatibility, excluding category catalog and relative time origin.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum AxisMeaning {
    /// Canonical core calculation-space descriptor. No implicit unit conversion is performed.
    Numeric(String),
    /// Exact source timestamp representation; origins may differ between views.
    Timestamp(TimestampType),
    /// Decoded stable labels; the receiver checks requested endpoints against its own catalog.
    Category,
}
fn meaning(axis: &ResolvedAxis) -> ChartResult<AxisMeaning> {
    Ok(match &axis.space {
        ValueSpace::Categorical { .. } => AxisMeaning::Category,
        ValueSpace::Timestamp { representation, .. } => {
            AxisMeaning::Timestamp(representation.clone())
        }
        other => AxisMeaning::Numeric(
            serde_json::to_string(other).map_err(|_| invalid("Invalid scale descriptor."))?,
        ),
    })
}
/// One linked semantic window in its sender's axis namespace.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LinkedWindow {
    /// Sender axis ID. The receiver supplies an explicit axis mapping.
    pub axis: ScaleId,
    /// Calculation semantics; incompatible transforms/representations reject.
    pub meaning: AxisMeaning,
    /// Source-unit values or category identities.
    pub window: AxisWindow,
}
/// Immutable root-origin message. Forward this unchanged, or suppress linked-origin events.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LinkMessage {
    /// Stable application view ID, 1..128 bytes.
    pub origin: String,
    /// Monotonic effective state revision in that view's lifetime.
    pub revision: Revision,
    /// Optional semantic windows; no pixels are serialized.
    pub windows: Vec<LinkedWindow>,
    /// Explicit selection, including an empty clear. None leaves receiver selection alone.
    pub selection: Option<Vec<LinkedTarget>>,
}
/// Explicit sender-to-receiver axis pairing; pairing asserts that application units agree.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisLink {
    /// Sender axis identity.
    pub source: ScaleId,
    /// Receiver axis identity.
    pub destination: ScaleId,
}
/// Explicit behavior for selection identities absent from a receiving view.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MissingMatch {
    /// Reject the whole message without changing any state.
    Reject,
    /// Select only exact matches and report every missing identity.
    ReportAndOmit,
}
/// Prepared common action and explicit unmatched selection report. Dispatch atomically afterward.
#[derive(Debug)]
pub struct LinkedUpdate {
    /// Common reducer action with the original origin/revision.
    pub action: ChartAction,
    /// Pair with action on the receiving reducer.
    pub origin: ActionOrigin,
    /// No placeholder/source substitution is made for these identities.
    pub unmatched: Vec<LinkedTarget>,
}
fn axis<'a>(
    chart: &'a LaidOutChart,
    id: ScaleId,
    panel: Option<&PanelKey>,
) -> ChartResult<&'a ResolvedAxis> {
    let chart = match panel {
        Some(key) => {
            &chart
                .panels()
                .iter()
                .find(|p| &p.key == key)
                .ok_or_else(|| invalid("Linked panel is absent."))?
                .chart
        }
        None => chart,
    };
    chart
        .axes()
        .get(&id)
        .ok_or_else(|| invalid("Linked axis is absent; provide an explicit panel/axis mapping."))
}
impl LinkMessage {
    /// Capture current state after a non-linked effective event; stamp the current state revision. Linked echoes return None instead of a new origin.
    /// Window capture requires acknowledgment of that viewport; selection can use the current frame.
    pub fn from_event(
        origin: &str,
        event: &StateEvent,
        inspector: &Inspector,
        state: &ChartState,
        axes: &[ScaleId],
        panel: Option<&PanelKey>,
        include_selection: bool,
    ) -> ChartResult<Option<Self>> {
        if matches!(event.origin, ActionOrigin::Linked(_)) || !event.durable {
            return Ok(None);
        }
        if event.revision > state.revision() {
            return Err(invalid(
                "Link trigger event is newer than the captured state.",
            ));
        }
        if axes.len() > 64 || axes.iter().collect::<BTreeSet<_>>().len() != axes.len() {
            return Err(invalid("Link capture supports at most 64 distinct axes."));
        }
        let chart = inspector.presented();
        if !axes.is_empty()
            && chart.prepared().state().viewport_revision() != state.viewport_revision()
        {
            return Err(Diagnostic::error(
                DiagnosticCode::Superseded,
                "Linked viewport has not been presented.",
                "Acknowledge the displayed viewport before capturing its semantic window.",
            ));
        }
        let mut windows = vec![];
        if !axes.is_empty() {
            let values = Navigator::new(chart.clone()).navigate(
                chart.scene().stamp(),
                axes,
                panel,
                Navigation::Zoom {
                    anchor: Point::new(0., 0.)?,
                    factor: 1.,
                },
                NavigationBoundary::Extend,
            )?;
            for id in axes {
                windows.push(LinkedWindow {
                    axis: *id,
                    meaning: meaning(axis(chart, *id, panel)?)?,
                    window: values[id].clone(),
                });
            }
        }
        let epoch = chart.prepared().source().get()?.epoch();
        let selection = if include_selection {
            let selected = state.selection();
            let mut remaining = selected.clone();
            let mut targets = BTreeSet::new();
            for entry in chart.prepared().semantic_targets() {
                let mark = MarkTarget {
                    epoch,
                    layer: entry.layer,
                    panel: entry.panel.cloned(),
                    identity: entry.target.into(),
                };
                if remaining.remove(&mark) {
                    targets.insert(LinkedTarget::from_target(entry.target, epoch));
                }
            }
            if !remaining.is_empty() {
                return Err(invalid(
                    "Selected identities are absent from the captured presented scene.",
                ));
            }
            Some(targets.into_iter().collect())
        } else {
            None
        };
        let result = Self {
            origin: origin.into(),
            revision: state.revision(),
            windows,
            selection,
        };
        result.validate()?;
        Ok(Some(result))
    }
    fn validate(&self) -> ChartResult<()> {
        if self.origin.is_empty()
            || self.origin.len() > 128
            || self.origin.chars().any(char::is_control)
            || self.revision == Revision::INITIAL
        {
            return Err(invalid(
                "Linked origin must be bounded and revision positive.",
            ));
        }
        if self.windows.len() > 64 || self.selection.as_ref().is_some_and(|v| v.len() > 4096) {
            return Err(invalid("Linked window/selection count exceeds its budget."));
        }
        if self
            .windows
            .iter()
            .map(|w| w.axis)
            .collect::<BTreeSet<_>>()
            .len()
            != self.windows.len()
        {
            return Err(invalid("Linked axes must be unique."));
        }
        let windows: AxisWindows = self
            .windows
            .iter()
            .map(|w| (w.axis, w.window.clone()))
            .collect();
        crate::state::validate_navigation_windows(&windows)?;
        for w in &self.windows {
            let n = match &w.meaning {
                AxisMeaning::Numeric(s) => s.len(),
                AxisMeaning::Timestamp(t) => t.timezone.len(),
                AxisMeaning::Category => 0,
            };
            if n > 8192 {
                return Err(invalid(
                    "Linked calculation-space metadata exceeds its budget.",
                ));
            }
        }
        if let Some(targets) = &self.selection {
            for t in targets {
                let size = match &t.identity {
                    TargetIdentity::Source { .. } => 0,
                    TargetIdentity::Aggregate { group, .. } => group.len(),
                    TargetIdentity::Derived {
                        model, datasets, ..
                    } => model.len() + datasets.len().saturating_mul(8),
                };
                if size > 8192 || t.inputs.len() > 64 {
                    return Err(invalid("Linked target metadata exceeds its budget."));
                }
            }
        }
        Ok(())
    }
    /// Match exact provenance and compatible declared axis spaces in a receiving scene.
    /// No state changes occur until the returned action is dispatched with the receiver's fences.
    pub fn resolve(
        &self,
        inspector: &Inspector,
        mappings: &[AxisLink],
        panel: Option<&PanelKey>,
        missing: MissingMatch,
    ) -> ChartResult<LinkedUpdate> {
        self.validate()?;
        if mappings.len() > 64
            || mappings
                .iter()
                .map(|m| m.destination)
                .collect::<BTreeSet<_>>()
                .len()
                != mappings.len()
        {
            return Err(invalid(
                "Linked axis destinations must be distinct and bounded.",
            ));
        }
        let chart = inspector.presented();
        let mut windows = AxisWindows::new();
        for w in &self.windows {
            let m = mappings
                .iter()
                .find(|m| m.source == w.axis)
                .ok_or_else(|| {
                    invalid("Every linked window needs an explicit destination axis.")
                })?;
            if meaning(axis(chart, m.destination, panel)?)? != w.meaning {
                return Err(Diagnostic::error(
                    DiagnosticCode::SchemaConflict,
                    "Linked scale meanings are incompatible.",
                    "Use matching calculation units, transforms and timestamp representations.",
                ));
            }
            Navigator::new(chart.clone()).set_range(
                chart.scene().stamp(),
                m.destination,
                panel,
                w.window.clone(),
            )?;
            windows.insert(m.destination, w.window.clone());
        }
        let epoch = chart.prepared().source().get()?.epoch();
        let mut unmatched = vec![];
        let selection = if let Some(wanted) = &self.selection {
            let wanted: BTreeSet<_> = wanted.iter().cloned().collect();
            let mut remaining = wanted.clone();
            let mut selected = BTreeSet::new();
            for entry in chart
                .prepared()
                .semantic_targets()
                .filter(|entry| entry.selectable)
            {
                let key = LinkedTarget::from_target(entry.target, epoch);
                if wanted.contains(&key) {
                    remaining.remove(&key);
                    selected.insert(MarkTarget {
                        epoch,
                        layer: entry.layer,
                        panel: entry.panel.cloned(),
                        identity: entry.target.into(),
                    });
                }
            }
            if !remaining.is_empty() && missing == MissingMatch::Reject {
                return Err(invalid(
                    "Linked target is absent or belongs to another input revision.",
                ));
            }
            unmatched.extend(remaining);
            Some(selected.into_iter().collect())
        } else {
            None
        };
        Ok(LinkedUpdate {
            action: ChartAction::Synchronize {
                revision: self.revision,
                viewport: None,
                windows: (!self.windows.is_empty()).then_some(windows),
                selection,
            },
            origin: ActionOrigin::Linked(self.origin.clone()),
            unmatched,
        })
    }
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use an explicit bounded mapping, shared source universe and unchanged root origin/revision.",
    )
}
