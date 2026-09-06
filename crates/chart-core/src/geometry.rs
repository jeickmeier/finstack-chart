//! Finite destination-space geometry; no unchecked arithmetic operators are exposed.

use crate::{ChartResult, Diagnostic, DiagnosticCode};

/// A finite point in the scene's declared destination units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    /// Reject NaN and either infinity without silently clamping coordinates.
    pub fn new(x: f64, y: f64) -> ChartResult<Self> {
        if !x.is_finite() || !y.is_finite() {
            return Err(numeric_error("Point coordinates must be finite."));
        }
        Ok(Self { x, y })
    }

    /// Horizontal coordinate.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Vertical coordinate.
    pub const fn y(self) -> f64 {
        self.y
    }
}

/// An axis-aligned rectangle with nonnegative extents and finite far edges.
/// Zero width/height represents empty space, not a substituted nonempty rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    origin: Point,
    width: f64,
    height: f64,
}

impl Rect {
    /// Validate the origin, extents and additions used to obtain the far edges.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> ChartResult<Self> {
        let origin = Point::new(x, y)?;
        if !width.is_finite()
            || !height.is_finite()
            || width < 0.0
            || height < 0.0
            || !(x + width).is_finite()
            || !(y + height).is_finite()
        {
            return Err(numeric_error(
                "Rectangle extents and far edges must be finite and extents nonnegative.",
            ));
        }
        Ok(Self {
            origin,
            width,
            height,
        })
    }

    /// Upper-left/minimum coordinate in the destination's axis convention.
    pub const fn origin(self) -> Point {
        self.origin
    }

    /// Nonnegative horizontal extent.
    pub const fn width(self) -> f64 {
        self.width
    }

    /// Nonnegative vertical extent.
    pub const fn height(self) -> f64 {
        self.height
    }

    /// Finite far horizontal edge.
    pub fn max_x(self) -> f64 {
        self.origin.x + self.width
    }

    /// Finite far vertical edge.
    pub fn max_y(self) -> f64 {
        self.origin.y + self.height
    }
}

pub(crate) fn numeric_error(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::NumericalDomain,
        message,
        "Supply finite destination values with valid nonnegative extents and positive sizes.",
    )
}

pub(crate) fn positive(value: f64, message: &str) -> ChartResult<()> {
    if !value.is_finite() || value <= 0.0 {
        return Err(numeric_error(message));
    }
    Ok(())
}
