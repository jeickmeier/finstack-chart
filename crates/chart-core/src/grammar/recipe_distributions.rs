//! Distribution recipe controls and geometry over shared generated/source schemas.
use super::{IntervalPoint, IntervalStroke, LineType};
/// Boundary strokes drawn around a filled density area.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AreaOutline {
    /// Draw only the estimated curve.
    #[default]
    Upper,
    /// Draw only the baseline.
    Lower,
    /// Draw curve and baseline as separate open paths.
    Both,
    /// Draw the entire closed polygon, including end segments.
    Full,
}
/// Density area uses common band runs with independently selected outline boundaries.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DensityRecipe {
    /// Visible boundary selection; upper by default.
    pub outline: AreaOutline,
}
/// Independent boxplot components; statistical estimators are configured on the stat.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BoxplotRecipe {
    /// Full box width; absent uses 0.9 times independent-axis resolution.
    pub width: Option<f64>,
    /// Draw notched sides around the median confidence interval.
    pub notch: bool,
    /// Fraction of the full width at the median notch.
    pub notch_width: f64,
    /// Fraction of the full width used by whisker-end staples; zero hides staples.
    pub staple_width: f64,
    /// Scale box width by the group's relative-width statistic.
    pub variable_width: bool,
    /// Retain and draw statistical outliers; false removes their domain contribution.
    pub outliers: bool,
    /// Bounded precomputed outliers keyed to source summary rows.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_outliers: Vec<SourceBoxOutliers>,
    /// Independent whisker styling.
    pub whisker: IntervalStroke,
    /// Independent whisker-end staple styling.
    pub staple: IntervalStroke,
    /// Independent median styling.
    pub median: IntervalStroke,
    /// Independent box styling.
    pub box_style: IntervalStroke,
    /// Independent outlier point styling.
    pub outlier: IntervalPoint,
    /// Independent outlier colour; absent inherits mapped row colour.
    pub outlier_color: Option<crate::color::Paint>,
    /// Independent outlier opacity; absent inherits mapped row opacity.
    pub outlier_alpha: Option<f64>,
    /// Median linewidth multiplier, including the supported legacy fatten control.
    pub fatten: f64,
}
impl Default for BoxplotRecipe {
    fn default() -> Self {
        Self {
            width: None,
            notch: false,
            notch_width: 0.5,
            staple_width: 0.,
            variable_width: false,
            outliers: true,
            source_outliers: vec![],
            whisker: Default::default(),
            staple: Default::default(),
            median: Default::default(),
            box_style: Default::default(),
            outlier: IntervalPoint {
                stroke: Some(0.5),
                ..Default::default()
            },
            outlier_color: None,
            outlier_alpha: None,
            fatten: 2.,
        }
    }
}
/// Violin polygon and optional quantile strokes; density/trim/normalization live on the stat.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ViolinRecipe {
    /// Full maximum width; absent uses 0.9 times independent-axis resolution.
    pub width: Option<f64>,
    /// Quantile-row stroke controls. The default blank linetype hides quantiles.
    pub quantile: IntervalStroke,
}
impl Default for ViolinRecipe {
    fn default() -> Self {
        Self {
            width: None,
            quantile: IntervalStroke {
                line_type: Some(LineType::Blank),
                ..Default::default()
            },
        }
    }
}
/// Bin coordinate of a dot plot, before any layer orientation transpose.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DotAxis {
    /// Bin x and stack vertically.
    #[default]
    X,
    /// Bin y and stack horizontally.
    Y,
}
/// Dot stack placement relative to its baseline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DotStack {
    /// Positive direction with half-dot baseline offset.
    #[default]
    Up,
    /// Negative direction with half-dot baseline offset.
    Down,
    /// Center the full stack symmetrically around the baseline.
    Center,
    /// Center using whole-dot shifts.
    CenterWhole,
}
/// Circle-stack geometry; bin membership is supplied by the dotplot statistic.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DotplotRecipe {
    /// Data coordinate binned by the statistic.
    pub bin_axis: DotAxis,
    /// Dot stack direction.
    pub stack: DotStack,
    /// Center-spacing multiplier relative to the dot diameter.
    pub stack_ratio: f64,
    /// Dot diameter multiplier relative to the bin width.
    pub dot_size: f64,
    /// Combine stacks from different groups at the same bin and baseline.
    pub stack_groups: bool,
    /// Full independent group width for y-bin domain training.
    pub width: Option<f64>,
}
impl Default for DotplotRecipe {
    fn default() -> Self {
        Self {
            bin_axis: DotAxis::X,
            stack: DotStack::Up,
            stack_ratio: 1.,
            dot_size: 1.,
            stack_groups: false,
            width: None,
        }
    }
}

