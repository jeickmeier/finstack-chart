//! ggplot2 4.0.3 / scales 1.4.0 palette policies over shared color and catalog owners.
use super::{ChartResult, Color, DiagnosticCode, SchemeId, error, scheme};
use crate::color::{ColorValue, d65};

/// Authored polar-Luv hue palette. Angles use degrees; chroma and luminance use
/// the D65 CIELUV convention used by scales, independently of D3's D50 HCL.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HuePalette {
    /// Inclusive hue interval before the full-circle endpoint adjustment.
    pub h: [f64; 2],
    /// Chroma.
    pub chroma: f64,
    /// Luminance.
    pub luminance: f64,
    /// Offset applied to every hue, modulo 360 degrees.
    pub start: f64,
    /// Reverse the completed palette.
    pub reverse: bool,
}
impl Default for HuePalette {
    fn default() -> Self {
        Self {
            h: [15., 375.],
            chroma: 100.,
            luminance: 65.,
            start: 0.,
            reverse: false,
        }
    }
}
fn cardinality(n: usize) -> ChartResult<()> {
    if n > 65_536 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Palette exceeds 65536 entries.",
        ));
    }
    Ok(())
}
impl HuePalette {
    /// Sample the complete palette; zero colors is rejected by the reference.
    pub fn colors(self, n: usize) -> ChartResult<Vec<Color>> {
        cardinality(n)?;
        if n == 0
            || !self
                .h
                .into_iter()
                .chain([self.chroma, self.luminance, self.start])
                .all(f64::is_finite)
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Hue palette requires finite arguments and at least one color.",
            ));
        }
        let mut end = self.h[1];
        if (end - self.h[0]).rem_euclid(360.) < 1. {
            end -= 360. / n as f64;
        }
        let mut result = (0..n)
            .map(|i| {
                let t = if n == 1 {
                    0.
                } else {
                    i as f64 / (n - 1) as f64
                };
                let h = (self.h[0] + t * (end - self.h[0]) + self.start).rem_euclid(360.);
                ColorValue::from(d65::from_hcl(h, self.chroma, self.luminance)).to_paint()
            })
            .collect::<Vec<_>>();
        if self.reverse {
            result.reverse();
        }
        Ok(result)
    }
}
/// Gamma-2.2 grey palette. A single sample is the start value; zero is empty.
pub fn grey(n: usize, start: f64, end: f64) -> ChartResult<Vec<Color>> {
    cardinality(n)?;
    if ![start, end]
        .iter()
        .all(|x| x.is_finite() && (0. ..=1.).contains(x))
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Grey endpoints must be in [0,1].",
        ));
    }
    let a = pxfm::f_pow(start, 2.2);
    let b = pxfm::f_pow(end, 2.2);
    Ok((0..n)
        .map(|i| {
            let t = if n == 1 {
                0.
            } else {
                i as f64 / (n - 1) as f64
            };
            let v = 255. * pxfm::f_pow(a + (b - a) * t, 1. / 2.2);
            ColorValue::from(crate::color::rgb(v, v, v)).to_paint()
        })
        .collect())
}
/// ColorBrewer's size policy over the existing exact tables. Requests below
/// three truncate the three-color table; overflow is retained as missing paint.
pub fn brewer(id: SchemeId, n: usize, reverse: bool) -> ChartResult<Vec<Option<Color>>> {
    cardinality(n)?;
    let info = id.info();
    if ![
        "BrBG", "PiYG", "PRGn", "PuOr", "RdBu", "RdGy", "RdYlBu", "RdYlGn", "Spectral", "Accent",
        "Dark2", "Paired", "Pastel1", "Pastel2", "Set1", "Set2", "Set3", "Blues", "BuGn", "BuPu",
        "GnBu", "Greens", "Greys", "Oranges", "OrRd", "PuBu", "PuBuGn", "PuRd", "Purples", "RdPu",
        "Reds", "YlGn", "YlGnBu", "YlOrBr", "YlOrRd",
    ]
    .contains(&id.name())
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Palette is not a ColorBrewer family.",
        ));
    }
    let max = *info.sizes.last().expect("built-in palette has sizes");
    let table = scheme(
        id,
        if info.sizes.len() == 1 {
            None
        } else {
            Some(n.clamp(3, max))
        },
        id.name() == "PuOr",
    )?;
    let mut result = (0..n).map(|i| table.get(i).copied()).collect::<Vec<_>>();
    if reverse {
        result.reverse();
    }
    Ok(result)
}
/// Compiled D65 Lab gradient with optional uneven stops and independent alpha.
#[derive(Clone, Debug)]
pub struct Gradient {
    positions: Vec<[f64; 2]>,
    colors: Vec<([f64; 3], f64)>,
    invalid_positions: bool,
}
impl Gradient {
    /// Construct from parsed colors. Explicit positions are sorted, and duplicate
    /// positions use the mean of their original uniform palette coordinates.
    pub fn new(colors: &[ColorValue], stops: Option<&[f64]>) -> ChartResult<Self> {
        Self::compile(colors, stops, true)
    }
    fn compile(colors: &[ColorValue], stops: Option<&[f64]>, strict: bool) -> ChartResult<Self> {
        cardinality(colors.len())?;
        if colors.is_empty() {
            return Err(error(
                DiagnosticCode::Validation,
                "Gradient requires a color.",
            ));
        }
        let colors = colors
            .iter()
            .map(|color| {
                let rgb = color.rgb();
                // The reference transparent name is transparent white. Explicit byte
                // colors retain their hidden RGB; only fully undefined zero-alpha RGB
                // takes this named-transparent fallback.
                if rgb.opacity == 0. && [rgb.r, rgb.g, rgb.b].iter().all(|v| v.is_nan()) {
                    return Ok(ColorValue::from(
                        crate::color::rgb(255., 255., 255.).opacity(0.),
                    ));
                }
                if ![rgb.r, rgb.g, rgb.b, rgb.opacity]
                    .iter()
                    .all(|v| v.is_finite())
                    || !(0. ..=1.).contains(&rgb.opacity)
                {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Gradient colors require finite RGB and alpha in [0,1].",
                    ));
                }
                Ok(*color)
            })
            .collect::<ChartResult<Vec<_>>>()?;
        let mut positions = Vec::new();
        let mut invalid_positions = false;
        if let Some(stops) = stops {
            cardinality(stops.len())?;
            let pairs = stops
                .iter()
                .enumerate()
                // pal_gradient_n remaps against seq(0, 1, length(values)),
                // independently of the number of color anchors.
                .map(|(i, x)| [*x, i as f64 / stops.len().saturating_sub(1) as f64])
                // approxfun removes NA pairs after assigning the original coordinates.
                .filter(|p| !p[0].is_nan())
                .collect::<Vec<_>>();
            positions = crate::scales::ggplot_approx::knots(pairs);
            if positions.len() < 2 {
                if strict {
                    return Err(Self::position_error());
                }
                invalid_positions = true;
            }
        }
        let colors = colors
            .iter()
            .map(|c| (d65::lab(c.rgb()), c.opacity()))
            .collect();
        Ok(Self {
            positions,
            colors,
            invalid_positions,
        })
    }
    fn position_error() -> crate::Diagnostic {
        error(
            DiagnosticCode::Validation,
            "Explicit gradient positions require two distinct nonmissing values.",
        )
    }
    /// Sample a normalized value. Missing/outside positions are missing; a
    /// single-color gradient without explicit positions is constant for finite input.
    pub fn sample(&self, t: f64) -> Option<Color> {
        if self.invalid_positions || t.is_nan() {
            return None;
        }
        let t = if self.positions.is_empty() {
            t
        } else {
            if t < self.positions[0][0] || t > self.positions.last()?[0] {
                return None;
            }
            let i = self
                .positions
                .partition_point(|p| p[0] <= t)
                .saturating_sub(1)
                .min(self.positions.len() - 2);
            let [x, a] = self.positions[i];
            let [y, b] = self.positions[i + 1];
            if t == x {
                a
            } else if t == y {
                b
            } else {
                a + (b - a) * ((t - x) / (y - x))
            }
        };
        if t.is_nan() {
            return None;
        }
        let (lab, alpha) = if self.colors.len() == 1 {
            self.colors[0]
        } else {
            if !(0. ..=1.).contains(&t) {
                return None;
            }
            let q = t * (self.colors.len() - 1) as f64;
            let i = (q.floor() as usize).min(self.colors.len() - 2);
            let f = q - i as f64;
            let (a, aa) = self.colors[i];
            let (b, ba) = self.colors[i + 1];
            (
                std::array::from_fn(|j| a[j] + (b[j] - a[j]) * f),
                aa + (ba - aa) * f,
            )
        };
        let mut paint = ColorValue::from(d65::from_lab(lab).opacity(alpha)).to_paint();
        paint.alpha = d65::alpha_byte(alpha);
        Some(paint)
    }
}

