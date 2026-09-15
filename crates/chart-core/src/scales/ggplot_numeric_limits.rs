//! Numeric function limits preserve reference endpoint order and vector arity.
pub(crate) use super::ggplot_continuous_guide::forward_values;
use super::*;
use crate::{grammar::ExtensionRegistry, interpolate::Number};

pub(super) fn coordinates(spec: &MappedScaleSpec) -> (NumericFamily, bool) {
    match &spec.function {
        ScaleFunctionSpec::GgplotNumericIdentity(identity) => {
            positional_coordinates(identity.transform.clone())
        }
        ScaleFunctionSpec::Interpolated(s) => match &s.normalization {
            NormalizationSpec::Ggplot {
                family, reverse, ..
            } => (family.clone(), *reverse),
            NormalizationSpec::Sequential { family, .. } => (family.clone(), false),
            _ => (NumericFamily::Linear, false),
        },
        ScaleFunctionSpec::Continuous(s) => (s.family.clone(), false),
        _ => (NumericFamily::Linear, false),
    }
}

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
    source.validate_training()?;
    if source.categorical() {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Numeric limit training requires a numeric population.",
        ));
    }
    let temporal = match &source.function {
        ScaleFunctionSpec::Interpolated(s) => match s.normalization {
            NormalizationSpec::Ggplot { timestamp, .. } => timestamp,
            _ => None,
        },
        _ => None,
    };
    let call = source.limits_function.as_ref().expect("selected function");
    let mut base = source.clone();
    base.limits_function = None;
    base.resolved_numeric_limits = None;
    match base.ggplot.as_deref_mut() {
        Some(GgplotScalePolicy::Continuous { limits, .. }) => *limits = None,
        // A function counts as authored limits, suppressing automatic bin extension.
        Some(GgplotScalePolicy::Binned(p)) => {
            p.limits = Some([None, None]);
            p.prepared_breaks = None;
        }
        _ => {}
    }
    if let ScaleFunctionSpec::GgplotNumericIdentity(identity) = &mut base.function {
        identity.limits = None;
    }
    if let Some(policy) = base.ggplot.clone() {
        policy.validate_numbers(&mut base)?;
    }
    if let ScaleFunctionSpec::GgplotNumericIdentity(identity) = &mut base.function {
        identity.train_batches(values, batches)?;
    }
    let mut next = base;
    let (family, reverse) = coordinates(&next);
    let trained = !values.is_empty()
        && !matches!(
            &next.function, ScaleFunctionSpec::GgplotNumericIdentity(s) if !s.guide
        );
    let result = if let ScaleFunctionSpec::GgplotNumericIdentity(identity) = &next.function {
        evaluate_transformed(
            call,
            identity.trained.into_iter().flatten().map(|v| v.0),
            family.clone(),
            reverse,
            temporal,
            trained,
            registry,
        )?
    } else {
        let raw = values
            .iter()
            .map(|v| v.map_or(f64::NAN, |v| v.0))
            .collect::<Vec<_>>();
        let transformed =
            super::ggplot_continuous_guide::forward_batches(&family, reverse, &raw, batches)?;
        evaluate_transformed(
            call,
            transformed.into_iter(),
            family.clone(),
            reverse,
            temporal,
            trained,
            registry,
        )?
    };
    // Keep arbitrary arity until sampling/guide evaluation, so identity output can
    // remain valid independently of an invalid guide limit vector.
    let mut pair = [
        result.first().copied().unwrap_or(Number(f64::NAN)),
        result
            .get(1)
            .or_else(|| result.first())
            .copied()
            .unwrap_or(Number(f64::NAN)),
    ];
    if result.len() == 1 && maximum(&next) {
        pair[1] = Number(f64::NAN);
    }
    match next.ggplot.as_deref_mut() {
        Some(GgplotScalePolicy::Continuous {
            limits,
            empty_population,
            nonfinite_population,
            ..
        }) => {
            *limits = Some(pair.map(Some));
            *empty_population = false;
            *nonfinite_population = false;
        }
        Some(GgplotScalePolicy::Binned(p)) => {
            p.limits = Some(pair.map(Some));
            p.empty_population = values.is_empty();
            p.nonfinite_population = false;
            p.prepared_breaks = None;
        }
        _ => {}
    }
    match &mut next.function {
        ScaleFunctionSpec::Interpolated(s) => match &mut s.normalization {
            NormalizationSpec::Ggplot { domain, .. }
            | NormalizationSpec::Sequential { domain, .. } => *domain = pair,
            _ => {}
        },
        ScaleFunctionSpec::Continuous(s) => s.domain = pair.to_vec(),
        ScaleFunctionSpec::GgplotNumericIdentity(s) => s.limits = Some(pair.map(Some)),
        _ => {}
    }
    if !matches!(next.function, ScaleFunctionSpec::GgplotNumericIdentity(_))
        && family.ggplot_transform().is_some_and(|t| !t.is_pointwise())
    {
        next.trained_transformed_bounds = if result.len() == 2 {
            let transformed = forward_values(
                &family,
                reverse,
                &result.iter().map(|v| v.0).collect::<Vec<_>>(),
            )?;
            Some([Number(transformed[0]), Number(transformed[1])])
        } else {
            None
        };
    }
    next.limits_function = source.limits_function.clone();
    next.resolved_numeric_limits = Some(Box::new(result));
    Ok(next)
}

