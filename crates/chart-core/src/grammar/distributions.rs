//! Shared distribution kernels; input indices refer to the caller's group slice.
mod bandwidth;
pub(crate) use bandwidth::select as select_bandwidth;
mod boxplot;
mod density;
mod dotplot;
mod quantiles;
mod violin;
pub(crate) use boxplot::boxplot;
pub(crate) use density::{DensityEstimate, DensityPoint, density};
pub(crate) use dotplot::{DotBin, dotdensity, histodot};
pub(crate) use quantiles::quantile;
pub(crate) use violin::{normalize_violin, violin};

/// Variance-standardized kernels used by the reference density estimator.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DensityKernel {
    /// Normal kernel.
    #[default]
    Gaussian,
    /// Parabolic compact kernel.
    Epanechnikov,
    /// Uniform compact kernel.
    Rectangular,
    /// Triangular compact kernel.
    Triangular,
    /// Quartic compact kernel.
    Biweight,
    /// Raised cosine kernel.
    Cosine,
    /// Cosine kernel with optimal support.
    Optcosine,
}
/// Bandwidth in calculation-space units or a pinned automatic selector.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bandwidth {
    /// Explicit positive bandwidth.
    Fixed(f64),
    /// Reference robust normal rule with zero-spread fallback.
    #[default]
    Nrd0,
    /// Scott normal-reference rule.
    Nrd,
    /// Unbiased cross-validation.
    Ucv,
    /// Biased cross-validation.
    Bcv,
    /// Sheather-Jones solve-the-equation selector.
    SjSte,
    /// Sheather-Jones direct plug-in selector.
    SjDpi,
}
/// Shared density and violin numerical controls.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DensityControls {
    /// Explicit or automatically selected bandwidth.
    pub bandwidth: Bandwidth,
    /// Positive bandwidth multiplier.
    pub adjust: f64,
    /// Variance-standardized smoothing kernel.
    pub kernel: DensityKernel,
    /// Requested output grid length.
    pub n: usize,
    /// Optional finite lower support bound, with reflection.
    pub lower: Option<f64>,
    /// Optional finite upper support bound, with reflection.
    pub upper: Option<f64>,
}
impl Default for DensityControls {
    fn default() -> Self {
        Self {
            bandwidth: Bandwidth::Nrd0,
            adjust: 1.,
            kernel: DensityKernel::Gaussian,
            n: 512,
            lower: None,
            upper: None,
        }
    }
}
fn invalid(message: &str) -> crate::Diagnostic {
    super::error(crate::DiagnosticCode::NumericalDomain, message)
}
