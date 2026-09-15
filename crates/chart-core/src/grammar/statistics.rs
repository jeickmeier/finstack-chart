use super::stats::{
    group_value, number, numeric_space, validate_group, validate_operation, warning,
};
use super::*;
use crate::data::{DatasetSnapshot, InvalidPolicy};
use crate::provenance::Target;
use crate::{
    AggregateId, ChartResult, DerivedId, Diagnostic, DiagnosticCode, RowKey, SchemaVersion,
};
use std::collections::BTreeMap;

pub(super) fn validate_space(space: &StatSpace) -> ChartResult<()> {
    if let StatSpace::Transformed(t) = space {
        validate_operation(&t.operation, "chart.affine")?;
        if !t.factor.is_finite() || t.factor == 0. || !t.offset.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Statistic affine space requires finite nonzero factor and finite offset.",
            ));
        }
    }
    Ok(())
}
pub(super) fn space(
    data: &DatasetSnapshot,
    input: &Numeric,
    declared: &StatSpace,
) -> ChartResult<ValueSpace> {
    let input = numeric_space(data, input)?;
    Ok(match declared {
        StatSpace::Data => input,
        StatSpace::Transformed(t) => ValueSpace::Transformed {
            input: Box::new(input),
            transform: t.clone(),
        },
    })
}
fn transformed(value: Option<f64>, space: &StatSpace) -> Option<f64> {
    value.and_then(|v| match space {
        StatSpace::Data => Some(v),
        StatSpace::Transformed(t) => {
            let v = t.factor * v + t.offset;
            v.is_finite().then_some(v)
        }
    })
}
pub(super) fn validate(stat: &Statistic, limits: CompileLimits) -> ChartResult<()> {
    match &stat.parameters {
        StatParameters::AutoBin(s) => {
            validate_operation(&stat.operation, "chart.auto_bin")?;
            if s.bins == 0 || s.bins.checked_add(1).is_none_or(|n| n > limits.max_edges) {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Automatic bin count must fit the positive edge budget.",
                ));
            }
            validate_space(&s.space)
        }
        StatParameters::Count(s) => {
            validate_operation(&stat.operation, "chart.count")?;
            if s.required.len() > limits.max_filters {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Count required-input budget exceeded.",
                ));
            }
            Ok(())
        }
        StatParameters::Summary(s) => {
            validate_operation(&stat.operation, "chart.summary")?;
            if s.quantiles.len() > limits.max_edges {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Quantile output budget exceeded.",
                ));
            }
            if s.quantiles
                .iter()
                .any(|p| !p.is_finite() || !(0. ..=1.).contains(p))
            {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Quantile probabilities must lie in [0,1].",
                ));
            }
            validate_space(&s.space)
        }
        StatParameters::Ols(s) => {
            validate_operation(&stat.operation, "chart.ols")?;
            validate_space(&s.x_space)?;
            validate_space(&s.y_space)
        }
        _ => unreachable!("handled by existing stat validation"),
    }
}
pub(super) fn schema(
    stat: &Statistic,
    data: &DatasetSnapshot,
    limits: CompileLimits,
) -> ChartResult<Vec<StatColumn>> {
    validate(stat, limits)?;
    let mut fields = vec![StatColumn {
        field: StatField::Count,
        kind: GeneratedKind::UInt64,
        nullable: false,
        space: ValueSpace::Data,
    }];
    let mut add = |field, space, nullable| {
        fields.push(StatColumn {
            field,
            space,
            nullable,
            kind: GeneratedKind::Float64,
        })
    };
    match &stat.parameters {
        StatParameters::Count(s) => {
            validate_group(data, &s.grouping)?;
            for input in &s.required {
                numeric_space(data, input)?;
            }
        }
        StatParameters::Summary(s) => {
            validate_group(data, &s.grouping)?;
            let v = space(data, &s.input, &s.space)?;
            for field in [
                StatField::Min,
                StatField::Max,
                StatField::Mean,
                StatField::Sum,
            ] {
                add(field, v.clone(), true);
            }
            for i in 0..s.quantiles.len() {
                add(StatField::Quantile(i), v.clone(), true);
            }
        }
        StatParameters::Ols(s) => {
            validate_group(data, &s.grouping)?;
            add(StatField::X, space(data, &s.x, &s.x_space)?, false);
            add(StatField::Y, space(data, &s.y, &s.y_space)?, false);
            // Coefficients describe the declared model, not coordinates to transform again.
            add(StatField::Intercept, ValueSpace::Data, false);
            add(StatField::Slope, ValueSpace::Data, false);
        }
        _ => {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Expected a count, summary or fit output.",
            ));
        }
    }
    let grouping = match &stat.parameters {
        StatParameters::Count(s) => &s.grouping,
        StatParameters::Summary(s) => &s.grouping,
        StatParameters::Ols(s) => &s.grouping,
        _ => unreachable!(),
    };
    let mut catalog = std::collections::BTreeSet::new();
    for group in data.rows().filter_map(|r| group_value(r, grouping)) {
        if !catalog.contains(&group) && catalog.len() >= limits.max_groups {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Generated group catalog budget exceeded.",
            ));
        }
        catalog.insert(group);
    }
    let mut groups: Vec<_> = catalog.into_iter().collect();
    if *grouping == Grouping::All && groups.is_empty() {
        groups.push(GroupValue::All);
    }
    if let Grouping::Field(f) = grouping
        && let Some(catalog) = data.categories(*f)
    {
        groups.sort_by_key(|g| {
            catalog
                .iter()
                .position(|v| {
                    Some(v)
                        == match g {
                            GroupValue::Text(v) => Some(v),
                            _ => None,
                        }
                })
                .unwrap_or(usize::MAX)
        });
    }
    fields.push(StatColumn {
        field: StatField::Group,
        kind: GeneratedKind::Categorical,
        nullable: false,
        space: ValueSpace::Categorical {
            categories: groups.iter().map(group_label).collect(),
        },
    });
    Ok(fields)
}
pub(super) fn group_label(group: &GroupValue) -> String {
    group.label()
}
/// Sorted-key compensated summation gives stable input reorder behavior.
pub(super) fn sum<I>(values: I) -> ChartResult<f64>
where
    I: IntoIterator<Item = f64>,
    I::IntoIter: Clone,
{
    fn compensated(values: impl Iterator<Item = f64>) -> f64 {
        let (mut total, mut correction) = (0_f64, 0_f64);
        for v in values {
            let next = total + v;
            correction += if total.abs() >= v.abs() {
                (total - next) + v
            } else {
                (v - next) + total
            };
            total = next;
        }
        total + correction
    }
    let values = values.into_iter();
    let total = compensated(values.clone());
    if total.is_finite() {
        return Ok(total);
    }
    // Avoid rejecting a representable result solely because an intermediate partial sum overflowed.
    let scale = values.clone().map(f64::abs).fold(0_f64, f64::max);
    finite(
        compensated(values.map(|v| v / scale)) * scale,
        "Statistic sum is not representable.",
    )
}

