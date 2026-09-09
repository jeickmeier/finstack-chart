//! Shared affine decomposition; CSS/SVG parsing stays headless and bounded.
use super::scalar::number;
use super::{ChartResult, DiagnosticCode, Sample, error, parameter};
use crate::number::ecmascript;
use crate::path::Affine;

/// Canonical transform text convention; all internal geometry uses finite matrices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TransformSyntax {
    /// CSS px lengths and degree angles.
    Css,
    /// SVG unitless user coordinates and degree angles.
    Svg,
}
/// The reference's translation, rotation, x-skew and signed scale decomposition.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Decomposed {
    /// Translation in user units.
    pub translate: [f64; 2],
    /// Rotation in degrees.
    pub rotate: f64,
    /// X skew in degrees.
    pub skew_x: f64,
    /// Signed x/y scale factors (reflection is represented on x).
    pub scale: [f64; 2],
}
impl Decomposed {
    /// Recompose in translation/rotation/skew/scale order. Singular decompositions
    /// keep the reference's explicitly tested behavior; they need not invert input matrices.
    pub fn matrix(self) -> ChartResult<Affine> {
        let r = rotation(self.rotate)?;
        translation(self.translate[0], self.translate[1])?
            .concatenate(r)?
            .concatenate(Affine::new([
                1.,
                0.,
                (self.skew_x * std::f64::consts::PI / 180.).tan(),
                1.,
                0.,
                0.,
            ])?)?
            .concatenate(Affine::new([self.scale[0], 0., 0., self.scale[1], 0., 0.])?)
    }
}
/// Decompose finite coefficients using the pinned operation order and reflection rule.
pub fn decompose(matrix: Affine) -> ChartResult<Decomposed> {
    let [mut a, mut b, mut c, mut d, e, f] = matrix.coefficients();
    let mut scale_x = (a * a + b * b).sqrt();
    if scale_x != 0. {
        a /= scale_x;
        b /= scale_x;
    }
    let mut skew_x = a * c + b * d;
    if skew_x != 0. {
        c -= a * skew_x;
        d -= b * skew_x;
    }
    let scale_y = (c * c + d * d).sqrt();
    if scale_y != 0. {
        c /= scale_y;
        d /= scale_y;
        skew_x /= scale_y;
    }
    if a * d < b * c {
        a = -a;
        b = -b;
        skew_x = -skew_x;
        scale_x = -scale_x;
    }
    let rotate = libm::atan2(b, a) * (180. / std::f64::consts::PI);
    let skew_x = libm::atan(skew_x) * (180. / std::f64::consts::PI);
    if ![e, f, rotate, skew_x, scale_x, scale_y]
        .iter()
        .all(|v| v.is_finite())
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Affine decomposition exceeded finite numerical range.",
        ));
    }
    Ok(Decomposed {
        translate: [e, f],
        rotate,
        skew_x,
        scale: [scale_x, scale_y],
    })
}
/// Compiled decomposition interpolation with shortest rotation and canonical text.
#[derive(Clone, Debug)]
pub struct TransformInterpolator {
    a: Decomposed,
    b: Decomposed,
    syntax: TransformSyntax,
    translate: bool,
    rotate: bool,
    skew: bool,
    scale: bool,
    varying_translate: bool,
    varying_rotate: bool,
    varying_skew: bool,
    varying_scale: bool,
}
impl TransformInterpolator {
    /// Compile two explicitly resolved matrices; no browser or CSS context is consulted.
    pub fn new(a: Affine, b: Affine, syntax: TransformSyntax) -> ChartResult<Self> {
        let mut a = decompose(a)?;
        let mut b = decompose(b)?;
        let varying_translate = a.translate != b.translate;
        let varying_rotate = a.rotate != b.rotate;
        let varying_skew = a.skew_x != b.skew_x;
        let varying_scale = a.scale != b.scale;
        if varying_rotate {
            if a.rotate - b.rotate > 180. {
                b.rotate += 360.;
            } else if b.rotate - a.rotate > 180. {
                a.rotate += 360.;
            }
        }
        Ok(Self {
            a,
            b,
            syntax,
            translate: varying_translate || b.translate != [0., 0.],
            rotate: varying_rotate || b.rotate != 0.,
            skew: varying_skew || b.skew_x != 0.,
            scale: varying_scale || b.scale != [1., 1.],
            varying_translate,
            varying_rotate,
            varying_skew,
            varying_scale,
        })
    }
    /// Parse bounded absolute 2D CSS or SVG input and compile once.
    pub fn from_text(a: &str, b: &str, syntax: TransformSyntax) -> ChartResult<Self> {
        Self::new(
            parse_transform(a, syntax)?,
            parse_transform(b, syntax)?,
            syntax,
        )
    }
    /// Pure sampled decomposition; no matrix-entry blending is used.
    pub fn components(&self, t: f64) -> ChartResult<Decomposed> {
        parameter(t)?;
        let blend = |a, b, varying| if varying { number(a, b, t) } else { b };
        let out = Decomposed {
            translate: std::array::from_fn(|i| {
                blend(
                    self.a.translate[i],
                    self.b.translate[i],
                    self.varying_translate,
                )
            }),
            rotate: blend(self.a.rotate, self.b.rotate, self.varying_rotate),
            skew_x: blend(self.a.skew_x, self.b.skew_x, self.varying_skew),
            scale: std::array::from_fn(|i| {
                blend(self.a.scale[i], self.b.scale[i], self.varying_scale)
            }),
        };
        if ![
            out.translate[0],
            out.translate[1],
            out.rotate,
            out.skew_x,
            out.scale[0],
            out.scale[1],
        ]
        .iter()
        .all(|v| v.is_finite())
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Sampled transform exceeded finite numerical range.",
            ));
        }
        Ok(out)
    }
    /// Recompose the sampled transform as the shared checked affine type.
    pub fn matrix(&self, t: f64) -> ChartResult<Affine> {
        self.components(t)?.matrix()
    }
    /// Canonical source-compatible component order and syntax; identity is an empty string.
    pub fn sample(&self, t: f64) -> ChartResult<String> {
        let v = self.components(t)?;
        let mut pieces = vec![];
        let css = self.syntax == TransformSyntax::Css;
        if self.translate {
            pieces.push(format!(
                "translate({}{}, {}{})",
                ecmascript(v.translate[0]),
                if css { "px" } else { "" },
                ecmascript(v.translate[1]),
                if css { "px" } else { "" }
            ));
        }
        if self.rotate {
            pieces.push(format!(
                "rotate({}{})",
                ecmascript(v.rotate),
                if css { "deg" } else { "" }
            ));
        }
        if self.skew {
            pieces.push(format!(
                "skewX({}{})",
                ecmascript(v.skew_x),
                if css { "deg" } else { "" }
            ));
        }
        if self.scale {
            pieces.push(format!(
                "scale({},{})",
                ecmascript(v.scale[0]),
                ecmascript(v.scale[1])
            ));
        }
        Ok(pieces.join(" "))
    }
}
impl Sample<String> for TransformInterpolator {
    fn sample(&self, t: f64) -> ChartResult<String> {
        self.sample(t)
    }
}
pub(super) fn translation(x: f64, y: f64) -> ChartResult<Affine> {
    Affine::new([1., 0., 0., 1., x, y])
}
pub(super) fn rotation(degrees: f64) -> ChartResult<Affine> {
    let degrees = degrees % 360.;
    let degrees = if degrees > 180. {
        degrees - 360.
    } else if degrees < -180. {
        degrees + 360.
    } else {
        degrees
    };
    let (sin, cos) = if degrees == 0. {
        (0., 1.)
    } else if degrees == 90. {
        (1., 0.)
    } else if degrees == -90. {
        (-1., 0.)
    } else if degrees.abs() == 180. {
        (0., -1.)
    } else {
        let r = degrees * (std::f64::consts::PI / 180.);
        (r.sin(), r.cos())
    };
    Affine::new([cos, sin, -sin, cos, 0., 0.])
}
pub use super::transform_parse::parse_transform;