/// Evaluate a callback against already transformed post-statistic observations.
/// Avoid an inverse/forward round trip before finding the training extent.
pub(crate) fn evaluate_transformed(
    call: &crate::grammar::ScaleLimitsOperation,
    values: impl Iterator<Item = f64>,
    family: NumericFamily,
    reverse: bool,
    temporal: Option<GgplotTimestampNormalization>,
    trained: bool,
    registry: &ExtensionRegistry,
) -> ChartResult<Vec<Number>> {
    evaluate_transformed_optional(call, values, family, reverse, temporal, trained, registry)
        .map(Option::unwrap_or_default)
}

pub(crate) fn evaluate_transformed_optional(
    call: &crate::grammar::ScaleLimitsOperation,
    values: impl Iterator<Item = f64>,
    family: NumericFamily,
    reverse: bool,
    temporal: Option<GgplotTimestampNormalization>,
    trained: bool,
    registry: &ExtensionRegistry,
) -> ChartResult<Option<Vec<Number>>> {
    use super::ggplot_continuous_guide::inverse_values;
    let mut extent = [f64::INFINITY, f64::NEG_INFINITY];
    for x in values.filter(|v| v.is_finite()) {
        extent[0] = extent[0].min(x);
        extent[1] = extent[1].max(x);
    }
    let domain = inverse_values(&family, reverse, &extent)?
        .into_iter()
        .map(|v| ScaleKey::Number(Number(v)))
        .collect::<Vec<_>>();
    let needs_domain = registry.limits_function.requires_domain(call)?;
    if !trained && (reverse || temporal.is_some()) && needs_domain {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "This limit function cannot inverse-transform an untrained NULL domain.",
        ));
    }
    let empty_domain = if !trained && needs_domain {
        family
            .ggplot_transform()
            .map(GgplotTransform::inverse_null)
            .transpose()?
            .unwrap_or_else(|| (family != NumericFamily::Linear).then(Vec::new))
    } else {
        Some(vec![])
    };
    let empty_domain = empty_domain.map(|values| {
        values
            .into_iter()
            .map(|v| ScaleKey::Number(Number(v)))
            .collect::<Vec<_>>()
    });
    let input_domain = if trained {
        Some(domain.as_slice())
    } else {
        empty_domain.as_deref()
    };
    let result = registry
        .limits_function
        .evaluate(call, input_domain, temporal)?;
    if result.is_none() {
        super::ggplot_continuous_guide::forward_null(&family, reverse)?;
    }
    result
        .map(|values| {
            values
                .iter()
                .map(|key| match GgplotDiscreteIdentity::map(Some(key))? {
                    crate::interpolate::Value::Number(v) => Ok(v),
                    crate::interpolate::Value::Missing => Ok(Number(f64::NAN)),
                    _ => Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Numeric limit functions must return numbers or missing values.",
                    )),
                })
                .collect::<ChartResult<Vec<_>>>()
        })
        .transpose()
}

