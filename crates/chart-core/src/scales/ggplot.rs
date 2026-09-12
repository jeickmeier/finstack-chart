//! ggplot2 adapters for population and missing/outside policies. Numeric
//! transforms, interpolation, classifiers and ordinal lookup stay in their owners.
use super::*;
use crate::{
    color::Paint,
    interpolate::{Number, Value},
};
use std::collections::BTreeSet;

pub(crate) fn zero_range(a: f64, b: f64) -> bool {
    a == b || (a != 0. && b != 0. && (a - b).abs() / a.abs().min(b.abs()) < 1000. * f64::EPSILON)
}

/// ggplot2/scales out-of-bounds functions, applied before range interpolation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GgplotOob {
    /// Replace finite outside values with missing; infinities are retained.
    #[default]
    Censor,
    /// Replace every outside value, including infinities, with missing.
    CensorAny,
    /// Saturate finite outside values; infinities are retained.
    Squish,
    /// Saturate every outside value, including infinities.
    SquishAny,
    /// Retain values for the range policy to evaluate.
    Keep,
    /// Saturate infinities only, retaining finite outside values.
    SquishInfinite,
}
impl GgplotOob {
    pub(super) fn apply(self, input: Option<f64>, domain: [f64; 2]) -> Option<f64> {
        let v = input.filter(|v| !v.is_nan())?;
        let [a, b] = domain;
        match self {
            Self::Censor if v.is_finite() && (v < a || v > b) => None,
            Self::CensorAny if v < a || v > b => None,
            Self::Squish | Self::SquishAny if self == Self::SquishAny || v.is_finite() => {
                let v = if v < a { a } else { v };
                Some(if v > b { b } else { v })
            }
            Self::SquishInfinite if v.is_infinite() => Some(if v < 0. { a } else { b }),
            _ => Some(v),
        }
    }
}
/// Palette selected after the complete discrete population is known.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GgplotDiscretePalette {
    /// Six reference point codes; additional categories map to missing.
    Shape {
        /// Select filled rather than hollow reference point codes.
        solid: bool,
    },
    /// Thirteen reference line patterns; additional categories map to missing.
    LineType,
    /// Reference ordinal size, alpha or linewidth sequence over the trained levels.
    NumericRange {
        /// First and last output; a single level receives the first value.
        range: [Number; 2],
        /// Interpolate squared endpoints and take the square root for ordinal size.
        area: bool,
    },
    /// Polar-Luv hue palette.
    Hue(chromatic::ggplot::HuePalette),
    /// Gamma-2.2 grey palette.
    Grey {
        /// First grey intensity.
        start: f64,
        /// Last grey intensity.
        end: f64,
    },
    /// Reference ColorBrewer table and truncation/overflow policy.
    Brewer {
        /// Exact family identity.
        id: chromatic::SchemeId,
        /// Reverse the selected palette.
        reverse: bool,
    },
    /// All viridisLite options with inclusive interval sampling.
    Viridis {
        /// Reference map identity.
        option: chromatic::ggplot::ViridisOption,
        /// First position.
        begin: f64,
        /// Last position.
        end: f64,
        /// Swap the endpoints.
        reverse: bool,
        /// Alpha coverage.
        alpha: f64,
    },
    /// Values matched by optional exact names; unnamed shortage is an error.
    Manual {
        /// Typed outputs.
        values: Vec<Value>,
        /// Exact identities corresponding to outputs.
        names: Option<Vec<ScaleKey>>,
    },
}
impl GgplotDiscretePalette {
    fn values_with_breaks(
        &self,
        keys: &[ScaleKey],
        breaks: Option<&[ScaleKey]>,
    ) -> ChartResult<Vec<Value>> {
        if let (
            Self::Manual {
                values,
                names: None,
            },
            Some(breaks),
        ) = (self, breaks)
        {
            let names = (0..values.len())
                .map(|i| breaks.get(i).cloned().unwrap_or(ScaleKey::Null))
                .collect();
            return Self::Manual {
                values: values.clone(),
                names: Some(names),
            }
            .values(keys);
        }
        self.values(keys)
    }
    fn values(&self, keys: &[ScaleKey]) -> ChartResult<Vec<Value>> {
        let n = keys.len();
        if let Self::Manual { values, names } = self {
            for value in values {
                value.validate()?;
            }
            if values.len() < n {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Insufficient values for the trained manual scale.",
                ));
            }
            if let Some(names) = names {
                if names.len() != values.len() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Manual names must match the number of outputs.",
                    ));
                }
                let indexes: std::collections::BTreeMap<_, _> = names
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(i, name)| (name, i))
                    .collect();
                return Ok(keys
                    .iter()
                    .map(|key| {
                        indexes
                            .get(key)
                            .map_or(Value::Missing, |i| values[*i].clone())
                    })
                    .collect());
            }
            return Ok(values[..n].to_vec());
        }
        self.count_values(n)
    }
    pub(super) fn count_values(&self, n: usize) -> ChartResult<Vec<Value>> {
        let paints = match self {
            Self::Shape { solid } => {
                let codes = if *solid {
                    [16, 17, 15, 3, 7, 8]
                } else {
                    [1, 2, 0, 3, 7, 8]
                };
                return Ok((0..n)
                    .map(|i| {
                        codes
                            .get(i)
                            .map_or(Value::Missing, |v| Value::Number(Number(f64::from(*v))))
                    })
                    .collect());
            }
            Self::LineType => {
                let patterns = [
                    "solid", "22", "42", "44", "13", "1343", "73", "2262", "12223242", "F282",
                    "F4448444", "224282F2", "F1",
                ];
                return Ok((0..n)
                    .map(|i| {
                        patterns
                            .get(i)
                            .map_or(Value::Missing, |v| Value::Text((*v).into()))
                    })
                    .collect());
            }
            Self::NumericRange { range, area } => {
                if range.iter().any(|v| !v.0.is_finite()) {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Ordinal numeric range must be finite.",
                    ));
                }
                let [a, b] = range.map(|v| if *area { v.0 * v.0 } else { v.0 });
                if !a.is_finite() || !b.is_finite() {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Ordinal area range overflows.",
                    ));
                }
                return Ok((0..n)
                    .map(|i| {
                        let t = if n <= 1 {
                            0.
                        } else {
                            i as f64 / (n - 1) as f64
                        };
                        let v = a * (1. - t) + b * t;
                        Value::Number(Number(if *area { v.sqrt() } else { v }))
                    })
                    .collect());
            }
            Self::Hue(spec) => {
                if n == 0 {
                    spec.colors(1)?;
                    vec![]
                } else {
                    spec.colors(n)?.into_iter().map(Some).collect()
                }
            }
            Self::Grey { start, end } => chromatic::ggplot::grey(n, *start, *end)?
                .into_iter()
                .map(Some)
                .collect(),
            Self::Brewer { id, reverse } => chromatic::ggplot::brewer(*id, n, *reverse)?,
            Self::Viridis {
                option,
                begin,
                end,
                reverse,
                alpha,
            } => chromatic::ggplot::ViridisPalette::new(*option)?
                .colors(n, *begin, *end, *reverse, *alpha)?
                .into_iter()
                .map(Some)
                .collect(),
            Self::Manual { .. } => {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Named/manual scales require category identities.",
                ));
            }
        };
        Ok(paints
            .into_iter()
            .map(|p| p.map_or(Value::Missing, |c| Value::Color(Paint::from(c).value())))
            .collect())
    }
}
/// Retained ggplot2 population/limit policy layered over an existing scale family.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GgplotScalePolicy {
    /// Palette values sampled at binned interval midpoints.
    Binned(Box<GgplotBinnedPolicy>),
    /// Finite eligible extent with optional partially authored endpoints.
    Continuous {
        /// Whether the most recent eligible population contained zero rows.
        #[serde(default)]
        empty_population: bool,
        /// Nonempty population with no finite transformed observations.
        #[serde(default)]
        nonfinite_population: bool,
        /// Raw-domain limits; missing endpoints use the trained extent.
        limits: Option<[Option<Number>; 2]>,
        /// Out-of-bounds mapping function.
        oob: GgplotOob,
    },
    /// Sorted character domains or explicit factor levels and unused-level behavior.
    Discrete {
        /// Whether the latest eligible population contained zero rows.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        empty_population: bool,
        /// Explicit limits replace the trained domain in authored order.
        limits: Option<Vec<ScaleKey>>,
        /// Factor levels in their original order.
        levels: Option<Vec<ScaleKey>>,
        /// Remove unobserved factor levels.
        drop: bool,
        /// Retain an observed missing level as the final guide entry.
        na_translate: bool,
        /// Palette compiled for the complete domain.
        palette: GgplotDiscretePalette,
    },
}
impl GgplotScalePolicy {
    pub(super) fn canonicalize(spec: &mut MappedScaleSpec) -> ChartResult<()> {
        if !matches!(
            spec.ggplot.as_deref(),
            Some(Self::Continuous { .. } | Self::Binned(_))
        ) {
            return Ok(());
        }
        // Adapt range interpolation to the shared ggplot normalizer once; do not
        // duplicate numeric family arithmetic in population or geometry consumers.
        if let ScaleFunctionSpec::Continuous(s) = &spec.function {
            if s.domain.len() != 2 || s.range.len() < 2 {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Trained ggplot continuous scales require two limits and at least two range values.",
                ));
            }
            spec.function = ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization: NormalizationSpec::Ggplot {
                    timestamp: None,
                    family: s.family,
                    domain: [s.domain[0], s.domain[1]],
                    reverse: false,
                    rescaler: GgplotRescaler::Range,
                },
                output: ScaleRangeFunction::Interpolate(
                    crate::interpolate::InterpolationSpec::Piecewise {
                        factory: s.factory.clone(),
                        values: s.range.clone(),
                    },
                ),
                unknown: s.unknown.clone(),
            });
        }
        if let ScaleFunctionSpec::Interpolated(s) = &mut spec.function
            && let NormalizationSpec::Sequential { family, domain, .. } = s.normalization
        {
            s.normalization = NormalizationSpec::Ggplot {
                timestamp: None,
                family,
                domain,
                reverse: false,
                rescaler: GgplotRescaler::Range,
            };
        }
        Ok(())
    }
    pub(super) fn validate_numbers(
        &self,
        spec: &mut MappedScaleSpec,
    ) -> ChartResult<NumericFamily> {
        if !matches!(self, Self::Continuous { .. } | Self::Binned(_)) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Discrete policy requires a key population.",
            ));
        }
        Self::canonicalize(spec)?;
        let family = match &spec.function {
            ScaleFunctionSpec::Continuous(s) => s.family,
            ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization:
                    NormalizationSpec::Sequential { family, .. }
                    | NormalizationSpec::Ggplot { family, .. },
                ..
            }) => *family,
            _ => {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Continuous population policy requires a two-endpoint continuous or sequential scale.",
                ));
            }
        };
        Ok(family)
    }
    pub(super) fn train_numbers(
        &self,
        spec: &mut MappedScaleSpec,
        values: &[Option<Number>],
    ) -> ChartResult<()> {
        let family = self.validate_numbers(spec)?;
        let limits = match self {
            Self::Continuous { limits, .. } => limits,
            Self::Binned(p) => &p.limits,
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Discrete policy requires a key population.",
                ));
            }
        };
        match spec.ggplot.as_deref_mut() {
            Some(Self::Continuous {
                empty_population, ..
            }) => *empty_population = values.is_empty(),
            Some(Self::Binned(policy)) => policy.empty_population = values.is_empty(),
            _ => {}
        }
        let mut extent = [f64::INFINITY, f64::NEG_INFINITY];
        for v in values.iter().flatten().map(|n| n.0).filter(|v| {
            v.is_finite()
                && (!matches!(family, NumericFamily::Log { .. }) || *v > 0.)
                && (!matches!(family,NumericFamily::Pow {exponent} if exponent.fract()!=0.)
                    || *v >= 0.)
        }) {
            extent[0] = extent[0].min(v);
            extent[1] = extent[1].max(v);
        }
        let nonfinite = !values.is_empty() && !extent[0].is_finite();
        match spec.ggplot.as_deref_mut() {
            Some(Self::Continuous {
                nonfinite_population,
                ..
            }) => *nonfinite_population = nonfinite,
            Some(Self::Binned(policy)) => policy.nonfinite_population = nonfinite,
            _ => {}
        }
        if !extent[0].is_finite() {
            extent = if matches!(family, NumericFamily::Log { .. }) {
                [1., 10.]
            } else {
                [0., 1.]
            };
        }
        let reverse = matches!(
            &spec.function,
            ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization: NormalizationSpec::Ggplot { reverse: true, .. },
                ..
            })
        );
        if reverse {
            extent.reverse();
        }
        if let Some(mut limits) = super::ggplot_continuous_guide::comparable_limits(*limits, family)
        {
            if let [Some(a), Some(b)] = limits {
                limits = if super::ggplot_continuous_guide::forward(family, reverse, a.0)
                    > super::ggplot_continuous_guide::forward(family, reverse, b.0)
                {
                    [Some(b), Some(a)]
                } else {
                    limits
                };
            }
            for (i, limit) in limits.into_iter().enumerate() {
                if let Some(value) = limit {
                    extent[i] = value.0;
                }
            }
        }
        let mut domain = extent.map(Number);
        if let Self::Binned(policy) = self {
            let hidden = matches!(spec.guide.as_deref(), Some(GgplotScaleGuide::Hidden));
            policy.validate_break_budget(4096)?;
            let prepared = if values.is_empty() && hidden {
                None
            } else {
                let resolved = policy.resolve(domain, family, reverse)?;
                // With no guide, R's first map captures trained limits before get_breaks()
                // extends them. Visible guides select breaks before the first mapping.
                if !hidden {
                    domain = resolved.0;
                }
                Some(resolved.1)
            };
            if let Some(next) = &mut spec.ggplot
                && let Self::Binned(next) = next.as_mut()
            {
                next.prepared_breaks = prepared;
            }
        }

        match &mut spec.function {
            ScaleFunctionSpec::Continuous(s) => s.domain = domain.to_vec(),
            ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization:
                    NormalizationSpec::Sequential { domain: d, .. }
                    | NormalizationSpec::Ggplot { domain: d, .. },
                ..
            }) => *d = domain,
            _ => unreachable!("validated scale family"),
        }
        Ok(())
    }
    pub(super) fn untrained_discrete_values(
        &self,
        guide: Option<&GgplotScaleGuide>,
    ) -> Option<ChartResult<Vec<Value>>> {
        let Self::Discrete {
            empty_population: true,
            limits: None,
            palette,
            ..
        } = self
        else {
            return None;
        };
        if matches!(
            palette,
            GgplotDiscretePalette::Manual { names: Some(_), .. }
        ) {
            return None;
        }
        // R supplies limits [0,1] only when mapping an untrained ordinary scale.
        // Keep this lazy: an empty chart does not ask a short manual palette for
        // two values, and its guide remains empty.
        Some(palette.values_with_breaks(
            &[ScaleKey::Text("0".into()), ScaleKey::Text("1".into())],
            match guide {
                Some(GgplotScaleGuide::Discrete(guide)) => guide.breaks.as_deref(),
                _ => None,
            },
        ))
    }
    pub(super) fn train_keys(
        &self,
        spec: &mut MappedScaleSpec,
        keys: &[ScaleKey],
    ) -> ChartResult<()> {
        let Self::Discrete {
            limits,
            levels,
            drop,
            na_translate,
            palette,
            ..
        } = self
        else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Continuous policy requires numbers.",
            ));
        };
        if let Some(Self::Discrete {
            empty_population, ..
        }) = spec.ggplot.as_deref_mut()
        {
            *empty_population = keys.is_empty();
        }
        let ScaleFunctionSpec::Ordinal(ordinal) = &mut spec.function else {
            return Err(error(
                DiagnosticCode::Validation,
                "Discrete policy requires the ordinal scale owner.",
            ));
        };
        let mut domain = discrete_domain(
            keys,
            limits.as_deref(),
            levels.as_deref(),
            *drop,
            *na_translate,
        );
        if limits.is_none()
            && let GgplotDiscretePalette::Manual {
                names: Some(names), ..
            } = palette
        {
            domain.retain(|key| *key == ScaleKey::Null || names.contains(key));
        }
        // Missing levels occupy authored guide slots without consuming a palette
        // color. Sample the nonmissing domain once, then restore those slots.
        let palette_domain = domain
            .iter()
            .filter(|key| **key != ScaleKey::Null)
            .cloned()
            .collect::<Vec<_>>();
        let mut values = palette
            .values_with_breaks(
                &palette_domain,
                match spec.guide.as_deref() {
                    Some(GgplotScaleGuide::Discrete(guide)) => guide.breaks.as_deref(),
                    _ => None,
                },
            )?
            .into_iter();
        let range = domain
            .iter()
            .map(|key| {
                if *key == ScaleKey::Null {
                    Value::Missing
                } else {
                    values
                        .next()
                        .expect("palette cardinality matches its domain")
                }
            })
            .collect();
        ordinal.domain = domain;
        ordinal.range = range;
        ordinal.unknown = OrdinalUnknown::Explicit(None);
        Ok(())
    }
    pub(super) fn population_bounds(
        &self,
        family: NumericFamily,
        reverse: bool,
    ) -> Option<[f64; 2]> {
        use super::ggplot_continuous_guide::{authored_bounds, finite_limits};
        match self {
            Self::Continuous {
                nonfinite_population: true,
                limits,
                ..
            } => Some(authored_bounds(
                *limits,
                family,
                reverse,
                [f64::INFINITY, f64::NEG_INFINITY],
            )),
            Self::Continuous {
                empty_population: true,
                limits,
                ..
            } if !finite_limits(*limits, family, reverse) => Some([0., 1.]),
            Self::Binned(policy) => policy.population_bounds(family, reverse),
            _ => None,
        }
    }
    pub(super) fn input(&self, input: Option<f64>, function: &ScaleFunctionSpec) -> Option<f64> {
        let oob = match self {
            Self::Continuous { oob, .. } => oob,
            Self::Binned(p) => &p.oob,
            _ => return input,
        };
        let (domain, family, reverse) = match function {
            ScaleFunctionSpec::Continuous(s) => {
                ([s.domain.first()?.0, s.domain.last()?.0], s.family, false)
            }
            ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization: NormalizationSpec::Sequential { domain, family, .. },
                ..
            }) => (domain.map(|n| n.0), *family, false),
            ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization:
                    NormalizationSpec::Ggplot {
                        domain,
                        family,
                        reverse,
                        ..
                    },
                ..
            }) => (domain.map(|n| n.0), *family, *reverse),
            _ => return input,
        };
        let x = input.filter(|x| {
            !x.is_nan()
                && (!matches!(family, NumericFamily::Log { .. }) || *x >= 0.)
                && (!matches!(family,NumericFamily::Pow {exponent} if exponent.fract()!=0.)
                    || *x >= 0.)
        })?;
        let timestamp = match function {
            ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization: NormalizationSpec::Ggplot { timestamp, .. },
                ..
            }) => *timestamp,
            _ => None,
        };
        let transform = |v| {
            timestamp.map_or_else(
                || super::ggplot_continuous_guide::forward(family, reverse, v),
                |context| context.absolute(v),
            )
        };
        let source_bounds = domain.map(transform);
        let td = self
            .population_bounds(family, reverse)
            .map(|bounds| bounds.map(|v| timestamp.map_or(v, |context| context.absolute(v))))
            .unwrap_or(source_bounds);
        let tx = transform(x);
        let mapped = oob.apply(Some(tx), td)?;
        if mapped == tx {
            Some(x)
        } else {
            let i = usize::from(mapped != td[0]);
            Some(if mapped == source_bounds[i] {
                domain[i]
            } else {
                super::ggplot_continuous_guide::inverse(family, reverse, mapped)
            })
        }
    }
}

