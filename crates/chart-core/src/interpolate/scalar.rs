use super::{ChartResult, count, parameter};

/// Pure sampling contract. Implementations return owned results; custom native
/// factories can implement it without becoming serializable portable operations.
pub trait Sample<T> {
    /// Evaluate a finite parameter; the family defines extrapolation or clamping.
    fn sample(&self, t: f64) -> ChartResult<T>;
}
impl<T, F: Fn(f64) -> ChartResult<T>> Sample<T> for F {
    fn sample(&self, t: f64) -> ChartResult<T> {
        self(t)
    }
}

/// Shared scalar factory, prepared once and safe to copy independently.
#[derive(Clone, Debug)]
pub struct ScalarInterpolator {
    kernel: Kernel,
}
#[derive(Clone, Debug)]
enum Kernel {
    Power {
        a: f64,
        b: f64,
        exponent: f64,
        absolute: bool,
    },
    Number {
        a: f64,
        b: f64,
        round: bool,
    },
    Basis {
        controls: Vec<f64>,
        closed: bool,
    },
}
impl ScalarInterpolator {
    /// Power of the normalized parameter followed by affine range interpolation.
    /// Negative bases with fractional exponents yield NaN unless absolute is selected.
    pub fn power(a: f64, b: f64, exponent: f64, absolute: bool) -> ChartResult<Self> {
        if ![a, b, exponent].iter().all(|v| v.is_finite()) {
            return Err(super::error(
                crate::DiagnosticCode::NumericalDomain,
                "Power range parameters must be finite.",
            ));
        }
        Ok(Self {
            kernel: Kernel::Power {
                a,
                b,
                exponent,
                absolute,
            },
        })
    }
    /// Weighted binary64 interpolation, including IEEE exceptional endpoint values.
    pub fn number(a: f64, b: f64) -> Self {
        Self {
            kernel: Kernel::Number { a, b, round: false },
        }
    }
    /// Weighted interpolation followed by JS rounding (ties toward positive infinity).
    pub fn round(a: f64, b: f64) -> Self {
        Self {
            kernel: Kernel::Number { a, b, round: true },
        }
    }
    /// Open uniform cubic basis, clamped at the endpoint control values.
    pub fn basis(controls: Vec<f64>) -> ChartResult<Self> {
        count(controls.len(), 2)?;
        Ok(Self {
            kernel: Kernel::Basis {
                controls,
                closed: false,
            },
        })
    }
    /// Periodic uniform cubic basis; a singleton supplies a constant control cycle.
    pub fn basis_closed(controls: Vec<f64>) -> ChartResult<Self> {
        count(controls.len(), 1)?;
        Ok(Self {
            kernel: Kernel::Basis {
                controls,
                closed: true,
            },
        })
    }
    /// Evaluate the compiled scalar factory without mutating controls.
    pub fn sample(&self, t: f64) -> ChartResult<f64> {
        parameter(t)?;
        Ok(self.evaluate(t))
    }
    pub(super) fn evaluate(&self, t: f64) -> f64 {
        match &self.kernel {
            Kernel::Power {
                a,
                b,
                exponent,
                absolute,
            } => {
                let t = if *absolute { t.abs() } else { t };
                // Reference area palettes use sqrt, whose negative-infinity
                // result differs from the generic IEEE power operation.
                let powered = if *exponent == 0.5 {
                    t.sqrt()
                } else {
                    pxfm::f_pow(t, *exponent)
                };
                a + (b - a) * powered
            }
            Kernel::Number { a, b, round } => {
                let v = number(*a, *b, t);
                if *round { js_round(v) } else { v }
            }
            Kernel::Basis { controls, closed } => spline(controls, *closed, t),
        }
    }
}
impl Sample<f64> for ScalarInterpolator {
    fn sample(&self, t: f64) -> ChartResult<f64> {
        self.sample(t)
    }
}
pub(super) fn number(a: f64, b: f64, t: f64) -> f64 {
    a * (1. - t) + b * t
}
pub(crate) fn js_round(v: f64) -> f64 {
    if !v.is_finite() || v == 0. {
        return v;
    }
    let floor = v.floor();
    let result = if v - floor < 0.5 { floor } else { floor + 1. };
    if result == 0. {
        result.copysign(v)
    } else {
        result
    }
}
fn basis(t: f64, v0: f64, v1: f64, v2: f64, v3: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    ((1. - 3. * t + 3. * t2 - t3) * v0
        + (4. - 6. * t2 + 3. * t3) * v1
        + (1. + 3. * t + 3. * t2 - 3. * t3) * v2
        + t3 * v3)
        / 6.
}
fn spline(values: &[f64], closed: bool, t: f64) -> f64 {
    if closed {
        let n = values.len();
        let mut t = t % 1.;
        if t < 0. {
            t += 1.;
        }
        let i = (t * n as f64).floor() as usize;
        basis(
            (t - i as f64 / n as f64) * n as f64,
            values[(i + n - 1) % n],
            values[i % n],
            values[(i + 1) % n],
            values[(i + 2) % n],
        )
    } else {
        let n = values.len() - 1;
        let t = t.clamp(0., 1.);
        let i = if t == 1. {
            n - 1
        } else {
            (t * n as f64).floor() as usize
        };
        let v1 = values[i];
        let v2 = values[i + 1];
        let v0 = if i > 0 { values[i - 1] } else { 2. * v1 - v2 };
        let v3 = if i < n - 1 {
            values[i + 2]
        } else {
            2. * v2 - v1
        };
        basis((t - i as f64 / n as f64) * n as f64, v0, v1, v2, v3)
    }
}