/// Precomputed source outliers belonging to one stable summary row.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceBoxOutliers {
    /// Source key in this layer's dataset.
    pub row: crate::RowKey,
    /// Dependent-axis values before the positional scale transform.
    pub values: Vec<f64>,
}
/// Deferred distribution marks; no destination object is retained in core grammar.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum PreparedDistribution {
    /// Circle with diameter derived from projected data-space bin width.
    Dot {
        /// Positioned baseline.
        center: crate::Point,
        /// Opposite bin endpoints, used to determine physical diameter.
        bin: [crate::Point; 2],
        /// Dot stack index after direction and centering rules.
        stack_position: f64,
        /// Spacing relative to diameter.
        stack_ratio: f64,
        /// Final baseline offset relative to diameter.
        stack_offset: f64,
        /// Diameter relative to bin size.
        dot_size: f64,
        /// Stack horizontally instead of vertically.
        horizontal: bool,
    },
    /// Reference point glyph belonging to the original outlier source target.
    Outlier {
        /// Positioned outlier coordinate.
        center: crate::Point,
        /// Reference size in millimeters.
        size: f64,
        /// Reference point stroke parameter in millimeters.
        stroke: f64,
        /// Reference shape code zero through 25.
        shape: u8,
    },
}
impl PreparedDistribution {
    pub(crate) fn points(&self) -> Vec<crate::Point> {
        match self {
            Self::Dot { center, bin, .. } => vec![*center, bin[0], bin[1]],
            Self::Outlier { center, .. } => vec![*center],
        }
    }
    pub(crate) fn transpose(&mut self) -> crate::ChartResult<()> {
        let flip = |p: crate::Point| crate::Point::new(p.y(), p.x());
        match self {
            Self::Dot {
                center,
                bin,
                horizontal,
                ..
            } => {
                *center = flip(*center)?;
                for p in bin {
                    *p = flip(*p)?;
                }
                *horizontal = !*horizontal;
            }
            Self::Outlier { center, .. } => *center = flip(*center)?,
        }
        Ok(())
    }
}
mod boxplot;
mod common;
mod dotplot;
mod violin;

