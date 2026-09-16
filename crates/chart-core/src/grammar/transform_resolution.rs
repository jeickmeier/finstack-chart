//! Resolve descriptor selections before semantic training or source evaluation.
use super::transform_extensions::TransformRegistrations;
use super::*;
use crate::ChartResult;
use crate::{layout::AxisScale, scales::ScaleTransform};
use std::borrow::Cow;

pub(crate) fn resolve<'a>(
    definition: &'a ChartDefinition,
    registry: &ExtensionRegistry,
    portable: bool,
) -> ChartResult<Cow<'a, ChartDefinition>> {
    if !definition.any_ggplot_transform(crate::scales::GgplotTransform::has_registered) {
        return Ok(Cow::Borrowed(definition));
    }
    let validation_error = std::cell::RefCell::new(None);
    definition.any_ggplot_transform(|transform| {
        if let Err(error) = transform.validate_authoring() {
            *validation_error.borrow_mut() = Some(error);
            true
        } else {
            false
        }
    });
    if let Some(error) = validation_error.into_inner() {
        return Err(error);
    }
    let mut result = definition.clone();
    let registrations = &registry.transforms_function;
    for scale in super::interpolation_extensions::mapped_scales_mut(&mut result) {
        if let Some(t) = scale.ggplot_transform_mut() {
            t.resolve_registrations(registrations, portable)?;
        }
    }
    for axis in &mut result.axes {
        let transform = match &mut axis.scale {
            AxisScale::Nonlinear { transform, .. } => transform.ggplot_transform_mut(),
            AxisScale::Binned { spec, prepared } => {
                if let Some(p) = prepared {
                    std::sync::Arc::make_mut(p).resolve_transform(registrations, portable)?;
                }
                spec.transform
                    .as_mut()
                    .and_then(ScaleTransform::ggplot_transform_mut)
            }
            AxisScale::Numeric(s) => s.family.ggplot_transform_mut(),
            AxisScale::Secondary {
                transform: Some(s), ..
            } => s.family.ggplot_transform_mut(),
            _ => None,
        };
        if let Some(t) = transform {
            t.resolve_registrations(registrations, portable)?;
        }
    }
    source(&mut result.mappings, registrations, portable)?;
    for t in &mut result.transforms {
        statistic(&mut t.statistic, registrations, portable)?;
        if let Some(g) = &mut t.grammar {
            source(&mut g.source, registrations, portable)?;
        }
        for f in &mut t.filters {
            numeric(&mut f.value, registrations, portable)?;
        }
    }
    for l in &mut result.layers {
        if let Mappings::Source(a) = &mut l.mappings {
            source(a, registrations, portable)?;
        }
        if let Some(g) = &mut l.grammar {
            source(&mut g.source, registrations, portable)?;
        }
        statistic(&mut l.statistic, registrations, portable)?;
        for f in &mut l.filters {
            numeric(&mut f.value, registrations, portable)?;
        }
        for c in l.color.iter_mut().chain(l.paint_scales.values_mut()) {
            color(&mut c.input, registrations, portable)?;
        }
        if let Some(s) = &mut l.symbol {
            color(&mut s.input, registrations, portable)?;
        }
        for s in l
            .numeric_scales
            .values_mut()
            .chain(l.value_scales.values_mut())
        {
            color(&mut s.input, registrations, portable)?;
        }
    }
    Ok(Cow::Owned(result))
}
fn color(input: &mut ColorInput, r: &TransformRegistrations, p: bool) -> ChartResult<()> {
    if let ColorInput::Numeric(n) = input {
        numeric(n, r, p)?;
    }
    Ok(())
}
fn source(a: &mut SourceAes, r: &TransformRegistrations, p: bool) -> ChartResult<()> {
    for n in [
        &mut a.x,
        &mut a.y,
        &mut a.x2,
        &mut a.y2,
        &mut a.low,
        &mut a.high,
        &mut a.size,
    ]
    .into_iter()
    .flatten()
    {
        numeric(n, r, p)?;
    }
    Ok(())
}
fn statistic(s: &mut Statistic, r: &TransformRegistrations, p: bool) -> ChartResult<()> {
    match &mut s.parameters {
        StatParameters::Distribution(s) => {
            for n in s.numerics_mut() {
                numeric(n, r, p)?;
            }
        }
        StatParameters::Univariate(s) => {
            for n in s.numerics_mut() {
                numeric(n, r, p)?;
            }
        }
        StatParameters::Bin(s) => numeric(&mut s.input, r, p)?,
        StatParameters::AutoBin(s) => numeric(&mut s.input, r, p)?,
        StatParameters::Summary(s) => numeric(&mut s.input, r, p)?,
        StatParameters::Ols(s) => {
            numeric(&mut s.x, r, p)?;
            numeric(&mut s.y, r, p)?;
        }
        StatParameters::Count(s) => {
            for n in &mut s.required {
                numeric(n, r, p)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn numeric(mut n: &mut Numeric, r: &TransformRegistrations, p: bool) -> ChartResult<()> {
    let mut depth = 0;
    while let Numeric::Scaled { input, scale, .. } = n {
        depth += 1;
        crate::limits::require_within(
            depth <= crate::interpolate::MAX_VALUE_DEPTH,
            "numeric transform depth",
        )?;
        if let Some(t) = scale
            .transform
            .as_mut()
            .and_then(ScaleTransform::ggplot_transform_mut)
        {
            t.resolve_registrations(r, p)?;
        }
        if let Some(b) = &mut scale.binned {
            std::sync::Arc::make_mut(b).resolve_transform(r, p)?;
        }
        n = input;
    }
    Ok(())
}
