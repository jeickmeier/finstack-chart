//! Smooth zoom trajectories and signed reference duration, with no scheduler.
use super::{ChartResult, DiagnosticCode, Sample, error, parameter};
/// Finite center and positive width, in one explicitly shared coordinate system.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "[f64;3]", into = "[f64;3]")]
pub struct ZoomView([f64; 3]);
impl ZoomView {
    /// Validate `[center_x, center_y, width]` before preparing a trajectory.
    pub fn new(values: [f64; 3]) -> ChartResult<Self> {
        if !values.iter().all(|v| v.is_finite()) || values[2] <= 0. {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Zoom needs finite centers and a positive finite width.",
            ));
        }
        Ok(Self(values))
    }
    /// Owned coordinate/width values.
    pub fn values(self) -> [f64; 3] {
        self.0
    }
}
impl TryFrom<[f64; 3]> for ZoomView {
    type Error = String;
    fn try_from(v: [f64; 3]) -> Result<Self, String> {
        Self::new(v).map_err(|e| e.to_string())
    }
}
impl From<ZoomView> for [f64; 3] {
    fn from(v: ZoomView) -> Self {
        v.0
    }
}
/// Prepared smooth camera interpolation. Width ratios and center distance determine
/// its reference duration; clocks and interruption belong to the caller.
#[derive(Clone, Debug)]
pub struct ZoomInterpolator {
    a: ZoomView,
    dx: f64,
    dy: f64,
    rho: f64,
    rho2: f64,
    s: f64,
    duration: f64,
    general: Option<(f64, f64)>,
}
impl ZoomInterpolator {
    /// Use the reference's exact default constants (sqrt(2), 2 and 4).
    pub fn new(a: ZoomView, b: ZoomView) -> ChartResult<Self> {
        Self::prepare(a, b, std::f64::consts::SQRT_2, 2., 4.)
    }
    /// Use finite custom rho, floored at 1e-3 exactly as the reference factory does.
    pub fn with_rho(a: ZoomView, b: ZoomView, rho: f64) -> ChartResult<Self> {
        if !rho.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Zoom rho must be finite.",
            ));
        }
        let rho = rho.max(1e-3);
        let rho2 = rho * rho;
        Self::prepare(a, b, rho, rho2, rho2 * rho2)
    }
    fn prepare(a: ZoomView, b: ZoomView, rho: f64, rho2: f64, rho4: f64) -> ChartResult<Self> {
        let [x0, y0, w0] = a.0;
        let [x1, y1, w1] = b.0;
        let dx = x1 - x0;
        let dy = y1 - y0;
        let d2 = dx * dx + dy * dy;
        let (s, general) = if d2 < 1e-12 {
            ((w1 / w0).ln() / rho, None)
        } else {
            let d1 = d2.sqrt();
            let b0 = (w1 * w1 - w0 * w0 + rho4 * d2) / (2. * w0 * rho2 * d1);
            let b1 = (w1 * w1 - w0 * w0 - rho4 * d2) / (2. * w1 * rho2 * d1);
            let r0 = ((b0 * b0 + 1.).sqrt() - b0).ln();
            let r1 = ((b1 * b1 + 1.).sqrt() - b1).ln();
            ((r1 - r0) / rho, Some((d1, r0)))
        };
        let duration = s * 1000. * rho / std::f64::consts::SQRT_2;
        if ![dx, dy, d2, rho, rho2, rho4, s, duration]
            .iter()
            .all(|v| v.is_finite())
            || general.is_some_and(|(d, r)| !d.is_finite() || !r.is_finite())
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Zoom trajectory is outside the finite conditioning range.",
            ));
        }
        Ok(Self {
            a,
            dx,
            dy,
            rho,
            rho2,
            s,
            duration,
            general,
        })
    }
    /// Actual rho after the reference's minimum-value rule.
    pub fn rho(&self) -> f64 {
        self.rho
    }
    /// Signed reference metadata; coincident-center zoom-in can be negative.
    pub fn duration_ms(&self) -> f64 {
        self.duration
    }
    /// Explicit nonnegative scheduling adaptation: absolute reference duration.
    pub fn scheduling_duration_ms(&self) -> f64 {
        self.duration.abs()
    }
    /// Sample the smooth trajectory, rejecting nonfinite/zero-width output.
    pub fn sample(&self, t: f64) -> ChartResult<ZoomView> {
        parameter(t)?;
        let [x0, y0, w0] = self.a.0;
        let values = if let Some((d1, r0)) = self.general {
            let s = t * self.s;
            let coshr0 = cosh(r0);
            let u = w0 / (self.rho2 * d1) * (coshr0 * tanh(self.rho * s + r0) - sinh(r0));
            [
                x0 + u * self.dx,
                y0 + u * self.dy,
                w0 * coshr0 / cosh(self.rho * s + r0),
            ]
        } else {
            [
                x0 + t * self.dx,
                y0 + t * self.dy,
                w0 * (self.rho * t * self.s).exp(),
            ]
        };
        ZoomView::new(values)
    }
}
fn cosh(x: f64) -> f64 {
    let x = x.exp();
    (x + 1. / x) / 2.
}
fn sinh(x: f64) -> f64 {
    let x = x.exp();
    (x - 1. / x) / 2.
}
fn tanh(x: f64) -> f64 {
    let x = (2. * x).exp();
    (x - 1.) / (x + 1.)
}
impl Sample<ZoomView> for ZoomInterpolator {
    fn sample(&self, t: f64) -> ChartResult<ZoomView> {
        self.sample(t)
    }
}
