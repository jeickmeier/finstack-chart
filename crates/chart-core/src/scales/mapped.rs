//! Shared typed aesthetic-scale preparation. All arithmetic remains in family owners.
use super::*;
use crate::{
    ChartResult, DiagnosticCode,
    interpolate::{Number, Value},
    scene::Color,
};
use serde::{Deserialize, Serialize};

/// Typed scale families available to chart aesthetics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScaleFunctionSpec {
    /// Arbitrarily spaced domain and typed range knots.
    Continuous(ContinuousScaleSpec),
    /// Sequential, diverging or empirical rank normalization and interpolation.
    Interpolated(InterpolatedScaleSpec),
    /// Sample quantile or equal-width quantize output buckets.
    Classifier(ClassifierSpec<Value>),
    /// Ordered typed cutpoints.
    Threshold(ThresholdSpec<ScaleKey, Value>),
    /// Frozen category catalog with arbitrary typed range values.
    Ordinal(OrdinalSpec<ScaleKey, Value>),
}
/// Population ownership, separate from viewport and output configuration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleTraining {
    /// Use the population retained in the descriptor.
    #[default]
    Authored,
    /// Train quantiles/ranks or ordinal keys from eligible post-stat observations sharing the identity.
    Eligible,
}
/// Shared mapping descriptor for color and numeric aesthetics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappedScaleSpec {
    /// Named discrete range identity; its canonical values must match the range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog: Option<chromatic::SchemeSpec>,
    /// Typed range mapping.
    pub function: ScaleFunctionSpec,
    /// Quantile/rank/ordinal population source; other families use authored training.
    #[serde(default)]
    pub training: ScaleTraining,
}
impl MappedScaleSpec {
    /// Construct a scale using its explicit authored domain.
    pub fn authored(function: ScaleFunctionSpec) -> Self {
        Self {
            catalog: None,
            function,
            training: ScaleTraining::Authored,
        }
    }
    /// Replace a discrete/continuous range with an exact named palette and retain its identity.
    pub fn with_palette(mut self, palette: chromatic::SchemeSpec) -> ChartResult<Self> {
        let values = palette.values()?;
        match &mut self.function {
            ScaleFunctionSpec::Continuous(s) => s.range = values,
            ScaleFunctionSpec::Ordinal(s) => s.range = values,
            ScaleFunctionSpec::Classifier(s) => s.range = values,
            ScaleFunctionSpec::Threshold(s) => s.range = values,
            ScaleFunctionSpec::Interpolated(_) => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Use a named chromatic interpolator for sequential or diverging output.",
                ));
            }
        }
        self.catalog = Some(palette);
        MappedScale::new(self.clone())?;
        Ok(self)
    }
    fn validate_catalog(&self) -> ChartResult<()> {
        let Some(palette) = self.catalog else {
            return Ok(());
        };
        let expected = palette.values()?;
        let actual = match &self.function {
            ScaleFunctionSpec::Continuous(s) => Some(&s.range),
            ScaleFunctionSpec::Ordinal(s) => Some(&s.range),
            ScaleFunctionSpec::Classifier(s) => Some(&s.range),
            ScaleFunctionSpec::Threshold(s) => Some(&s.range),
            ScaleFunctionSpec::Interpolated(_) => None,
        };
        if actual != Some(&expected) {
            return Err(error(
                DiagnosticCode::Validation,
                "Named palette metadata and canonical scale range disagree.",
            ));
        }
        Ok(())
    }
    /// Whether this mapping requires the version-six catalog capability.
    pub fn has_chromatic(&self) -> bool {
        self.catalog.is_some()
            || matches!(&self.function,ScaleFunctionSpec::Interpolated(s) if matches!(&s.output,ScaleRangeFunction::Interpolate(crate::interpolate::InterpolationSpec::Chromatic {..})))
    }
    /// Whether source fields should retain exact keys instead of numerical conversion.
    pub fn categorical(&self) -> bool {
        match &self.function {
            ScaleFunctionSpec::Ordinal(_) => true,
            ScaleFunctionSpec::Threshold(s) => {
                s.domain.is_empty()
                    || !s
                        .domain
                        .iter()
                        .all(|key| matches!(key, ScaleKey::Number(_)))
            }
            _ => false,
        }
    }
    /// Maximum authored guide entries (rank guides sample five population quantiles).
    pub fn guide_entries(&self) -> usize {
        match &self.function {
            ScaleFunctionSpec::Continuous(s) => s.domain.len(),
            ScaleFunctionSpec::Interpolated(s) => match s.normalization {
                NormalizationSpec::Sequential { .. } => 2,
                NormalizationSpec::Diverging { .. } => 3,
                NormalizationSpec::Quantile { .. } => 5,
            },
            ScaleFunctionSpec::Classifier(s) => s.range.len(),
            ScaleFunctionSpec::Threshold(s) => s.range.len(),
            ScaleFunctionSpec::Ordinal(s) => s.domain.len(),
        }
    }
    /// Return a new descriptor trained from the complete eligible numeric population.
    pub fn trained(&self, values: &[Option<Number>]) -> ChartResult<Self> {
        let mut next = self.clone();
        if self.training == ScaleTraining::Eligible {
            match &mut next.function {
                ScaleFunctionSpec::Classifier(ClassifierSpec {
                    domain: ClassifierDomain::Quantile(samples),
                    ..
                }) => *samples = values.to_vec(),
                ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                    normalization: NormalizationSpec::Quantile { samples },
                    ..
                }) => *samples = values.to_vec(),
                _ => {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Eligible population training is only defined for quantile and empirical rank scales.",
                    ));
                }
            }
        }
        Ok(next)
    }
    /// Extend an eligible ordinal catalog in first-seen order, starting from authored keys.
    /// Explicit ordinal unknown policies do not add new keys.
    pub fn trained_keys(&self, keys: &[ScaleKey]) -> ChartResult<Self> {
        let mut next = self.clone();
        if self.training == ScaleTraining::Eligible {
            let ScaleFunctionSpec::Ordinal(spec) = &mut next.function else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Typed key populations require an ordinal scale.",
                ));
            };
            *spec = OrdinalScale::new(spec.clone())
                .train(
                    keys.iter()
                        .filter(|key| !self.has_chromatic() || finite_key(key))
                        .cloned(),
                )
                .spec()
                .clone();
        }
        Ok(next)
    }
    pub(crate) fn validate_training(&self) -> ChartResult<()> {
        if self.training == ScaleTraining::Eligible
            && !matches!(
                self.function,
                ScaleFunctionSpec::Ordinal(_)
                    | ScaleFunctionSpec::Classifier(ClassifierSpec {
                        domain: ClassifierDomain::Quantile(_),
                        ..
                    })
                    | ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                        normalization: NormalizationSpec::Quantile { .. },
                        ..
                    })
            )
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Eligible training requires a quantile, empirical rank or ordinal scale.",
            ));
        }
        Ok(())
    }
    pub(crate) fn trained_population(
        &self,
        population: Option<&ScalePopulation>,
    ) -> ChartResult<Self> {
        if self.training == ScaleTraining::Authored {
            return Ok(self.clone());
        }
        match (&self.function, population) {
            (ScaleFunctionSpec::Ordinal(_), Some(ScalePopulation::Keys(keys))) => {
                self.trained_keys(keys)
            }
            (ScaleFunctionSpec::Ordinal(_), None) => self.trained_keys(&[]),
            (_, Some(ScalePopulation::Numbers(values))) => self.trained(values),
            (_, None) => self.trained(&[]),
            _ => Err(error(
                DiagnosticCode::SchemaConflict,
                "Shared scale population kinds disagree.",
            )),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) enum ScalePopulation {
    Numbers(Vec<Option<Number>>),
    Keys(Vec<ScaleKey>),
}
#[derive(Clone, Debug)]
enum PreparedMapping {
    Continuous(ContinuousScale),
    Interpolated(InterpolatedScale),
    Classifier(ClassifierScale<usize>),
    Threshold(ThresholdScale<ScaleKey, usize>),
    Ordinal(OrdinalScale<ScaleKey, usize>),
}
/// Compiled aesthetic mapping; discrete outputs and their paint conversions are pooled once.
#[derive(Clone, Debug)]
pub struct MappedScale {
    spec: MappedScaleSpec,
    mapping: PreparedMapping,
    values: Vec<Value>,
    paints: Option<Vec<Option<crate::color::Paint>>>,
}
impl MappedScale {
    /// Compile an authored or already trained descriptor.
    pub fn new(spec: MappedScaleSpec) -> ChartResult<Self> {
        // Validate population policy without discarding an already trained population.
        spec.validate_training()?;
        spec.validate_catalog()?;
        let mut values = vec![];
        let mut pool = |value: &Value| -> ChartResult<usize> {
            value.validate()?;
            let i = values.len();
            values.push(value.clone());
            Ok(i)
        };
        let mapping = match &spec.function {
            ScaleFunctionSpec::Continuous(s) => {
                PreparedMapping::Continuous(ContinuousScale::new(s.clone())?)
            }
            ScaleFunctionSpec::Interpolated(s) => {
                PreparedMapping::Interpolated(InterpolatedScale::new(s.clone())?)
            }
            ScaleFunctionSpec::Classifier(s) => {
                PreparedMapping::Classifier(ClassifierScale::new(ClassifierSpec {
                    domain: s.domain.clone(),
                    range: s.range.iter().map(&mut pool).collect::<ChartResult<_>>()?,
                    unknown: s.unknown.as_ref().map(&mut pool).transpose()?,
                })?)
            }
            ScaleFunctionSpec::Threshold(s) => {
                PreparedMapping::Threshold(ThresholdScale::new(ThresholdSpec {
                    domain: s.domain.clone(),
                    range: s.range.iter().map(&mut pool).collect::<ChartResult<_>>()?,
                    unknown: s.unknown.as_ref().map(&mut pool).transpose()?,
                })?)
            }
            ScaleFunctionSpec::Ordinal(s) => {
                let range = s.range.iter().map(&mut pool).collect::<ChartResult<_>>()?;
                let unknown = match &s.unknown {
                    OrdinalUnknown::Implicit => OrdinalUnknown::Implicit,
                    OrdinalUnknown::Explicit(v) => {
                        OrdinalUnknown::Explicit(v.as_ref().map(&mut pool).transpose()?)
                    }
                };
                PreparedMapping::Ordinal(OrdinalScale::new(OrdinalSpec {
                    domain: s.domain.clone(),
                    range,
                    unknown,
                }))
            }
        };
        Ok(Self {
            spec,
            mapping,
            values,
            paints: None,
        })
    }
    /// Compile and validate color outputs; category colors are parsed/quantized once.
    pub fn for_colors(spec: MappedScaleSpec) -> ChartResult<Self> {
        let mut scale = Self::new(spec)?;
        scale.paints = Some(
            scale
                .values
                .iter()
                .map(value_paint)
                .collect::<ChartResult<_>>()?,
        );
        match &scale.mapping {
            PreparedMapping::Continuous(s) => {
                scale.paints = Some(vec![value_paint(&s.spec().unknown)?]);
                s.validate_color_output()?;
            }
            PreparedMapping::Interpolated(s) => {
                scale.paints = Some(vec![value_paint(&s.spec().unknown)?]);
                s.validate_color_output()?;
            }
            _ => {}
        }
        Ok(scale)
    }
    /// Compile and validate numeric outputs even when the current layer has no observations.
    pub fn for_numbers(spec: MappedScaleSpec) -> ChartResult<Self> {
        let scale = Self::new(spec)?;
        for value in &scale.values {
            numeric_output(value)?;
        }
        match &scale.mapping {
            PreparedMapping::Continuous(s) => {
                numeric_output(&s.spec().unknown)?;
                s.validate_numeric_output()?;
            }
            PreparedMapping::Interpolated(s) => {
                numeric_output(&s.spec().unknown)?;
                s.validate_numeric_output()?;
            }
            _ => {}
        }
        Ok(scale)
    }
    /// Retained descriptor with the exact prepared population.
    pub fn spec(&self) -> &MappedScaleSpec {
        &self.spec
    }
    fn index(&self, numeric: Option<f64>, key: Option<&ScaleKey>) -> Option<usize> {
        match &self.mapping {
            PreparedMapping::Classifier(s) => s.map(numeric).copied(),
            PreparedMapping::Threshold(s) => s.map_key(key).copied(),
            PreparedMapping::Ordinal(s) => key.map_or_else(
                || match s.spec().unknown {
                    OrdinalUnknown::Explicit(Some(i)) => Some(i),
                    _ => None,
                },
                |k| s.map(k).copied(),
            ),
            _ => None,
        }
    }
    /// Owned numerical or typed output. Numeric keys do not pass through string conversion.
    pub fn numeric(&self, input: Option<f64>) -> ChartResult<Value> {
        match &self.mapping {
            PreparedMapping::Continuous(s) => s.map(input),
            PreparedMapping::Interpolated(s) => s.map(input),
            _ => {
                let key = input
                    .filter(|x| !x.is_nan())
                    .map(|v| ScaleKey::Number(Number(v)));
                Ok(self
                    .index(input, key.as_ref())
                    .map_or(Value::Missing, |i| self.values[i].clone()))
            }
        }
    }
    /// Category/threshold output with typed identity intact.
    pub fn category(&self, key: Option<&ScaleKey>) -> ChartResult<Value> {
        Ok(self
            .index(None, key)
            .map_or(Value::Missing, |i| self.values[i].clone()))
    }
    /// Lower a sampled paint at the scene boundary.
    pub fn color(
        &self,
        input: Option<f64>,
        key: Option<&ScaleKey>,
        missing: Color,
    ) -> ChartResult<Color> {
        self.paint(input, key, missing.into())
            .map(crate::color::Paint::resolve)
    }
    /// Prepared paint sampling; no per-row CSS parsing or range-factory construction.
    pub fn paint(
        &self,
        input: Option<f64>,
        key: Option<&ScaleKey>,
        missing: crate::color::Paint,
    ) -> ChartResult<crate::color::Paint> {
        // Named chart palettes use the declared missing color for all non-finite
        // observations; direct ramp calls keep their explicit rejection contract.
        let input = if self.spec.has_chromatic() {
            input.filter(|value| value.is_finite())
        } else {
            input
        };
        let key = key.filter(|key| !self.spec.has_chromatic() || finite_key(key));
        let paints = self.paints.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::Validation,
                "Prepare this mapped scale for color outputs before paint sampling.",
            )
        })?;
        Ok(match &self.mapping {
            PreparedMapping::Continuous(s) => s
                .map_color(input)?
                .map_or(paints.first().copied().flatten().unwrap_or(missing), |v| {
                    v.into()
                }),
            PreparedMapping::Interpolated(s) => s
                .map_color(input)?
                .map_or(paints.first().copied().flatten().unwrap_or(missing), |v| {
                    v.into()
                }),
            _ => {
                let numeric_key = input
                    .filter(|x| !x.is_nan())
                    .map(|v| ScaleKey::Number(Number(v)));
                self.index(input, key.or(numeric_key.as_ref()))
                    .and_then(|i| paints[i])
                    .unwrap_or(missing)
            }
        })
    }
    /// Guide entries and interval metadata from the same prepared mapping as marks.
    pub fn legend(&self, id: crate::ScaleId, missing: Color) -> ChartResult<ColorLegend> {
        let mut entries = vec![];
        let mut intervals = vec![];
        let mut midpoint = None;
        let continuous = matches!(
            self.mapping,
            PreparedMapping::Continuous(_) | PreparedMapping::Interpolated(_)
        );
        match &self.mapping {
            PreparedMapping::Classifier(s) => {
                let formatter = interval_formatter(s.spec().range.iter().flat_map(|index| {
                    let e = s.invert_extent(index);
                    [e.lower, e.upper].into_iter().flatten()
                }))?;
                for (i, index) in s.spec().range.iter().enumerate() {
                    let e = s.invert_extent(index);
                    let label = interval_text(
                        e.lower.map(|n| formatter.format(n.0)),
                        e.upper.map(|n| formatter.format(n.0)),
                    );
                    let paint = self
                        .paints
                        .as_ref()
                        .and_then(|p| p[*index])
                        .map(crate::color::Paint::resolve)
                        .unwrap_or(missing);
                    entries.push((label, paint));
                    intervals.push(GuideInterval {
                        lower: e.lower.map(ScaleKey::Number),
                        upper: e.upper.map(ScaleKey::Number),
                        entry: i,
                    });
                }
            }
            PreparedMapping::Threshold(s) => {
                for (i, index) in s.spec().range.iter().enumerate() {
                    let e = s.invert_extent(index);
                    entries.push((
                        interval_text(
                            e.lower.as_ref().map(key_label),
                            e.upper.as_ref().map(key_label),
                        ),
                        self.paints
                            .as_ref()
                            .and_then(|p| p[*index])
                            .map(crate::color::Paint::resolve)
                            .unwrap_or(missing),
                    ));
                    intervals.push(GuideInterval {
                        lower: e.lower,
                        upper: e.upper,
                        entry: i,
                    });
                }
            }
            PreparedMapping::Ordinal(s) => {
                for k in &s.spec().domain {
                    entries.push((key_label(k), self.color(None, Some(k), missing)?));
                }
            }
            PreparedMapping::Continuous(s) => {
                for d in &s.spec().domain {
                    entries.push((
                        crate::number::ecmascript(d.0),
                        self.color(Some(d.0), None, missing)?,
                    ));
                }
            }
            PreparedMapping::Interpolated(s) => {
                let domain = s.normalizer().domain();
                if matches!(s.spec().normalization, NormalizationSpec::Diverging { .. }) {
                    midpoint = domain.get(1).copied();
                }
                let points: Vec<_> =
                    if matches!(s.spec().normalization, NormalizationSpec::Quantile { .. }) {
                        s.normalizer()
                            .quantiles(4.)?
                            .into_iter()
                            .flatten()
                            .collect()
                    } else {
                        domain.to_vec()
                    };
                for d in points {
                    entries.push((
                        crate::number::ecmascript(d.0),
                        self.color(Some(d.0), None, missing)?,
                    ));
                }
            }
        }
        Ok(ColorLegend {
            title: None,
            id,
            entries,
            continuous,
            missing,
            intervals,
            midpoint,
            mapping: Some(self.spec.clone()),
        })
    }
}
fn value_paint(value: &Value) -> ChartResult<Option<crate::color::Paint>> {
    match value {
        Value::Missing | Value::Null => Ok(None),
        Value::Color(c) => Ok(Some((*c).into())),
        Value::Text(s) => crate::color::parse(s)
            .map(|v| v.map(Into::into))
            .and_then(|v| {
                v.ok_or_else(|| {
                    error(
                        DiagnosticCode::Validation,
                        "A color scale range contains invalid color text.",
                    )
                })
            })
            .map(Some),
        _ => Err(error(
            DiagnosticCode::SchemaConflict,
            "A color scale requires color-valued outputs.",
        )),
    }
}
// Labels are presentation: start with six significant digits, increasing precision
// until distinct finite cutpoints remain distinct. Exact extents stay in GuideInterval.
fn interval_formatter(
    values: impl Iterator<Item = Number>,
) -> ChartResult<crate::typography::NumericFormatter> {
    let mut values: Vec<_> = values.map(|n| n.0).filter(|n| n.is_finite()).collect();
    values.sort_by(f64::total_cmp);
    values.dedup_by(|a, b| *a == *b);
    for precision in 6..=17 {
        let format = crate::typography::NumericFormat {
            specifier: format!(".{precision}~g"),
            locale: crate::typography::NumericLocale {
                minus: "-".into(),
                ..Default::default()
            },
        }
        .prepare()?;
        if precision == 17
            || values
                .windows(2)
                .all(|v| format.format(v[0]) != format.format(v[1]))
        {
            return Ok(format);
        }
    }
    unreachable!("binary64 needs at most seventeen significant decimal digits")
}
fn interval_text(a: Option<String>, b: Option<String>) -> String {
    match (a, b) {
        (None, None) => "All values".into(),
        (None, Some(b)) => format!("< {b}"),
        (Some(a), None) => format!(">= {a}"),
        (Some(a), Some(b)) => format!("{a} – {b}"),
    }
}
fn key_label(key: &ScaleKey) -> String {
    match key {
        ScaleKey::Null => "null".into(),
        ScaleKey::Boolean(v) => v.to_string(),
        ScaleKey::Number(v) => crate::number::ecmascript(v.0),
        ScaleKey::Integer(v) | ScaleKey::Timestamp(v) => v.to_string(),
        ScaleKey::Unsigned(v) => v.to_string(),
        ScaleKey::Text(v) => v.clone(),
    }
}
/// A guide entry's real data interval; labels are presentation only.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuideInterval {
    /// Inclusive lower cut, absent for an unbounded tail.
    pub lower: Option<ScaleKey>,
    /// Upper cut, absent for an unbounded tail.
    pub upper: Option<ScaleKey>,
    /// Index in the legend's entries.
    pub entry: usize,
}

pub(super) fn numeric_output(value: &Value) -> ChartResult<()> {
    if matches!(value, Value::Number(_) | Value::Missing | Value::Null) {
        Ok(())
    } else {
        Err(error(
            DiagnosticCode::SchemaConflict,
            "Numeric aesthetic scales require numeric or missing range outputs.",
        ))
    }
}

// Categorical numeric keys retain exact identity, but named chart palettes treat
// exceptional source numbers as missing just like numerical color observations.
fn finite_key(key: &ScaleKey) -> bool {
    !matches!(key, ScaleKey::Number(value) if !value.0.is_finite())
}
