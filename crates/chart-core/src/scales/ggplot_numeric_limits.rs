//! Numeric function limits preserve reference endpoint order and vector arity.
use super::*;
use crate::{grammar::ExtensionRegistry, interpolate::Number};

pub(super) fn coordinates(spec: &MappedScaleSpec) -> (NumericFamily, bool) {
    match &spec.function {
        ScaleFunctionSpec::GgplotNumericIdentity(identity) => match identity.transform {
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
        },
        ScaleFunctionSpec::Interpolated(s) => match s.normalization {
            NormalizationSpec::Ggplot {
                family, reverse, ..
            } => (family, reverse),
            NormalizationSpec::Sequential { family, .. } => (family, false),
            _ => (NumericFamily::Linear, false),
        },
        ScaleFunctionSpec::Continuous(s) => (s.family, false),
        _ => (NumericFamily::Linear, false),
    }
}

pub(super) fn train(
    source: &MappedScaleSpec,
    values: &[Option<Number>],
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
        identity.train(values)?;
    }
    let mut next = base;
    let (family, reverse) = coordinates(&next);
    let trained = !values.is_empty()
        && !matches!(
            &next.function, ScaleFunctionSpec::GgplotNumericIdentity(s) if !s.guide
        );
    let result = evaluate(call, values, family, reverse, temporal, trained, registry)?;
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
            p.empty_population = false;
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
    next.limits_function = source.limits_function.clone();
    next.resolved_numeric_limits = Some(Box::new(result));
    Ok(next)
}

/// Resolve one population stage. Callers retain the returned order and arity;
/// finite ordered bounds would lose reference singleton and reversed semantics.
pub(super) fn evaluate(
    call: &crate::grammar::ScaleLimitsOperation,
    values: &[Option<Number>],
    family: NumericFamily,
    reverse: bool,
    temporal: Option<GgplotTimestampNormalization>,
    trained: bool,
    registry: &ExtensionRegistry,
) -> ChartResult<Vec<Number>> {
    use super::ggplot_continuous_guide::{forward, inverse};
    let mut extent = [f64::INFINITY, f64::NEG_INFINITY];
    for x in values
        .iter()
        .flatten()
        .map(|v| forward(family, reverse, v.0))
        .filter(|v| v.is_finite())
    {
        extent[0] = extent[0].min(x);
        extent[1] = extent[1].max(x);
    }
    let domain = extent.map(|v| ScaleKey::Number(Number(inverse(family, reverse, v))));
    if !trained && (reverse || temporal.is_some()) {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "This limit function cannot inverse-transform an untrained NULL domain.",
        ));
    }
    let input_domain = if trained {
        Some(domain.as_slice())
    } else if family == NumericFamily::Linear {
        None
    } else {
        Some(&[][..])
    };
    let result = registry
        .limits_function
        .evaluate(call, input_domain, temporal)?;
    result
        .unwrap_or_default()
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
            .map_or(f64::NAN, |v| forward(family, reverse, v.0)),
        values
            .get(1)
            .map_or(f64::NAN, |v| forward(family, reverse, v.0)),
    ];
    let transformed = input.map(|v| forward(family, reverse, v));
    oob.apply(transformed, bounds)
        .map(|v| inverse(family, reverse, v))
}

pub(super) fn maximum(spec: &MappedScaleSpec) -> bool {
    matches!(&spec.function, ScaleFunctionSpec::Interpolated(s) if matches!(s.normalization, NormalizationSpec::Ggplot { rescaler: GgplotRescaler::Maximum, .. }))
}
