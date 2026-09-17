use super::*;
use crate::RowKey;
use crate::provenance::Target;
use std::sync::Arc;

/// Automatic edges use the filtered eligible population, independently of zoom.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AutoBinSpec {
    /// Reference break and weighting policies; absent preserves legacy equal-width bins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ggplot: Option<GgplotBinOptions>,
    /// Numeric input.
    pub input: Numeric,
    /// Positive number of equal-width bins; defaults to thirty.
    pub bins: usize,
    /// Independent population groups; edges span all eligible groups.
    pub grouping: Grouping,
    /// Explicit input calculation space.
    pub space: StatSpace,
}
impl AutoBinSpec {
    /// Thirty equal-width bins in source units.
    pub fn new(input: impl Into<Numeric>) -> Self {
        Self {
            ggplot: None,
            input: input.into(),
            bins: 30,
            grouping: Grouping::All,
            space: StatSpace::Data,
        }
    }
}
/// Required numeric fields and grouping for a count; an empty list counts valid groups.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CountSpec {
    /// Explicit reference count-by-position policy; absent preserves raw group counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ggplot: Option<GgplotCountOptions>,
    /// Every listed input must be finite and exactly representable.
    pub required: Vec<Numeric>,
    /// Population partition.
    pub grouping: Grouping,
}
/// Finite-value exact summary; empty numeric outputs are missing by default.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SummarySpec {
    /// Reference summary-by-position helper; absent preserves raw descriptive summaries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ggplot: Option<GgplotSummaryOptions>,
    /// Numeric input.
    pub input: Numeric,
    /// Population partition.
    pub grouping: Grouping,
    /// Requested probabilities in authored order, each in `[0,1]`.
    pub quantiles: Vec<f64>,
    /// Explicitly return zero instead of missing for an empty sum only.
    pub empty_sum_zero: bool,
    /// Input calculation space; count remains dimensionless.
    pub space: StatSpace,
}
impl SummarySpec {
    /// Count/min/max/mean/sum and median, in data units.
    pub fn new(input: impl Into<Numeric>) -> Self {
        Self {
            input: input.into(),
            grouping: Grouping::All,
            ggplot: None,
            quantiles: vec![0.5],
            empty_sum_zero: false,
            space: StatSpace::Data,
        }
    }
}
/// Intercept OLS produces two fitted endpoints plus model coefficients and membership.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OlsSpec {
    /// Predictor input.
    pub x: Numeric,
    /// Response input.
    pub y: Numeric,
    /// Independent model groups.
    pub grouping: Grouping,
    /// Predictor calculation space.
    pub x_space: StatSpace,
    /// Response calculation space.
    pub y_space: StatSpace,
}
impl OlsSpec {
    /// Fit an intercept and slope in data units, without confidence bands.
    pub fn new(x: impl Into<Numeric>, y: impl Into<Numeric>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            grouping: Grouping::All,
            x_space: StatSpace::Data,
            y_space: StatSpace::Data,
        }
    }
}
impl Statistic {
    /// Construct a shared spatial statistic.
    pub fn spatial(spec: SpatialSpec) -> Self {
        Self {
            operation: OperationRef::builtin("chart.spatial"),
            parameters: StatParameters::Spatial(Box::new(spec)),
        }
    }
    /// Select a built-in distribution statistic with explicit estimator controls.
    pub fn distribution(spec: DistributionSpec) -> Self {
        Self {
            operation: OperationRef::builtin("chart.distribution"),
            parameters: StatParameters::Distribution(spec),
        }
    }
    /// Select a built-in univariate analysis or source-row helper.
    pub fn univariate(spec: UnivariateSpec) -> Self {
        Self {
            operation: OperationRef::builtin("chart.univariate"),
            parameters: StatParameters::Univariate(spec),
        }
    }
    /// Automatic equal-width bins.
    pub fn auto_bin(spec: AutoBinSpec) -> Self {
        Self {
            operation: OperationRef::builtin("chart.auto_bin"),
            parameters: StatParameters::AutoBin(spec),
        }
    }
    /// Required-input count.
    pub fn count(spec: CountSpec) -> Self {
        Self {
            operation: OperationRef::builtin("chart.count"),
            parameters: StatParameters::Count(spec),
        }
    }
    /// Exact finite grouped summaries.
    pub fn summary(spec: SummarySpec) -> Self {
        Self {
            operation: OperationRef::builtin("chart.summary"),
            parameters: StatParameters::Summary(spec),
        }
    }
    /// Intercept least-squares fit.
    pub fn ols(spec: OlsSpec) -> Self {
        Self {
            operation: OperationRef::builtin("chart.ols"),
            parameters: StatParameters::Ols(spec),
        }
    }
    /// Explicit bins support exact unfiltered full-source chunk updates. Filtered/faceted or
    /// transformed-source populations, automatic bins and other statistics use exact batch fallbacks.
    pub fn incremental_capabilities(&self) -> IncrementalCapabilities {
        IncrementalCapabilities {
            append: matches!(&self.parameters, StatParameters::Bin(s) if s.ggplot.as_ref().is_none_or(|o| o.weight.is_none())),
            window: matches!(&self.parameters, StatParameters::Bin(s) if s.ggplot.as_ref().is_none_or(|o| o.weight.is_none())),
            correction: matches!(&self.parameters, StatParameters::Bin(s) if s.ggplot.as_ref().is_none_or(|o| o.weight.is_none())),
            full_recompute: true,
        }
    }
}
/// Exact supported update paths; false means the exact batch fallback is required.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncrementalCapabilities {
    /// Exact specialized append path.
    pub append: bool,
    /// Exact specialized rolling-window path.
    pub window: bool,
    /// Exact specialized correction/removal path.
    pub correction: bool,
    /// Exact full population evaluation.
    pub full_recompute: bool,
}
/// Stage-safe generated field identity; quantile indexes address authored probabilities.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum StatField {
    /// Generated cell height in statistical calculation space.
    Height,
    /// Contour threshold or ellipse level.
    Level,
    /// Isoband lower threshold.
    LevelLow,
    /// Isoband upper threshold.
    LevelHigh,
    /// Isoband midpoint between its two thresholds.
    LevelMid,
    /// Contour level divided by the largest emitted level (upper edge for bands).
    NormalizedLevel,
    /// Stable generated path piece within its level.
    Piece,
    /// Ring identity within a filled polygon piece.
    Subgroup,

    /// Mean-estimation standard error in model calculation or link space.
    StandardError,
    /// Requested quantile probability for a quantile regression curve.
    QuantileProbability,

    /// Field in a registered custom generated schema; never a source FieldId.
    Custom(String),
    /// Stable group label, projected through the generated schema catalog.
    Group,
    /// Membership count.
    Count,
    /// Signed sum of weights, independently of exact membership count.
    WeightedCount,
    /// Spatial weighted count divided by its group maximum.
    NormalizedCount,
    /// Signed count divided by sum of absolute counts within the group.
    Proportion,
    /// Summary lower endpoint.
    Lower,
    /// Summary upper endpoint.
    Upper,
    /// Reference interval or bar width.
    Width,
    /// Boxplot median.
    Middle,
    /// Observed lower nonoutlier whisker.
    WhiskerLower,
    /// Observed upper nonoutlier whisker.
    WhiskerUpper,
    /// Lower median notch endpoint.
    NotchLower,
    /// Upper median notch endpoint.
    NotchUpper,
    /// Square root of the group weight used by variable-width boxes.
    RelativeWidth,
    /// Normalized width of one violin density row.
    ViolinWidth,
    /// Requested violin quantile probability; missing on ordinary density rows.
    QuantileFlag,
    /// Data-space diameter of one dot bin.
    BinWidth,
    /// Dot center offset before physical spacing is applied.
    StackPosition,
    /// Probability density per input unit.
    Density,
    /// Density divided by its group maximum.
    ScaledDensity,
    /// Density multiplied by total observation weight.
    WeightedDensity,
    /// Density multiplied by unweighted group size; distinct from exact membership count.
    DensityCount,
    /// Empirical cumulative distribution value.
    Ecdf,
    /// One for artificial zero-height alignment padding, zero otherwise.
    AlignPadding,
    /// Minimum finite value.
    Min,
    /// Maximum finite value.
    Max,
    /// Stable arithmetic mean.
    Mean,
    /// Stable sum.
    Sum,
    /// Quantile by authored probability index.
    Quantile(usize),
    /// Fitted endpoint predictor.
    X,
    /// Fitted endpoint response.
    Y,
    /// Model intercept.
    Intercept,
    /// Model slope.
    Slope,
}
/// Explicit field kind, nullability and calculation-space metadata.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct StatColumn {
    /// Typed accessor.
    pub field: StatField,
    /// Physical value kind.
    pub kind: GeneratedKind,
    /// Whether missing outputs are possible.
    pub nullable: bool,
    /// Calculation space of this field, already applied by the stat.
    pub space: ValueSpace,
}
/// One finite or missing computed numeric output.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct StatValue {
    /// Typed field.
    pub field: StatField,
    /// Missing never silently substitutes zero.
    pub value: Option<f64>,
}
/// One observed boxplot outlier, retaining its own source provenance.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct StatOutlier {
    /// Value in the statistic's dependent calculation space.
    pub value: f64,
    /// Original source observation, distinct from the box aggregate.
    pub target: Target,
}
/// Typed summary/model output, distinct from source and bin rows.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct StatisticalRow {
    /// Boxplot observations emitted after positioning the summary once.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outliers: Vec<StatOutlier>,
    /// Proven identical source aesthetic values retained by a partitioning statistic.
    /// This is independent of the original group used for normalization.
    #[serde(skip_serializing_if = "GroupValue::is_all")]
    pub retained: GroupValue,
    /// Exact evaluated numeric aesthetic inputs retained by joint-position statistics.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub retained_numeric: Vec<(Numeric, Option<f64>)>,

    /// Stable partition identity.
    pub group: GroupValue,
    /// Exact usable count; kept as a decimal string on portable surfaces.
    #[serde(with = "crate::portable::unsigned")]
    pub count: u64,
    /// Numeric generated values (count is stored separately).
    pub values: Vec<StatValue>,
    /// Exact filtered usable model/aggregate population, in source-key order.
    pub members: Arc<[RowKey]>,
    /// Aggregate or model provenance, never a representative source row.
    pub target: Target,
}
impl StatisticalRow {
    /// Checked generated access. Count narrowing beyond 2^53 is never silent.
    pub fn value(&self, field: &StatField) -> Option<f64> {
        if *field == StatField::Count {
            (self.count <= 1_u64 << 53).then_some(self.count as f64)
        } else {
            self.values
                .iter()
                .find(|v| &v.field == field)
                .and_then(|v| v.value)
        }
    }
}
/// Explicit generated field or finite literal.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum StatNumeric {
    /// Expression over generated fields after scale back-transformation.
    Expression(super::Expression<StatField>),
    /// Typed generated field.
    Field(StatField),
    /// Constant encoding.
    Literal(f64),
}
impl From<super::Expression<StatField>> for StatNumeric {
    fn from(value: super::Expression<StatField>) -> Self {
        Self::Expression(value)
    }
}
impl From<f64> for StatNumeric {
    fn from(value: f64) -> Self {
        Self::Literal(value)
    }
}
impl From<StatField> for StatNumeric {
    fn from(v: StatField) -> Self {
        Self::Field(v)
    }
}
/// Generated encodings never inherit source callbacks or field IDs.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StatAes {
    /// First x coordinate.
    pub x: StatNumeric,
    /// First y coordinate.
    pub y: StatNumeric,
    /// Optional second x endpoint.
    pub x2: Option<StatNumeric>,
    /// Optional second y endpoint/baseline.
    pub y2: Option<StatNumeric>,
    /// Optional destination size.
    pub size: Option<StatNumeric>,
}
impl StatAes {
    /// Bind generated x/y coordinates. Source IDs cannot receive generated outputs.
    ///
    /// ```compile_fail
    /// use chart_core::{FieldId, grammar::StatAes};
    /// let generated = StatAes::new(FieldId::new(1), FieldId::new(2));
    /// ```
    pub fn new(x: impl Into<StatNumeric>, y: impl Into<StatNumeric>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            x2: None,
            y2: None,
            size: None,
        }
    }
}
impl Layer {
    /// Author a count/summary/model layer with explicitly typed generated encodings.
    pub fn statistical(
        id: crate::LayerId,
        data: impl Into<DataRef>,
        statistic: Statistic,
        geom: Geom,
        mappings: StatAes,
    ) -> Self {
        Self {
            statistic,
            mappings: Mappings::Statistical(mappings),
            inherit: false,
            ..Self::new(id, data, geom, SourceAes::new())
        }
    }
    /// Fit recipe: data-space, whole filtered population, two straight model endpoints.
    pub fn fit(id: crate::LayerId, data: impl Into<DataRef>, spec: OlsSpec) -> Self {
        Self::statistical(
            id,
            data,
            Statistic::ols(spec),
            Geom::line(),
            StatAes::new(StatField::X, StatField::Y),
        )
    }
}
/// Separate sign totals, with caller-declared stable series order.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StackSpec {
    /// Series order; every eligible group must occur exactly once.
    pub order: Vec<GroupValue>,
    /// Normalize each present sign side to unit magnitude.
    pub normalize: bool,
}
/// Reference stack position over a tidy group-by-sample table.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ShapeStackSpec {
    /// Stable series catalog, including groups absent from an individual sample.
    pub groups: Vec<GroupValue>,
    /// Rank policy; explicit permutations address the group catalog.
    pub order: crate::shape::StackOrder,
    /// Reference baseline and normalization policy.
    pub offset: crate::shape::StackOffset,
    /// Missing-cell treatment, without inventing source targets.
    pub missing: crate::shape::StackMissing,
}
/// Horizontal categorical band-relative slot positioning.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DodgeSpec {
    /// Fixed slot order, including absent groups.
    pub order: Vec<GroupValue>,
    /// Fraction of the category band occupied by all slots, in (0,1].
    pub width: f64,
}
/// Explicit jitter units; no implicit destination/data conversion.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum JitterUnits {
    /// Source/calculation units; affects domain contributions.
    Data,
    /// Destination logical units/points; does not affect domains.
    Display,
}
/// Reorder-invariant jitter keyed by stable target and group identity.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct JitterSpec {
    /// Exact explicit seed.
    #[serde(with = "crate::portable::unsigned")]
    pub seed: u64,
    /// Nonnegative horizontal half-width.
    pub x: f64,
    /// Nonnegative vertical half-width.
    pub y: f64,
    /// Displacement units.
    pub units: JitterUnits,
}
