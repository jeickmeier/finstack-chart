//! Reference hms duration labels; seconds never acquire a calendar or timezone.
use crate::{ChartResult, DiagnosticCode};

/// Format a vector of elapsed seconds with hms 1.1.4 default six-digit precision,
/// shared fractional precision, hour width and final right alignment. Reject values
/// whose microsecond representation exceeds the exact integer range of an f64.
pub fn ggplot_duration_labels(values: &[f64], max_bytes: usize) -> ChartResult<Vec<String>> {
    crate::limits::require_within(values.len() <= max_bytes, "duration label byte")?;
    let mut parts = Vec::with_capacity(values.len());
    let mut decimals = 0;
    let mut hour_width = 2;
    for &value in values {
        let micros = value * 1_000_000.;
        if !micros.is_finite() || micros.abs() > 9_007_199_254_740_991. {
            return Err(crate::scales::error(
                DiagnosticCode::PrecisionLoss,
                "Duration labels require finite seconds with exactly representable microseconds.",
            ));
        }
        let rounded = micros.round_ties_even();
        let total = rounded.abs() as u64;
        let fraction = total % 1_000_000;
        let precision = if fraction == 0 && rounded != micros {
            6
        } else if fraction == 0 {
            0
        } else {
            format!("{fraction:06}").trim_end_matches('0').len()
        };
        decimals = decimals.max(precision);
        let hours = total / 3_600_000_000;
        hour_width = hour_width.max(hours.to_string().len());
        parts.push((
            rounded < 0.,
            hours,
            (total / 60_000_000) % 60,
            (total / 1_000_000) % 60,
            fraction,
        ));
    }
    let mut used = 0usize;
    let labels = parts
        .into_iter()
        .map(|(negative, hours, minutes, seconds, fraction)| {
            // hms pads the hours before joining, then right-aligns the complete vector.
            let mut label = format!(
                "{}{hours:>hour_width$}:{minutes:02}:{seconds:02}",
                if negative { "-" } else { "" },
                hours = format!("{hours:02}")
            );
            if decimals > 0 {
                label.push('.');
                label.push_str(&format!("{fraction:06}")[..decimals]);
            }
            crate::limits::require_within(
                label.len() <= max_bytes.saturating_sub(used),
                "duration label byte",
            )?;
            used += label.len();
            Ok(label)
        })
        .collect::<ChartResult<Vec<_>>>()?;
    let width = labels.iter().map(String::len).max().unwrap_or(0);
    crate::limits::require_within(
        width
            .checked_mul(labels.len())
            .is_some_and(|n| n <= max_bytes),
        "duration label byte",
    )?;
    Ok(labels
        .into_iter()
        .map(|label| format!("{label:>width$}"))
        .collect())
}