/// All eight viridisLite reference options; letters are retained as wire aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ViridisOption {
    /// Magma (A).
    #[serde(alias = "A")]
    Magma,
    /// Inferno (B).
    #[serde(alias = "B")]
    Inferno,
    /// Plasma (C).
    #[serde(alias = "C")]
    Plasma,
    /// Viridis (D).
    #[serde(alias = "D")]
    Viridis,
    /// Cividis (E).
    #[serde(alias = "E")]
    Cividis,
    /// Rocket (F).
    #[serde(alias = "F")]
    Rocket,
    /// Mako (G).
    #[serde(alias = "G")]
    Mako,
    /// Turbo (H).
    #[serde(alias = "H")]
    Turbo,
}
/// Prepared viridisLite 0.4.3 palette, sharing the scalar spline and D65 owners.
#[derive(Clone, Debug)]
pub struct ViridisPalette {
    channels: [crate::interpolate::CubicSpline; 3],
}
impl ViridisPalette {
    /// Convert the reference seed table once and retain three immutable splines.
    pub fn new(option: ViridisOption) -> ChartResult<Self> {
        use super::ggplot_data as d;
        use ViridisOption::*;
        let table: &[u32] = match option {
            Magma => super::data::lookup(super::InterpolatorId::Magma).expect("shared seed table"),
            Inferno => {
                super::data::lookup(super::InterpolatorId::Inferno).expect("shared seed table")
            }
            Plasma => {
                super::data::lookup(super::InterpolatorId::Plasma).expect("shared seed table")
            }
            Viridis => {
                super::data::lookup(super::InterpolatorId::Viridis).expect("shared seed table")
            }
            Cividis => &d::E,
            Rocket => &d::F,
            Mako => &d::G,
            Turbo => &d::H,
        };
        let lab = table
            .iter()
            .map(|v| {
                d65::device_lab(crate::color::rgb(
                    f64::from((v >> 16) as u8),
                    f64::from((v >> 8) as u8),
                    f64::from(*v as u8),
                ))
            })
            .collect::<Vec<_>>();
        let channel =
            |j| crate::interpolate::CubicSpline::new(&lab.iter().map(|v| v[j]).collect::<Vec<_>>());
        Ok(Self {
            channels: [channel(0)?, channel(1)?, channel(2)?],
        })
    }
    /// Sample an inclusive interval. Reversal swaps interval endpoints before
    /// sampling, so a single reversed color comes from `end`.
    pub fn colors(
        &self,
        n: usize,
        begin: f64,
        end: f64,
        reverse: bool,
        alpha: f64,
    ) -> ChartResult<Vec<Color>> {
        cardinality(n)?;
        if ![begin, end, alpha]
            .iter()
            .all(|v| v.is_finite() && (0. ..=1.).contains(v))
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Viridis endpoints and alpha must be in [0,1].",
            ));
        }
        let (begin, end) = if reverse { (end, begin) } else { (begin, end) };
        (0..n)
            .map(|i| {
                let t = begin
                    + (end - begin)
                        * if n == 1 {
                            0.
                        } else {
                            i as f64 / (n - 1) as f64
                        };
                let lab = [
                    self.channels[0].sample(t)?,
                    self.channels[1].sample(t)?,
                    self.channels[2].sample(t)?,
                ];
                Ok(ColorValue::from(d65::from_device_lab(lab).opacity(alpha)).to_paint())
            })
            .collect()
    }
}

