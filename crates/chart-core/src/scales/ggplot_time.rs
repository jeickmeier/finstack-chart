//! Fixed elapsed-time break selection; calendar-width selection has separate semantics.
use super::{TimeBounds, error};
use crate::{ChartResult, DiagnosticCode, data::TimeUnit};

/// Parse the reference unit grammar without interpreting calendar progression.
/// Trailing space fields are discarded as in R strsplit; repeated interior spaces
/// remain invalid. Some elapsed selectors ignore fields after the unit.
pub(crate) fn parse_width(text: &str) -> ChartResult<(super::CalendarUnit, f64, bool)> {
    use super::{CalendarUnit, WeekStart};
    let mut parts = text.trim_end_matches(' ').split(' ');
    let first = parts.next().unwrap_or_default();
    let second = parts.next();
    let (count, unit) = if let Some(unit) = second {
        (
            first
                .trim_matches(['\t', '\r', '\n'])
                .parse::<f64>()
                .unwrap_or(f64::NAN),
            unit,
        )
    } else {
        (1., first)
    };
    if !count.is_finite() || count <= 0. {
        return Err(error(
            DiagnosticCode::Validation,
            "Reference time width requires a finite positive multiplier.",
        ));
    }
    let unit = match unit.strip_suffix('s').unwrap_or(unit) {
        "sec" => CalendarUnit::Second,
        "min" => CalendarUnit::Minute,
        "hour" => CalendarUnit::Hour,
        "day" => CalendarUnit::Day,
        "week" => CalendarUnit::Week(WeekStart::Monday),
        "month" => CalendarUnit::Month,
        "year" => CalendarUnit::Year,
        _ => {
            return Err(error(
                DiagnosticCode::Validation,
                "Reference time width unit must be sec, min, hour, day, week, month or year, optionally plural.",
            ));
        }
    };
    Ok((unit, count, parts.next().is_some()))
}
pub(crate) fn width_seconds(unit: super::CalendarUnit, count: f64) -> ChartResult<f64> {
    use super::CalendarUnit;
    let seconds = count
        * match unit {
            CalendarUnit::Second => 1.,
            CalendarUnit::Minute => 60.,
            CalendarUnit::Hour => 3600.,
            CalendarUnit::Day => 86400.,
            CalendarUnit::Week(_) => 604800.,
            CalendarUnit::Month => 2678400.,
            CalendarUnit::Year => 31536000.,
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Unsupported elapsed reference width unit.",
                ));
            }
        };
    if !seconds.is_finite() || seconds <= 0. {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Elapsed reference width must be finite and positive.",
        ));
    }
    Ok(seconds)
}

/// ggplot2 fixed-second break lattice, cropped to the visible timestamp interval.
/// Widths are measured in elapsed seconds, independent of timezone transitions.
/// Source timestamps and lattice arithmetic remain integer-exact; widths finer than
/// the declared source quantum reject instead of silently rounding tick identity.
pub fn ggplot_breaks_seconds(
    bounds: TimeBounds,
    unit: TimeUnit,
    seconds: f64,
    budget: usize,
) -> ChartResult<Vec<i64>> {
    integer_lattice(bounds, seconds_step(seconds, unit)?, 0, budget)
}
fn seconds_step(seconds: f64, unit: TimeUnit) -> ChartResult<i128> {
    let factor = match unit {
        TimeUnit::Seconds => 1.,
        TimeUnit::Milliseconds => 1_000.,
        TimeUnit::Microseconds => 1_000_000.,
        TimeUnit::Nanoseconds => 1_000_000_000.,
    };
    let step = seconds * factor;
    if !seconds.is_finite() || seconds <= 0. {
        return Err(error(
            DiagnosticCode::Validation,
            "Time break width must be finite and positive.",
        ));
    }
    if !(1. ..=9_007_199_254_740_992.).contains(&step) || step.fract() != 0. {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Time break width must be an exact source-unit integer no larger than 2^53.",
        ));
    }
    Ok(step as i128)
}
/// Fractional character widths retain R's rounded alignment and integer sequence step.
/// Both widths are exact source quanta, so source timestamps never pass through f64.
pub(crate) fn aligned_seconds(
    bounds: TimeBounds,
    unit: TimeUnit,
    alignment_seconds: f64,
    step_seconds: f64,
    budget: usize,
) -> ChartResult<Vec<i64>> {
    let alignment = seconds_step(alignment_seconds, unit)?;
    let step = seconds_step(step_seconds, unit)?;
    let start = i128::from(bounds.start.min(bounds.end));
    integer_lattice(bounds, step, rounded_multiple(start, alignment), budget)
}

