//! Bounded portable descriptions for native accessibility, copy and replaceable data tables.
use crate::data::{FieldKind, ValueRef};
use crate::inspection::{InspectedTarget, Inspector};
use crate::provenance::Target;
use crate::state::{ChartState, MarkTarget};
use crate::{ChartResult, Diagnostic, DiagnosticCode, SceneStamp};
use serde::Serialize;

/// One named source/custom value, preserving exact integer strings and explicit missing values.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct AccessibleCell {
    /// Source label/name or custom semantic field.
    pub field: String,
    /// Explicit presentation value. This string is never used for numerical calculations.
    pub value: String,
    /// Text was bounded; retrieve the full source through its retained snapshot if needed.
    pub truncated: bool,
}
/// Meaningful focus/table row; aggregate/model identities never impersonate source measurements.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct AccessibleTarget {
    /// Exact stable identity accepted by common focus/selection actions.
    pub target: MarkTarget,
    /// Human-readable source/aggregate/model identity and scope.
    pub description: String,
    /// At most 16 fields, each bounded to 512 characters; original source access remains available.
    pub cells: Vec<AccessibleCell>,
    /// Additional source/custom fields omitted from this compact row.
    pub omitted_fields: usize,
    /// Exact aggregate membership count, absent for source/derived targets.
    pub member_count: Option<usize>,
    /// Current effective selection state.
    pub selected: bool,
    /// Current keyboard focus state.
    pub focused: bool,
}
/// Bounded page over deterministic meaningful target order, independent of window entities.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct AccessiblePage {
    /// Snapshot used for descriptions and subsequent fenced actions.
    pub scene: SceneStamp,
    /// Summary includes target count and declared presentation status.
    pub summary: String,
    /// Total meaningful targets, including those outside this page.
    pub total: usize,
    /// Starting target offset.
    pub offset: usize,
    /// Up to 256 rows; pagination does not allocate one native entity per source point.
    pub targets: Vec<AccessibleTarget>,
}
fn bounded(s: &str) -> (String, bool) {
    let mut chars = s.chars();
    let text: String = chars.by_ref().take(512).collect();
    (text, chars.next().is_some())
}
fn cell(field: &str, value: &str) -> AccessibleCell {
    let (field, a) = bounded(field);
    let (value, b) = bounded(value);
    AccessibleCell {
        field,
        value,
        truncated: a || b,
    }
}
impl Inspector {
    /// Describe one presented target for tooltips, announcements and host-owned tables/copy.
    pub fn describe_target(
        &self,
        hit: &InspectedTarget,
        state: &ChartState,
    ) -> ChartResult<AccessibleTarget> {
        let source = self.presented().prepared().source().get()?;
        let target = MarkTarget::from_inspected(hit, source.epoch());
        let hit = self
            .target(&target)?
            .ok_or_else(|| invalid("Description target is not present in this scene."))?;
        let mut cells = vec![];
        let mut omitted_fields = 0;
        let mut member_count = None;
        let description = match &hit.target {
            Target::Source(s) => {
                let dataset = source.dataset(s.dataset)?;
                let row = dataset.row(s.key).ok_or_else(|| {
                    invalid("Presented source target is absent from its own snapshot.")
                })?;
                let fields = dataset.schema().fields();
                omitted_fields = fields.len().saturating_sub(16);
                for field in fields.iter().take(16) {
                    let name = field.label.as_deref().unwrap_or(&field.name);
                    let value = match row.value(field.id) {
                        None => "Missing".into(),
                        Some(ValueRef::Float64(v)) if !v.is_finite() => {
                            "Invalid non-finite value".into()
                        }
                        Some(ValueRef::Float64(v)) => v.to_string(),
                        Some(ValueRef::Int64(v)) | Some(ValueRef::Timestamp(v)) => v.to_string(),
                        Some(ValueRef::UInt64(v)) => v.to_string(),
                        Some(ValueRef::Boolean(v)) => v.to_string(),
                        Some(ValueRef::Utf8(v)) | Some(ValueRef::Category(v)) => bounded(v).0,
                    };
                    let mut c = cell(name, row.formatted(field.id).unwrap_or(&value));
                    if let Some(ValueRef::Utf8(v) | ValueRef::Category(v)) = row.value(field.id) {
                        c.truncated |= v.chars().take(513).count() > 512;
                    }
                    if let FieldKind::Timestamp(t) = &field.kind {
                        c.value
                            .push_str(&format!(" {:?} {}", t.unit, bounded(&t.timezone).0));
                    } else if let Some(unit) = &field.unit {
                        c.value.push_str(&format!(" {}", bounded(unit).0));
                    }
                    cells.push(c);
                }
                format!("Source row {} in dataset {}", s.key.get(), s.dataset.get())
            }
            Target::Aggregate {
                id,
                group,
                input,
                members,
            } => {
                if source.dataset(input.dataset)?.version() != *input {
                    return Err(invalid(
                        "Aggregate description requires its exact input revision.",
                    ));
                }
                member_count = Some(members.len());
                format!(
                    "Aggregate {}: {}; {} source members; input revision {}",
                    id.get(),
                    bounded(group).0,
                    members.len(),
                    input.revision.get()
                )
            }
            Target::Derived {
                id,
                model,
                model_version,
                inputs,
            } => {
                for input in inputs {
                    if source.dataset(input.dataset)?.version() != *input {
                        return Err(invalid(
                            "Derived description requires its exact input revisions.",
                        ));
                    }
                }
                format!(
                    "Derived {} from model {} version {}; {} input datasets",
                    id.get(),
                    bounded(model).0,
                    model_version.get(),
                    inputs.len()
                )
            }
        };
        let available = 16usize.saturating_sub(cells.len());
        for (name, value) in hit.values.iter().take(available) {
            let value = serde_json::to_string(value)
                .map_err(|_| invalid("Custom semantic value cannot be described."))?;
            cells.push(cell(name, &value));
        }
        omitted_fields += hit.values.len().saturating_sub(available);
        Ok(AccessibleTarget {
            description,
            selected: state.selection().contains(&target),
            focused: state.focus() == Some(&target),
            target,
            cells,
            omitted_fields,
            member_count,
        })
    }
    /// Produce a bounded accessible data alternative; pages never silently claim to be the full table.
    pub fn accessible_page(
        &self,
        state: &ChartState,
        offset: usize,
        limit: usize,
    ) -> ChartResult<AccessiblePage> {
        let total = self.semantic_targets().len();
        if !(1..=256).contains(&limit) || offset > total {
            return Err(invalid(
                "Accessible pages require limit 1..256 and an offset within the target count.",
            ));
        }
        let targets = self
            .semantic_targets()
            .skip(offset)
            .take(limit)
            .map(|h| self.describe_target(h, state))
            .collect::<ChartResult<_>>()?;
        Ok(AccessiblePage {
            scene: self.presented().scene().stamp(),
            summary: format!(
                "Chart with {total} visible semantic targets; {} selected; {:?}.",
                state.selection().len(),
                self.presented().status()
            ),
            total,
            offset,
            targets,
        })
    }
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use the retained presented target/snapshot and bounded page options.",
    )
}
