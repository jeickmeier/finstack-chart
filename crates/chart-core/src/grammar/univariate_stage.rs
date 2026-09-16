//! Shared preparation for empirical, theoretical and interpolated one-dimensional rows.
use super::*;
use crate::data::{DatasetSnapshot, InvalidPolicy};
use crate::provenance::{SourceRef, Target};
use crate::{ChartResult, DerivedId, Diagnostic, DiagnosticCode, Revision, RowKey, SchemaVersion};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
fn invalid(message: &str) -> Diagnostic {
    error(DiagnosticCode::NumericalDomain, message)
}
pub(super) fn validate(spec: &UnivariateSpec, limits: CompileLimits) -> ChartResult<()> {
    statistics::validate_space(&spec.space)?;
    statistics::validate_space(&spec.second_space)?;
    if spec.retained_fields.len() + spec.retained_numeric.len() > limits.max_filters {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Analytic aesthetic budget exceeded.",
        ));
    }
    if spec
        .training_range
        .is_some_and(|r| r.iter().any(|v| !v.is_finite()))
    {
        return Err(invalid("Analytic training range must be finite."));
    }
    match &spec.kind {
        UnivariateKind::Ecdf { n, pad } => {
            if n.is_some_and(|n| {
                n.checked_add(usize::from(*pad) * 2)
                    .is_none_or(|v| v > limits.max_prepared_rows)
            }) {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "ECDF grid exceeds row budget.",
                ));
            }
        }
        UnivariateKind::Qq {
            probabilities,
            quantiles,
            ..
        } => {
            if probabilities
                .iter()
                .any(|p| !p.is_finite() || !(0. ..=1.).contains(p))
                || quantiles.as_ref().is_some_and(|q| {
                    q.len() > limits.max_prepared_rows
                        || q.iter().any(|p| !p.is_finite() || !(0. ..=1.).contains(p))
                })
            {
                return Err(invalid(
                    "QQ probabilities must lie in [0,1] and fit the output budget.",
                ));
            }
        }
        UnivariateKind::Function { n, range, .. } => {
            if *n > limits.max_prepared_rows {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Function grid exceeds row budget.",
                ));
            }
            if range.is_some_and(|r| r.iter().any(|v| !v.is_finite())) {
                return Err(invalid("Function range must be finite."));
            }
        }
        UnivariateKind::Connect {
            connection: Connection::Matrix(m),
        } if m.is_empty()
            || m.len() > limits.max_prepared_rows
            || m.iter().flatten().any(|v| !v.is_finite()) =>
        {
            return Err(invalid(
                "Connection matrix must be nonempty, finite and bounded.",
            ));
        }
        _ => {}
    }
    if matches!(
        spec.kind,
        UnivariateKind::Align | UnivariateKind::Connect { .. }
    ) && spec.second.is_none()
    {
        return Err(invalid("Align and connect require both positional inputs."));
    }
    if let Some(scale) = &spec.output_scale {
        scale.validate()?;
    }
    Ok(())
}
pub(super) fn validate_registry(
    spec: &UnivariateSpec,
    registry: &ExtensionRegistry,
    portable: bool,
) -> ChartResult<()> {
    match &spec.kind {
        UnivariateKind::Function { function, .. } => function.validate_function(registry, portable),
        UnivariateKind::Qq { distribution, .. } => {
            distribution.validate_function(registry, portable)
        }
        _ => Ok(()),
    }
}
fn pointwise_output(spec: &UnivariateSpec) -> Option<&ScaleProjection> {
    spec.output_scale.as_ref().filter(|scale| {
        !scale
            .transform
            .as_ref()
            .and_then(|t| t.ggplot_transform())
            .is_some_and(|t| !t.is_pointwise())
    })
}
pub(super) fn schema(
    spec: &UnivariateSpec,
    data: &DatasetSnapshot,
    limits: CompileLimits,
) -> ChartResult<Vec<StatColumn>> {
    validate(spec, limits)?;
    stats::validate_group(data, &spec.grouping)?;
    for numeric in spec.numerics() {
        stats::numeric_space(data, numeric)?;
    }
    for field in &spec.retained_fields {
        stats::validate_group(data, &Grouping::Interaction(vec![*field]))?;
    }
    let sample = statistics::space(data, &spec.input, &spec.space)?;
    let secondary = spec
        .second
        .as_ref()
        .map(|v| statistics::space(data, v, &spec.second_space))
        .transpose()?
        .unwrap_or(ValueSpace::Data);
    let (x, y) = match &spec.kind {
        UnivariateKind::Qq { .. } => (ValueSpace::Data, sample),
        UnivariateKind::Ecdf { .. } => (sample, ValueSpace::Data),
        UnivariateKind::Function { .. } => (
            sample,
            pointwise_output(spec).map_or(ValueSpace::Data, |p| p.space(ValueSpace::Data)),
        ),
        _ => (sample, secondary),
    };
    let mut fields = vec![
        StatColumn {
            field: StatField::Count,
            kind: GeneratedKind::UInt64,
            nullable: false,
            space: ValueSpace::Data,
        },
        StatColumn {
            field: StatField::X,
            kind: GeneratedKind::Float64,
            nullable: true,
            space: x,
        },
        StatColumn {
            field: StatField::Y,
            kind: GeneratedKind::Float64,
            nullable: true,
            space: y,
        },
    ];
    let extra: &[StatField] = match spec.kind {
        UnivariateKind::Ecdf { .. } => &[StatField::Ecdf],
        UnivariateKind::Qq { line: true, .. } => &[StatField::Slope, StatField::Intercept],
        UnivariateKind::Align => &[StatField::AlignPadding],
        _ => &[],
    };
    fields.extend(extra.iter().cloned().map(|field| StatColumn {
        field,
        kind: GeneratedKind::Float64,
        nullable: true,
        space: ValueSpace::Data,
    }));
    fields.push(statistics::group_column(data, &spec.grouping, limits)?);
    Ok(fields)
}
#[derive(Clone)]
struct Sample {
    key: RowKey,
    x: f64,
    y: Option<f64>,
    weight: Option<f64>,
    retained: GroupValue,
    numeric: Vec<(Numeric, Option<f64>)>,
}
fn transformed(v: Option<f64>, space: &StatSpace) -> Option<f64> {
    v.and_then(|v| {
        let v = match space {
            StatSpace::Data => v,
            StatSpace::Transformed(t) => v * t.factor + t.offset,
        };
        v.is_finite().then_some(v)
    })
}
fn inverse_input(input: &Numeric, value: f64) -> ChartResult<f64> {
    if let Numeric::Scaled { scale, .. } = input {
        scale
            .transform
            .as_ref()
            .map_or(Ok(value), |t| t.inverse(value))
    } else {
        Ok(value)
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    table: &mut PreparedTable,
    data: &DatasetSnapshot,
    spec: &UnivariateSpec,
    policy: InvalidPolicy,
    scope: &str,
    limits: CompileLimits,
    counts: &mut PopulationCounts,
    diagnostics: &mut Vec<Diagnostic>,
    registry: &ExtensionRegistry,
) -> ChartResult<(Grouping, StatSpace)> {
    let fields = schema(spec, data, limits)?;
    validate_registry(spec, registry, false)?;
    let PreparedRows::Source(source) = &table.rows else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Univariate statistics require source observations.",
        ));
    };
    let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
    let mut groups: BTreeMap<GroupValue, Vec<Sample>> = BTreeMap::new();
    let mut excluded = vec![];
    let needs_y = matches!(
        spec.kind,
        UnivariateKind::Align | UnivariateKind::Connect { .. } | UnivariateKind::Unique
    ) && spec.second.is_some();
    for row in source.iter() {
        let r = index[&row.key];
        let group = stats::group_value(r, &spec.grouping);
        let x = if spec.default_input {
            Some(0.)
        } else {
            transformed(stats::number(r, &spec.input), &spec.space)
        };
        let y = spec
            .second
            .as_ref()
            .and_then(|v| transformed(stats::number(r, v), &spec.second_space));
        if group.is_none() || x.is_none() || needs_y && y.is_none() {
            counts.invalid_stat += 1;
            if excluded.len() < 32 {
                excluded.push(row.key);
            }
            continue;
        }
        let group = group.unwrap();
        if !groups.contains_key(&group) && groups.len() >= limits.max_groups {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Analytic group budget exceeded.",
            ));
        }
        let retained = stats::group_value(r, &Grouping::Interaction(spec.retained_fields.clone()))
            .unwrap_or(GroupValue::All);
        groups.entry(group).or_default().push(Sample {
            key: row.key,
            x: x.unwrap(),
            y,
            weight: spec
                .weight
                .as_ref()
                .map_or(Some(1.), |w| stats::raw_number(r, w)),
            retained,
            numeric: spec
                .retained_numeric
                .iter()
                .map(|n| (n.clone(), stats::number(r, n)))
                .collect(),
        });
    }
    stats::warning(
        policy,
        counts.invalid_stat,
        excluded,
        "Univariate statistic excluded missing required values.",
        diagnostics,
    )?;
    if groups.is_empty() && matches!(spec.kind, UnivariateKind::Function { .. }) {
        groups.insert(GroupValue::All, vec![]);
    }
    let aligned = if matches!(spec.kind, UnivariateKind::Align) {
        Some(univariate_kernels::align(
            &groups
                .values()
                .map(|g| g.iter().map(|r| [r.x, r.y.unwrap_or(0.)]).collect())
                .collect::<Vec<_>>(),
            limits.max_prepared_rows,
        )?)
    } else {
        None
    };
    let mut output = Vec::new();
    for (group_index, (group, rows)) in groups.into_iter().enumerate() {
        let mut members: Vec<_> = rows.iter().map(|r| r.key).collect();
        members.sort();
        members.dedup();
        let members: Arc<[RowKey]> = members.into();
        let components = spec
            .retained_fields
            .iter()
            .filter_map(|field| {
                let first = rows.first()?.retained.component(*field)?;
                rows.iter()
                    .all(|row| row.retained.component(*field) == Some(first))
                    .then(|| (*field, first.clone()))
            })
            .collect::<Vec<_>>();
        let constant_numeric = rows
            .first()
            .map(|first| {
                first
                    .numeric
                    .iter()
                    .filter(|(input, value)| {
                        rows.iter()
                            .all(|row| row.numeric.iter().any(|(n, v)| n == input && v == value))
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if components.len() < spec.retained_fields.len()
            || constant_numeric.len() < spec.retained_numeric.len()
        {
            let mut warning = invalid("Univariate statistic dropped varying source aesthetics.");
            warning.severity = crate::Severity::Warning;
            diagnostics.push(warning);
        }
        let constant_retained = if components.is_empty() {
            GroupValue::All
        } else {
            GroupValue::Interaction(components)
        };
        let mut push = |point: [Option<f64>; 2],
                        extra: Vec<StatValue>,
                        source: Option<&Sample>|
         -> ChartResult<()> {
            if output
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(fields.len()))
                .is_none_or(|n| n > limits.max_prepared_rows)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Analytic generated table exceeds row budget.",
                ));
            }
            let target = source.map_or_else(
                || Target::Derived {
                    id: DerivedId::new(output.len() as u64),
                    model: format!("chart.univariate/{scope}/{group:?}/{}", output.len()),
                    model_version: Revision::new(1),
                    inputs: vec![table.input],
                },
                |s| {
                    Target::Source(SourceRef {
                        dataset: table.input.dataset,
                        key: s.key,
                    })
                },
            );
            let mut values = vec![
                StatValue {
                    field: StatField::X,
                    value: point[0],
                },
                StatValue {
                    field: StatField::Y,
                    value: point[1],
                },
            ];
            values.extend(extra);
            output.push(StatisticalRow {
                outliers: vec![],
                retained: source.map_or_else(|| constant_retained.clone(), |r| r.retained.clone()),
                retained_numeric: source
                    .map_or_else(|| constant_numeric.clone(), |r| r.numeric.clone()),
                group: group.clone(),
                count: source.map_or(members.len() as u64, |_| 1),
                members: source.map_or_else(|| members.clone(), |r| Arc::from([r.key])),
                target,
                values,
            });
            Ok(())
        };
        match &spec.kind {
            UnivariateKind::Ecdf { n, pad } => {
                let values: Vec<_> = rows.iter().map(|r| r.x).collect();
                let weights: Vec<_> = rows.iter().map(|r| r.weight).collect();
                match univariate_kernels::ecdf(
                    &values,
                    spec.weight.as_ref().map(|_| weights.as_slice()),
                    *n,
                    *pad,
                    limits.max_prepared_rows,
                ) {
                    Ok(result) => {
                        if result.replaced_weights > 0 || result.near_zero {
                            let mut d = invalid(
                                "ECDF replaced missing weights with zero or has near-zero total weight.",
                            );
                            d.severity = crate::Severity::Warning;
                            diagnostics.push(d);
                        }
                        for [x, y] in result.points {
                            push(
                                [Some(x), Some(y)],
                                vec![StatValue {
                                    field: StatField::Ecdf,
                                    value: Some(y),
                                }],
                                None,
                            )?;
                        }
                    }
                    Err(mut e)
                        if e.code == DiagnosticCode::NumericalDomain
                            && policy != InvalidPolicy::Strict =>
                    {
                        e.severity = crate::Severity::Warning;
                        diagnostics.push(e);
                    }
                    Err(e) => return Err(e),
                }
            }
            UnivariateKind::Qq {
                distribution,
                quantiles,
                line,
                probabilities,
                full_range,
            } => {
                let mut sample: Vec<_> = rows.iter().map(|r| r.x).collect();
                sample.sort_by(f64::total_cmp);
                if sample.is_empty() {
                    continue;
                }
                let q = quantiles
                    .clone()
                    .unwrap_or_else(|| univariate_kernels::plotting_positions(sample.len()));
                if q.len() != sample.len() {
                    return Err(invalid("QQ quantiles must match sample length."));
                }
                let theoretical =
                    distribution.evaluate_function(&q, registry, limits.max_prepared_rows)?;
                if *line {
                    let x = distribution.evaluate_function(
                        probabilities,
                        registry,
                        limits.max_prepared_rows,
                    )?;
                    let y = [
                        distributions::quantile(&sample, probabilities[0], 7)?,
                        distributions::quantile(&sample, probabilities[1], 7)?,
                    ];
                    let slope = x[0].zip(x[1]).map(|(a, b)| (y[1] - y[0]) / (b - a));
                    let intercept = x[0].zip(slope).map(|(x, s)| y[0] - s * x);
                    let range = if *full_range {
                        spec.training_range.unwrap_or([0., 1.])
                    } else {
                        let finite: Vec<_> = theoretical.iter().flatten().copied().collect();
                        [
                            finite.iter().copied().fold(f64::INFINITY, f64::min),
                            finite.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        ]
                    };
                    for x in range {
                        let y = slope
                            .zip(intercept)
                            .map(|(s, i)| s * x + i)
                            .filter(|v| v.is_finite());
                        push(
                            [Some(x), y],
                            vec![
                                StatValue {
                                    field: StatField::Slope,
                                    value: slope.filter(|v| v.is_finite()),
                                },
                                StatValue {
                                    field: StatField::Intercept,
                                    value: intercept.filter(|v| v.is_finite()),
                                },
                            ],
                            None,
                        )?;
                    }
                } else {
                    for (x, y) in theoretical.into_iter().zip(sample) {
                        push([x, Some(y)], vec![], None)?;
                    }
                }
            }
            UnivariateKind::Function { function, n, range } => {
                let own = if rows.is_empty() || spec.default_input {
                    [0., 1.]
                } else {
                    [
                        rows.iter().map(|r| r.x).fold(f64::INFINITY, f64::min),
                        rows.iter().map(|r| r.x).fold(f64::NEG_INFINITY, f64::max),
                    ]
                };
                let range = range.or(spec.training_range).unwrap_or(own);
                let x = univariate_kernels::grid(range, *n, limits.max_prepared_rows)?;
                let inverse: Vec<_> = x
                    .iter()
                    .map(|&x| inverse_input(&spec.input, x))
                    .collect::<ChartResult<_>>()?;
                let y = function.evaluate_function(&inverse, registry, limits.max_prepared_rows)?;
                for (x, y) in x.into_iter().zip(y) {
                    let y = pointwise_output(spec).map_or(y, |s| s.project_optional(y));
                    push([Some(x), y], vec![], None)?;
                }
            }
            UnivariateKind::Unique => {
                let mut seen = BTreeSet::new();
                for row in &rows {
                    let key = (
                        if row.x == 0. { 0 } else { row.x.to_bits() },
                        row.y.map(|v| if v == 0. { 0 } else { v.to_bits() }),
                        row.retained.clone(),
                        row.numeric
                            .iter()
                            .map(|(_, v)| v.map(f64::to_bits))
                            .collect::<Vec<_>>(),
                    );
                    if seen.insert(key) {
                        push([Some(row.x), row.y], vec![], Some(row))?;
                    }
                }
            }
            UnivariateKind::Connect { connection } => {
                if rows.len() < 2 {
                    continue;
                }
                if let Connection::Matrix(fractions) = connection {
                    let input: Vec<_> = rows.iter().map(|r| [r.x, r.y.unwrap_or(0.)]).collect();
                    for [x, y] in
                        univariate_kernels::connect(&input, fractions, limits.max_prepared_rows)?
                    {
                        push([Some(x), Some(y)], vec![], None)?;
                    }
                } else {
                    for row in &rows {
                        push([Some(row.x), row.y], vec![], Some(row))?;
                    }
                }
            }
            UnivariateKind::Align => {
                for (point, padding) in &aligned.as_ref().unwrap()[group_index] {
                    push(
                        [Some(point[0]), Some(point[1])],
                        vec![StatValue {
                            field: StatField::AlignPadding,
                            value: Some(f64::from(*padding)),
                        }],
                        None,
                    )?;
                }
            }
        }
    }
    table.schema = OutputSchema::Statistical {
        version: SchemaVersion::new(3),
        fields,
    };
    table.rows = PreparedRows::Statistical(output.into());
    Ok((spec.grouping.clone(), spec.space.clone()))
}