/// Default ggplot2 color mapping with finite continuous training or sorted hue
/// categories. Missing output is `grey50`, retained independently of the palette.
pub fn ggplot_color_default(continuous: bool) -> ChartResult<ColorScale<Paint>> {
    let mapped = if continuous {
        MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
            normalization: NormalizationSpec::sequential(NumericFamily::Linear),
            output: ScaleRangeFunction::Interpolate(
                crate::interpolate::InterpolationSpec::GgplotPalette {
                    spec: chromatic::ggplot::PaletteSpec::Gradient {
                        colors: vec![Paint::from_css("#132B43")?, Paint::from_css("#56B1F7")?],
                        values: None,
                    },
                },
            ),
            unknown: Value::Missing,
        }))
        .with_ggplot(GgplotScalePolicy::Continuous {
            empty_population: false,
            nonfinite_population: false,
            limits: None,
            oob: GgplotOob::Censor,
        })?
    } else {
        MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default())).with_ggplot(
            GgplotScalePolicy::Discrete {
                empty_population: false,
                limits: None,
                levels: None,
                drop: true,
                na_translate: true,
                palette: GgplotDiscretePalette::Hue(Default::default()),
            },
        )?
    };
    Ok(ColorScale::Mapped {
        scale: mapped,
        missing: crate::theme::rgb(127, 127, 127).into(),
    })
}

