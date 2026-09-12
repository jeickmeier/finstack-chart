use super::RadialParameters;
use super::{
    AutoBinSpec, CountSpec, DodgeSpec, JitterSpec, OlsSpec, ShapeStackSpec, StackSpec, StatAes,
    SummarySpec,
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
    /// Typed source-stage expression; reductions use the selected dataset before filtering and facets.
    Expression(super::Expression<super::SourceRead>),
    /// Explicit scale-stage numeric mapping; retained space prevents a second transform.
    Scaled {
        /// Prior source-stage numeric mapping.
        input: Box<Numeric>,
        /// Resolved transform, limits, out-of-bounds policy and scale identity.
        scale: Box<super::ScaleProjection>,
    },
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
impl From<super::Expression<super::SourceRead>> for Numeric {
    fn from(value: super::Expression<super::SourceRead>) -> Self {
        Self::Expression(value)
    }
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
    /// Ordered interaction of distinct exact fields; missing values are an explicit group.
    /// The vector is bounded by the compiler's filter budget and may contain one field.
    Interaction(Vec<FieldId>),
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
    fn requires_stages(&self) -> bool {
        if matches!(self.grouping(), Some(Grouping::Interaction(_))) {
            return true;
        }
        let staged = |n: &Numeric| matches!(n, Numeric::Scaled { .. } | Numeric::Expression(_));
        match &self.parameters {
            StatParameters::Bin(s) => staged(&s.input),
            StatParameters::AutoBin(s) => staged(&s.input),
            StatParameters::Summary(s) => staged(&s.input),
            StatParameters::Ols(s) => staged(&s.x) || staged(&s.y),
            StatParameters::Count(s) => s.required.iter().any(staged),
            _ => false,
        }
    }
    /// Declared source population grouping; identity operations preserve their input groups.
    pub fn grouping(&self) -> Option<&Grouping> {
        match &self.parameters {
            StatParameters::Identity => None,
            StatParameters::Bin(s) => Some(&s.grouping),
            StatParameters::AutoBin(s) => Some(&s.grouping),
            StatParameters::Count(s) => Some(&s.grouping),
            StatParameters::Summary(s) => Some(&s.grouping),
            StatParameters::Ols(s) => Some(&s.grouping),
            StatParameters::Custom(s) => Some(&s.grouping),
        }
    }

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
    /// Retained source-stage context for compatibility grouping and scale preparation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grammar: Option<super::TransformGrammar>,
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
            grammar: None,
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
    /// Explicit whole-population or multi-field grouping, overriding the legacy group field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grouping: Option<Grouping>,
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
    fn requires_stages(&self) -> bool {
        self.grouping.is_some()
            || [
                &self.x, &self.y, &self.x2, &self.y2, &self.low, &self.high, &self.size,
            ]
            .into_iter()
            .flatten()
            .any(|n| matches!(n, Numeric::Scaled { .. } | Numeric::Expression(_)))
    }
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
        self.grouping = None;
        self
    }
    /// Explicit whole population or exact field interaction.
    pub fn grouped(mut self, grouping: Grouping) -> Self {
        self.grouping = Some(grouping);
        self.group = None;
        self
    }
    pub(crate) fn inherit(&self, base: &Self) -> Self {
        Self {
            grouping: self.grouping.clone().or_else(|| {
                self.group
                    .is_none()
                    .then(|| base.grouping.clone())
                    .flatten()
            }),
            x: self.x.clone().or_else(|| base.x.clone()),
            y: self.y.clone().or_else(|| base.y.clone()),
            x2: self.x2.clone().or_else(|| base.x2.clone()),
            y2: self.y2.clone().or_else(|| base.y2.clone()),
            low: self.low.clone().or_else(|| base.low.clone()),
            high: self.high.clone().or_else(|| base.high.clone()),
            size: self.size.clone().or_else(|| base.size.clone()),
            group: self
                .group
                .or_else(|| self.grouping.is_none().then_some(base.group).flatten()),
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
    /// Expression over generated bin fields after scale back-transformation.
    Expression(super::Expression<BinField>),
    /// Generated typed field.
    Field(BinField),
    /// Explicit constant coordinate/baseline.
    Literal(f64),
}
impl From<super::Expression<BinField>> for BinNumeric {
    fn from(value: super::Expression<BinField>) -> Self {
        Self::Expression(value)
    }
}
impl From<f64> for BinNumeric {
    fn from(value: f64) -> Self {
        Self::Literal(value)
    }
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

impl Mappings {
    fn requires_stages(&self) -> bool {
        match self {
            Self::Source(a) => a.requires_stages(),
            Self::Statistical(a) => [
                Some(&a.x),
                Some(&a.y),
                a.x2.as_ref(),
                a.y2.as_ref(),
                a.size.as_ref(),
            ]
            .into_iter()
            .flatten()
            .any(|v| matches!(v, super::StatNumeric::Expression(_))),
            Self::Binned(a) => [
                Some(&a.x),
                Some(&a.y),
                a.x2.as_ref(),
                a.y2.as_ref(),
                a.size.as_ref(),
            ]
            .into_iter()
            .flatten()
            .any(|v| matches!(v, BinNumeric::Expression(_))),
        }
    }
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
    /// Deferred hierarchy nodes/edges calculated after destination panel allocation.
    Hierarchy,
    /// Radial authored runs around one x/y center per run, with destination-unit radii.
    ShapeLineRadial {
        /// Authored order, or start-angle order for X.
        order: LineOrder,
        /// Explicitly bridge missing polar coordinates.
        connect_gaps: bool,
        /// Complete checked Cartesian curve after polar conversion.
        curve: crate::shape::CurveSpec,
        /// Constant start angle and inner radius, overridden by Angle/Radius channels.
        parameters: RadialParameters,
    },
    /// Paired radial boundaries around one x/y center per run.
    ShapeAreaRadial {
        /// Authored order, or start-angle order for X.
        order: LineOrder,
        /// Explicitly bridge missing polar pairs.
        connect_gaps: bool,
        /// Checked area-capable curve.
        curve: crate::shape::CurveSpec,
        /// Constant independent boundaries, overridden by named radial channels.
        parameters: RadialParameters,
    },
    /// One edge per row from x/y to x2/y2, curved after both endpoint projections.
    ShapeLink {
        /// Generic two-point curve; BumpX/BumpY produce horizontal/vertical tangents.
        curve: crate::shape::CurveSpec,
    },
    /// One radial edge per row, centered at x/y with named polar endpoints.
    ShapeLinkRadial {
        /// Source lower and target upper polar endpoints, overridden by named channels.
        parameters: RadialParameters,
    },
    /// Area/stroke-size symbols, independent of legacy point radius semantics.
    ShapeSymbol {
        /// Constant type, overridden by an explicit categorical symbol mapping.
        kind: crate::shape::SymbolKind,
        /// Area/stroke size, overridden by the AreaSize numeric channel.
        size: f64,
        /// Filled/stroked policy; Auto follows the chosen symbol topology.
        paint: crate::shape::SymbolPaint,
    },
    /// One circular sector per row; x/y is the center and radii use destination units.
    ShapeArc {
        /// Constant radii, sweep, padding and corners, overridden by named numeric mappings.
        parameters: crate::shape::ArcParameters,
    },
    /// Grouped pie layout feeding the same arc engine; named PieValue supplies weights.
    ShapePie {
        /// Constant radii/corners; layout replaces datum start/end/pad angles.
        parameters: crate::shape::ArcParameters,
        /// Pie-wide sweep and padding.
        angles: crate::shape::PieAngles,
        /// Stable angular sorting, independent of source output order.
        order: crate::shape::PieOrder,
        /// Partition weights by retained row group; false combines the current layer/panel.
        grouped: bool,
    },
    /// Authored-order D3 line geometry; curves evaluate after destination projection.
    ShapeLine {
        /// Source order within each group.
        order: LineOrder,
        /// Explicitly bridge missing coordinates.
        connect_gaps: bool,
        /// Complete checked built-in curve family.
        curve: crate::shape::CurveSpec,
    },
    /// General D3 area: (x,y) lower and independent (x2,y2) upper coordinates.
    ShapeArea {
        /// Source order within each group.
        order: LineOrder,
        /// Explicitly bridge missing paired coordinates.
        connect_gaps: bool,
        /// Area-capable checked curve; bundle is rejected.
        curve: crate::shape::CurveSpec,
    },
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
    pub(crate) fn reference_linewidth(self) -> bool {
        matches!(
            self,
            Self::Line { .. }
                | Self::Area { .. }
                | Self::Ribbon { .. }
                | Self::Bar { .. }
                | Self::Rule
                | Self::Rectangle
        )
    }
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
            Self::ShapeLineRadial {
                order,
                connect_gaps,
                ..
            }
            | Self::ShapeAreaRadial {
                order,
                connect_gaps,
                ..
            }
            | Self::ShapeLine {
                order,
                connect_gaps,
                ..
            }
            | Self::ShapeArea {
                order,
                connect_gaps,
                ..
            }
            | Self::Line {
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
    /// Reference order/offset stack over explicit tidy groups and sorted samples.
    ShapeStack(ShapeStackSpec),
    /// Fixed band-relative slots; missing groups keep their configured slot.
    Dodge(DodgeSpec),
    /// Stable per-target displacement in declared data or destination units.
    Jitter(JitterSpec),
}

/// Constant solid styling, separate from data aesthetic mappings.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Style<P = Color> {
    /// Fill/stroke color, before any future palette scale.
    pub color: P,
    /// Explicit alpha replaces paint alpha, matching the ggplot2 alpha aesthetic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,
    /// Independent fill override; absent preserves the established color channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<P>,
    /// Independent outline override; transparent paint suppresses the outline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<P>,
    /// Explicit dimensional units; absent preserves destination units.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<super::AestheticUnits>,
    /// Independent line type, with dash lengths relative to stroke width.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_type: Option<super::LineType>,
    /// Positive point radius in eventual destination units.
    pub radius: f64,
    /// Positive stroke width in eventual destination units.
    pub stroke_width: f64,
}
impl<P: From<Color>> Default for Style<P> {
    fn default() -> Self {
        Self {
            color: Color {
                red: 35,
                green: 90,
                blue: 150,
                alpha: 255,
            }
            .into(),
            alpha: None,
            units: None,
            line_type: None,
            fill: None,
            stroke: None,
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
    /// Explicit style constants override their corresponding mapped channels.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub aesthetic_values:
        std::collections::BTreeMap<super::ValueAesthetic, crate::interpolate::Value>,
    /// Text and line-type channels evaluated through the shared typed scale engine.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub value_scales: std::collections::BTreeMap<super::ValueAesthetic, super::NumericEncoding>,
    /// Independently trained fill and outline mappings, evaluated by the common color engine.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub paint_scales: std::collections::BTreeMap<super::PaintAesthetic, super::ColorEncoding>,
    /// Explicit shared hierarchy topology and layout recipe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchy: Option<super::HierarchyRecipe>,
    /// Versioned native shape protocols selected in the shared compiler (wire v9).
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub shape_protocols: std::collections::BTreeMap<super::ShapeFamily, super::ShapeOperation>,
    /// Explicit categorical symbol types, independent of color and grouping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<super::SymbolEncoding>,
    /// Input samples evaluated through the actual area-size mapping for guide glyphs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol_size_guide: Option<super::SymbolSizeGuide>,
    /// Independent numeric style scales, prepared after statistics and before after-scale expressions.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub numeric_scales: std::collections::BTreeMap<super::NumericAesthetic, super::NumericEncoding>,
    /// Physical independent-axis direction; mappings remain in physical x/y coordinates.
    #[serde(default, skip_serializing_if = "super::Orientation::is_vertical")]
    pub orientation: super::Orientation,
    /// Nonpositional expressions evaluated after scales and positions, before mark styling.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub after_scale: std::collections::BTreeMap<
        super::AfterScaleAesthetic,
        super::Expression<super::AfterScaleRead>,
    >,
    /// Source-stage mappings retained independently of generated encodings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grammar: Option<super::LayerGrammar>,
    /// Optional versioned custom geometry after shared encoding and positioning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry_extension: Option<super::GeometryExtension>,
    /// Direction-dependent candle colors; supplied OHLC values remain unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candle_colors: Option<super::CandleColors<crate::color::Paint>>,
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
    pub style: Style<crate::color::Paint>,
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
            aesthetic_values: Default::default(),
            value_scales: Default::default(),
            paint_scales: Default::default(),
            hierarchy: None,
            id,
            grammar: None,
            orientation: super::Orientation::Vertical,
            after_scale: Default::default(),
            numeric_scales: Default::default(),
            shape_protocols: Default::default(),
            symbol: None,
            symbol_size_guide: None,
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
    pub fn styled<P: Into<crate::color::Paint>>(mut self, style: Style<P>) -> Self {
        self.style = style.map_color(Into::into);
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
    /// Canonical compatibility provenance and resolved execution policy.
    /// Absence preserves LibraryV1 byte and behavioral defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantics: Option<super::ExecutionSemantics>,
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
    /// Independently identified guides reusing declared positional scales (wire v8).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub guides: Vec<crate::layout::GuideSpec>,
    /// Submission order is paint order; duplicate layer IDs reject.
    pub layers: Vec<Layer>,
}
impl ChartDefinition {
    /// Minimum definition-envelope version required by its retained capabilities.
    pub fn wire_version(&self) -> u32 {
        if super::interpolation_extensions::mapped_scales(self).any(|s| s.limits_function.is_some())
        {
            return 28;
        }
        if super::interpolation_extensions::mapped_scales(self).any(|scale| {
            matches!(scale.ggplot.as_deref(), Some(crate::scales::GgplotScalePolicy::Binned(p)) if p.palette.is_some())
        }) { return 27; }
        if super::interpolation_extensions::mapped_scales(self).any(|scale| {
            matches!(scale.guide.as_deref(), Some(crate::scales::GgplotScaleGuide::Temporal(g)) if g.arguments != crate::scales::GgplotTemporalGuideArguments::default())
                || matches!(&scale.function, crate::scales::ScaleFunctionSpec::Interpolated(s) if matches!(s.normalization, crate::scales::NormalizationSpec::Ggplot { timestamp: Some(crate::scales::GgplotTimestampNormalization { date: true, .. }), .. }))
        }) {
            return 26;
        }
        if super::interpolation_extensions::mapped_scales(self).any(|scale| {
            matches!(&scale.function, crate::scales::ScaleFunctionSpec::Interpolated(s)
                if matches!(s.normalization, crate::scales::NormalizationSpec::Ggplot { timestamp: Some(_), .. }))
        }) {
            return 25;
        }
        if super::interpolation_extensions::mapped_scales(self).any(|scale| {
            matches!(
                scale.guide.as_deref(),
                Some(crate::scales::GgplotScaleGuide::Temporal(_))
            )
        }) {
            return 24;
        }

        if self
            .axes
            .iter()
            .any(|axis| axis.discrete.as_ref().is_some_and(|p| p.palette.is_some()))
        {
            return 23;
        }
        if self.axes.iter().any(|axis| axis.discrete.is_some())
            || self
                .axes
                .iter()
                .map(|axis| &axis.guide)
                .chain(self.guides.iter().map(|guide| &guide.style))
                .any(|guide| {
                    guide
                        .tick_values
                        .iter()
                        .flatten()
                        .chain(guide.guide_ticks.iter().flatten().map(|tick| &tick.value))
                        .any(|value| {
                            matches!(value, crate::composition::ScaleValue::MissingCategory)
                        })
                })
            || self.figure.as_ref().is_some_and(|figure| {
                figure
                    .annotations
                    .iter()
                    .flat_map(|a| std::iter::once(&a.anchor).chain(a.callout.as_ref()))
                    .chain(figure.paths.iter().map(|path| &path.anchor))
                    .any(|anchor| {
                        matches!(
                            anchor,
                            crate::composition::Anchor::Data {
                                x: crate::composition::ScaleValue::MissingCategory,
                                ..
                            } | crate::composition::Anchor::Data {
                                y: crate::composition::ScaleValue::MissingCategory,
                                ..
                            }
                        )
                    })
            })
        {
            return 22;
        }
        if super::interpolation_extensions::mapped_scales(self).any(|scale| {
            matches!(
                scale.ggplot.as_deref(),
                Some(crate::scales::GgplotScalePolicy::Discrete {
                    empty_population: true,
                    ..
                })
            )
        }) {
            return 21;
        }
        if self
            .axes
            .iter()
            .any(|axis| axis.continuous_limits.is_some())
        {
            return 20;
        }
        if self
            .axes
            .iter()
            .map(|a| &a.guide)
            .chain(self.guides.iter().map(|g| &g.style))
            .any(|g| g.minor_breaks.is_some())
        {
            return 19;
        }
        if self
            .axes
            .iter()
            .any(|a| matches!(a.scale, crate::layout::AxisScale::Binned { .. }))
            || self.has_scale_mapping(|scale| scale.binned.is_some())
        {
            return 18;
        }
        if self
            .axes
            .iter()
            .map(|axis| &axis.guide)
            .chain(self.guides.iter().map(|guide| &guide.style))
            .any(|guide| {
                matches!(
                    guide.tick_format,
                    Some(crate::layout::GuideFormatter::GgplotTime(_))
                ) || guide
                    .tick_arguments
                    .as_ref()
                    .is_some_and(|args| args.seconds.is_some() || args.time_width.is_some())
            })
            || self.axes.iter().any(|axis| {
                if let crate::layout::AxisScale::Secondary { source, .. } = axis.scale {
                    self.axes.iter().any(|primary| {
                        primary.id == source
                            && matches!(
                                primary.scale,
                                crate::layout::AxisScale::Utc { .. }
                                    | crate::layout::AxisScale::Date { .. }
                                    | crate::layout::AxisScale::Calendar { .. }
                                    | crate::layout::AxisScale::Auto
                            )
                    })
                } else {
                    false
                }
            })
            || self.has_reference_scale_mapping()
            || self.axes.iter().any(|axis| {
                axis.expansion.is_some()
                    || matches!(
                        axis.scale,
                        crate::layout::AxisScale::Date { .. }
                            | crate::layout::AxisScale::Duration(_)
                    )
                    || matches!(
                        axis.scale,
                        crate::layout::AxisScale::Secondary {
                            transform: Some(_),
                            ..
                        }
                    )
                    || matches!(
                        axis.scale,
                        crate::layout::AxisScale::Nonlinear {
                            transform: crate::scales::ScaleTransform::Reverse
                                | crate::scales::ScaleTransform::Sqrt,
                            ..
                        }
                    )
            })
            || self.layers.iter().any(|layer| {
                layer
                    .value_scales
                    .contains_key(&super::ValueAesthetic::Shape)
                    || layer
                        .aesthetic_values
                        .contains_key(&super::ValueAesthetic::Shape)
            })
        {
            return 17;
        }
        if self.layers.iter().any(|l| l.numeric_scales.values().chain(l.value_scales.values()).any(|s|s.scale.has_ggplot()) || l.color.iter().chain(l.paint_scales.values()).any(|c|matches!(&c.scale,crate::scales::ColorScale::Mapped {scale,..} if scale.has_ggplot()))) {return 17;}

        if self.layers.iter().any(|l| {
            matches!(
                l.geom,
                Geom::ShapeSymbol {
                    kind: crate::shape::SymbolKind::Ggplot(_),
                    ..
                } | Geom::ShapeSymbol {
                    paint: crate::shape::SymbolPaint::FillStroke
                        | crate::shape::SymbolPaint::ColorFill
                        | crate::shape::SymbolPaint::ColorFillStroke,
                    ..
                }
            ) || l.symbol.as_ref().is_some_and(|s| {
                s.palette
                    .iter()
                    .chain(s.missing.iter())
                    .any(|k| matches!(k, crate::shape::SymbolKind::Ggplot(_)))
            }) || l
                .grammar
                .as_ref()
                .is_some_and(|g| g.default_radius.is_some() || g.default_line_width.is_some())
                || l.after_scale.keys().any(|a| {
                    !matches!(
                        a,
                        super::AfterScaleAesthetic::Size | super::AfterScaleAesthetic::Color
                    )
                })
                || !l.value_scales.is_empty()
                || !l.aesthetic_values.is_empty()
                || l.style.alpha.is_some()
                || l.numeric_scales
                    .contains_key(&super::NumericAesthetic::Alpha)
                || l.style.units.is_some()
                || l.style.line_type.is_some()
                || !l.paint_scales.is_empty()
                || l.style.fill.is_some()
                || l.style.stroke.is_some()
                || (l.geom == Geom::Point
                    && l.numeric_scales
                        .contains_key(&super::NumericAesthetic::AreaSize))
        }) {
            return 16;
        }
        if self
            .layers
            .iter()
            .any(|l| l.hierarchy.is_some() || l.geom == Geom::Hierarchy)
        {
            return 15;
        }
        if self.axes.iter().any(|a| a.components.is_some())
            || self.guides.iter().any(|a| a.components.is_some())
        {
            return 14;
        }
        if self.axes.iter().any(|axis| axis.geometry.is_some())
            || self.guides.iter().any(|guide| guide.geometry.is_some())
        {
            return 13;
        }
        if super::interpolation_extensions::mapped_scales(self)
            .any(crate::scales::MappedScaleSpec::has_registered_interpolation)
        {
            return 12;
        }
        if self
            .axes
            .iter()
            .map(|axis| &axis.guide)
            .chain(self.guides.iter().map(|guide| &guide.style))
            .any(crate::layout::GuideStyle::uses_tick_configuration)
        {
            return 11;
        }
        if self
            .axes
            .iter()
            .any(|axis| matches!(axis.scale, crate::layout::AxisScale::Registered { .. }))
        {
            return 10;
        }
        if self.layers.iter().any(|l| !l.shape_protocols.is_empty()) {
            return 9;
        }
        if !self.guides.is_empty() {
            return 8;
        }
        if self.layers.iter().any(|l| {
            matches!(
                l.geom,
                Geom::ShapeSymbol { .. }
                    | Geom::ShapeLineRadial { .. }
                    | Geom::ShapeAreaRadial { .. }
                    | Geom::ShapeLink { .. }
                    | Geom::ShapeLinkRadial { .. }
                    | Geom::ShapeLine { .. }
                    | Geom::ShapeArea { .. }
                    | Geom::ShapeArc { .. }
                    | Geom::ShapePie { .. }
            ) || matches!(l.position, Position::ShapeStack(_))
        }) {
            return 7;
        }
        if self.layers.iter().any(|l|l.numeric_scales.values().any(|s|s.scale.has_chromatic())
            || l.color.as_ref().is_some_and(|c|matches!(&c.scale,crate::scales::ColorScale::Mapped {scale,..} if scale.has_chromatic()))) { return 6; }
        if self.layers.iter().any(|l| {
            !l.numeric_scales.is_empty()
                || l.color
                    .as_ref()
                    .is_some_and(|c| matches!(c.scale, crate::scales::ColorScale::Mapped { .. }))
        }) {
            return 5;
        }
        if self.axes.iter().any(|a| {
            a.numeric_format.is_some()
                || a.time_format.is_some()
                || matches!(
                    a.scale,
                    crate::layout::AxisScale::Numeric(_)
                        | crate::layout::AxisScale::D3Band(_)
                        | crate::layout::AxisScale::D3Point(_)
                        | crate::layout::AxisScale::Calendar { .. }
                )
        }) {
            return 5;
        }
        if self.has_floating_paint() {
            return 4;
        }
        if self.theme.as_ref().is_some_and(|t| t.geometry.is_some())
            || self.semantics.is_some()
            || self.mappings.requires_stages()
            || self
                .axes
                .iter()
                .any(|a| a.population_oob.is_some() || a.scale_stage.is_some())
            || self.layers.iter().any(|l| {
                l.orientation == super::Orientation::Horizontal
                    || !l.after_scale.is_empty()
                    || l.grammar.is_some()
                    || l.mappings.requires_stages()
                    || l.statistic.requires_stages()
                    || l.filters
                        .iter()
                        .any(|f| matches!(f.value, Numeric::Scaled { .. } | Numeric::Expression(_)))
                    || l.color.as_ref().is_some_and(|c| {
                        matches!(
                            c.input,
                            super::ColorInput::GroupField(_)
                                | super::ColorInput::Numeric(
                                    Numeric::Scaled { .. } | Numeric::Expression(_)
                                )
                        )
                    })
            })
            || self.transforms.iter().any(|t| {
                t.grammar.is_some()
                    || t.statistic.requires_stages()
                    || t.filters
                        .iter()
                        .any(|f| matches!(f.value, Numeric::Scaled { .. } | Numeric::Expression(_)))
            })
        {
            3
        } else {
            self.figure.as_ref().map_or(1, |f| f.version)
        }
    }
    /// Start an empty, valid definition at an explicit revision.
    pub fn new(revision: Revision) -> Self {
        Self {
            revision,
            semantics: None,
            facets: None,
            theme: None,
            figure: None,
            mappings: SourceAes::new(),
            transforms: vec![],
            axes: vec![],
            guides: vec![],
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

impl<P> Style<P> {
    /// Transform the authored color while preserving numeric styling.
    pub fn map_color<Q>(self, mut map: impl FnMut(P) -> Q) -> Style<Q> {
        Style {
            color: map(self.color),
            alpha: self.alpha,
            units: self.units,
            line_type: self.line_type,
            fill: self.fill.map(&mut map),
            stroke: self.stroke.map(&mut map),
            radius: self.radius,
            stroke_width: self.stroke_width,
        }
    }
}
impl Style<crate::color::Paint> {
    /// Resolve constant paint once before preparing marks.
    pub fn resolve(self) -> Style {
        self.map_color(crate::color::Paint::resolve)
    }
}

impl ChartDefinition {
    /// Walk authored paint inputs without converting colors or inspecting source rows.
    pub fn has_floating_paint(&self) -> bool {
        self.theme.as_ref().is_some_and(crate::theme::ThemeSpec::has_floating_paint)
            || self.figure.as_ref().is_some_and(crate::composition::FigureComposition::has_floating_paint)
            || self.axes.iter().any(|a| a.title.as_ref().is_some_and(crate::typography::RichText::has_floating_paint)
                || a.typography.as_ref().is_some_and(|r| r.color.is_some_and(crate::color::Paint::is_floating)))
            || self.layers.iter().any(|l| l.style.color.is_floating()
                || l.style.fill.is_some_and(crate::color::Paint::is_floating)
                || l.style.stroke.is_some_and(crate::color::Paint::is_floating)
                || l.paint_scales.values().any(|c| c.scale.has_floating())
                || l.candle_colors.is_some_and(|c| c.up.is_floating() || c.down.is_floating())
                || l.color.as_ref().is_some_and(|c| c.scale.has_floating())
                || l.after_scale.values().any(|e| e.nodes.iter().any(|n| matches!(n, super::ExpressionNode::Literal(super::ExpressionValue::Color(p)) if p.is_floating()))))
    }
}

impl ChartDefinition {
    // Authored scale projections can occur without an AxisSpec, including inside a
    // statistic or a nonpositional mapping. They share the same wire capability.
    fn has_reference_scale_mapping(&self) -> bool {
        self.has_scale_mapping(|scale| {
            scale.timestamp.is_some()
                || matches!(
                    scale.transform,
                    Some(
                        crate::scales::ScaleTransform::Reverse
                            | crate::scales::ScaleTransform::Sqrt
                    )
                )
        })
    }
    fn has_scale_mapping(&self, predicate: fn(&super::ScaleProjection) -> bool) -> bool {
        let numeric = |mut value: &Numeric| {
            while let Numeric::Scaled { input, scale } = value {
                if predicate(scale) {
                    return true;
                }
                value = input;
            }
            false
        };
        let source = |a: &SourceAes| {
            [&a.x, &a.y, &a.x2, &a.y2, &a.low, &a.high, &a.size]
                .into_iter()
                .flatten()
                .any(numeric)
        };
        let statistic = |s: &Statistic| match &s.parameters {
            StatParameters::Bin(s) => numeric(&s.input),
            StatParameters::AutoBin(s) => numeric(&s.input),
            StatParameters::Summary(s) => numeric(&s.input),
            StatParameters::Ols(s) => numeric(&s.x) || numeric(&s.y),
            StatParameters::Count(s) => s.required.iter().any(numeric),
            _ => false,
        };
        let color =
            |c: &super::ColorInput| matches!(c, super::ColorInput::Numeric(n) if numeric(n));
        source(&self.mappings)
            || self.transforms.iter().any(|t| {
                statistic(&t.statistic)
                    || t.grammar.as_ref().is_some_and(|g| source(&g.source))
                    || t.filters.iter().any(|f| numeric(&f.value))
            })
            || self.layers.iter().any(|l| {
                matches!(&l.mappings, Mappings::Source(a) if source(a))
                    || l.grammar.as_ref().is_some_and(|g| source(&g.source))
                    || statistic(&l.statistic)
                    || l.filters.iter().any(|f| numeric(&f.value))
                    || l.color
                        .iter()
                        .chain(l.paint_scales.values())
                        .any(|c| color(&c.input))
                    || l.symbol.as_ref().is_some_and(|s| color(&s.input))
                    || l.numeric_scales
                        .values()
                        .chain(l.value_scales.values())
                        .any(|s| color(&s.input))
            })
    }
}
