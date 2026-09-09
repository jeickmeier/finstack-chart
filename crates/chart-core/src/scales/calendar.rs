//! Calendar arithmetic uses supplied timezone transitions and shared Gregorian components.
//! Interval semantics follow d3-time 3.1.0 (ISC notice in `LICENSE-d3-time`).
use super::{TimeBounds, civil, error};
use crate::{ChartResult, DiagnosticCode, Revision, data::TimeUnit};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub(super) const SECOND: i128 = 1_000_000_000;
pub(super) const MINUTE: i128 = 60 * SECOND;
pub(super) const HOUR: i128 = 60 * MINUTE;
pub(super) const DAY: i128 = 24 * HOUR;
const DATE_LIMIT: i128 = 100_000_000 * DAY;
const WORK_LIMIT: usize = 1_000_000;

/// UTC transition instant and the offset that applies at and after it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeZoneTransition {
    /// Exact Unix milliseconds, serialized without floating conversion.
    #[serde(with = "crate::portable::signed")]
    pub at_millis: i64,
    /// Seconds east of UTC.
    pub offset_seconds: i32,
}
/// Versioned, bounded timezone resource. Core never resolves a machine timezone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeZoneRules {
    /// Resource schema version, currently one.
    pub version: u32,
    /// Stable zone identifier, such as `America/New_York`.
    pub zone: String,
    /// Owner-supplied resource revision.
    pub revision: Revision,
    /// Identity of the source timezone database.
    pub tzdata: String,
    /// Inclusive covered UTC endpoints, in Unix milliseconds.
    pub coverage: TimeBounds,
    /// Offset at the first covered instant, in seconds east of UTC.
    pub initial_offset_seconds: i32,
    /// Strictly increasing transitions inside coverage, after its first endpoint.
    pub transitions: Vec<TimeZoneTransition>,
}
impl TimeZoneRules {
    /// Validate identity, coverage, resource size and chronological transition order.
    pub fn validate(&self) -> ChartResult<()> {
        if self.version != 1
            || [&self.zone, &self.tzdata]
                .into_iter()
                .any(|s| s.is_empty() || s.len() > 128 || s.chars().any(char::is_control))
            || self.coverage.start >= self.coverage.end
            || self.transitions.len() > 100_000
            || self.initial_offset_seconds.unsigned_abs() > 86400
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Invalid supplied timezone identity, coverage or resource budget.",
            ));
        }
        date_ns(i128::from(self.coverage.start) * 1_000_000)?;
        date_ns(i128::from(self.coverage.end) * 1_000_000)?;
        let mut previous = self.coverage.start;
        for transition in &self.transitions {
            if transition.at_millis <= previous
                || transition.at_millis > self.coverage.end
                || transition.offset_seconds.unsigned_abs() > 86400
            {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Timezone transitions must be ordered, covered, and within one day of UTC.",
                ));
            }
            previous = transition.at_millis;
        }
        Ok(())
    }
    fn offset(&self, ns: i128) -> ChartResult<i32> {
        self.covered(ns)?;
        let index = self
            .transitions
            .partition_point(|t| i128::from(t.at_millis) * 1_000_000 <= ns);
        Ok(index
            .checked_sub(1)
            .map_or(self.initial_offset_seconds, |i| {
                self.transitions[i].offset_seconds
            }))
    }
    fn covered(&self, ns: i128) -> ChartResult<()> {
        if ns < i128::from(self.coverage.start) * 1_000_000
            || ns > i128::from(self.coverage.end) * 1_000_000
        {
            return Err(error(
                DiagnosticCode::MissingResource,
                format!(
                    "Timezone {} revision {} does not cover this UTC instant.",
                    self.zone,
                    self.revision.get()
                ),
            ));
        }
        Ok(())
    }
    fn resolve_wall(&self, wall: i128, offsets: &[i32]) -> ChartResult<i128> {
        // Adjacent wall mappings can overlap (fold) or leave a gap. ECMAScript chooses
        // the earlier fold instant and shifts a missing wall time forward by the gap.
        let low = i128::from(self.coverage.start) * 1_000_000;
        let high = i128::from(self.coverage.end) * 1_000_000;
        let mut first = None;
        for offset in offsets {
            let candidate = wall - i128::from(*offset) * SECOND;
            if candidate >= low && candidate <= high && self.offset(candidate)? == *offset {
                first = Some(first.map_or(candidate, |v: i128| v.min(candidate)));
            }
        }
        let mut gap = None;
        if first.is_none() {
            let begin = self
                .transitions
                .partition_point(|t| i128::from(t.at_millis) * 1_000_000 < wall - DAY);
            for (i, next) in self.transitions.iter().enumerate().skip(begin) {
                let at = i128::from(next.at_millis) * 1_000_000;
                if at > wall + DAY {
                    break;
                }
                let previous = i.checked_sub(1).map_or(self.initial_offset_seconds, |i| {
                    self.transitions[i].offset_seconds
                });
                if next.offset_seconds > previous
                    && wall >= at + i128::from(previous) * SECOND
                    && wall < at + i128::from(next.offset_seconds) * SECOND
                {
                    gap = Some(wall - i128::from(previous) * SECOND);
                    break;
                }
            }
        }
        let instant = first.or(gap).ok_or_else(||error(DiagnosticCode::MissingResource,"Supplied timezone rules cannot resolve this local wall time inside their coverage."))?;
        self.covered(instant)?;
        Ok(instant)
    }
}
/// Explicit UTC or a supplied immutable local calendar resource.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CalendarZone {
    /// UTC needs no external rules.
    #[default]
    Utc,
    /// Local floor, offset and formatting all consume this same resource.
    Local(Arc<TimeZoneRules>),
}
/// Week alignment, retaining the existing explicit Monday option.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeekStart {
    /// Reference automatic week boundary.
    Sunday,
    /// Explicit ISO-style week boundary.
    Monday,
}
impl WeekStart {
    fn index(self) -> u32 {
        match self {
            Self::Sunday => 0,
            Self::Monday => 1,
        }
    }
}
/// Calendar field to which `every` filtering applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CalendarUnit {
    /// Source-unit steps from epoch, including microseconds and nanoseconds.
    SourceTick,
    /// Unix millisecond multiples.
    Millisecond,
    /// Second-of-minute field.
    Second,
    /// Minute-of-hour field, using elapsed-minute offsets.
    Minute,
    /// Local hour-of-day field, using elapsed-hour offsets through folds/gaps.
    Hour,
    /// Day of month minus one; filtering restarts each month.
    Day,
    /// Whole UTC days from epoch; used by UTC automatic two-day ticks.
    UnixDay,
    /// Weeks from a Sunday or Monday boundary.
    Week(WeekStart),
    /// Month index from January, restarting each year.
    Month,
    /// Proleptic Gregorian year multiples.
    Year,
}
/// Explicit interval and positive every(k) filter. Offset is distinct from floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarInterval {
    /// Calendar field.
    pub unit: CalendarUnit,
    /// Positive every(k) step.
    pub step: u32,
}
/// Numeric density hint or an explicit filtered calendar interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CalendarTicks {
    /// D3 count hint; NaN/nonpositive selects no interval.
    Count(crate::interpolate::Number),
    /// Explicit calendar boundaries.
    Interval(CalendarInterval),
}
impl Default for CalendarTicks {
    fn default() -> Self {
        Self::Count(crate::interpolate::Number(10.))
    }
}
impl CalendarInterval {
    /// Unfiltered base interval.
    pub const fn new(unit: CalendarUnit) -> Self {
        Self { unit, step: 1 }
    }
    /// Reference numeric every(k): floor the hint, return None for nonpositive/nonfinite.
    pub fn every(unit: CalendarUnit, step: f64) -> ChartResult<Option<Self>> {
        let step = step.floor();
        if !step.is_finite() || step <= 0. {
            return Ok(None);
        }
        if step > u32::MAX as f64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Calendar interval multiplier exceeds its resource bound.",
            ));
        }
        Ok(Some(Self {
            unit,
            step: step as u32,
        }))
    }
    /// Reject zero descriptors before any calendar population is inspected.
    pub fn validate(self) -> ChartResult<()> {
        if self.step == 0 {
            return Err(error(
                DiagnosticCode::Validation,
                "Calendar interval step must be positive.",
            ));
        }
        Ok(())
    }
}
/// Local calendar components with exact subsecond data and their UTC offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CalendarDateTime {
    /// Proleptic Gregorian year, including zero and negative years.
    pub year: i32,
    /// Month 1 through 12.
    pub month: u32,
    /// Day 1 through 31, validated by calendar month.
    pub day: u32,
    /// Hour 0 through 23.
    pub hour: u32,
    /// Minute 0 through 59.
    pub minute: u32,
    /// Second 0 through 59; POSIX timestamps do not include leap seconds.
    pub second: u32,
    /// Nanoseconds within the second, preserving the source resolution.
    pub nanosecond: u32,
    /// Seconds east of UTC at the supplied instant.
    pub offset_seconds: i32,
}
/// Prepared calendar shared by intervals, scales, labels and host operation adapters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Calendar {
    zone: CalendarZone,
    offsets: Vec<i32>,
}
impl Calendar {
    /// Validate supplied resources once; no system timezone or locale is read.
    pub fn new(zone: CalendarZone) -> ChartResult<Self> {
        let mut offsets = Vec::new();
        if let CalendarZone::Local(rules) = &zone {
            rules.validate()?;
            offsets.push(rules.initial_offset_seconds);
            offsets.extend(rules.transitions.iter().map(|t| t.offset_seconds));
            offsets.sort_unstable();
            offsets.dedup();
        }
        Ok(Self { zone, offsets })
    }
    /// Exact immutable zone descriptor and revision.
    pub fn zone(&self) -> &CalendarZone {
        &self.zone
    }
    /// Select the nearest reference interval; count is independent from the output budget.
    pub fn auto_interval(
        &self,
        domain: TimeBounds,
        unit: TimeUnit,
        count: f64,
    ) -> ChartResult<Option<CalendarInterval>> {
        let delta = (i128::from(domain.end) - i128::from(domain.start)).unsigned_abs() as f64;
        let source_millis = quantum(unit) as f64 / 1_000_000.;
        let target = delta * source_millis / count;
        if target.is_nan() || count <= 0. {
            return Ok(None);
        }
        if target < source_millis || target < 1. && quantum(unit) < 1_000_000 {
            let step = super::ticks::tick_step(0., delta, count);
            return CalendarInterval::every(
                CalendarUnit::SourceTick,
                if step.is_nan() { step } else { step.max(1.) },
            );
        }
        let day = if matches!(self.zone, CalendarZone::Utc) {
            CalendarUnit::UnixDay
        } else {
            CalendarUnit::Day
        };
        let choices = [
            (CalendarUnit::Second, 1, 1000.),
            (CalendarUnit::Second, 5, 5000.),
            (CalendarUnit::Second, 15, 15000.),
            (CalendarUnit::Second, 30, 30000.),
            (CalendarUnit::Minute, 1, 60000.),
            (CalendarUnit::Minute, 5, 300000.),
            (CalendarUnit::Minute, 15, 900000.),
            (CalendarUnit::Minute, 30, 1800000.),
            (CalendarUnit::Hour, 1, 3600000.),
            (CalendarUnit::Hour, 3, 10800000.),
            (CalendarUnit::Hour, 6, 21600000.),
            (CalendarUnit::Hour, 12, 43200000.),
            (day, 1, 86400000.),
            (day, 2, 172800000.),
            (CalendarUnit::Week(WeekStart::Sunday), 1, 604800000.),
            (CalendarUnit::Month, 1, 2592000000.),
            (CalendarUnit::Month, 3, 7776000000.),
            (CalendarUnit::Year, 1, 31536000000.),
        ];
        let i = choices.partition_point(|(_, _, duration)| *duration <= target);
        let millis = |v| {
            let ns = source_ns(v, unit);
            (ns / 1_000_000) as f64 + (ns % 1_000_000) as f64 / 1_000_000.
        };
        if i == choices.len() {
            return CalendarInterval::every(
                CalendarUnit::Year,
                super::ticks::tick_step(
                    millis(domain.start) / 31536000000.,
                    millis(domain.end) / 31536000000.,
                    count,
                ),
            );
        }
        if i == 0 {
            let step = super::ticks::tick_step(millis(domain.start), millis(domain.end), count);
            return CalendarInterval::every(
                CalendarUnit::Millisecond,
                if step.is_nan() { step } else { step.max(1.) },
            );
        }
        let (field, step, _) = choices[if target / choices[i - 1].2 < choices[i].2 / target {
            i - 1
        } else {
            i
        }];
        Ok(Some(CalendarInterval { unit: field, step }))
    }
    fn selected_interval(
        &self,
        domain: TimeBounds,
        unit: TimeUnit,
        selection: CalendarTicks,
    ) -> ChartResult<Option<CalendarInterval>> {
        match selection {
            CalendarTicks::Count(n) => self.auto_interval(domain, unit, n.0),
            CalendarTicks::Interval(i) => {
                i.validate()?;
                Ok(Some(i))
            }
        }
    }
    /// Raw inclusive endpoint ticks in authored orientation; no text or collision thinning.
    pub fn ticks(
        &self,
        domain: TimeBounds,
        unit: TimeUnit,
        selection: CalendarTicks,
        budget: usize,
    ) -> ChartResult<Vec<i64>> {
        let low = domain.start.min(domain.end);
        let high = domain.start.max(domain.end);
        self.offset_ns(source_ns(low, unit))?;
        self.offset_ns(source_ns(high, unit))?;
        let Some(interval) = self.selected_interval(
            TimeBounds {
                start: low,
                end: high,
            },
            unit,
            selection,
        )?
        else {
            return Ok(vec![]);
        };
        let mut ticks = self.range_ns(
            source_ns(low, unit),
            source_ns(high, unit) + quantum(unit),
            unit,
            interval,
            budget,
        )?;
        if domain.start > domain.end {
            ticks.reverse();
        }
        Ok(ticks)
    }
    /// Nice only the two outer endpoints, retaining descending orientation.
    pub fn nice(
        &self,
        domain: TimeBounds,
        unit: TimeUnit,
        selection: CalendarTicks,
    ) -> ChartResult<TimeBounds> {
        let low = domain.start.min(domain.end);
        let high = domain.start.max(domain.end);
        self.offset_ns(source_ns(low, unit))?;
        self.offset_ns(source_ns(high, unit))?;
        let Some(interval) = self.selected_interval(domain, unit, selection)? else {
            return Ok(domain);
        };
        let a = self.floor(low, unit, interval)?;
        let b = self.ceil(high, unit, interval)?;
        Ok(if domain.start > domain.end {
            TimeBounds { start: b, end: a }
        } else {
            TimeBounds { start: a, end: b }
        })
    }
    pub(super) fn offset_ns(&self, ns: i128) -> ChartResult<i32> {
        date_ns(ns)?;
        match &self.zone {
            CalendarZone::Utc => Ok(0),
            CalendarZone::Local(rules) => rules.offset(ns),
        }
    }
    fn wall(&self, ns: i128) -> ChartResult<i128> {
        Ok(ns + i128::from(self.offset_ns(ns)?) * SECOND)
    }
    fn resolve_wall(&self, wall: i128) -> ChartResult<i128> {
        let ns = match &self.zone {
            CalendarZone::Utc => wall,
            CalendarZone::Local(rules) => rules.resolve_wall(wall, &self.offsets)?,
        };
        date_ns(ns)?;
        Ok(ns)
    }
    /// Decode a source timestamp without absolute floating conversion.
    pub fn components(&self, value: i64, unit: TimeUnit) -> ChartResult<CalendarDateTime> {
        self.components_ns(source_ns(value, unit))
    }
    pub(super) fn components_ns(&self, ns: i128) -> ChartResult<CalendarDateTime> {
        let offset_seconds = self.offset_ns(ns)?;
        let wall = ns + i128::from(offset_seconds) * SECOND;
        let (year, month, day) = civil::civil_from_days((wall.div_euclid(DAY)) as i64)?;
        let within = wall.rem_euclid(DAY);
        Ok(CalendarDateTime {
            year,
            month,
            day,
            hour: (within / HOUR) as u32,
            minute: (within % HOUR / MINUTE) as u32,
            second: (within % MINUTE / SECOND) as u32,
            nanosecond: (within % SECOND) as u32,
            offset_seconds,
        })
    }
    /// Resolve an explicit local wall date. Offset metadata is output-only; supplied rules decide folds/gaps.
    pub fn from_components(&self, date: CalendarDateTime, unit: TimeUnit) -> ChartResult<i64> {
        if date.hour >= 24
            || date.minute >= 60
            || date.second >= 60
            || date.nanosecond >= 1_000_000_000
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Invalid local time-of-day.",
            ));
        }
        let day = civil::days_from_civil(date.year, date.month, date.day)?;
        let wall = i128::from(day) * DAY
            + i128::from(date.hour) * HOUR
            + i128::from(date.minute) * MINUTE
            + i128::from(date.second) * SECOND
            + i128::from(date.nanosecond);
        source_value(self.resolve_wall(wall)?, unit)
    }
    /// Calendar floor, including reference every(k) field filtering.
    pub fn floor(
        &self,
        value: i64,
        unit: TimeUnit,
        interval: CalendarInterval,
    ) -> ChartResult<i64> {
        source_value(self.floor_ns(source_ns(value, unit), unit, interval)?, unit)
    }
    /// Calendar ceil; an already aligned timestamp remains unchanged.
    pub fn ceil(&self, value: i64, unit: TimeUnit, interval: CalendarInterval) -> ChartResult<i64> {
        source_value(self.ceil_ns(source_ns(value, unit), unit, interval)?, unit)
    }
    /// Nearest interval boundary, selecting the upper boundary on a tie.
    pub fn round(
        &self,
        value: i64,
        unit: TimeUnit,
        interval: CalendarInterval,
    ) -> ChartResult<i64> {
        let ns = source_ns(value, unit);
        let a = self.floor_ns(ns, unit, interval)?;
        let b = self.ceil_ns(ns, unit, interval)?;
        source_value(if ns - a < b - ns { a } else { b }, unit)
    }
    /// Offset without first flooring; whole calendar days preserve wall-clock time across DST.
    pub fn offset(
        &self,
        value: i64,
        unit: TimeUnit,
        interval: CalendarInterval,
        steps: f64,
    ) -> ChartResult<i64> {
        if !steps.is_finite() || steps.floor().abs() > WORK_LIMIT as f64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Calendar offset step count exceeds its finite resource budget.",
            ));
        }
        source_value(
            self.shift_ns(source_ns(value, unit), unit, interval, steps.floor() as i64)?,
            unit,
        )
    }
    /// Inclusive-start, exclusive-stop interval range, with a separate output budget.
    pub fn range(
        &self,
        start: i64,
        stop: i64,
        unit: TimeUnit,
        interval: CalendarInterval,
        budget: usize,
    ) -> ChartResult<Vec<i64>> {
        self.range_ns(
            source_ns(start, unit),
            source_ns(stop, unit),
            unit,
            interval,
            budget,
        )
    }
    pub(super) fn range_ns(
        &self,
        start: i128,
        stop: i128,
        unit: TimeUnit,
        interval: CalendarInterval,
        budget: usize,
    ) -> ChartResult<Vec<i64>> {
        interval.validate()?;
        self.offset_ns(start)?;
        // Scale ticks may request one source quantum beyond their covered inclusive endpoint.
        if stop > start {
            self.offset_ns(stop - quantum(unit))?;
        }
        let mut output = Vec::new();
        if start >= stop {
            return Ok(output);
        }
        let mut current = self.ceil_ns(start, unit, interval)?;
        while current < stop {
            if output.len() >= budget.min(WORK_LIMIT) {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Calendar ticks exceed their independent output budget.",
                ));
            }
            output.push(source_value(current, unit)?);
            // No following resource lookup is necessary when the final possible source instant was emitted.
            if current + quantum(unit) >= stop {
                break;
            }
            let next = self.floor_ns(self.shift_ns(current, unit, interval, 1)?, unit, interval)?;
            if next <= current {
                break;
            }
            current = next;
        }
        Ok(output)
    }
    pub(super) fn floor_ns(
        &self,
        mut ns: i128,
        unit: TimeUnit,
        interval: CalendarInterval,
    ) -> ChartResult<i128> {
        interval.validate()?;
        self.offset_ns(ns)?;
        if matches!(
            interval.unit,
            CalendarUnit::SourceTick | CalendarUnit::Millisecond
        ) {
            let step = if interval.unit == CalendarUnit::SourceTick {
                quantum(unit)
            } else {
                1_000_000
            } * i128::from(interval.step);
            let ns = ns.div_euclid(step) * step;
            self.offset_ns(ns)?;
            return Ok(ns);
        }
        if interval.unit == CalendarUnit::Year {
            let c = self.components_ns(ns)?;
            let year =
                i64::from(c.year).div_euclid(i64::from(interval.step)) * i64::from(interval.step);
            let ns = self.set_date(ns, year, 1, 1)?;
            return self.midnight(ns);
        }
        for _ in 0..WORK_LIMIT {
            ns = self.base_floor(ns, interval.unit)?;
            if self.aligned(ns, interval)? {
                return Ok(ns);
            }
            ns -= quantum(unit).min(1_000_000);
        }
        Err(error(
            DiagnosticCode::ResourceLimit,
            "Calendar floor exhausted its bounded field search.",
        ))
    }
    pub(super) fn ceil_ns(
        &self,
        ns: i128,
        unit: TimeUnit,
        interval: CalendarInterval,
    ) -> ChartResult<i128> {
        let floor = self.floor_ns(ns, unit, interval)?;
        if floor == ns {
            return Ok(ns);
        }
        self.floor_ns(self.shift_ns(floor, unit, interval, 1)?, unit, interval)
    }
    fn midnight(&self, ns: i128) -> ChartResult<i128> {
        let wall = self.wall(ns)?;
        self.resolve_wall(wall.div_euclid(DAY) * DAY)
    }
    fn base_floor(&self, ns: i128, field: CalendarUnit) -> ChartResult<i128> {
        let c = self.components_ns(ns)?;
        let result = match field {
            CalendarUnit::Second => ns - i128::from(c.nanosecond),
            CalendarUnit::Minute => ns - i128::from(c.second) * SECOND - i128::from(c.nanosecond),
            CalendarUnit::Hour => {
                ns - i128::from(c.minute) * MINUTE
                    - i128::from(c.second) * SECOND
                    - i128::from(c.nanosecond)
            }
            CalendarUnit::Day => self.midnight(ns)?,
            CalendarUnit::UnixDay => ns.div_euclid(DAY) * DAY,
            CalendarUnit::Week(start) => {
                let days = civil::days_from_civil(c.year, c.month, c.day)?;
                let delta = (civil::weekday(days) + 7 - start.index()) % 7;
                self.midnight(self.set_date(
                    ns,
                    i64::from(c.year),
                    i64::from(c.month),
                    i64::from(c.day) - i64::from(delta),
                )?)?
            }
            CalendarUnit::Month => {
                self.midnight(self.set_date(ns, i64::from(c.year), i64::from(c.month), 1)?)?
            }
            _ => {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Invalid base calendar floor.",
                ));
            }
        };
        self.offset_ns(result)?;
        Ok(result)
    }
    fn aligned(&self, ns: i128, interval: CalendarInterval) -> ChartResult<bool> {
        if interval.step == 1 {
            return Ok(true);
        }
        let c = self.components_ns(ns)?;
        let value = match interval.unit {
            CalendarUnit::Second => ns.div_euclid(SECOND).rem_euclid(60) as i64,
            CalendarUnit::Minute => i64::from(c.minute),
            CalendarUnit::Hour => i64::from(c.hour),
            CalendarUnit::Day => i64::from(c.day) - 1,
            CalendarUnit::UnixDay => (ns.div_euclid(DAY)) as i64,
            CalendarUnit::Week(start) => {
                let day = civil::days_from_civil(c.year, c.month, c.day)?;
                let anchor = -(4 - i64::from(start.index())).rem_euclid(7);
                (day - anchor).div_euclid(7)
            }
            CalendarUnit::Month => i64::from(c.month) - 1,
            CalendarUnit::Year => i64::from(c.year),
            _ => return Ok(true),
        };
        Ok(value.rem_euclid(i64::from(interval.step)) == 0)
    }
    fn set_date(&self, ns: i128, year: i64, month: i64, day: i64) -> ChartResult<i128> {
        let month_index = i128::from(year) * 12 + i128::from(month) - 1;
        let year = i32::try_from(month_index.div_euclid(12))
            .map_err(|_| error(DiagnosticCode::PrecisionLoss, "Calendar year overflow."))?;
        let month = month_index.rem_euclid(12) as u32 + 1;
        let day = i128::from(civil::days_from_civil(year, month, 1)?) + i128::from(day) - 1;
        self.resolve_wall(day * DAY + self.wall(ns)?.rem_euclid(DAY))
    }
    fn base_offset(
        &self,
        ns: i128,
        unit: TimeUnit,
        field: CalendarUnit,
        steps: i64,
    ) -> ChartResult<i128> {
        let n = i128::from(steps);
        let result = match field {
            CalendarUnit::SourceTick => ns + n * quantum(unit),
            CalendarUnit::Millisecond => ns + n * 1_000_000,
            CalendarUnit::Second => ns + n * SECOND,
            CalendarUnit::Minute => ns + n * MINUTE,
            CalendarUnit::Hour => ns + n * HOUR,
            CalendarUnit::UnixDay => ns + n * DAY,
            CalendarUnit::Day
            | CalendarUnit::Week(_)
            | CalendarUnit::Month
            | CalendarUnit::Year => {
                let c = self.components_ns(ns)?;
                let (y, m, d) = (i64::from(c.year), i64::from(c.month), i64::from(c.day));
                match field {
                    CalendarUnit::Day => self.set_date(ns, y, m, d + steps)?,
                    CalendarUnit::Week(_) => self.set_date(ns, y, m, d + steps * 7)?,
                    CalendarUnit::Month => self.set_date(ns, y, m + steps, d)?,
                    CalendarUnit::Year => self.set_date(ns, y + steps, m, d)?,
                    _ => unreachable!(),
                }
            }
        };
        self.offset_ns(result)?;
        Ok(result)
    }
    fn shift_ns(
        &self,
        mut ns: i128,
        unit: TimeUnit,
        interval: CalendarInterval,
        steps: i64,
    ) -> ChartResult<i128> {
        interval.validate()?;
        self.offset_ns(ns)?;
        if interval.step == 1 {
            return self.base_offset(ns, unit, interval.unit, steps);
        }
        if matches!(
            interval.unit,
            CalendarUnit::SourceTick | CalendarUnit::Millisecond | CalendarUnit::Year
        ) {
            return self.base_offset(
                ns,
                unit,
                interval.unit,
                steps.checked_mul(i64::from(interval.step)).ok_or_else(|| {
                    error(
                        DiagnosticCode::PrecisionLoss,
                        "Calendar offset multiplier overflow.",
                    )
                })?,
            );
        }
        let mut remaining = steps.unsigned_abs();
        let direction = steps.signum();
        for _ in 0..WORK_LIMIT {
            if remaining == 0 {
                return Ok(ns);
            }
            ns = self.base_offset(ns, unit, interval.unit, direction)?;
            if self.aligned(ns, interval)? {
                remaining -= 1;
            }
        }
        Err(error(
            DiagnosticCode::ResourceLimit,
            "Calendar offset exhausted its bounded field search.",
        ))
    }
}
pub(super) fn quantum(unit: TimeUnit) -> i128 {
    match unit {
        TimeUnit::Seconds => SECOND,
        TimeUnit::Milliseconds => 1_000_000,
        TimeUnit::Microseconds => 1000,
        TimeUnit::Nanoseconds => 1,
    }
}
pub(super) fn source_ns(value: i64, unit: TimeUnit) -> i128 {
    i128::from(value) * quantum(unit)
}
pub(super) fn source_value(ns: i128, unit: TimeUnit) -> ChartResult<i64> {
    let quantum = quantum(unit);
    if ns.rem_euclid(quantum) != 0 {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Calendar boundary is finer than the declared source timestamp unit.",
        ));
    }
    i64::try_from(ns / quantum).map_err(|_| {
        error(
            DiagnosticCode::PrecisionLoss,
            "Calendar timestamp exceeds the source integer representation.",
        )
    })
}
fn date_ns(ns: i128) -> ChartResult<()> {
    if !(-DATE_LIMIT..=DATE_LIMIT).contains(&ns) {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Calendar instant exceeds the common ECMAScript Date range.",
        ));
    }
    Ok(())
}
