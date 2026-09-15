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
    /// Explicit discrete palette coerced by a public binned paint constructor.
    Discrete(Box<GgplotDiscretePalette>),
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
            Self::Discrete(palette) => return palette.count_values(count),
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
    /// Names aligned with prepared function cuts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepared_break_names: Option<Vec<String>>,
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
            prepared_break_names: None,
        }
    }
}
impl GgplotBinnedPolicy {
    pub(super) fn validate_break_budget(&self, budget: usize) -> ChartResult<()> {
        if let Some(names) = &self.prepared_break_names {
            crate::limits::require_within(
                self.prepared_breaks
                    .as_ref()
                    .is_some_and(|cuts| cuts.len() == names.len())
                    && names.len() <= budget,
                "prepared binned break names",
            )?;
            crate::limits::require_within(
                names.iter().fold(0usize, |n, s| n.saturating_add(s.len()))
                    <= crate::interpolate::MAX_VALUE_BYTES,
                "prepared binned break name bytes",
            )?;
        }
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
            family.clone(),
            reverse,
            [f64::INFINITY, f64::NEG_INFINITY],
        );
        let (bounds, cuts) = self.resolve_transformed(bounds, family.clone(), reverse, budget)?;
        numeric_guide_entries(
            cuts.into_iter()
                .map(|v| forward(&family, reverse, v.0))
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
            domain.map(|v| forward(&family, reverse, v.0)),
            family.clone(),
            reverse,
            4096,
        )?;
        Ok((bounds.map(|v| Number(inverse(&family, reverse, v))), cuts))
    }
    pub(super) fn resolve_transformed(
        &self,
        mut bounds: [f64; 2],
        family: NumericFamily,
        reverse: bool,
        budget: usize,
    ) -> ChartResult<([f64; 2], Vec<Number>)> {
        use super::ggplot_continuous_guide::{forward_values, inverse_values};
        self.validate_break_budget(budget)?;
        let raw = inverse_values(&family, reverse, &bounds)?;
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
                super::ggplot_continuous_guide::numeric_breaks(&family, limits, *n, budget)?
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
                let candidates = forward_values(&family, reverse, &candidate)?;
                for i in 0..2 {
                    if candidates[i].is_finite() {
                        limits[i] = candidate[i];
                    }
                }
                bounds = forward_values(&family, reverse, &limits)?
                    .try_into()
                    .expect("length-preserving transform");
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
    // Match seq's step-first arithmetic: a final-bit difference can decide
    // whether an extrapolated show.limits candidate is inside the panel.
    let step = (limits[1] - limits[0]) / (count + 1) as f64;
    Ok((1..=count).map(|i| limits[0] + step * i as f64).collect())
}
#[derive(Clone, Debug)]
pub(super) struct GgplotBinnedMapping {
    normalizer: ScaleNormalizer,
    thresholds: ThresholdScale<f64, usize>,
    pub bounds: Vec<Number>,
    pub right: bool,
    parameter_bounds: [f64; 2],
    sampling_valid: bool,
    null_palette: bool,
    pipeline: Option<Box<BinnedVectorPipeline>>,
}
/// Cache belongs to this immutable prepared population. A NULL palette is not
/// cached, matching reference lookup; empty vectors are cached.
#[derive(Clone, Debug)]
struct BinnedVectorPipeline {
    stages: super::ggplot_vector::VectorPipeline,
    cuts: Vec<Number>,
    output: InterpolatedScale,
    palette: Option<Box<crate::grammar::ScalePaletteOperation>>,
    builtin: Option<GgplotBinnedPalette>,
    registry: std::sync::Arc<crate::grammar::scale_palette_extensions::ScalePaletteRegistrations>,
    cache: std::sync::Arc<std::sync::OnceLock<Vec<Value>>>,
}
impl BinnedVectorPipeline {
    fn map(
        &self,
        normalizer: &ScaleNormalizer,
        right: bool,
        inputs: &[Option<f64>],
    ) -> ChartResult<super::ggplot_palette::PaletteBatch<Value>> {
        use super::ggplot_palette::PaletteBatch;
        if inputs.is_empty() {
            return Ok(PaletteBatch {
                values: Some(vec![]),
                indices: vec![],
            });
        }
        let samples = self.stages.map(normalizer, inputs)?;
        self.map_samples(normalizer, right, &samples)
    }
    fn map_transformed(
        &self,
        normalizer: &ScaleNormalizer,
        right: bool,
        values: &[Number],
    ) -> ChartResult<super::ggplot_palette::PaletteBatch<Value>> {
        if values.is_empty() {
            return Ok(super::ggplot_palette::PaletteBatch {
                values: Some(vec![]),
                indices: vec![],
            });
        }
        let samples = self.stages.map_transformed(normalizer, values)?;
        self.map_samples(normalizer, right, &samples)
    }
    fn map_samples(
        &self,
        normalizer: &ScaleNormalizer,
        right: bool,
        samples: &[Number],
    ) -> ChartResult<super::ggplot_palette::PaletteBatch<Value>> {
        use super::ggplot_palette::PaletteBatch;
        let positions = self.stages.rescale(normalizer, &self.cuts)?;
        let midpoints = if positions.len() <= 1 {
            vec![Number(0.5)]
        } else {
            positions
                .windows(2)
                .map(|p| Number(p[1].0 - (p[1].0 - p[0].0) / 2.))
                .collect()
        };
        // cut() sorts independently, but palette samples retain rescaler order.
        let mut sorted = positions.iter().map(|p| p.0).collect::<Vec<_>>();
        sorted.sort_by(f64::total_cmp);
        if sorted.len() > 1
            && (sorted.iter().any(|v| v.is_nan()) || sorted.windows(2).any(|p| p[0] == p[1]))
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Binned callback boundaries must be distinct and comparable.",
            ));
        }
        let values = if let Some(values) = self.cache.get() {
            Some(values.clone())
        } else {
            let values = if let Some(call) = &self.palette {
                self.registry
                    .evaluate(
                        call,
                        crate::grammar::ScalePaletteDomain::Normalized(&midpoints),
                    )?
                    .values
            } else if let Some(palette) = &self.builtin {
                Some(palette.values(midpoints.len())?)
            } else {
                Some(
                    midpoints
                        .iter()
                        .map(|p| self.output.sample_parameter(p.0))
                        .collect::<ChartResult<Vec<_>>>()?,
                )
            };
            if let Some(values) = &values {
                let _ = self.cache.set(values.clone());
            }
            values
        };
        let Some(values) = values else {
            return Ok(PaletteBatch {
                values: None,
                indices: vec![],
            });
        };
        let mut values = (0..midpoints.len())
            .map(|i| match values.get(i) {
                None | Some(Value::Missing) => self.output.spec().unknown.clone(),
                Some(Value::Number(Number(v))) if v.is_nan() => self.output.spec().unknown.clone(),
                Some(value) => value.clone(),
            })
            .collect::<Vec<_>>();
        let unknown = values.len();
        values.push(self.output.spec().unknown.clone());
        let indices = if positions.len() <= 1 {
            vec![0]
        } else {
            let count = midpoints.len();
            let thresholds = ThresholdScale::new(ThresholdSpec {
                domain: sorted[1..sorted.len() - 1].to_vec(),
                range: (0..count).collect::<Vec<_>>(),
                unknown: Some(unknown),
            })?;
            samples
                .iter()
                .map(|sample| {
                    let sample = Some(sample.0)
                        .filter(|v| sorted[0] <= *v && *v <= sorted[sorted.len() - 1]);
                    thresholds
                        .map_by(sample.as_ref(), |d, x| if right { d < x } else { d <= x })
                        .copied()
                        .unwrap_or(unknown)
                })
                .collect()
        };
        Ok(PaletteBatch {
            values: Some(values),
            indices,
        })
    }
}
impl GgplotBinnedMapping {
    pub fn new(
        spec: &MappedScaleSpec,
        registry: &crate::grammar::ExtensionRegistry,
        pool: &mut impl FnMut(&Value) -> ChartResult<usize>,
    ) -> ChartResult<Self> {
        let Some(GgplotScalePolicy::Binned(policy)) = spec.ggplot.as_deref() else {
            unreachable!("binned scale")
        };
        let function = &spec.function;
        let function_limits = spec.resolved_numeric_limits.as_deref().map(Vec::as_slice);
        let palette_active = !policy.empty_population
            || (policy.limits.is_some()
                && !matches!(&policy.breaks, GgplotBreaks::Explicit(breaks) if breaks.is_empty())
                && !matches!(spec.guide.as_deref(), Some(GgplotScaleGuide::Hidden)));
        let palette_call = spec
            .palette_function
            .as_deref()
            .map(|call| (call, palette_active));
        let pipeline = super::ggplot_vector::VectorPipeline::new(spec, registry).or_else(|| {
            if !palette_active
                && matches!(function, ScaleFunctionSpec::Interpolated(source)
                if matches!(&source.output, ScaleRangeFunction::Interpolate(
                    crate::interpolate::InterpolationSpec::GgplotPalette { .. }
                )))
            {
                super::ggplot_vector::VectorPipeline::reference(spec, registry)
            } else {
                None
            }
        });
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
            ref family,
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
        use super::ggplot_continuous_guide::{comparable_limits, forward};
        let mut output = InterpolatedScale::new_with_registry(source.clone(), registry)?;
        let unknown = pool(&source.unknown)?;
        policy.validate_break_budget(4096)?;
        if pipeline.is_none()
            && let Some(values) = function_limits
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
            return Self::constant(
                policy,
                &output,
                pool,
                unknown,
                value,
                palette_call.map(|(c, active)| (c, &*registry.palette_function, active)),
            );
        }
        if policy.empty_population
            && policy.limits.is_some()
            && !comparable_limits(policy.limits, family.clone())
                .is_some_and(|limits| limits.iter().all(Option::is_some))
        {
            return Self::unavailable(output.normalizer().clone(), unknown, policy.right);
        }
        let source_bounds = spec.trained_transformed_bounds.map_or_else(
            || domain.map(|v| forward(family, reverse, v.0)),
            |v| v.map(|v| v.0),
        );
        let resolved = if policy.empty_population && policy.limits.is_none() {
            Ok(([0., 1.], vec![]))
        } else if spec.trained_transformed_bounds.is_some() {
            if let Some(cuts) = &policy.prepared_breaks {
                Ok((source_bounds, cuts.clone()))
            } else {
                policy.resolve_transformed(source_bounds, family.clone(), reverse, 4096)
            }
        } else if let Some(bounds) = policy.population_bounds(family.clone(), reverse) {
            policy.resolve_transformed(bounds, family.clone(), reverse, 4096)
        } else if let Some(cuts) = &policy.prepared_breaks {
            Ok((source_bounds, cuts.clone()))
        } else {
            policy.resolve_transformed(source_bounds, family.clone(), reverse, 4096)
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
        let transformed_cuts = super::ggplot_continuous_guide::forward_values(
            family,
            reverse,
            &cuts.iter().map(|v| v.0).collect::<Vec<_>>(),
        )?;
        let raw_bounds = super::ggplot_continuous_guide::inverse_values(family, reverse, &bounds)?;
        let mut transformed_bounds = transformed_cuts
            .into_iter()
            .zip(cuts)
            .chain(bounds.into_iter().zip(raw_bounds.into_iter().map(Number)))
            .filter(|(v, _)| !v.is_nan())
            .collect::<Vec<_>>();
        transformed_bounds.sort_by(|a, b| a.0.total_cmp(&b.0));
        transformed_bounds.dedup_by(|a, b| a.0 == b.0);
        if let Some(stages) = pipeline {
            let bounds = transformed_bounds.iter().map(|p| p.1).collect();
            let cuts = transformed_bounds
                .into_iter()
                .map(|p| Number(p.0))
                .collect();
            return Ok(Self {
                normalizer,
                bounds,
                right: policy.right,
                parameter_bounds: [0., 1.],
                sampling_valid: true,
                null_palette: false,
                thresholds: ThresholdScale::new(ThresholdSpec {
                    domain: vec![],
                    range: vec![unknown],
                    unknown: Some(unknown),
                })?,
                pipeline: Some(Box::new(BinnedVectorPipeline {
                    stages,
                    cuts,
                    output,
                    palette: spec.palette_function.clone(),
                    builtin: policy.palette.clone(),
                    registry: registry.palette_function.clone(),
                    cache: std::sync::Arc::new(std::sync::OnceLock::new()),
                })),
            });
        }
        if function_limits.is_some() && transformed_bounds.len() <= 1 {
            // R branches on pre-rescale boundary count, even when its only
            // boundary later normalizes to NaN (for example -Inf / -Inf).
            let value = transformed_bounds.first().map_or(Number(f64::NAN), |v| v.1);
            return Self::constant(
                policy,
                &output,
                pool,
                unknown,
                value,
                palette_call.map(|(c, active)| (c, &*registry.palette_function, active)),
            );
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
        let mut null_palette = false;
        let values = if let Some((call, true)) = palette_call {
            let samples = if palette_positions.len() == 1 {
                vec![Number(0.5)]
            } else {
                palette_positions
                    .windows(2)
                    .map(|p| Number(p[1] - (p[1] - p[0]) / 2.))
                    .collect()
            };
            let output = registry.palette_function.evaluate(
                call,
                crate::grammar::ScalePaletteDomain::Normalized(&samples),
            )?;
            null_palette = output.values.is_none();
            let values = output.values.unwrap_or_default();
            (0..samples.len())
                .map(|i| match values.get(i) {
                    None | Some(Value::Missing) => Ok(unknown),
                    Some(Value::Number(Number(v))) if v.is_nan() => Ok(unknown),
                    Some(value) => pool(value),
                })
                .collect::<ChartResult<_>>()?
        } else if palette_call.is_some() {
            vec![unknown; palette_positions.len().saturating_sub(1).max(1)]
        } else if let Some(palette) = &policy.palette {
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
            null_palette,
            pipeline: None,
        })
    }
    fn constant(
        policy: &GgplotBinnedPolicy,
        output: &InterpolatedScale,
        pool: &mut impl FnMut(&Value) -> ChartResult<usize>,
        unknown: usize,
        value: Number,
        palette_call: Option<(
            &crate::grammar::ScalePaletteOperation,
            &crate::grammar::scale_palette_extensions::ScalePaletteRegistrations,
            bool,
        )>,
    ) -> ChartResult<Self> {
        let mut null_palette = false;
        let value_index = if let Some((call, registry, true)) = palette_call {
            let result = registry.evaluate(
                call,
                crate::grammar::ScalePaletteDomain::Normalized(&[Number(0.5)]),
            )?;
            null_palette = result.values.is_none();
            match result.values.as_deref().and_then(|v| v.first()) {
                None | Some(Value::Missing) => unknown,
                Some(Value::Number(Number(v))) if v.is_nan() => unknown,
                Some(value) => pool(value)?,
            }
        } else if palette_call.is_some() {
            unknown
        } else if let Some(palette) = &policy.palette {
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
            null_palette,
            pipeline: None,
        })
    }
    pub(super) fn validate_guide_mapping(&self) -> ChartResult<()> {
        if self.null_palette {
            return Err(error(
                DiagnosticCode::Validation,
                "A NULL binned palette cannot supply guide keys.",
            ));
        }
        if !self.sampling_valid {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Binned guide mapping requires valid unique rescaled boundaries.",
            ));
        }
        Ok(())
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
            null_palette: false,
            pipeline: None,
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
    pub(super) fn has_null_palette(&self) -> bool {
        self.null_palette
    }
    pub fn batch_indices(&self, inputs: impl Iterator<Item = Option<f64>>) -> Vec<usize> {
        if self.null_palette {
            return vec![];
        }
        if self.bounds.len() == 1 {
            return vec![self.thresholds.spec().range[0]];
        }
        inputs
            .map(|input| {
                self.index(input)
                    .expect("binned mapping retains its unknown slot")
            })
            .collect()
    }
    pub fn pipeline_palette(&self) -> Option<&[Value]> {
        self.pipeline.as_ref()?.cache.get().map(Vec::as_slice)
    }
    pub fn pipeline_batch(
        &self,
        inputs: &[Option<f64>],
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Value>>> {
        self.pipeline
            .as_ref()
            .map(|pipeline| pipeline.map(&self.normalizer, self.right, inputs))
            .transpose()
    }
    pub fn pipeline_transformed_batch(
        &self,
        values: &[Number],
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Value>>> {
        self.pipeline
            .as_ref()
            .map(|pipeline| pipeline.map_transformed(&self.normalizer, self.right, values))
            .transpose()
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

/// Reference default color-step key selection for function-labelled numeric cuts.
/// Scale-level cuts remain unchanged; composition censors before the label callback.
pub(super) fn prepare_color_label_inputs(
    entries: &mut [GgplotContinuousGuideEntry],
    limits: [f64; 2],
) -> Vec<Option<Number>> {
    for entry in entries.iter_mut() {
        if entry.transformed.0.is_finite() && !entry.visible {
            entry.value = Number(f64::NAN);
        }
    }
    let mut boundaries = limits
        .into_iter()
        .chain(
            entries
                .iter()
                .filter(|e| !e.value.0.is_nan())
                .map(|e| e.transformed.0),
        )
        .filter(|v| !v.is_nan())
        .collect::<Vec<_>>();
    boundaries.sort_by(f64::total_cmp);
    boundaries.dedup_by(|a, b| *a == *b);
    let bins = boundaries.len().saturating_sub(1);
    let shift = usize::from(
        entries
            .first()
            .is_some_and(|e| !e.value.0.is_nan() && limits.contains(&e.transformed.0)),
    );
    let mut position = 0;
    let mut positions = Vec::with_capacity(entries.len());
    for entry in entries.iter_mut() {
        if entry.value.0.is_nan() {
            entry.visible = false;
            positions.push(None);
        } else {
            position += 1;
            entry.visible = position - shift <= bins;
            positions.push(
                (entry.visible && bins > 0)
                    .then(|| Number((position - shift) as f64 / bins as f64)),
            );
        }
    }
    positions
}
