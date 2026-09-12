//! Portable decoration identity, independent of data interaction targets.
use crate::{ChartResult, DiagnosticCode, GuideId, composition::ScaleValue};

/// Stable semantic tick identity; equal values retain distinct occurrences.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct GuideTickIdentity {
    /// Original number, exact timestamp with unit, or category.
    pub value: ScaleValue,
    /// Zero-based occurrence among equal original values in selection order.
    pub occurrence: usize,
}
/// Addressable axis component role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum GuideRole {
    /// The range-derived path, including outer caps.
    Domain,
    /// The line belonging to one semantic tick group.
    Line,
    /// Plain or shaped logical label belonging to one tick group.
    Label,
}
/// Retained component metadata on the exact scene primitive consumed by every host.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct GuideComponent {
    /// Retained sampled group identity and continuous opacity, absent in a final static scene.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation: Option<GuideAnimation>,
    /// Nested panel/inset scope, so repeated guide IDs do not merge across views.
    pub scope: Vec<String>,
    /// Independent guide identity, not a positional scale identity.
    pub guide: GuideId,
    /// Primitive role within the guide/tick hierarchy.
    pub role: GuideRole,
    /// Retained side for orientation replacement and portable presentation.
    pub side: crate::layout::AxisSide,
    /// Absent only for the domain path.
    pub tick: Option<GuideTickIdentity>,
    /// Original selected-list index before hiding/thinning.
    pub index: Option<usize>,
    /// Full logical label, including an explicitly blank label.
    pub label: Option<String>,
}
impl GuideComponent {
    pub(super) fn validate(&self, remaining: &mut usize) -> ChartResult<()> {
        let domain = self.role == GuideRole::Domain;
        if self
            .animation
            .is_some_and(|a| domain || !a.opacity.is_finite() || !(0. ..=1.).contains(&a.opacity))
        {
            return Err(crate::scales::error(
                DiagnosticCode::Validation,
                "Invalid guide animation metadata.",
            ));
        }
        if domain != self.tick.is_none()
            || domain != self.index.is_none()
            || domain != self.label.is_none()
        {
            return Err(crate::scales::error(
                DiagnosticCode::Validation,
                "Guide component role and tick metadata disagree.",
            ));
        }
        crate::limits::require_within(self.scope.len() <= 64, "guide component scope depth")?;
        let mut bytes = self.label.as_ref().map_or(0, String::len);
        for scope in &self.scope {
            bytes = bytes.saturating_add(scope.len());
        }
        if let Some(tick) = &self.tick {
            match &tick.value {
                ScaleValue::Category(s) => bytes = bytes.saturating_add(s.len()),
                ScaleValue::Number(n) if !n.is_finite() => {
                    return Err(crate::scales::error(
                        DiagnosticCode::Validation,
                        "Guide component value must be finite.",
                    ));
                }
                _ => {}
            }
        }
        crate::limits::require_within(bytes <= *remaining, "guide component metadata bytes")?;
        *remaining -= bytes;
        Ok(())
    }
}

/// Presentation metadata for one sampled tick group (scene wire v15).
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct GuideAnimation {
    /// Stable per-guide lifecycle node, distinct from semantic value/occurrence.
    pub identity: usize,
    /// Continuous group alpha, in `[0, 1]`, before component color alpha.
    pub opacity: f64,
}
