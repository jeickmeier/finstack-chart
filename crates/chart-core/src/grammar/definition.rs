use super::{
    AutoBinSpec, CountSpec, DodgeSpec, JitterSpec, OlsSpec, StackSpec, StatAes, SummarySpec,
};
use crate::data::InvalidPolicy;
use crate::scene::Color;
use crate::{DatasetId, FieldId, LayerId, Revision, ScaleId, TransformId};

/// Exact builtin operation identity/version; unknown or mismatched registrations fail.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OperationRef {
    /// Registered name; no executable code is loaded by this descriptor.
    pub id: String,
    /// Definition version, independent of input data revision.
    pub version: Revision,
}
impl OperationRef {
    /// Select a versioned operation. Current builtins use version one.
    pub fn new(id: impl Into<String>, version: Revision) -> Self {
        Self {
            id: id.into(),
            version,
        }
    }
    pub(crate) fn builtin(id: &str) -> Self {
        Self::new(id, Revision::new(1))
    }
}

/// Dataset or named prepared output; dependency IDs are never array offsets.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum DataRef {
    /// Immutable registered dataset.
    Dataset(DatasetId),
    /// Named output in the definition's transform graph.
    Transform(TransformId),
}
impl From<DatasetId> for DataRef {
    fn from(id: DatasetId) -> Self {
        Self::Dataset(id)
    }
}
impl From<TransformId> for DataRef {
    fn from(id: TransformId) -> Self {
        Self::Transform(id)
    }
}

/// Portable coordinate/numeric mapping evaluated against source rows only.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Numeric {
    /// Numeric source field; integer precision is checked, timestamps require `Timestamp`.
    Field(FieldId),
    /// Stable categorical label projected through a band scale; invalid in numeric statistics.
    Category(FieldId),
    /// Explicit constant encoding, separate from a geometry's constant style.
    Literal(f64),
    /// Timestamp projection in original ticks after checked integer-origin subtraction.
    Timestamp {
        /// Source timestamp field.
        field: FieldId,
        /// Explicit origin in that field's integer units.
        #[serde(with = "crate::portable::signed")]
        origin: i64,
    },
}
impl From<FieldId> for Numeric {
    fn from(id: FieldId) -> Self {
        Self::Field(id)
    }
}

/// Stable source grouping within the declared statistical population scope.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Grouping {
    /// Whole filtered population; the default for histogram recipes.
    #[default]
    All,
    /// Independent groups of a categorical, UTF-8, integer or boolean field.
    Field(FieldId),
}

/// Source filtering changes the population before statistics; viewport actions do not.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SourceFilter {
    /// Source numeric mapping to compare.
    pub value: Numeric,
    /// Inclusive finite lower bound, if present.
    pub minimum: Option<f64>,
    /// Inclusive finite upper bound, if present.
    pub maximum: Option<f64>,
}

/// Supported versioned pre-stat numeric transform. No implicit scale transform is applied.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NumericTransform {
    /// Must resolve `chart.affine`, version one.
    pub operation: OperationRef,
    /// Finite nonzero multiplier.
    pub factor: f64,
    /// Finite offset; non-finite results are invalid inputs.
    pub offset: f64,
}
impl NumericTransform {
    /// Declare `factor * source + offset` before the declared statistic.
    pub fn affine(factor: f64, offset: f64) -> Self {
        Self {
            operation: OperationRef::builtin("chart.affine"),
            factor,
            offset,
        }
    }
}

/// Declared statistical input/output space, retained in prepared metadata.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum StatSpace {
    /// Source data units over the filtered population.
    #[default]
    Data,
    /// Explicit pre-stat transform; generated intervals already occupy this space.
    Transformed(NumericTransform),
}

/// Behavior outside explicit bin edges.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum OutlierPolicy {
    /// Exclude and report below/above counts.
    #[default]
    Exclude,
    /// Assign outside values to the nearest finite edge bin, retaining counts and membership.
    Overflow,
    /// Reject instead of excluding any finite value outside the edges.
    Error,
}

