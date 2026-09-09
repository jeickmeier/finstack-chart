//! Built-in d3-shape symbols, using the shared checked path destination.
use super::{ShapeLimits, domain};
use crate::{
    ChartResult,
    path::{Path, Precision},
};
use std::f64::consts::{PI, TAU};

/// The thirteen distinct built-in symbol geometries, separate from legacy radius marks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SymbolKind {
    /// Filled circle with area equal to the requested size.
    #[default]
    Circle,
    /// Filled Greek cross.
    Cross,
    /// Filled thirty-degree diamond.
    Diamond,
    /// Filled axis-aligned square.
    Square,
    /// Filled five-point star.
    Star,
    /// Filled equilateral triangle.
    Triangle,
    /// Filled three-armed wye.
    Wye,
    /// Open horizontal and vertical strokes.
    Plus,
    /// Open diagonal strokes; X is the reference alias.
    #[serde(alias = "X")]
    Times,
    /// Three open intersecting strokes.
    Asterisk,
    /// Stroke-oriented diamond sizing.
    Diamond2,
    /// Stroke-oriented square sizing.
    Square2,
    /// Stroke-oriented equilateral triangle sizing.
    Triangle2,
}
/// Ordered reference palette designed for filled marks.
pub const SYMBOLS_FILL: [SymbolKind; 7] = [
    SymbolKind::Circle,
    SymbolKind::Cross,
    SymbolKind::Diamond,
    SymbolKind::Square,
    SymbolKind::Star,
    SymbolKind::Triangle,
    SymbolKind::Wye,
];
/// Ordered reference palette designed for stroked marks.
pub const SYMBOLS_STROKE: [SymbolKind; 7] = [
    SymbolKind::Circle,
    SymbolKind::Plus,
    SymbolKind::Times,
    SymbolKind::Triangle2,
    SymbolKind::Asterisk,
    SymbolKind::Square2,
    SymbolKind::Diamond2,
];
/// Reference `symbols` alias for the filled palette.
pub const SYMBOLS: [SymbolKind; 7] = SYMBOLS_FILL;
impl SymbolKind {
    /// Reference `symbolX` alias for the times geometry.
    pub const X: Self = Self::Times;
    /// Whether this type contains open subpaths which must be stroked to produce ink.
    pub fn is_open(self) -> bool {
        matches!(self, Self::Plus | Self::Times | Self::Asterisk)
    }
}
/// Symbol paint policy; Auto uses the built-in filled or stroked size convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SymbolPaint {
    /// Use filled styling for the fill family and stroked styling for stroke-only family types.
    #[default]
    Auto,
    /// Filled geometry; open symbols reject this explicit mode.
    Fill,
    /// Stroked geometry, including the circle in the stroke palette.
    Stroke,
}
impl SymbolPaint {
    /// Resolve topology-dependent painting before geometry submission.
    pub fn resolve(self, kind: SymbolKind) -> ChartResult<Self> {
        match self {
            Self::Auto => Ok(
                if matches!(
                    kind,
                    SymbolKind::Plus
                        | SymbolKind::Times
                        | SymbolKind::Asterisk
                        | SymbolKind::Diamond2
                        | SymbolKind::Square2
                        | SymbolKind::Triangle2
                ) {
                    Self::Stroke
                } else {
                    Self::Fill
                },
            ),
            Self::Fill if kind.is_open() => {
                Err(super::invalid("Open symbols require stroked painting."))
            }
            value => Ok(value),
        }
    }
}
/// Native symbol protocol; custom implementations draw through checked shared paths.
/// A failing draw may leave a prefix in a caller-owned destination.
pub trait SymbolDraw {
    /// Append one symbol about the origin with finite, nonnegative size.
    fn draw(&self, context: &mut Path, size: f64) -> ChartResult<()>;
}
fn size_valid(size: f64) -> ChartResult<()> {
    if size.is_finite() && size >= 0. {
        Ok(())
    } else {
        Err(domain("Symbol size must be finite and nonnegative."))
    }
}
fn polygon(context: &mut Path, points: &[[f64; 2]]) -> ChartResult<()> {
    if let Some(p) = points.first() {
        context.move_to(p[0], p[1])?;
    }
    for p in points.iter().skip(1) {
        context.line_to(p[0], p[1])?;
    }
    context.close_path()
}
impl SymbolDraw for SymbolKind {
    fn draw(&self, c: &mut Path, size: f64) -> ChartResult<()> {
        size_valid(size)?;
        match self {
            Self::Circle => {
                let r = (size / PI).sqrt();
                c.move_to(r, 0.)?;
                c.arc([0., 0.], r, 0., TAU, false)
            }
            Self::Cross => {
                let r = (size / 5.).sqrt() / 2.;
                polygon(
                    c,
                    &[
                        [-3. * r, -r],
                        [-r, -r],
                        [-r, -3. * r],
                        [r, -3. * r],
                        [r, -r],
                        [3. * r, -r],
                        [3. * r, r],
                        [r, r],
                        [r, 3. * r],
                        [-r, 3. * r],
                        [-r, r],
                        [-3. * r, r],
                    ],
                )
            }
            Self::Diamond => {
                let t = (1_f64 / 3.).sqrt();
                let y = (size / (t * 2.)).sqrt();
                let x = y * t;
                polygon(c, &[[0., -y], [x, 0.], [0., y], [-x, 0.]])
            }
            Self::Square => {
                let w = size.sqrt();
                let x = -w / 2.;
                c.rect(x, x, w, w)
            }
            Self::Star => {
                // Fixed D3 rotations preserve the reference binary64 evaluation across
                // libm implementations, including SVG coordinates above 1e21.
                // kx/ky are sin/cos(TAU/10) * sin(PI/10)/sin(7*PI/10).
                let kx = 0.224_513_988_289_792_66;
                let ky = -0.309_016_994_374_947_4;
                // (cos(TAU*i/5), sin(TAU*i/5)), i = 1..4.
                const ROTATIONS: [(f64, f64); 4] = [
                    (0.309_016_994_374_947_45, 0.951_056_516_295_153_5),
                    (-0.809_016_994_374_947_3, 0.587_785_252_292_473_2),
                    (-0.809_016_994_374_947_5, -0.587_785_252_292_473),
                    (0.309_016_994_374_947_23, -0.951_056_516_295_153_6),
                ];
                let r = (size * 0.890_813_091_529_285_2).sqrt();
                let x = kx * r;
                let y = ky * r;
                c.move_to(0., -r)?;
                c.line_to(x, y)?;
                for (ca, sa) in ROTATIONS {
                    c.line_to(sa * r, -ca * r)?;
                    c.line_to(ca * x - sa * y, sa * x + ca * y)?;
                }
                c.close_path()
            }
            Self::Triangle => {
                let t = 3_f64.sqrt();
                let y = -(size / (t * 3.)).sqrt();
                polygon(c, &[[0., y * 2.], [-t * y, -y], [t * y, -y]])
            }
            Self::Wye => {
                let co = -0.5;
                let si = 3_f64.sqrt() / 2.;
                let k = 1. / 12_f64.sqrt();
                let a = (k / 2. + 1.) * 3.;
                let r = (size / a).sqrt();
                let x0 = r / 2.;
                let y0 = r * k;
                let x1 = x0;
                let y1 = r * k + r;
                let x2 = -x1;
                let y2 = y1;
                polygon(
                    c,
                    &[
                        [x0, y0],
                        [x1, y1],
                        [x2, y2],
                        [co * x0 - si * y0, si * x0 + co * y0],
                        [co * x1 - si * y1, si * x1 + co * y1],
                        [co * x2 - si * y2, si * x2 + co * y2],
                        [co * x0 + si * y0, co * y0 - si * x0],
                        [co * x1 + si * y1, co * y1 - si * x1],
                        [co * x2 + si * y2, co * y2 - si * x2],
                    ],
                )
            }
            Self::Plus => {
                let r = (size - (size / 7.).min(2.)).sqrt() * 0.87559;
                c.move_to(-r, 0.)?;
                c.line_to(r, 0.)?;
                c.move_to(0., r)?;
                c.line_to(0., -r)
            }
            Self::Times => {
                let r = (size - (size / 6.).min(1.7)).sqrt() * 0.6189;
                c.move_to(-r, -r)?;
                c.line_to(r, r)?;
                c.move_to(-r, r)?;
                c.line_to(r, -r)
            }
            Self::Asterisk => {
                let r = (size + (size / 28.).min(0.75)).sqrt() * 0.59436;
                let t = r / 2.;
                let u = t * 3_f64.sqrt();
                c.move_to(0., r)?;
                c.line_to(0., -r)?;
                c.move_to(-u, -t)?;
                c.line_to(u, t)?;
                c.move_to(-u, t)?;
                c.line_to(u, -t)
            }
            Self::Diamond2 => {
                let r = size.sqrt() * 0.62625;
                polygon(c, &[[0., -r], [r, 0.], [0., r], [-r, 0.]])
            }
            Self::Square2 => {
                let r = size.sqrt() * 0.4431;
                polygon(c, &[[r, r], [r, -r], [-r, -r], [-r, r]])
            }
            Self::Triangle2 => {
                let s = size.sqrt() * 0.6824;
                let t = s / 2.;
                let u = (s * 3_f64.sqrt()) / 2.;
                polygon(c, &[[0., -s], [u, t], [-u, t]])
            }
        }
    }
}
/// Reusable symbol generator, defaulting to a filled circle of area 64.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Symbol {
    kind: SymbolKind,
    pub(super) size: f64,
    digits: Option<f64>,
    limits: ShapeLimits,
}
impl Default for Symbol {
    fn default() -> Self {
        Self {
            kind: SymbolKind::Circle,
            size: 64.,
            digits: Some(3.),
            limits: ShapeLimits::default(),
        }
    }
}
impl Symbol {
    /// Default circle, size 64 and three SVG fractional digits.
    pub fn new() -> Self {
        Self::default()
    }
    /// Select a built-in type; legacy radius symbols remain a separate chart mode.
    pub fn kind(mut self, kind: SymbolKind) -> Self {
        self.kind = kind;
        self
    }
    /// Set area size, or the reference stroke-size convention for stroke-oriented types.
    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }
    /// Set SVG text precision without changing retained numeric geometry.
    pub fn digits(mut self, digits: Option<f64>) -> ChartResult<Self> {
        precision(digits)?;
        self.digits = digits;
        Ok(self)
    }
    /// Set source-entry and shared path bounds.
    pub fn limits(mut self, limits: ShapeLimits) -> Self {
        self.limits = limits;
        self
    }
    /// Validate a deserialized configuration independently of invocation.
    pub fn validate(&self) -> ChartResult<()> {
        size_valid(self.size)?;
        precision(self.digits)?;
        Ok(())
    }
    /// Generate an independent owned numeric path.
    pub fn generate(&self) -> ChartResult<Path> {
        self.generate_with(&self.kind, self.size)
    }
    /// Resolve built-in type and size from a native datum accessor.
    pub fn generate_by<T>(
        &self,
        datum: &T,
        mut accessor: impl FnMut(&T) -> ChartResult<(SymbolKind, f64)>,
    ) -> ChartResult<Path> {
        let (kind, size) = accessor(datum)?;
        self.generate_with(&kind, size)
    }
    /// Invoke a native symbol implementation through the same checked path owner.
    pub fn generate_with(
        &self,
        symbol: &(impl SymbolDraw + ?Sized),
        size: f64,
    ) -> ChartResult<Path> {
        self.validate()?;
        size_valid(size)?;
        if self.limits.max_points == 0 {
            return Err(super::limit("Symbol source-entry limit exceeded."));
        }
        let mut path = Path::with_options(precision(self.digits)?, self.limits.path)?;
        symbol.draw(&mut path, size)?;
        Ok(path)
    }
}
fn precision(digits: Option<f64>) -> ChartResult<Precision> {
    digits
        .map(Precision::from_digits)
        .transpose()
        .map(|p| p.unwrap_or_default())
}
