//! Reference datetime guide selection over the shared calendar and formatter.
//! Candidate policies are checked against pinned R fixtures; no calendar engine lives here.
use super::{Bounds, Calendar, CalendarInterval, CalendarUnit, TimeFormat, WeekStart, error, utc};
use crate::{ChartResult, DiagnosticCode, data::TimeUnit};

/// Selected reference datetime candidates and their default labels.
#[derive(Clone, Debug, PartialEq)]
pub struct GgplotTimeBreaks {
    /// Exact source timestamps after visible-range censoring.
    pub values: Vec<i64>,
    /// Reference-selected formatting, evaluated by the shared time formatter.
    pub labels: Vec<String>,
}
#[derive(Clone, Copy)]
enum Progress {
    Elapsed(i64),
    Days(i32),
    Months(i32),
    HalfMonth,
    Years(i32),
}
#[derive(Clone, Copy)]
struct Step {
    seconds: f64,
    anchor: CalendarInterval,
    progress: Progress,
    pattern: &'static str,
}
fn steps(span: f64) -> Vec<Step> {
    use CalendarUnit::{Day, Hour, Minute, Month, Week, Year};
    let minute = CalendarInterval::new(Minute);
    let hour = CalendarInterval::new(Hour);
    let day = CalendarInterval::new(Day);
    let month = CalendarInterval::new(Month);
    let year = CalendarInterval::new(Year);
    let mut steps = Vec::with_capacity(33);
    for seconds in [1, 2, 5, 10, 15, 30] {
        steps.push(Step {
            seconds: seconds as f64,
            anchor: minute,
            progress: Progress::Elapsed(seconds),
            pattern: if seconds < 30 { "%S" } else { "%H:%M:%S" },
        });
    }
    for minutes in [1, 2, 5, 10, 15, 30] {
        steps.push(Step {
            seconds: (minutes * 60) as f64,
            anchor: if minutes == 1 { minute } else { hour },
            progress: Progress::Elapsed(minutes * 60),
            pattern: "%H:%M",
        });
    }
    for hours in [1, 3, 6, 12] {
        steps.push(Step {
            seconds: (hours * 3600) as f64,
            anchor: if hours == 1 { hour } else { day },
            progress: Progress::Elapsed(hours * 3600),
            pattern: if hours <= 3 && span <= 86400. {
                "%H:%M"
            } else {
                "%b %d %H:%M"
            },
        });
    }
    for days in [1, 2] {
        steps.push(Step {
            seconds: f64::from(days) * 86400.,
            anchor: day,
            progress: Progress::Days(days),
            pattern: "%b %d",
        });
    }
    steps.push(Step {
        seconds: 7. * 86400.,
        anchor: CalendarInterval::new(Week(WeekStart::Monday)),
        progress: Progress::Elapsed(7 * 86400),
        pattern: "%b %d",
    });
    const YEAR_SECONDS: f64 = 365.25 * 86400.;
    steps.push(Step {
        seconds: YEAR_SECONDS / 24.,
        anchor: month,
        progress: Progress::HalfMonth,
        pattern: "%b %d",
    });
    for months in [1, 3, 6] {
        steps.push(Step {
            seconds: f64::from(months) * YEAR_SECONDS / 12.,
            anchor: if months == 1 { month } else { year },
            progress: Progress::Months(months),
            pattern: if months == 6 {
                "%Y-%m"
            } else if span < YEAR_SECONDS {
                "%b"
            } else {
                "%b %Y"
            },
        });
    }
    for years in [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000] {
        steps.push(Step {
            seconds: f64::from(years) * YEAR_SECONDS,
            anchor: CalendarInterval {
                unit: Year,
                step: if years < 2 {
                    1
                } else if years < 20 {
                    10
                } else {
                    100
                },
            },
            progress: Progress::Years(years),
            pattern: "%Y",
        });
    }
    steps
}
struct Search<'a> {
    calendar: &'a Calendar,
    unit: TimeUnit,
    origin: i64,
    work: usize,
}
impl Search<'_> {
    fn advance(&mut self, anchor: i64, step: Step, index: i32) -> ChartResult<i64> {
        self.work += 1;
        crate::limits::require_within(self.work <= 16384, "automatic datetime search work")?;
        let interval = |unit| CalendarInterval::new(unit);
        match step.progress {
            Progress::Elapsed(seconds) => {
                let value = i128::from(anchor)
                    + i128::from(seconds) * i128::from(index) * utc::ticks_per_second(self.unit);
                i64::try_from(value).map_err(|_| {
                    error(
                        DiagnosticCode::PrecisionLoss,
                        "Datetime candidate exceeds source integer range.",
                    )
                })
            }
            Progress::Days(n) => self.calendar.offset(
                anchor,
                self.unit,
                interval(CalendarUnit::Day),
                f64::from(index) * f64::from(n),
            ),
            Progress::Months(n) => self.calendar.offset(
                anchor,
                self.unit,
                interval(CalendarUnit::Month),
                f64::from(index) * f64::from(n),
            ),
            Progress::Years(n) => self.calendar.offset(
                anchor,
                self.unit,
                interval(CalendarUnit::Year),
                f64::from(index) * f64::from(n),
            ),
            Progress::HalfMonth => {
                let value = self.calendar.offset(
                    anchor,
                    self.unit,
                    interval(CalendarUnit::Month),
                    f64::from(index.div_euclid(2)),
                )?;
                if index.rem_euclid(2) == 0 {
                    Ok(value)
                } else {
                    let mut date = self.calendar.components(value, self.unit)?;
                    date.day = 15;
                    self.calendar.from_components(date, self.unit)
                }
            }
        }
    }
    fn relative(&self, value: i64) -> f64 {
        (i128::from(value) - i128::from(self.origin)) as f64
    }
    fn candidates(&mut self, view: Bounds, step: Step) -> ChartResult<Vec<i64>> {
        let low = utc::absolute_number(view.minimum().floor(), self.origin)?;
        let anchor = self.calendar.floor(low, self.unit, step.anchor)?;
        let mut values = Vec::new();
        let mut previous = None;
        for index in 0..16384 {
            let value = self.advance(anchor, step, index)?;
            if previous.is_some_and(|p| value <= p) {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Datetime candidates must advance strictly.",
                ));
            }
            previous = Some(value);
            values.push(value);
            if self.relative(value) >= view.maximum() {
                break;
            }
        }
        let before = values.partition_point(|v| self.relative(*v) <= view.minimum());
        if before > 1 {
            values.drain(..before - 1);
        }
        Ok(values)
    }
}

