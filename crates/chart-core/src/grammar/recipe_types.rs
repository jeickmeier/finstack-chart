//! Shared built-in recipe declarations; semantic calculation stays in the grammar engine.
use super::{recipe_distributions::*, recipe_marks::*, recipe_surfaces::*};
/// Extra recipe channels use the existing source/statistical input reader.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum RecipeAesthetic {
    /// Lower dependent-axis endpoint.
    Lower,
    /// Upper dependent-axis endpoint.
    Upper,
    /// Independent-axis width.
    Width,
    /// Dependent-axis height.
    Height,
    /// Mathematical angle in radians.
    Angle,
    /// Signed data-space radius.
    Radius,
    /// Data-space slope; never an axis training observation.
    Slope,
    /// Data-space intercept.
    Intercept,
    /// Box median in dependent-axis calculation space.
    Middle,
    /// Lower whisker endpoint.
    WhiskerLower,
    /// Upper whisker endpoint.
    WhiskerUpper,
    /// Lower notch endpoint.
    NotchLower,
    /// Upper notch endpoint.
    NotchUpper,
    /// Group width multiplier before normalization.
    RelativeWidth,
    /// Normalized violin half-width multiplier.
    ViolinWidth,
    /// Quantile probability marking a violin row.
    QuantileFlag,
    /// Source or statistic bin width.
    BinWidth,
    /// Supplied dot-stack offset.
    StackPosition,
    /// Number of dots represented by a bin row.
    Count,
    /// Polygon subgroup identity, independent of the outer group.
    Subgroup,
}
/// Endpoint selection for an arrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ArrowEnds {
    /// First endpoint.
    First,
    /// Last endpoint.
    Last,
    /// Both endpoints.
    Both,
}
/// One portable arrow policy shared by segments, curves and reference lines.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrowSpec {
    /// Angle between the shaft and either arrow edge, in degrees.
    pub angle: f64,
    /// Arrow edge length in millimetres.
    pub length_mm: f64,
    /// Selected endpoints.
    pub ends: ArrowEnds,
    /// Filled closed triangle instead of an open tip.
    pub closed: bool,
}
impl ArrowSpec {
    pub(crate) fn validate(&self) -> crate::ChartResult<()> {
        if !self.angle.is_finite()
            || self.angle <= 0.
            || self.angle >= 180.
            || !self.length_mm.is_finite()
            || self.length_mm < 0.
        {
            return Err(crate::scales::error(
                crate::DiagnosticCode::Validation,
                "Arrow angle and length are invalid.",
            ));
        }
        Ok(())
    }
}
impl Default for ArrowSpec {
    fn default() -> Self {
        Self {
            angle: 30.,
            length_mm: 6.35,
            ends: ArrowEnds::Last,
            closed: false,
        }
    }
}
/// Interval components emitted from one positioned row and one original target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IntervalKind {
    /// Stem only.
    LineRange,
    /// Stem and point at y.
    PointRange,
    /// Stem and endpoint caps.
    ErrorBar,
    /// Rectangle and central rule.
    Crossbar,
}
/// Optional independent paint controls for an interval component.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IntervalStroke {
    /// Component colour; absent inherits the row colour.
    pub color: Option<crate::color::Paint>,
    /// Component linewidth before physical unit conversion.
    pub linewidth: Option<f64>,
    /// Component dash pattern.
    pub line_type: Option<super::LineType>,
}
/// Point controls independent from its interval stem.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct IntervalPoint {
    /// Size before fatten; absent uses mapped size or reference 0.5.
    pub size: Option<f64>,
    /// Reference point shape code zero through 25.
    pub shape: Option<u8>,
    /// Interior fill for independently filled symbols.
    pub fill: Option<crate::color::Paint>,
    /// Point outline stroke, independent from stem linewidth.
    pub stroke: Option<f64>,
}
/// Canonical independent-x/dependent-y interval recipe; layer orientation transposes once.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalRecipe {
    /// Components.
    pub kind: IntervalKind,
    /// Full cap/box width; absent uses 0.9 times resolution.
    pub width: Option<f64>,
    /// Point-size or middle-line multiplier; absent uses family default.
    #[serde(default)]
    pub fatten: Option<f64>,
    /// Independent crossbar middle styling.
    #[serde(default)]
    pub middle: IntervalStroke,
    /// Independent crossbar box styling.
    #[serde(default)]
    pub box_style: IntervalStroke,
    /// Independent point-range point styling.
    #[serde(default)]
    pub point: IntervalPoint,
}
impl Default for IntervalRecipe {
    fn default() -> Self {
        Self {
            kind: IntervalKind::LineRange,
            width: None,
            fatten: None,
            middle: Default::default(),
            box_style: Default::default(),
            point: Default::default(),
        }
    }
}
/// Data-space reference-line family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ReferenceKind {
    /// y = slope*x + intercept.
    Abline,
    /// Constant dependent coordinate.
    Horizontal,
    /// Constant independent coordinate.
    Vertical,
}
/// Deferred reference line; bounds come from the resolved plot axes.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceRecipe {
    /// Equation kind.
    pub kind: ReferenceKind,
    /// Default slope (abline only).
    pub slope: f64,
    /// Default intercept.
    pub intercept: f64,
    /// Optional endpoint arrow.
    pub arrow: Option<ArrowSpec>,
}
/// Reference step transition direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StepDirection {
    /// Horizontal then vertical.
    Hv,
    /// Vertical then horizontal.
    Vh,
    /// Transition halfway between x observations.
    Mid,
}
/// Checked built-in recipes over common geometry, scales and provenance.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BuiltinRecipe {
    /// Fitted mean line with a separate uncertainty ribbon.
    Smooth,
    /// Compound interval geometry.
    Interval(IntervalRecipe),
    /// Deferred data-space reference line.
    Reference(ReferenceRecipe),
    /// Ordered steps using the shared curve kernel.
    Step(StepDirection),
    /// Segment with optional arrows.
    Segment {
        /// Arrow controls.
        arrow: Option<ArrowSpec>,
    },
    /// Group/subgroup polygon paths.
    Polygon(PolygonRecipe),
    /// Data-space rectangular tiles.
    Tile(TileRecipe),
    /// Offset-lattice hexagons using shared portable polygon geometry.
    Hexagon(TileRecipe),
    /// Sampled raster grid.
    Raster(RasterRecipe),
    /// Identity-count columns with data-space widths.
    Column(ColumnRecipe),
    /// Joint aesthetic count marks.
    Count(CountRecipe),
    /// Boundary rug marks.
    Rug(RugRecipe),
    /// Grid-compatible curved segments.
    Curve(CurveRecipe),
    /// Tukey boxes, whiskers, notches and outliers.
    Boxplot(Box<BoxplotRecipe>),
    /// Density area with selected boundary strokes.
    Density(super::DensityRecipe),
    /// Mirrored density polygons and quantile marks.
    Violin(ViolinRecipe),
    /// Binned circle stacks.
    Dotplot(DotplotRecipe),
    /// Mathematical angle/radius spokes.
    Spoke(SpokeRecipe),
}
/// Typed deferred recipe payloads; destinations use one shared projection dispatcher.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum PreparedRecipe {
    /// Distribution glyphs with destination-dependent diameter or point units.
    Distribution(PreparedDistribution),
    /// Surface recipe data.
    Surface(PreparedSurface),
    /// Mark/run recipe data.
    Mark(PreparedMarkRecipe),
    /// Reference-line equation and controls, without domain-training points.
    Reference(ReferenceRecipe),
}
impl PreparedRecipe {
    pub(crate) fn points(&self) -> Vec<crate::Point> {
        match self {
            Self::Distribution(v) => v.points(),
            Self::Surface(v) => v.points(),
            Self::Mark(v) => v.points(),
            Self::Reference(_) => vec![],
        }
    }
    pub(crate) fn transpose(&mut self) -> crate::ChartResult<()> {
        match self {
            Self::Distribution(v) => v.transpose(),
            Self::Surface(v) => v.transpose(),
            Self::Mark(v) => v.transpose(),
            Self::Reference(_) => Ok(()),
        }
    }
}
