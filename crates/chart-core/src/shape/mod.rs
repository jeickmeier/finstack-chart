//! Checked d3-shape-compatible generators over the shared numeric path engine.
//!
//! Generator precision defaults to three digits and never changes retained geometry.
//! An empty generator result is an empty owned path. Native accessors and custom curve
//! factories use the same point/lifecycle boundary as materialized portable inputs.
mod arc;
mod curve;
mod curves;
mod generators;
mod link;
mod pie;
mod radial;
mod registered;
pub(crate) use radial::curve_point_radial;
mod ggplot_symbol;
mod stack;
mod symbol;
use crate::{Diagnostic, DiagnosticCode};
pub use arc::{Arc, ArcDatum, ArcParameters};
pub use curve::{CurveContext, CurveFactory, CurveProtocol, CurveSpec};
pub use generators::{Area, AreaBoundary, AreaPoint, Coordinate, Defined, Line, ShapeLimits};
pub use link::{Link, LinkDatum, LinkEndpoint, LinkRadial};
pub use pie::{Pie, PieAngles, PieComparator, PieOrder, PieSlice};
pub use radial::{AreaRadial, LineRadial, RadialBoundary, point_radial};
pub use stack::{
    Stack, StackLimits, StackMissing, StackOffset, StackOffsetting, StackOrder, StackOrdering,
    StackPoint, StackSeries,
};
pub use symbol::{
    SYMBOLS, SYMBOLS_FILL, SYMBOLS_STROKE, Symbol, SymbolDraw, SymbolKind, SymbolPaint,
};
fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use a valid finite shape configuration and ordered curve lifecycle.",
    )
}
fn domain(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::NumericalDomain,
        message,
        "Supply finite coordinates and parameters, or mark the observation undefined.",
    )
}
fn limit(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::ResourceLimit,
        message,
        "Reduce shape input/work or explicitly select a sufficient bounded limit.",
    )
}
