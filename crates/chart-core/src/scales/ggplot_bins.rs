//! Binned ggplot2 policies over the existing normalizer, palette and threshold search.
use super::*;
use crate::interpolate::{Number, Value};

/// Authored ggplot2 break selection.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GgplotBreaks {
    /// Fixed data-space cuts; an empty vector produces one bin.
    Explicit(Vec<Number>),
    /// Equally spaced interior cuts in data space.
    Equal(f64),
    /// Reference nice breaks, with an approximate desired count.
    Nice(f64),
}
/// Built-in palettes evaluated once using the resolved number of bins.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GgplotBinnedPalette {
    /// Reference six-symbol sequence, with missing overflow.
    Shape {
        /// Filled or hollow symbols.
        solid: bool,
    },
    /// Reference thirteen-pattern sequence, with missing overflow.
    LineType,
    /// Reference Brewer count palette (fermenter).
    Brewer {
        /// Brewer family.
        id: chromatic::SchemeId,
        /// Reverse the selected colors.
        reverse: bool,
    },
}
impl GgplotBinnedPalette {
    fn values(&self, count: usize) -> ChartResult<Vec<Value>> {
        match self {
            Self::Shape { solid } => GgplotDiscretePalette::Shape { solid: *solid },
            Self::LineType => GgplotDiscretePalette::LineType,
            Self::Brewer { id, reverse } => GgplotDiscretePalette::Brewer {
                id: *id,
                reverse: *reverse,
            },
        }
        .count_values(count)
    }
}
/// Binned training and closure policy. Prepared cuts are retained as immutable
/// population metadata because nice breaks may also extend the trained limits.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotBinnedPolicy {
    /// Optional count palette; requires an identity range function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub palette: Option<GgplotBinnedPalette>,
    /// Whether the most recent eligible batch contained no rows.
    #[serde(default)]
    pub empty_population: bool,
    /// Nonempty batch without finite transformed observations.
    #[serde(default)]
    pub nonfinite_population: bool,
    /// Optional partially authored raw limits.
    pub limits: Option<[Option<Number>; 2]>,
    /// Out-of-bounds policy before binning; reference default squishes finite values.
    pub oob: GgplotOob,
    /// Break selection before palette evaluation at each interval midpoint.
    pub breaks: GgplotBreaks,
    /// Equality belongs to the preceding interval; the lowest endpoint is included.
    pub right: bool,
    /// Exact cuts from population training; authored recipes normally omit these.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_breaks: Option<Vec<Number>>,
}
impl Default for GgplotBinnedPolicy {
    fn default() -> Self {
        Self {
            palette: None,
            empty_population: false,
            nonfinite_population: false,
            limits: None,
            oob: GgplotOob::Squish,
            breaks: GgplotBreaks::Nice(5.),
            right: true,
            prepared_breaks: None,
        }
    }
}
impl GgplotBinnedPolicy {
    pub(super) fn validate_break_budget(&self, budget: usize) -> ChartResult<()> {
        let count = match &self.breaks {
            GgplotBreaks::Explicit(v) => v.len() as f64,
            GgplotBreaks::Equal(n) | GgplotBreaks::Nice(n) => *n,
        };
        if count > budget as f64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Binned breaks exceed their count budget.",
            ));
        }
        if !count.is_finite()
            || count < 0.
            || matches!(self.breaks, GgplotBreaks::Nice(n) if n < 1.)
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Binned counts must be finite and nonnegative; nice counts must be at least one.",
            ));
        }
        Ok(())
    }
    pub(super) fn population_bounds(
        &self,
        family: NumericFamily,
        reverse: bool,
    ) -> Option<[f64; 2]> {
        if self.nonfinite_population {
            Some(super::ggplot_continuous_guide::authored_bounds(
                self.limits,
                family,
                reverse,
                [f64::INFINITY, f64::NEG_INFINITY],
            ))
        } else if self.empty_population && self.limits.is_none() {
            Some([0., 1.])
        } else {
            None
        }
    }
    pub(super) fn nonfinite_guide_entries(
        &self,
        family: NumericFamily,
        reverse: bool,
        labels: &GgplotGuideLabels,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        use super::ggplot_continuous_guide::{authored_bounds, forward, numeric_guide_entries};
        let bounds = authored_bounds(
            self.limits,
            family,
            reverse,
            [f64::INFINITY, f64::NEG_INFINITY],
        );
        let (bounds, cuts) = self.resolve_transformed(bounds, family, reverse, budget)?;
        numeric_guide_entries(
            cuts.into_iter()
                .map(|v| forward(family, reverse, v.0))
                .collect(),
            bounds,
            family,
            reverse,
            labels,
            label_budget,
        )
    }
    pub(super) fn resolve(
        &self,
        domain: [Number; 2],
        family: NumericFamily,
        reverse: bool,
    ) -> ChartResult<([Number; 2], Vec<Number>)> {
        use super::ggplot_continuous_guide::{forward, inverse};
        let (bounds, cuts) = self.resolve_transformed(
            domain.map(|v| forward(family, reverse, v.0)),
            family,
            reverse,
            4096,
        )?;
        Ok((bounds.map(|v| Number(inverse(family, reverse, v))), cuts))
    }
    fn resolve_transformed(
        &self,
        mut bounds: [f64; 2],
        family: NumericFamily,
        reverse: bool,
        budget: usize,
    ) -> ChartResult<([f64; 2], Vec<Number>)> {
        use super::ggplot_continuous_guide::{forward, inverse};
        self.validate_break_budget(budget)?;
        let raw = bounds.map(|v| inverse(family, reverse, v));
        let descending = (!raw.iter().any(|v| v.is_nan())).then(|| raw[1] < raw[0]);
        // R sort() drops missing inverse endpoints. Indexing the absent second
        // endpoint subsequently yields NA; preserve that state for discard().
        let mut sorted = raw.into_iter().filter(|v| !v.is_nan()).collect::<Vec<_>>();
        sorted.sort_by(f64::total_cmp);
        let mut limits = [
            sorted.first().copied().unwrap_or(f64::NAN),
            sorted.get(1).copied().unwrap_or(f64::NAN),
        ];
        let automatic = !matches!(self.breaks, GgplotBreaks::Explicit(_));
        let mut breaks = match &self.breaks {
            GgplotBreaks::Explicit(v) => v.iter().map(|v| v.0).collect::<Vec<_>>(),
            GgplotBreaks::Equal(n) => equal_breaks(limits, *n)?,
            GgplotBreaks::Nice(n) => {
                if let NumericFamily::Log { base } = family {
                    ggplot_breaks_log(limits, *n, base, budget)?
                } else {
                    ggplot_breaks_extended(limits, *n, budget)?
                }
            }
        };
        if breaks.len() > budget {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Binned breaks exceed their count budget.",
            ));
        }
        if automatic {
            breaks = breaks
                .into_iter()
                .filter_map(|v| {
                    if !v.is_finite() {
                        return Some(v);
                    }
                    if v < limits[0] || v > limits[1] {
                        None
                    } else if limits.iter().any(|v| v.is_nan()) {
                        Some(f64::NAN)
                    } else {
                        Some(v)
                    }
                })
                .collect();
            if self.limits.is_none() {
                let descending = descending.ok_or_else(|| {
                    error(
                        DiagnosticCode::NumericalDomain,
                        "Binned inverse limits do not have a comparable orientation.",
                    )
                })?;
                breaks.retain(|v| {
                    !limits
                        .iter()
                        .any(|limit| *v == *limit || v.is_nan() && limit.is_nan())
                });
                let n = breaks.len();
                let candidate = if n >= 2 {
                    let mut candidate = [
                        breaks[0] + (breaks[0] - breaks[1]),
                        breaks[n - 1] + (breaks[n - 1] - breaks[n - 2]),
                    ];
                    if breaks[n - 1] > limits[1] {
                        candidate[1] = breaks[n - 1];
                        breaks.pop();
                    }
                    if breaks[0] < limits[0] {
                        candidate[0] = breaks[0];
                        breaks.remove(0);
                    }
                    candidate
                } else if n == 1 {
                    let span = (breaks[0] - limits[0]).max(limits[1] - breaks[0]);
                    [breaks[0] - span, breaks[0] + span]
                } else {
                    let domain = if super::ggplot::zero_range(limits[0], limits[1]) {
                        [limits[0] - 0.05, limits[1] + 0.05]
                    } else {
                        limits
                    };
                    breaks = domain.to_vec();
                    domain
                };
                for i in 0..2 {
                    if forward(family, reverse, candidate[i]).is_finite() {
                        limits[i] = candidate[i];
                    }
                }
                bounds = limits.map(|v| forward(family, reverse, v));
                if descending {
                    bounds.reverse();
                }
            }
        }
        if breaks.len() > budget {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Binned breaks exceed their count budget.",
            ));
        }
        Ok((bounds, breaks.into_iter().map(Number).collect()))
    }
}