/// Saturating target selection; each returned value is an independent clone.
#[derive(Clone, Debug)]
pub struct Discrete<T> {
    values: Vec<T>,
}
impl<T: Clone> Discrete<T> {
    /// Retain a nonempty, bounded sequence.
    pub fn new(values: Vec<T>) -> ChartResult<Self> {
        count(values.len(), 1)?;
        Ok(Self { values })
    }
    /// Select the floor of `t * len`, saturating at the first and last value.
    pub fn sample(&self, t: f64) -> ChartResult<T> {
        parameter(t)?;
        let i = (t * self.values.len() as f64)
            .floor()
            .clamp(0., (self.values.len() - 1) as f64) as usize;
        Ok(self.values[i].clone())
    }
}
impl<T: Clone> Sample<T> for Discrete<T> {
    fn sample(&self, t: f64) -> ChartResult<T> {
        self.sample(t)
    }
}

/// Piecewise composition of adjacent compiled factories. Only the first and last
/// segment extrapolate; factory construction never occurs during sampling.
#[derive(Clone, Debug)]
pub struct Piecewise<I> {
    segments: Vec<I>,
}
impl<I> Piecewise<I> {
    /// Build exactly one factory for each adjacent control pair.
    pub fn new<T>(
        values: &[T],
        mut factory: impl FnMut(&T, &T) -> ChartResult<I>,
    ) -> ChartResult<Self> {
        count(values.len(), 2)?;
        let segments = values
            .windows(2)
            .map(|v| factory(&v[0], &v[1]))
            .collect::<ChartResult<_>>()?;
        Ok(Self { segments })
    }
    pub(super) fn segments(&self) -> &[I] {
        &self.segments
    }
    /// Sample the selected segment at its locally normalized parameter.
    pub fn sample<T>(&self, t: f64) -> ChartResult<T>
    where
        I: Sample<T>,
    {
        let (segment, t) = self.segment(t)?;
        segment.sample(t)
    }
    pub(super) fn segment(&self, t: f64) -> ChartResult<(&I, f64)> {
        parameter(t)?;
        self.scale_segment(t)
    }
    pub(super) fn scale_segment(&self, t: f64) -> ChartResult<(&I, f64)> {
        if t.is_nan() {
            parameter(t)?;
        }
        let t = t * self.segments.len() as f64;
        let i = t.floor().clamp(0., (self.segments.len() - 1) as f64) as usize;
        Ok((&self.segments[i], t - i as f64))
    }
}
impl<T, I: Sample<T>> Sample<T> for Piecewise<I> {
    fn sample(&self, t: f64) -> ChartResult<T> {
        self.sample(t)
    }
}

/// Independently owned, uniformly spaced samples including both endpoints.
pub fn quantize<T>(interpolator: &impl Sample<T>, samples: usize) -> ChartResult<Vec<T>> {
    count(samples, 2)?;
    (0..samples)
        .map(|i| interpolator.sample(i as f64 / (samples - 1) as f64))
        .collect()
}
