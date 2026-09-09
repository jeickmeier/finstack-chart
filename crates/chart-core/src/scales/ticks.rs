//! Shared D3 numeric tick candidates, step and nice preparation.
//! Adapted from d3-array 3.2.4; its ISC notice is retained in `LICENSE-d3-array`.
use super::error;
use crate::interpolate::js_round;
use crate::{
    ChartResult, DiagnosticCode,
    typography::{NumericFormatter, NumericLocale, NumericSpecifier},
};

/// Signed D3 tick step. The requested count is a hint, including fractional counts.
pub fn tick_step(start: f64, stop: f64, count: f64) -> f64 {
    let reverse = stop < start;
    let inc = if reverse {
        tick_spec(stop, start, count).2
    } else {
        tick_spec(start, stop, count).2
    };
    (if reverse { -1. } else { 1. }) * if inc < 0. { 1. / -inc } else { inc }
}
/// Raw data-space candidates, before any guide collision thinning. A hard budget never changes the step.
pub fn tick_candidates(start: f64, stop: f64, count: f64, budget: usize) -> ChartResult<Vec<f64>> {
    if count.is_nan() || count <= 0. {
        return Ok(Vec::new());
    }
    if start == stop {
        if budget == 0 {
            return Err(budget_error());
        }
        return Ok(vec![start]);
    }
    let reverse = stop < start;
    let (i1, i2, inc) = if reverse {
        tick_spec(stop, start, count)
    } else {
        tick_spec(start, stop, count)
    };
    if i2.partial_cmp(&i1).is_none_or(|order| order.is_lt()) {
        return Ok(Vec::new());
    }
    let n = i2 - i1 + 1.;
    if !n.is_finite() || n > budget.min(1_000_000) as f64 {
        return Err(budget_error());
    }
    Ok((0..n as usize)
        .map(|i| {
            let i = if reverse {
                i2 - i as f64
            } else {
                i1 + i as f64
            };
            if inc < 0. { i / -inc } else { i * inc }
        })
        .collect())
}
fn budget_error() -> crate::Diagnostic {
    error(
        DiagnosticCode::ResourceLimit,
        "Raw tick candidates exceed the independent resource budget.",
    )
}
impl super::NumericFamily {
    /// Tick candidates use the first/last data endpoints; interior knots do not change the grid.
    pub fn ticks(
        self,
        domain: &[crate::interpolate::Number],
        count: f64,
        budget: usize,
    ) -> ChartResult<Vec<f64>> {
        let start = domain.first().map_or(f64::NAN, |n| n.0);
        let stop = domain.last().map_or(f64::NAN, |n| n.0);
        if let Self::Log { base } = self {
            log_tick_candidates(start, stop, base, count, budget)
        } else {
            tick_candidates(start, stop, count, budget)
        }
    }
    /// Tick labels are prepared separately from projection or collision thinning.
    pub fn tick_format(
        self,
        domain: &[crate::interpolate::Number],
        count: f64,
        specifier: Option<&str>,
        locale: NumericLocale,
    ) -> ChartResult<NumericFormatter> {
        let start = domain.first().map_or(f64::NAN, |n| n.0);
        let stop = domain.last().map_or(f64::NAN, |n| n.0);
        if let Self::Log { base } = self {
            log_tick_format(start, stop, base, count, specifier, locale)
        } else {
            crate::typography::tick_format(start, stop, count, specifier, locale)
        }
    }
}
pub(crate) fn log_value(base: f64, negative: bool, x: f64) -> f64 {
    let x = if negative { -x } else { x };
    let value = if base == 10. {
        libm::log10(x)
    } else if base == 2. {
        libm::log2(x)
    } else if base == std::f64::consts::E {
        libm::log(x)
    } else {
        libm::log(x) / libm::log(base)
    };
    if negative { -value } else { value }
}
pub(crate) fn log_power(base: f64, negative: bool, x: f64) -> f64 {
    let x = if negative { -x } else { x };
    let value = if base == 10. {
        if x.is_finite() {
            format!("1e{}", crate::number::ecmascript(x))
                .parse()
                .unwrap_or(f64::NAN)
        } else if x < 0. {
            0.
        } else {
            x
        }
    } else if base == std::f64::consts::E {
        pxfm::f_exp(x)
    } else {
        crate::number::ecma_pow(base, x)
    };
    if negative { -value } else { value }
}
/// Raw signed-log major/minor candidates using the reference base and count semantics.
pub fn log_tick_candidates(
    start: f64,
    stop: f64,
    base: f64,
    count: f64,
    budget: usize,
) -> ChartResult<Vec<f64>> {
    let negative = start < 0.;
    let reverse = stop < start;
    let (u, v) = if reverse {
        (stop, start)
    } else {
        (start, stop)
    };
    let (mut i, mut j) = (log_value(base, negative, u), log_value(base, negative, v));
    let mut ticks = Vec::new();
    if base % 1. == 0. && j - i < count {
        i = i.floor();
        j = j.ceil();
        // Bound iteration as well as retained output, including invalid/infinite log domains.
        let work = (j - i + 1.).max(0.) * (base - 1.).max(0.);
        if !i.is_finite() || !j.is_finite() || !work.is_finite() || work > 1_000_000. {
            return Err(budget_error());
        }
        while i <= j {
            let mut k = if u > 0. { 1. } else { base - 1. };
            while if u > 0. { k < base } else { k >= 1. } {
                let t = if if u > 0. { i < 0. } else { i > 0. } {
                    k / log_power(base, negative, -i)
                } else {
                    k * log_power(base, negative, i)
                };
                if t > v {
                    break;
                }
                if t >= u {
                    if ticks.len() >= budget.min(1_000_000) {
                        return Err(budget_error());
                    }
                    ticks.push(t);
                }
                k += if u > 0. { 1. } else { -1. };
            }
            i += 1.;
        }
        if (ticks.len() as f64) * 2. < count {
            ticks = tick_candidates(u, v, count, budget)?;
        }
    } else {
        let n = if (j - i).is_nan() || count.is_nan() {
            f64::NAN
        } else {
            (j - i).min(count)
        };
        ticks = tick_candidates(i, j, n, budget)?
            .into_iter()
            .map(|v| log_power(base, negative, v))
            .collect();
    }
    if reverse {
        ticks.reverse();
    }
    Ok(ticks)
}
/// Log formatting is independently observable, including suppressed minor labels.
pub fn log_tick_format(
    start: f64,
    stop: f64,
    base: f64,
    count: f64,
    specifier: Option<&str>,
    locale: NumericLocale,
) -> ChartResult<NumericFormatter> {
    let mut spec =
        NumericSpecifier::parse(specifier.unwrap_or(if base == 10. { "s" } else { "," }))?;
    if base % 1. == 0. && spec.precision.is_none() {
        spec.trim = true;
    }
    let f = NumericFormatter::new(spec, locale)?;
    if count == f64::INFINITY {
        return Ok(f);
    }
    let n = log_tick_candidates(start, stop, base, 10., 1_000_000)?.len();
    let k = base * count / n as f64;
    Ok(f.suppress(base, start < 0., if k.is_nan() { k } else { k.max(1.) }))
}
// d3-array 3.2.4 tickSpec; count is a hint, never a resource budget.
fn tick_spec(start: f64, stop: f64, mut count: f64) -> (f64, f64, f64) {
    loop {
        let step = (stop - start) / if count.is_nan() { count } else { count.max(0.) };
        let power = libm::log10(step).floor();
        let error = step / pxfm::f_pow(10., power);
        let factor = if error >= 50_f64.sqrt() {
            10.
        } else if error >= 10_f64.sqrt() {
            5.
        } else if error >= 2_f64.sqrt() {
            2.
        } else {
            1.
        };
        let (i1, i2, inc) = if power < 0. {
            let inc = pxfm::f_pow(10., -power) / factor;
            let mut i1 = js_round(start * inc);
            let mut i2 = js_round(stop * inc);
            if i1 / inc < start {
                i1 += 1.;
            }
            if i2 / inc > stop {
                i2 -= 1.;
            }
            (i1, i2, -inc)
        } else {
            let inc = pxfm::f_pow(10., power) * factor;
            let mut i1 = js_round(start / inc);
            let mut i2 = js_round(stop / inc);
            if i1 * inc < start {
                i1 += 1.;
            }
            if i2 * inc > stop {
                i2 -= 1.;
            }
            (i1, i2, inc)
        };
        if i2 < i1 && (0.5..2.).contains(&count) {
            count *= 2.;
            continue;
        }
        return (i1, i2, inc);
    }
}
pub(super) fn nice(start: f64, stop: f64, count: f64) -> (f64, f64) {
    let reversed = stop < start;
    let (mut a, mut b) = if reversed {
        (stop, start)
    } else {
        (start, stop)
    };
    let mut previous = f64::NAN;
    for _ in 0..10 {
        let step = tick_spec(a, b, count).2;
        if step == previous {
            return if reversed { (b, a) } else { (a, b) };
        }
        if step > 0. {
            a = (a / step).floor() * step;
            b = (b / step).ceil() * step;
        } else if step < 0. {
            a = (a * step).ceil() / step;
            b = (b * step).floor() / step;
        } else {
            break;
        }
        previous = step;
    }
    (start, stop)
}
pub(super) fn nice_log(start: f64, stop: f64, base: f64) -> (f64, f64) {
    let reversed = stop < start;
    let (a, b) = if reversed {
        (stop, start)
    } else {
        (start, stop)
    };
    let negative = start < 0.;
    let log = |x| log_value(base, false, x);
    let pow = |x| log_power(base, false, x);
    let lower = if negative {
        -pow(-(-log(-a)).floor())
    } else {
        pow(log(a).floor())
    };
    let upper = if negative {
        -pow(-(-log(-b)).ceil())
    } else {
        pow(log(b).ceil())
    };
    if reversed {
        (upper, lower)
    } else {
        (lower, upper)
    }
}
