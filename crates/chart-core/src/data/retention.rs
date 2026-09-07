use super::{DatasetSnapshot, FieldKind, NormalizedBatch, ValueRef, error};
use crate::{ChartResult, DiagnosticCode, FieldId};

/// Policy for incoming observations older than the retained event-time horizon.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LateDataPolicy {
    /// Reject the whole transaction; no observations are accepted or lost.
    #[default]
    Reject,
    /// Explicitly discard only late incoming rows, reporting their count in the receipt.
    Drop,
}

/// Event-time retention in the declared timestamp field's integer ticks.
/// Keep timestamps >= watermark - width - allowed_lateness (inclusive lower boundary).
/// The application supplies the watermark; incoming values never advance it.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EventTimeWindow {
    /// Required non-null timestamp field. Its unit/timezone is defined by the dataset schema.
    pub field: FieldId,
    /// Strictly positive window width in source ticks.
    #[serde(with = "crate::portable::signed")]
    pub width: i64,
    /// Nonnegative extra historical interval retained for late/corrected observations.
    #[serde(with = "crate::portable::signed")]
    pub allowed_lateness: i64,
    /// Explicit application event-time watermark in source ticks; cannot regress in one policy.
    #[serde(with = "crate::portable::signed")]
    pub watermark: i64,
    /// Rejection is lossless; dropping is explicit and counted.
    #[serde(default)]
    pub late: LateDataPolicy,
}
impl EventTimeWindow {
    pub(crate) fn validate(&self, data: &DatasetSnapshot) -> ChartResult<()> {
        if self.width <= 0 || self.allowed_lateness < 0 {
            return Err(error(
                DiagnosticCode::Validation,
                "Event-time width must be positive and permitted lateness nonnegative.",
            ));
        }
        if !data
            .schema()
            .field(self.field)
            .is_some_and(|(_, f)| matches!(f.kind, FieldKind::Timestamp(_)))
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Event-time retention requires a timestamp field in the current schema.",
            ));
        }
        Ok(())
    }
    pub(crate) fn cutoff(self) -> i128 {
        i128::from(self.watermark) - i128::from(self.width) - i128::from(self.allowed_lateness)
    }
    /// Number of incoming rows discarded under the explicit late-drop policy.
    pub(crate) fn filter(self, batch: &NormalizedBatch) -> ChartResult<(NormalizedBatch, usize)> {
        let column = batch.column(self.field).ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Incoming batch omits the retention timestamp field.",
            )
        })?;
        let mut keep = Vec::with_capacity(batch.len());
        for i in 0..batch.len() {
            let Some(ValueRef::Timestamp(t)) = column.value(i) else {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Event-time retention requires valid timestamps; null timestamps cannot be retained indefinitely.",
                ));
            };
            if i128::from(t) < self.cutoff() {
                if self.late == LateDataPolicy::Reject {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Incoming observation precedes the explicit event-time horizon; resynchronize or choose an explicit late-drop policy.",
                    ));
                }
            } else {
                keep.push(i);
            }
        }
        let dropped = batch.len() - keep.len();
        Ok((
            if dropped == 0 {
                batch.clone()
            } else {
                batch.selected(&keep)
            },
            dropped,
        ))
    }
}
