//! Portable model terms and controls; numerical implementation remains shared core.
use super::*;

/// A model-matrix term evaluated over the predictor, independent of source field IDs.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ModelTerm {
    /// Predictor raised to an integer power; zero is the intercept.
    Power(u8),
    /// Typed numeric expression evaluated at each predictor.
    Expression(Expression<FunctionArgument>),
}
/// Canonical generalized-linear response family and link.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ModelFamily {
    /// Gaussian response with identity link.
    Gaussian,
    /// Binomial response with logit link.
    Binomial,
    /// Poisson response with log link.
    Poisson,
}
/// Local regression evaluation surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LoessSurface {
    /// Reference interpolated local-polynomial surface.
    Interpolate,
    /// Evaluate local regression at every requested coordinate.
    Direct,
}
/// Local-regression response weighting family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LoessFamily {
    /// Squared-error local regression.
    Gaussian,
    /// Robust symmetric residual reweighting.
    Symmetric,
}
/// Built-in fitting method, or an exactly registered arbitrary model.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ModelMethod {
    /// LOESS below 1,000 rows in the largest panel group, Gaussian GAM otherwise.
    Auto,
    /// Weighted least squares with explicit model terms.
    Linear,
    /// Canonical-family iteratively reweighted least squares.
    Glm {
        /// Response family and link.
        family: ModelFamily,
        /// Relative deviance convergence tolerance.
        epsilon: f64,
        /// Maximum fitting iterations.
        iterations: usize,
    },
    /// Univariate local-polynomial regression.
    Loess {
        /// Fraction of observations in each neighborhood.
        span: f64,
        /// Local polynomial degree, zero through two.
        degree: usize,
        /// Interpolation cell parameter.
        cell: f64,
        /// Direct or interpolated predictions.
        surface: LoessSurface,
        /// Gaussian or robust symmetric fitting.
        family: LoessFamily,
        /// Maximum robust reweighting iterations.
        iterations: usize,
        /// Normalize predictor coordinates.
        normalize: bool,
        /// Exact rather than approximate residual statistics.
        exact_statistics: bool,
        /// Approximate rather than exact influence trace.
        approximate_trace: bool,
    },
    /// Gaussian shrinkage cubic regression spline with REML smoothing selection.
    Gam {
        /// Cubic regression basis dimension.
        basis_dimension: usize,
        /// Optional strictly increasing authored knot sequence.
        knots: Option<Vec<f64>>,
        /// Maximum smoothing-selection iterations.
        iterations: usize,
        /// Smoothing-selection tolerance.
        tolerance: f64,
    },
    /// Weighted check-loss regression using the shared bounded BR tableau.
    Quantile {
        /// Probabilities in authored output order.
        probabilities: Vec<f64>,
        /// Maximum tableau iterations per probability.
        iterations: usize,
    },
    /// Trusted, installed model operation; executable objects never enter the wire.
    Registered {
        /// Exact registered identity and version.
        operation: OperationRef,
        /// Bounded declarative model parameters.
        parameters: serde_json::Value,
    },
}
/// Model options independent of source mappings and statistical population.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelOptions {
    /// Fitting method; automatic by default.
    pub method: ModelMethod,
    /// Explicit design terms for linear/GLM/quantile models; absent uses intercept and x.
    pub terms: Option<Vec<ModelTerm>>,
    /// Prediction count when no explicit grid is supplied.
    pub n: usize,
    /// Explicit predictor coordinates in calculation space.
    pub xseq: Option<Vec<f64>>,
    /// Predict across the trained scale range rather than the group sample range.
    pub full_range: bool,
    /// Include mean-estimation uncertainty when supported by the method.
    pub se: bool,
    /// Confidence probability, strictly between zero and one.
    pub level: f64,
}
impl Default for ModelOptions {
    fn default() -> Self {
        Self {
            method: ModelMethod::Auto,
            terms: None,
            n: 80,
            xseq: None,
            full_range: false,
            se: true,
            level: 0.95,
        }
    }
}
/// Model source inputs and resolved cross-panel policies.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSpec {
    /// Predictor input.
    pub x: Numeric,
    /// Response input.
    pub y: Numeric,
    /// Optional nonnegative observation weights.
    pub weight: Option<Numeric>,
    /// Exact statistical group identities.
    pub grouping: Grouping,
    /// Predictor calculation space.
    pub x_space: StatSpace,
    /// Response calculation space.
    pub y_space: StatSpace,
    /// Shared model and prediction controls.
    pub options: ModelOptions,
    /// Source aesthetics retained only when constant within a model group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_fields: Vec<crate::FieldId>,
    /// Evaluated numeric aesthetics retained only when constant within a model group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_numeric: Vec<Numeric>,
    /// Compiler-resolved predictor range before coordinate zoom.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub training_range: Option<[f64; 2]>,
    /// Compiler-resolved largest group across the layer's panels for automatic selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub largest_group: Option<usize>,
}
impl ModelSpec {
    pub(crate) fn numerics(&self) -> impl Iterator<Item = &Numeric> {
        [&self.x, &self.y].into_iter().chain(self.weight.iter()).chain(self.retained_numeric.iter())
    }
    pub(crate) fn numerics_mut(&mut self) -> impl Iterator<Item = &mut Numeric> {
        [&mut self.x, &mut self.y].into_iter().chain(self.weight.iter_mut()).chain(self.retained_numeric.iter_mut())
    }
}