/// Explicit-edge bin parameters. Intervals are [left,right), with final right included.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BinSpec {
    /// Required source channel.
    pub input: Numeric,
    /// At least two finite strictly increasing edges in the declared calculation space.
    pub edges: Vec<f64>,
    /// Explicit treatment of values outside the edge range.
    pub outliers: OutlierPolicy,
    /// Population grouping, independent of visual highlight/viewport.
    pub grouping: Grouping,
    /// Space in which edges and generated intervals are expressed.
    pub space: StatSpace,
}
impl BinSpec {
    /// Whole-population source-space histogram with explicit edges and reported outliers.
    pub fn new(input: impl Into<Numeric>, edges: Vec<f64>) -> Self {
        Self {
            input: input.into(),
            edges,
            outliers: OutlierPolicy::Exclude,
            grouping: Grouping::All,
            space: StatSpace::Data,
        }
    }
}

/// Implemented statistical parameter contracts; generated outputs have a separate type.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum StatParameters {
    /// Parameters for an explicitly registered custom statistic.
    Custom(super::ExtensionParameters),
    /// Preserve source or already-generated rows/provenance.
    Identity,
    /// Thirty equal-width bins by default, over the eligible filtered population.
    AutoBin(AutoBinSpec),
    /// Count rows satisfying every declared required numeric input.
    Count(CountSpec),
    /// Exact grouped finite-value summaries and configurable interpolated quantiles.
    Summary(SummarySpec),
    /// Intercept ordinary least-squares over finite paired observations.
    Ols(OlsSpec),
    /// Aggregate source observations into explicit bins.
    Bin(BinSpec),
}

/// Operation registration and its parameters must agree before any data is evaluated.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Statistic {
    /// Known operation identity and version.
    pub operation: OperationRef,
    /// Typed builtin parameters.
    pub parameters: StatParameters,
}
impl Statistic {
    /// Select an explicitly registered extension; no executable code is serialized.
    pub fn custom(operation: OperationRef, parameters: super::ExtensionParameters) -> Self {
        Self {
            operation,
            parameters: StatParameters::Custom(parameters),
        }
    }
    /// Preserve input rows and their exact provenance.
    pub fn identity() -> Self {
        Self {
            operation: OperationRef::builtin("chart.identity"),
            parameters: StatParameters::Identity,
        }
    }
    /// Explicit bin operation, version one.
    pub fn bin(spec: BinSpec) -> Self {
        Self {
            operation: OperationRef::builtin("chart.bin"),
            parameters: StatParameters::Bin(spec),
        }
    }
}
impl Default for Statistic {
    fn default() -> Self {
        Self::identity()
    }
}

/// Named acyclic operation output that can feed multiple layers through shared storage.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TransformDefinition {
    /// Explicit missing-facet and panel targeting policy.
    #[serde(default)]
    pub facet: super::FacetTarget,
    /// Group, facet or whole-chart statistical population.
    #[serde(default)]
    pub scope: super::StatScope,
    /// Stable graph identity.
    pub id: TransformId,
    /// Dataset or earlier dependency (declaration order is irrelevant).
    pub input: DataRef,
    /// Ordered source filters applied before the statistic.
    pub filters: Vec<SourceFilter>,
    /// Registered computation.
    pub statistic: Statistic,
    /// Invalid required inputs either exclude with counts or reject.
    pub invalid: InvalidPolicy,
}
impl TransformDefinition {
    /// Declare a source or transform computation with no filters and exclusion diagnostics.
    pub fn new(id: TransformId, input: impl Into<DataRef>, statistic: Statistic) -> Self {
        Self {
            id,
            facet: super::FacetTarget::default(),
            scope: super::StatScope::default(),
            input: input.into(),
            filters: vec![],
            statistic,
            invalid: InvalidPolicy::Exclude,
        }
    }
}

