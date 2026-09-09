//! Explicit d3-time-format 4.1.0 calendar labels; ISC notice in `LICENSE-d3-time-format`.
use super::{
    Calendar, CalendarDateTime, CalendarInterval, CalendarUnit, CalendarZone, WeekStart, calendar,
    civil, error,
};
use crate::{ChartResult, DiagnosticCode, data::TimeUnit};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Explicit date/time patterns and names; no system locale lookup occurs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default, rename_all = "camelCase")]
pub struct TimeLocale {
    /// Expansion of `%c`.
    pub date_time: String,
    /// Expansion of `%x`.
    pub date: String,
    /// Expansion of `%X`.
    pub time: String,
    /// AM and PM labels.
    pub periods: [String; 2],
    /// Sunday through Saturday.
    pub days: [String; 7],
    /// Abbreviated Sunday through Saturday.
    pub short_days: [String; 7],
    /// January through December.
    pub months: [String; 12],
    /// Abbreviated January through December.
    pub short_months: [String; 12],
}
impl Default for TimeLocale {
    fn default() -> Self {
        Self {
            date_time: "%x, %X".into(),
            date: "%-m/%-d/%Y".into(),
            time: "%-I:%M:%S %p".into(),
            periods: ["AM", "PM"].map(Into::into),
            days: [
                "Sunday",
                "Monday",
                "Tuesday",
                "Wednesday",
                "Thursday",
                "Friday",
                "Saturday",
            ]
            .map(Into::into),
            short_days: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map(Into::into),
            months: [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ]
            .map(Into::into),
            short_months: [
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
            ]
            .map(Into::into),
        }
    }
}
impl TimeLocale {
    /// Validate bounded printable names and reject recursive locale pattern cycles.
    pub fn validate(&self) -> ChartResult<()> {
        if self
            .periods
            .iter()
            .chain(&self.days)
            .chain(&self.short_days)
            .chain(&self.months)
            .chain(&self.short_months)
            .any(|s| s.len() > 128 || s.chars().any(char::is_control))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Time locale names exceed their printable text budget.",
            ));
        }
        for pattern in [&self.date_time, &self.date, &self.time] {
            compile(pattern, self)?;
        }
        Ok(())
    }
}
/// Default conditional labels or a custom D3 time pattern and explicit locale.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeFormat {
    /// Missing selects default conditional labels; unknown directives remain literal.
    #[serde(default)]
    pub pattern: Option<String>,
    /// Explicit patterns/names, defaulting to the pinned en-US descriptor.
    #[serde(default)]
    pub locale: TimeLocale,
}
impl TimeFormat {
    /// Validate and prepare with the same calendar used by tick arithmetic.
    pub fn prepare(&self, calendar: Calendar) -> ChartResult<TimeFormatter> {
        TimeFormatter::new(calendar, self.clone())
    }
}
#[derive(Clone, Debug)]
enum Token {
    Text(String),
    Field(char, Option<char>),
}
/// Immutable prepared labels sharing the calendar's exact timezone revision.
#[derive(Clone, Debug)]
pub struct TimeFormatter {
    calendar: Calendar,
    spec: Arc<TimeFormat>,
    pattern: Option<Vec<Token>>,
    defaults: [Vec<Token>; 8],
}
impl TimeFormatter {
    /// Compile patterns and validate explicit locale resources once.
    pub fn new(calendar: Calendar, spec: TimeFormat) -> ChartResult<Self> {
        spec.locale.validate()?;
        let pattern = spec
            .pattern
            .as_ref()
            .map(|p| compile(p, &spec.locale))
            .transpose()?;
        let defaults = [".%L", ":%S", "%I:%M", "%I %p", "%a %d", "%b %d", "%B", "%Y"]
            .into_iter()
            .map(|p| compile(p, &spec.locale))
            .collect::<ChartResult<Vec<_>>>()?
            .try_into()
            .expect("eight default patterns");
        Ok(Self {
            calendar,
            spec: Arc::new(spec),
            pattern,
            defaults,
        })
    }
    /// Original owned formatting policy.
    pub fn spec(&self) -> &TimeFormat {
        &self.spec
    }
    /// Calendar identity shared with floor, ticks and nice.
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }
    /// Format an exact source timestamp; submillisecond defaults retain native precision.
    pub fn format(&self, value: i64, unit: TimeUnit) -> ChartResult<String> {
        let c = self.calendar.components(value, unit)?;
        let tokens = if let Some(pattern) = &self.pattern {
            pattern
        } else {
            if c.nanosecond % 1000 != 0 {
                return Ok(format!(".{:09}", c.nanosecond));
            }
            if c.nanosecond % 1_000_000 != 0 {
                return Ok(format!(".{:06}", c.nanosecond / 1000));
            }
            let earlier =
                |field| match self
                    .calendar
                    .floor(value, unit, CalendarInterval::new(field))
                {
                    Ok(floor) => Ok(floor < value),
                    // D3 compares an invalid floor Date as false at the supported Date edges.
                    Err(e)
                        if matches!(self.calendar.zone(), CalendarZone::Utc)
                            && e.code == DiagnosticCode::NumericalDomain =>
                    {
                        Ok(false)
                    }
                    Err(e) => Err(e),
                };
            let index = if earlier(CalendarUnit::Second)? {
                0
            } else if earlier(CalendarUnit::Minute)? {
                1
            } else if earlier(CalendarUnit::Hour)? {
                2
            } else if earlier(CalendarUnit::Day)? {
                3
            } else if earlier(CalendarUnit::Month)? {
                if earlier(CalendarUnit::Week(WeekStart::Sunday))? {
                    4
                } else {
                    5
                }
            } else if earlier(CalendarUnit::Year)? {
                6
            } else {
                7
            };
            &self.defaults[index]
        };
        let mut result = String::new();
        for token in tokens {
            match token {
                Token::Text(s) => result.push_str(s),
                Token::Field(field, padding) => {
                    result.push_str(&self.field(*field, *padding, c, value, unit)?)
                }
            }
            if result.len() > 16384 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Formatted time label exceeds its byte budget.",
                ));
            }
        }
        Ok(result)
    }
    fn field(
        &self,
        field: char,
        padding: Option<char>,
        c: CalendarDateTime,
        value: i64,
        unit: TimeUnit,
    ) -> ChartResult<String> {
        let locale = &self.spec.locale;
        let day = civil::days_from_civil(c.year, c.month, c.day)?;
        let weekday = civil::weekday(day);
        let jan1 = civil::days_from_civil(c.year, 1, 1)?;
        let yearday = day - jan1;
        let iso = || -> ChartResult<(i32, i64)> {
            let thursday = day + 3 - i64::from((weekday + 6) % 7);
            let (year, _, _) = civil::civil_from_days(thursday)?;
            Ok((
                year,
                (thursday - civil::days_from_civil(year, 1, 1)?).div_euclid(7) + 1,
            ))
        };
        let pad = |n: i64, width| padded(n, padding, width);
        if matches!(self.calendar.zone(), CalendarZone::Utc) {
            // Reference formatters use Date-valued year/week intermediates. Preserve
            // their invalid-Date text, including padding, when an intermediate overflows.
            let min = -100_000_000;
            let previous_thursday = day - i64::from((weekday + 3) % 7);
            let iso_invalid = previous_thursday < min
                || (weekday < 4
                    && weekday != 0
                    && calendar::source_ns(value, unit) == -100_000_000 * calendar::DAY);
            let invalid = match field {
                'j' => jan1 < min,
                'U' => jan1 - 1 - i64::from(civil::weekday(jan1 - 1)) < min,
                'W' => jan1 - 1 - i64::from((civil::weekday(jan1 - 1) + 6) % 7) < min,
                'g' | 'G' => iso_invalid,
                'V' => {
                    iso_invalid || {
                        let first = civil::days_from_civil(iso()?.0, 1, 1)?;
                        first - i64::from((civil::weekday(first) + 3) % 7) < min
                    }
                }
                _ => false,
            };
            if invalid {
                let width: usize = if field == 'G' {
                    4
                } else if field == 'j' {
                    3
                } else {
                    2
                };
                return Ok(format!(
                    "{}NaN",
                    padding.map_or_else(String::new, |c| c
                        .to_string()
                        .repeat(width.saturating_sub(3)))
                ));
            }
        }

        Ok(match field {
            'a' => locale.short_days[weekday as usize].clone(),
            'A' => locale.days[weekday as usize].clone(),
            'b' => locale.short_months[c.month as usize - 1].clone(),
            'B' => locale.months[c.month as usize - 1].clone(),
            'd' | 'e' => pad(i64::from(c.day), 2),
            'H' => pad(i64::from(c.hour), 2),
            'I' => pad(
                i64::from(if c.hour.is_multiple_of(12) {
                    12
                } else {
                    c.hour % 12
                }),
                2,
            ),
            'j' => pad(yearday + 1, 3),
            'L' => pad(i64::from(c.nanosecond / 1_000_000), 3),
            // On millisecond inputs this is exactly D3's millisecond text plus three zeros.
            // Finer typed timestamps preserve their additional microsecond digits.
            'f' => format!(
                "{}{:03}",
                pad(i64::from(c.nanosecond / 1_000_000), 3),
                c.nanosecond / 1000 % 1000
            ),
            'm' => pad(i64::from(c.month), 2),
            'M' => pad(i64::from(c.minute), 2),
            'S' => pad(i64::from(c.second), 2),
            'p' => locale.periods[(c.hour >= 12) as usize].clone(),
            'q' => ((c.month - 1) / 3 + 1).to_string(),
            'u' => if weekday == 0 { 7 } else { weekday }.to_string(),
            'w' => weekday.to_string(),
            'U' => pad((yearday + 7 - i64::from(weekday)).div_euclid(7), 2),
            'W' => pad(
                (yearday + 7 - i64::from((weekday + 6) % 7)).div_euclid(7),
                2,
            ),
            'V' => pad(iso()?.1, 2),
            'g' => pad(i64::from(iso()?.0 % 100), 2),
            'G' => pad(i64::from(iso()?.0 % 10000), 4),
            'y' => pad(i64::from(c.year % 100), 2),
            'Y' => pad(i64::from(c.year % 10000), 4),
            'Z' => {
                let minutes = c.offset_seconds / 60;
                format!(
                    "{}{:02}{:02}",
                    if minutes < 0 { '-' } else { '+' },
                    minutes.unsigned_abs() / 60,
                    minutes.unsigned_abs() % 60
                )
            }
            'Q' => calendar::source_ns(value, unit)
                .div_euclid(1_000_000)
                .to_string(),
            's' => calendar::source_ns(value, unit)
                .div_euclid(calendar::SECOND)
                .to_string(),
            '%' => "%".into(),
            _ => field.to_string(),
        })
    }
}
fn padded(value: i64, padding: Option<char>, width: usize) -> String {
    let digits = value.unsigned_abs().to_string();
    format!(
        "{}{}{}",
        if value < 0 { "-" } else { "" },
        padding.map_or_else(String::new, |c| c
            .to_string()
            .repeat(width.saturating_sub(digits.len()))),
        digits
    )
}
fn compile(pattern: &str, locale: &TimeLocale) -> ChartResult<Vec<Token>> {
    fn append(
        pattern: &str,
        locale: &TimeLocale,
        depth: usize,
        out: &mut Vec<Token>,
    ) -> ChartResult<()> {
        if depth > 8 || pattern.len() > 4096 || pattern.chars().any(char::is_control) {
            return Err(error(
                DiagnosticCode::Validation,
                "Time pattern exceeds its printable size or recursive expansion budget.",
            ));
        }
        let mut chars = pattern.chars();
        let mut text = String::new();
        while let Some(c) = chars.next() {
            if c != '%' {
                text.push(c);
                continue;
            }
            if !text.is_empty() {
                out.push(Token::Text(std::mem::take(&mut text)));
            }
            let Some(mut field) = chars.next() else {
                break;
            };
            let padding = match field {
                '-' => {
                    field = chars.next().unwrap_or('\0');
                    None
                }
                '_' => {
                    field = chars.next().unwrap_or('\0');
                    Some(' ')
                }
                '0' => {
                    field = chars.next().unwrap_or('\0');
                    Some('0')
                }
                _ => Some(if field == 'e' { ' ' } else { '0' }),
            };
            match field {
                'c' => append(&locale.date_time, locale, depth + 1, out)?,
                'x' => append(&locale.date, locale, depth + 1, out)?,
                'X' => append(&locale.time, locale, depth + 1, out)?,
                '\0' => {}
                _ => out.push(Token::Field(field, padding)),
            }
            if out.len() > 1024 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Time pattern token expansion exceeds its budget.",
                ));
            }
        }
        if !text.is_empty() {
            out.push(Token::Text(text));
        }
        Ok(())
    }
    let mut tokens = Vec::new();
    append(pattern, locale, 0, &mut tokens)?;
    Ok(tokens)
}
