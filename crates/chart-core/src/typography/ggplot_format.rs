//! Independent implementation of the reference's vector-wide default numeric labels.
use crate::{ChartResult, DiagnosticCode};

fn scientific(value: f64, decimals: usize, exponent_width: usize) -> String {
    let text = format!("{value:.decimals$e}");
    let (mantissa, exponent) = text
        .split_once('e')
        .expect("finite scientific representation");
    let exponent: i32 = exponent.parse().expect("bounded numeric exponent");
    format!(
        "{mantissa}e{}{abs:0exponent_width$}",
        if exponent < 0 { '-' } else { '+' },
        abs = exponent.unsigned_abs()
    )
}

/// Format finite ggplot2 default numeric labels as one vector (R default digits=7).
/// Precision and fixed/scientific notation are shared across the vector. The output
/// uses ASCII decimal punctuation and is bounded by the supplied total byte budget.
pub fn ggplot_numeric_labels(values: &[f64], max_bytes: usize) -> ChartResult<Vec<String>> {
    crate::limits::require_within(values.len() <= max_bytes, "numeric label byte")?;
    let mut decimals = 0;
    let mut scientific_decimals = 0;
    let exponent_width = 2;
    for &value in values {
        if !value.is_finite() {
            return Err(crate::scales::error(
                DiagnosticCode::NumericalDomain,
                "Numeric guide labels require finite values.",
            ));
        }
        let text = format!("{value:.6e}");
        let (mantissa, exponent) = text
            .split_once('e')
            .expect("finite scientific representation");
        let exponent: i32 = exponent.parse().expect("bounded numeric exponent");
        let fraction = mantissa
            .split_once('.')
            .map_or("", |(_, tail)| tail)
            .trim_end_matches('0')
            .len();
        scientific_decimals = scientific_decimals.max(fraction);
        decimals = decimals.max((fraction as i32 - exponent).max(0) as usize);
    }
    let mut fixed_width = 0;
    let mut sci_width = 0;
    for &value in values {
        let value = if value == 0. { 0. } else { value };
        fixed_width = fixed_width.max(format!("{value:.decimals$}").len());
        sci_width = sci_width.max(scientific(value, scientific_decimals, exponent_width).len());
    }
    let fixed = fixed_width <= sci_width;
    let mut used = 0usize;
    values
        .iter()
        .map(|&value| {
            let value = if value == 0. { 0. } else { value };
            let text = if fixed {
                format!("{value:.decimals$}")
            } else {
                scientific(value, scientific_decimals, exponent_width)
            };
            crate::limits::require_within(
                text.len() <= max_bytes.saturating_sub(used),
                "numeric label byte",
            )?;
            used += text.len();
            Ok(text)
        })
        .collect()
}
