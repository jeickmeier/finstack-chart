//! Pure compiled interpolation, with explicit sampling and owned results.
//!
//! The supported d3-interpolate 3.0.1 profile is defined in ADR-017. Algorithms
//! adapted from D3 retain its ISC notice in `LICENSE`. Scales own normalization;
//! this module only samples ranges and never reads a clock or performs I/O.
mod authoring;
mod color;
mod descriptor;
mod scalar;
mod structured;
mod text;
mod transform;
mod transform_parse;
mod value;
mod zoom;

use crate::{ChartResult, Diagnostic, DiagnosticCode};
pub use authoring::InterpolationOptions;
pub use color::{ColorInterpolator, ColorRoute, HueInterpolator};
pub use descriptor::{
    FactoryKind, InterpolationFactory, InterpolationRegistration, InterpolationSpec, Interpolator,
};
pub(crate) use scalar::js_round;
pub use scalar::{Discrete, Piecewise, Sample, ScalarInterpolator, quantize};
pub use structured::{ValueInterpolator, ValueOperation};
pub use transform::{
    Decomposed, TransformInterpolator, TransformSyntax, decompose, parse_transform,
};
pub use value::{MAX_VALUE_BYTES, MAX_VALUE_DEPTH, Number, NumericArray, NumericKind, Value};
pub use zoom::{ZoomInterpolator, ZoomView};

/// Maximum control points, samples or aggregate value nodes per operation.
pub const MAX_VALUES: usize = 200_000;
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use the documented interpolation kinds, finite parameters and resource limits.",
    )
}
fn parameter(t: f64) -> ChartResult<()> {
    if t.is_finite() {
        Ok(())
    } else {
        Err(error(
            DiagnosticCode::NumericalDomain,
            "Interpolation parameter must be finite.",
        ))
    }
}
fn count(actual: usize, minimum: usize) -> ChartResult<()> {
    if actual < minimum {
        Err(error(
            DiagnosticCode::Validation,
            format!("Interpolation needs at least {minimum} values."),
        ))
    } else if actual > MAX_VALUES {
        Err(error(
            DiagnosticCode::ResourceLimit,
            "Interpolation value count exceeds its budget.",
        ))
    } else {
        Ok(())
    }
}
