use super::*;
use crate::scene::Color;
use std::collections::BTreeSet;

/// Portable palette and domain policy; changing color never contributes positional domains.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::large_enum_variant,
    reason = "Per-scale authoring descriptor; optional guide adds one pointer, not per-mark storage. Keep the existing by-value mapped API."
)]
pub enum ColorScale<P = Color> {
    /// Typed D3-compatible mapping with independent population and output configuration.
    Mapped {
        /// Shared core scale; interpolated color outputs retain floating channels.
        scale: MappedScaleSpec,
        /// Missing source/output paint.
        missing: P,
    },
    /// Exact category mapping in authored/retained order, cycling a declared palette.
    Discrete {
        /// Optional fixed category order.
        domain: Option<Vec<String>>,
        /// Nonempty palette.
        palette: Vec<P>,
        /// Null and unknown explicit-domain style.
        missing: P,
    },
    /// Piecewise sRGB-byte interpolation along a finite numeric domain.
    Continuous {
        /// Exact data domain, descending allowed.
        domain: Bounds,
        /// At least two colors, equally spaced in parameter space.
        palette: Vec<P>,
        /// Clamp to endpoint colors; false uses missing outside.
        clamp: bool,
        /// Null/nonfinite/outside style.
        missing: P,
    },
}
/// One scale-space sample of a continuous color guide's actual mapping.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct ColorGuideSample {
    /// Transformed value used by the reference mapping.
    pub value: crate::interpolate::Number,
    /// Resolved palette output at this sample.
    pub color: Color,
}
/// One directed transformed interval of a stepped color guide.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct ColorGuideStep {
    /// First boundary in guide order, before destination placement.
    pub start: crate::interpolate::Number,
    /// Second boundary in guide order.
    pub end: crate::interpolate::Number,
    /// Resolved palette output for this interval.
    pub color: Color,
}
/// Semantic legend metadata, independent of its eventual destination layout.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct ColorLegend {
    /// Declared semantic guide title, used when checking guide compatibility; empty omits it.
    pub title: Option<String>,
    /// Scale identity.
    pub id: crate::ScaleId,
    /// Exact category or numeric labels paired with colors.
    pub entries: Vec<(String, Color)>,
    /// Reference numeric break candidates and labels before guide composition.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub numeric_breaks: Vec<super::GgplotContinuousGuideEntry>,
    /// Default reference colorbar samples, retained for guide composition.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub colorbar: Vec<ColorGuideSample>,
    /// Prepared stepped guide cells; colors share the scale's interval palette batch.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub colorsteps: Vec<ColorGuideStep>,
    /// Ordinal step-key positions aligned with numeric breaks; absent keys are censored.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub colorstep_positions: Vec<Option<crate::interpolate::Number>>,
    /// Whether colors interpolate between stops.
    pub continuous: bool,
    /// Missing-value swatch.
    pub missing: Color,
    /// Actual classifier intervals, absent for ordinary categorical and continuous guides.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub intervals: Vec<GuideInterval>,
    /// Exact mapped scale contract for guide compatibility, including interpolation and training.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mapping: Option<MappedScaleSpec>,
    /// Independently authored diverging midpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub midpoint: Option<crate::interpolate::Number>,
}
impl<P> ColorScale<P> {
    /// Whether source inputs are category keys.
    pub fn is_categorical(&self) -> bool {
        match self {
            Self::Discrete { .. } => true,
            Self::Continuous { .. } => false,
            Self::Mapped { scale, .. } => scale.categorical(),
        }
    }
    /// Validate palettes and explicitly installed interpolation factories.
    pub fn validate_with_registry(
        &self,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<()> {
        if let Self::Mapped { scale, .. } = self {
            scale.validate_definition_with_registry(registry)
        } else {
            self.validate()
        }
    }
    /// Validate palette/domain, even when the current population is empty.
    pub fn validate(&self) -> ChartResult<()> {
        match self {
            Self::Mapped { scale, .. } => {
                scale
                    .validate_definition_with_registry(&crate::grammar::ExtensionRegistry::new())?;
            }
            Self::Discrete {
                domain, palette, ..
            } => {
                if palette.is_empty()
                    || domain
                        .as_ref()
                        .is_some_and(|v| v.iter().collect::<BTreeSet<_>>().len() != v.len())
                {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Discrete color needs a nonempty palette and distinct domain.",
                    ));
                }
            }
            Self::Continuous {
                domain, palette, ..
            } => {
                Bounds::new(domain.start(), domain.end())?.distinct()?;
                if palette.len() < 2 {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Continuous color needs at least two palette entries.",
                    ));
                }
            }
        }
        Ok(())
    }
}
impl ColorScale {
    /// Resolve one categorical value; unknown/null values get the explicit missing style.
    pub fn categorical(&self, value: Option<&str>, first_seen: &[String]) -> ChartResult<Color> {
        self.validate()?;
        let Self::Discrete {
            domain,
            palette,
            missing,
        } = self
        else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Categorical color requires a discrete scale.",
            ));
        };
        Ok(value
            .and_then(|v| {
                domain
                    .as_deref()
                    .unwrap_or(first_seen)
                    .iter()
                    .position(|s| s == v)
            })
            .map_or(*missing, |i| palette[i % palette.len()]))
    }
    /// Resolve one numeric value, never changing the original source value.
    pub fn numeric(&self, value: Option<f64>) -> ChartResult<Color> {
        self.validate()?;
        if let Self::Mapped { scale, missing } = self {
            return MappedScale::for_colors(scale.clone())?.color(value, None, *missing);
        }
        self.numeric_validated(value)
    }
    pub(crate) fn numeric_validated(&self, value: Option<f64>) -> ChartResult<Color> {
        let Self::Continuous {
            domain,
            palette,
            clamp,
            missing,
        } = self
        else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Numeric color requires a continuous scale.",
            ));
        };
        let Some(v) = value.filter(|v| v.is_finite()) else {
            return Ok(*missing);
        };
        if !clamp && !domain.contains(v) {
            return Ok(*missing);
        }
        let t = super::linear::fraction(*domain, v.clamp(domain.minimum(), domain.maximum()))?
            .clamp(0., 1.)
            * (palette.len() - 1) as f64;
        let i = (t.floor() as usize).min(palette.len() - 2);
        let f = t - i as f64;
        let (a, b) = (palette[i], palette[i + 1]);
        let lerp = |a: u8, b: u8| ((1. - f) * f64::from(a) + f * f64::from(b)).round() as u8;
        Ok(Color {
            red: lerp(a.red, b.red),
            green: lerp(a.green, b.green),
            blue: lerp(a.blue, b.blue),
            alpha: lerp(a.alpha, b.alpha),
        })
    }
    /// Return exact guide identity and swatches/stops for compatible guide composition.
    pub fn legend(&self, id: crate::ScaleId, first_seen: &[String]) -> ChartResult<ColorLegend> {
        self.validate()?;
        let (continuous, missing, entries) = match self {
            Self::Mapped { scale, missing } => {
                return MappedScale::for_colors(scale.clone())?.legend(id, *missing);
            }
            Self::Discrete {
                domain,
                palette,
                missing,
            } => (
                false,
                *missing,
                domain
                    .as_deref()
                    .unwrap_or(first_seen)
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (s.clone(), palette[i % palette.len()]))
                    .collect(),
            ),
            Self::Continuous {
                domain,
                palette,
                missing,
                ..
            } => (
                true,
                *missing,
                palette
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        Ok((
                            format_number(super::linear::interpolate(
                                *domain,
                                i as f64 / (palette.len() - 1) as f64,
                            )?),
                            *c,
                        ))
                    })
                    .collect::<ChartResult<_>>()?,
            ),
        };
        Ok(ColorLegend {
            title: None,
            id,
            continuous,
            missing,
            entries,
            numeric_breaks: vec![],
            colorbar: vec![],
            colorsteps: vec![],
            colorstep_positions: vec![],
            intervals: vec![],
            midpoint: None,
            mapping: None,
        })
    }
}
/// Categorical point centers. Padding is measured in point steps at each end.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PointOptions {
    /// Exact domain or retained first-seen catalog.
    pub domain: Option<Vec<String>>,
    /// Nonnegative outer step padding.
    pub padding: f64,
}
impl Default for PointOptions {
    fn default() -> Self {
        Self {
            domain: None,
            padding: 0.5,
        }
    }
}
/// Immutable point mapping with category lookup, no numeric inverse or fabricated band width.
#[derive(Clone, Debug, PartialEq)]
pub struct PointScale {
    labels: Vec<String>,
    window: std::ops::Range<usize>,
    index: std::collections::BTreeMap<String, usize>,
    spacing: super::spacing::Spacing,
}
impl PointScale {
    /// Resolve stable centers in either destination direction.
    pub fn resolve(
        first_seen: &[String],
        options: &PointOptions,
        range: Bounds,
    ) -> ChartResult<Self> {
        let labels = options.domain.as_deref().unwrap_or(first_seen);
        if !options.padding.is_finite()
            || options.padding < 0.
            || !(labels.len() as f64 + 2. * options.padding).is_finite()
            || labels.iter().collect::<BTreeSet<_>>().len() != labels.len()
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Point scale needs finite nonnegative padding and distinct categories.",
            ));
        }
        range.distinct()?;
        Ok(Self {
            labels: labels.to_vec(),
            window: 0..labels.len(),
            index: labels
                .iter()
                .enumerate()
                .map(|(i, k)| (k.clone(), i))
                .collect(),
            spacing: super::spacing::Spacing::new(
                labels.len(),
                range,
                &super::BandSpec::<String> {
                    domain: None,
                    padding_inner: 1.,
                    padding_outer: options.padding,
                    align: 0.5,
                    round: false,
                },
                super::ScaleCompatibility::Legacy,
                true,
            )?,
        })
    }
    /// Resolve explicit D3 point alignment, rounding and zero-default padding.
    pub fn resolve_d3(
        first_seen: &[String],
        options: &super::PointSpec,
        range: Bounds,
    ) -> ChartResult<Self> {
        let prepared = super::CategoryScale::point(
            super::PointSpec {
                domain: Some(options.domain.as_deref().unwrap_or(first_seen).to_vec()),
                ..options.clone()
            },
            range,
        )?;
        let labels = prepared.domain().to_vec();
        Ok(Self {
            index: labels
                .iter()
                .enumerate()
                .map(|(i, k)| (k.clone(), i))
                .collect(),
            window: 0..labels.len(),
            spacing: super::spacing::Spacing::new(
                labels.len(),
                range,
                prepared.spec(),
                super::ScaleCompatibility::D3,
                true,
            )?,
            labels,
        })
    }
    /// Project a numeric minor candidate in the reference category-index space.
    pub(crate) fn reference_viewport(&self) -> Option<[crate::interpolate::Number; 2]> {
        self.spacing.reference_viewport()
    }
    pub(crate) fn reference_minor(&self, value: f64) -> ChartResult<Option<f64>> {
        self.spacing.reference_minor(value)
    }
    pub(crate) fn with_reference_expansion(
        mut self,
        expansion: super::GgplotExpansion,
        limits: Option<&[crate::interpolate::Number]>,
        observed: &[String],
    ) -> ChartResult<Self> {
        let continuous = super::spacing::observed_extent(
            observed
                .iter()
                .filter_map(|label| self.index.get(label).copied()),
            !observed.is_empty(),
        );
        self.spacing = self
            .spacing
            .with_reference_expansion(expansion, limits, continuous)?;
        Ok(self)
    }
    /// Nonnegative interval between neighboring points.
    pub fn step(&self) -> f64 {
        self.spacing.step()
    }
    /// Points have zero bandwidth.
    pub fn bandwidth(&self) -> f64 {
        0.
    }
    /// Exact category order.
    pub fn domain(&self) -> &[String] {
        &self.labels
    }
    /// Current category window, preserving the complete trained domain separately.
    pub fn visible_domain(&self) -> &[String] {
        &self.labels[self.window.clone()]
    }
    /// Restrict presentation to an inclusive stable-label window without changing training.
    pub fn with_window(mut self, first: &str, last: &str) -> ChartResult<Self> {
        self.window = super::category_window(&self.labels, first, last)?;
        self.spacing = self.spacing.resize(self.window.len())?;
        Ok(self)
    }
    /// Destination range.
    pub fn range(&self) -> Bounds {
        self.spacing.range
    }
    /// Position of one known category; a singleton is always centered.
    pub fn center(&self, label: &str) -> ChartResult<Option<f64>> {
        let Some(&i) = self.index.get(label) else {
            return Ok(None);
        };
        if !self.window.contains(&i) {
            return Ok(None);
        }
        self.spacing.center(i - self.window.start)
    }
    /// Nearest category within the destination range, with stable earlier-category ties.
    pub fn category_at(&self, p: f64) -> ChartResult<Option<&str>> {
        Ok(self
            .spacing
            .point_at(p)?
            .map(|i| self.labels[self.window.start + i].as_str()))
    }
    /// Category lookup only, with zero extent at each point.
    pub fn capabilities(&self) -> ScaleCapabilities {
        ScaleCapabilities {
            numeric_inverse: false,
            category_lookup: self.spacing.supports_lookup(),
        }
    }
}

