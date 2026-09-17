use super::*;
use crate::data::{DatasetSnapshot, FieldKind, InvalidPolicy, RowView, ValueRef};
use crate::provenance::Target;
use crate::{
    AggregateId, ChartResult, Diagnostic, DiagnosticCode, FieldId, RowKey, SchemaVersion, Severity,
};
use std::{collections::BTreeMap, sync::Arc};

pub(crate) fn validate_operation(operation: &OperationRef, expected: &str) -> ChartResult<()> {
    if operation.id != expected || operation.version != crate::Revision::new(1) {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            format!(
                "Operation {} version {} does not resolve to {expected} version 1; arbitrary/native-only operations cannot be executed by the portable compiler.",
                operation.id,
                operation.version.get()
            ),
        ));
    }
    Ok(())
}
pub(crate) fn validate_stat(stat: &Statistic, limits: CompileLimits) -> ChartResult<()> {
    match &stat.parameters {
        StatParameters::Distribution(_)
        | StatParameters::Spatial(_)
        | StatParameters::Model(_)
        | StatParameters::Univariate(_)
        | StatParameters::AutoBin(_)
        | StatParameters::Count(_)
        | StatParameters::Summary(_)
        | StatParameters::Ols(_) => super::statistics::validate(stat, limits),
        StatParameters::Custom(p) => extensions::validate_parameters(p, limits),
        StatParameters::Identity => validate_operation(&stat.operation, "chart.identity"),
        StatParameters::Bin(spec) => {
            validate_operation(&stat.operation, "chart.bin")?;
            if let Some(options) = &spec.ggplot {
                options.validate()?;
            }
            if spec.edges.len() > limits.max_edges {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Bin edge budget exceeded.",
                ));
            }
            if spec.edges.len() < 2
                || spec.edges.iter().any(|v| !v.is_finite())
                || spec.edges.windows(2).any(|v| v[0] >= v[1])
            {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Explicit bin edges must be finite, strictly increasing and contain at least two values.",
                ));
            }
            super::statistics::validate_space(&spec.space)
        }
    }
}
pub(crate) fn validate_filters(filters: &[SourceFilter], limits: CompileLimits) -> ChartResult<()> {
    if filters.len() > limits.max_filters {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Source filter count budget exceeded.",
        ));
    }
    for filter in filters {
        if filter.minimum.is_some_and(|v| !v.is_finite())
            || filter.maximum.is_some_and(|v| !v.is_finite())
            || matches!((filter.minimum,filter.maximum),(Some(a),Some(b)) if a>b)
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Source filter bounds must be finite and ordered.",
            ));
        }
    }
    Ok(())
}
pub(crate) fn numeric_space(data: &DatasetSnapshot, value: &Numeric) -> ChartResult<ValueSpace> {
    if let Numeric::Scaled { input, scale, .. } = value {
        let mut cursor = value;
        let mut depth = 0;
        while let Numeric::Scaled { input, .. } = cursor {
            depth += 1;
            cursor = input;
        }
        if depth > 64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Numeric scale-stage nesting exceeds 64 operations.",
            ));
        }
        scale.validate()?;
        let input = numeric_space(data, input)?;
        scale.validate_input(&input)?;
        if scale.timestamp.is_some() && scale.outside == ScaleOob::Keep {
            // Censor/squish are bounded by the validated integer limits. Keeping
            // observations requires a checked pass before Option-valued evaluation
            // could otherwise confuse unrepresentable timestamps with missing data.
            for row in data.rows() {
                if let Some((value, origin)) = timestamp_value(row, value)
                    && (i128::from(value) - i128::from(origin)).unsigned_abs() > 1_u128 << 53
                {
                    return Err(error(
                        DiagnosticCode::PrecisionLoss,
                        "Kept timestamp exceeds exact origin-relative precision.",
                    ));
                }
            }
        }
        return Ok(scale.space(input));
    }
    let (id, timestamp) = match value {
        Numeric::Expression(expr) => {
            if expr
                .nodes
                .len()
                .checked_mul(data.len().max(1))
                .is_none_or(|n| n > ExpressionLimits::default().max_cells)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Source expression population budget exceeded.",
                ));
            }
            if expr.validate(ExpressionLimits::default(), |r| {
                numeric_space(data, &r.numeric())?;
                Ok(ExpressionType::Number)
            })? != ExpressionType::Number
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Source numeric mapping requires a numeric expression.",
                ));
            }
            return Ok(ValueSpace::Data);
        }
        Numeric::Scaled { .. } => unreachable!("handled above"),
        Numeric::Literal(v) => {
            return if v.is_finite() {
                Ok(ValueSpace::Data)
            } else {
                Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Literal mappings must be finite.",
                ))
            };
        }
        Numeric::Category(id) => {
            return Err(field_error(
                *id,
                "Categorical encodings cannot be used as numeric statistic/filter/size inputs.",
            ));
        }
        Numeric::Field(id) => (*id, None),
        Numeric::Timestamp { field, origin } => (*field, Some(*origin)),
    };
    let (_, field) = data.schema().field(id).ok_or_else(|| {
        field_error(
            id,
            "Numeric mapping field is absent; override incompatible inherited mappings.",
        )
    })?;
    match (&field.kind, timestamp) {
        (FieldKind::Float64 | FieldKind::Int64 | FieldKind::UInt64, None) => Ok(ValueSpace::Data),
        (FieldKind::Timestamp(representation), Some(origin)) => Ok(ValueSpace::Timestamp {
            representation: representation.clone(),
            origin,
        }),
        _ => Err(field_error(
            id,
            "Numeric mapping type is incompatible; timestamps require an explicit integer origin.",
        )),
    }
}
pub(crate) fn validate_group(data: &DatasetSnapshot, group: &Grouping) -> ChartResult<()> {
    if let Grouping::Interaction(fields) = group {
        if fields.is_empty()
            || fields.len() > 64
            || fields
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != fields.len()
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Group interactions require 1–64 distinct source fields.",
            ));
        }
        for field in fields {
            let (_, schema) = data
                .schema()
                .field(*field)
                .ok_or_else(|| field_error(*field, "Grouping field is absent."))?;
            if !matches!(schema.kind, FieldKind::Float64 | FieldKind::Timestamp(_)) {
                validate_group(data, &Grouping::Field(*field))?;
            }
        }
    }
    if let Grouping::Field(id) = group {
        let (_, field) = data
            .schema()
            .field(*id)
            .ok_or_else(|| field_error(*id, "Grouping field is absent."))?;
        if !matches!(
            field.kind,
            FieldKind::Categorical
                | FieldKind::Utf8
                | FieldKind::Int64
                | FieldKind::UInt64
                | FieldKind::Boolean
        ) {
            return Err(field_error(
                *id,
                "Grouping requires exact categorical, string, integer or boolean values.",
            ));
        }
    }
    Ok(())
}
fn field_error(id: FieldId, message: &str) -> Diagnostic {
    let mut e = error(DiagnosticCode::SchemaConflict, message);
    e.context.field = Some(id);
    e
}
pub(super) fn timestamp_value(row: RowView<'_>, value: &Numeric) -> Option<(i64, i64)> {
    match value {
        Numeric::Timestamp { field, origin } => match row.value(*field)? {
            ValueRef::Timestamp(value) => Some((value, *origin)),
            _ => None,
        },
        Numeric::Scaled { input, scale, .. } => {
            let time = scale.timestamp.as_ref()?;
            let (value, origin) = timestamp_value(row, input)?;
            if time.origin != origin {
                return None;
            }
            Some((
                if scale.function_limits.is_some() {
                    value
                } else {
                    time.project(value, scale.outside)?
                },
                origin,
            ))
        }
        _ => None,
    }
}
pub(crate) fn number(row: RowView<'_>, value: &Numeric) -> Option<f64> {
    raw_number(row, value).filter(|v| v.is_finite())
}
// Nonpositional reference mappings retain source IEEE values until scale evaluation.
pub(crate) fn raw_number(row: RowView<'_>, value: &Numeric) -> Option<f64> {
    match value {
        Numeric::Expression(expr) => expr
            .evaluate(
                1,
                ExpressionLimits::default(),
                |_| Ok(ExpressionType::Number),
                |r, _| {
                    number(row, &r.numeric()).map_or(
                        ExpressionValue::Missing(ExpressionType::Number),
                        ExpressionValue::Number,
                    )
                },
            )
            .ok()
            .and_then(|v| v[0].number()),
        Numeric::Scaled {
            samples: Some(samples),
            ..
        } => samples.get(&row.key()).map(|v| v.0).filter(|v| !v.is_nan()),
        Numeric::Scaled { scale, .. } if scale.timestamp.is_some() => timestamp_value(row, value)
            .and_then(|(value, origin)| {
                let relative = i128::from(value) - i128::from(origin);
                let value = (relative.unsigned_abs() <= 1_u128 << 53).then_some(relative as f64)?;
                if scale.function_limits.is_some() {
                    scale.project(value)
                } else {
                    Some(value)
                }
            }),
        Numeric::Scaled { input, scale, .. } => {
            if let Some(bins) = &scale.binned {
                bins.project_source(raw_number(row, input))
            } else {
                scale.project_optional(raw_number(row, input))
            }
        }
        Numeric::Category(_) => None,
        Numeric::Literal(v) => Some(*v),
        Numeric::Field(id) => match row.value(*id)? {
            ValueRef::Float64(v) => Some(v),
            ValueRef::Int64(v) if v.unsigned_abs() <= 1_u64 << 53 => Some(v as f64),
            ValueRef::UInt64(v) if v <= 1_u64 << 53 => Some(v as f64),
            _ => None,
        },
        Numeric::Timestamp { field, origin } => match row.value(*field)? {
            ValueRef::Timestamp(v) => v
                .checked_sub(*origin)
                .filter(|v| v.unsigned_abs() <= 1_u64 << 53)
                .map(|v| v as f64),
            _ => None,
        },
    }
}
pub(crate) fn filter_matches(row: RowView<'_>, filter: &SourceFilter) -> Option<bool> {
    number(row, &filter.value).map(|v| {
        !filter.minimum.is_some_and(|min| v < min) && !filter.maximum.is_some_and(|max| v > max)
    })
}
pub(crate) fn group_value(row: RowView<'_>, group: &Grouping) -> Option<GroupValue> {
    match group {
        Grouping::Interaction(fields) => Some(GroupValue::Interaction(
            fields
                .iter()
                .map(|field| {
                    (
                        *field,
                        match row.value(*field) {
                            Some(ValueRef::Float64(v)) => {
                                GroupNumber::try_from(v).ok().map(GroupValue::Number)
                            }
                            Some(ValueRef::Timestamp(v)) => Some(GroupValue::Int(v)),
                            _ => group_value(row, &Grouping::Field(*field)),
                        }
                        .unwrap_or(GroupValue::Missing),
                    )
                })
                .collect(),
        )),
        Grouping::All => Some(GroupValue::All),
        Grouping::Field(id) => match row.value(*id)? {
            ValueRef::Category(v) | ValueRef::Utf8(v) => Some(GroupValue::Text(v.into())),
            ValueRef::Int64(v) => Some(GroupValue::Int(v)),
            ValueRef::UInt64(v) => Some(GroupValue::UInt(v)),
            ValueRef::Boolean(v) => Some(GroupValue::Boolean(v)),
            _ => None,
        },
    }
}
pub(crate) fn source_table(
    data: &DatasetSnapshot,
    max_rows: usize,
) -> ChartResult<Arc<PreparedTable>> {
    source_table_where(data, max_rows, |_| true)
}
pub(crate) fn source_table_where(
    data: &DatasetSnapshot,
    max_rows: usize,
    mut eligible: impl FnMut(RowView<'_>) -> bool,
) -> ChartResult<Arc<PreparedTable>> {
    let mut rows = Vec::new();
    for row in data.rows().filter(|row| eligible(*row)) {
        if rows.len() == max_rows {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Source preparation row budget exceeded.",
            ));
        }
        rows.push(SourceRow {
            key: row.key(),
            ordinal: row.ordinal(),
        });
    }
    Ok(Arc::new(PreparedTable {
        schema: OutputSchema::Source(data.schema().clone()),
        rows: PreparedRows::Source(rows.into()),
        input: data.version(),
        space: ValueSpace::Data,
        operations: vec![],
    }))
}

