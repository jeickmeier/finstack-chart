use super::{
    Coordinate, CurveFactory, CurveProtocol, CurveSpec, Line, ShapeLimits, domain, point_radial,
};
use crate::{ChartResult, path::Path};

/// One materialized edge. Numeric endpoint rows carry no invented hierarchy identity.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkDatum {
    /// Source endpoint numeric fields.
    pub source: Vec<f64>,
    /// Target endpoint numeric fields.
    pub target: Vec<f64>,
}
/// Select either edge endpoint or a fixed numeric endpoint row.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LinkEndpoint {
    /// Read the datum's source row.
    Source,
    /// Read the datum's target row.
    Target,
    /// Use a constant numeric endpoint row.
    Constant(Vec<f64>),
}
impl LinkEndpoint {
    fn read<'a>(&'a self, datum: &'a LinkDatum) -> &'a [f64] {
        match self {
            Self::Source => &datum.source,
            Self::Target => &datum.target,
            Self::Constant(v) => v,
        }
    }
}
/// A two-endpoint link using the same checked line lifecycle as other generators.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Link {
    source: LinkEndpoint,
    target: LinkEndpoint,
    x: Coordinate,
    y: Coordinate,
    curve: CurveSpec,
    digits: Option<f64>,
    limits: ShapeLimits,
}
impl Default for Link {
    fn default() -> Self {
        Self {
            source: LinkEndpoint::Source,
            target: LinkEndpoint::Target,
            x: Coordinate::Column(0),
            y: Coordinate::Column(1),
            curve: CurveSpec::BumpX,
            digits: Some(3.),
            limits: ShapeLimits::default(),
        }
    }
}
impl Link {
    /// Construct a generic link with any checked built-in curve.
    pub fn new(curve: CurveSpec) -> ChartResult<Self> {
        curve.validate()?;
        Ok(Self {
            curve,
            ..Self::default()
        })
    }
    /// A horizontal-tangent cubic link (bumpX).
    pub fn horizontal() -> Self {
        Self::default()
    }
    /// A vertical-tangent cubic link (bumpY).
    pub fn vertical() -> Self {
        Self {
            curve: CurveSpec::BumpY,
            ..Self::default()
        }
    }
    /// Select the source endpoint row.
    pub fn source(mut self, value: LinkEndpoint) -> Self {
        self.source = value;
        self
    }
    /// Select the target endpoint row.
    pub fn target(mut self, value: LinkEndpoint) -> Self {
        self.target = value;
        self
    }
    /// Select the x coordinate of each selected endpoint.
    pub fn x(mut self, value: Coordinate) -> Self {
        self.x = value;
        self
    }
    /// Select the y coordinate of each selected endpoint.
    pub fn y(mut self, value: Coordinate) -> Self {
        self.y = value;
        self
    }
    /// Set SVG-only precision.
    pub fn digits(mut self, value: Option<f64>) -> ChartResult<Self> {
        Line::new().digits(value)?;
        self.digits = value;
        Ok(self)
    }
    /// Set input and shared path bounds; every edge consumes two source points.
    pub fn limits(mut self, value: ShapeLimits) -> Self {
        self.limits = value;
        self
    }
    /// Validate the reusable descriptor independently of a datum.
    pub fn validate(&self) -> ChartResult<()> {
        self.line()?.validate()?;
        for endpoint in [&self.source, &self.target] {
            if matches!(endpoint, LinkEndpoint::Constant(v) if v.iter().any(|x| !x.is_finite())) {
                return Err(domain("Constant link endpoint fields must be finite."));
            }
        }
        Ok(())
    }
    fn line(&self) -> ChartResult<Line> {
        Line::new()
            .x(self.x)
            .y(self.y)
            .curve(self.curve)?
            .digits(self.digits)
            .map(|line| line.limits(self.limits))
    }
    pub(super) fn points(&self, datum: &LinkDatum) -> ChartResult<[[f64; 2]; 2]> {
        let source = self.source.read(datum);
        let target = self.target.read(datum);
        Ok([
            [self.x.read(source)?, self.y.read(source)?],
            [self.x.read(target)?, self.y.read(target)?],
        ])
    }
    /// Generate one edge from selected materialized source and target rows.
    pub fn generate(&self, datum: &LinkDatum) -> ChartResult<Path> {
        self.generate_by(datum, |d| self.points(d))
    }
    /// Generate from a native endpoint accessor replacing all materialized selectors.
    pub fn generate_by<T>(
        &self,
        datum: &T,
        accessor: impl FnOnce(&T) -> ChartResult<[[f64; 2]; 2]>,
    ) -> ChartResult<Path> {
        self.generate_with(datum, &self.curve, accessor)
    }
    /// Generate two native endpoints through a reusable custom curve factory.
    pub fn generate_with<T>(
        &self,
        datum: &T,
        factory: &(impl CurveFactory + ?Sized),
        accessor: impl FnOnce(&T) -> ChartResult<[[f64; 2]; 2]>,
    ) -> ChartResult<Path> {
        self.validate()?;
        self.limits.validate_len(2)?;
        let points = accessor(datum)?;
        self.line()?
            .generate_with(&points, factory, |p, _, _| Ok(Some(*p)))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Link", default, deny_unknown_fields)]
struct RadialLinkDef {
    source: LinkEndpoint,
    target: LinkEndpoint,
    #[serde(rename = "angle")]
    x: Coordinate,
    #[serde(rename = "radius")]
    y: Coordinate,
    #[serde(skip)]
    curve: CurveSpec,
    digits: Option<f64>,
    limits: ShapeLimits,
}
/// Radial-tangent cubic link with a shared midpoint radius and clockwise angles.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct LinkRadial(#[serde(with = "RadialLinkDef")] Link);
impl LinkRadial {
    /// Default source/target rows, angle column zero and radius column one.
    pub fn new() -> Self {
        Self::default()
    }
    /// Select the source endpoint row.
    pub fn source(mut self, value: LinkEndpoint) -> Self {
        self.0 = self.0.source(value);
        self
    }
    /// Select the target endpoint row.
    pub fn target(mut self, value: LinkEndpoint) -> Self {
        self.0 = self.0.target(value);
        self
    }
    /// Select each endpoint angle in radians.
    pub fn angle(mut self, value: Coordinate) -> Self {
        self.0 = self.0.x(value);
        self
    }
    /// Select each signed endpoint radius.
    pub fn radius(mut self, value: Coordinate) -> Self {
        self.0 = self.0.y(value);
        self
    }
    /// Set SVG-only precision.
    pub fn digits(mut self, value: Option<f64>) -> ChartResult<Self> {
        self.0 = self.0.digits(value)?;
        Ok(self)
    }
    /// Set input and shared path bounds.
    pub fn limits(mut self, value: ShapeLimits) -> Self {
        self.0 = self.0.limits(value);
        self
    }
    /// Validate a reusable descriptor independently of a datum.
    pub fn validate(&self) -> ChartResult<()> {
        self.0.validate()
    }
    /// Generate one materialized radial edge.
    pub fn generate(&self, datum: &LinkDatum) -> ChartResult<Path> {
        self.generate_by(datum, |d| self.0.points(d))
    }
    /// Generate using native polar source/target endpoints replacing selectors.
    pub fn generate_by<T>(
        &self,
        datum: &T,
        accessor: impl FnOnce(&T) -> ChartResult<[[f64; 2]; 2]>,
    ) -> ChartResult<Path> {
        self.0.generate_with(datum, &RadialBumpFactory, accessor)
    }
}
struct RadialBumpFactory;
struct RadialBump<'a> {
    path: &'a mut Path,
    previous: Option<[f64; 2]>,
}
impl CurveFactory for RadialBumpFactory {
    fn supports_area(&self) -> bool {
        false
    }
    fn create<'a>(
        &'a self,
        path: &'a mut Path,
        _: usize,
    ) -> ChartResult<Box<dyn CurveProtocol + 'a>> {
        Ok(Box::new(RadialBump {
            path,
            previous: None,
        }))
    }
}
impl CurveProtocol for RadialBump<'_> {
    fn line_start(&mut self) -> ChartResult<()> {
        self.previous = None;
        Ok(())
    }
    fn line_end(&mut self) -> ChartResult<()> {
        Ok(())
    }
    fn point(&mut self, angle: f64, radius: f64) -> ChartResult<()> {
        if let Some([a, r]) = self.previous {
            let midpoint = (r + radius) / 2.;
            let p0 = point_radial(a, r)?;
            let p1 = point_radial(a, midpoint)?;
            let p2 = point_radial(angle, midpoint)?;
            let p3 = point_radial(angle, radius)?;
            self.path.move_to(p0[0], p0[1])?;
            self.path
                .bezier_curve_to(p1[0], p1[1], p2[0], p2[1], p3[0], p3[1])?;
        }
        self.previous = Some([angle, radius]);
        Ok(())
    }
}

impl Default for RadialLinkDef {
    fn default() -> Self {
        let base = Link::default();
        Self {
            source: base.source,
            target: base.target,
            x: base.x,
            y: base.y,
            curve: base.curve,
            digits: base.digits,
            limits: base.limits,
        }
    }
}
