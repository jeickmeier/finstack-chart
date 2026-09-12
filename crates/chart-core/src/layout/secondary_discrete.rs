//! Discrete duplicate guides reuse primary spacing and expose numeric category coordinates.
use super::*;
use crate::scales::error;
use crate::{ChartResult, DiagnosticCode, composition::ScaleValue, grammar::PreparedChart};

pub(super) fn viewport(primary: &ResolvedAxis) -> ChartResult<crate::scales::Bounds> {
    let view = match &primary.scale {
        ResolvedScale::Band(s) => s.reference_viewport(),
        ResolvedScale::Point(s) => s.reference_viewport(),
        ResolvedScale::Provider(s) => s.category_viewport(),
        _ => unreachable!(),
    }
    .ok_or_else(|| {
        error(
            DiagnosticCode::UnsupportedCapability,
            "Discrete secondary axes require reference category spacing.",
        )
    })?;
    crate::scales::Bounds::new(view[0].0, view[1].0)?.distinct()
}

fn coordinate(primary: &ResolvedAxis, value: &ScaleValue) -> ChartResult<Option<f64>> {
    let domain = match &primary.scale {
        ResolvedScale::Band(s) => s.domain(),
        ResolvedScale::Point(s) => s.domain(),
        ResolvedScale::Provider(s) => return s.category_coordinate(value),
        _ => unreachable!(),
    };
    let ScaleValue::Category(value) = value else {
        return Ok(None);
    };
    Ok(domain
        .iter()
        .position(|key| key == value)
        .map(|i| 1. + i as f64))
}

pub(super) fn position(primary: &ResolvedAxis, value: f64) -> ChartResult<Option<f64>> {
    match &primary.scale {
        ResolvedScale::Band(s) => s.reference_minor(value),
        ResolvedScale::Point(s) => s.reference_minor(value),
        ResolvedScale::Provider(s) => s.category_minor(value),
        _ => unreachable!("validated discrete secondary source"),
    }
}

pub(super) fn resolve(
    chart: &PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    request: &LayoutRequest,
) -> ChartResult<Vec<GuideTick>> {
    if style != &axis.spec.guide {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Additional guides must reference a primary positional scale.",
        ));
    }
    super::axes::validate_style(style, request.limits)?;
    let ResolvedScale::SecondaryDiscrete { primary, .. } = &axis.scale else {
        unreachable!()
    };
    let inherited = style.tick_values.is_none() && style.guide_ticks.is_none();
    let values: Vec<_> = if let Some(values) = &style.tick_values {
        values.clone()
    } else if let Some(ticks) = &style.guide_ticks {
        ticks.iter().map(|t| t.value.clone()).collect()
    } else if let Some(values) = &primary.spec.guide.tick_values {
        values.clone()
    } else if let Some(ticks) = &primary.spec.guide.guide_ticks {
        ticks.iter().map(|tick| tick.value.clone()).collect()
    } else {
        super::guide_selection::values(
            primary,
            &primary
                .spec
                .guide
                .tick_arguments
                .clone()
                .unwrap_or_default(),
            request.max_ticks,
            true,
        )?
    };
    crate::limits::require_within(values.len() <= request.max_ticks, "discrete secondary tick")?;
    let primary_breaks_authored = primary
        .spec
        .discrete
        .as_ref()
        .is_some_and(|p| p.guide.breaks.is_some())
        || primary.spec.guide.tick_values.is_some();
    let primary_labels_automatic = primary.spec.discrete.as_ref().is_none_or(|p| {
        matches!(p.guide.labels, crate::scales::GgplotGuideLabels::Automatic)
            && p.guide.break_names.is_none()
    }) && primary.spec.guide.tick_format.is_none()
        && primary.spec.guide.guide_ticks.is_none();
    let formatter_hint = style
        .tick_arguments
        .as_ref()
        .is_some_and(|args| args.specifier.is_some());
    let authored_labels = if let Some(ticks) = &style.guide_ticks {
        Some(ticks.iter().map(|t| t.label.clone()).collect::<Vec<_>>())
    } else if let Some(GuideFormatter::Labels(labels)) = &style.tick_format {
        if labels.len() != values.len() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Explicit guide labels must match the complete selected value list.",
            ));
        }
        Some(labels.clone())
    } else if inherited
        && !formatter_hint
        && !(primary_breaks_authored && primary_labels_automatic)
        && style.tick_format.is_none()
        && style.number_format.is_none()
        && style.numeric_format.is_none()
        && style.time_format.is_none()
    {
        Some(
            values
                .iter()
                .map(|value| {
                    primary
                        .ticks
                        .iter()
                        .find(|tick| &tick.value == value)
                        .map_or_else(String::new, |tick| tick.label.clone())
                })
                .collect(),
        )
    } else {
        None
    };
    let mut selected = Vec::new();
    let mut labels = Vec::new();
    for (i, value) in values.iter().enumerate() {
        let value = match value {
            ScaleValue::Number(value) => Some(*value),
            ScaleValue::Category(_) | ScaleValue::MissingCategory => coordinate(primary, value)?,
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Discrete secondary breaks require category keys or numeric indices.",
                ));
            }
        };
        if let Some(value) = value
            && position(primary, value)?.is_some()
        {
            selected.push(ScaleValue::Number(value));
            if let Some(authored) = &authored_labels {
                labels.push(authored[i].clone());
            }
        }
    }
    let mut normalized = style.clone();
    normalized.guide_ticks = None;
    if authored_labels.is_some() {
        normalized.tick_format = Some(GuideFormatter::Labels(labels));
    } else if !formatter_hint
        && normalized.tick_format.is_none()
        && normalized.number_format.is_none()
        && normalized.numeric_format.is_none()
        && normalized.time_format.is_none()
    {
        let numeric = selected
            .iter()
            .map(|v| match v {
                ScaleValue::Number(v) => *v,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        normalized.tick_format = Some(GuideFormatter::Labels(
            crate::typography::ggplot_numeric_labels(&numeric, request.limits.max_text_bytes)?,
        ));
    }
    normalized.tick_values = Some(selected);
    super::guide_ticks::configured(chart, axis, &normalized, request)
}
