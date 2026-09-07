//! Checked foundational linear, categorical-band and UTC scales.
//! Domain training is separate from visible viewport and destination range projection.

mod band;
mod linear;
mod utc;

pub use band::*;
pub use linear::*;
pub use utc::*;

use crate::{ChartResult, Diagnostic, DiagnosticCode};

/// Finite ordered or descending endpoints; equality is allowed before domain resolution.
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
    /// Categorical centers/extents.
    Band,
    /// Integer Unix timestamps with explicit units and UTC calendar ticks.
    Utc,
}
impl ScaleKind {
    /// Resolve the implemented family; `log`, `symlog` and unknown names remain WP-11.
    pub fn from_name(name: &str) -> ChartResult<Self> {
        match name {
            "linear" => Ok(Self::Linear),
            "band" => Ok(Self::Band),
            "utc" => Ok(Self::Utc),
            _ => Err(error(
                DiagnosticCode::UnsupportedCapability,
                format!(
                    "Scale family {name:?} is not implemented; use linear/band/UTC or wait for the full WP-11 family."
                ),
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