/// Source-stage aesthetic mappings. Every mapped field is validated against each layer.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SourceAes {
    /// First horizontal coordinate.
    pub x: Option<Numeric>,
    /// First vertical coordinate.
    pub y: Option<Numeric>,
    /// Rule/rectangle second horizontal endpoint.
    pub x2: Option<Numeric>,
    /// Rule/rectangle second vertical endpoint or explicit baseline.
    pub y2: Option<Numeric>,
    /// OHLC lower price bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub low: Option<Numeric>,
    /// OHLC upper price bound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub high: Option<Numeric>,
    /// Positive point radius or stroke width, interpreted in eventual destination units.
    pub size: Option<Numeric>,
    /// Group lines independently; default is a single authored group.
    pub group: Option<FieldId>,
}
impl SourceAes {
    /// Begin explicit source mappings.
    pub fn new() -> Self {
        Self::default()
    }
    /// Bind x to a numeric field, literal, timestamp or explicit category encoding.
    /// Generated accessors cannot enter the source stage.
    ///
    /// ```compile_fail
    /// use chart_core::grammar::{SourceAes, StatField};
    /// let source = SourceAes::new().x(StatField::Count);
    /// ```
    pub fn x(mut self, value: impl Into<Numeric>) -> Self {
        self.x = Some(value.into());
        self
    }
    /// Bind y to a numeric field, literal, timestamp or explicit category encoding.
    pub fn y(mut self, value: impl Into<Numeric>) -> Self {
        self.y = Some(value.into());
        self
    }
    /// Bind the second x endpoint.
    pub fn x2(mut self, value: impl Into<Numeric>) -> Self {
        self.x2 = Some(value.into());
        self
    }
    /// Bind the second y endpoint/baseline.
    pub fn y2(mut self, value: impl Into<Numeric>) -> Self {
        self.y2 = Some(value.into());
        self
    }
    /// Bind OHLC low and high; y is open and y2 is close.
    pub fn bounds(mut self, low: impl Into<Numeric>, high: impl Into<Numeric>) -> Self {
        self.low = Some(low.into());
        self.high = Some(high.into());
        self
    }
    /// Bind point radius/stroke width; this is distinct from constant styling.
    pub fn size(mut self, value: impl Into<Numeric>) -> Self {
        self.size = Some(value.into());
        self
    }
    /// Bind a stable source group.
    pub fn group(mut self, field: FieldId) -> Self {
        self.group = Some(field);
        self
    }
    pub(crate) fn inherit(&self, base: &Self) -> Self {
        Self {
            x: self.x.clone().or_else(|| base.x.clone()),
            y: self.y.clone().or_else(|| base.y.clone()),
            x2: self.x2.clone().or_else(|| base.x2.clone()),
            y2: self.y2.clone().or_else(|| base.y2.clone()),
            low: self.low.clone().or_else(|| base.low.clone()),
            high: self.high.clone().or_else(|| base.high.clone()),
            size: self.size.clone().or_else(|| base.size.clone()),
            group: self.group.or(base.group),
        }
    }
}

/// Typed generated bin fields; these cannot be used as source field IDs/accessors.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum BinField {
    /// Left interval edge in the stat's output space.
    Start,
    /// Right interval edge in the stat's output space.
    End,
    /// Overflow-safe midpoint of the interval.
    Midpoint,
    /// Exact membership count (checked when projected to f64).
    Count,
}

/// Mapping evaluated against `BinnedRow`, never the original observation.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum BinNumeric {
    /// Generated typed field.
    Field(BinField),
    /// Explicit constant coordinate/baseline.
    Literal(f64),
}
impl From<BinField> for BinNumeric {
    fn from(field: BinField) -> Self {
        Self::Field(field)
    }
}

/// Generated-stage aesthetics, with no inheritance from source-row accessors.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BinAes {
    /// First horizontal coordinate.
    pub x: BinNumeric,
    /// First vertical coordinate.
    pub y: BinNumeric,
    /// Second horizontal endpoint for rules/rectangles.
    pub x2: Option<BinNumeric>,
    /// Second vertical endpoint or baseline.
    pub y2: Option<BinNumeric>,
    /// Optional destination-unit radius/stroke width mapping.
    pub size: Option<BinNumeric>,
}
impl BinAes {
    /// Map explicit generated x/y fields or literals.
    pub fn new(x: impl Into<BinNumeric>, y: impl Into<BinNumeric>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            x2: None,
            y2: None,
            size: None,
        }
    }
    /// Interval bars with an explicit zero baseline.
    pub fn histogram() -> Self {
        Self {
            x: BinField::Start.into(),
            y: BinField::Count.into(),
            x2: Some(BinField::End.into()),
            y2: Some(BinNumeric::Literal(0.)),
            size: None,
        }
    }
}

