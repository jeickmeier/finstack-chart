//! Shared proleptic Gregorian arithmetic for legacy UTC and supplied calendar rules.
use super::error;
use crate::{ChartResult, DiagnosticCode};

pub(super) fn leap(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}
pub(super) fn month_days(year: i32, month: u32) -> u32 {
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
pub(super) fn days_from_civil(year: i32, month: u32, day: u32) -> ChartResult<i64> {
    if !(1..=12).contains(&month) || day == 0 || day > month_days(year, month) {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Invalid proleptic Gregorian calendar date.",
        ));
    }
    let mut days = days_before_year(year) - days_before_year(1970);
    for m in 1..month {
        days += i64::from(month_days(year, m));
    }
    Ok(days + i64::from(day) - 1)
}
pub(super) fn civil_from_days(day: i64) -> ChartResult<(i32, u32, u32)> {
    let from_2000 = i128::from(day) - 10957;
    let era = from_2000.div_euclid(146097);
    let mut year = i32::try_from(2000 + 400 * era).map_err(|_| {
        error(
            DiagnosticCode::PrecisionLoss,
            "Calendar year exceeds its integer representation.",
        )
    })?;
    let mut remaining = from_2000.rem_euclid(146097) as i64;
    loop {
        let days = if leap(year) { 366 } else { 365 };
        if remaining < days {
            break;
        }
        remaining -= days;
        year = year
            .checked_add(1)
            .ok_or_else(|| error(DiagnosticCode::PrecisionLoss, "Calendar year overflow."))?;
    }
    let mut month = 1;
    while remaining >= i64::from(month_days(year, month)) {
        remaining -= i64::from(month_days(year, month));
        month += 1;
    }
    Ok((year, month, remaining as u32 + 1))
}
/// Sunday zero, independent of locale and UTC offset.
pub(super) fn weekday(day: i64) -> u32 {
    ((i128::from(day) + 4).rem_euclid(7)) as u32
}
