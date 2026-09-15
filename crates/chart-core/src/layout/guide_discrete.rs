//! Primary categorical callbacks reuse the shared discrete key and recycling policy.
use super::*;
use crate::{ChartResult, DiagnosticCode, composition::ScaleValue, scales::*};

pub(super) fn registered(
    chart: &crate::grammar::PreparedChart,
    axis: &ResolvedAxis,
    style: &GuideStyle,
    request: &LayoutRequest,
) -> ChartResult<Option<Vec<GuideTick>>> {
    let formatter = match &style.tick_format {
        Some(GuideFormatter::Registered {
            operation,
            parameters,
        }) => Some((operation, parameters)),
        _ => None,
    };
    if formatter.is_none() && style.breaks_function.is_none() {
        return Ok(None);
    }
    if chart.definition().profile() != crate::grammar::Profile::Ggplot2_4_0_3
        || style.profile != GuideProfile::LibraryV1
        || style.guide_ticks.is_some()
        || !axis.space.is_categorical()
        || !matches!(
            axis.spec.scale,
            AxisScale::Auto | AxisScale::Band(_) | AxisScale::Point(_)
        )
    {
        return Ok(None);
    }
    let ResolvedScale::Provider(provider) = &axis.scale else {
        return Ok(None);
    };
    let key = |value: &ScaleValue| match value {
        ScaleValue::Category(value) => Ok(ScaleKey::Text(value.clone())),
        ScaleValue::MissingCategory => Ok(ScaleKey::Null),
        _ => Err(crate::scales::error(
            DiagnosticCode::SchemaConflict,
            "Discrete label breaks require category keys.",
        )),
    };
    let domain = provider
        .domain()
        .iter()
        .map(key)
        .collect::<ChartResult<Vec<_>>>()?;
    let mut guide = axis
        .spec
        .discrete
        .as_deref()
        .map(|p| p.guide.clone())
        .unwrap_or_default();
    if let Some(values) = &style.tick_values {
        crate::limits::require_within(values.len() <= request.max_ticks, "selected guide tick")?;
        guide.breaks = Some(values.iter().map(key).collect::<ChartResult<Vec<_>>>()?);
        guide.break_names = None;
    }
    if let Some((operation, parameters)) = formatter {
        chart.guide_registrations.validate(
            operation,
            parameters,
            request.units == crate::services::Units::Points,
        )?;
        guide.labels = GgplotGuideLabels::Registered {
            operation: operation.clone(),
            parameters: parameters.clone(),
        };
    }
    let observed = match &axis.space {
        crate::grammar::ValueSpace::Categorical { categories } => !categories.is_empty(),
        crate::grammar::ValueSpace::NullableCategorical { categories } => !categories.is_empty(),
        _ => false,
    };
    if let Some(call) = &style.breaks_function {
        if observed || !domain.is_empty() {
            let output = chart.break_registrations.evaluate(call, &domain, None)?;
            let values = output.values.unwrap_or_default();
            crate::limits::require_within(
                values.len() <= request.max_ticks,
                "discrete positional break function",
            )?;
            guide.breaks = Some(values);
            guide.break_names = output.names;
        } else {
            guide.breaks = Some(vec![]);
            guide.break_names = None;
        }
    }
    let mut limits = request.limits;
    limits.max_items = limits.max_items.min(request.max_ticks);
    let entries = guide.resolve_using_context(
        &domain,
        &chart.guide_registrations,
        observed || !domain.is_empty(),
        limits,
    )?;
    let automatic = matches!(guide.labels, GgplotGuideLabels::Automatic);
    let mut configured = style.clone();
    configured.breaks_function = None;
    configured.tick_values = Some(
        entries
            .iter()
            .map(|e| match &e.key {
                ScaleKey::Text(value) => ScaleValue::Category(value.clone()),
                ScaleKey::Null => ScaleValue::MissingCategory,
                _ => unreachable!("validated categorical domain"),
            })
            .collect(),
    );
    if style.tick_format.is_none() || formatter.is_some() {
        configured.tick_format = Some(GuideFormatter::Labels(
            entries
                .into_iter()
                .map(|e| {
                    e.label.unwrap_or_else(|| {
                        if automatic {
                            "NA".into()
                        } else {
                            String::new()
                        }
                    })
                })
                .collect(),
        ));
    }
    super::guide_ticks::configured(chart, axis, &configured, request).map(Some)
}
