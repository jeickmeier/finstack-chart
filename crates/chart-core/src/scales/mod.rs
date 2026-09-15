//! Checked foundational linear, categorical-band and UTC scales.
//! Domain training is separate from visible viewport and destination range projection.

mod band;
mod calendar;
/// Named chromatic catalog, independent from scale normalization.
pub mod chromatic;
mod civil;
mod classifier;
mod color;
mod extended;
mod log_breaks;
pub use log_breaks::ggplot_breaks_log;
mod ggplot;
pub(crate) mod ggplot_approx;
mod ggplot_transform;
pub use ggplot_transform::GgplotTransform;
mod ggplot_binned_breaks;
mod ggplot_bins;
mod ggplot_continuous_guide;
mod ggplot_discrete_guide;
mod ggplot_discrete_position;
pub(crate) mod ggplot_named;
mod ggplot_vector;
pub use ggplot_discrete_position::{GgplotDiscretePosition, PreparedGgplotDiscretePosition};
mod ggplot_position_bins;
pub use ggplot_continuous_guide::{
    GgplotColorbarDisplay, GgplotColorbarOptions, GgplotContinuousGuide,
    GgplotContinuousGuideEntry, GgplotScaleGuide,
};
pub(crate) use ggplot_continuous_guide::{
    forward_null as ggplot_forward_null, forward_values as ggplot_forward_values,
    inverse_values as ggplot_inverse_values, numeric_breaks as ggplot_numeric_breaks,
    transform_numeric_labels,
};
mod ggplot_identity;
pub use ggplot_discrete_guide::{GgplotDiscreteGuide, GgplotDiscreteGuideEntry, GgplotGuideLabels};
pub use ggplot_identity::{GgplotDiscreteIdentity, GgplotNumericIdentity};
mod ggplot_expansion;
mod ggplot_time;
pub(crate) use ggplot_time::{
    aesthetic_width, aligned_dates, aligned_seconds, parse_width, width_seconds,
};
mod ggplot_temporal_guide;
mod ggplot_time_pretty;
pub use ggplot_temporal_guide::{
    GgplotTemporalBreaks, GgplotTemporalGuide, GgplotTemporalGuideArguments,
};
mod secondary;
pub use extended::{ggplot_breaks_duration, ggplot_breaks_extended};
pub(crate) use ggplot::zero_range as ggplot_zero_range;
pub use ggplot::{
    GgplotDiscretePalette, GgplotNumericPalette, GgplotOob, GgplotQualitativeColors,
    GgplotScalePolicy, ggplot_color_default, ggplot_color_ordinal, ggplot_numeric_default,
    ggplot_numeric_ordinal,
};
pub use ggplot_bins::{GgplotBinnedPalette, GgplotBinnedPolicy, GgplotBreaks};
pub use ggplot_expansion::GgplotExpansion;
pub use ggplot_position_bins::{GgplotBinnedPosition, PreparedGgplotBinnedPosition};
pub use ggplot_time::{ggplot_breaks_duration_width, ggplot_breaks_seconds, ggplot_breaks_width};
pub use ggplot_time_pretty::{
    GgplotTimeBreaks, ggplot_breaks_pretty_date, ggplot_breaks_pretty_time,
};
pub(crate) use secondary::secondary_mapping;
mod interpolated;
mod linear;
mod mapped;
mod nonlinear;
mod numeric;
mod ordinal;
mod provider;
mod session;
mod spacing;
mod standalone;
mod standalone_authoring;
pub(crate) mod ticks;
mod time;
mod time_format;
mod utc;

pub use band::*;
pub use calendar::*;
pub use classifier::*;
pub use color::*;
pub use interpolated::*;
pub use linear::*;
pub use mapped::*;
pub use nonlinear::*;
pub use numeric::*;
pub use ordinal::*;
pub use provider::*;
pub use session::*;
pub use spacing::{BandSpec, CategoryScale, PointSpec};
pub use standalone::*;
pub use standalone_authoring::*;
pub use ticks::{log_tick_candidates, log_tick_format, tick_candidates, tick_step};
pub use time::*;
pub use time_format::*;
pub use utc::*;

use crate::{ChartResult, Diagnostic, DiagnosticCode};

