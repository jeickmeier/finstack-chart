//! Reference regular minor intervals in transformed numeric coordinates.
use super::error;
use crate::{ChartResult, DiagnosticCode, interpolate::Number};

/// Subdivide adjacent major breaks once, extending the first observed spacing at
/// either tail before censoring. Input order and repeated candidates are retained.
/// Reversal follows the transformation, independently of authored break order.
pub fn ggplot_minor_breaks(
    major: &[Number],
    limits: [Number; 2],
    reverse: bool,
    budget: usize,
) -> ChartResult<Vec<Number>> {
    crate::limits::require_within(major.len() <= 4096 && budget <= 4096, "minor break input")?;
    let [lo, hi] = limits.map(|v| v.0);
    if lo.is_nan() || hi.is_nan() {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Minor-break limits must be comparable.",
        ));
    }
    if super::ggplot::zero_range(lo, hi) {
        return Ok(vec![]);
    }
    let mut major = major
        .iter()
        .copied()
        .filter(|v| v.0.is_finite())
        .collect::<Vec<_>>();
    if major.len() < 2 {
        return Ok(vec![]);
    }
    let spacing = major[1].0 - major[0].0;
    let minimum = major.iter().map(|v| v.0).fold(f64::INFINITY, f64::min);
    let maximum = major.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max);
    if if reverse {
        hi.max(lo) > maximum
    } else {
        lo.min(hi) < minimum
    } {
        major.insert(0, Number(major[0].0 - spacing));
    }
    let minimum = major.iter().map(|v| v.0).fold(f64::INFINITY, f64::min);
    let maximum = major.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max);
    if if reverse {
        lo.min(hi) < minimum
    } else {
        hi.max(lo) > maximum
    } {
        major.push(Number(major.last().unwrap().0 + spacing));
    }
    if major.iter().any(|v| !v.0.is_finite()) {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Minor-break extension exceeds finite precision.",
        ));
    }
    let mut result = Vec::new();
    let mut include = |v: f64| -> ChartResult<()> {
        if v >= lo.min(hi) && v <= lo.max(hi) {
            crate::limits::require_within(result.len() < budget, "minor break output")?;
            result.push(Number(v));
        }
        Ok(())
    };
    for pair in major.windows(2) {
        include(pair[0].0)?;
        include(pair[0].0 + (pair[1].0 - pair[0].0) / 2.)?;
    }
    include(major.last().unwrap().0)?;
    Ok(result)
}
