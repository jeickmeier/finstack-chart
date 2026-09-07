use super::{Bounds, ContinuousDomain, LinearScale, OutsidePolicy, error};
use crate::data::TimeUnit;
use crate::{ChartResult, DiagnosticCode};

/// Exact integer Unix-timestamp endpoints, in the scale's explicit source units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimeBounds {
    /// First endpoint; descending order is allowed.
    pub start: i64,
    /// Second endpoint; constant domains expand by one source second where representable.
    pub end: i64,
}

/// UTC tick alignment; no timezone database, receipt clock, local locale or exchange calendar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UtcInterval {
    /// Aligned multiples of source ticks from the Unix epoch; primarily subsecond labels.
    Ticks(u64),
    /// Aligned whole seconds from the Unix epoch.
    Seconds(u32),
    /// UTC midnight-aligned day steps.
    Days(u32),
    /// Monday-aligned whole UTC weeks.
    Weeks(u32),
    /// Calendar month starts; multiples align from January of year zero.
    Months(u32),
    /// Calendar year starts; multiples align by the proleptic Gregorian year number.
    Years(u32),
}

/// Exact source timestamp plus a deterministic UTC calendar label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UtcTick {
    /// Original integer timestamp, never stored as an absolute f64.
    pub value: i64,
    /// UTC label, independent of the source timezone metadata or machine timezone.
    pub label: String,
}

