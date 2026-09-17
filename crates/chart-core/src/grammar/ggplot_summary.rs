//! Reference count/summary populations and bounded deterministic summary helpers.
use super::statistics::{mean, quantile, sum};
use super::stats::{group_value, number, numeric_space, warning};
use super::*;
use crate::data::{DatasetSnapshot, InvalidPolicy};
use crate::provenance::Target;
use crate::{AggregateId, ChartResult, Diagnostic, DiagnosticCode, RowKey, SchemaVersion};
use std::{collections::BTreeMap, sync::Arc};

/// Opt-in count-by-position and signed weight semantics.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotCountOptions {
    /// Numeric position within each group; absent uses one categorical/group output.
    pub position: Option<Numeric>,
    /// Opt-in StatSum second coordinate; preserves the original count contract when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub joint_position: Option<Numeric>,
    /// Additional mapped source aesthetics partition joint counts without altering prop groups.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joint_aesthetics: Vec<crate::FieldId>,
    /// Evaluated numeric aesthetic inputs partition joint counts by their results.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub joint_numeric: Vec<Numeric>,
    /// Missing weights contribute zero while membership remains exact.
    pub weight: Option<Numeric>,
    /// Explicit bar width; absent uses 0.9 times minimum distinct position spacing.
    pub width: Option<f64>,
}
/// One numeric partition shared by all groups for reference binned summaries.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryBins {
    /// Explicit breaks take precedence over automatic controls.
    pub breaks: Option<Vec<f64>>,
    /// Number of automatic bins.
    pub bins: usize,
    /// Shared reference break arithmetic; weighting/padding do not apply to summaries.
    pub options: GgplotBinOptions,
}
impl Default for SummaryBins {
    fn default() -> Self {
        Self {
            breaks: None,
            bins: 30,
            options: GgplotBinOptions::default(),
        }
    }
}
/// Unweighted reference helpers; summary functions receive values, not aesthetic weights.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SummaryHelper {
    /// Mean plus/minus multiplier times standard error (default multiplier one).
    MeanSe {
        /// Standard-error multiplier.
        mult: f64,
    },
    /// Mean plus/minus multiplier times sample standard deviation (reference default two).
    MeanSdl {
        /// Standard-deviation multiplier.
        mult: f64,
    },
    /// Median and equal-tailed empirical quantiles.
    MedianHilow {
        /// Central interval probability.
        confidence: f64,
    },
    /// Mean and Student-t interval using sample degrees of freedom.
    MeanClNormal {
        /// Central interval probability.
        confidence: f64,
    },
    /// Percentile bootstrap with supplied zero-based draws, one n-element draw per replicate.
    /// Explicit draws preserve portable reproducibility without emulating hidden R RNG state.
    MeanClBoot {
        /// Central interval probability.
        confidence: f64,
        /// Explicit resample indices in source-key order.
        draws: Vec<Vec<usize>>,
    },
    /// Built-in percentile bootstrap with deterministic SplitMix64 resampling per population.
    MeanClBootSeeded {
        /// Central probability; defaults to 0.95.
        #[serde(default = "default_confidence")]
        confidence: f64,
        /// Independent replicate count; defaults to 1,000.
        #[serde(default = "default_samples")]
        samples: usize,
        /// Explicit portable seed; defaults to zero, independent of ambient host RNG.
        #[serde(default, with = "crate::portable::unsigned")]
        seed: u64,
    },
    /// Independently chosen center and endpoint exact descriptive summaries.
    Functions {
        /// Central value function.
        center: SummaryFunction,
        /// Lower endpoint function.
        lower: SummaryFunction,
        /// Upper endpoint function.
        upper: SummaryFunction,
    },
}
/// Exact scalar summary function for independently composed endpoints.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SummaryFunction {
    /// Arithmetic mean.
    Mean,
    /// Type-seven median.
    Median,
    /// Minimum.
    Min,
    /// Maximum.
    Max,
    /// Sum.
    Sum,
}
fn default_confidence() -> f64 {
    0.95
}
fn default_samples() -> usize {
    1000
}
impl SummaryHelper {
    /// Reference percentile bootstrap with 1,000 draws and explicit deterministic seed zero.
    pub fn mean_cl_boot() -> Self {
        Self::MeanClBootSeeded {
            confidence: 0.95,
            samples: 1000,
            seed: 0,
        }
    }
}
impl Default for SummaryHelper {
    fn default() -> Self {
        Self::MeanSe { mult: 1. }
    }
}
/// Summary response at each unique or binned predictor position.
#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotSummaryOptions {
    /// Predictor in source units; absent summarizes once per declared categorical group.
    pub position: Option<Numeric>,
    /// Optional shared predictor bins.
    pub bins: Option<SummaryBins>,
    /// Reference summary helper, unweighted like ggplot's built-in wrappers.
    pub helper: SummaryHelper,
}
fn invalid(message: &str) -> crate::Diagnostic {
    error(DiagnosticCode::NumericalDomain, message)
}
fn confidence(p: f64) -> ChartResult<()> {
    if p.is_finite() && p > 0. && p < 1. {
        Ok(())
    } else {
        Err(invalid(
            "Summary confidence must lie strictly between zero and one.",
        ))
    }
}
impl SummaryHelper {
    pub(super) fn validate(&self, limits: CompileLimits) -> ChartResult<()> {
        match self {
            Self::MeanSe { mult } | Self::MeanSdl { mult } if !mult.is_finite() => {
                return Err(invalid("Summary multiplier must be finite."));
            }
            Self::MedianHilow { confidence: p }
            | Self::MeanClNormal { confidence: p }
            | Self::MeanClBoot { confidence: p, .. }
            | Self::MeanClBootSeeded { confidence: p, .. } => confidence(*p)?,
            _ => {}
        }
        if let Self::MeanClBootSeeded { samples, .. } = self
            && (*samples == 0 || *samples > limits.max_prepared_rows)
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Bootstrap replicate count exceeds work budget.",
            ));
        }
        if let Self::MeanClBoot { draws, .. } = self
            && (draws.is_empty()
                || draws
                    .iter()
                    .try_fold(0usize, |n, d| n.checked_add(d.len()))
                    .is_none_or(|n| n > limits.max_prepared_rows))
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Bootstrap draws must fit the prepared-work budget.",
            ));
        }
        Ok(())
    }
}
pub(super) fn validate_count(s: &GgplotCountOptions) -> ChartResult<()> {
    if s.joint_numeric
        .len()
        .saturating_add(s.joint_aesthetics.len())
        > 64
    {
        return Err(invalid(
            "StatSum accepts at most 64 additional aesthetic partitions.",
        ));
    }
    if s.joint_position.is_some() && s.position.is_none() {
        return Err(invalid("StatSum requires both positional mappings."));
    }
    if s.width.is_some_and(|w| !w.is_finite() || w < 0.) {
        Err(invalid("Count width must be finite and nonnegative."))
    } else {
        Ok(())
    }
}
pub(super) fn validate_summary(s: &GgplotSummaryOptions, limits: CompileLimits) -> ChartResult<()> {
    s.helper.validate(limits)?;
    if let Some(b) = &s.bins {
        b.options.validate()?;
        if s.position.is_none()
            || b.bins == 0
            || b.bins >= limits.max_edges
            || b.options.weight.is_some()
            || b.options.pad
        {
            return Err(invalid(
                "Summary bins require a numeric position, bounded positive count, and unweighted unpadded break controls.",
            ));
        }
        if let Some(e) = &b.breaks
            && (e.len() < 2
                || e.len() > limits.max_edges
                || e.iter().any(|x| !x.is_finite())
                || e.windows(2).any(|x| x[0] >= x[1]))
        {
            return Err(invalid(
                "Summary breaks must be finite and strictly increasing.",
            ));
        }
    }
    Ok(())
}
pub(super) fn extra_schema(
    stat: &Statistic,
    data: &DatasetSnapshot,
) -> ChartResult<Vec<StatColumn>> {
    let mut out = vec![];
    let mut add = |field, space, nullable| {
        out.push(StatColumn {
            field,
            space,
            nullable,
            kind: GeneratedKind::Float64,
        })
    };
    match &stat.parameters {
        StatParameters::Count(s) => {
            if let Some(g) = &s.ggplot {
                if let Some(w) = &g.weight {
                    numeric_space(data, w)?;
                }
                if let Some(x) = &g.position {
                    add(StatField::X, numeric_space(data, x)?, false);
                }
                if let Some(y) = &g.joint_position {
                    add(StatField::Y, numeric_space(data, y)?, false);
                }
                for n in &g.joint_numeric {
                    numeric_space(data, n)?;
                }
                if !g.joint_aesthetics.is_empty() {
                    super::stats::validate_group(
                        data,
                        &Grouping::Interaction(g.joint_aesthetics.clone()),
                    )?;
                }
                add(StatField::WeightedCount, ValueSpace::Data, false);
                add(StatField::Proportion, ValueSpace::Data, true);
                add(StatField::Width, ValueSpace::Data, false);
            }
        }
        StatParameters::Summary(s) => {
            if let Some(g) = &s.ggplot {
                if let Some(x) = &g.position {
                    add(StatField::X, numeric_space(data, x)?, false);
                }
                let space = super::statistics::space(data, &s.input, &s.space)?;
                for f in [StatField::Y, StatField::Lower, StatField::Upper] {
                    add(f, space.clone(), true);
                }
                if g.bins.is_some() {
                    add(StatField::Width, ValueSpace::Data, false);
                }
            }
        }
        _ => {}
    }
    Ok(out)
}

