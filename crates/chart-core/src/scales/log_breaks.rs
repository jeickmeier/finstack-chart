//! ggplot2 logarithmic breaks, adapted from scales 1.4.0; see SCALES-LICENSE.
use super::{ChartResult, DiagnosticCode, error, ggplot_breaks_extended};
fn logarithm(x: f64, base: f64) -> f64 {
    if base == 10. {
        x.log10()
    } else if base == 2. {
        x.log2()
    } else {
        x.ln() / base.ln()
    }
}
fn series(lo: f64, hi: f64, step: f64, base: f64) -> ChartResult<Vec<f64>> {
    let n = ((hi - lo) / step + 1e-10).floor();
    if !n.is_finite() || !(0. ..200_000.).contains(&n) {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Log break exponent sequence exceeds its budget.",
        ));
    }
    Ok((0..=n as usize)
        .map(|i| pxfm::f_pow(base, lo + i as f64 * step))
        .collect())
}
/// ggplot2 major logarithmic breaks, adding reference intermediate factors when
/// powers alone are too sparse. The returned list can retain one outside neighbor.
pub fn ggplot_breaks_log(
    domain: [f64; 2],
    count: f64,
    base: f64,
    max_ticks: usize,
) -> ChartResult<Vec<f64>> {
    builtin_breaks_log(domain, count, base, max_ticks, false)
}
/// Built-in transform bases may decrease; retain the reference exponent direction.
pub(crate) fn builtin_breaks_log(
    domain: [f64; 2],
    count: f64,
    base: f64,
    max_ticks: usize,
    decreasing: bool,
) -> ChartResult<Vec<f64>> {
    if !count.is_finite()
        || count <= 0.
        || count > max_ticks as f64
        || !base.is_finite()
        || base <= 0.
        || base == 1.
        || (base < 1. && !decreasing)
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Log breaks require nonnegative finite limits, base greater than one and a bounded positive count.",
        ));
    }
    let domain = match domain.map(f64::is_nan) {
        [false, false] => domain,
        [false, true] => [domain[0]; 2],
        [true, false] => [domain[1]; 2],
        [true, true] => return Ok(vec![]),
    };
    if domain.iter().any(|v| !v.is_finite()) {
        return Ok(vec![]);
    }
    if domain.iter().any(|v| *v < 0.) {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Log breaks require a nonnegative range.",
        ));
    }
    let [a, b] = domain;
    let lo = logarithm(a.min(b), base);
    let hi = logarithm(a.max(b), base);
    let min = lo.floor();
    let max = hi.ceil();
    let raw = [pxfm::f_pow(base, lo), pxfm::f_pow(base, hi)];
    let relevant = |values: &[f64]| {
        values
            .iter()
            .filter(|x| raw[0] <= **x && **x <= raw[1])
            .count()
    };
    let finish = |values: Vec<f64>| {
        if values.len() <= max_ticks {
            Ok(values)
        } else {
            Err(error(
                DiagnosticCode::ResourceLimit,
                "Log break output exceeds its tick budget.",
            ))
        }
    };
    if min == max {
        return finish(vec![pxfm::f_pow(base, min)]);
    }
    if !min.is_finite() || !max.is_finite() {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Nonconstant log breaks require a finite lower exponent.",
        ));
    }
    let mut by = ((max - min) / count).floor() + 1.;
    if by == 0. || (max - min) / by < 0. {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Log break exponent step has an invalid direction.",
        ));
    }
    let mut iterations = 0;
    loop {
        iterations += 1;
        if iterations > 200_000 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Log break search exceeds its work budget.",
            ));
        }
        let breaks = series(min, max, by, base)?;
        if relevant(&breaks) as f64 >= count - 2. {
            return finish(breaks);
        }
        if by <= 1. {
            break;
        }
        by -= 1.;
    }
    if base <= 2. {
        return finish(series(min, max, if max < min { -1. } else { 1. }, base)?);
    }
    if base > 10_000. {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Log intermediate-factor search exceeds its candidate budget.",
        ));
    }
    let mut steps = vec![1.];
    let mut candidates = (2..=base.floor() as usize)
        .map(|i| i as f64)
        .filter(|v| *v < base)
        .collect::<Vec<_>>();
    let powers = series(min, max, 1., base)?;
    let mut breaks = vec![];
    while !candidates.is_empty() {
        let mut best = 0;
        let mut best_delta = f64::NEG_INFINITY;
        for (i, value) in candidates.iter().enumerate() {
            iterations += steps.len();
            if iterations > 2_000_000 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Log intermediate-factor search exceeds its work budget.",
                ));
            }
            let mut sorted = steps.clone();
            sorted.extend([*value, base]);
            sorted.sort_by(f64::total_cmp);
            let delta = sorted
                .windows(2)
                .map(|p| logarithm(p[1], base) - logarithm(p[0], base))
                .fold(f64::INFINITY, f64::min);
            if delta > best_delta {
                best_delta = delta;
                best = i;
            }
        }
        steps.push(candidates.remove(best));
        if powers.len().saturating_mul(steps.len()) > 200_000 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Log intermediate breaks exceed their output work budget.",
            ));
        }
        breaks = steps
            .iter()
            .flat_map(|s| powers.iter().map(move |p| s * p))
            .collect();
        if relevant(&breaks) as f64 >= count - 2. {
            break;
        }
    }
    if relevant(&breaks) as f64 >= count - 2. {
        breaks.sort_by(f64::total_cmp);
        let first = breaks.partition_point(|x| *x < raw[0]).saturating_sub(1);
        let last = breaks
            .partition_point(|x| *x <= raw[1])
            .min(breaks.len() - 1);
        finish(breaks[first..=last].to_vec())
    } else {
        ggplot_breaks_extended(raw, count, max_ticks)
    }
}
