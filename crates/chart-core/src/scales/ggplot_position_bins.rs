//! Positional bin classification before statistics and interval mapping afterward.
use super::*;
use crate::interpolate::{Number, Value};

/// Reference positional binned scale, sharing numeric population and break policies.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GgplotBinnedPosition {
    /// Shared limits, cuts, out-of-bounds and closure policy.
    pub bins: GgplotBinnedPolicy,
    /// Transformation before classification; absent selects identity.
    pub transform: Option<ScaleTransform>,
    /// Guide labels for the selected cuts.
    pub labels: GgplotGuideLabels,
    /// Replacement in transformed units after OOB handling and before classification.
    pub missing: Option<Number>,
    /// Add the final scale limits to the guide candidates.
    pub show_limits: bool,
}
impl Default for GgplotBinnedPosition {
    fn default() -> Self {
        Self {
            bins: GgplotBinnedPolicy {
                breaks: GgplotBreaks::Nice(10.),
                ..Default::default()
            },
            transform: None,
            labels: GgplotGuideLabels::Automatic,
            missing: None,
            show_limits: false,
        }
    }
}
impl GgplotBinnedPosition {
    fn family(&self) -> (NumericFamily, bool) {
        match self.transform {
            None => (NumericFamily::Linear, false),
            Some(ScaleTransform::Reverse) => (NumericFamily::Linear, true),
            Some(ScaleTransform::Sqrt) => (NumericFamily::Pow { exponent: 0.5 }, false),
            Some(ScaleTransform::Log { base }) => (NumericFamily::Log { base }, false),
            Some(ScaleTransform::Symlog { threshold }) => (
                NumericFamily::Symlog {
                    constant: threshold,
                },
                false,
            ),
        }
    }
    /// Check authored parameters without selecting cuts on a placeholder population.
    pub fn validate(&self) -> ChartResult<()> {
        if self.bins.palette.is_some() {
            return Err(error(
                DiagnosticCode::Validation,
                "Position bins do not accept aesthetic palettes.",
            ));
        }
        if let Some(transform) = self.transform {
            transform.validate()?;
        }
        self.bins.validate_break_budget(4096)?;
        self.labels.validate(match &self.bins.breaks {
            GgplotBreaks::Explicit(v) if !self.show_limits => Some(v.len()),
            _ => None,
        })
    }
    /// Train the complete pre-statistic population using the shared numeric owner.
    pub fn train(&self, values: &[Option<Number>]) -> ChartResult<PreparedGgplotBinnedPosition> {
        self.validate()?;
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "positional bin population",
        )?;
        let (family, reverse) = self.family();
        let spec = MappedScaleSpec {
            resolved_numeric_limits: None,
            limits_function: None,
            training: ScaleTraining::Eligible,
            ggplot: Some(Box::new(GgplotScalePolicy::Binned(Box::new(
                self.bins.clone(),
            )))),
            guide: Some(Box::new(GgplotScaleGuide::Binned(if self.show_limits {
                GgplotGuideLabels::Automatic
            } else {
                self.labels.clone()
            }))),
            catalog: None,
            function: ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization: NormalizationSpec::Ggplot {
                    timestamp: None,
                    family,
                    domain: [Number(1.), Number(10.)],
                    reverse,
                    rescaler: GgplotRescaler::Range,
                },
                output: ScaleRangeFunction::Identity,
                unknown: Value::Missing,
            }),
        }
        .trained(values)?;
        let ScaleFunctionSpec::Interpolated(function) = &spec.function else {
            unreachable!()
        };
        let NormalizationSpec::Ggplot { domain, .. } = function.normalization else {
            unreachable!()
        };
        let Some(GgplotScalePolicy::Binned(policy)) = spec.ggplot.as_deref() else {
            unreachable!()
        };
        let limits = policy
            .population_bounds(family, reverse)
            .unwrap_or_else(|| {
                domain.map(|v| ggplot_continuous_guide::forward(family, reverse, v.0))
            });
        let mut entries = MappedScale::new(spec)?
            .binned_guide_entries(4096, 4096)?
            .unwrap();
        let selected_breaks = entries.iter().map(|e| e.transformed).collect();
        if self.show_limits {
            let mut cuts = entries
                .iter()
                .map(|v| v.transformed.0)
                .chain(limits)
                .filter(|v| !v.is_nan())
                .collect::<Vec<_>>();
            cuts.sort_by(f64::total_cmp);
            cuts.dedup_by(|a, b| *a == *b);
            entries = ggplot_continuous_guide::numeric_guide_entries(
                cuts,
                limits,
                family,
                reverse,
                &self.labels,
                4096,
            )?;
        }
        let mut boundaries = entries
            .iter()
            .map(|v| v.transformed)
            .chain(limits.map(Number))
            .filter(|v| !v.0.is_nan())
            .collect::<Vec<_>>();
        boundaries.sort_by(|a, b| a.0.total_cmp(&b.0));
        boundaries.dedup_by(|a, b| a.0 == b.0);
        let result = PreparedGgplotBinnedPosition {
            spec: self.clone(),
            limits: limits.map(Number),
            entries,
            boundaries,
            source_boundaries: None,
            statistic_boundaries: None,
            selected_breaks,
        };
        result.validate()?;
        Ok(result)
    }
}

