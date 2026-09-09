use super::{
    Area, AreaBoundary, AreaPoint, Coordinate, CurveFactory, CurveProtocol, CurveSpec, Defined,
    Line, ShapeLimits, domain,
};
use crate::{ChartResult, path::Path};

/// Convert a clockwise angle from twelve o'clock and a signed radius to x/y.
/// This preserves pointRadial's angle-minus-pi/2 evaluation order.
pub fn point_radial(angle: f64, radius: f64) -> ChartResult<[f64; 2]> {
    if !angle.is_finite() || !radius.is_finite() {
        return Err(domain("Radial angle and radius must be finite."));
    }
    let a = angle - std::f64::consts::FRAC_PI_2;
    Ok([radius * a.cos(), radius * a.sin()])
}

pub(super) struct RadialFactory<'a, F: ?Sized>(pub(super) &'a F);
struct RadialCurve<'a>(Box<dyn CurveProtocol + 'a>);
impl<F: CurveFactory + ?Sized> CurveFactory for RadialFactory<'_, F> {
    fn command_bound(&self, max_points: usize) -> Option<usize> {
        self.0.command_bound(max_points)
    }
    fn supports_area(&self) -> bool {
        self.0.supports_area()
    }
    fn create<'a>(
        &'a self,
        path: &'a mut Path,
        max_points: usize,
    ) -> ChartResult<Box<dyn CurveProtocol + 'a>> {
        Ok(Box::new(RadialCurve(self.0.create(path, max_points)?)))
    }
}
impl CurveProtocol for RadialCurve<'_> {
    fn area_start(&mut self) -> ChartResult<()> {
        self.0.area_start()
    }
    fn area_end(&mut self) -> ChartResult<()> {
        self.0.area_end()
    }
    fn line_start(&mut self) -> ChartResult<()> {
        self.0.line_start()
    }
    fn line_end(&mut self) -> ChartResult<()> {
        self.0.line_end()
    }
    fn point(&mut self, angle: f64, radius: f64) -> ChartResult<()> {
        // CurveRadial deliberately uses this arithmetic, unlike pointRadial.
        let p = curve_point_radial(angle, radius)?;
        self.0.point(p[0], p[1])
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Line", default, deny_unknown_fields)]
struct RadialLineDef {
    #[serde(rename = "angle")]
    x: Coordinate,
    #[serde(rename = "radius")]
    y: Coordinate,
    defined: Defined,
    curve: CurveSpec,
    digits: Option<f64>,
    limits: ShapeLimits,
}

/// Authored-order radial line over the common Cartesian curve lifecycle.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct LineRadial(#[serde(with = "RadialLineDef")] pub(super) Line);
impl LineRadial {
    /// Angle column zero, radius column one, linear curve and three SVG digits.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the materialized angle selector in radians.
    pub fn angle(mut self, value: Coordinate) -> Self {
        self.0 = self.0.x(value);
        self
    }
    /// Set the signed radius selector in destination units.
    pub fn radius(mut self, value: Coordinate) -> Self {
        self.0 = self.0.y(value);
        self
    }
    /// Set validity; false entries split authored runs.
    pub fn defined(mut self, value: impl Into<Defined>) -> Self {
        self.0 = self.0.defined(value);
        self
    }
    /// Select any built-in curve before radial coordinate conversion.
    pub fn curve(mut self, value: CurveSpec) -> ChartResult<Self> {
        self.0 = self.0.curve(value)?;
        Ok(self)
    }
    /// Select SVG-only precision; None leaves coordinates unrounded.
    pub fn digits(mut self, value: Option<f64>) -> ChartResult<Self> {
        self.0 = self.0.digits(value)?;
        Ok(self)
    }
    /// Set explicit input and path bounds.
    pub fn limits(mut self, value: ShapeLimits) -> Self {
        self.0 = self.0.limits(value);
        self
    }
    /// Validate a reusable descriptor independently of data.
    pub fn validate(&self) -> ChartResult<()> {
        self.0.validate()
    }
    /// Generate using selected angle/radius columns or constants.
    pub fn generate<R: AsRef<[f64]>>(&self, data: &[R]) -> ChartResult<Path> {
        self.0
            .generate_materialized_with(data, &RadialFactory(&self.0.curve))
    }
    /// Generate with a fallible native angle/radius accessor replacing selectors and mask.
    pub fn generate_by<T>(
        &self,
        data: &[T],
        accessor: impl FnMut(&T, usize, &[T]) -> ChartResult<Option<[f64; 2]>>,
    ) -> ChartResult<Path> {
        self.generate_with(data, &self.0.curve, accessor)
    }
    /// Project native polar points into a reusable custom Cartesian curve factory.
    pub fn generate_with<T>(
        &self,
        data: &[T],
        factory: &(impl CurveFactory + ?Sized),
        accessor: impl FnMut(&T, usize, &[T]) -> ChartResult<Option<[f64; 2]>>,
    ) -> ChartResult<Path> {
        self.0
            .generate_with(data, &RadialFactory(factory), accessor)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Area", default, deny_unknown_fields)]
struct RadialAreaDef {
    #[serde(rename = "start_angle")]
    x0: Coordinate,
    #[serde(rename = "inner_radius")]
    y0: Coordinate,
    #[serde(rename = "end_angle")]
    x1: Option<Coordinate>,
    #[serde(rename = "outer_radius")]
    y1: Option<Coordinate>,
    defined: Defined,
    curve: CurveSpec,
    digits: Option<f64>,
    limits: ShapeLimits,
}
/// Named radial boundary helper; absent optional endpoints resolve to zero in helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RadialBoundary {
    /// Start angle and inner radius.
    StartAngle,
    /// End angle (or zero) and inner radius.
    EndAngle,
    /// Start angle and inner radius.
    InnerRadius,
    /// Start angle and outer radius (or zero).
    OuterRadius,
}
/// Paired radial boundaries sharing the checked area lifecycle and all area-capable curves.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub struct AreaRadial(#[serde(with = "RadialAreaDef")] pub(super) Area);
impl<'de> serde::Deserialize<'de> for AreaRadial {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let mut fields =
            std::collections::BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
        for (shared, start, end) in [
            ("angle", "start_angle", "end_angle"),
            ("radius", "inner_radius", "outer_radius"),
        ] {
            if let Some(value) = fields.remove(shared) {
                if fields.contains_key(start) || fields.contains_key(end) {
                    return Err(D::Error::custom(
                        "Shared radial selectors cannot coexist with independent boundary selectors.",
                    ));
                }
                fields.insert(start.into(), value);
                fields.insert(end.into(), serde_json::Value::Null);
            }
        }
        let value = serde_json::to_value(fields).map_err(D::Error::custom)?;
        RadialAreaDef::deserialize(value)
            .map(Self)
            .map_err(D::Error::custom)
    }
}
impl AreaRadial {
    /// Start angle column zero, inner radius zero, shared end angle, outer radius column one.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the shared start/end angle, clearing the optional end selector.
    pub fn angle(mut self, value: Coordinate) -> Self {
        self.0 = self.0.x(value);
        self
    }
    /// Set the shared inner/outer radius, clearing the optional outer selector.
    pub fn radius(mut self, value: Coordinate) -> Self {
        self.0 = self.0.y(value);
        self
    }
    /// Set the start angle without changing the end selector.
    pub fn start_angle(mut self, value: Coordinate) -> Self {
        self.0 = self.0.x0(value);
        self
    }
    /// Set the end angle; None reuses the start angle during area generation.
    pub fn end_angle(mut self, value: Option<Coordinate>) -> Self {
        self.0 = self.0.x1(value);
        self
    }
    /// Set the inner radius without changing the outer selector.
    pub fn inner_radius(mut self, value: Coordinate) -> Self {
        self.0 = self.0.y0(value);
        self
    }
    /// Set the outer radius; None reuses the inner radius during area generation.
    pub fn outer_radius(mut self, value: Option<Coordinate>) -> Self {
        self.0 = self.0.y1(value);
        self
    }
    /// Set validity; false rows split both boundaries.
    pub fn defined(mut self, value: impl Into<Defined>) -> Self {
        self.0 = self.0.defined(value);
        self
    }
    /// Select a checked area-capable curve.
    pub fn curve(mut self, value: CurveSpec) -> ChartResult<Self> {
        self.0 = self.0.curve(value)?;
        Ok(self)
    }
    /// Select SVG-only precision.
    pub fn digits(mut self, value: Option<f64>) -> ChartResult<Self> {
        self.0 = self.0.digits(value)?;
        Ok(self)
    }
    /// Set explicit input and path bounds.
    pub fn limits(mut self, value: ShapeLimits) -> Self {
        self.0 = self.0.limits(value);
        self
    }
    /// Validate a reusable descriptor independently of data.
    pub fn validate(&self) -> ChartResult<()> {
        self.0.validate()
    }
    /// Return a radial boundary line inheriting curve/mask and default three SVG digits.
    pub fn boundary(&self, boundary: RadialBoundary) -> LineRadial {
        LineRadial(self.0.boundary(match boundary {
            RadialBoundary::StartAngle => AreaBoundary::X0,
            RadialBoundary::EndAngle => AreaBoundary::X1,
            RadialBoundary::InnerRadius => AreaBoundary::Y0,
            RadialBoundary::OuterRadius => AreaBoundary::Y1,
        }))
    }
    /// Generate paired polar boundaries from materialized numeric rows.
    pub fn generate<R: AsRef<[f64]>>(&self, data: &[R]) -> ChartResult<Path> {
        self.0
            .generate_materialized_with(data, &RadialFactory(&self.0.curve))
    }
    /// Generate with native polar lower/upper pairs replacing selectors and mask.
    pub fn generate_by<T>(
        &self,
        data: &[T],
        accessor: impl FnMut(&T, usize, &[T]) -> ChartResult<Option<AreaPoint>>,
    ) -> ChartResult<Path> {
        self.generate_with(data, &self.0.curve, accessor)
    }
    /// Project polar boundary pairs into a reusable custom area-capable factory.
    pub fn generate_with<T>(
        &self,
        data: &[T],
        factory: &(impl CurveFactory + ?Sized),
        accessor: impl FnMut(&T, usize, &[T]) -> ChartResult<Option<AreaPoint>>,
    ) -> ChartResult<Path> {
        self.0
            .generate_with(data, &RadialFactory(factory), accessor)
    }
}

impl Default for RadialLineDef {
    fn default() -> Self {
        let base = Line::default();
        Self {
            x: base.x,
            y: base.y,
            defined: base.defined,
            curve: base.curve,
            digits: base.digits,
            limits: base.limits,
        }
    }
}

impl Default for RadialAreaDef {
    fn default() -> Self {
        let base = Area::default();
        Self {
            x0: base.x0,
            y0: base.y0,
            x1: base.x1,
            y1: base.y1,
            defined: base.defined,
            curve: base.curve,
            digits: base.digits,
            limits: base.limits,
        }
    }
}

// Shared chart source anchors use the curve adapter's exact evaluation order.
pub(crate) fn curve_point_radial(angle: f64, radius: f64) -> ChartResult<[f64; 2]> {
    if !angle.is_finite() || !radius.is_finite() {
        return Err(domain("Radial angle and radius must be finite."));
    }
    Ok([radius * angle.sin(), -radius * angle.cos()])
}