fn scalar(f: SummaryFunction, v: &[f64], m: f64) -> ChartResult<f64> {
    Ok(match f {
        SummaryFunction::Mean => m,
        SummaryFunction::Median => quantile(v, 0.5).unwrap(),
        SummaryFunction::Min => v[0],
        SummaryFunction::Max => v[v.len() - 1],
        SummaryFunction::Sum => sum(v.iter().copied())?,
    })
}
/// Evaluate one finite nonempty population without ambient random state.
pub fn summary_values(helper: &SummaryHelper, values: &[f64]) -> ChartResult<[Option<f64>; 3]> {
    helper.validate(CompileLimits::default())?;
    if values.len() > CompileLimits::default().max_prepared_rows {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Summary population exceeds work budget.",
        ));
    }
    if values.iter().any(|v| !v.is_finite()) {
        return Err(invalid("Summary values must be finite."));
    }
    if values.is_empty() {
        return Ok([None, None, None]);
    }
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let m = mean(v.iter().copied(), v.len())?;
    let result = match helper {
        SummaryHelper::MedianHilow { confidence: p } => [
            quantile(&v, 0.5),
            quantile(&v, (1. - p) / 2.),
            quantile(&v, (1. + p) / 2.),
        ],
        SummaryHelper::Functions {
            center,
            lower,
            upper,
        } => [
            Some(scalar(*center, &v, m)?),
            Some(scalar(*lower, &v, m)?),
            Some(scalar(*upper, &v, m)?),
        ],
        SummaryHelper::MeanClBootSeeded {
            confidence: p,
            samples,
            seed,
        } => {
            if samples
                .checked_mul(values.len())
                .is_none_or(|n| n > CompileLimits::default().max_prepared_rows)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Bootstrap resampling exceeds work budget.",
                ));
            }
            let mut state = *seed;
            let n = values.len() as u64;
            let zone = u64::MAX - u64::MAX % n;
            let mut generated = Vec::with_capacity(*samples);
            for _ in 0..*samples {
                let mut sample = Vec::with_capacity(values.len());
                for _ in values {
                    let index = loop {
                        state = state.wrapping_add(0x9e3779b97f4a7c15);
                        let mut z = state;
                        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                        z ^= z >> 31;
                        if z < zone {
                            break (z % n) as usize;
                        }
                    };
                    sample.push(values[index]);
                }
                generated.push(mean(sample, values.len())?);
            }
            generated.sort_by(f64::total_cmp);
            [
                Some(m),
                quantile(&generated, (1. - p) / 2.),
                quantile(&generated, (1. + p) / 2.),
            ]
        }
        SummaryHelper::MeanClBoot {
            confidence: p,
            draws,
        } => {
            let mut samples = Vec::with_capacity(draws.len());
            for d in draws {
                if d.len() != values.len() || d.iter().any(|i| *i >= values.len()) {
                    return Err(invalid(
                        "Every bootstrap draw must contain n valid zero-based source-population indices.",
                    ));
                }
                samples.push(mean(d.iter().map(|i| values[*i]), d.len())?);
            }
            samples.sort_by(f64::total_cmp);
            [
                Some(m),
                quantile(&samples, (1. - p) / 2.),
                quantile(&samples, (1. + p) / 2.),
            ]
        }
        _ => {
            if v.len() == 1 {
                return Ok([Some(m), None, None]);
            }
            let scale = v.iter().map(|x| (x - m).abs()).fold(0_f64, f64::max);
            let sd = if scale == 0. {
                0.
            } else {
                scale
                    * (sum(v.iter().map(|x| ((x - m) / scale).powi(2)))? / (v.len() - 1) as f64)
                        .sqrt()
            };
            let half = match helper {
                SummaryHelper::MeanSe { mult } => mult * sd / (v.len() as f64).sqrt(),
                SummaryHelper::MeanSdl { mult } => mult * sd,
                SummaryHelper::MeanClNormal { confidence: p } => {
                    student_quantile((1. + p) / 2., (v.len() - 1) as f64)? * sd
                        / (v.len() as f64).sqrt()
                }
                _ => unreachable!(),
            };
            [Some(m), Some(m - half), Some(m + half)]
        }
    };
    if result.iter().flatten().any(|v| !v.is_finite()) {
        return Err(invalid("Summary result is not representable."));
    }
    Ok(result)
}
// Bounded incomplete-beta continued fraction and bisection for Student-t intervals.
fn beta_cf(a: f64, b: f64, x: f64) -> ChartResult<f64> {
    let tiny = 1e-300;
    let mut c = 1.;
    let mut d = 1. - (a + b) * x / (a + 1.);
    if d.abs() < tiny {
        d = tiny;
    }
    d = 1. / d;
    let mut h = d;
    for m in 1..=256 {
        let m = m as f64;
        for (step, aa) in [
            m * (b - m) * x / ((a + 2. * m - 1.) * (a + 2. * m)),
            -(a + m) * (a + b + m) * x / ((a + 2. * m) * (a + 2. * m + 1.)),
        ]
        .into_iter()
        .enumerate()
        {
            d = 1. + aa * d;
            if d.abs() < tiny {
                d = tiny;
            }
            c = 1. + aa / c;
            if c.abs() < tiny {
                c = tiny;
            }
            d = 1. / d;
            let delta = d * c;
            h *= delta;
            if step == 1 && (delta - 1.).abs() < 2e-14 {
                return Ok(h);
            }
        }
    }
    Err(invalid("Student-t beta evaluation failed to converge."))
}
fn beta_regularized(a: f64, b: f64, x: f64) -> ChartResult<f64> {
    if x <= 0. {
        return Ok(0.);
    }
    if x >= 1. {
        return Ok(1.);
    }
    let f =
        (libm::lgamma(a + b) - libm::lgamma(a) - libm::lgamma(b) + a * x.ln() + b * (-x).ln_1p())
            .exp();
    if x < (a + 1.) / (a + b + 2.) {
        Ok(f * beta_cf(a, b, x)? / a)
    } else {
        Ok(1. - f * beta_cf(b, a, 1. - x)? / b)
    }
}
pub(crate) fn student_quantile(p: f64, df: f64) -> ChartResult<f64> {
    let tail = 2. * (1. - p);
    let mut lo = 0.;
    let mut hi = 1.;
    for _ in 0..1024 {
        if beta_regularized(df / 2., 0.5, df / (df + hi * hi))? <= tail {
            break;
        }
        hi *= 2.;
    }
    for _ in 0..100 {
        let mid = lo + (hi - lo) / 2.;
        if beta_regularized(df / 2., 0.5, df / (df + mid * mid))? > tail {
            lo = mid
        } else {
            hi = mid
        }
    }
    Ok(lo + (hi - lo) / 2.)
}