pub(super) fn mean<I>(values: I, count: usize) -> ChartResult<f64>
where
    I: IntoIterator<Item = f64>,
    I::IntoIter: Clone,
{
    if count == 0 {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Mean requires at least one value.",
        ));
    }
    let values = values.into_iter();
    if let Ok(total) = sum(values.clone()) {
        let initial = total / count as f64;
        // A residual pass corrects rounding in the accumulated sum and division.
        // If subtraction overflows for extreme finite inputs, retain the stable
        // first estimate instead of rejecting a representable mean.
        let residuals = values.clone().map(|v| v - initial);
        if residuals.clone().all(f64::is_finite)
            && let Ok(residual) = sum(residuals)
        {
            return finite(
                initial + residual / count as f64,
                "Statistic mean is not representable.",
            );
        }
        return Ok(initial);
    }
    let scale = values.clone().map(f64::abs).fold(0_f64, f64::max);
    finite(
        (sum(values.map(|v| v / scale))? / count as f64) * scale,
        "Statistic mean is not representable.",
    )
}
fn finite(v: f64, message: &str) -> ChartResult<f64> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(error(DiagnosticCode::PrecisionLoss, message))
    }
}
fn quantile(values: &[f64], p: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let h = (values.len() - 1) as f64 * p;
    let i = h.floor() as usize;
    let a = values[i];
    let b = values[(i + 1).min(values.len() - 1)];
    let t = h - i as f64;
    Some(if a.signum() == b.signum() {
        a + (b - a) * t
    } else {
        a * (1. - t) + b * t
    })
}
struct Observation {
    key: RowKey,
    x: f64,
    y: f64,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    table: &mut PreparedTable,
    data: &DatasetSnapshot,
    stat: &Statistic,
    policy: InvalidPolicy,
    scope: &str,
    limits: CompileLimits,
    counts: &mut PopulationCounts,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<(Grouping, StatSpace)> {
    let fields = schema(stat, data, limits)?;
    let PreparedRows::Source(rows) = &table.rows else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Statistics require source observations; generated inputs need identity mappings.",
        ));
    };
    let (grouping, declared) = match &stat.parameters {
        StatParameters::Count(s) => (&s.grouping, StatSpace::Data),
        StatParameters::Summary(s) => (&s.grouping, s.space.clone()),
        StatParameters::Ols(s) => (&s.grouping, s.x_space.clone()),
        _ => unreachable!(),
    };
    let mut groups: BTreeMap<GroupValue, Vec<Observation>> = BTreeMap::new();
    if *grouping == Grouping::All {
        groups.insert(GroupValue::All, vec![]);
    }
    let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
    let mut samples = vec![];
    for row in rows.iter() {
        let source = index[&row.key];
        let group = group_value(source, grouping);
        if let Some(group) = &group {
            if !groups.contains_key(group) && groups.len() >= limits.max_groups {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Statistic group budget exceeded.",
                ));
            }
            groups.entry(group.clone()).or_default();
        }
        let pair = match &stat.parameters {
            StatParameters::Count(s) => s
                .required
                .iter()
                .all(|v| number(source, v).is_some())
                .then_some((0., 0.)),
            StatParameters::Summary(s) => {
                transformed(number(source, &s.input), &s.space).map(|v| (v, 0.))
            }
            StatParameters::Ols(s) => transformed(number(source, &s.x), &s.x_space)
                .zip(transformed(number(source, &s.y), &s.y_space)),
            _ => unreachable!(),
        };
        if let (Some(group), Some((x, y))) = (group, pair) {
            groups
                .get_mut(&group)
                .expect("inserted group")
                .push(Observation { key: row.key, x, y });
        } else {
            counts.invalid_stat += 1;
            if samples.len() < 32 {
                samples.push(row.key);
            }
        }
    }
    warning(
        policy,
        counts.invalid_stat,
        samples,
        "Statistic excluded invalid required values or groups.",
        diagnostics,
    )?;
    let multiplier = if matches!(stat.parameters, StatParameters::Ols(_)) {
        2
    } else {
        1
    };
    if groups
        .len()
        .checked_mul(multiplier)
        .and_then(|n| n.checked_mul(fields.len()))
        .is_none_or(|n| n > limits.max_prepared_rows)
        || groups.len() > limits.max_groups
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Statistical output row budget exceeded.",
        ));
    }
    let mut output = vec![];
    for (group, mut rows) in groups {
        rows.sort_by_key(|r| r.key);
        let members: std::sync::Arc<[RowKey]> = rows.iter().map(|r| r.key).collect();
        let count = rows.len() as u64;
        let mut values = vec![];
        let mut push = |field, value| values.push(StatValue { field, value });
        let target = Target::Aggregate {
            id: AggregateId::new(0),
            group: format!("{scope}/{group:?}/{}", stat.operation.id),
            input: table.input,
            members: members.clone(),
        };
        match &stat.parameters {
            StatParameters::Count(_) => {}
            StatParameters::Summary(s) => {
                let mut sorted: Vec<_> = rows.iter().map(|r| r.x).collect();
                sorted.sort_by(f64::total_cmp);
                let total = if rows.is_empty() {
                    s.empty_sum_zero.then_some(0.)
                } else {
                    Some(sum(rows.iter().map(|r| r.x))?)
                };
                let mean = if rows.is_empty() {
                    None
                } else {
                    Some(mean(rows.iter().map(|r| r.x), rows.len())?)
                };
                push(StatField::Min, sorted.first().copied());
                push(StatField::Max, sorted.last().copied());
                push(StatField::Sum, total);
                push(StatField::Mean, mean);
                for (i, p) in s.quantiles.iter().enumerate() {
                    push(StatField::Quantile(i), quantile(&sorted, *p));
                }
            }
            StatParameters::Ols(_) => {
                if rows.len() < 2 {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "OLS requires at least two usable distinct x values in every declared group.",
                    ));
                }
                let xmin = rows
                    .iter()
                    .map(|r| r.x)
                    .min_by(f64::total_cmp)
                    .expect("nonempty");
                let xmax = rows
                    .iter()
                    .map(|r| r.x)
                    .max_by(f64::total_cmp)
                    .expect("nonempty");
                let xspan = finite(xmax - xmin, "OLS predictor span is not representable.")?;
                if xspan <= 0. {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "OLS predictor is singular: no distinct x values.",
                    ));
                }
                // Shift and scale before moments; large origins cannot cause catastrophic x*x cancellation.
                let xmean = sum(rows
                    .iter()
                    .map(|r| ((r.x - xmin) / xspan) / rows.len() as f64))?;
                let yscale = rows
                    .iter()
                    .map(|r| r.y.abs())
                    .fold(0_f64, f64::max)
                    .max(f64::MIN_POSITIVE);
                let ymean = sum(rows.iter().map(|r| (r.y / yscale) / rows.len() as f64))?;
                let xx = sum(rows.iter().map(|r| {
                    let d = (r.x - xmin) / xspan - xmean;
                    d * d
                }))?;
                let xy = sum(rows
                    .iter()
                    .map(|r| ((r.x - xmin) / xspan - xmean) * (r.y / yscale - ymean)))?;
                if xx <= f64::EPSILON {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "OLS centered predictor moments are numerically singular.",
                    ));
                }
                let beta = xy / xx;
                let slope = finite((beta * yscale) / xspan, "OLS slope is not representable.")?;
                let at_min = finite(
                    (ymean - beta * xmean) * yscale,
                    "OLS prediction is not representable.",
                )?;
                let intercept =
                    finite(at_min - slope * xmin, "OLS intercept is not representable.")?;
                let target = Target::Derived {
                    id: DerivedId::new(0),
                    model: format!("{}/{scope}/{group:?}", stat.operation.id),
                    model_version: stat.operation.version,
                    inputs: vec![table.input],
                };
                for x in [xmin, xmax] {
                    let y = finite(
                        (ymean + beta * ((x - xmin) / xspan - xmean)) * yscale,
                        "OLS endpoint is not representable.",
                    )?;
                    output.push(StatisticalRow {
                        group: group.clone(),
                        count,
                        members: members.clone(),
                        target: target.clone(),
                        values: vec![
                            StatValue {
                                field: StatField::X,
                                value: Some(x),
                            },
                            StatValue {
                                field: StatField::Y,
                                value: Some(y),
                            },
                            StatValue {
                                field: StatField::Intercept,
                                value: Some(intercept),
                            },
                            StatValue {
                                field: StatField::Slope,
                                value: Some(slope),
                            },
                        ],
                    });
                }
                continue;
            }
            _ => unreachable!(),
        }
        output.push(StatisticalRow {
            group,
            count,
            values,
            members,
            target,
        });
    }
    table.schema = OutputSchema::Statistical {
        version: SchemaVersion::new(1),
        fields,
    };
    table.rows = PreparedRows::Statistical(output.into());
    Ok((grouping.clone(), declared))
}
pub(super) fn automatic(
    spec: &AutoBinSpec,
    table: &PreparedTable,
    data: &DatasetSnapshot,
) -> ChartResult<BinSpec> {
    let PreparedRows::Source(rows) = &table.rows else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Automatic bins require source rows.",
        ));
    };
    numeric_space(data, &spec.input)?;
    validate_group(data, &spec.grouping)?;
    let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
    let mut bounds: Option<(f64, f64)> = None;
    for row in rows.iter() {
        let r = index[&row.key];
        if group_value(r, &spec.grouping).is_none() {
            continue;
        }
        if let Some(v) = transformed(number(r, &spec.input), &spec.space) {
            bounds = Some(bounds.map_or((v, v), |(a, b)| (a.min(v), b.max(v))));
        }
    }
    let (mut lo, mut hi) = bounds.unwrap_or((0., 1.));
    if lo == hi {
        let half = if lo == 0. {
            1.
        } else {
            (lo.abs() * 0.05).max(f64::MIN_POSITIVE)
        };
        lo -= half;
        hi += half;
    }
    let edges: Vec<_> = (0..=spec.bins)
        .map(|i| {
            let t = i as f64 / spec.bins as f64;
            if i == 0 {
                lo
            } else if i == spec.bins {
                hi
            } else {
                lo * (1. - t) + hi * t
            }
        })
        .collect();
    if edges.iter().any(|v| !v.is_finite()) || edges.windows(2).any(|v| v[0] >= v[1]) {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "Automatic equal-width edges cannot be represented distinctly; choose fewer or explicit edges.",
        ));
    }
    Ok(BinSpec {
        input: spec.input.clone(),
        edges,
        outliers: OutlierPolicy::Exclude,
        grouping: spec.grouping.clone(),
        space: spec.space.clone(),
    })
}
