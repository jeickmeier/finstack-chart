//! Typed built-in analytical statistics; portable functions remain data descriptors.
use super::*;

/// Normalization across violin groups within a panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ViolinScale {
    /// Equal integrated areas before trimming.
    #[default]
    Area,
    /// Area proportional to group membership.
    Count,
    /// Equal maximum widths.
    Width,
}
/// One-dimensional dot bin construction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DotBinMethod {
    /// Successive sorted bins centered on each occupied range.
    #[default]
    DotDensity,
    /// Histogram edges and closure from the shared bin engine.
    HistoDot,
}
/// Distribution statistic, independent of its downstream geometry.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DistributionKind {
    /// Observed whiskers, hinges, notches and exact outlier membership.
    Boxplot {
        /// Nonnegative IQR fence multiplier.
        coefficient: f64,
        /// Hyndman-Fan quantile rule, one through nine.
        quantile_type: u8,
    },
    /// Reference FFT kernel density estimate.
    Density {
        /// Evaluate only over each group's observed range instead of the trained scale range.
        #[serde(default)]
        trim: bool,
        /// Shared estimator, kernel, bandwidth and grid controls.
        controls: DensityControls,
    },
    /// Kernel density with panel-wide violin normalization.
    Violin {
        /// Shared estimator controls.
        controls: DensityControls,
        /// Restrict evaluation to each group's observed range.
        trim: bool,
        /// Panel-wide width normalization.
        scale: ViolinScale,
        /// Probabilities inserted as explicit quantile rows.
        quantiles: Vec<f64>,
        /// Remove groups containing fewer than two observations.
        drop: bool,
    },
    /// Integer-weight dot counts retaining exact contributing rows.
    Dotplot {
        /// Axis containing the sample being binned.
        bin_axis: DotAxis,
        /// Binning algorithm.
        method: DotBinMethod,
        /// Positive data-space width; absent selects range divided by thirty.
        bin_width: Option<f64>,
        /// Share bin locations across groups within a panel.
        bin_positions_all: bool,
        /// Histogram boundary closure.
        closed: BinClosure,
        /// Optional histogram boundary anchor.
        origin: Option<f64>,
    },
}
/// Inputs and population policy for a distribution statistic.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DistributionSpec {
    /// Source aesthetics retained only when constant within a statistical group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_fields: Vec<crate::FieldId>,
    /// Evaluated numeric aesthetic inputs retained only when constant within a group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_numeric: Vec<Numeric>,
    /// Resolved panel/common sample range supplied by scale training.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub training_range: Option<[f64; 2]>,
    /// Numeric sample input.
    pub input: Numeric,
    /// Independent position for grouped boxplots, violins or dotplots.
    pub position: Option<Numeric>,
    /// Optional observation weights.
    pub weight: Option<Numeric>,
    /// Exact group identities.
    pub grouping: Grouping,
    /// Declared sample calculation space.
    pub space: StatSpace,
    /// Independent position calculation space.
    pub position_space: StatSpace,
    /// Optional data-space width, resolved before positions.
    pub width: Option<f64>,
    /// Actual built-in operation.
    pub kind: DistributionKind,
}
impl DistributionSpec {
    pub(crate) fn sample_axis(&self) -> usize {
        match self.kind {
            DistributionKind::Density { .. }
            | DistributionKind::Dotplot {
                bin_axis: DotAxis::X,
                ..
            } => 0,
            _ => 1,
        }
    }
    pub(crate) fn numerics(&self) -> impl Iterator<Item = &Numeric> {
        std::iter::once(&self.input)
            .chain(self.position.iter())
            .chain(self.weight.iter())
            .chain(self.retained_numeric.iter())
    }
    pub(crate) fn numerics_mut(&mut self) -> impl Iterator<Item = &mut Numeric> {
        std::iter::once(&mut self.input)
            .chain(self.position.iter_mut())
            .chain(self.weight.iter_mut())
            .chain(self.retained_numeric.iter_mut())
    }
}
/// Argument of a portable scalar expression, unrelated to a source field identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FunctionArgument {
    /// Current function coordinate or quantile probability.
    Value,
}
/// Pure numeric function evaluated by the shared analytic owner.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AnalyticFunction {
    /// Stage-typed arithmetic over the supplied evaluation coordinates.
    Expression(Expression<FunctionArgument>),
    /// Normal quantile, sharing the existing portable probability kernel.
    NormalQuantile {
        /// Location parameter.
        mean: f64,
        /// Positive scale parameter.
        sd: f64,
    },
    /// Uniform quantile.
    UniformQuantile {
        /// Lower distribution endpoint.
        min: f64,
        /// Upper distribution endpoint.
        max: f64,
    },
    /// Logistic quantile.
    LogisticQuantile {
        /// Location parameter.
        location: f64,
        /// Positive scale parameter.
        scale: f64,
    },
    /// Explicitly installed pure function; no executable host data is serialized.
    Registered {
        /// Exact registered identity and version.
        operation: OperationRef,
        /// Bounded declarative parameters.
        parameters: serde_json::Value,
    },
}
impl Default for AnalyticFunction {
    fn default() -> Self {
        Self::NormalQuantile { mean: 0., sd: 1. }
    }
}
/// Path connection policy; named steps retain the original statistic rows.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Connection {
    /// Horizontal then vertical.
    Hv,
    /// Vertical then horizontal.
    Vh,
    /// Step at each segment midpoint.
    Mid,
    /// Relative x/y coordinates interpolated into generated statistic rows.
    Matrix(Vec<[f64; 2]>),
}
/// Univariate analysis and source-row helpers.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum UnivariateKind {
    /// Weighted empirical distribution function.
    Ecdf {
        /// Optional evenly spaced evaluation count.
        n: Option<usize>,
        /// Retain the negative/positive infinite endpoint sentinels.
        pad: bool,
    },
    /// Sample versus theoretical quantiles or their fitted reference line.
    Qq {
        /// Distribution quantile function.
        distribution: AnalyticFunction,
        /// Optional probability for each sorted sample; absent uses reference plotting positions.
        quantiles: Option<Vec<f64>>,
        /// Emit two line endpoints instead of sample quantile pairs.
        line: bool,
        /// Probabilities defining the reference line.
        probabilities: [f64; 2],
        /// Extend the reference line over the panel's trained range.
        full_range: bool,
    },
    /// Sample a function on an evenly spaced transformed-axis grid.
    Function {
        /// Pure function evaluated in inverse-transformed coordinates.
        function: AnalyticFunction,
        /// Output grid count; zero produces no samples.
        n: usize,
        /// Optional evaluation range in the current x calculation space.
        range: Option<[f64; 2]>,
    },
    /// Retain the first source row for each distinct mapped-aesthetic tuple.
    Unique,
    /// Interpolate consecutive sorted observations with relative coordinates.
    Connect {
        /// Each pair supplies the relative x and y fraction along a segment.
        connection: Connection,
    },
    /// Align groups to common x coordinates, crossings and endpoint padding.
    Align,
}
/// Shared source inputs for a one-dimensional analytical operation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnivariateSpec {
    /// Function has no mapped sample coordinate; use trained axes or the default unit range.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default_input: bool,
    /// Resolved common x evaluation range for function sampling or full-range QQ lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub training_range: Option<[f64; 2]>,
    /// Resolved function-output projection, evaluated after the scalar function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_scale: Option<ScaleProjection>,
    /// Primary sample/position input.
    pub input: Numeric,
    /// Dependent input for connection, uniqueness and alignment.
    pub second: Option<Numeric>,
    /// Optional empirical-distribution weights.
    pub weight: Option<Numeric>,
    /// Stable population partition.
    pub grouping: Grouping,
    /// Primary calculation space.
    pub space: StatSpace,
    /// Secondary calculation space.
    pub second_space: StatSpace,
    /// Additional mapped source fields participating in uniqueness.
    pub retained_fields: Vec<crate::FieldId>,
    /// Evaluated mapped numeric expressions participating in uniqueness.
    pub retained_numeric: Vec<Numeric>,
    /// Actual built-in helper.
    pub kind: UnivariateKind,
}
impl UnivariateSpec {
    pub(crate) fn sample_axis(&self) -> usize {
        usize::from(matches!(self.kind, UnivariateKind::Qq { .. }))
    }
    pub(crate) fn numerics(&self) -> impl Iterator<Item = &Numeric> {
        std::iter::once(&self.input)
            .chain(self.second.iter())
            .chain(self.weight.iter())
            .chain(self.retained_numeric.iter())
    }
    pub(crate) fn numerics_mut(&mut self) -> impl Iterator<Item = &mut Numeric> {
        std::iter::once(&mut self.input)
            .chain(self.second.iter_mut())
            .chain(self.weight.iter_mut())
            .chain(self.retained_numeric.iter_mut())
    }
}
