use super::*;
use crate::scene::Color;
use std::collections::BTreeSet;

/// Portable palette and domain policy; changing color never contributes positional domains.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ColorScale {
    /// Exact category mapping in authored/retained order, cycling a declared palette.
    Discrete {
        /// Optional fixed category order.
        domain: Option<Vec<String>>,
        /// Nonempty palette.
        palette: Vec<Color>,
        /// Null and unknown explicit-domain style.
        missing: Color,
    },
    /// Piecewise sRGB-byte interpolation along a finite numeric domain.
    Continuous {
        /// Exact data domain, descending allowed.
        domain: Bounds,
        /// At least two colors, equally spaced in parameter space.
        palette: Vec<Color>,
        /// Clamp to endpoint colors; false uses missing outside.
        clamp: bool,
        /// Null/nonfinite/outside style.
        missing: Color,
    },
}
/// Semantic legend metadata, independent of its eventual destination layout.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct ColorLegend {
    /// Declared semantic guide title, used when checking guide compatibility.
    pub title: Option<String>,
    /// Scale identity.
    pub id: crate::ScaleId,
    /// Exact category or numeric labels paired with colors.
    pub entries: Vec<(String, Color)>,
    /// Whether colors interpolate between stops.
    pub continuous: bool,
    /// Missing-value swatch.
    pub missing: Color,
}
impl ColorScale {
    /// Validate palette/domain, even when the current population is empty.
    pub fn validate(&self) -> ChartResult<()> {
        match self {
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
    range: Bounds,
    padding: f64,
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
            range,
            padding: options.padding,
        })
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
        Ok(self)
    }
    /// Destination range.
    pub fn range(&self) -> Bounds {
        self.range
    }
    /// Position of one known category; a singleton is always centered.
    pub fn center(&self, label: &str) -> ChartResult<Option<f64>> {
        let Some(i) = self.visible_domain().iter().position(|s| s == label) else {
            return Ok(None);
        };
        let t = if self.window.len() == 1 {
            0.5
        } else {
            (i as f64 + self.padding) / (self.window.len() as f64 - 1. + 2. * self.padding)
        };
        Ok(Some(super::linear::interpolate(self.range, t)?))
    }
    /// Nearest category within the destination range, with stable earlier-category ties.
    pub fn category_at(&self, p: f64) -> ChartResult<Option<&str>> {
        if !p.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Point lookup must be finite.",
            ));
        }
        if !self.range.contains(p) {
            return Ok(None);
        }
        let mut best = None;
        let mut distance = f64::INFINITY;
        for label in self.visible_domain() {
            let d =
                (super::linear::fraction(self.range, self.center(label)?.expect("known label"))?
                    - super::linear::fraction(self.range, p)?)
                .abs();
            if d < distance {
                best = Some(label.as_str());
                distance = d;
            }
        }
        Ok(best)
    }
    /// Category lookup only, with zero extent at each point.
    pub fn capabilities(&self) -> ScaleCapabilities {
        ScaleCapabilities {
            numeric_inverse: false,
            category_lookup: true,
        }
    }
}