/// Reference numeric aesthetic palette, independently selected from its input transform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GgplotNumericPalette {
    /// Square-root parameter interpolation from one to six.
    Size,
    /// Absolute square root, with zero mapping to zero and maximum to six.
    Area,
    /// Linear radius from one to six.
    Radius,
    /// Coverage from 0.1 to one.
    Alpha,
    /// Linear line width from one to six.
    Linewidth,
}
/// Default numeric output policy, ready for an ordinary numeric aesthetic encoding.
pub fn ggplot_numeric_default(kind: GgplotNumericPalette) -> ChartResult<MappedScaleSpec> {
    use GgplotNumericPalette::*;
    let range = match kind {
        Area => [0., 6.],
        Alpha => [0.1, 1.],
        _ => [1., 6.],
    };
    let exponent = if matches!(kind, Size | Area) { 0.5 } else { 1. };
    MappedScaleSpec::authored(ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
        normalization: NormalizationSpec::Ggplot {
            timestamp: None,
            family: NumericFamily::Linear,
            domain: [Number(0.), Number(1.)],
            reverse: false,
            rescaler: if kind == Area {
                GgplotRescaler::Maximum
            } else {
                GgplotRescaler::Range
            },
        },
        output: ScaleRangeFunction::Interpolate(
            crate::interpolate::InterpolationSpec::PowerRange {
                range: range.map(Number),
                exponent: Number(exponent),
                absolute: kind == Area,
            },
        ),
        unknown: Value::Missing,
    }))
    .with_ggplot(GgplotScalePolicy::Continuous {
        empty_population: false,
        nonfinite_population: false,
        limits: None,
        oob: GgplotOob::Censor,
    })
}

