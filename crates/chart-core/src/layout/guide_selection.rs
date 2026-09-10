//! Guide policy over shared scale ticks and formatters; never trains or replaces a scale.
use super::{ResolvedAxis, ResolvedScale};
use crate::{
    ChartResult, DiagnosticCode,
    composition::ScaleValue,
    scales::*,
    typography::{NumericFormat, NumericLocale},
};

fn unsupported(message: &str) -> crate::Diagnostic {
    crate::scales::error(DiagnosticCode::UnsupportedCapability, message)
}

/// Recover the retained numeric viewport, without reconstructing a mapping.
fn numeric(scale: &ResolvedScale) -> Option<(NumericFamily, Bounds)> {
    match scale {
        ResolvedScale::Linear(s) => Some((NumericFamily::Linear, s.viewport())),
        ResolvedScale::Numeric(s) => Some((s.family(), s.viewport())),
        ResolvedScale::Nonlinear(s) => Some((
            match s.transform() {
                ScaleTransform::Log { base } => NumericFamily::Log { base },
                ScaleTransform::Symlog { threshold } => NumericFamily::Symlog {
                    constant: threshold,
                },
            },
            s.viewport(),
        )),
        _ => None,
    }
}

/// D3 count arguments are density hints; the collection bound is always independent.
pub(super) fn values(
    axis: &ResolvedAxis,
    arguments: &GuideTickArguments,
    budget: usize,
) -> ChartResult<Vec<ScaleValue>> {
    let count = arguments.count.unwrap_or(10.);
    if let Some((family, domain)) = numeric(&axis.scale) {
        if arguments.interval.is_some() {
            return Err(unsupported("Calendar intervals require a time guide."));
        }
        return Ok(family
            .ticks(&[domain.start().into(), domain.end().into()], count, budget)?
            .into_iter()
            .map(ScaleValue::Number)
            .collect());
    }
    let selection = arguments
        .interval
        .map_or(CalendarTicks::Count(count.into()), CalendarTicks::Interval);
    let timestamp = |values: Vec<i64>, unit| {
        values
            .into_iter()
            .map(|value| ScaleValue::Timestamp { value, unit })
            .collect()
    };
    let categories = |values: &[String]| -> ChartResult<Vec<ScaleValue>> {
        crate::limits::require_within(values.len() <= budget, "guide domain tick")?;
        Ok(values.iter().cloned().map(ScaleValue::Category).collect())
    };
    match &axis.scale {
        ResolvedScale::Provider(s) => s.ticks(arguments, budget),
        ResolvedScale::Band(s) => categories(s.visible_domain()),
        ResolvedScale::Point(s) => categories(s.visible_domain()),
        ResolvedScale::Calendar(s) => Ok(timestamp(s.ticks(selection, budget)?, s.unit())),
        ResolvedScale::Utc(s) => Ok(timestamp(
            Calendar::new(CalendarZone::Utc)?.ticks(s.viewport(), s.unit(), selection, budget)?,
            s.unit(),
        )),
        _ => Err(unsupported(
            "This scale has no D3 automatic tick policy; supply explicit semantic values.",
        )),
    }
}

/// Format original values with shared family precision and explicit locale/calendar resources.
/// This function never generates candidate values, including for an empty explicit list.
pub(super) fn labels(
    axis: &ResolvedAxis,
    values: &[ScaleValue],
    arguments: &GuideTickArguments,
    numeric_format: Option<&NumericFormat>,
    time_format: Option<&TimeFormat>,
    budget: usize,
) -> ChartResult<Vec<String>> {
    if let Some((family, domain)) = numeric(&axis.scale) {
        if time_format.is_some() || arguments.interval.is_some() {
            return Err(unsupported(
                "Time formatting/intervals require a time guide.",
            ));
        }
        let formatter = family.tick_format(
            &[domain.start().into(), domain.end().into()],
            arguments.count.unwrap_or(10.),
            numeric_format
                .map(|f| f.specifier.as_str())
                .or(arguments.specifier.as_deref()),
            numeric_format.map_or_else(NumericLocale::default, |f| f.locale.clone()),
        )?;
        return values
            .iter()
            .map(|value| match value {
                ScaleValue::Number(n) if n.is_finite() => Ok(formatter.format(*n)),
                _ => Err(unsupported(
                    "Numeric tick formatting requires finite numeric values.",
                )),
            })
            .collect();
    }
    match &axis.scale {
        ResolvedScale::Provider(s) if numeric_format.is_none() && time_format.is_none() => {
            return s.labels(values, arguments, budget);
        }
        ResolvedScale::Calendar(_) | ResolvedScale::Utc(_) => {
            if numeric_format.is_some() {
                return Err(unsupported("Numeric formatting requires a numeric guide."));
            }
            // D3's time scale ignores the numeric formatter argument. A supplied time
            // descriptor is the independent override and uses this scale's calendar.
            let calendar = match &axis.scale {
                ResolvedScale::Calendar(s) => s.calendar().clone(),
                _ => Calendar::new(CalendarZone::Utc)?,
            };
            let formatter = time_format.cloned().unwrap_or_default().prepare(calendar)?;
            return values
                .iter()
                .map(|value| match value {
                    ScaleValue::Timestamp { value, unit } => formatter.format(*value, *unit),
                    _ => Err(unsupported(
                        "Time tick formatting requires exact timestamps.",
                    )),
                })
                .collect();
        }
        _ => {}
    }
    if numeric_format.is_some() || time_format.is_some() {
        return Err(unsupported(
            "This guide cannot apply the requested built-in formatter.",
        ));
    }
    Ok(values
        .iter()
        .map(|value| match value {
            ScaleValue::Number(n) => crate::number::ecmascript(*n),
            ScaleValue::Category(s) => s.clone(),
            ScaleValue::Timestamp { value, .. } => value.to_string(),
        })
        .collect())
}
