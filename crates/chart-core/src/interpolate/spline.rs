//! Uniform cubic interpolation with endpoint slopes of curvature inferred from
//! the first and last four samples (FMM boundary conditions).
use super::{ChartResult, DiagnosticCode, count, error, parameter};
/// Owned scalar cubic spline on evenly spaced knots spanning `[0,1]`.
#[derive(Clone, Debug)]
pub struct CubicSpline {
    values: Vec<f64>,
    curvature: Vec<f64>,
}
impl CubicSpline {
    /// Compile the unique C2 spline with endpoint third derivatives matching
    /// the cubic through the adjacent four knots. Two/three knots use a line/parabola.
    pub fn new(values: &[f64]) -> ChartResult<Self> {
        count(values.len(), 2)?;
        if !values.iter().all(|x| x.is_finite()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Spline samples must be finite.",
            ));
        }
        let n = values.len();
        let scale = ((n - 1) as f64).powi(2);
        let mut curvature = vec![0.; n];
        if n == 3 {
            curvature.fill((values[2] - 2. * values[1] + values[0]) * scale);
        }
        if n >= 4 {
            let first = (values[3] - 3. * values[2] + 3. * values[1] - values[0]) * scale;
            let last =
                (values[n - 1] - 3. * values[n - 2] + 3. * values[n - 3] - values[n - 4]) * scale;
            let mut diagonal = vec![4.; n - 2];
            diagonal[0] = 5.;
            diagonal[n - 3] = 5.;
            let mut rhs = values
                .windows(3)
                .map(|p| 6. * (p[2] - 2. * p[1] + p[0]) * scale)
                .collect::<Vec<_>>();
            rhs[0] += first;
            rhs[n - 3] -= last;
            for i in 1..n - 2 {
                let q = 1. / diagonal[i - 1];
                diagonal[i] -= q;
                rhs[i] -= q * rhs[i - 1];
            }
            curvature[n - 2] = rhs[n - 3] / diagonal[n - 3];
            for i in (0..n - 3).rev() {
                curvature[i + 1] = (rhs[i] - curvature[i + 2]) / diagonal[i];
            }
            curvature[0] = curvature[1] - first;
            curvature[n - 1] = curvature[n - 2] + last;
        }
        Ok(Self {
            values: values.to_vec(),
            curvature,
        })
    }
    /// Sample the spline, extending the nearest end polynomial outside `[0,1]`.
    pub fn sample(&self, t: f64) -> ChartResult<f64> {
        parameter(t)?;
        let n = self.values.len();
        let x = t * (n - 1) as f64;
        let i = (x.floor().max(0.) as usize).min(n - 2);
        let b = x - i as f64;
        let a = 1. - b;
        Ok(a * self.values[i]
            + b * self.values[i + 1]
            + ((a * a * a - a) * self.curvature[i] + (b * b * b - b) * self.curvature[i + 1])
                / (6. * ((n - 1) as f64).powi(2)))
    }
}