/// Integer-origin UTC scale. A visible span/origin offset larger than 2^53 source ticks
/// rejects at this boundary; use a coarser source unit when finer resolution is unnecessary.
#[derive(Clone, Debug, PartialEq)]
pub struct UtcScale {
    domain: TimeBounds,
    view: TimeBounds,
    origin: i64,
    unit: TimeUnit,
    linear: LinearScale,
    outside: OutsidePolicy,
}
impl UtcScale {
    /// Resolve exact integer training/viewport endpoints and subtract an integer origin
    /// before any floating conversion. A constant expands by one source second on each side.
    pub fn new(
        domain: TimeBounds,
        viewport: Option<TimeBounds>,
        unit: TimeUnit,
        range: Bounds,
        outside: OutsidePolicy,
    ) -> ChartResult<Self> {
        let domain = expand_time(domain, unit)?;
        let view = viewport.unwrap_or(domain);
        if view.start == view.end {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "UTC viewport endpoints must be distinct.",
            ));
        }
        let origin = view.start;
        let end = relative(view.end, origin)?;
        let linear = LinearScale::resolve(
            None,
            ContinuousDomain::explicit(Bounds::new(0., end)?),
            range,
            None,
            outside,
        )?;
        Ok(Self {
            domain,
            view,
            origin,
            unit,
            linear,
            outside,
        })
    }
    /// Exact trained source endpoints; zoom does not change them.
    pub fn domain(&self) -> TimeBounds {
        self.domain
    }
    /// Exact visible source endpoints.
    pub fn viewport(&self) -> TimeBounds {
        self.view
    }
    /// Integer origin subtracted before floating projection.
    pub fn origin(&self) -> i64 {
        self.origin
    }
    /// Source timestamp representation.
    pub fn unit(&self) -> TimeUnit {
        self.unit
    }
    /// Destination range.
    pub fn range(&self) -> Bounds {
        self.linear.range()
    }
    /// Project one integer timestamp, keeping the original available for inspection.
    pub fn map(&self, value: i64) -> ChartResult<Option<f64>> {
        let low = self.view.start.min(self.view.end);
        let high = self.view.start.max(self.view.end);
        let value = match self.outside {
            OutsidePolicy::Omit if value < low || value > high => return Ok(None),
            OutsidePolicy::Clamp => value.clamp(low, high),
            _ => value,
        };
        self.linear.map(relative(value, self.origin)?)
    }
    /// Invert a finite destination position to nearest integer source tick (half away from zero).
    pub fn invert(&self, position: f64) -> ChartResult<i64> {
        absolute(self.linear.invert(position)?.round(), self.origin)
    }
    /// Select a calendar-aware interval for a bounded requested tick density.
    pub fn auto_interval(&self, target: usize) -> ChartResult<UtcInterval> {
        if !(2..=128).contains(&target) {
            return Err(error(
                DiagnosticCode::Validation,
                "UTC tick target must be 2–128.",
            ));
        }
        let ticks = (i128::from(self.view.end) - i128::from(self.view.start)).unsigned_abs() as f64
            / (target - 1) as f64;
        let seconds = ticks / ticks_per_second(self.unit) as f64;
        if seconds < 1. {
            let step =
                super::linear::tick_step(Bounds::new(0., ticks * (target - 1) as f64)?, target)?
                    .ceil()
                    .max(1.);
            return Ok(UtcInterval::Ticks(step as u64));
        }
        for step in [1, 5, 15, 30, 60, 300, 900, 1800, 3600, 10800, 21600, 43200] {
            if seconds <= f64::from(step) {
                return Ok(UtcInterval::Seconds(step));
            }
        }
        if seconds <= 86400. {
            return Ok(UtcInterval::Days(1));
        }
        if seconds <= 7. * 86400. {
            return Ok(UtcInterval::Weeks(1));
        }
        if seconds <= 31. * 86400. {
            return Ok(UtcInterval::Months(1));
        }
        if seconds <= 93. * 86400. {
            return Ok(UtcInterval::Months(3));
        }
        for years in [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000] {
            if seconds <= f64::from(years) * 366. * 86400. {
                return Ok(UtcInterval::Years(years));
            }
        }
        Ok(UtcInterval::Years(5000))
    }
    /// Generate bounded calendar ticks within the viewport (years 0001–9999).
    pub fn ticks(&self, interval: UtcInterval, max_ticks: usize) -> ChartResult<Vec<UtcTick>> {
        if max_ticks == 0 || max_ticks > 4096 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "UTC tick budget must be 1–4096.",
            ));
        }
        let low = self.view.start.min(self.view.end);
        let high = self.view.start.max(self.view.end);
        civil_at(low, self.unit)?;
        civil_at(high, self.unit)?;
        let per_second = ticks_per_second(self.unit);
        let fixed = match interval {
            UtcInterval::Ticks(n) => Some((i128::from(n), 0)),
            UtcInterval::Seconds(n) => Some((i128::from(n) * per_second, 0)),
            UtcInterval::Days(n) => Some((i128::from(n) * 86400 * per_second, 0)),
            UtcInterval::Weeks(n) => Some((
                i128::from(n) * 7 * 86400 * per_second,
                4 * 86400 * per_second,
            )),
            UtcInterval::Months(n) | UtcInterval::Years(n) if n == 0 => {
                return Err(error(
                    DiagnosticCode::Validation,
                    "UTC calendar interval must be positive.",
                ));
            }
            _ => None,
        };
        let mut ticks = vec![];
        if let Some((step, offset)) = fixed {
            if step == 0 {
                return Err(error(
                    DiagnosticCode::Validation,
                    "UTC tick interval must be positive.",
                ));
            }
            let low = i128::from(low);
            let high = i128::from(high);
            let mut value = (low - offset).div_euclid(step) * step + offset;
            if value < low {
                value += step;
            }
            while value <= high {
                push_tick(&mut ticks, value, self.unit, interval, max_ticks)?;
                value += step;
            }
        } else {
            let civil = civil_at(low, self.unit)?;
            let period = match interval {
                UtcInterval::Months(n) => i64::from(n),
                UtcInterval::Years(n) => i64::from(n) * 12,
                _ => return Err(error(DiagnosticCode::Validation, "Invalid UTC interval.")),
            };
            let month_index = i64::from(civil.year) * 12 + i64::from(civil.month) - 1;
            let mut next = month_index.div_euclid(period) * period;
            loop {
                let year = next.div_euclid(12);
                let month = next.rem_euclid(12) + 1;
                if year > 9999 {
                    break;
                }
                if year >= 1 {
                    let day = days_from_civil(year as i32, month as u32, 1)?;
                    let value = i128::from(day) * 86400 * per_second;
                    if value > i128::from(high) {
                        break;
                    }
                    if value >= i128::from(low) {
                        push_tick(&mut ticks, value, self.unit, interval, max_ticks)?;
                    }
                }
                next = next.checked_add(period).ok_or_else(|| {
                    error(DiagnosticCode::PrecisionLoss, "Calendar step overflow.")
                })?;
            }
        }
        if self.view.start > self.view.end {
            ticks.reverse();
        }
        Ok(ticks)
    }
}
fn expand_time(bounds: TimeBounds, unit: TimeUnit) -> ChartResult<TimeBounds> {
    if bounds.start != bounds.end {
        return Ok(bounds);
    }
    let second = i64::try_from(ticks_per_second(unit))
        .map_err(|_| error(DiagnosticCode::PrecisionLoss, "Timestamp unit overflow."))?;
    let start = bounds.start.checked_sub(second);
    let end = bounds.end.checked_add(second);
    match (start, end) {
        (Some(start), Some(end)) => Ok(TimeBounds { start, end }),
        _ => Err(error(
            DiagnosticCode::PrecisionLoss,
            "Symmetric constant-time expansion exceeds the source integer representation.",
        )),
    }
}
pub(crate) fn relative(value: i64, origin: i64) -> ChartResult<f64> {
    let delta = i128::from(value) - i128::from(origin);
    if delta.unsigned_abs() > 1_u128 << 53 {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Timestamp origin/span exceeds exact source-tick projection; choose a closer origin or coarser source unit.",
        ));
    }
    Ok(delta as f64)
}
pub(crate) fn absolute(relative: f64, origin: i64) -> ChartResult<i64> {
    if !relative.is_finite() || relative.fract() != 0. || relative.abs() > (1_u64 << 53) as f64 {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Relative time coordinate cannot preserve integer source ticks.",
        ));
    }
    i64::try_from(i128::from(origin) + relative as i128).map_err(|_| {
        error(
            DiagnosticCode::PrecisionLoss,
            "Projected timestamp exceeds the source integer representation.",
        )
    })
}
pub(crate) fn ticks_per_second(unit: TimeUnit) -> i128 {
    match unit {
        TimeUnit::Seconds => 1,
        TimeUnit::Milliseconds => 1000,
        TimeUnit::Microseconds => 1_000_000,
        TimeUnit::Nanoseconds => 1_000_000_000,
    }
}
fn push_tick(
    ticks: &mut Vec<UtcTick>,
    value: i128,
    unit: TimeUnit,
    interval: UtcInterval,
    max: usize,
) -> ChartResult<()> {
    if ticks.len() >= max {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "UTC tick count exceeds the explicit budget.",
        ));
    }
    let value = i64::try_from(value).map_err(|_| {
        error(
            DiagnosticCode::PrecisionLoss,
            "UTC tick exceeds the source integer representation.",
        )
    })?;
    ticks.push(UtcTick {
        value,
        label: format_utc(value, unit, interval)?,
    });
    Ok(())
}

