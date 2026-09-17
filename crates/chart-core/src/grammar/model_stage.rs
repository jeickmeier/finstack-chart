//! Grouped model preparation, retaining the common generated-table/provenance contract.
use super::*;
use crate::data::{DatasetSnapshot, InvalidPolicy};
use crate::provenance::Target;
use crate::{ChartResult, DerivedId, Diagnostic, DiagnosticCode, Revision, RowKey, SchemaVersion};
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn validate(spec: &ModelSpec, limits: CompileLimits) -> ChartResult<()> {
    statistics::validate_space(&spec.x_space)?;
    statistics::validate_space(&spec.y_space)?;
    model_predict::validate(&spec.options, limits)?;
    if spec.retained_fields.len() + spec.retained_numeric.len() > limits.max_filters
        || spec
            .training_range
            .is_some_and(|r| r.iter().any(|v| !v.is_finite()))
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Invalid model input or retention budget.",
        ));
    }
    Ok(())
}
pub(super) fn schema(
    spec: &ModelSpec,
    data: &DatasetSnapshot,
    limits: CompileLimits,
) -> ChartResult<Vec<StatColumn>> {
    validate(spec, limits)?;
    stats::validate_group(data, &spec.grouping)?;
    for n in spec.numerics() {
        stats::numeric_space(data, n)?;
    }
    for f in &spec.retained_fields {
        stats::validate_group(data, &Grouping::Field(*f))?;
    }
    let x = statistics::space(data, &spec.x, &spec.x_space)?;
    let y = statistics::space(data, &spec.y, &spec.y_space)?;
    let mut fields = vec![StatColumn {
        field: StatField::Count,
        kind: GeneratedKind::UInt64,
        nullable: false,
        space: ValueSpace::Data,
    }];
    for (field, space) in [
        (StatField::X, x),
        (StatField::Y, y.clone()),
        (StatField::Lower, y.clone()),
        (StatField::Upper, y),
        (StatField::StandardError, ValueSpace::Data),
        (StatField::QuantileProbability, ValueSpace::Data),
    ] {
        fields.push(StatColumn {
            field,
            kind: GeneratedKind::Float64,
            nullable: true,
            space,
        });
    }
    let mut group_column = statistics::group_column(data, &spec.grouping, limits)?;
    if let ModelMethod::Quantile { probabilities, .. } = &spec.options.method {
        let mut groups = data
            .rows()
            .filter_map(|r| stats::group_value(r, &spec.grouping))
            .collect::<std::collections::BTreeSet<_>>();
        if groups.is_empty() && spec.grouping == Grouping::All {
            groups.insert(GroupValue::All);
        }
        if groups
            .len()
            .checked_mul(probabilities.len())
            .is_none_or(|n| n > limits.max_groups)
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Quantile curve group budget exceeded.",
            ));
        }
        group_column.space = ValueSpace::Categorical {
            categories: groups
                .iter()
                .flat_map(|g| {
                    probabilities
                        .iter()
                        .map(move |q| curve_group(g, Some(*q)).label())
                })
                .collect(),
        };
    }
    fields.push(group_column);
    Ok(fields)
}
struct Sample {
    key: RowKey,
    x: f64,
    y: f64,
    weight: f64,
    retained: GroupValue,
    numeric: Vec<(Numeric, Option<f64>)>,
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    table: &mut PreparedTable,
    data: &DatasetSnapshot,
    spec: &ModelSpec,
    policy: InvalidPolicy,
    scope: &str,
    limits: CompileLimits,
    counts: &mut PopulationCounts,
    diagnostics: &mut Vec<Diagnostic>,
    registry: &ExtensionRegistry,
) -> ChartResult<(Grouping, StatSpace)> {
    let fields = schema(spec, data, limits)?;
    let PreparedRows::Source(source) = &table.rows else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Models require source observations.",
        ));
    };
    let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
    let mut groups: BTreeMap<GroupValue, Vec<Sample>> = BTreeMap::new();
    let mut excluded = vec![];
    for row in source.iter() {
        let r = index[&row.key];
        let group = stats::group_value(r, &spec.grouping);
        let x = statistics::transformed(stats::number(r, &spec.x), &spec.x_space);
        let y = statistics::transformed(stats::number(r, &spec.y), &spec.y_space);
        let w = spec
            .weight
            .as_ref()
            .map_or(Some(1.), |n| stats::number(r, n));
        let (Some(group), Some(x), Some(y), Some(weight)) = (group, x, y, w) else {
            counts.invalid_stat += 1;
            if excluded.len() < 32 {
                excluded.push(row.key);
            }
            continue;
        };
        if !groups.contains_key(&group) && groups.len() >= limits.max_groups {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Model group budget exceeded.",
            ));
        }
        let retained = stats::group_value(r, &Grouping::Interaction(spec.retained_fields.clone()))
            .unwrap_or(GroupValue::All);
        let numeric = spec
            .retained_numeric
            .iter()
            .map(|n| (n.clone(), stats::number(r, n)))
            .collect();
        groups.entry(group).or_default().push(Sample {
            key: row.key,
            x,
            y,
            weight,
            retained,
            numeric,
        });
    }
    stats::warning(
        policy,
        counts.invalid_stat,
        excluded,
        "Model excluded missing required observations.",
        diagnostics,
    )?;
    let largest_group = spec
        .largest_group
        .unwrap_or_else(|| groups.values().map(Vec::len).max().unwrap_or(0));
    let mut output = vec![];
    for (group, rows) in groups {
        let x: Vec<_> = rows.iter().map(|r| r.x).collect();
        let y: Vec<_> = rows.iter().map(|r| r.y).collect();
        let w: Vec<_> = rows.iter().map(|r| r.weight).collect();
        let mut unique = x.clone();
        unique.sort_by(f64::total_cmp);
        unique.dedup();
        if unique.len() < 2 {
            let mut d = error(
                DiagnosticCode::NumericalDomain,
                "Model group has fewer than two distinct predictor values.",
            );
            d.severity = crate::Severity::Warning;
            diagnostics.push(d);
            continue;
        }
        let grid = if let Some(grid) = &spec.options.xseq {
            grid.clone()
        } else {
            let range = if spec.options.full_range {
                spec.training_range
                    .unwrap_or([unique[0], *unique.last().unwrap()])
            } else {
                [unique[0], *unique.last().unwrap()]
            };
            if !matches!(spec.options.method, ModelMethod::Quantile { .. })
                && spec.x_space == StatSpace::Data
                && integer_input(data, &spec.x)
            {
                if spec.options.full_range {
                    range.to_vec()
                } else {
                    unique.clone()
                }
            } else {
                univariate_kernels::grid(range, spec.options.n, limits.max_prepared_rows)?
            }
        };
        let predictions = if let ModelMethod::Registered {
            operation,
            parameters,
        } = &spec.options.method
        {
            model_extensions::predict(
                registry,
                operation,
                parameters,
                &x,
                &y,
                &w,
                &grid,
                &spec.options,
                limits,
            )
        } else {
            model_predict::predict(&x, &y, &w, &spec.options, &grid, largest_group, limits)
        };
        let predictions = match predictions {
            Ok(p) => p,
            Err(mut diagnostic)
                if diagnostic.code == DiagnosticCode::NumericalDomain
                    && !matches!(spec.options.method, ModelMethod::Quantile { .. }) =>
            {
                diagnostic.severity = crate::Severity::Warning;
                diagnostics.push(diagnostic);
                continue;
            }
            Err(diagnostic) => return Err(diagnostic),
        };
        diagnostics.extend(predictions.diagnostics);
        let components = spec
            .retained_fields
            .iter()
            .filter_map(|f| {
                let first = rows.first()?.retained.component(*f)?;
                rows.iter()
                    .all(|r| r.retained.component(*f) == Some(first))
                    .then(|| (*f, first.clone()))
            })
            .collect::<Vec<_>>();
        let retained = if components.is_empty() {
            GroupValue::All
        } else {
            GroupValue::Interaction(components.clone())
        };
        let retained_numeric = rows[0]
            .numeric
            .iter()
            .filter(|(n, v)| {
                rows.iter()
                    .all(|r| r.numeric.iter().any(|(a, b)| a == n && b == v))
            })
            .cloned()
            .collect::<Vec<_>>();
        if components.len() < spec.retained_fields.len()
            || retained_numeric.len() < spec.retained_numeric.len()
        {
            let mut d = error(
                DiagnosticCode::NumericalDomain,
                "Model dropped varying source aesthetics.",
            );
            d.severity = crate::Severity::Warning;
            diagnostics.push(d);
        }
        let mut members: Vec<_> = rows.iter().map(|r| r.key).collect();
        members.sort();
        members.dedup();
        let members: Arc<[RowKey]> = members.into();
        for p in predictions.rows {
            if output
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(fields.len()))
                .is_none_or(|n| n > limits.max_prepared_rows)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Model output budget exceeded.",
                ));
            }
            let values = vec![
                (StatField::X, Some(p.x)),
                (StatField::Y, p.mean),
                (StatField::Lower, p.lower),
                (StatField::Upper, p.upper),
                (StatField::StandardError, p.se),
                (StatField::QuantileProbability, p.quantile),
            ]
            .into_iter()
            .map(|(field, value)| StatValue { field, value })
            .collect();
            output.push(StatisticalRow {
                group: curve_group(&group, p.quantile),
                count: members.len() as u64,
                members: members.clone(),
                target: Target::Derived {
                    id: DerivedId::new(output.len() as u64),
                    model: format!("chart.model/{scope}/{group:?}/{}", output.len()),
                    model_version: Revision::new(1),
                    inputs: vec![table.input],
                },
                values,
                retained: retained.clone(),
                retained_numeric: retained_numeric.clone(),
                outliers: vec![],
            });
        }
    }
    table.schema = OutputSchema::Statistical {
        version: SchemaVersion::new(3),
        fields,
    };
    table.rows = PreparedRows::Statistical(output.into());
    Ok((spec.grouping.clone(), spec.x_space.clone()))
}

fn curve_group(group: &GroupValue, quantile: Option<f64>) -> GroupValue {
    quantile.map_or_else(
        || group.clone(),
        |p| GroupValue::Text(format!("{group:?} / quantile {p}")),
    )
}

fn integer_input(data: &DatasetSnapshot, input: &Numeric) -> bool {
    match input {
        Numeric::Field(id) => data.schema().field(*id).is_some_and(|(_, f)| {
            matches!(
                f.kind,
                crate::data::FieldKind::Int64 | crate::data::FieldKind::UInt64
            )
        }),
        Numeric::Scaled { input, scale, .. } if scale.transform.is_none() => {
            integer_input(data, input)
        }
        _ => false,
    }
}