/// Ordinal defaults for the reference size, alpha and linewidth scale families.
pub fn ggplot_numeric_ordinal(kind: GgplotNumericPalette) -> ChartResult<MappedScaleSpec> {
    let (range, area) = match kind {
        GgplotNumericPalette::Size => ([2., 6.], true),
        GgplotNumericPalette::Alpha => ([0.1, 1.], false),
        GgplotNumericPalette::Linewidth => ([2., 6.], false),
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "The reference has no ordinal area or radius scale family.",
            ));
        }
    };
    MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default())).with_ggplot(
        GgplotScalePolicy::Discrete {
            empty_population: false,
            limits: None,
            levels: None,
            drop: true,
            na_translate: true,
            palette: GgplotDiscretePalette::NumericRange {
                range: range.map(Number),
                area,
            },
        },
    )
}

// Shared reference domain ordering, factor dropping and missing-level placement.
pub(super) fn discrete_domain(
    keys: &[ScaleKey],
    limits: Option<&[ScaleKey]>,
    levels: Option<&[ScaleKey]>,
    drop: bool,
    na_translate: bool,
) -> Vec<ScaleKey> {
    // Training an empty factor does not install its declared levels in R.
    if keys.is_empty() && limits.is_none() {
        return vec![];
    }
    let seen = keys.iter().cloned().collect::<BTreeSet<_>>();
    let mut domain = limits.map(<[ScaleKey]>::to_vec).unwrap_or_else(|| {
        levels.map_or_else(
            || {
                seen.iter()
                    .filter(|key| **key != ScaleKey::Null)
                    .cloned()
                    .collect()
            },
            |levels| {
                levels
                    .iter()
                    .filter(|key| !drop || seen.contains(key))
                    .cloned()
                    .collect()
            },
        )
    });
    if limits.is_none() {
        if !na_translate {
            domain.retain(|key| *key != ScaleKey::Null);
        } else if seen.contains(&ScaleKey::Null) && !domain.contains(&ScaleKey::Null) {
            domain.push(ScaleKey::Null);
        }
    }
    let mut distinct = BTreeSet::new();
    domain.retain(|key| distinct.insert(key.clone()));
    domain
}