/// Finite ordered or descending endpoints; equality is allowed before domain resolution.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    start: f64,
    end: f64,
}
impl Bounds {
    /// Validate finite endpoints without imposing direction or silently expanding constants.
    pub fn new(start: f64, end: f64) -> ChartResult<Self> {
        if !start.is_finite() || !end.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Scale endpoints must be finite.",
            ));
        }
        Ok(Self { start, end })
    }
    /// First endpoint in authored orientation.
    pub fn start(self) -> f64 {
        self.start
    }
    /// Second endpoint in authored orientation.
    pub fn end(self) -> f64 {
        self.end
    }
    /// Smaller endpoint.
    pub fn minimum(self) -> f64 {
        self.start.min(self.end)
    }
    /// Larger endpoint.
    pub fn maximum(self) -> f64 {
        self.start.max(self.end)
    }
    /// Inclusive membership, independent of orientation.
    pub fn contains(self, value: f64) -> bool {
        value.is_finite() && value >= self.minimum() && value <= self.maximum()
    }
    pub(crate) fn distinct(self) -> ChartResult<Self> {
        if self.start == self.end {
            Err(error(
                DiagnosticCode::NumericalDomain,
                "This scale range or viewport needs distinct endpoints.",
            ))
        } else {
            Ok(self)
        }
    }
}

/// Declared handling outside the visible scale domain; statistics are never filtered here.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum OutsidePolicy {
    /// Preserve finite extrapolated geometry for clipping at the plot bounds (default).
    #[default]
    Extend,
    /// Clamp to the nearest visible-domain edge.
    Clamp,
    /// Omit outside vertices/marks; a line is split when an omitted vertex is encountered.
    Omit,
}

/// Explicit scale capabilities; category lookup is distinct from a numeric inverse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScaleCapabilities {
    /// A continuous numeric inverse is available.
    pub numeric_inverse: bool,
    /// A categorical lookup/extent is available.
    pub category_lookup: bool,
}

/// Supported scale names for declarative callers. Unsupported families fail explicitly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaleKind {
    /// Continuous affine mapping.
    Linear,
    /// Positive logarithmic numeric coordinates.
    Log,
    /// Signed logarithmic numeric coordinates with a linear threshold.
    Symlog,
    /// Categorical centers with no width.
    Point,
    /// Supplied active-session timestamps.
    Session,
    /// Categorical centers/extents.
    Band,
    /// Integer Unix timestamps with explicit units and UTC calendar ticks.
    Utc,
}
impl ScaleKind {
    /// Resolve an implemented positional family by its portable name.
    pub fn from_name(name: &str) -> ChartResult<Self> {
        match name {
            "linear" => Ok(Self::Linear),
            "log" => Ok(Self::Log),
            "symlog" => Ok(Self::Symlog),
            "point" => Ok(Self::Point),
            "session" => Ok(Self::Session),
            "band" => Ok(Self::Band),
            "utc" => Ok(Self::Utc),
            _ => Err(error(
                DiagnosticCode::UnsupportedCapability,
                format!("Unknown positional scale family {name:?}."),
            )),
        }
    }
}

pub(crate) fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Correct the domain, range, precision, tick or layout request before resolving the scene again.",
    )
}

pub(crate) fn category_window(
    labels: &[String],
    first: &str,
    last: &str,
) -> ChartResult<std::ops::Range<usize>> {
    let a = labels.iter().position(|s| s == first);
    let b = labels.iter().position(|s| s == last);
    match (a, b) {
        (Some(a), Some(b)) if a <= b => Ok(a..b + 1),
        _ => Err(error(
            DiagnosticCode::Validation,
            "Category window endpoints must exist in ascending domain order.",
        )),
    }
}

mod ggplot_unbounded;
pub use ggplot_unbounded::GgplotUnboundedScale;
pub(crate) use ggplot_unbounded::coordinate_position as ggplot_coordinate_position;
mod ggplot_minor_breaks;
pub use ggplot_minor_breaks::ggplot_minor_breaks;

pub(crate) mod ggplot_numeric_limits;

pub(crate) use ggplot_time_pretty::{pretty_date, pretty_time};

mod ggplot_palette;
pub(crate) use ggplot_palette::PaletteBatch;
