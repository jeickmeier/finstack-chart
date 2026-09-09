//! Color blending owns no parser or conversion formula; those remain in `color`.
use super::scalar::js_round;
use super::{ChartResult, DiagnosticCode, Sample, ScalarInterpolator, error, parameter};
use crate::color::{ColorSpace, ColorValue, cubehelix, hcl, hsl, lab, rgb};

#[derive(Clone, Copy, Debug)]
enum Blend {
    Constant(f64),
    Linear { a: f64, d: f64 },
    Gamma { a: f64, d: f64, inverse: f64 },
}
impl Blend {
    fn new(a: f64, b: f64, gamma: f64, short_hue: bool) -> Self {
        let mut d = b - a;
        if d == 0. || d.is_nan() {
            return Self::Constant(if a.is_nan() { b } else { a });
        }
        if short_hue && !(-180. ..=180.).contains(&d) {
            d -= 360. * js_round(d / 360.);
        }
        if gamma == 1. {
            Self::Linear { a, d }
        } else {
            let a = pxfm::f_pow(a, gamma);
            Self::Gamma {
                a,
                d: pxfm::f_pow(b, gamma) - a,
                inverse: 1. / gamma,
            }
        }
    }
    fn sample(self, t: f64) -> f64 {
        match self {
            Self::Constant(v) => v,
            Self::Linear { a, d } => a + t * d,
            Self::Gamma { a, d, inverse } => pxfm::f_pow(a + t * d, inverse),
        }
    }
}

/// A shortest-path angular interpolator normalized into [0, 360) when finite.
#[derive(Clone, Copy, Debug)]
pub struct HueInterpolator {
    blend: Blend,
}
impl HueInterpolator {
    /// Retain endpoints with the reference's undefined and antipodal semantics.
    pub fn new(a: f64, b: f64) -> Self {
        Self {
            blend: Blend::new(a, b, 1., true),
        }
    }
    /// Sample and normalize; undefined endpoints can produce a tagged NaN result.
    pub fn sample(self, t: f64) -> ChartResult<f64> {
        parameter(t)?;
        let x = self.blend.sample(t);
        Ok(x - 360. * (x / 360.).floor())
    }
}
impl Sample<f64> for HueInterpolator {
    fn sample(&self, t: f64) -> ChartResult<f64> {
        (*self).sample(t)
    }
}

