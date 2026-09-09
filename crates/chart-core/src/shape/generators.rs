use super::{CurveFactory, CurveProtocol, CurveSpec, domain, invalid, limit};
use crate::{
    ChartResult,
    path::{Path, PathLimits, Precision},
};

/// Explicit input-entry and shared path resource limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShapeLimits {
    /// Maximum source entries, including undefined entries.
    pub max_points: usize,
    /// Shared authoring, geometry, SVG, replay and destination bounds.
    pub path: PathLimits,
}
impl Default for ShapeLimits {
    fn default() -> Self {
        Self {
            max_points: 1_000_000,
            path: PathLimits::default(),
        }
    }
}
impl ShapeLimits {
    pub(super) fn validate_len(self, n: usize) -> ChartResult<()> {
        if n > self.max_points {
            Err(limit("Shape source-entry limit exceeded."))
        } else {
            Ok(())
        }
    }
}

/// A finite constant or zero-based column in a materialized numeric row.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Coordinate {
    /// Read a numeric column from each defined row.
    Column(usize),
    /// Use a fixed coordinate for every defined row.
    Constant(f64),
}
impl Coordinate {
    pub(super) fn validate(self) -> ChartResult<()> {
        if matches!(self,Self::Constant(v) if !v.is_finite()) {
            Err(domain("Shape coordinate constants must be finite."))
        } else {
            Ok(())
        }
    }
    pub(super) fn read(self, row: &[f64]) -> ChartResult<f64> {
        match self {
            Self::Constant(v) => Ok(v),
            Self::Column(i) => row.get(i).copied().ok_or_else(|| {
                invalid("A defined shape row does not contain its selected coordinate column.")
            }),
        }
    }
}

/// A constant defined predicate or an exact-length materialized validity mask.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Defined {
    /// Include all or none of the rows.
    Constant(bool),
    /// Include exactly the positions marked true, preserving gaps and input order.
    Mask(Vec<bool>),
}
impl Default for Defined {
    fn default() -> Self {
        Self::Constant(true)
    }
}
impl From<bool> for Defined {
    fn from(value: bool) -> Self {
        Self::Constant(value)
    }
}
impl From<Vec<bool>> for Defined {
    fn from(value: Vec<bool>) -> Self {
        Self::Mask(value)
    }
}
impl Defined {
    fn validate_len(&self, n: usize) -> ChartResult<()> {
        if matches!(self,Self::Mask(v) if v.len()!=n) {
            Err(invalid(
                "A shape defined mask must match the source-entry count.",
            ))
        } else {
            Ok(())
        }
    }
    fn at(&self, i: usize) -> bool {
        match self {
            Self::Constant(v) => *v,
            Self::Mask(v) => v[i],
        }
    }
}
fn precision(digits: Option<f64>) -> ChartResult<Precision> {
    digits
        .map(Precision::from_digits)
        .transpose()
        .map(|v| v.unwrap_or_default())
}
fn finite(point: [f64; 2]) -> ChartResult<()> {
    if point.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(domain("Defined shape coordinates must be finite."))
    }
}

