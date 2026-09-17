//! Portable two-dimensional statistics, independent of rendering backends.
use super::{Grouping, Numeric, StatSpace, SummaryBins, SummaryFunction};

/// Contour threshold selection; explicit breaks take precedence over automatic controls.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContourLevels {
    /// Explicit ordered finite thresholds.
    pub breaks: Option<Vec<f64>>,
    /// Requested automatic threshold count.
    pub bins: Option<usize>,
    /// Positive threshold interval.
    pub binwidth: Option<f64>,
}
/// Density quantity supplied to contour extraction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DensityContour {
    /// Probability density.
    #[default]
    Density,
    /// Density divided by its group maximum.
    Normalized,
    /// Density multiplied by unweighted observation count.
    Count,
}
/// Reference ellipse covariance estimator and radius interpretation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EllipseKind {
    /// Iteratively reweighted multivariate Student covariance, five degrees of freedom.
    #[default]
    T,
    /// Weighted unbiased covariance.
    Normal,
    /// Circle with radius equal to the authored level.
    Euclidean,
}
/// A built-in spatial statistical operation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SpatialKind {
    /// Rectangular counts or unweighted response summaries using shared reference bins.
    Rectangular {
        /// Independent x and y bin controls.
        axes: [SummaryBins; 2],
        /// Absent means weighted counts; present summarizes the response input.
        summary: Option<SummaryFunction>,
        /// Remove unoccupied cross-product cells.
        drop: bool,
    },
    /// Hexagonal counts or response summaries with reference hexbin membership.
    Hexagonal {
        /// Data-space width and height; absent uses trained ranges divided by bins.
        binwidth: Option<[f64; 2]>,
        /// Independent automatic bin counts.
        bins: [usize; 2],
        /// Absent means weighted counts; present summarizes response values.
        summary: Option<SummaryFunction>,
        /// Remove missing summary outputs.
        drop: bool,
    },
    /// Product Gaussian KDE; source weights do not enter the reference estimator.
    Density {
        /// Reference MASS bandwidths, divided by four inside the Gaussian kernel.
        bandwidth: Option<[f64; 2]>,
        /// Multipliers used only when bandwidth is selected automatically.
        adjust: [f64; 2],
        /// Bounded grid dimensions, with x varying fastest in output.
        n: [usize; 2],
        /// Absent emits grid values; present extracts contours from the selected quantity.
        contour: Option<ContourLevels>,
        /// Density quantity supplied to optional contour extraction.
        contour_var: DensityContour,
        /// Emit filled bands instead of contour lines.
        filled: bool,
    },
    /// Contours or isobands from an authored rectangular x/y/response grid.
    Contour {
        /// Shared threshold controls.
        levels: ContourLevels,
        /// Emit filled bands instead of contour lines.
        filled: bool,
    },
    /// Weighted covariance ellipse or Euclidean circle.
    Ellipse {
        /// Covariance/radius method.
        kind: EllipseKind,
        /// Confidence probability for normal/t, direct radius for Euclidean.
        level: f64,
        /// Number of segments; output repeats the first endpoint.
        segments: usize,
    },
}
/// Inputs and grouping for a two-dimensional statistic.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialSpec {
    /// Horizontal source coordinate.
    pub x: Numeric,
    /// Vertical source coordinate.
    pub y: Numeric,
    /// Response values for summaries and authored contours.
    pub z: Option<Numeric>,
    /// Counts/ellipse weights; ignored by reference KDE and response summaries.
    pub weight: Option<Numeric>,
    /// Exact group identities.
    pub grouping: Grouping,
    /// Horizontal calculation space.
    pub x_space: StatSpace,
    /// Vertical calculation space.
    pub y_space: StatSpace,
    /// Response calculation space.
    pub z_space: StatSpace,
    /// Resolved panel/common ranges supplied by shared scale training.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub training_ranges: Option<[[f64; 2]; 2]>,
    /// Source aesthetics retained only when constant within a statistical group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_fields: Vec<crate::FieldId>,
    /// Evaluated numeric aesthetics retained only when constant within a group.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_numeric: Vec<Numeric>,
    /// Actual statistical operation.
    pub kind: SpatialKind,
}
impl SpatialSpec {
    pub(crate) fn numerics(&self) -> impl Iterator<Item = &Numeric> {
        [&self.x, &self.y]
            .into_iter()
            .chain(self.z.iter())
            .chain(self.weight.iter())
            .chain(self.retained_numeric.iter())
    }
    pub(crate) fn numerics_mut(&mut self) -> impl Iterator<Item = &mut Numeric> {
        [&mut self.x, &mut self.y]
            .into_iter()
            .chain(self.z.iter_mut())
            .chain(self.weight.iter_mut())
            .chain(self.retained_numeric.iter_mut())
    }
}
