//! Custom geometry lowers to checked numeric paint and explicit inspection contracts.
use super::*;
use crate::{ChartResult, DiagnosticCode, Point, Rect};
use std::collections::BTreeSet;

/// Explicit registered geometry selection; base `Geom` declares required encoded channels.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GeometryExtension {
    /// Exact registered name/version.
    pub operation: OperationRef,
    /// Bounded declarative parameters, never executable code.
    pub parameters: serde_json::Value,
}
/// Atomic target selection behavior declared by custom geometry.
#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionPolicy {
    /// This decoration is inspectable but not selectable.
    Disabled,
    /// Select its exact source/aggregate/derived target as one semantic unit.
    AtomicTarget,
}
/// Exact values supplied by a custom geometry for inspection/host accessibility.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum SemanticValue {
    /// A finite computed number in declared units.
    Number(f64),
    /// Logical text, with no markup interpretation.
    Text(String),
    /// Exact signed source/computed integer.
    Signed(#[serde(with = "crate::portable::signed")] i64),
    /// Exact unsigned source/computed integer.
    Unsigned(#[serde(with = "crate::portable::unsigned")] u64),
}
/// Explicit hit shape. Prepared coordinates use calculation units; presented shapes use
/// destination units. Point tolerance/radius is always in destination units.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum HitGeometry {
    /// Circular hit region.
    Point {
        /// Center.
        center: Point,
        /// Positive destination radius.
        radius: f64,
    },
    /// Axis-aligned interval, supporting descending input endpoints.
    Rectangle {
        /// First corner.
        from: Point,
        /// Opposite corner.
        to: Point,
    },
    /// Closed polygon using the even-odd interior test, with at least three vertices.
    Polygon(Vec<Point>),
}
impl HitGeometry {
    /// Bounded finite shape validation before allocation/projection/inspection.
    pub fn validate(&self, remaining: usize) -> ChartResult<usize> {
        let n = match self {
            Self::Point { radius, .. } if radius.is_finite() && *radius > 0. => 1,
            Self::Point { .. } => {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Hit radius must be finite and positive.",
                ));
            }
            Self::Rectangle { .. } => 2,
            Self::Polygon(p) => {
                if p.len() < 3 {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Hit polygons need at least three vertices.",
                    ));
                }
                p.len()
            }
        };
        if n > remaining {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Custom hit geometry exceeds the vertex budget.",
            ));
        }
        Ok(n)
    }
    /// Anchor for a tooltip/focus overlay, derived from the explicitly supplied hit geometry.
    pub fn anchor(&self) -> ChartResult<Point> {
        match self {
            Self::Point { center, .. } => Ok(*center),
            Self::Rectangle { from, to } => {
                Point::new(from.x().midpoint(to.x()), from.y().midpoint(to.y()))
            }
            Self::Polygon(p) => {
                let b = self.bounds()?;
                let _ = p;
                Point::new(
                    b.origin().x() + b.width() / 2.,
                    b.origin().y() + b.height() / 2.,
                )
            }
        }
    }
    /// Bounding rectangle in the shape's coordinate system.
    pub fn bounds(&self) -> ChartResult<Rect> {
        let points: Vec<_> = match self {
            Self::Point { center, radius } => {
                return Rect::new(
                    center.x() - radius,
                    center.y() - radius,
                    2. * radius,
                    2. * radius,
                );
            }
            Self::Rectangle { from, to } => vec![*from, *to],
            Self::Polygon(p) => p.clone(),
        };
        let Some(first) = points.first() else {
            return Err(error(DiagnosticCode::Validation, "Empty hit polygon."));
        };
        let (mut x0, mut x1, mut y0, mut y1) = (first.x(), first.x(), first.y(), first.y());
        for p in points {
            x0 = x0.min(p.x());
            x1 = x1.max(p.x());
            y0 = y0.min(p.y());
            y1 = y1.max(p.y());
        }
        Rect::new(x0, y0, x1 - x0, y1 - y0)
    }
    /// Exact closed region hit, including polygon/rectangle boundaries.
    pub fn contains(&self, p: Point) -> bool {
        match self {
            Self::Point { center, radius } => {
                (p.x() - center.x()).hypot(p.y() - center.y()) <= *radius
            }
            Self::Rectangle { from, to } => {
                p.x() >= from.x().min(to.x())
                    && p.x() <= from.x().max(to.x())
                    && p.y() >= from.y().min(to.y())
                    && p.y() <= from.y().max(to.y())
            }
            Self::Polygon(points) => {
                let mut inside = false;
                for (a, b) in points
                    .iter()
                    .zip(points.iter().cycle().skip(1))
                    .take(points.len())
                {
                    let (dx, dy) = (b.x() - a.x(), b.y() - a.y());
                    let cross = (p.x() - a.x()) * dy - (p.y() - a.y()) * dx;
                    if cross == 0.
                        && p.x() >= a.x().min(b.x())
                        && p.x() <= a.x().max(b.x())
                        && p.y() >= a.y().min(b.y())
                        && p.y() <= a.y().max(b.y())
                    {
                        return true;
                    }
                    if (a.y() > p.y()) != (b.y() > p.y())
                        && p.x() < a.x() + (p.y() - a.y()) / dy * dx
                    {
                        inside = !inside;
                    }
                }
                inside
            }
        }
    }
}
/// Complete non-paint interaction contract carried from preparation to the presented scene.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct GeometryInteraction {
    /// Explicit geometric region, independent of decoration around it.
    pub hit: HitGeometry,
    /// Bounded ordered labels and exact semantic values.
    pub values: Vec<(String, SemanticValue)>,
    /// Atomic target selection policy.
    pub selection: SelectionPolicy,
    /// Stable traversal key within this layer/panel, separate from paint order.
    #[serde(with = "crate::portable::unsigned")]
    pub keyboard_order: u64,
}
impl GeometryInteraction {
    pub(crate) fn validate(&self, remaining: usize) -> ChartResult<usize> {
        let n = self.hit.validate(remaining)?;
        if self.values.len() > 32 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Custom semantic values exceed 32 fields.",
            ));
        }
        let mut names = BTreeSet::new();
        let mut bytes = 0usize;
        for (name, value) in &self.values {
            if name.is_empty() || !names.insert(name) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Custom semantic labels must be nonempty and unique.",
                ));
            }
            bytes = bytes.saturating_add(name.len());
            match value {
                SemanticValue::Number(v) if !v.is_finite() => {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Custom semantic numbers must be finite.",
                    ));
                }
                SemanticValue::Text(s) => bytes = bytes.saturating_add(s.len()),
                _ => {}
            }
        }
        if bytes > 8192 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Custom semantic text exceeds 8 KiB.",
            ));
        }
        Ok(n)
    }
}
/// Custom numeric paint tied to one existing prepared semantic target.
#[derive(Clone, Debug)]
pub struct CustomMark {
    /// Index into `CustomGeomInput::marks`; provenance/group is copied and cannot be invented.
    pub input_mark: usize,
    /// Data/calculation-space geometry, normalized before the common layout/renderers.
    pub geometry: PreparedGeometry,
    /// Explicit hit, semantic, selection and keyboard contracts.
    pub interaction: GeometryInteraction,
}
/// A custom geom receives checked post-stat/post-position marks and their typed source table.
pub struct CustomGeomInput<'a> {
    /// Checked base geometry selected by the layer's required channel contract.
    pub marks: &'a [PreparedMark],
    /// Immutable generated/source output, for semantic values; no source accessor sees a stat row.
    pub table: &'a PreparedTable,
    /// Validated numeric calculation spaces shared with named axes.
    pub domains: &'a DomainContributions,
    /// Explicit bounded parameters.
    pub parameters: &'a serde_json::Value,
    /// Remaining output vertex budget, including paint and hit geometry.
    pub max_vertices: usize,
}
/// Geometry extensions produce common numeric primitives; no renderer fork is needed.
pub trait CustomGeom: Send + Sync {
    /// Exact native/portable capability and version; batch-only preparation.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate the base channel/position contract and parameters before evaluating sources.
    fn validate(&self, layer: &Layer, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Supply paint and the complete interaction contract for each atomic output.
    fn prepare(&self, input: CustomGeomInput<'_>) -> ChartResult<Vec<CustomMark>>;
}
/// Separate upward/downward candle paint; equal open/close follows `up`.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandleColors {
    /// Close greater than or equal to open.
    pub up: crate::scene::Color,
    /// Close less than open.
    pub down: crate::scene::Color,
}
impl Layer {
    /// Extend checked base geometry with a registered implementation and declarative parameters.
    pub fn with_geometry_extension(mut self, extension: GeometryExtension) -> Self {
        self.geometry_extension = Some(extension);
        self
    }
    /// Explicit candle direction colors; mapped colors retain priority.
    pub fn with_candle_colors(mut self, colors: CandleColors) -> Self {
        self.candle_colors = Some(colors);
        self
    }
}

/// Validate an explicit native-paint descriptor before a renderer clones or dispatches it.
pub fn validate_native_paint(
    painter: &OperationRef,
    parameters: &serde_json::Value,
) -> ChartResult<()> {
    super::extensions::validate_name(&painter.id)?;
    if painter.version == crate::Revision::INITIAL {
        return Err(error(
            DiagnosticCode::Validation,
            "Native painter version must be positive.",
        ));
    }
    super::extensions::parameter_size(parameters).map(|_| ())
}

/// Validated parameter size charged to the shared scene payload budget.
pub(crate) fn native_parameter_size(parameters: &serde_json::Value) -> ChartResult<usize> {
    super::extensions::parameter_size(parameters)
}