/// Reusable authored-order line generator; legacy chart line policies are separate.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Line {
    pub(super) x: Coordinate,
    pub(super) y: Coordinate,
    pub(super) defined: Defined,
    pub(super) curve: CurveSpec,
    pub(super) digits: Option<f64>,
    pub(super) limits: ShapeLimits,
}
impl Default for Line {
    fn default() -> Self {
        Self {
            x: Coordinate::Column(0),
            y: Coordinate::Column(1),
            defined: Defined::default(),
            curve: CurveSpec::default(),
            digits: Some(3.),
            limits: ShapeLimits::default(),
        }
    }
}
impl Line {
    /// Default x/y columns zero/one, all defined, linear curve and three SVG digits.
    pub fn new() -> Self {
        Self::default()
    }
    /// Select the x coordinate for materialized rows.
    pub fn x(mut self, value: Coordinate) -> Self {
        self.x = value;
        self
    }
    /// Select the y coordinate for materialized rows.
    pub fn y(mut self, value: Coordinate) -> Self {
        self.y = value;
        self
    }
    /// Select the materialized validity policy; false rows split runs.
    pub fn defined(mut self, value: impl Into<Defined>) -> Self {
        self.defined = value.into();
        self
    }
    /// Select a checked built-in curve and finite parameters.
    pub fn curve(mut self, value: CurveSpec) -> ChartResult<Self> {
        value.validate()?;
        self.curve = value;
        Ok(self)
    }
    /// Set SVG digits, or None for unrounded text; numeric geometry is unchanged.
    pub fn digits(mut self, value: Option<f64>) -> ChartResult<Self> {
        precision(value)?;
        self.digits = value;
        Ok(self)
    }
    /// Set explicit input/path work bounds.
    pub fn limits(mut self, value: ShapeLimits) -> Self {
        self.limits = value;
        self
    }
    /// Validate a deserialized reusable configuration independently of source rows.
    pub fn validate(&self) -> ChartResult<()> {
        self.x.validate()?;
        self.y.validate()?;
        self.curve.validate()?;
        precision(self.digits)?;
        Ok(())
    }
    /// Generate from materialized rows, using the selected columns/constants and mask.
    pub fn generate<R: AsRef<[f64]>>(&self, data: &[R]) -> ChartResult<Path> {
        self.generate_materialized_with(data, &self.curve)
    }
    pub(super) fn generate_materialized_with<R: AsRef<[f64]>>(
        &self,
        data: &[R],
        factory: &(impl CurveFactory + ?Sized),
    ) -> ChartResult<Path> {
        self.defined.validate_len(data.len())?;
        self.generate_with(data, factory, |datum, i, _| {
            if self.defined.at(i) {
                Ok(Some([
                    self.x.read(datum.as_ref())?,
                    self.y.read(datum.as_ref())?,
                ]))
            } else {
                Ok(None)
            }
        })
    }
    /// Generate using a fallible native accessor receiving datum, index and input.
    /// The optional point replaces the materialized coordinate selectors/defined mask.
    pub fn generate_by<T>(
        &self,
        data: &[T],
        accessor: impl FnMut(&T, usize, &[T]) -> ChartResult<Option<[f64; 2]>>,
    ) -> ChartResult<Path> {
        self.generate_with(data, &self.curve, accessor)
    }
    /// Generate using a reusable native curve factory and fallible point accessor.
    pub fn generate_with<T>(
        &self,
        data: &[T],
        factory: &(impl CurveFactory + ?Sized),
        mut accessor: impl FnMut(&T, usize, &[T]) -> ChartResult<Option<[f64; 2]>>,
    ) -> ChartResult<Path> {
        self.validate()?;
        self.limits.validate_len(data.len())?;
        let mut path_limits = self.limits.path;
        if let Some(bound) = factory.command_bound(data.len()) {
            path_limits.max_commands = path_limits.max_commands.min(bound);
        }
        let mut path = Path::with_options(precision(self.digits)?, path_limits)?;
        {
            let mut curve = factory.create(&mut path, data.len())?;
            let mut active = false;
            for (i, datum) in data.iter().enumerate() {
                if let Some(p) = accessor(datum, i, data)? {
                    finite(p)?;
                    if !active {
                        curve.line_start()?;
                        active = true;
                    }
                    curve.point(p[0], p[1])?;
                } else if active {
                    curve.line_end()?;
                    active = false;
                }
            }
            if active {
                curve.line_end()?;
            }
        }
        Ok(path.restore_command_limit(self.limits.path.max_commands))
    }
}

