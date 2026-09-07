use super::*;
use crate::grammar::{PreparedChart, ValueSpace};

// Default follow policy: keep each explicit horizontal window's width and align its
// high endpoint to the latest retained prepared x extent. Automatic windows stay automatic.
// Vertical windows, source populations, visibility and authored annotations are untouched.
pub(super) fn advance(state: &mut ChartState, prepared: &PreparedChart) -> ChartResult<()> {
    if let (Some((a, b)), Some(extent)) = (state.durable.viewport.x, prepared.domains().x) {
        state.durable.viewport.x = Some(numeric(a, b, extent.maximum)?);
    }
    for (id, window) in &mut state.durable.windows {
        let Some(domain) = prepared.scale_domains().get(id) else {
            continue;
        };
        let Some(extent) = domain.x else { continue };
        match window {
            AxisWindow::Numeric(a, b) => {
                (*a, *b) = numeric(*a, *b, extent.maximum)?;
            }
            AxisWindow::Timestamp(a, b) => {
                let Some(ValueSpace::Timestamp { origin, .. }) = &domain.x_space else {
                    continue;
                };
                if extent.maximum.abs() > 9_007_199_254_740_992. || extent.maximum.fract() != 0. {
                    return Err(error(
                        DiagnosticCode::PrecisionLoss,
                        "Follow endpoint cannot preserve the source timestamp resolution.",
                    ));
                }
                let high = i128::from(*origin) + extent.maximum as i128;
                let width = (i128::from(*b) - i128::from(*a)).abs();
                let low = high - width;
                let (low, high) = (i64::try_from(low), i64::try_from(high));
                let (Ok(low), Ok(high)) = (low, high) else {
                    return Err(error(
                        DiagnosticCode::PrecisionLoss,
                        "Following this timestamp span would exceed the source integer range.",
                    ));
                };
                if *a < *b {
                    (*a, *b) = (low, high)
                } else {
                    (*a, *b) = (high, low)
                }
            }
            AxisWindow::Category { first, last } => {
                let Some(ValueSpace::Categorical { categories }) = &domain.x_space else {
                    continue;
                };
                let (Some(a), Some(b)) = (
                    categories.iter().position(|v| v == first),
                    categories.iter().position(|v| v == last),
                ) else {
                    continue;
                };
                if categories.is_empty() || extent.maximum < 0. {
                    continue;
                }
                let end = (extent.maximum.floor() as usize).min(categories.len() - 1);
                let start = end.saturating_sub(b.abs_diff(a));
                *first = categories[start].clone();
                *last = categories[end].clone();
            }
        }
    }
    state.durable.viewport.validate()?;
    super::validate_navigation_windows(&state.durable.windows)
}
fn numeric(a: f64, b: f64, high: f64) -> ChartResult<(f64, f64)> {
    let low = high - (b - a).abs();
    if !low.is_finite() || low == high {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Following this numeric span cannot preserve distinct finite endpoints.",
        ));
    }
    Ok(if a < b { (low, high) } else { (high, low) })
}
