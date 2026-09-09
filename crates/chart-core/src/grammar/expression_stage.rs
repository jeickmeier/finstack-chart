//! Source expressions use the selected registered dataset before chart filters and facets.
use super::*;
use crate::{
    ChartResult,
    data::{DatasetSnapshot, StoreSnapshot},
};

fn numeric(value: &mut Numeric, data: &DatasetSnapshot) -> ChartResult<()> {
    match value {
        Numeric::Scaled { input, .. } => numeric(input, data),
        Numeric::Expression(expr) => {
            super::stats::numeric_space(data, &Numeric::Expression(expr.clone()))?;
            {
                let rows = data.rows().collect::<Vec<_>>();
                *expr = expr.specialize_reductions(
                    rows.len(),
                    ExpressionLimits::default(),
                    |_| Ok(ExpressionType::Number),
                    |r, i| {
                        super::stats::number(rows[i], &r.numeric()).map_or(
                            ExpressionValue::Missing(ExpressionType::Number),
                            ExpressionValue::Number,
                        )
                    },
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
fn aes(
    aes: &mut SourceAes,
    f: &mut impl FnMut(&mut Numeric) -> ChartResult<()>,
) -> ChartResult<()> {
    for n in [
        &mut aes.x,
        &mut aes.y,
        &mut aes.x2,
        &mut aes.y2,
        &mut aes.low,
        &mut aes.high,
        &mut aes.size,
    ]
    .into_iter()
    .flatten()
    {
        f(n)?;
    }
    Ok(())
}
fn statistic(
    stat: &mut Statistic,
    f: &mut impl FnMut(&mut Numeric) -> ChartResult<()>,
) -> ChartResult<()> {
    match &mut stat.parameters {
        StatParameters::Bin(s) => f(&mut s.input)?,
        StatParameters::AutoBin(s) => f(&mut s.input)?,
        StatParameters::Summary(s) => f(&mut s.input)?,
        StatParameters::Ols(s) => {
            f(&mut s.x)?;
            f(&mut s.y)?;
        }
        StatParameters::Count(s) => {
            for n in &mut s.required {
                f(n)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn layer(
    layer: &mut Layer,
    f: &mut impl FnMut(&mut Numeric) -> ChartResult<()>,
) -> ChartResult<()> {
    if let Mappings::Source(a) = &mut layer.mappings {
        aes(a, f)?;
    }
    if let Some(g) = &mut layer.grammar {
        aes(&mut g.source, f)?;
    }
    statistic(&mut layer.statistic, f)?;
    for filter in &mut layer.filters {
        f(&mut filter.value)?;
    }
    if let Some(ColorEncoding {
        input: ColorInput::Numeric(n),
        ..
    }) = &mut layer.color
    {
        f(n)?;
    }
    Ok(())
}
fn dataset(
    input: DataRef,
    definition: &ChartDefinition,
    source: &StoreSnapshot,
) -> ChartResult<crate::DatasetId> {
    let mut input = input;
    for _ in 0..=definition.transforms.len() {
        match input {
            DataRef::Dataset(id) => {
                source.dataset(id)?;
                return Ok(id);
            }
            DataRef::Transform(id) => {
                input = definition
                    .transforms
                    .iter()
                    .find(|t| t.id == id)
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::MissingResource,
                            "Expression input transform is absent.",
                        )
                    })?
                    .input
            }
        }
    }
    Err(error(
        DiagnosticCode::Validation,
        "Expression source dependency cycle.",
    ))
}
pub(super) fn specialize<'a>(
    definition: &'a ChartDefinition,
    source: &StoreSnapshot,
) -> ChartResult<std::borrow::Cow<'a, ChartDefinition>> {
    // Definitions are already schema-validated; this execution-local copy never enters interchange.
    let mut result = definition.clone();
    for node in &mut result.transforms {
        let data = source.dataset(dataset(node.input, definition, source)?)?;
        statistic(&mut node.statistic, &mut |n| numeric(n, data))?;
        if let Some(g) = &mut node.grammar {
            aes(&mut g.source, &mut |n| numeric(n, data))?;
        }
        for filter in &mut node.filters {
            numeric(&mut filter.value, data)?;
        }
    }
    for l in &mut result.layers {
        let data = source.dataset(dataset(l.data, definition, source)?)?;
        if let Mappings::Source(a) = &mut l.mappings
            && l.inherit
        {
            *a = a.inherit(&definition.mappings);
            l.inherit = false;
        }
        layer(l, &mut |n| numeric(n, data))?;
    }
    Ok(std::borrow::Cow::Owned(result))
}
fn has_numeric(value: &Numeric) -> bool {
    let mut value = value;
    for _ in 0..=64 {
        match value {
            Numeric::Expression(_) => return true,
            Numeric::Scaled { input, .. } => value = input,
            _ => return false,
        }
    }
    true // Validation rejects excessive scale nesting before specialization.
}
fn has_aes(a: &SourceAes) -> bool {
    [&a.x, &a.y, &a.x2, &a.y2, &a.low, &a.high, &a.size]
        .into_iter()
        .flatten()
        .any(has_numeric)
}
fn has_stat(s: &Statistic) -> bool {
    match &s.parameters {
        StatParameters::Bin(s) => has_numeric(&s.input),
        StatParameters::AutoBin(s) => has_numeric(&s.input),
        StatParameters::Summary(s) => has_numeric(&s.input),
        StatParameters::Ols(s) => has_numeric(&s.x) || has_numeric(&s.y),
        StatParameters::Count(s) => s.required.iter().any(has_numeric),
        _ => false,
    }
}
pub(super) fn has_expressions(d: &ChartDefinition) -> bool {
    has_aes(&d.mappings)
        || d.layers.iter().any(|l| {
            matches!(&l.mappings,Mappings::Source(a) if has_aes(a))
                || l.grammar.as_ref().is_some_and(|g| has_aes(&g.source))
                || has_stat(&l.statistic)
                || l.filters.iter().any(|f| has_numeric(&f.value))
                || l.color
                    .as_ref()
                    .is_some_and(|c| matches!(&c.input,ColorInput::Numeric(n) if has_numeric(n)))
        })
        || d.transforms.iter().any(|t| {
            has_stat(&t.statistic)
                || t.grammar.as_ref().is_some_and(|g| has_aes(&g.source))
                || t.filters.iter().any(|f| has_numeric(&f.value))
        })
}