impl<P> ColorScale<P> {
    /// Transform palette inputs while retaining domain, order and outside policies.
    pub fn map_colors<Q>(self, mut map: impl FnMut(P) -> Q) -> ColorScale<Q> {
        match self {
            Self::Mapped { scale, missing } => ColorScale::Mapped {
                scale,
                missing: map(missing),
            },
            Self::Discrete {
                domain,
                palette,
                missing,
            } => ColorScale::Discrete {
                domain,
                palette: palette.into_iter().map(&mut map).collect(),
                missing: map(missing),
            },
            Self::Continuous {
                domain,
                palette,
                clamp,
                missing,
            } => ColorScale::Continuous {
                domain,
                palette: palette.into_iter().map(&mut map).collect(),
                clamp,
                missing: map(missing),
            },
        }
    }
}
/// Prepared palette shared by mapped marks and guide stops; no per-row parsing or factories.
#[derive(Clone, Debug)]
pub struct PreparedColorScale {
    bytes: ColorScale,
    ramps: Option<Vec<crate::interpolate::ColorInterpolator>>,
    mapped: Option<MappedScale>,
    missing: crate::color::Paint,
}
impl ColorScale<crate::color::Paint> {
    /// Compile floating RGB ramps once, preserving exact legacy byte interpolation.
    pub fn prepare(&self) -> ChartResult<PreparedColorScale> {
        self.prepare_with_registry(&crate::grammar::ExtensionRegistry::new())
    }
    /// Prepare color and legend consumers using the same captured native factory registry.
    pub fn prepare_with_registry(
        &self,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<PreparedColorScale> {
        self.validate_with_registry(registry)?;
        let ramps = if let Self::Continuous { palette, .. } = self {
            if palette.iter().any(|p| p.is_floating()) {
                Some(
                    palette
                        .windows(2)
                        .map(|p| {
                            crate::interpolate::ColorInterpolator::new(
                                crate::interpolate::ColorRoute::Rgb,
                                p[0].value(),
                                p[1].value(),
                                None,
                            )
                        })
                        .collect::<ChartResult<_>>()?,
                )
            } else {
                None
            }
        } else {
            None
        };
        Ok(PreparedColorScale {
            bytes: self.clone().map_colors(crate::color::Paint::resolve),
            ramps,
            missing: match self {
                Self::Mapped { missing, .. }
                | Self::Continuous { missing, .. }
                | Self::Discrete { missing, .. } => *missing,
            },
            mapped: if let Self::Mapped { scale, .. } = self {
                Some(MappedScale::for_colors_with_registry(
                    scale.clone(),
                    registry,
                )?)
            } else {
                None
            },
        })
    }
    /// Whether palette or missing paint requires the floating-color capability.
    pub fn has_floating(&self) -> bool {
        let (palette, missing) = match self {
            Self::Mapped { .. } => return true,
            Self::Discrete {
                palette, missing, ..
            }
            | Self::Continuous {
                palette, missing, ..
            } => (palette, missing),
        };
        missing.is_floating() || palette.iter().any(|p| p.is_floating())
    }
}
impl PreparedColorScale {
    pub(crate) fn palette_paints(
        &self,
        inputs: &[Option<f64>],
    ) -> ChartResult<Option<super::ggplot_palette::PaletteBatch<Option<crate::color::Paint>>>> {
        self.mapped
            .as_ref()
            .map_or(Ok(None), |scale| scale.palette_paints(inputs, self.missing))
    }
    pub(crate) fn missing_paint(&self, input: Option<f64>, key: Option<&ScaleKey>) -> bool {
        self.mapped
            .as_ref()
            .is_some_and(|m| m.missing_paint(input, key))
    }
    /// Map a typed category, retaining floating color channels until final paint lowering.
    pub fn category_paint(&self, key: Option<&ScaleKey>) -> ChartResult<crate::color::Paint> {
        if let Some(scale) = &self.mapped {
            return scale.paint(None, key, self.missing);
        }
        Err(error(
            DiagnosticCode::SchemaConflict,
            "Typed category color requires a mapped scale.",
        ))
    }
    /// Map a typed category to final byte paint.
    pub fn category(&self, key: Option<&ScaleKey>) -> ChartResult<Color> {
        self.category_paint(key).map(crate::color::Paint::resolve)
    }
    /// Prepared categorical palette and domain; continuous callers use `numeric`.
    pub fn byte_scale(&self) -> &ColorScale {
        &self.bytes
    }
    /// Map a numeric value to final byte paint.
    pub fn numeric(&self, value: Option<f64>) -> ChartResult<Color> {
        self.numeric_paint(value).map(crate::color::Paint::resolve)
    }
    /// Map a numeric value using the prepared ramp and exact outside/null policy.
    pub fn numeric_paint(&self, value: Option<f64>) -> ChartResult<crate::color::Paint> {
        if let Some(scale) = &self.mapped {
            return scale.paint(value, None, self.missing);
        }
        let Some(ramps) = &self.ramps else {
            return self.bytes.numeric_validated(value).map(Into::into);
        };
        let ColorScale::Continuous { domain, clamp, .. } = &self.bytes else {
            unreachable!("ramps require continuous scale")
        };
        let Some(v) = value.filter(|v| v.is_finite()) else {
            return Ok(self.missing);
        };
        if !clamp && !domain.contains(v) {
            return Ok(self.missing);
        }
        let t = super::linear::fraction(*domain, v.clamp(domain.minimum(), domain.maximum()))?
            .clamp(0., 1.)
            * ramps.len() as f64;
        let i = (t.floor() as usize).min(ramps.len() - 1);
        Ok(ramps[i].sample_color(t - i as f64)?.into())
    }
    /// Guide stops sample the same prepared floating ramp as marks.
    pub fn legend(&self, id: crate::ScaleId, first_seen: &[String]) -> ChartResult<ColorLegend> {
        if let (Some(scale), ColorScale::Mapped { missing, .. }) = (&self.mapped, &self.bytes) {
            return scale.legend(id, *missing);
        }
        let mut legend = self.bytes.legend(id, first_seen)?;
        if let Some(ramps) = &self.ramps {
            let n = legend.entries.len();
            for (i, (_, paint)) in legend.entries.iter_mut().enumerate() {
                let index = i.min(n - 2);
                *paint = ramps[index]
                    .sample_color(if i == n - 1 { 1. } else { 0. })?
                    .to_paint();
            }
        }
        Ok(legend)
    }
}