/// Reference automatic datetime candidates for an origin-relative view in source units.
/// Density is bounded separately from returned tick count and calendar search work.
pub fn ggplot_breaks_pretty_time(
    origin: i64,
    view: Bounds,
    unit: TimeUnit,
    calendar: &Calendar,
    count: f64,
    budget: usize,
) -> ChartResult<GgplotTimeBreaks> {
    pretty_time(origin, view, unit, calendar, count, budget, true)
}

pub(crate) fn pretty_time(
    origin: i64,
    view: Bounds,
    unit: TimeUnit,
    calendar: &Calendar,
    count: f64,
    budget: usize,
    crop: bool,
) -> ChartResult<GgplotTimeBreaks> {
    if !count.is_finite() || !(1. ..=128.).contains(&count) {
        return Err(error(
            DiagnosticCode::Validation,
            "Automatic datetime density must be finite and between 1 and 128.",
        ));
    }
    let view = Bounds::new(view.minimum(), view.maximum())?;
    if view.minimum().ceil() > view.maximum().floor() {
        return Ok(GgplotTimeBreaks {
            values: Vec::new(),
            labels: Vec::new(),
        });
    }
    let factor = utc::ticks_per_second(unit) as f64;
    let epoch = origin as f64 / factor;
    let constant =
        super::ggplot::zero_range(epoch + view.start() / factor, epoch + view.end() / factor);
    let (values, pattern) = if constant {
        let value = utc::absolute_number(view.start(), origin)?;
        let date = calendar.components(value, unit)?;
        let pattern =
            if date.hour == 0 && date.minute == 0 && date.second == 0 && date.nanosecond == 0 {
                "%Y-%m-%d"
            } else {
                "%Y-%m-%d %H:%M:%S"
            };
        (vec![value], pattern)
    } else {
        let span = (view.end() - view.start()) / factor;
        let search_view = if span < 1. {
            let margin = (count / 2.).min(30.);
            let remainder = origin.rem_euclid(utc::ticks_per_second(unit) as i64) as f64;
            Bounds::new(
                ((remainder + view.start()) / factor - margin).floor() * factor - remainder,
                ((remainder + view.end()) / factor + margin).ceil() * factor - remainder,
            )?
        } else {
            view
        };
        let span = (search_view.end() - search_view.start()) / factor;
        let options = steps(span);
        let initial = options
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (span / a.seconds - count)
                    .abs()
                    .total_cmp(&(span / b.seconds - count).abs())
            })
            .expect("fixed candidates")
            .0;
        let mut index = initial;
        let mut search = Search {
            calendar,
            unit,
            origin,
            work: 0,
        };
        let mut chosen = search.candidates(search_view, options[index])?;
        let minimum = (count / 2.).floor() as usize;
        let mut right = true;
        while chosen.len().saturating_sub(1) < minimum {
            if index > 0 {
                index -= 1;
                chosen = search.candidates(search_view, options[index])?;
            } else {
                let anchor = if right {
                    *chosen.last().expect("candidates")
                } else {
                    chosen[0]
                };
                let value = search.advance(anchor, options[0], if right { 1 } else { -1 })?;
                if right {
                    chosen.push(value);
                } else {
                    chosen.insert(0, value);
                }
                right = !right;
            }
        }
        let excess = chosen.len() as f64 - 1. - count;
        if excess > 0. {
            let mut left = chosen
                .iter()
                .filter(|v| search.relative(**v) <= view.start())
                .count()
                .saturating_sub(1);
            let mut right = chosen
                .iter()
                .filter(|v| search.relative(**v) >= view.end())
                .count()
                .saturating_sub(1);
            let outside = left + right;
            if outside > 0 && excess < outside as f64 {
                left = (excess * left as f64 / outside as f64).round_ties_even() as usize;
                right = (excess - left as f64).max(0.) as usize;
            }
            chosen.truncate(chosen.len().saturating_sub(right));
            chosen.drain(..left.min(chosen.len()));
        }
        let difference = chosen.len() as f64 - 1. - count;
        if difference != 0.
            && !(difference > 0. && index < initial)
            && !(difference < 0. && index == 0)
        {
            let neighbor = if difference > 0. {
                (index + 1).min(options.len() - 1)
            } else {
                index - 1
            };
            let next = search.candidates(search_view, options[neighbor])?;
            let intervals = next.len().saturating_sub(1);
            if intervals >= minimum && (intervals as f64 - count).abs() < difference.abs() {
                chosen = next;
                index = neighbor;
            }
        }
        if crop {
            chosen.retain(|value| view.contains(search.relative(*value)));
        }
        (chosen, options[index].pattern)
    };
    crate::limits::require_within(values.len() <= budget, "automatic datetime tick")?;
    let formatter = TimeFormat {
        pattern: Some(pattern.into()),
        ..Default::default()
    }
    .prepare(calendar.clone())?;
    let labels = values
        .iter()
        .map(|value| formatter.format(*value, unit))
        .collect::<ChartResult<_>>()?;
    Ok(GgplotTimeBreaks { values, labels })
}

