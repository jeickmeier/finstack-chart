//! Versioned guide policy and independent semantic formatter descriptions.
use crate::{
    grammar::OperationRef,
    scales::{GgplotTimeFormat, TimeFormat},
    typography::NumericFormat,
};

/// Guide policy is a destination presentation choice and never changes scale training.
/// It travels in the common chart definition and primary authoring envelope.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GuideProfile {
    /// Existing adaptive guide defaults and legacy automatic tick policies.
    #[default]
    LibraryV1,
    /// d3-axis 3.0.0 guide policy over the explicitly selected shared scale family.
    D3_3_0_0,
}
impl GuideProfile {
    pub(super) fn is_legacy(&self) -> bool {
        *self == Self::LibraryV1
    }
}

/// Formatting is independent of selected values. `None` restores scale-default labels.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GuideFormatter {
    /// Exact labels in selected-value order; its length must equal the complete list.
    Labels(Vec<String>),
    /// Shared numeric specifier/locale, with precision inferred from the tick arguments.
    Numeric(Box<NumericFormat>),
    /// Shared calendar formatting using the scale's explicit calendar/zone.
    Time(Box<TimeFormat>),
    /// Explicit R date_labels patterns with supplied calendar and locale resources.
    GgplotTime(Box<GgplotTimeFormat>),
    /// Native semantic callback installed under a captured versioned identity.
    Registered {
        /// Exact installed implementation identity.
        operation: OperationRef,
        /// Bounded declarative parameters, never executable source.
        parameters: serde_json::Value,
    },
}

/// One nested destination in a guide snapshot's complete scope path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum GuideScope {
    /// Exact facet key, independent of row/column ordering.
    Panel(crate::grammar::PanelKey),
    /// Authored inset identity in its enclosing figure/panel.
    Inset(String),
}

/// Coherent guide configuration and selected semantic ticks from one immutable layout.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuideSnapshot {
    /// Minor values and positions from the same immutable layout.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub minor_ticks: Vec<MinorGuideTick>,
    /// Nested facet/inset destinations; empty for the root chart.
    pub scope: Vec<GuideScope>,
    /// Stable guide/scale identity and effective authored presentation controls.
    pub spec: super::GuideSpec,
    /// Original values, order, labels and resolved positions from this same layout.
    pub ticks: Vec<super::GuideTick>,
}

/// Reference minor-break selection. Selection is retained independently from
/// major labels and does not change scale training or mark coordinates.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MinorBreaks {
    /// Subdivide the reference major intervals in transformed coordinates.
    Automatic,
    /// Select no minor breaks.
    Hidden,
    /// Explicit raw numeric values; missing/nonfinite transformed positions are omitted.
    Numeric(Vec<crate::interpolate::Number>),
    /// Explicit timestamp candidates in source units; missing entries are omitted.
    /// Every present value must be a timestamp with the axis source unit.
    Timestamps(Vec<Option<crate::composition::ScaleValue>>),
    /// Date, datetime or elapsed-time width using the shared reference alignment rules.
    TimeWidth(String),
}

/// One drawable minor candidate. A transformed expansion can have a valid position
/// without a finite raw inverse (for example below zero on a square-root axis).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MinorGuideTick {
    /// Finite raw value when the transformation has one; missing otherwise.
    pub value: Option<crate::composition::ScaleValue>,
    /// Finite destination position, independent of major-label thinning.
    pub position: f64,
}