pub(super) fn validate_result(values: &[Number]) -> ChartResult<()> {
    if !(1..=2).contains(&values.len()) || values.len() == 2 && values.iter().any(|v| v.0.is_nan())
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Numeric function limits require one or two comparable values for mapping and guides.",
        ));
    }
    Ok(())
}

pub(super) fn input(spec: &MappedScaleSpec, values: &[Number], input: Option<f64>) -> Option<f64> {
    let (family, reverse) = coordinates(spec);
    use super::ggplot_continuous_guide::{forward, inverse};
    let oob = match spec.ggplot.as_deref() {
        Some(GgplotScalePolicy::Continuous { oob, .. }) => *oob,
        Some(GgplotScalePolicy::Binned(p)) => p.oob,
        _ => return input,
    };
    let bounds = [
        values
            .first()
            .map_or(f64::NAN, |v| forward(&family, reverse, v.0)),
        values
            .get(1)
            .map_or(f64::NAN, |v| forward(&family, reverse, v.0)),
    ];
    let transformed = input.map(|v| forward(&family, reverse, v));
    oob.apply(transformed, bounds)
        .map(|v| inverse(&family, reverse, v))
}

pub(super) fn maximum(spec: &MappedScaleSpec) -> bool {
    matches!(&spec.function, ScaleFunctionSpec::Interpolated(s) if matches!(s.normalization, NormalizationSpec::Ggplot { rescaler: GgplotRescaler::Maximum, .. }))
}

/// Shared family identity for numeric positional and identity policies.
pub(crate) fn positional_coordinates(transform: Option<ScaleTransform>) -> (NumericFamily, bool) {
    match transform {
        Some(ScaleTransform::Ggplot { transform }) => (NumericFamily::Ggplot { transform }, false),
        Some(ScaleTransform::Reverse) => (NumericFamily::Linear, true),
        Some(ScaleTransform::Sqrt) => (NumericFamily::Pow { exponent: 0.5 }, false),
        Some(ScaleTransform::Log { base }) => (NumericFamily::Log { base }, false),
        Some(ScaleTransform::Symlog { threshold }) => (
            NumericFamily::Symlog {
                constant: threshold,
            },
            false,
        ),
        _ => (NumericFamily::Linear, false),
    }
}

/// Resolve authored numeric endpoints against an already transformed population.
/// Empty reference scales use [0, 1] unless both authored endpoints are finite.
pub(crate) fn authored_transformed(
    limits: [Option<Number>; 2],
    transform: Option<ScaleTransform>,
    values: impl Iterator<Item = f64>,
    trained: bool,
) -> ChartResult<[Number; 2]> {
    let (family, reverse) = positional_coordinates(transform);
    let mut extent = [f64::INFINITY, f64::NEG_INFINITY];
    for v in values.filter(|v| v.is_finite()) {
        extent[0] = extent[0].min(v);
        extent[1] = extent[1].max(v);
    }
    let bounds = super::ggplot_continuous_guide::authored_bounds_batch(
        Some(limits),
        &family,
        reverse,
        extent,
    )?;
    if !trained
        && !super::ggplot_continuous_guide::authored_bounds_batch(
            Some(limits),
            &family,
            reverse,
            [f64::NAN; 2],
        )?
        .iter()
        .all(|v| v.is_finite())
    {
        Ok([Number(0.), Number(1.)])
    } else {
        Ok(bounds.map(Number))
    }
}