/// Reference Date guides over exact UTC timestamps. Expansion and density are
/// interpreted in days; fractional-day observations keep their source timestamps.
pub fn ggplot_breaks_pretty_date(
    origin: i64,
    view: Bounds,
    unit: TimeUnit,
    count: f64,
    budget: usize,
) -> ChartResult<GgplotTimeBreaks> {
    pretty_date(origin, view, unit, count, budget, true)
}

pub(crate) fn pretty_date(
    origin: i64,
    view: Bounds,
    unit: TimeUnit,
    count: f64,
    budget: usize,
    crop: bool,
) -> ChartResult<GgplotTimeBreaks> {
    if !count.is_finite() || !(1. ..=128.).contains(&count) {
        return Err(error(
            DiagnosticCode::Validation,
            "Automatic Date density must be finite and between 1 and 128.",
        ));
    }
    let view = Bounds::new(view.minimum(), view.maximum())?;
    let calendar = Calendar::new(super::CalendarZone::Utc)?;
    let day = utc::ticks_per_second(unit) * 86400;
    let factor = day as f64;
    let epoch = origin as f64 / factor;
    let constant =
        super::ggplot::zero_range(epoch + view.start() / factor, epoch + view.end() / factor);
    let span = (view.end() - view.start()) / factor;
    if !constant && span > count {
        // At this density the common time selector uses whole-day or coarser
        // UTC boundaries; the shared calendar produces exact Date positions.
        return pretty_time(origin, view, unit, &calendar, count, budget, crop);
    }
    let values = if constant {
        vec![utc::absolute_number(view.start(), origin)?]
    } else {
        let low = utc::absolute_number(view.start().floor(), origin)?;
        let high = utc::absolute_number(view.end().floor(), origin)?;
        let padding = (count - span).round_ties_even() as i128;
        let left = (padding.div_euclid(2)).max(0);
        let right = left + padding.rem_euclid(2);
        let mut first_day = i128::from(low).div_euclid(day) - left;
        let mut last_day = i128::from(high).div_euclid(day) + right;
        let minimum = (count / 2.).floor() as i128 + 1;
        while last_day - first_day + 1 < minimum {
            if i128::from(low).div_euclid(day) - first_day
                < last_day - i128::from(high).div_euclid(day)
            {
                first_day -= 1;
            } else {
                last_day += 1;
            }
        }
        let length = (last_day - first_day + 1).max(0);
        // Requested density is bounded to 128; padding may lie outside the viewport.
        crate::limits::require_within(length <= budget as i128 + 130, "automatic Date candidate")?;
        let mut values = Vec::new();
        for index in first_day..=last_day {
            let value = i64::try_from(index * day).map_err(|_| {
                error(
                    DiagnosticCode::PrecisionLoss,
                    "Date break exceeds source integer range.",
                )
            })?;
            if !crop || view.contains((i128::from(value) - i128::from(origin)) as f64) {
                values.push(value);
            }
        }
        values
    };
    crate::limits::require_within(values.len() <= budget, "automatic Date tick")?;
    let formatter = TimeFormat {
        pattern: Some(if constant { "%Y-%m-%d" } else { "%b %d" }.into()),
        ..Default::default()
    }
    .prepare(calendar)?;
    let labels = values
        .iter()
        .map(|v| formatter.format(*v, unit))
        .collect::<ChartResult<_>>()?;
    Ok(GgplotTimeBreaks { values, labels })
}