/// Portable UTC calendar components; POSIX seconds do not model leap seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UtcDateTime {
    /// Proleptic Gregorian year in 1–9999.
    pub year: i32,
    /// Calendar month 1–12.
    pub month: u32,
    /// Day of month.
    pub day: u32,
    /// Hour 0–23.
    pub hour: u32,
    /// Minute 0–59.
    pub minute: u32,
    /// Second 0–59; leap seconds are not represented.
    pub second: u32,
}
impl UtcDateTime {
    /// Checked whole Unix seconds, useful for explicit calendar fixtures/domains.
    pub fn unix_seconds(self) -> ChartResult<i64> {
        if self.hour >= 24 || self.minute >= 60 || self.second >= 60 {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Invalid UTC time-of-day.",
            ));
        }
        Ok(days_from_civil(self.year, self.month, self.day)? * 86400
            + i64::from(self.hour) * 3600
            + i64::from(self.minute) * 60
            + i64::from(self.second))
    }
    /// Decode whole Unix seconds without reading machine timezone or locale.
    pub fn from_unix_seconds(seconds: i64) -> ChartResult<Self> {
        civil_at(seconds, TimeUnit::Seconds)
    }
}
fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
fn month_days(year: i32, month: u32) -> u32 {
    match month {
        2 => {
            if leap(year) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
fn days_before_year(year: i32) -> i64 {
    let previous = i64::from(year) - 1;
    365 * previous + previous.div_euclid(4) - previous.div_euclid(100) + previous.div_euclid(400)
}
fn days_from_civil(year: i32, month: u32, day: u32) -> ChartResult<i64> {
    if !(1..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day == 0
        || day > month_days(year, month)
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "UTC calendar date must be valid in years 0001–9999.",
        ));
    }
    let mut days = days_before_year(year) - days_before_year(1970);
    for m in 1..month {
        days += i64::from(month_days(year, m));
    }
    Ok(days + i64::from(day) - 1)
}
fn civil_at(value: i64, unit: TimeUnit) -> ChartResult<UtcDateTime> {
    let seconds = i128::from(value).div_euclid(ticks_per_second(unit));
    let day = seconds.div_euclid(86400);
    let within = seconds.rem_euclid(86400);
    if day < i128::from(days_from_civil(1, 1, 1)?)
        || day > i128::from(days_from_civil(9999, 12, 31)?)
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "UTC calendar formatting supports years 0001–9999.",
        ));
    }
    // Gregorian years repeat every 146097 days. At most 399 year and 11 month steps.
    let from_2000 = day - 10957;
    let era = from_2000.div_euclid(146097);
    let mut year = 2000 + 400 * era as i32;
    let mut remaining = from_2000.rem_euclid(146097) as i64;
    loop {
        let days = if leap(year) { 366 } else { 365 };
        if remaining < days {
            break;
        }
        remaining -= days;
        year += 1;
    }
    let mut month = 1;
    while remaining >= i64::from(month_days(year, month)) {
        remaining -= i64::from(month_days(year, month));
        month += 1;
    }
    Ok(UtcDateTime {
        year,
        month,
        day: remaining as u32 + 1,
        hour: (within / 3600) as u32,
        minute: ((within % 3600) / 60) as u32,
        second: (within % 60) as u32,
    })
}
/// Deterministic UTC labels, with calendar precision matching the declared interval.
pub fn format_utc(value: i64, unit: TimeUnit, interval: UtcInterval) -> ChartResult<String> {
    let c = civil_at(value, unit)?;
    Ok(match interval {
        UtcInterval::Years(_) => format!("{:04}", c.year),
        UtcInterval::Months(_) => format!("{:04}-{:02}", c.year, c.month),
        UtcInterval::Days(_) | UtcInterval::Weeks(_) => {
            format!("{:04}-{:02}-{:02}", c.year, c.month, c.day)
        }
        UtcInterval::Seconds(_) => format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}Z",
            c.year, c.month, c.day, c.hour, c.minute, c.second
        ),
        UtcInterval::Ticks(_) => {
            let digits = match unit {
                TimeUnit::Seconds => 0,
                TimeUnit::Milliseconds => 3,
                TimeUnit::Microseconds => 6,
                TimeUnit::Nanoseconds => 9,
            };
            let fraction = i128::from(value).rem_euclid(ticks_per_second(unit));
            if digits == 0 {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}Z",
                    c.year, c.month, c.day, c.hour, c.minute, c.second
                )
            } else {
                format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{fraction:0digits$}Z",
                    c.year, c.month, c.day, c.hour, c.minute, c.second
                )
            }
        }
    })
}