/// Independent lower and upper coordinates for a general paired-boundary area.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AreaPoint {
    /// Lower boundary; traversal reverses after the upper run.
    pub lower: [f64; 2],
    /// Upper boundary; traversal follows authored order.
    pub upper: [f64; 2],
}
/// Reference boundary-line helper selection, before optional coordinates resolve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AreaBoundary {
    /// Lower x and lower y.
    X0,
    /// Explicit upper x (or zero if absent) and lower y.
    X1,
    /// Lower x and lower y; alias of X0's coordinate semantics.
    Y0,
    /// Lower x and explicit upper y (or zero if absent).
    Y1,
}

/// General area generator with independent x0/x1/y0/y1 controls and defined gaps.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Area {
    pub(super) x0: Coordinate,
    pub(super) y0: Coordinate,
    pub(super) x1: Option<Coordinate>,
    pub(super) y1: Option<Coordinate>,
    pub(super) defined: Defined,
    pub(super) curve: CurveSpec,
    pub(super) digits: Option<f64>,
    pub(super) limits: ShapeLimits,
}
impl Default for Area {
    fn default() -> Self {
        Self {
            x0: Coordinate::Column(0),
            y0: Coordinate::Constant(0.),
            x1: None,
            y1: Some(Coordinate::Column(1)),
            defined: Defined::default(),
            curve: CurveSpec::default(),
            digits: Some(3.),
            limits: ShapeLimits::default(),
        }
    }
}
impl Area {
    /// Default vertical area with x column zero, baseline zero and y1 column one.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set lower x and clear the optional upper x, making both area boundaries share x.
    pub fn x(mut self, value: Coordinate) -> Self {
        self.x0 = value;
        self.x1 = None;
        self
    }
    /// Set lower y and clear the optional upper y, making both area boundaries share y.
    pub fn y(mut self, value: Coordinate) -> Self {
        self.y0 = value;
        self.y1 = None;
        self
    }
    /// Set lower x without changing upper x.
    pub fn x0(mut self, value: Coordinate) -> Self {
        self.x0 = value;
        self
    }
    /// Set optional upper x; None reuses lower x during area generation.
    pub fn x1(mut self, value: Option<Coordinate>) -> Self {
        self.x1 = value;
        self
    }
    /// Set lower y without changing upper y.
    pub fn y0(mut self, value: Coordinate) -> Self {
        self.y0 = value;
        self
    }
    /// Set optional upper y; None reuses lower y during area generation.
    pub fn y1(mut self, value: Option<Coordinate>) -> Self {
        self.y1 = value;
        self
    }
    /// Set materialized validity; false rows split complete paired boundaries.
    pub fn defined(mut self, value: impl Into<Defined>) -> Self {
        self.defined = value.into();
        self
    }
    /// Select an area-capable checked curve; bundle areas explicitly reject.
    pub fn curve(mut self, value: CurveSpec) -> ChartResult<Self> {
        value.validate()?;
        if !value.supports_area() {
            return Err(invalid("Bundle curves support lines only."));
        }
        self.curve = value;
        Ok(self)
    }
    /// Set SVG-only precision, retaining full numeric boundary/control coordinates.
    pub fn digits(mut self, value: Option<f64>) -> ChartResult<Self> {
        precision(value)?;
        self.digits = value;
        Ok(self)
    }
    /// Set explicit input/path work bounds.
    pub fn limits(mut self, value: ShapeLimits) -> Self {
        self.limits = value;
        self
    }
    /// Validate portable configuration; reference optional-coordinate meaning is retained.
    pub fn validate(&self) -> ChartResult<()> {
        self.x0.validate()?;
        self.y0.validate()?;
        for c in [self.x1, self.y1].into_iter().flatten() {
            c.validate()?;
        }
        self.curve.validate()?;
        if !self.curve.supports_area() {
            return Err(invalid("Bundle curves support lines only."));
        }
        precision(self.digits)?;
        Ok(())
    }
    /// Derive a boundary line with inherited curve/mask and default three SVG digits.
    /// Reference helpers turn an absent x1/y1 into constant zero rather than area fallback.
    pub fn boundary(&self, boundary: AreaBoundary) -> Line {
        let (x, y) = match boundary {
            AreaBoundary::X0 | AreaBoundary::Y0 => (self.x0, self.y0),
            AreaBoundary::X1 => (self.x1.unwrap_or(Coordinate::Constant(0.)), self.y0),
            AreaBoundary::Y1 => (self.x0, self.y1.unwrap_or(Coordinate::Constant(0.))),
        };
        Line {
            x,
            y,
            defined: self.defined.clone(),
            curve: self.curve,
            digits: Some(3.),
            limits: self.limits,
        }
    }
    /// Generate independent paired boundaries from materialized rows.
    pub fn generate<R: AsRef<[f64]>>(&self, data: &[R]) -> ChartResult<Path> {
        self.generate_materialized_with(data, &self.curve)
    }
    pub(super) fn generate_materialized_with<R: AsRef<[f64]>>(
        &self,
        data: &[R],
        factory: &(impl CurveFactory + ?Sized),
    ) -> ChartResult<Path> {
        self.defined.validate_len(data.len())?;
        self.generate_with(data, factory, |datum, i, _| {
            if !self.defined.at(i) {
                return Ok(None);
            }
            let row = datum.as_ref();
            let lower = [self.x0.read(row)?, self.y0.read(row)?];
            let upper = [
                self.x1
                    .map(|c| c.read(row))
                    .transpose()?
                    .unwrap_or(lower[0]),
                self.y1
                    .map(|c| c.read(row))
                    .transpose()?
                    .unwrap_or(lower[1]),
            ];
            Ok(Some(AreaPoint { lower, upper }))
        })
    }
    /// Generate using a native optional paired-point accessor, replacing materialized fields/mask.
    pub fn generate_by<T>(
        &self,
        data: &[T],
        accessor: impl FnMut(&T, usize, &[T]) -> ChartResult<Option<AreaPoint>>,
    ) -> ChartResult<Path> {
        self.generate_with(data, &self.curve, accessor)
    }
    /// Generate with a reusable area-capable native curve factory and accessor.
    pub fn generate_with<T>(
        &self,
        data: &[T],
        factory: &(impl CurveFactory + ?Sized),
        mut accessor: impl FnMut(&T, usize, &[T]) -> ChartResult<Option<AreaPoint>>,
    ) -> ChartResult<Path> {
        self.validate()?;
        self.limits.validate_len(data.len())?;
        if !factory.supports_area() {
            return Err(invalid(
                "The selected native curve factory does not support areas.",
            ));
        }
        let max_points = data
            .len()
            .checked_mul(2)
            .ok_or_else(|| limit("Area boundary point limit overflow."))?;
        let mut path_limits = self.limits.path;
        if let Some(bound) = factory.command_bound(max_points) {
            path_limits.max_commands = path_limits.max_commands.min(bound);
        }
        let mut path = Path::with_options(precision(self.digits)?, path_limits)?;
        {
            let mut curve = factory.create(&mut path, max_points)?;
            let mut lower = Vec::new();
            let mut active = false;
            for (i, datum) in data.iter().enumerate() {
                if let Some(p) = accessor(datum, i, data)? {
                    finite(p.lower)?;
                    finite(p.upper)?;
                    if !active {
                        curve.area_start()?;
                        curve.line_start()?;
                        active = true;
                    }
                    lower.push(p.lower);
                    curve.point(p.upper[0], p.upper[1])?;
                } else if active {
                    end_area(curve.as_mut(), &mut lower)?;
                    active = false;
                }
            }
            if active {
                end_area(curve.as_mut(), &mut lower)?;
            }
        }
        Ok(path.restore_command_limit(self.limits.path.max_commands))
    }
}
fn end_area(curve: &mut dyn CurveProtocol, lower: &mut Vec<[f64; 2]>) -> ChartResult<()> {
    curve.line_end()?;
    curve.line_start()?;
    for p in lower.iter().rev() {
        curve.point(p[0], p[1])?;
    }
    curve.line_end()?;
    curve.area_end()?;
    lower.clear();
    Ok(())
}