/// Stage-safe mapping choice. A generated mapping cannot accidentally read source fields.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Mappings {
    /// Source mappings, optionally inheriting chart defaults.
    Source(SourceAes),
    /// Explicit mappings of count, summary or fitted output fields.
    Statistical(StatAes),
    /// Explicit mappings of the bin output schema.
    Binned(BinAes),
}

/// Source/order policy for straight line runs.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum LineOrder {
    /// Increasing x; equal x values use stable source insertion ordinal.
    #[default]
    X,
    /// Preserve authored row order within each group.
    Authored,
}

/// Implemented geometry contracts. Coordinates remain in data/calculation space.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Geom {
    /// Circular points.
    Point,
    /// Straight runs; isolated valid points remain explicit one-vertex runs.
    Line {
        /// Ordering before gap splitting.
        order: LineOrder,
        /// Explicitly bridge invalid rows; false is the default recipe policy.
        connect_gaps: bool,
    },
    /// Filled run between y and an explicit calculation-space baseline.
    Area {
        /// Run ordering.
        order: LineOrder,
        /// Bridge missing values only when explicitly requested.
        connect_gaps: bool,
        /// Finite baseline; zero by default through `Geom::area()`.
        baseline: f64,
    },
    /// Filled run between lower y and upper y2 at each x; lower must not exceed upper.
    Ribbon {
        /// Run ordering.
        order: LineOrder,
        /// Bridge missing values only when explicitly requested.
        connect_gaps: bool,
    },
    /// Interval bar centered at x, from y to explicit y2 baseline.
    Bar {
        /// Positive width in destination units.
        width: f64,
        /// Reject negative y values independently (for volume and other nonnegative measures).
        nonnegative: bool,
    },
    /// OHLC candle: y=open, y2=close, low/high bound both.
    Ohlc {
        /// Positive body width in destination units.
        width: f64,
    },
    /// Rule from (x,y) to (x2,y2).
    Rule,
    /// Rectangle spanning both supplied endpoints, including its explicit baseline.
    Rectangle,
}
impl Geom {
    /// Zero-baseline area, ordered by x and split at gaps.
    pub fn area() -> Self {
        Self::Area {
            order: LineOrder::X,
            connect_gaps: false,
            baseline: 0.,
        }
    }
    /// Lower/upper ribbon, ordered by x and split at gaps.
    pub fn ribbon() -> Self {
        Self::Ribbon {
            order: LineOrder::X,
            connect_gaps: false,
        }
    }
    pub(crate) fn run(self) -> Option<(LineOrder, bool)> {
        match self {
            Self::Line {
                order,
                connect_gaps,
            }
            | Self::Area {
                order,
                connect_gaps,
                ..
            }
            | Self::Ribbon {
                order,
                connect_gaps,
            } => Some((order, connect_gaps)),
            _ => None,
        }
    }
    /// Straight x-ordered lines split at every invalid row by default.
    pub fn line() -> Self {
        Self::Line {
            order: LineOrder::X,
            connect_gaps: false,
        }
    }
}

/// Semantic positions precede domains; display adjustments follow scale projection.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Position {
    /// Preserve prepared endpoints.
    #[default]
    Identity,
    /// Add positive and negative heights separately, in explicit group order.
    Stack(StackSpec),
    /// Fixed band-relative slots; missing groups keep their configured slot.
    Dodge(DodgeSpec),
    /// Stable per-target displacement in declared data or destination units.
    Jitter(JitterSpec),
}

/// Constant solid styling, separate from data aesthetic mappings.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Style {
    /// Fill/stroke color, before any future palette scale.
    pub color: Color,
    /// Positive point radius in eventual destination units.
    pub radius: f64,
    /// Positive stroke width in eventual destination units.
    pub stroke_width: f64,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            color: Color {
                red: 35,
                green: 90,
                blue: 150,
                alpha: 255,
            },
            radius: 3.,
            stroke_width: 1.5,
        }
    }
}