/// Captured reference bin state. Statistics consume indices; geometry consumes intervals.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedGgplotBinnedPosition {
    spec: GgplotBinnedPosition,
    limits: [Number; 2],
    entries: Vec<GgplotContinuousGuideEntry>,
    boundaries: Vec<Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_boundaries: Option<Vec<Number>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    statistic_boundaries: Option<Vec<Number>>,
    selected_breaks: Vec<Number>,
}
impl PreparedGgplotBinnedPosition {
    /// Validate captured state before accepting it from a portable descriptor.
    pub fn validate(&self) -> ChartResult<()> {
        self.spec.validate()?;
        crate::limits::require_within(
            self.boundaries.len() <= 4098
                && self.entries.len() <= 4098
                && self.selected_breaks.len() <= 4096,
            "prepared positional bin",
        )?;
        if self.boundaries.iter().any(|v| v.0.is_nan())
            || self.boundaries.windows(2).any(|v| v[0].0 >= v[1].0)
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Positional bin boundaries must be comparable and strictly increasing.",
            ));
        }
        if let Some(cuts) = &self.source_boundaries {
            crate::limits::require_within(cuts.len() <= 4097, "scalar positional cut")?;
            if self.boundaries.len() != 1
                || cuts.len() < 2
                || cuts.iter().any(|v| !v.0.is_finite())
                || cuts.windows(2).any(|v| v[0].0 >= v[1].0)
            {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Captured scalar positional cuts are invalid.",
                ));
            }
        }
        if let Some(cuts) = &self.statistic_boundaries {
            crate::limits::require_within(cuts.len() <= 4098, "statistic positional cut")?;
            if cuts.iter().any(|v| v.0.is_nan()) || cuts.windows(2).any(|v| v[0].0 >= v[1].0) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Invalid statistic positional cuts.",
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn bind_source(mut self, values: &[Option<Number>]) -> ChartResult<Self> {
        if self.boundaries.len() == 1 && !values.is_empty() {
            let input = values
                .iter()
                .map(|v| self.input(v.map(|v| v.0)))
                .collect::<Vec<_>>();
            self.source_boundaries = Some(cut_boundaries(&self.boundaries, &input)?);
        }
        // The reference reset retains all initial cuts as its trained range,
        // then reapplies authored limits before mapping statistic indices.
        let mut cuts = self
            .selected_breaks
            .iter()
            .copied()
            .chain(self.panel_limits())
            .filter(|v| !v.0.is_nan())
            .collect::<Vec<_>>();
        cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
        cuts.dedup_by(|a, b| a.0 == b.0);
        self.statistic_boundaries = Some(cuts);
        self.validate()?;
        Ok(self)
    }
    pub(crate) fn authored(&self) -> &GgplotBinnedPosition {
        &self.spec
    }
    fn input(&self, value: Option<f64>) -> Option<f64> {
        let (family, reverse) = self.spec.family();
        self.spec
            .bins
            .oob
            .apply(
                value.map(|v| ggplot_continuous_guide::forward(family, reverse, v)),
                self.limits.map(|v| v.0),
            )
            .or(self.spec.missing.map(|v| v.0))
            .filter(|v| !v.is_nan())
    }
    pub(crate) fn project_source(&self, value: Option<f64>) -> Option<f64> {
        classify(
            self.source_boundaries
                .as_deref()
                .unwrap_or(&self.boundaries),
            self.input(value)?,
            self.spec.bins.right,
        )
        .map(|i| i as f64)
    }
    pub(crate) fn project_statistic(&self, value: f64) -> Option<f64> {
        let boundaries = self
            .statistic_boundaries
            .as_deref()
            .unwrap_or(&self.boundaries);
        let n = boundaries.len().saturating_sub(1);
        if n == 0 || !value.is_finite() || value < 0.5 || value > n as f64 + 0.5 {
            return None;
        }
        let index = if self.spec.bins.right {
            (value - 0.5).ceil()
        } else {
            (value - 0.5).floor() + 1.
        }
        .max(1.) as usize;
        if index > n {
            return None;
        }
        let a = boundaries[index - 1].0;
        let b = boundaries[index].0;
        Some((value - index as f64 + 0.5) * (b - a) + a)
    }
    /// Final limits in transformed scale units, preserving authored orientation.
    pub fn limits(&self) -> [Number; 2] {
        self.limits
    }
    pub(crate) fn panel_limits(&self) -> [Number; 2] {
        let (family, reverse) = self.spec.family();
        let fallback = [
            self.boundaries.first().map_or(self.limits[0].0, |v| v.0),
            self.boundaries.last().map_or(self.limits[1].0, |v| v.0),
        ];
        ggplot_continuous_guide::authored_bounds(self.spec.bins.limits, family, reverse, fallback)
            .map(Number)
    }
    /// Unthinned reference candidates and labels before axis composition.
    pub fn guide_entries(&self) -> &[GgplotContinuousGuideEntry] {
        &self.entries
    }
    /// Sorted interval boundaries in transformed units.
    pub fn boundaries(&self) -> &[Number] {
        &self.boundaries
    }
    /// Classify a complete raw source batch into one-based statistic inputs.
    pub fn map_before_statistics(
        &self,
        values: &[Option<Number>],
    ) -> ChartResult<Vec<Option<Number>>> {
        self.validate()?;
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "positional bin mapping",
        )?;
        let transformed = values
            .iter()
            .map(|v| self.input(v.map(|v| v.0)))
            .collect::<Vec<_>>();
        let boundaries = cut_boundaries(&self.boundaries, &transformed)?;
        Ok(transformed
            .into_iter()
            .map(|v| {
                v.and_then(|v| classify(&boundaries, v, self.spec.bins.right))
                    .map(|v| Number(v as f64))
            })
            .collect())
    }
    /// Map statistic indices and fractional offsets back into transformed intervals.
    pub fn map_after_statistics(
        &self,
        values: &[Option<Number>],
    ) -> ChartResult<Vec<Option<Number>>> {
        self.validate()?;
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "post-statistic positional bin mapping",
        )?;
        Ok(values
            .iter()
            .map(|v| v.and_then(|v| self.project_statistic(v.0)).map(Number))
            .collect())
    }
}
fn classify(bounds: &[Number], x: f64, right: bool) -> Option<usize> {
    if bounds.len() < 2 || x < bounds[0].0 || x > bounds.last()?.0 || x.is_nan() {
        return None;
    }
    Some(
        super::classifier::bisect_right(
            &bounds[1..bounds.len() - 1],
            |v| if right { v.0 < x } else { v.0 <= x },
            0,
        ) + 1,
    )
}
fn cut_boundaries(bounds: &[Number], values: &[Option<f64>]) -> ChartResult<Vec<Number>> {
    if bounds.len() != 1 {
        return Ok(bounds.to_vec());
    }
    let count = bounds[0].0;
    if !count.is_finite() || count < 2. {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "A scalar positional cut count must be finite and at least two.",
        ));
    }
    crate::limits::require_within(count <= 4096., "scalar positional cut count")?;
    let [mut lo, mut hi] = values
        .iter()
        .flatten()
        .fold([f64::INFINITY, f64::NEG_INFINITY], |[lo, hi], v| {
            [lo.min(*v), hi.max(*v)]
        });
    if !lo.is_finite() || !hi.is_finite() {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Scalar positional cuts require finite population endpoints.",
        ));
    }
    let count = (count + 1.).trunc() as usize;
    let span = hi - lo;
    let constant = span == 0.;
    let margin = if constant {
        if lo == 0. { 1. } else { lo.abs() }
    } else {
        span
    } / 1000.;
    if constant {
        lo -= margin;
        hi += margin;
    }
    let mut cuts = (0..count)
        .map(|i| Number(lo + (hi - lo) * i as f64 / (count - 1) as f64))
        .collect::<Vec<_>>();
    if !constant {
        cuts[0].0 -= margin;
        cuts[count - 1].0 += margin;
    }
    Ok(cuts)
}
