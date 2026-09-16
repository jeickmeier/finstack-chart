use super::{DensityControls, DensityKernel};
use crate::ChartResult;
use std::f64::consts::PI;
#[derive(Clone, Debug)]
pub(crate) struct DensityPoint {
    pub x: f64,
    pub density: f64,
    pub scaled: f64,
    pub count: f64,
    pub wdensity: f64,
    pub n: usize,
}
#[derive(Clone, Debug)]
pub(crate) struct DensityEstimate {
    pub points: Vec<DensityPoint>,
    pub removed_bounds: bool,
    pub too_few: bool,
    pub boundary_minimum: bool,
}
#[derive(Clone, Copy, Default)]
struct Complex {
    re: f64,
    im: f64,
}
fn fft(a: &mut [Complex], inverse: bool) {
    let n = a.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            a.swap(i, j);
        }
    }
    let mut size = 2;
    while size <= n {
        let angle = if inverse { 2. } else { -2. } * PI / size as f64;
        for start in (0..n).step_by(size) {
            for k in 0..size / 2 {
                let theta = angle * k as f64;
                let (c, s) = (libm::cos(theta), libm::sin(theta));
                let left = a[start + k];
                let right = a[start + k + size / 2];
                let re = right.re * c - right.im * s;
                let im = right.re * s + right.im * c;
                a[start + k] = Complex {
                    re: left.re + re,
                    im: left.im + im,
                };
                a[start + k + size / 2] = Complex {
                    re: left.re - re,
                    im: left.im - im,
                };
            }
        }
        size *= 2;
    }
}
fn kernel(x: f64, bw: f64, kind: DensityKernel) -> f64 {
    if kind == DensityKernel::Gaussian {
        return libm::exp(-0.5 * (x / bw) * (x / bw)) / (bw * libm::sqrt(2. * PI));
    }
    let a = bw
        * match kind {
            DensityKernel::Epanechnikov => libm::sqrt(5.),
            DensityKernel::Rectangular => libm::sqrt(3.),
            DensityKernel::Triangular => libm::sqrt(6.),
            DensityKernel::Biweight => libm::sqrt(7.),
            DensityKernel::Cosine => 1. / libm::sqrt(1. / 3. - 2. / (PI * PI)),
            DensityKernel::Optcosine => 1. / libm::sqrt(1. - 8. / (PI * PI)),
            _ => unreachable!(),
        };
    let u = x / a;
    if u.abs() >= 1. {
        return 0.;
    }
    match kind {
        DensityKernel::Epanechnikov => 0.75 * (1. - u * u) / a,
        DensityKernel::Rectangular => 0.5 / a,
        DensityKernel::Triangular => (1. - u.abs()) / a,
        DensityKernel::Biweight => 15. / 16. * (1. - u * u) * (1. - u * u) / a,
        DensityKernel::Cosine => (1. + libm::cos(PI * u)) / (2. * a),
        DensityKernel::Optcosine => PI / 4. * libm::cos(PI * u / 2.) / a,
        _ => unreachable!(),
    }
}
fn seq(a: f64, b: f64, n: usize) -> Vec<f64> {
    if n == 1 {
        return vec![a];
    }
    (0..n)
        .map(|i| {
            if i + 1 == n {
                b
            } else {
                a + (b - a) * i as f64 / (n - 1) as f64
            }
        })
        .collect()
}
fn approx(x: f64, grid: &[f64], y: &[f64]) -> f64 {
    if !x.is_finite() || x < grid[0] || x > grid[grid.len() - 1] {
        return 0.;
    }
    let hi = grid.partition_point(|v| *v < x);
    if hi == 0 {
        return y[0];
    }
    if hi == grid.len() {
        return y[y.len() - 1];
    }
    let frac = (x - grid[hi - 1]) / (grid[hi] - grid[hi - 1]);
    (1. - frac) * y[hi - 1] + frac * y[hi]
}
/// Weighted linear binning, padded FFT convolution, interpolation and support reflection.
pub(crate) fn density(
    values: &[f64],
    weights: Option<&[f64]>,
    from: f64,
    to: f64,
    controls: &DensityControls,
    max_grid: usize,
) -> ChartResult<DensityEstimate> {
    if values.iter().any(|v| !v.is_finite())
        || weights.is_some_and(|w| w.len() != values.len() || w.iter().any(|v| !v.is_finite()))
        || !from.is_finite()
        || !to.is_finite()
        || from > to
        || !controls.adjust.is_finite()
        || controls.adjust <= 0.
        || controls.n == 0
        || controls.lower.is_some_and(|v| !v.is_finite())
        || controls.upper.is_some_and(|v| !v.is_finite())
    {
        return Err(super::invalid(
            "Density inputs, positive adjustment, finite ordered grid and support bounds are required.",
        ));
    }
    let total = weights.map_or(values.len() as f64, |w| w.iter().sum());
    let mut xs = vec![];
    let mut ws = vec![];
    for (i, x) in values.iter().enumerate() {
        if controls.lower.is_none_or(|b| *x >= b) && controls.upper.is_none_or(|b| *x <= b) {
            xs.push(*x);
            ws.push(weights.map_or(1., |w| w[i]) / total);
        }
    }
    let removed = xs.len() != values.len();
    let keptmass = ws.iter().sum::<f64>();
    if removed && keptmass > 0. {
        for w in &mut ws {
            *w /= keptmass;
        }
    }
    let mass = if removed { total * keptmass } else { total };
    if xs.len() < 2 {
        return Ok(DensityEstimate {
            points: vec![],
            removed_bounds: removed,
            too_few: true,
            boundary_minimum: false,
        });
    }
    if ws.iter().any(|v| !v.is_finite() || *v < 0.) {
        return Err(super::invalid(
            "Normalized density weights must be finite and nonnegative.",
        ));
    }
    let (selected, boundary_minimum) = super::bandwidth::select(&xs, controls.bandwidth)?;
    let bw = selected * controls.adjust;
    if !bw.is_finite() || bw <= 0. {
        return Err(super::invalid(
            "Density bandwidth must be finite and positive.",
        ));
    }
    let reflect = usize::from(controls.lower.is_some()) + usize::from(controls.upper.is_some());
    let count = controls
        .n
        .checked_mul(reflect + 1)
        .ok_or_else(|| super::invalid("Density grid size overflow."))?;
    let n = count
        .max(512)
        .checked_next_power_of_two()
        .ok_or_else(|| super::invalid("Density FFT size overflow."))?;
    let size = n
        .checked_mul(2)
        .ok_or_else(|| super::invalid("Density FFT size overflow."))?;
    if size > max_grid {
        return Err(super::super::error(
            crate::DiagnosticCode::ResourceLimit,
            "Density FFT grid exceeds configured budget.",
        ));
    }
    let width = to - from;
    let begin = from - if controls.lower.is_some() { width } else { 0. };
    let end = to + if controls.upper.is_some() { width } else { 0. };
    let low = begin - 4. * bw;
    let high = end + 4. * bw;
    let dx = (high - low) / (n - 1) as f64;
    let mut data = vec![Complex::default(); size];
    for (x, w) in xs.iter().zip(&ws) {
        let position = (x - low) / dx;
        let i = position.floor() as isize;
        let frac = position - i as f64;
        if i >= 0 && (i as usize) < n {
            data[i as usize].re += (1. - frac) * w;
        }
        if i + 1 >= 0 && ((i + 1) as usize) < n {
            data[(i + 1) as usize].re += frac * w;
        }
    }
    let mut curve = (0..size)
        .map(|i| Complex {
            re: kernel(
                if i <= n {
                    i as f64 * dx
                } else {
                    -((size - i) as f64) * dx
                },
                bw,
                controls.kernel,
            ),
            im: 0.,
        })
        .collect::<Vec<_>>();
    fft(&mut data, false);
    fft(&mut curve, false);
    for (a, b) in data.iter_mut().zip(curve) {
        let re = a.re * b.re + a.im * b.im;
        let im = a.im * b.re - a.re * b.im;
        *a = Complex { re, im };
    }
    fft(&mut data, true);
    let y = data[..n]
        .iter()
        .map(|v| (v.re / size as f64).max(0.))
        .collect::<Vec<_>>();
    let grid = seq(low, high, n);
    let base_x = seq(begin, end, count);
    let base_y = base_x
        .iter()
        .map(|v| approx(*v, &grid, &y))
        .collect::<Vec<_>>();
    let out_x = if reflect > 0 {
        seq(
            controls.lower.unwrap_or(f64::NEG_INFINITY).max(begin),
            controls.upper.unwrap_or(f64::INFINITY).min(end),
            count,
        )
    } else {
        base_x.clone()
    };
    let out_y = if reflect > 0 {
        out_x
            .iter()
            .map(|v| {
                approx(*v, &base_x, &base_y)
                    + controls
                        .lower
                        .map_or(0., |b| approx(2. * b - v, &base_x, &base_y))
                    + controls
                        .upper
                        .map_or(0., |b| approx(2. * b - v, &base_x, &base_y))
            })
            .collect::<Vec<_>>()
    } else {
        base_y
    };
    let maximum = out_y.iter().copied().fold(0., f64::max);
    let points = out_x
        .into_iter()
        .zip(out_y)
        .map(|(x, d)| DensityPoint {
            x,
            density: d,
            scaled: d / maximum,
            count: d * xs.len() as f64,
            wdensity: d * mass,
            n: xs.len(),
        })
        .collect();
    Ok(DensityEstimate {
        points,
        removed_bounds: removed,
        too_few: false,
        boundary_minimum,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_all_kernel_grids_and_reflection() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/distribution-controls.json"
        ))
        .unwrap();
        for c in f["cases"].as_array().unwrap() {
            let name = c["name"].as_str().unwrap();
            if !name.starts_with("density-") || name.contains("missing") {
                continue;
            }
            let v = &c["controls"];
            let vals = |v: &serde_json::Value| {
                v.as_array().map_or_else(
                    || vec![v.as_f64().unwrap()],
                    |a| {
                        a.iter()
                            .map(|v| v.as_f64().unwrap_or(f64::NAN))
                            .collect::<Vec<_>>()
                    },
                )
            };
            let x = vals(&v["x"]);
            let w = v.get("weight").filter(|v| !v.is_null()).map(vals);
            let mut controls = DensityControls {
                n: v["n"].as_u64().unwrap() as usize,
                ..Default::default()
            };
            if let Some(bw) = v["bw"].as_f64() {
                controls.bandwidth = super::super::Bandwidth::Fixed(bw);
            }
            controls.kernel = match v["kernel"].as_str().unwrap_or("gaussian") {
                "gaussian" => DensityKernel::Gaussian,
                "epanechnikov" => DensityKernel::Epanechnikov,
                "rectangular" => DensityKernel::Rectangular,
                "triangular" => DensityKernel::Triangular,
                "biweight" => DensityKernel::Biweight,
                "cosine" => DensityKernel::Cosine,
                "optcosine" => DensityKernel::Optcosine,
                _ => unreachable!(),
            };
            if let Some(b) = v["bounds"].as_array() {
                controls.lower = b[0].as_f64();
                controls.upper = b[1].as_f64();
            }
            let r = density(
                &x,
                w.as_deref(),
                v["from"].as_f64().unwrap(),
                v["to"].as_f64().unwrap(),
                &controls,
                1 << 20,
            );
            if c["result"]["ok"] == false {
                assert!(r.is_err(), "{name}");
                continue;
            }
            let r = r.unwrap();
            let expected = c["result"]["value"].as_array().unwrap();
            if expected[0]["density"].is_null() {
                assert!(r.too_few);
                continue;
            }
            assert_eq!(r.points.len(), expected.len(), "{name}");
            for (a, b) in r.points.iter().zip(expected) {
                for (k, val) in [
                    ("x", a.x),
                    ("density", a.density),
                    ("scaled", a.scaled),
                    ("count", a.count),
                    ("wdensity", a.wdensity),
                ] {
                    assert!(
                        (val - b[k].as_f64().unwrap()).abs() < 1e-10,
                        "{name} {k} {val} !={}",
                        b[k]
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[test]
    fn fft_working_grid_is_bounded_before_allocation() {
        let controls = DensityControls::default();
        assert!(density(&[0., 1.], None, 0., 1., &controls, 1023).is_err());
        let too_large = DensityControls {
            n: usize::MAX,
            ..Default::default()
        };
        assert!(density(&[0., 1.], None, 0., 1., &too_large, usize::MAX).is_err());
        let reflected = DensityControls {
            lower: Some(0.),
            upper: Some(1.),
            ..Default::default()
        };
        assert!(density(&[0., 1.], None, 0., 1., &reflected, 1024).is_err());
    }
}