/// Portable continuous palette recipe, independent of domain training and OOB.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PaletteSpec {
    /// Lab interpolation through a fixed count palette, as used by distiller/viridis_c.
    CountGradient {
        /// Existing count palette owner, evaluated once at compilation.
        palette: Box<crate::scales::GgplotDiscretePalette>,
        /// Number of anchor colors; reference constructors use seven or six.
        count: usize,
        /// Independent remapping positions, including reference NA and infinities.
        values: Option<Vec<crate::interpolate::Number>>,
    },
    /// Piecewise Lab interpolation with optional uneven positions.
    Gradient {
        /// Colors retain hidden RGB channels even when alpha is zero.
        colors: Vec<crate::color::Paint>,
        /// Independent remapping positions; omitted samples the color ramp directly.
        values: Option<Vec<crate::interpolate::Number>>,
    },
    /// Continuous sampling of the viridisLite spline over an interval.
    Viridis {
        /// Reference map identity.
        option: ViridisOption,
        /// First sampled position.
        begin: f64,
        /// Last sampled position.
        end: f64,
        /// Swap begin and end before evaluation.
        reverse: bool,
        /// Coverage replacing the source palette's opaque coverage.
        alpha: f64,
    },
}
/// Compiled palette recipe, reusable by ordinary interpolation and mapped scales.
#[derive(Clone, Debug)]
pub struct PaletteRamp {
    kernel: PaletteKernel,
}
#[derive(Clone, Debug)]
enum PaletteKernel {
    Gradient(Gradient),
    Viridis {
        palette: ViridisPalette,
        begin: f64,
        end: f64,
        alpha: f64,
    },
}
impl PaletteRamp {
    /// Validate and compile the recipe once, without retaining external resources.
    pub fn new(spec: &PaletteSpec) -> ChartResult<Self> {
        let kernel = match spec {
            PaletteSpec::CountGradient {
                palette,
                count,
                values,
            } => {
                cardinality(*count)?;
                let colors = palette
                    .count_values(*count)?
                    .into_iter()
                    .map(|value| match value {
                        crate::interpolate::Value::Color(color) => Ok(color),
                        crate::interpolate::Value::Text(text) => {
                            crate::color::parse_r(&text).map(|p| p.value())
                        }
                        _ => Err(error(
                            DiagnosticCode::Validation,
                            "A count gradient requires paint anchors.",
                        )),
                    })
                    .collect::<ChartResult<Vec<_>>>()?;
                PaletteKernel::Gradient(Gradient::compile(
                    &colors,
                    values
                        .as_ref()
                        .map(|v| v.iter().map(|n| n.0).collect::<Vec<_>>())
                        .as_deref(),
                    false,
                )?)
            }
            PaletteSpec::Gradient { colors, values } => PaletteKernel::Gradient(Gradient::compile(
                &colors.iter().map(|c| c.value()).collect::<Vec<_>>(),
                values
                    .as_ref()
                    .map(|v| v.iter().map(|n| n.0).collect::<Vec<_>>())
                    .as_deref(),
                false,
            )?),
            PaletteSpec::Viridis {
                option,
                begin,
                end,
                reverse,
                alpha,
            } => {
                let palette = ViridisPalette::new(*option)?;
                palette.colors(0, *begin, *end, *reverse, *alpha)?;
                let (begin, end) = if *reverse {
                    (*end, *begin)
                } else {
                    (*begin, *end)
                };
                PaletteKernel::Viridis {
                    palette,
                    begin,
                    end,
                    alpha: *alpha,
                }
            }
        };
        Ok(Self { kernel })
    }
    /// Sample a normalized value; nonfinite values and OOB gradients are missing.
    pub fn sample(&self, t: f64) -> ChartResult<Option<Color>> {
        match &self.kernel {
            PaletteKernel::Gradient(g) if g.invalid_positions => Err(Gradient::position_error()),
            PaletteKernel::Gradient(g) => Ok(g.sample(t)),
            PaletteKernel::Viridis {
                palette,
                begin,
                end,
                alpha,
            } => {
                if !(0. ..=1.).contains(&t) {
                    return Ok(None);
                }
                let p = begin + (end - begin) * t;
                // Direct scalar evaluation avoids a one-element palette allocation per mark.
                let lab = [
                    palette.channels[0].sample(p)?,
                    palette.channels[1].sample(p)?,
                    palette.channels[2].sample(p)?,
                ];
                Ok(Some(
                    ColorValue::from(d65::from_device_lab(lab).opacity(*alpha)).to_paint(),
                ))
            }
        }
    }
}
