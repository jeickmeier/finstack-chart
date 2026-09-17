//! Bounded shared kernels for two-dimensional statistics.
mod levels;
pub(crate) use levels::contour_levels;
mod contour;
mod isoband;
pub(crate) use contour::isolines;
pub(crate) use isoband::isobands;
mod density;
mod ellipse;
mod hexagonal;
pub(crate) use hexagonal::hexagonal;
mod rectangular;
pub(crate) use density::density;
pub(crate) use ellipse::ellipse;
pub(crate) use rectangular::rectangular;
fn invalid(message: &str) -> crate::Diagnostic {
    super::error(crate::DiagnosticCode::NumericalDomain, message)
}

fn response_summary(
    function: super::SummaryFunction,
    values: &[f64],
) -> crate::ChartResult<Option<f64>> {
    if values.iter().any(|v| !v.is_finite()) {
        return Ok(None);
    }
    if values.is_empty() && function == super::SummaryFunction::Sum {
        return Ok(Some(0.));
    }
    Ok(super::summary_values(
        &super::SummaryHelper::Functions {
            center: function,
            lower: function,
            upper: function,
        },
        values,
    )?[0])
}

pub(crate) mod grid;

pub(crate) mod labels;