fn ordered(v: f64) -> u64 {
    let bits = if v == 0. {
        0_f64.to_bits()
    } else {
        v.to_bits()
    };
    if bits >> 63 != 0 {
        !bits
    } else {
        bits ^ (1 << 63)
    }
}
struct Sample {
    key: RowKey,
    value: f64,
    weight: f64,
}
struct Population {
    x: Option<f64>,
    joint_y: Option<f64>,
    retained_numeric: Vec<(Numeric, Option<f64>)>,
    width: Option<f64>,
    samples: Vec<Sample>,
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    table: &mut PreparedTable,
    data: &DatasetSnapshot,
    stat: &Statistic,
    policy: InvalidPolicy,
    scope: &str,
    limits: CompileLimits,
    counts: &mut PopulationCounts,
    diagnostics: &mut Vec<Diagnostic>,
    fields: Vec<StatColumn>,
) -> ChartResult<(Grouping, StatSpace)> {
    let (grouping, position, declared) = match &stat.parameters {
        StatParameters::Count(s) => (
            &s.grouping,
            s.ggplot.as_ref().unwrap().position.as_ref(),
            StatSpace::Data,
        ),
        StatParameters::Summary(s) => (
            &s.grouping,
            s.ggplot.as_ref().unwrap().position.as_ref(),
            s.space.clone(),
        ),
        _ => unreachable!(),
    };
    let joint = if let StatParameters::Count(s) = &stat.parameters {
        s.ggplot.as_ref().unwrap().joint_position.as_ref()
    } else {
        None
    };
    let partitions = if let StatParameters::Count(s) = &stat.parameters {
        s.ggplot.as_ref().unwrap().joint_aesthetics.as_slice()
    } else {
        &[]
    };
    let numeric_partitions = if let StatParameters::Count(s) = &stat.parameters {
        s.ggplot.as_ref().unwrap().joint_numeric.as_slice()
    } else {
        &[]
    };
    let breaks = if let StatParameters::Summary(s) = &stat.parameters {
        if let Some(b) = &s.ggplot.as_ref().unwrap().bins {
            Some(if let Some(e) = &b.breaks {
                e.clone()
            } else {
                super::statistics::automatic(
                    &AutoBinSpec {
                        input: position.unwrap().clone(),
                        bins: b.bins,
                        grouping: grouping.clone(),
                        space: StatSpace::Data,
                        ggplot: Some(b.options.clone()),
                    },
                    table,
                    data,
                )?
                .edges
            })
        } else {
            None
        }
    } else {
        None
    };
    let closed = if let StatParameters::Summary(s) = &stat.parameters {
        s.ggplot
            .as_ref()
            .unwrap()
            .bins
            .as_ref()
            .map(|b| b.options.closed)
            .unwrap_or_default()
    } else {
        BinClosure::Right
    };
    let fuzzy = breaks
        .as_ref()
        .map(|e| super::ggplot_stats::fuzzy_edges(e, closed));
    let PreparedRows::Source(rows) = &table.rows else {
        return Err(invalid("Reference summaries require source rows."));
    };
    let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
    type PopulationKey = (GroupValue, u64, u64, GroupValue, Vec<Option<u64>>);
    let mut populations: BTreeMap<PopulationKey, Population> = BTreeMap::new();
    let mut excluded = vec![];
    for row in rows.iter() {
        let source = index[&row.key];
        let group = group_value(source, grouping);
        let x = position.and_then(|p| number(source, p));
        let joint_y = joint.and_then(|p| number(source, p));
        let partition = group_value(source, &Grouping::Interaction(partitions.to_vec()));
        let retained_numeric = numeric_partitions
            .iter()
            .map(|n| (n.clone(), number(source, n)))
            .collect::<Vec<_>>();
        let numeric_key = retained_numeric
            .iter()
            .map(|(_, v)| v.map(ordered))
            .collect::<Vec<_>>();
        let (value, weight, usable) = match &stat.parameters {
            StatParameters::Count(s) => (
                0.,
                s.ggplot.as_ref().unwrap().weight.as_ref().map_or(1., |w| {
                    super::stats::raw_number(source, w)
                        .filter(|v| !v.is_nan())
                        .unwrap_or(0.)
                }),
                s.required.iter().all(|r| number(source, r).is_some()),
            ),
            StatParameters::Summary(s) => {
                let v = number(source, &s.input).and_then(|v| match &s.space {
                    StatSpace::Data => Some(v),
                    StatSpace::Transformed(t) => {
                        let v = v * t.factor + t.offset;
                        v.is_finite().then_some(v)
                    }
                });
                (v.unwrap_or(0.), 1., v.is_some())
            }
            _ => unreachable!(),
        };
        if group.is_none()
            || !usable
            || position.is_some() && x.is_none()
            || joint.is_some() && joint_y.is_none()
        {
            counts.invalid_stat += 1;
            if excluded.len() < 32 {
                excluded.push(row.key)
            }
            continue;
        }
        let (key, x, width) = if let (Some(e), Some(fuzzy), Some(x)) = (&breaks, &fuzzy, x) {
            let bin = match closed {
                BinClosure::Right => fuzzy.partition_point(|e| *e < x).saturating_sub(1),
                BinClosure::Left => fuzzy.partition_point(|e| *e <= x).saturating_sub(1),
            };
            if x < fuzzy[0] || x > *fuzzy.last().unwrap() || bin >= e.len() - 1 {
                counts.invalid_stat += 1;
                continue;
            }
            (
                bin as u64,
                Some(e[bin].midpoint(e[bin + 1])),
                Some(e[bin + 1] - e[bin]),
            )
        } else {
            (x.map_or(0, ordered), x, None)
        };
        let entry = populations
            .entry((
                group.unwrap(),
                key,
                joint_y.map_or(0, ordered),
                partition.unwrap_or(GroupValue::All),
                numeric_key,
            ))
            .or_insert_with(|| Population {
                x,
                joint_y,
                retained_numeric,
                width,
                samples: vec![],
            });
        entry.samples.push(Sample {
            key: row.key,
            value,
            weight,
        });
        if populations.len() > limits.max_groups
            || populations
                .len()
                .checked_mul(fields.len())
                .is_none_or(|n| n > limits.max_prepared_rows)
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Reference summary output budget exceeded.",
            ));
        }
    }
    warning(
        policy,
        counts.invalid_stat,
        excluded,
        "Reference count/summary excluded invalid positions, responses or groups.",
        diagnostics,
    )?;
    let mut xs = populations.values().filter_map(|p| p.x).collect::<Vec<_>>();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    let resolution = xs
        .windows(2)
        .map(|x| x[1] - x[0])
        .filter(|v| *v > 0.)
        .fold(f64::INFINITY, f64::min);
    let resolution = if resolution.is_finite() {
        resolution
    } else {
        1.
    };
    let mut totals: BTreeMap<GroupValue, f64> = BTreeMap::new();
    if matches!(stat.parameters, StatParameters::Count(_)) {
        for ((g, _, _, _, _), p) in &populations {
            let count = sum(p.samples.iter().map(|s| s.weight))?;
            let v = totals.entry(g.clone()).or_default();
            *v += if joint.is_some() { count } else { count.abs() };
            if !v.is_finite() {
                return Err(invalid(
                    "Reference count normalization is not representable.",
                ));
            }
        }
    }
    let mut output = vec![];
    for ((group, key, joint_key, partition, numeric_key), mut p) in populations {
        p.samples.sort_by_key(|s| s.key);
        let members: Arc<[RowKey]> = p.samples.iter().map(|s| s.key).collect();
        let mut values = vec![];
        let mut push = |field, value| values.push(StatValue { field, value });
        if let Some(x) = p.x {
            push(StatField::X, Some(x))
        }
        match &stat.parameters {
            StatParameters::Count(s) => {
                let count = sum(p.samples.iter().map(|s| s.weight))?;
                if let Some(y) = p.joint_y {
                    push(StatField::Y, Some(y));
                }
                push(StatField::WeightedCount, Some(count));
                push(
                    StatField::Proportion,
                    (totals[&group] != 0.).then(|| count / totals[&group]),
                );
                push(
                    StatField::Width,
                    Some(s.ggplot.as_ref().unwrap().width.unwrap_or(0.9 * resolution)),
                );
            }
            StatParameters::Summary(s) => {
                let source = p.samples.iter().map(|s| s.value).collect::<Vec<_>>();
                let summary = summary_values(&s.ggplot.as_ref().unwrap().helper, &source)?;
                for (f, v) in [StatField::Y, StatField::Lower, StatField::Upper]
                    .into_iter()
                    .zip(summary)
                {
                    push(f, v)
                }
                let mut sorted = source.clone();
                sorted.sort_by(f64::total_cmp);
                push(StatField::Min, sorted.first().copied());
                push(StatField::Max, sorted.last().copied());
                push(
                    StatField::Mean,
                    Some(mean(source.iter().copied(), source.len())?),
                );
                push(StatField::Sum, Some(sum(source.iter().copied())?));
                for (i, q) in s.quantiles.iter().enumerate() {
                    push(StatField::Quantile(i), quantile(&sorted, *q));
                }
                if let Some(w) = p.width {
                    push(StatField::Width, Some(w));
                }
            }
            _ => unreachable!(),
        }
        let target = Target::Aggregate {
            id: AggregateId::new(0),
            group: if joint.is_some() {
                format!(
                    "{scope}/{group:?}/{key}/{joint_key}/{partition:?}/{numeric_key:?}/{}",
                    stat.operation.id
                )
            } else {
                format!("{scope}/{group:?}/{key}/{}", stat.operation.id)
            },
            input: table.input,
            members: members.clone(),
        };
        output.push(StatisticalRow {
            outliers: vec![],
            retained: if joint.is_some() {
                partition
            } else {
                GroupValue::All
            },
            retained_numeric: p.retained_numeric,
            group,
            count: members.len() as u64,
            values,
            members,
            target,
        });
    }
    table.schema = OutputSchema::Statistical {
        version: SchemaVersion::new(2),
        fields,
    };
    table.rows = PreparedRows::Statistical(output.into());
    Ok((grouping.clone(), declared))
}