pub(crate) fn warning(
    policy: InvalidPolicy,
    count: usize,
    samples: Vec<RowKey>,
    message: impl Into<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<()> {
    if count == 0 {
        return Ok(());
    }
    let mut e = error(DiagnosticCode::NumericalDomain, message);
    e.context.affected_rows = count as u64;
    e.context.row_samples = samples;
    if policy == InvalidPolicy::Strict {
        return Err(e);
    }
    e.severity = Severity::Warning;
    diagnostics.push(e);
    Ok(())
}

#[derive(Clone, Copy)]
pub(crate) struct StatRequest<'a> {
    pub stat: &'a Statistic,
    pub population: StatScope,
    pub panel: Option<&'a PanelKey>,
    pub filters: &'a [SourceFilter],
    pub policy: InvalidPolicy,
    pub scope: &'a str,
}
pub(crate) fn run(
    extensions: &ExtensionRegistry,
    bin_cache: &mut super::incremental_bins::BinCache,
    input: Arc<PreparedTable>,
    data: &DatasetSnapshot,
    request: StatRequest<'_>,
    limits: CompileLimits,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<Arc<PreparedTable>> {
    let StatRequest {
        stat,
        population,
        panel,
        filters,
        policy,
        scope,
    } = request;
    validate_stat(stat, limits)?;
    validate_filters(filters, limits)?;
    let mut table = (*input).clone();
    let mut counts = PopulationCounts {
        input: table.rows.len(),
        ..Default::default()
    };
    let index: BTreeMap<_, _> = if !filters.is_empty() {
        data.rows().map(|r| (r.key(), r)).collect()
    } else {
        BTreeMap::new()
    };
    if !filters.is_empty() {
        let PreparedRows::Source(rows) = &table.rows else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Source filters cannot be applied to generated rows; filter observations before the statistic.",
            ));
        };
        for filter in filters {
            numeric_space(data, &filter.value)?;
        }
        let mut kept = Vec::new();
        let mut samples = Vec::new();
        for row in rows.iter() {
            let source = index[&row.key];
            let mut keep = true;
            for filter in filters {
                match filter_matches(source, filter) {
                    None => {
                        counts.invalid_filter += 1;
                        if samples.len() < 32 {
                            samples.push(row.key);
                        }
                        keep = false;
                        break;
                    }
                    Some(false) => {
                        counts.filtered += 1;
                        keep = false;
                        break;
                    }
                    _ => {}
                }
            }
            if keep {
                kept.push(row.clone());
            }
        }
        warning(
            policy,
            counts.invalid_filter,
            samples,
            format!(
                "Source filters excluded {} rows with null, non-finite or imprecise required values.",
                counts.invalid_filter
            ),
            diagnostics,
        )?;
        table.rows = PreparedRows::Source(kept.into());
    }
    let resolved = if let StatParameters::AutoBin(spec) = &stat.parameters {
        Some(StatParameters::Bin(super::statistics::automatic(
            spec, &table, data,
        )?))
    } else {
        None
    };
    let (grouping, space) = match resolved.as_ref().unwrap_or(&stat.parameters) {
        StatParameters::AutoBin(_) => unreachable!("resolved above"),
        StatParameters::Distribution(_)
        | StatParameters::Spatial(_)
        | StatParameters::Model(_)
        | StatParameters::Univariate(_)
        | StatParameters::Count(_)
        | StatParameters::Summary(_)
        | StatParameters::Ols(_) => super::statistics::run(
            &mut table,
            data,
            stat,
            policy,
            scope,
            limits,
            &mut counts,
            diagnostics,
            extensions,
        )?,
        StatParameters::Custom(p) => {
            super::extensions::run_custom(
                extensions,
                &mut table,
                data,
                &request,
                limits,
                &mut counts,
                diagnostics,
            )?;
            (p.grouping.clone(), p.space.clone())
        }
        StatParameters::Identity => (Grouping::All, StatSpace::Data),
        StatParameters::Bin(spec) => {
            let PreparedRows::Source(rows) = &table.rows else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Bin inputs must be source observations, not generated bins.",
                ));
            };
            if spec.edges.len() > limits.max_edges {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Resolved bin edge budget exceeded.",
                ));
            }
            let input_space = numeric_space(data, &spec.input)?;
            validate_group(data, &spec.grouping)?;
            table.space = match &spec.space {
                StatSpace::Data => input_space,
                StatSpace::Transformed(transform) => ValueSpace::Transformed {
                    input: Box::new(input_space),
                    transform: transform.clone(),
                },
            };
            let accumulated = bin_cache.run(
                data,
                rows,
                spec,
                scope,
                filters.is_empty()
                    && input.operations.is_empty()
                    && spec.ggplot.as_ref().is_none_or(|o| o.weight.is_none())
                    && matches!(stat.parameters, StatParameters::Bin(_)),
                limits,
            )?;
            let (invalid, below, above) = accumulated.counts();
            counts.invalid_stat = invalid;
            counts.below = below;
            counts.above = above;
            let samples = accumulated.invalid_samples(rows);
            let groups = accumulated.groups(data, spec);
            bin_cache.restore(scope, accumulated);
            warning(
                policy,
                counts.invalid_stat,
                samples,
                format!(
                    "Bin stat excluded {} rows with invalid required values/groups.",
                    counts.invalid_stat
                ),
                diagnostics,
            )?;
            warning(
                if spec.outliers == OutlierPolicy::Error {
                    InvalidPolicy::Strict
                } else {
                    policy
                },
                if spec.outliers == OutlierPolicy::Overflow {
                    0
                } else {
                    counts.below + counts.above
                },
                vec![],
                format!(
                    "Bin stat excluded {} below and {} above explicit edges.",
                    counts.below, counts.above
                ),
                diagnostics,
            )?;
            let weight = spec.ggplot.as_ref().and_then(|o| o.weight.as_ref());
            if let Some(weight) = weight {
                numeric_space(data, weight)?;
            }
            let weight_rows: BTreeMap<_, _> = if weight.is_some() {
                data.rows().map(|r| (r.key(), r)).collect()
            } else {
                BTreeMap::new()
            };
            let mut output = Vec::new();
            for (group, bins) in groups {
                let generated = if spec.ggplot.is_some() {
                    let counts = bins
                        .iter()
                        .map(|members| {
                            if let Some(weight) = weight {
                                super::statistics::sum(members.iter().map(|key| {
                                    raw_number(weight_rows[key], weight)
                                        .filter(|v| !v.is_nan())
                                        .unwrap_or(0.)
                                }))
                            } else {
                                Ok(members.len() as f64)
                            }
                        })
                        .collect::<ChartResult<Vec<_>>>()?;
                    Some(super::ggplot_stats::statistics(&counts, &spec.edges)?)
                } else {
                    None
                };
                for (i, mut members) in bins.into_iter().enumerate() {
                    members.sort_unstable();
                    let count = members.len() as u64;
                    let target = Target::Aggregate {
                        id: AggregateId::new(i as u64),
                        group: format!(
                            "{scope}/{group:?}/{:016x}:{:016x}",
                            spec.edges[i].to_bits(),
                            spec.edges[i + 1].to_bits()
                        ),
                        input: table.input,
                        members: members.into(),
                    };
                    output.push(BinnedRow {
                        statistics: generated.as_ref().map(|v| v[i].clone()),
                        start: spec.edges[i],
                        end: spec.edges[i + 1],
                        count,
                        group: group.clone(),
                        target,
                    });
                }
            }
            if spec.ggplot.as_ref().is_some_and(|o| o.pad) {
                let n = spec.edges.len() - 1;
                if output.len().saturating_add(output.len() / n * 2) > limits.max_prepared_rows {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Padded bin output exceeds row budget.",
                    ));
                }
                let mut padded = Vec::with_capacity(output.len() + output.len() / n * 2);
                for chunk in output.chunks(n) {
                    let make = |row: &BinnedRow, start: f64, end: f64| -> ChartResult<BinnedRow> {
                        if !start.is_finite() || !end.is_finite() || start >= end {
                            return Err(error(
                                DiagnosticCode::PrecisionLoss,
                                "Padded bin edge is not representable.",
                            ));
                        }
                        let Target::Aggregate { input, .. } = &row.target else {
                            unreachable!()
                        };
                        Ok(BinnedRow {
                            start,
                            end,
                            count: 0,
                            group: row.group.clone(),
                            statistics: row.statistics.as_ref().map(|s| BinStatistics {
                                count: 0.,
                                density: s.density.map(|_| 0.),
                                ncount: s.ncount.map(|_| 0.),
                                ndensity: s.ndensity.map(|_| 0.),
                            }),
                            target: Target::Aggregate {
                                id: AggregateId::new(u64::MAX),
                                group: format!(
                                    "{scope}/{:?}/{:016x}:{:016x}",
                                    row.group,
                                    start.to_bits(),
                                    end.to_bits()
                                ),
                                input: *input,
                                members: Arc::from([]),
                            },
                        })
                    };
                    let first = &chunk[0];
                    let last = &chunk[n - 1];
                    padded.push(make(
                        first,
                        first.start - (first.end - first.start),
                        first.start,
                    )?);
                    padded.extend_from_slice(chunk);
                    padded.push(make(last, last.end, last.end + (last.end - last.start))?);
                }
                output = padded;
            }
            table.rows = PreparedRows::Binned(output.into());
            table.schema = OutputSchema::Binned {
                version: SchemaVersion::new(1),
                fields: vec![
                    GeneratedField {
                        nullable: false,
                        field: BinField::Start,
                        kind: GeneratedKind::Float64,
                    },
                    GeneratedField {
                        nullable: false,
                        field: BinField::End,
                        kind: GeneratedKind::Float64,
                    },
                    GeneratedField {
                        nullable: false,
                        field: BinField::Midpoint,
                        kind: GeneratedKind::Float64,
                    },
                    GeneratedField {
                        nullable: false,
                        field: BinField::Count,
                        kind: if spec.ggplot.is_some() {
                            GeneratedKind::Float64
                        } else {
                            GeneratedKind::UInt64
                        },
                    },
                ],
            };
            if spec.ggplot.is_some()
                && let OutputSchema::Binned { version, fields } = &mut table.schema
            {
                *version = SchemaVersion::new(2);
                fields.extend(
                    [
                        BinField::Width,
                        BinField::Density,
                        BinField::NCount,
                        BinField::NDensity,
                    ]
                    .into_iter()
                    .map(|field| GeneratedField {
                        nullable: !matches!(field, BinField::Width),
                        field,
                        kind: GeneratedKind::Float64,
                    }),
                );
            }
            (spec.grouping.clone(), spec.space.clone())
        }
    };
    counts.output = table.rows.len();
    table.operations.push(OperationRecord {
        scope: population,
        panel: panel.cloned(),
        operation: stat.operation.clone(),
        parameters: stat.parameters.clone(),
        input: table.input,
        filters: filters.to_vec(),
        grouping,
        space,
        incremental: stat.incremental_capabilities(),
        counts,
    });
    Ok(Arc::new(table))
}