/// Named positional scale bindings. IDs zero and one are the default x and y scales.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScaleBindings {
    /// Horizontal scale identity.
    pub x: ScaleId,
    /// Vertical scale identity.
    pub y: ScaleId,
}
impl Default for ScaleBindings {
    fn default() -> Self {
        Self {
            x: ScaleId::new(0),
            y: ScaleId::new(1),
        }
    }
}
/// Destination clip shared by painting and future hit testing/export.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ClipPolicy {
    /// Clip to the resolved plot rectangle.
    #[default]
    Plot,
    /// Declared annotation overflow, bounded by the figure.
    Figure,
}

/// One heterogeneous grammar layer, erased to portable fields before preparation.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    /// Optional versioned custom geometry after shared encoding and positioning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry_extension: Option<super::GeometryExtension>,
    /// Direction-dependent candle colors; supplied OHLC values remain unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candle_colors: Option<super::CandleColors>,
    /// Explicit missing-facet and panel targeting policy.
    #[serde(default)]
    pub facet: super::FacetTarget,
    /// Group, facet or whole-chart statistical population.
    #[serde(default)]
    pub scope: super::StatScope,
    /// Stable identity; vector order controls paint order.
    pub id: LayerId,
    /// Named horizontal and vertical scale bindings.
    pub scales: ScaleBindings,
    /// Explicit annotation overflow policy.
    pub clip: ClipPolicy,
    /// Dataset or shared transform output.
    pub data: DataRef,
    /// Source filters, before statistics and domains.
    pub filters: Vec<SourceFilter>,
    /// Shared built-in statistic, using the same implementation as named transforms.
    pub statistic: Statistic,
    /// Source or generated-stage encodings.
    pub mappings: Mappings,
    /// Whether missing source mappings inherit chart defaults. Generated mappings never do.
    pub inherit: bool,
    /// Portable geometry family.
    pub geom: Geom,
    /// Semantic position applied before domain collection.
    pub position: Position,
    /// Constant styling.
    pub style: Style,
    /// Optional stage-aware color mapping with semantic legend metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<super::ColorEncoding>,
    /// Invalid required values exclude with counts or fail preparation.
    pub invalid: InvalidPolicy,
}
impl Layer {
    /// Author a source identity layer; callers can supply independent mappings/schema.
    pub fn new(id: LayerId, data: impl Into<DataRef>, geom: Geom, mappings: SourceAes) -> Self {
        Self {
            id,
            geometry_extension: None,
            candle_colors: None,
            facet: super::FacetTarget::default(),
            scope: super::StatScope::default(),
            scales: ScaleBindings::default(),
            clip: ClipPolicy::default(),
            data: data.into(),
            filters: vec![],
            statistic: Statistic::identity(),
            mappings: Mappings::Source(mappings),
            inherit: true,
            geom,
            position: Position::Identity,
            style: Style::default(),
            color: None,
            invalid: InvalidPolicy::Exclude,
        }
    }
    /// Author a layer over a named bin output or an explicit bin statistic.
    pub fn binned(id: LayerId, data: impl Into<DataRef>, geom: Geom, mappings: BinAes) -> Self {
        Self {
            mappings: Mappings::Binned(mappings),
            inherit: false,
            ..Self::new(id, data, geom, SourceAes::new())
        }
    }
    /// Histogram recipe: ordinary bin stat + rectangle geom + typed generated mappings.
    pub fn histogram(id: LayerId, data: impl Into<DataRef>, spec: BinSpec) -> Self {
        Self {
            statistic: Statistic::bin(spec),
            ..Self::binned(id, data, Geom::Rectangle, BinAes::histogram())
        }
    }
    /// Interval bar recipe with an explicit zero baseline; signed values remain valid.
    pub fn bars(
        id: LayerId,
        data: impl Into<DataRef>,
        x: impl Into<Numeric>,
        value: impl Into<Numeric>,
        width: f64,
    ) -> Self {
        Self::new(
            id,
            data,
            Geom::Bar {
                width,
                nonnegative: false,
            },
            SourceAes::new().x(x).y(value).y2(Numeric::Literal(0.)),
        )
    }
    /// Nonnegative volume recipe, validated independently from any price layer.
    pub fn volume(
        id: LayerId,
        data: impl Into<DataRef>,
        x: impl Into<Numeric>,
        value: impl Into<Numeric>,
        width: f64,
    ) -> Self {
        Self::new(
            id,
            data,
            Geom::Bar {
                width,
                nonnegative: true,
            },
            SourceAes::new().x(x).y(value).y2(Numeric::Literal(0.)),
        )
    }
    /// OHLC candle recipe using x, y=open, y2=close and low/high source bounds.
    pub fn ohlc(id: LayerId, data: impl Into<DataRef>, mappings: SourceAes, width: f64) -> Self {
        Self::new(id, data, Geom::Ohlc { width }, mappings)
    }
    /// Rectangular heatmap cells using explicit x/x2/y/y2 intervals and color values.
    pub fn cells(
        id: LayerId,
        data: impl Into<DataRef>,
        mappings: SourceAes,
        color: super::ColorEncoding,
    ) -> Self {
        Self::new(id, data, Geom::Rectangle, mappings).colored(color)
    }
    /// Bind this layer to independently trained named scales.
    pub fn scaled(mut self, x: ScaleId, y: ScaleId) -> Self {
        self.scales = ScaleBindings { x, y };
        self
    }
    /// Bind color without changing positional domains.
    pub fn colored(mut self, color: super::ColorEncoding) -> Self {
        self.color = Some(color);
        self
    }
    /// Choose an explicit constant style.
    pub fn styled(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    /// Disable chart mapping inheritance for independent/annotation schemas.
    pub fn independent(mut self) -> Self {
        self.inherit = false;
        self
    }
}

/// Normalized chart definition; no native callbacks, windows or original typed rows.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ChartDefinition {
    /// Optional portable figure furniture and prepared-data inset views.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub figure: Option<crate::composition::FigureComposition>,
    /// Optional versioned headless theme cascade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<crate::theme::ThemeSpec>,
    /// Optional explicit facet catalog and panel layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facets: Option<super::FacetSpec>,
    /// Caller-owned definition revision; advance on effective authored changes.
    pub revision: Revision,
    /// Optional default source aesthetic mappings, checked per inheriting layer.
    pub mappings: SourceAes,
    /// Named operations; declaration order need not match dependency order.
    pub transforms: Vec<TransformDefinition>,
    /// Optional portable axes; when nonempty these replace destination default axes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub axes: Vec<crate::layout::AxisSpec>,
    /// Submission order is paint order; duplicate layer IDs reject.
    pub layers: Vec<Layer>,
}
impl ChartDefinition {
    /// Start an empty, valid definition at an explicit revision.
    pub fn new(revision: Revision) -> Self {
        Self {
            revision,
            facets: None,
            theme: None,
            figure: None,
            mappings: SourceAes::new(),
            transforms: vec![],
            axes: vec![],
            layers: vec![],
        }
    }
    /// Set inherited source mappings.
    pub fn mapped(mut self, mappings: SourceAes) -> Self {
        self.mappings = mappings;
        self
    }
    /// Append a named graph node.
    pub fn transform(mut self, transform: TransformDefinition) -> Self {
        self.transforms.push(transform);
        self
    }
    /// Author a portable named axis and guide.
    pub fn axis(mut self, axis: crate::layout::AxisSpec) -> Self {
        self.axes.push(axis);
        self
    }
    /// Append an erased heterogeneous layer in paint order.
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layers.push(layer);
        self
    }
}

/// Explicit compiler work budgets; source allocation and retained caller handles are separate.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CompileLimits {
    /// Maximum declared layers.
    pub max_layers: usize,
    /// Maximum named transform nodes.
    pub max_transforms: usize,
    /// Maximum filters on one computation.
    pub max_filters: usize,
    /// Maximum groups in one computation.
    pub max_groups: usize,
    /// Maximum edges in an explicit bin request.
    pub max_edges: usize,
    /// Maximum rows across prepared outputs; statistical rows charge one unit per generated field.
    pub max_prepared_rows: usize,
    /// Maximum total geometry vertices/endpoints across layers.
    pub max_vertices: usize,
}
impl Default for CompileLimits {
    fn default() -> Self {
        Self {
            max_layers: 256,
            max_transforms: 256,
            max_filters: 64,
            max_groups: 10_000,
            max_edges: 10_001,
            max_prepared_rows: 1_000_000,
            max_vertices: 1_000_000,
        }
    }
}