/// Required interpolation routes; `Long` keeps authored hue differences.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ColorRoute {
    /// RGB channels, with optional channel gamma.
    Rgb,
    /// HSL with shortest hue.
    Hsl,
    /// HSL with direct long hue.
    HslLong,
    /// Cartesian D50 Lab.
    Lab,
    /// Cylindrical Lab with shortest hue.
    Hcl,
    /// Cylindrical Lab with direct long hue.
    HclLong,
    /// Cubehelix with shortest hue and optional lightness gamma.
    Cubehelix,
    /// Cubehelix with direct long hue and optional lightness gamma.
    CubehelixLong,
}
/// Compiled floating color interpolation; byte formatting is an explicit final step.
#[derive(Clone, Debug)]
pub struct ColorInterpolator {
    kernel: ColorKernel,
}
#[derive(Clone, Debug)]
enum ColorKernel {
    Channels {
        space: ColorSpace,
        channels: [Blend; 4],
        lightness_gamma: f64,
    },
    Spline {
        r: ScalarInterpolator,
        g: ScalarInterpolator,
        b: ScalarInterpolator,
    },
}
impl ColorInterpolator {
    /// Compile channel conversions once. Gamma defaults to one and is configurable
    /// only for RGB and the two Cubehelix routes; it must be positive and finite.
    pub fn new(
        route: ColorRoute,
        a: ColorValue,
        b: ColorValue,
        gamma: Option<f64>,
    ) -> ChartResult<Self> {
        if gamma.is_some()
            && !matches!(
                route,
                ColorRoute::Rgb | ColorRoute::Cubehelix | ColorRoute::CubehelixLong
            )
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Gamma is unsupported for this color route.",
            ));
        }
        let gamma = gamma.unwrap_or(1.);
        if !gamma.is_finite() || gamma <= 0. {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Color gamma must be positive and finite.",
            ));
        }
        let space = match route {
            ColorRoute::Rgb => ColorSpace::Rgb,
            ColorRoute::Hsl | ColorRoute::HslLong => ColorSpace::Hsl,
            ColorRoute::Lab => ColorSpace::Lab,
            ColorRoute::Hcl | ColorRoute::HclLong => ColorSpace::Hcl,
            ColorRoute::Cubehelix | ColorRoute::CubehelixLong => ColorSpace::Cubehelix,
        };
        let channels = |c: ColorValue| match space {
            ColorSpace::Rgb => {
                let v = c.rgb();
                [v.r, v.g, v.b, v.opacity]
            }
            ColorSpace::Hsl => {
                let v = c.hsl();
                [v.h, v.s, v.l, v.opacity]
            }
            ColorSpace::Lab => {
                let v = c.lab();
                [v.l, v.a, v.b, v.opacity]
            }
            ColorSpace::Hcl => {
                let v = c.hcl();
                [v.h, v.c, v.l, v.opacity]
            }
            ColorSpace::Cubehelix => {
                let v = c.cubehelix();
                [v.h, v.s, v.l, v.opacity]
            }
        };
        let a = channels(a);
        let b = channels(b);
        let short = matches!(
            route,
            ColorRoute::Hsl | ColorRoute::Hcl | ColorRoute::Cubehelix
        );
        let channels = std::array::from_fn(|i| {
            Blend::new(
                a[i],
                b[i],
                if route == ColorRoute::Rgb && i < 3 {
                    gamma
                } else {
                    1.
                },
                i == 0 && short,
            )
        });
        Ok(Self {
            kernel: ColorKernel::Channels {
                space,
                channels,
                lightness_gamma: if space == ColorSpace::Cubehelix {
                    gamma
                } else {
                    1.
                },
            },
        })
    }
    /// Open/closed RGB basis with the reference's opaque output, including when
    /// controls have transparent or undefined channels. Shares scalar spline code.
    pub fn rgb_basis(colors: &[ColorValue], closed: bool) -> ChartResult<Self> {
        super::count(colors.len(), if closed { 1 } else { 2 })?;
        let values: Vec<_> = colors.iter().map(|v| v.rgb()).collect();
        let spline = |channel: fn(crate::color::Rgb) -> f64| {
            let controls = values
                .iter()
                .map(|v| {
                    let n = channel(*v);
                    if n.is_nan() || n == 0. { 0. } else { n }
                })
                .collect();
            if closed {
                ScalarInterpolator::basis_closed(controls)
            } else {
                ScalarInterpolator::basis(controls)
            }
        };
        Ok(Self {
            kernel: ColorKernel::Spline {
                r: spline(|v| v.r)?,
                g: spline(|v| v.g)?,
                b: spline(|v| v.b)?,
            },
        })
    }
    /// Floating result in the interpolation space; no intermediate byte quantization.
    pub fn sample_color(&self, t: f64) -> ChartResult<ColorValue> {
        parameter(t)?;
        self.scale_color(t)
    }
    pub(crate) fn scale_color(&self, t: f64) -> ChartResult<ColorValue> {
        Ok(match &self.kernel {
            ColorKernel::Spline { r, g, b } => {
                rgb(r.evaluate(t), g.evaluate(t), b.evaluate(t)).into()
            }
            ColorKernel::Channels {
                space,
                channels,
                lightness_gamma,
            } => {
                let mut v = channels.map(|c| c.sample(t));
                if *space == ColorSpace::Cubehelix {
                    v[2] = channels[2].sample(pxfm::f_pow(t, *lightness_gamma));
                }
                match space {
                    ColorSpace::Rgb => rgb(v[0], v[1], v[2]).opacity(v[3]).into(),
                    ColorSpace::Hsl => hsl(v[0], v[1], v[2]).opacity(v[3]).into(),
                    ColorSpace::Lab => lab(v[0], v[1], v[2]).opacity(v[3]).into(),
                    ColorSpace::Hcl => hcl(v[0], v[1], v[2]).opacity(v[3]).into(),
                    ColorSpace::Cubehelix => cubehelix(v[0], v[1], v[2]).opacity(v[3]).into(),
                }
            }
        })
    }
    /// Canonical RGB CSS result, rounded and clamped by the shared color formatter.
    pub fn sample(&self, t: f64) -> ChartResult<String> {
        Ok(self.sample_color(t)?.format_rgb())
    }
}
impl Sample<String> for ColorInterpolator {
    fn sample(&self, t: f64) -> ChartResult<String> {
        self.sample(t)
    }
}