/// Date scales floor numeric day candidates after fractional-width alignment.
/// Flooring the anchor first gives the same integer-day progression without losing
/// an upper-edge candidate by cropping fractional dates too early.
pub(crate) fn aligned_dates(
    bounds: TimeBounds,
    unit: TimeUnit,
    count: f64,
    budget: usize,
) -> ChartResult<Vec<i64>> {
    let day = seconds_step(86400., unit)?;
    let start = bounds.start.min(bounds.end) as f64 / day as f64;
    let anchor = ((start / count).round_ties_even() * count).floor();
    if !anchor.is_finite()
        || anchor.abs() > 9_007_199_254_740_992.
        || count.floor() < 1.
        || count.floor() > i32::MAX as f64
    {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Fractional Date width exceeds exact day bounds.",
        ));
    }
    integer_lattice(
        bounds,
        day * count.floor() as i128,
        anchor as i128 * day,
        budget,
    )
}

/// Fixed duration width in numeric seconds, using the reference's floor/ceiling
/// lattice. Returned candidates include enclosure endpoints; the guide crops them.
pub fn ggplot_breaks_duration_width(
    bounds: super::Bounds,
    seconds: f64,
    budget: usize,
) -> ChartResult<Vec<f64>> {
    if !seconds.is_finite() || seconds <= 0. {
        return Err(error(
            DiagnosticCode::Validation,
            "Duration break width must be finite and positive.",
        ));
    }
    let start = (bounds.minimum() / seconds).floor() * seconds;
    let end = (bounds.maximum() / seconds).ceil() * seconds;
    let intervals = ((end - start) / seconds + 1e-10).floor();
    if !start.is_finite()
        || !end.is_finite()
        || !intervals.is_finite()
        || intervals < 0.
        || intervals >= budget as f64
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Duration break lattice exceeds its tick budget.",
        ));
    }
    Ok((0..=intervals as usize)
        .map(|i| (start + i as f64 * seconds).min(end))
        .collect())
}

/// ggplot2 width progression from the lower endpoint's local calendar boundary.
/// Unlike `CalendarTicks::Interval`, multipliers do not filter calendar fields.
/// Hours, days and weeks advance by elapsed time; months and years advance in
/// the supplied local calendar. Seconds/minutes use an epoch-aligned lattice.
pub fn ggplot_breaks_width(
    bounds: TimeBounds,
    unit: TimeUnit,
    calendar: &super::Calendar,
    width: super::CalendarInterval,
    budget: usize,
) -> ChartResult<Vec<i64>> {
    breaks_width_from(
        bounds,
        bounds.start.min(bounds.end),
        unit,
        calendar,
        width,
        budget,
    )
}

pub(super) fn breaks_width_from(
    bounds: TimeBounds,
    lower_enclosure: i64,
    unit: TimeUnit,
    calendar: &super::Calendar,
    width: super::CalendarInterval,
    budget: usize,
) -> ChartResult<Vec<i64>> {
    use super::{CalendarInterval, CalendarUnit, WeekStart};
    validate_width(width)?;
    let elapsed = match width.unit {
        CalendarUnit::Second => {
            return ggplot_breaks_seconds(bounds, unit, f64::from(width.step), budget);
        }
        CalendarUnit::Minute => {
            return ggplot_breaks_seconds(bounds, unit, 60. * f64::from(width.step), budget);
        }
        CalendarUnit::Hour => Some(3600_i128),
        CalendarUnit::Day => Some(86400),
        CalendarUnit::Week(WeekStart::Monday) => Some(7 * 86400),
        CalendarUnit::Month | CalendarUnit::Year => None,
        _ => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Reference time widths support second, minute, hour, day, Monday week, month and year units.",
            ));
        }
    };
    let low = bounds.start.min(bounds.end);
    let high = bounds.start.max(bounds.end);
    let base = CalendarInterval::new(width.unit);
    let anchor = calendar.floor(lower_enclosure, unit, base)?;
    if let Some(seconds) = elapsed {
        let factor = super::utc::ticks_per_second(unit);
        let step = seconds * i128::from(width.step) * factor;
        return integer_lattice(bounds, step, i128::from(anchor), budget);
    }
    let start = calendar.components(anchor, unit)?;
    let end = calendar.components(high, unit)?;
    let span = match width.unit {
        CalendarUnit::Month => {
            (i64::from(end.year) - i64::from(start.year)) * 12 + i64::from(end.month)
                - i64::from(start.month)
        }
        CalendarUnit::Year => i64::from(end.year) - i64::from(start.year),
        _ => unreachable!("elapsed units returned above"),
    };
    let first = i64::from(anchor < low);
    let last = span / i64::from(width.step);
    let count = (last - first + 1).max(0) as usize;
    crate::limits::require_within(count <= budget, "reference time width")?;
    let mut values = Vec::with_capacity(count);
    for index in first..=last {
        // Offset each candidate from the original anchor so a DST change cannot
        // accumulate wall-clock drift. The shared calendar owns all arithmetic.
        let value = calendar.offset(anchor, unit, base, index as f64 * f64::from(width.step))?;
        if value >= low && value <= high {
            values.push(value);
        }
    }
    Ok(values)
}

