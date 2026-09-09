//! Circular sectors with the d3-shape 3.2.0 corner and padding construction.
use super::{ShapeLimits, domain};
use crate::{
    ChartResult,
    path::{Path, Precision},
};
use std::f64::consts::{FRAC_PI_2, PI, TAU};
const EPSILON: f64 = 1e-12;

/// Materialized arc datum; angles are radians clockwise from twelve o'clock.
/// Missing required fields diagnose unless the generator overrides them with constants.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ArcDatum {
    /// Inner radius before the generator's radius swap.
    pub inner_radius: f64,
    /// Outer radius before the generator's radius swap.
    pub outer_radius: f64,
    /// Starting angle in radians.
    pub start_angle: f64,
    /// Ending angle in radians.
    pub end_angle: f64,
    /// Angular gap, defaulting to zero.
    #[serde(default)]
    pub pad_angle: f64,
}
impl Default for ArcDatum {
    fn default() -> Self {
        Self {
            inner_radius: f64::NAN,
            outer_radius: f64::NAN,
            start_angle: f64::NAN,
            end_angle: f64::NAN,
            pad_angle: 0.,
        }
    }
}
/// Arc parameters after native accessors or materialized constants are resolved.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArcParameters {
    /// Radius, sweep and pad angle values.
    pub datum: ArcDatum,
    /// Corner radius; negative values have no rounding effect.
    pub corner_radius: f64,
    /// Explicit padding radius, or the Euclidean combination of the two radii.
    pub pad_radius: Option<f64>,
}
impl ArcParameters {
    fn validate(self) -> ChartResult<()> {
        let d = self.datum;
        if [
            d.inner_radius,
            d.outer_radius,
            d.start_angle,
            d.end_angle,
            d.pad_angle,
            self.corner_radius,
        ]
        .iter()
        .all(|v| v.is_finite())
            && self.pad_radius.is_none_or(f64::is_finite)
        {
            Ok(())
        } else {
            Err(domain("Arc coordinates and parameters must be finite."))
        }
    }
}
/// Reusable arc generator with optional constants overriding datum fields.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Arc {
    inner_radius: Option<f64>,
    outer_radius: Option<f64>,
    start_angle: Option<f64>,
    end_angle: Option<f64>,
    pad_angle: Option<f64>,
    corner_radius: f64,
    pad_radius: Option<f64>,
    digits: Option<f64>,
    limits: ShapeLimits,
}
impl Default for Arc {
    fn default() -> Self {
        Self {
            inner_radius: None,
            outer_radius: None,
            start_angle: None,
            end_angle: None,
            pad_angle: None,
            corner_radius: 0.,
            pad_radius: None,
            digits: Some(3.),
            limits: ShapeLimits::default(),
        }
    }
}
impl Arc {
    /// Use datum radii/angles, zero corner radius, automatic pad radius and three digits.
    pub fn new() -> Self {
        Self::default()
    }
    /// Override the inner radius, or restore the datum accessor.
    pub fn inner_radius(mut self, value: Option<f64>) -> Self {
        self.inner_radius = value;
        self
    }
    /// Override the outer radius, or restore the datum accessor.
    pub fn outer_radius(mut self, value: Option<f64>) -> Self {
        self.outer_radius = value;
        self
    }
    /// Override the start angle, or restore the datum accessor.
    pub fn start_angle(mut self, value: Option<f64>) -> Self {
        self.start_angle = value;
        self
    }
    /// Override the end angle, or restore the datum accessor.
    pub fn end_angle(mut self, value: Option<f64>) -> Self {
        self.end_angle = value;
        self
    }
    /// Override the pad angle, or restore the datum accessor.
    pub fn pad_angle(mut self, value: Option<f64>) -> Self {
        self.pad_angle = value;
        self
    }
    /// Set the corner-radius constant.
    pub fn corner_radius(mut self, value: f64) -> Self {
        self.corner_radius = value;
        self
    }
    /// Set an explicit pad radius, or use the radii's Euclidean combination.
    pub fn pad_radius(mut self, value: Option<f64>) -> Self {
        self.pad_radius = value;
        self
    }
    /// Set SVG digits; None preserves unrounded numeric text.
    pub fn digits(mut self, value: Option<f64>) -> ChartResult<Self> {
        precision(value)?;
        self.digits = value;
        Ok(self)
    }
    /// Set input and shared path work bounds.
    pub fn limits(mut self, value: ShapeLimits) -> Self {
        self.limits = value;
        self
    }
    /// Validate a deserialized reusable configuration.
    pub fn validate(&self) -> ChartResult<()> {
        precision(self.digits)?;
        if [
            self.inner_radius,
            self.outer_radius,
            self.start_angle,
            self.end_angle,
            self.pad_angle,
            self.pad_radius,
            Some(self.corner_radius),
        ]
        .into_iter()
        .flatten()
        .all(f64::is_finite)
        {
            Ok(())
        } else {
            Err(domain("Arc constants must be finite."))
        }
    }
    fn resolve(&self, d: ArcDatum) -> ArcParameters {
        ArcParameters {
            datum: ArcDatum {
                inner_radius: self.inner_radius.unwrap_or(d.inner_radius),
                outer_radius: self.outer_radius.unwrap_or(d.outer_radius),
                start_angle: self.start_angle.unwrap_or(d.start_angle),
                end_angle: self.end_angle.unwrap_or(d.end_angle),
                pad_angle: self.pad_angle.unwrap_or(d.pad_angle),
            },
            corner_radius: self.corner_radius,
            pad_radius: self.pad_radius,
        }
    }
    /// Generate an owned numeric path; precision affects SVG text only.
    pub fn generate(&self, datum: ArcDatum) -> ChartResult<Path> {
        self.generate_by(&datum, |d| Ok(self.resolve(*d)))
    }
    /// Invoke a native accessor for all resolved parameters, replacing configured accessors.
    pub fn generate_by<T>(
        &self,
        datum: &T,
        accessor: impl FnOnce(&T) -> ChartResult<ArcParameters>,
    ) -> ChartResult<Path> {
        self.validate()?;
        if self.limits.max_points == 0 {
            return Err(super::limit("Arc source-entry limit exceeded."));
        }
        let parameters = accessor(datum)?;
        parameters.validate()?;
        let mut path = Path::with_options(precision(self.digits)?, self.limits.path)?;
        draw(&mut path, parameters)?;
        Ok(path)
    }
    /// Midpoint of the center line, not the area centroid; corners/padding do not alter it.
    pub fn centroid(&self, datum: ArcDatum) -> ChartResult<[f64; 2]> {
        self.centroid_by(&datum, |d| Ok(self.resolve(*d)))
    }
    /// Resolve native parameters and evaluate the center-line midpoint.
    pub fn centroid_by<T>(
        &self,
        datum: &T,
        accessor: impl FnOnce(&T) -> ChartResult<ArcParameters>,
    ) -> ChartResult<[f64; 2]> {
        self.validate()?;
        if self.limits.max_points == 0 {
            return Err(super::limit("Arc source-entry limit exceeded."));
        }
        let p = accessor(datum)?;
        p.validate()?;
        let d = p.datum;
        let r = (d.inner_radius + d.outer_radius) / 2.;
        let a = (d.start_angle + d.end_angle) / 2. - FRAC_PI_2;
        let point = [a.cos() * r, a.sin() * r];
        if point.iter().all(|v| v.is_finite()) {
            Ok(point)
        } else {
            Err(domain("Arc centroid overflowed."))
        }
    }
}
fn precision(digits: Option<f64>) -> ChartResult<Precision> {
    Ok(digits
        .map(Precision::from_digits)
        .transpose()?
        .unwrap_or_default())
}
fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}
fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}
fn acos(v: f64) -> f64 {
    if v > 1. {
        0.
    } else if v < -1. {
        PI
    } else {
        v.acos()
    }
}
fn asin(v: f64) -> f64 {
    if v >= 1. {
        FRAC_PI_2
    } else if v <= -1. {
        -FRAC_PI_2
    } else {
        v.asin()
    }
}
fn intersect(p: [f64; 2], q: [f64; 2], r: [f64; 2], s: [f64; 2]) -> Option<[f64; 2]> {
    let x10 = q[0] - p[0];
    let y10 = q[1] - p[1];
    let x32 = s[0] - r[0];
    let y32 = s[1] - r[1];
    let t = y32 * x10 - x32 * y10;
    if t * t < EPSILON {
        return None;
    }
    let t = (x32 * (p[1] - r[1]) - y32 * (p[0] - r[0])) / t;
    Some([p[0] + t * x10, p[1] + t * y10])
}
struct Tangent {
    cx: f64,
    cy: f64,
    x01: f64,
    y01: f64,
    x11: f64,
    y11: f64,
}
impl Tangent {
    fn start(&self) -> f64 {
        self.y01.atan2(self.x01)
    }
    fn end(&self) -> f64 {
        self.y11.atan2(self.x11)
    }
    fn ring(&self) -> f64 {
        (self.cy + self.y11).atan2(self.cx + self.x11)
    }
}
fn tangents(p: [f64; 2], q: [f64; 2], r1: f64, rc: f64, cw: bool) -> Tangent {
    let x01 = p[0] - q[0];
    let y01 = p[1] - q[1];
    let lo = if cw { rc } else { -rc } / (x01 * x01 + y01 * y01).sqrt();
    let ox = lo * y01;
    let oy = -lo * x01;
    let x11 = p[0] + ox;
    let y11 = p[1] + oy;
    let x10 = q[0] + ox;
    let y10 = q[1] + oy;
    let x00 = (x11 + x10) / 2.;
    let y00 = (y11 + y10) / 2.;
    let dx = x10 - x11;
    let dy = y10 - y11;
    let d2 = dx * dx + dy * dy;
    let r = r1 - rc;
    let det = x11 * y10 - x10 * y11;
    let d = if dy < 0. { -1. } else { 1. } * max(0., r * r * d2 - det * det).sqrt();
    let mut cx0 = (det * dy - dx * d) / d2;
    let mut cy0 = (-det * dx - dy * d) / d2;
    let cx1 = (det * dy + dx * d) / d2;
    let cy1 = (-det * dx + dy * d) / d2;
    let dx0 = cx0 - x00;
    let dy0 = cy0 - y00;
    let dx1 = cx1 - x00;
    let dy1 = cy1 - y00;
    if dx0 * dx0 + dy0 * dy0 > dx1 * dx1 + dy1 * dy1 {
        cx0 = cx1;
        cy0 = cy1;
    }
    Tangent {
        cx: cx0,
        cy: cy0,
        x01: -ox,
        y01: -oy,
        x11: cx0 * (r1 / r - 1.),
        y11: cy0 * (r1 / r - 1.),
    }
}
fn corner(path: &mut Path, t: &Tangent, r: f64, a0: f64, a1: f64, cw: bool) -> ChartResult<()> {
    path.arc([t.cx, t.cy], r, a0, a1, !cw)
}
fn draw(path: &mut Path, p: ArcParameters) -> ChartResult<()> {
    let d = p.datum;
    let mut r0 = d.inner_radius;
    let mut r1 = d.outer_radius;
    let a0 = d.start_angle - FRAC_PI_2;
    let a1 = d.end_angle - FRAC_PI_2;
    let da = (a1 - a0).abs();
    let cw = a1 > a0;
    if r1 < r0 {
        std::mem::swap(&mut r1, &mut r0);
    }
    if r1 <= EPSILON {
        path.move_to(0., 0.)?;
    } else if da > TAU - EPSILON {
        path.move_to(r1 * a0.cos(), r1 * a0.sin())?;
        path.arc([0., 0.], r1, a0, a1, !cw)?;
        if r0 > EPSILON {
            path.move_to(r0 * a1.cos(), r0 * a1.sin())?;
            path.arc([0., 0.], r0, a1, a0, cw)?;
        }
    } else {
        let (mut a01, mut a11, mut a00, mut a10) = (a0, a1, a0, a1);
        let (mut da0, mut da1) = (da, da);
        let ap = d.pad_angle / 2.;
        let rp = if ap > EPSILON {
            p.pad_radius.unwrap_or_else(|| (r0 * r0 + r1 * r1).sqrt())
        } else {
            0.
        };
        let rc = min((r1 - r0).abs() / 2., p.corner_radius);
        let (mut rc0, mut rc1) = (rc, rc);
        if rp > EPSILON {
            let mut p0 = asin(rp / r0 * ap.sin());
            let mut p1 = asin(rp / r1 * ap.sin());
            da0 -= p0 * 2.;
            if da0 > EPSILON {
                p0 *= if cw { 1. } else { -1. };
                a00 += p0;
                a10 -= p0;
            } else {
                da0 = 0.;
                a00 = (a0 + a1) / 2.;
                a10 = a00;
            }
            da1 -= p1 * 2.;
            if da1 > EPSILON {
                p1 *= if cw { 1. } else { -1. };
                a01 += p1;
                a11 -= p1;
            } else {
                da1 = 0.;
                a01 = (a0 + a1) / 2.;
                a11 = a01;
            }
        }
        let x01 = r1 * a01.cos();
        let y01 = r1 * a01.sin();
        let x10 = r0 * a10.cos();
        let y10 = r0 * a10.sin();
        let x11 = r1 * a11.cos();
        let y11 = r1 * a11.sin();
        let x00 = r0 * a00.cos();
        let y00 = r0 * a00.sin();
        if rc > EPSILON && da < PI {
            if let Some(oc) = intersect([x01, y01], [x00, y00], [x11, y11], [x10, y10]) {
                let ax = x01 - oc[0];
                let ay = y01 - oc[1];
                let bx = x11 - oc[0];
                let by = y11 - oc[1];
                let kc = 1.
                    / (acos(
                        (ax * bx + ay * by)
                            / ((ax * ax + ay * ay).sqrt() * (bx * bx + by * by).sqrt()),
                    ) / 2.)
                        .sin();
                let lc = (oc[0] * oc[0] + oc[1] * oc[1]).sqrt();
                rc0 = min(rc, (r0 - lc) / (kc - 1.));
                rc1 = min(rc, (r1 - lc) / (kc + 1.));
            } else {
                rc0 = 0.;
                rc1 = 0.;
            }
        }
        if da1 <= EPSILON {
            path.move_to(x01, y01)?;
        } else if rc1 > EPSILON {
            let t0 = tangents([x00, y00], [x01, y01], r1, rc1, cw);
            let t1 = tangents([x11, y11], [x10, y10], r1, rc1, cw);
            path.move_to(t0.cx + t0.x01, t0.cy + t0.y01)?;
            if rc1 < rc {
                corner(path, &t0, rc1, t0.start(), t1.start(), cw)?;
            } else {
                corner(path, &t0, rc1, t0.start(), t0.end(), cw)?;
                path.arc([0., 0.], r1, t0.ring(), t1.ring(), !cw)?;
                corner(path, &t1, rc1, t1.end(), t1.start(), cw)?;
            }
        } else {
            path.move_to(x01, y01)?;
            path.arc([0., 0.], r1, a01, a11, !cw)?;
        }
        if r0 <= EPSILON || da0 <= EPSILON {
            path.line_to(x10, y10)?;
        } else if rc0 > EPSILON {
            let t0 = tangents([x10, y10], [x11, y11], r0, -rc0, cw);
            let t1 = tangents([x01, y01], [x00, y00], r0, -rc0, cw);
            path.line_to(t0.cx + t0.x01, t0.cy + t0.y01)?;
            if rc0 < rc {
                corner(path, &t0, rc0, t0.start(), t1.start(), cw)?;
            } else {
                corner(path, &t0, rc0, t0.start(), t0.end(), cw)?;
                path.arc([0., 0.], r0, t0.ring(), t1.ring(), cw)?;
                corner(path, &t1, rc0, t1.end(), t1.start(), cw)?;
            }
        } else {
            path.arc([0., 0.], r0, a10, a00, cw)?;
        }
    }
    path.close_path()
}