pub(super) fn validate(recipe: &super::BuiltinRecipe) -> crate::ChartResult<()> {
    let valid = match recipe {
        super::BuiltinRecipe::Boxplot(s) => {
            let mut keys = std::collections::BTreeSet::new();
            s.width.is_none_or(|v| v.is_finite() && v >= 0.)
                && s.notch_width.is_finite()
                && (0. ..=1.).contains(&s.notch_width)
                && s.staple_width.is_finite()
                && s.staple_width >= 0.
                && s.fatten.is_finite()
                && s.fatten >= 0.
                && [&s.whisker, &s.staple, &s.median, &s.box_style]
                    .into_iter()
                    .all(common::stroke_valid)
                && s.outlier.size.is_none_or(|v| v.is_finite() && v >= 0.)
                && s.outlier.stroke.is_none_or(|v| v.is_finite() && v >= 0.)
                && s.outlier.shape.is_none_or(|v| v <= 25)
                && s.outlier_alpha
                    .is_none_or(|v| v.is_finite() && (0. ..=1.).contains(&v))
                && s.source_outliers
                    .iter()
                    .all(|v| keys.insert(v.row) && v.values.iter().all(|v| v.is_finite()))
        }
        super::BuiltinRecipe::Violin(s) => {
            s.width.is_none_or(|v| v.is_finite() && v >= 0.) && common::stroke_valid(&s.quantile)
        }
        super::BuiltinRecipe::Dotplot(s) => {
            s.width.is_none_or(|v| v.is_finite() && v >= 0.)
                && s.stack_ratio.is_finite()
                && s.stack_ratio > 0.
                && s.dot_size.is_finite()
                && s.dot_size > 0.
        }
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(common::error(
            "Invalid distribution recipe control or duplicate source outlier key.",
        ))
    }
}
pub(super) fn setup(
    layer: &super::Layer,
    rows: &mut [super::compiler::EncodedRow],
    limits: super::CompileLimits,
) -> crate::ChartResult<()> {
    match &layer.recipe {
        Some(super::BuiltinRecipe::Boxplot(s)) => boxplot::setup(layer, s, rows, limits),
        Some(super::BuiltinRecipe::Violin(s)) => violin::setup(s, rows),
        Some(super::BuiltinRecipe::Dotplot(s)) => dotplot::setup(s, rows),
        _ => Ok(()),
    }
}
pub(super) fn emit(
    layer: &super::Layer,
    rows: &[super::compiler::EncodedRow],
    prepared: &mut super::PreparedLayer,
    vertices: &mut usize,
) -> crate::ChartResult<bool> {
    match &layer.recipe {
        Some(super::BuiltinRecipe::Boxplot(s)) => {
            boxplot::emit(layer, s, rows, prepared, vertices)?
        }
        Some(super::BuiltinRecipe::Violin(s)) => violin::emit(layer, s, rows, prepared, vertices)?,
        Some(super::BuiltinRecipe::Dotplot(s)) => {
            dotplot::emit(layer, s, rows, prepared, vertices)?
        }
        _ => return Ok(false),
    }
    Ok(true)
}

pub(super) fn apply_defaults(
    layer: &mut super::Layer,
    theme: &crate::theme::GeometryTheme<crate::color::Paint>,
) {
    use super::{BuiltinRecipe, PaintAesthetic};
    let box_or_violin = matches!(
        layer.recipe,
        Some(BuiltinRecipe::Boxplot(_) | BuiltinRecipe::Violin(_))
    );
    let dot = matches!(layer.recipe, Some(BuiltinRecipe::Dotplot(_)));
    let density = matches!(layer.recipe, Some(BuiltinRecipe::Density(_)));
    if !box_or_violin && !dot && !density {
        return;
    }
    if density {
        layer.style.line_join.get_or_insert(super::LineJoin::Round);
    }
    let Some(grammar) = &layer.grammar else {
        return;
    };
    if box_or_violin && grammar.default_color {
        let ink = theme.ink.resolve();
        let paper = theme.paper.resolve();
        let mix = |a: u8, b: u8| (f64::from(a) * 0.8 + f64::from(b) * 0.2).round() as u8;
        layer.style.color = crate::scene::Color {
            red: mix(ink.red, paper.red),
            green: mix(ink.green, paper.green),
            blue: mix(ink.blue, paper.blue),
            alpha: mix(ink.alpha, paper.alpha),
        }
        .into();
    }
    if !density
        && layer.style.fill.is_none()
        && !layer.paint_scales.contains_key(&PaintAesthetic::Fill)
    {
        layer.style.fill = Some(if box_or_violin {
            theme.paper
        } else {
            theme.ink
        });
    }
    if dot && grammar.default_line_width.unwrap_or(grammar.default_size) {
        layer.style.stroke_width = theme.line_width * 2.;
    }
}
