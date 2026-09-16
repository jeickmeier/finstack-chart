use super::Bandwidth;
use crate::ChartResult;
use std::f64::consts::PI;
fn pow(x: f64, y: f64) -> f64 {
    libm::pow(x, y)
}
/// Reference bandwidth selectors; weights intentionally do not enter selection.
pub(crate) fn select(x: &[f64], method: Bandwidth) -> ChartResult<(f64, bool)> {
    if let Bandwidth::Fixed(h) = method {
        return if h.is_finite() && h > 0. {
            Ok((h, false))
        } else {
            Err(super::invalid("Bandwidth must be finite and positive."))
        };
    }
    if x.len() < 2 || x.iter().any(|v| !v.is_finite()) {
        return Err(super::invalid(
            "Bandwidth selection requires two finite observations.",
        ));
    }
    let n = x.len() as f64;
    let mean = x.iter().sum::<f64>() / n;
    let sd = libm::sqrt(x.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / (n - 1.));
    let mut sorted = x.to_vec();
    sorted.sort_by(f64::total_cmp);
    let iqr = super::quantile(&sorted, 0.75, 7)? - super::quantile(&sorted, 0.25, 7)?;
    if matches!(method, Bandwidth::Nrd0 | Bandwidth::Nrd) {
        let mut scale = sd.min(iqr / 1.34);
        if method == Bandwidth::Nrd0 && scale == 0. {
            scale = if sd > 0. {
                sd
            } else if x[0] != 0. {
                x[0].abs()
            } else {
                1.
            };
        }
        let h = if method == Bandwidth::Nrd0 { 0.9 } else { 1.06 } * scale * pow(n, -0.2);
        return Ok((h, false));
    }
    let d = (sorted[sorted.len() - 1] - sorted[0]) * 1.01 / 1000.;
    if d <= 0. || !d.is_finite() {
        return Err(super::invalid(
            "Bandwidth distance bins require nonconstant finite data.",
        ));
    }
    let offset = if x.len() <= 500
        && (sorted[0] / d < i32::MIN as f64 || sorted[sorted.len() - 1] / d > i32::MAX as f64)
    {
        sorted[0]
    } else {
        0.
    };
    let indices = x
        .iter()
        .map(|v| ((v - offset) / d).trunc() as i64)
        .collect::<Vec<_>>();
    let minimum = *indices.iter().min().unwrap();
    let mut bins = [0_f64; 1000];
    for i in indices {
        let k = (i - minimum) as usize;
        if k < 1000 {
            bins[k] += 1.;
        }
    }
    let mut counts = [0_f64; 1000];
    for i in 0..1000 {
        counts[0] += bins[i] * (bins[i] - 1.) / 2.;
        for j in 0..i {
            counts[i - j] += bins[i] * bins[j];
        }
    }
    let derivative = |h: f64, degree: u8| {
        let mut sum = 0.;
        for (i, c) in counts.iter().enumerate() {
            let u = i as f64 * d / h;
            let z = u * u;
            if z >= 1000. {
                break;
            }
            let polynomial = if degree == 4 {
                z * z - 6. * z + 3.
            } else {
                z * z * z - 15. * z * z + 45. * z - 15.
            };
            sum += c * libm::exp(-z / 2.) * polynomial;
        }
        let diagonal = if degree == 4 { 3. * n } else { -15. * n };
        (2. * sum + diagonal) / (n * (n - 1.) * pow(h, degree as f64 + 1.) * libm::sqrt(2. * PI))
    };
    if matches!(method, Bandwidth::Ucv | Bandwidth::Bcv) {
        let upper = 1.144 * sd * pow(n, -0.2);
        let lower = 0.1 * upper;
        let tol = 0.1 * lower;
        let objective = |h: f64| {
            let mut sum = 0.;
            for (i, c) in counts.iter().enumerate() {
                let z = pow(i as f64 * d / h, 2.);
                if z >= 1000. {
                    break;
                }
                sum += c * if method == Bandwidth::Ucv {
                    libm::exp(-z / 4.) - libm::sqrt(8.) * libm::exp(-z / 2.)
                } else {
                    libm::exp(-z / 4.) * (z * z - 12. * z + 12.)
                };
            }
            if method == Bandwidth::Ucv {
                (0.5 + sum / n) / (n * h * libm::sqrt(PI))
            } else {
                (1. + sum / (32. * n)) / (2. * n * h * libm::sqrt(PI))
            }
        };
        let h = minimize(objective, lower, upper, tol)?;
        return Ok((h, h < lower + tol || h > upper - tol));
    }
    let scale = sd.min(iqr / 1.349);
    let a = 1.24 * scale * pow(n, -1. / 7.);
    let b = 1.23 * scale * pow(n, -1. / 9.);
    let c1 = 1. / (2. * libm::sqrt(PI) * n);
    let td = -derivative(b, 6);
    if !td.is_finite() || td <= 0. {
        return Err(super::invalid(
            "Sample is too sparse for Sheather-Jones pilot estimation.",
        ));
    }
    if method == Bandwidth::SjDpi {
        return Ok((
            pow(c1 / derivative(pow(2.394 / (n * td), 1. / 7.), 4), 0.2),
            false,
        ));
    }
    let alpha = 1.357 * pow(derivative(a, 4) / td, 1. / 7.);
    let f = |h: f64| pow(c1 / derivative(alpha * pow(h, 5. / 7.), 4), 0.2) - h;
    let mut upper = 1.144 * scale * pow(n, -0.2);
    let mut lower = 0.1 * upper;
    let tolerance = 0.1 * lower;
    for i in 1..=100 {
        if f(lower) * f(upper) <= 0. {
            return Ok((root(f, lower, upper, tolerance)?, false));
        }
        if i % 2 == 1 {
            upper *= 1.2;
        } else {
            lower /= 1.2;
        }
    }
    Err(super::invalid(
        "Sheather-Jones bandwidth root could not be bracketed.",
    ))
}
// Bounded Brent interpolation with golden-section fallback.
fn minimize(f: impl Fn(f64) -> f64, mut a: f64, mut b: f64, tol: f64) -> ChartResult<f64> {
    let golden = (3. - libm::sqrt(5.)) / 2.;
    let mut x = a + golden * (b - a);
    let (mut w, mut v) = (x, x);
    let (mut fx, mut fw, mut fv) = (f(x), f(x), f(x));
    let (mut step, mut previous) = (0_f64, 0_f64);
    for _ in 0..1000 {
        let mid = (a + b) / 2.;
        let t = libm::sqrt(f64::EPSILON) * x.abs() + tol / 3.;
        if (x - mid).abs() <= 2. * t - (b - a) / 2. {
            return Ok(x);
        }
        let mut parabolic = false;
        if previous.abs() > t {
            let r = (x - w) * (fx - fv);
            let q = (x - v) * (fx - fw);
            let mut p = (x - v) * q - (x - w) * r;
            let mut q = 2. * (q - r);
            if q > 0. {
                p = -p;
            } else {
                q = -q;
            }
            let saved = previous;
            previous = step;
            if p.abs() < (0.5 * q * saved).abs() && p > q * (a - x) && p < q * (b - x) {
                step = p / q;
                let u = x + step;
                if u - a < 2. * t || b - u < 2. * t {
                    step = if x < mid { t } else { -t };
                }
                parabolic = true;
            }
        }
        if !parabolic {
            previous = if x < mid { b - x } else { a - x };
            step = golden * previous;
        }
        let u = x + if step.abs() >= t {
            step
        } else if step > 0. {
            t
        } else {
            -t
        };
        let fu = f(u);
        if !fu.is_finite() {
            return Err(super::invalid("Nonfinite bandwidth objective."));
        }
        if fu <= fx {
            if u < x {
                b = x;
            } else {
                a = x;
            }
            v = w;
            fv = fw;
            w = x;
            fw = fx;
            x = u;
            fx = fu;
        } else {
            if u < x {
                a = u;
            } else {
                b = u;
            }
            if fu <= fw || w == x {
                v = w;
                fv = fw;
                w = u;
                fw = fu;
            } else if fu <= fv || v == x || v == w {
                v = u;
                fv = fu;
            }
        }
    }
    Err(super::invalid("Bandwidth minimization did not converge."))
}
fn root(f: impl Fn(f64) -> f64, mut a: f64, mut b: f64, tol: f64) -> ChartResult<f64> {
    let (mut fa, mut fb) = (f(a), f(b));
    let (mut c, mut fc) = (a, fa);
    for _ in 0..1000 {
        let old = b - a;
        if fb.abs() > fc.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }
        let actual = 2. * f64::EPSILON * b.abs() + tol / 2.;
        let mut step = (c - b) / 2.;
        if step.abs() <= actual || fb == 0. {
            return Ok(b);
        }
        if old.abs() >= actual && fa.abs() > fb.abs() {
            let mut p;
            let mut q;
            if a == c {
                let t = fb / fa;
                p = (c - b) * t;
                q = 1. - t;
            } else {
                let q0 = fa / fc;
                let t = fb / fc;
                let s = fb / fa;
                p = s * ((c - b) * q0 * (q0 - t) - (b - a) * (t - 1.));
                q = (q0 - 1.) * (t - 1.) * (s - 1.);
            }
            if p > 0. {
                q = -q;
            } else {
                p = -p;
            }
            if p < (0.75 * (c - b) * q - (actual * q).abs()) && p < (old * q / 2.).abs() {
                step = p / q;
            }
        }
        if step.abs() < actual {
            step = if step > 0. { actual } else { -actual };
        }
        a = b;
        fa = fb;
        b += step;
        fb = f(b);
        if (fb > 0. && fc > 0.) || (fb < 0. && fc < 0.) {
            c = a;
            fc = fa;
        }
    }
    Err(super::invalid("Bandwidth root did not converge."))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_bandwidth_selectors() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../fixtures/parity/ggplot2/distribution-controls.json"
        ))
        .unwrap();
        for c in f["cases"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["name"].as_str().unwrap().starts_with("bandwidth-"))
        {
            let x = c["controls"]["x"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>();
            let method = match c["controls"]["bw"].as_str().unwrap() {
                "nrd0" => Bandwidth::Nrd0,
                "nrd" => Bandwidth::Nrd,
                "ucv" => Bandwidth::Ucv,
                "bcv" => Bandwidth::Bcv,
                "SJ-ste" => Bandwidth::SjSte,
                "SJ-dpi" => Bandwidth::SjDpi,
                _ => unreachable!(),
            };
            let actual = select(&x, method).unwrap().0;
            let expected = c["result"]["value"]["bw"].as_f64().unwrap();
            assert!(
                (actual - expected).abs() < 1e-10,
                "{method:?} actual{actual} expected{expected}"
            );
        }
    }
}
