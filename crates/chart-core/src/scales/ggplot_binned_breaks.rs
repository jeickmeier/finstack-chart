//! Resolve function cuts once for both bin mapping and guide selection.
use super::*;
use crate::{grammar::ExtensionRegistry, interpolate::Number};

pub(super) fn train(
    source: &MappedScaleSpec,
    values: &[Option<Number>],
    registry: &ExtensionRegistry,
) -> ChartResult<MappedScaleSpec> {
    train_batches(source, values, &[values.len()], registry)
}

pub(super) fn train_batches(
    source: &MappedScaleSpec,
    values: &[Option<Number>],
    batches: &[usize],
    registry: &ExtensionRegistry,
) -> ChartResult<MappedScaleSpec> {
    use super::ggplot_continuous_guide::forward;
    source.validate_training()?;
    let call = source.breaks_function.as_ref().expect("selected function");
    registry.breaks_function.validate(call, false)?;
    let Some(GgplotScalePolicy::Binned(authored)) = source.ggplot.as_deref() else {
        unreachable!("selected binned policy")
    };
    let count = match authored.breaks {
        GgplotBreaks::Equal(n) | GgplotBreaks::Nice(n) => n,
        GgplotBreaks::Explicit(_) => unreachable!("validated function breaks"),
    };
    // Suppress automatic cuts and their domain extension while training limits.
    let mut base = source.clone();
    base.breaks_function = None;
    if let Some(GgplotScalePolicy::Binned(p)) = base.ggplot.as_deref_mut() {
        p.breaks = GgplotBreaks::Explicit(vec![]);
        p.prepared_breaks = None;
        p.prepared_break_names = None;
    }
    let mut next = if base.limits_function.is_some() {
        super::ggplot_numeric_limits::train_batches(&base, values, batches, registry)?
    } else {
        let policy = base.ggplot.clone().expect("binned policy");
        policy.train_number_batches(&mut base, values, batches)?;
        base
    };
    let ScaleFunctionSpec::Interpolated(s) = &next.function else {
        unreachable!("validated binned function")
    };
    let NormalizationSpec::Ggplot {
        ref family,
        reverse,
        domain,
        ..
    } = s.normalization
    else {
        unreachable!("validated binned normalization")
    };
    let Some(GgplotScalePolicy::Binned(p)) = next.ggplot.as_deref_mut() else {
        unreachable!("trained binned policy")
    };
    if p.empty_population && p.limits.is_none() {
        return Ok(next);
    }
    let bounds = next
        .trained_transformed_bounds
        .map(|v| v.map(|v| v.0))
        .or_else(|| p.population_bounds(family.clone(), reverse))
        .unwrap_or_else(|| domain.map(|v| forward(family, reverse, v.0)));
    let selected = select(
        call,
        &registry.breaks_function,
        Some(bounds),
        family.clone(),
        reverse,
        count,
        4096,
    )?;
    let cuts = selected.values;
    p.breaks = GgplotBreaks::Explicit(cuts.clone());
    p.prepared_breaks = Some(cuts);
    p.prepared_break_names = selected.names;
    Ok(next)
}

pub(super) struct SelectedBinnedBreaks {
    pub is_null: bool,
    pub values: Vec<Number>,
    pub names: Option<Vec<String>>,
}
/// One shared inverse/count/typed-result contract for mapping and positional views.
pub(super) fn select(
    call: &crate::grammar::ScaleBreaksOperation,
    registry: &crate::grammar::scale_break_extensions::ScaleBreakRegistrations,
    bounds: Option<[f64; 2]>,
    family: NumericFamily,
    reverse: bool,
    count: f64,
    budget: usize,
) -> ChartResult<SelectedBinnedBreaks> {
    let mut limits = super::ggplot_continuous_guide::inverse_values(
        &family,
        reverse,
        &bounds.unwrap_or([f64::NAN; 2]),
    )?
    .into_iter()
    .filter(|v| !v.is_nan())
    .collect::<Vec<_>>();
    limits.sort_by(f64::total_cmp);
    let domain = limits
        .into_iter()
        .map(|v| ScaleKey::Number(Number(v)))
        .collect::<Vec<_>>();
    let output = registry.evaluate_binned(call, bounds.map(|_| domain.as_slice()), count)?;
    let is_null = if output.values.is_none() {
        super::ggplot_continuous_guide::forward_null(&family, reverse)?.is_none()
    } else {
        false
    };
    let cuts = output
        .values
        .unwrap_or_default()
        .into_iter()
        .map(|v| match v {
            ScaleKey::Number(n) => Ok(n),
            ScaleKey::Null => Ok(Number(f64::NAN)),
            _ => Err(error(
                DiagnosticCode::Validation,
                "Binned break functions must return numeric values.",
            )),
        })
        .collect::<ChartResult<Vec<_>>>()?;
    crate::limits::require_within(cuts.len() <= budget, "binned break function")?;
    Ok(SelectedBinnedBreaks {
        is_null,
        values: cuts,
        names: output.names,
    })
}