fn equal_breaks(limits: [f64; 2], count: f64) -> ChartResult<Vec<f64>> {
    if count > 4096. {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Equal breaks exceed the bin budget.",
        ));
    }
    if limits.iter().any(|v| !v.is_finite()) {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Equally spaced binned guides require finite limits.",
        ));
    }
    // R seq(length.out = n.breaks + 2) rounds the requested length up.
    let count = (count + 2.).ceil() as usize - 2;
    Ok((1..=count)
        .map(|i| limits[0] + (limits[1] - limits[0]) * i as f64 / (count + 1) as f64)
        .collect())
}
#[derive(Clone, Debug)]
pub(super) struct GgplotBinnedMapping {
    normalizer: ScaleNormalizer,
    thresholds: ThresholdScale<f64, usize>,
    pub bounds: Vec<Number>,
    pub right: bool,
    parameter_bounds: [f64; 2],
    sampling_valid: bool,
}
impl GgplotBinnedMapping {
    pub fn new(
        policy: &GgplotBinnedPolicy,
        function: &ScaleFunctionSpec,
        function_limits: Option<&[Number]>,
        registry: &crate::grammar::ExtensionRegistry,
        pool: &mut impl FnMut(&Value) -> ChartResult<usize>,
    ) -> ChartResult<Self> {
        let ScaleFunctionSpec::Interpolated(source) = function else {
            return Err(error(
                DiagnosticCode::Validation,
                "Binned scales require a shared interpolated range.",
            ));
        };
        if policy.palette.is_some() && !matches!(source.output, ScaleRangeFunction::Identity) {
            return Err(error(
                DiagnosticCode::Validation,
                "Count palettes require an identity range function.",
            ));
        }
        let NormalizationSpec::Ggplot {
            family,
            domain,
            reverse,
            ..
        } = source.normalization
        else {
            return Err(error(
                DiagnosticCode::Validation,
                "Binned scales require ggplot normalization.",
            ));
        };
        use super::ggplot_continuous_guide::{comparable_limits, forward, inverse};
        let mut output = InterpolatedScale::new_with_registry(source.clone(), registry)?;
        let unknown = pool(&source.unknown)?;
        policy.validate_break_budget(4096)?;
        if let Some(values) = function_limits
            && (values.len() == 1
                || values.is_empty()
                    && matches!(
                        source.normalization,
                        NormalizationSpec::Ggplot {
                            rescaler: GgplotRescaler::Maximum,
                            ..
                        }
                    ))
        {
            let value = values.first().copied().unwrap_or(Number(f64::NAN));
            return Self::constant(policy, &output, pool, unknown, value);
        }
        if policy.empty_population
            && policy.limits.is_some()
            && !comparable_limits(policy.limits, family)
                .is_some_and(|limits| limits.iter().all(Option::is_some))
        {
            return Self::unavailable(output.normalizer().clone(), unknown, policy.right);
        }
        let source_bounds = domain.map(|v| forward(family, reverse, v.0));
        let resolved = if policy.empty_population && policy.limits.is_none() {
            Ok(([0., 1.], vec![]))
        } else if let Some(bounds) = policy.population_bounds(family, reverse) {
            policy.resolve_transformed(bounds, family, reverse, 4096)
        } else if let Some(cuts) = &policy.prepared_breaks {
            Ok((source_bounds, cuts.clone()))
        } else {
            policy.resolve_transformed(source_bounds, family, reverse, 4096)
        };
        let (bounds, cuts) = match resolved {
            Ok(value) => value,
            Err(_) if policy.nonfinite_population || policy.empty_population => {
                return Self::unavailable(output.normalizer().clone(), unknown, policy.right);
            }
            Err(error) => return Err(error),
        };
        if cuts.len() > 4096 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Prepared binned cuts exceed their count budget.",
            ));
        }
        output.set_reference_bounds(bounds);
        let normalizer = output.normalizer().clone();
        // R removes duplicates before rescaling, then cut() sorts its numeric
        // boundaries independently of the palette's transformed-space order.
        let mut transformed_bounds = cuts
            .into_iter()
            .map(|n| (forward(family, reverse, n.0), n))
            .chain(bounds.map(|v| (v, Number(inverse(family, reverse, v)))))
            .filter(|(v, _)| !v.is_nan())
            .collect::<Vec<_>>();
        transformed_bounds.sort_by(|a, b| a.0.total_cmp(&b.0));
        transformed_bounds.dedup_by(|a, b| a.0 == b.0);
        if function_limits.is_some() && transformed_bounds.len() <= 1 {
            // R branches on pre-rescale boundary count, even when its only
            // boundary later normalizes to NaN (for example -Inf / -Inf).
            let value = transformed_bounds.first().map_or(Number(f64::NAN), |v| v.1);
            return Self::constant(policy, &output, pool, unknown, value);
        }
        let expected_count = transformed_bounds.len();
        let mut boundaries = transformed_bounds
            .into_iter()
            .map(|(v, n)| (normalizer.reference_parameter(v), n))
            .collect::<Vec<_>>();
        if boundaries.iter().any(|(v, _)| v.is_nan()) {
            return Self::unavailable(normalizer, unknown, policy.right);
        }
        let palette_positions = boundaries.iter().map(|(v, _)| *v).collect::<Vec<_>>();
        boundaries.sort_by(|a, b| a.0.total_cmp(&b.0));
        boundaries.dedup_by(|a, b| a.0 == b.0);
        if boundaries.is_empty() || boundaries.len() != expected_count {
            return Self::unavailable(normalizer, unknown, policy.right);
        }
        let values = if let Some(palette) = &policy.palette {
            palette
                .values(palette_positions.len().saturating_sub(1).max(1))?
                .iter()
                .map(|value| {
                    if matches!(value, Value::Missing) {
                        Ok(unknown)
                    } else {
                        pool(value)
                    }
                })
                .collect::<ChartResult<_>>()?
        } else if palette_positions.len() == 1 {
            vec![pool(&output.sample_parameter(0.5)?)?]
        } else {
            palette_positions
                .windows(2)
                .map(|p| pool(&output.sample_parameter(p[1] - (p[1] - p[0]) / 2.)?))
                .collect::<ChartResult<_>>()?
        };
        let parameter_bounds = [boundaries[0].0, boundaries.last().expect("nonempty cuts").0];
        let cuts = if boundaries.len() > 2 {
            boundaries[1..boundaries.len() - 1]
                .iter()
                .map(|p| p.0)
                .collect()
        } else {
            vec![]
        };
        let bounds = boundaries.into_iter().map(|p| p.1).collect();
        Ok(Self {
            normalizer,
            thresholds: ThresholdScale::new(ThresholdSpec {
                domain: cuts,
                range: values,
                unknown: Some(unknown),
            })?,
            bounds,
            right: policy.right,
            parameter_bounds,
            sampling_valid: true,
        })
    }
    fn constant(
        policy: &GgplotBinnedPolicy,
        output: &InterpolatedScale,
        pool: &mut impl FnMut(&Value) -> ChartResult<usize>,
        unknown: usize,
        value: Number,
    ) -> ChartResult<Self> {
        let value_index = if let Some(palette) = &policy.palette {
            pool(&palette.values(1)?[0])?
        } else {
            pool(&output.sample_parameter(0.5)?)?
        };
        Ok(Self {
            normalizer: output.normalizer().clone(),
            thresholds: ThresholdScale::new(ThresholdSpec {
                domain: vec![],
                range: vec![value_index],
                unknown: Some(unknown),
            })?,
            bounds: vec![value],
            right: policy.right,
            parameter_bounds: [0.5, 0.5],
            sampling_valid: true,
        })
    }
    fn unavailable(normalizer: ScaleNormalizer, unknown: usize, right: bool) -> ChartResult<Self> {
        Ok(Self {
            normalizer,
            thresholds: ThresholdScale::new(ThresholdSpec {
                domain: vec![],
                range: vec![unknown],
                unknown: Some(unknown),
            })?,
            bounds: vec![],
            right,
            parameter_bounds: [0., 0.],
            sampling_valid: false,
        })
    }
    pub fn validate_sampling(&self) -> ChartResult<()> {
        if self.sampling_valid {
            Ok(())
        } else {
            Err(error(
                DiagnosticCode::NumericalDomain,
                "Binned mapping requires a comparable population range and distinct rescaled cuts.",
            ))
        }
    }
    pub fn index(&self, input: Option<f64>) -> Option<usize> {
        if self.bounds.len() == 1 {
            return self.thresholds.spec().range.first().copied();
        }
        let t = self
            .normalizer
            .parameter(input)
            .filter(|v| self.parameter_bounds[0] <= *v && *v <= self.parameter_bounds[1]);
        self.thresholds
            .map_by(t.as_ref(), |d, x| if self.right { d < x } else { d <= x })
            .copied()
    }
    pub fn range(&self) -> &[usize] {
        &self.thresholds.spec().range
    }
}
