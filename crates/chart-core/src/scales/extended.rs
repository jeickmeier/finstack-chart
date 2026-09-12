//! Extended Wilkinson breaks, adapted from labeling 0.4.3 (Justin Talbot).
//! The MIT notice is retained in LABELING-LICENSE. Search and output are bounded.
use super::{ChartResult, DiagnosticCode, error};

/// ggplot2's default continuous break search, with labeling's fixed preferred
/// steps and score weights. Desired count may differ from the returned count.
pub fn ggplot_breaks_extended(
    domain: [f64; 2],
    count: f64,
    max_ticks: usize,
) -> ChartResult<Vec<f64>> {
    let domain = match domain.map(f64::is_finite) {
        [true, true] => domain,
        [true, false] => [domain[0]; 2],
        [false, true] => [domain[1]; 2],
        [false, false] => return Ok(vec![]),
    };
    extended(domain, count, max_ticks, &[1., 5., 2., 2.5, 4., 3.])
}

/// Reference elapsed-duration breaks in seconds. The unit changes with the visible
/// span; this is independent of calendar dates and timezone resources.
pub fn ggplot_breaks_duration(
    domain: [f64; 2],
    count: f64,
    max_ticks: usize,
) -> ChartResult<Vec<f64>> {
    let span = (domain[1] - domain[0]).abs();
    let unit = if span <= 120. {
        1.
    } else if span <= 7200. {
        60.
    } else if span <= 172800. {
        3600.
    } else if span <= 1209600. {
        86400.
    } else {
        604800.
    };
    extended(
        domain.map(|v| v / unit),
        count,
        max_ticks,
        &[1., 2., 1.5, 4., 3.],
    )
    .map(|values| values.into_iter().map(|v| v * unit).collect())
}

fn extended(
    domain: [f64; 2],
    count: f64,
    max_ticks: usize,
    preferred: &[f64],
) -> ChartResult<Vec<f64>> {
    if !count.is_finite()
        || count < 1.
        || count > max_ticks as f64
        || !domain.iter().all(|x| x.is_finite())
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Extended breaks require finite limits and a desired count from one through the tick budget.",
        ));
    }
    let [mut lo, mut hi] = domain;
    if lo > hi {
        std::mem::swap(&mut lo, &mut hi);
    }
    let span = hi - lo;
    if span < 100. * f64::EPSILON || span > f64::MAX.sqrt() {
        let length = count.ceil() as usize;
        if length == 1 {
            return Ok(vec![lo]);
        }
        return Ok((0..length)
            .map(|i| {
                let t = i as f64 / (length - 1) as f64;
                lo * (1. - t) + hi * t
            })
            .collect());
    }
    if count == 1. {
        return Err(error(
            DiagnosticCode::Validation,
            "A nondegenerate extended break search requires a desired count greater than one.",
        ));
    }
    let desired = count;
    let mut best_score = -2.;
    let mut best = None;
    let mut work = 0usize;
    let mut charge = || -> ChartResult<()> {
        work += 1;
        if work > 2_000_000 {
            Err(error(
                DiagnosticCode::ResourceLimit,
                "Extended break search exceeds its work budget.",
            ))
        } else {
            Ok(())
        }
    };
    let mut j = 1.;
    'search: loop {
        for (index, q) in preferred.iter().copied().enumerate() {
            charge()?;
            let simplicity_max = 1. - index as f64 / (preferred.len() - 1) as f64 - j + 1.;
            if 0.25 * simplicity_max + 0.2 + 0.5 + 0.05 < best_score {
                break 'search;
            }
            let mut k = 2.;
            loop {
                charge()?;
                let density_max = if k >= desired {
                    2. - (k - 1.) / (desired - 1.)
                } else {
                    1.
                };
                if 0.25 * simplicity_max + 0.2 + 0.5 * density_max + 0.05 < best_score {
                    break;
                }
                let delta = span / (k + 1.) / j / q;
                let mut exponent = delta.log10().ceil();
                loop {
                    charge()?;
                    let step = j * q * pxfm::f_pow(10., exponent);
                    if !step.is_finite() || step <= 0. {
                        break;
                    }
                    let candidate_span = step * (k - 1.);
                    let half = (candidate_span - span) / 2.;
                    let coverage_max = if candidate_span > span {
                        1. - half * half / (0.1 * span).powi(2)
                    } else {
                        1.
                    };
                    if 0.25 * simplicity_max + 0.2 * coverage_max + 0.5 * density_max + 0.05
                        < best_score
                    {
                        break;
                    }
                    let min_start = (hi / step).floor() * j - (k - 1.) * j;
                    let max_start = (lo / step).ceil() * j;
                    if min_start <= max_start {
                        let starts = max_start - min_start;
                        if !starts.is_finite() || starts > 2_000_000. {
                            return Err(error(
                                DiagnosticCode::ResourceLimit,
                                "Extended break candidates exceed their work budget.",
                            ));
                        }
                        for i in 0..=starts.floor() as usize {
                            charge()?;
                            let start = min_start + i as f64;
                            let a = start * (step / j);
                            let b = a + step * (k - 1.);
                            let remainder = a.rem_euclid(step);
                            let zero = if (remainder < 100. * f64::EPSILON
                                || step - remainder < 100. * f64::EPSILON)
                                && a <= 0.
                                && b >= 0.
                            {
                                1.
                            } else {
                                0.
                            };
                            let simplicity =
                                1. - index as f64 / (preferred.len() - 1) as f64 - j + zero;
                            let coverage = 1.
                                - 0.5 * ((hi - b).powi(2) + (lo - a).powi(2))
                                    / (0.1 * span).powi(2);
                            let actual_density = (k - 1.) / (b - a);
                            let target_density = (desired - 1.) / (b.max(hi) - a.min(lo));
                            let density = 2.
                                - (actual_density / target_density)
                                    .max(target_density / actual_density);
                            let score = 0.25 * simplicity + 0.2 * coverage + 0.5 * density + 0.05;
                            if score > best_score {
                                best_score = score;
                                best = Some((a, b, step));
                            }
                        }
                    }
                    exponent += 1.;
                }
                k += 1.;
            }
        }
        j += 1.;
    }
    let (a, b, step) = best.ok_or_else(|| {
        error(
            DiagnosticCode::PrecisionLoss,
            "No finite extended break layout could be resolved.",
        )
    })?;
    let intervals = ((b - a) / step + 1e-10).floor();
    if !intervals.is_finite() || intervals < 0. || intervals >= max_ticks as f64 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Extended break output exceeds its tick budget.",
        ));
    }
    Ok((0..=intervals as usize)
        .map(|i| (a + i as f64 * step).min(b))
        .collect())
}