pub(super) fn validate_width(width: super::CalendarInterval) -> ChartResult<()> {
    use super::{CalendarUnit, WeekStart};
    width.validate()?;
    if !matches!(
        width.unit,
        CalendarUnit::Second
            | CalendarUnit::Minute
            | CalendarUnit::Hour
            | CalendarUnit::Day
            | CalendarUnit::Week(WeekStart::Monday)
            | CalendarUnit::Month
            | CalendarUnit::Year
    ) {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Reference time widths support second, minute, hour, day, Monday week, month and year units.",
        ));
    }
    Ok(())
}

fn integer_lattice(
    bounds: TimeBounds,
    step: i128,
    anchor: i128,
    budget: usize,
) -> ChartResult<Vec<i64>> {
    if step <= 0 {
        return Err(error(
            DiagnosticCode::Validation,
            "Time break width must be positive.",
        ));
    }
    let low = i128::from(bounds.start.min(bounds.end));
    let high = i128::from(bounds.start.max(bounds.end));
    let first = low + (anchor - low).rem_euclid(step);
    let count = if first > high {
        0
    } else {
        (high - first) / step + 1
    };
    crate::limits::require_within(count <= budget as i128, "reference time break")?;
    Ok((0..count).map(|i| (first + i * step) as i64).collect())
}

pub(super) fn date_day_width(
    bounds: TimeBounds,
    unit: TimeUnit,
    step: u32,
    budget: usize,
) -> ChartResult<Vec<i64>> {
    integer_lattice(
        bounds,
        i128::from(step) * 86400 * super::utc::ticks_per_second(unit),
        0,
        budget,
    )
}

fn rounded_multiple(value: i128, alignment: i128) -> i128 {
    let mut multiple = value.div_euclid(alignment);
    let remainder = value.rem_euclid(alignment);
    if remainder * 2 > alignment || remainder * 2 == alignment && multiple.rem_euclid(2) == 1 {
        multiple += 1;
    }
    multiple * alignment
}

/// Uncropped fullseq boundaries for reference aesthetic guides. Calendar progression
/// and integer lattices remain shared with positional width selection.
pub(super) fn aesthetic_width(
    bounds: TimeBounds,
    unit: TimeUnit,
    calendar: &super::Calendar,
    date: bool,
    text: &str,
    budget: usize,
) -> ChartResult<Vec<i64>> {
    use super::{CalendarInterval, CalendarUnit};
    let (kind, count, extra) = parse_width(text)?;
    if date
        && matches!(
            kind,
            CalendarUnit::Second | CalendarUnit::Minute | CalendarUnit::Hour
        )
        || extra && kind != CalendarUnit::Second
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Invalid reference temporal width.",
        ));
    }
    let low = i128::from(bounds.start.min(bounds.end));
    let high = i128::from(bounds.start.max(bounds.end));
    let alignment = seconds_step(width_seconds(kind, count)?, unit)?;
    let upper = high.checked_add(alignment).ok_or_else(|| {
        error(
            DiagnosticCode::PrecisionLoss,
            "Temporal width exceeds its source range.",
        )
    })?;
    let exact = |v| {
        i64::try_from(v).map_err(|_| {
            error(
                DiagnosticCode::PrecisionLoss,
                "Temporal width exceeds its source range.",
            )
        })
    };
    if kind == CalendarUnit::Second
        || kind == CalendarUnit::Minute
        || date && kind == CalendarUnit::Day
    {
        let first = rounded_multiple(low, alignment);
        let last = rounded_multiple(upper, alignment);
        let step = seconds_step(
            width_seconds(
                kind,
                if kind == CalendarUnit::Second {
                    count
                } else {
                    count.floor()
                },
            )?,
            unit,
        )?;
        return integer_lattice(
            TimeBounds {
                start: exact(first)?,
                end: exact(last)?,
            },
            step,
            first,
            budget,
        );
    }
    if count.floor() < 1. || count.floor() > f64::from(u32::MAX) {
        return Err(error(
            DiagnosticCode::Validation,
            "Calendar width requires a positive bounded sequence step.",
        ));
    }
    let base = CalendarInterval::new(kind);
    let first = calendar.floor(exact(low)?, unit, base)?;
    let last = calendar.floor(exact(upper)?, unit, base)?;
    breaks_width_from(
        TimeBounds {
            start: first,
            end: last,
        },
        first,
        unit,
        calendar,
        CalendarInterval {
            unit: kind,
            step: count.floor() as u32,
        },
        budget,
    )
}
