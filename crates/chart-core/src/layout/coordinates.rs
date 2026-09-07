//! Explicit Cartesian coordinate capabilities for extensions and host tools.
use super::{ResolvedAxis, ResolvedScale};
use crate::composition::ScaleValue;
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect};
/// Coordinate behavior that callers can inspect before enabling an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoordinateCapabilities {
    /// Both axes support a semantic inverse (numeric or exact timestamp).
    pub inverse: bool,
    /// Rectangular clipping is available and shared with the scene.
    pub rectangular_clip: bool,
    /// Automatic subdivision of arbitrary data-space curves is implemented.
    pub path_subdivision: bool,
}
/// Borrow the actual resolved named axes; custom tools never invent a second transform.
pub struct Cartesian<'a> {
    x: &'a ResolvedAxis,
    y: &'a ResolvedAxis,
    clip: Rect,
}
impl<'a> Cartesian<'a> {
    /// Bind one horizontal and one vertical coordinate scale plus the existing plot clip.
    pub fn new(x: &'a ResolvedAxis, y: &'a ResolvedAxis, clip: Rect) -> ChartResult<Self> {
        if !x.spec.side.horizontal()
            || y.spec.side.horizontal()
            || matches!(x.scale, ResolvedScale::Secondary { .. })
            || matches!(y.scale, ResolvedScale::Secondary { .. })
        {
            return Err(unsupported(
                "Cartesian projection requires horizontal/vertical primary coordinate axes.",
            ));
        }
        Ok(Self { x, y, clip })
    }
    /// Report supported operations; category lookup is never disguised as numeric inversion.
    pub fn capabilities(&self) -> CoordinateCapabilities {
        CoordinateCapabilities {
            inverse: self.x.capabilities().numeric_inverse && self.y.capabilities().numeric_inverse,
            rectangular_clip: true,
            path_subdivision: false,
        }
    }
    /// Project declared values with the same omission/clamp/extrapolation policies as marks.
    pub fn project(&self, x: &ScaleValue, y: &ScaleValue) -> ChartResult<Option<Point>> {
        match (self.x.map_value(x)?, self.y.map_value(y)?) {
            (Some(x), Some(y)) => Ok(Some(Point::new(x, y)?)),
            _ => Ok(None),
        }
    }
    /// Recover numeric calculation values or exact source-unit timestamps; categories reject.
    pub fn inverse(&self, p: Point) -> ChartResult<(ScaleValue, ScaleValue)> {
        Ok((self.x.invert_value(p.x())?, self.y.invert_value(p.y())?))
    }
    /// Exact rectangle shared with painter/hit/export clipping; projection alone does not crop data.
    pub fn clip(&self) -> Rect {
        self.clip
    }
}
impl ResolvedAxis {
    /// Actual capabilities of the resolved scale, including guide-only secondary axes.
    pub fn capabilities(&self) -> crate::scales::ScaleCapabilities {
        use crate::scales::ScaleCapabilities;
        match &self.scale {
            ResolvedScale::Linear(_)
            | ResolvedScale::Nonlinear(_)
            | ResolvedScale::Utc(_)
            | ResolvedScale::Session(_) => ScaleCapabilities {
                numeric_inverse: true,
                category_lookup: false,
            },
            ResolvedScale::Band(_) | ResolvedScale::Point(_) => ScaleCapabilities {
                numeric_inverse: false,
                category_lookup: true,
            },
            ResolvedScale::Secondary { .. } => ScaleCapabilities {
                numeric_inverse: false,
                category_lookup: false,
            },
        }
    }
    /// Invert a destination position with the resolved numeric/time policy.
    pub fn invert_value(&self, p: f64) -> ChartResult<ScaleValue> {
        match &self.scale {
            ResolvedScale::Linear(s) => s.invert(p).map(ScaleValue::Number),
            ResolvedScale::Nonlinear(s) => s.invert(p).map(ScaleValue::Number),
            ResolvedScale::Utc(s) => Ok(ScaleValue::Timestamp {
                value: s.invert(p)?,
                unit: s.unit(),
            }),
            ResolvedScale::Session(s) => Ok(ScaleValue::Timestamp {
                value: s.invert(p)?,
                unit: s.calendar().unit,
            }),
            ResolvedScale::Band(_) | ResolvedScale::Point(_) => Err(unsupported(
                "Category scales provide category lookup, not a numeric inverse.",
            )),
            ResolvedScale::Secondary { .. } => Err(unsupported(
                "A secondary guide cannot be used as an independent coordinate inverse.",
            )),
        }
    }
}
fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::UnsupportedCapability,
        message,
        "Check resolved scale/coordinate capabilities and use an appropriate primary axis or category lookup.",
    )
}
