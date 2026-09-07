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
        StatParameters::Identity => validate_operation(&stat.operation, "chart.identity"),
        StatParameters::Bin(spec) => {
            validate_operation(&stat.operation, "chart.bin")?;
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
            if let StatSpace::Transformed(transform) = &spec.space {
                validate_operation(&transform.operation, "chart.affine")?;
                if !transform.factor.is_finite()
                    || transform.factor == 0.
                    || !transform.offset.is_finite()
                {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Affine statistic space requires a finite nonzero factor and finite offset.",
                    ));
                }
            }
            Ok(())
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
    let (id, timestamp) = match value {
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
pub(crate) fn number(row: RowView<'_>, value: &Numeric) -> Option<f64> {
    let value = match value {
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
    };
    value.filter(|v| v.is_finite())
}
pub(crate) fn group_value(row: RowView<'_>, group: &Grouping) -> Option<GroupValue> {
    match group {
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
    if data.len() > max_rows {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Source preparation row budget exceeded.",
        ));
    }
    Ok(Arc::new(PreparedTable {
        schema: OutputSchema::Source(data.schema().clone()),
        rows: PreparedRows::Source(
            data.rows()
                .map(|r| SourceRow {
                    key: r.key(),
                    ordinal: r.ordinal(),
                })
                .collect(),
        ),
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

pub(crate) struct StatRequest<'a> {
    pub stat: &'a Statistic,
    pub filters: &'a [SourceFilter],
    pub policy: InvalidPolicy,
    pub scope: &'a str,
}
pub(crate) fn run(
    input: Arc<PreparedTable>,
    data: &DatasetSnapshot,
    request: StatRequest<'_>,
    limits: CompileLimits,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<Arc<PreparedTable>> {
    let StatRequest {
        stat,
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
    let index: BTreeMap<_, _> =
        if !filters.is_empty() || matches!(stat.parameters, StatParameters::Bin(_)) {
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
                match number(source, &filter.value) {
                    None => {
                        counts.invalid_filter += 1;
                        if samples.len() < 32 {
                            samples.push(row.key);
                        }
                        keep = false;
                        break;
                    }
                    Some(value)
                        if filter.minimum.is_some_and(|min| value < min)
                            || filter.maximum.is_some_and(|max| value > max) =>
                    {
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
    let (grouping, space) = match &stat.parameters {
        StatParameters::Identity => (Grouping::All, StatSpace::Data),
        StatParameters::Bin(spec) => {
            let PreparedRows::Source(rows) = &table.rows else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Bin inputs must be source observations, not generated bins.",
                ));
            };
            let input_space = numeric_space(data, &spec.input)?;
            validate_group(data, &spec.grouping)?;
            table.space = match &spec.space {
                StatSpace::Data => input_space,
                StatSpace::Transformed(transform) => ValueSpace::Transformed {
                    input: Box::new(input_space),
                    transform: transform.clone(),
                },
            };
            let n = spec.edges.len() - 1;
            let mut groups: Vec<(GroupValue, Vec<Vec<RowKey>>)> = Vec::new();
            let mut by_group = BTreeMap::new();
            if spec.grouping == Grouping::All {
                add_group(GroupValue::All, n, &mut groups, &mut by_group, limits)?;
            }
            let mut samples = Vec::new();
            for row in rows.iter() {
                let source = index[&row.key];
                let Some(group) = group_value(source, &spec.grouping) else {
                    counts.invalid_stat += 1;
                    if samples.len() < 32 {
                        samples.push(row.key);
                    }
                    continue;
                };
                let group_index = if let Some(i) = by_group.get(&group) {
                    *i
                } else {
                    add_group(group, n, &mut groups, &mut by_group, limits)?
                };
                let value = number(source, &spec.input).and_then(|v| match &spec.space {
                    StatSpace::Data => Some(v),
                    StatSpace::Transformed(t) => {
                        let v = t.factor * v + t.offset;
                        v.is_finite().then_some(v)
                    }
                });
                let Some(value) = value else {
                    counts.invalid_stat += 1;
                    if samples.len() < 32 {
                        samples.push(row.key);
                    }
                    continue;
                };
                if value < spec.edges[0] {
                    counts.below += 1;
                    continue;
                }
                if value > spec.edges[n] {
                    counts.above += 1;
                    continue;
                }
                let bin = (spec.edges.partition_point(|edge| *edge <= value) - 1).min(n - 1);
                groups[group_index].1[bin].push(row.key);
            }
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
                counts.below + counts.above,
                vec![],
                format!(
                    "Bin stat excluded {} below and {} above explicit edges.",
                    counts.below, counts.above
                ),
                diagnostics,
            )?;
            // For categorical fields, retain the store's explicit first-seen category history.
            if let Grouping::Field(field) = &spec.grouping
                && let Some(order) = data.categories(*field)
            {
                groups.sort_by_key(|(g, _)| match g {
                    GroupValue::Text(label) => {
                        order.iter().position(|s| s == label).unwrap_or(usize::MAX)
                    }
                    _ => usize::MAX,
                });
            }
            let mut output = Vec::new();
            for (group, bins) in groups {
                for (i, members) in bins.into_iter().enumerate() {
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
                        start: spec.edges[i],
                        end: spec.edges[i + 1],
                        count,
                        group: group.clone(),
                        target,
                    });
                }
            }
            table.rows = PreparedRows::Binned(output.into());
            table.schema = OutputSchema::Binned {
                version: SchemaVersion::new(1),
                fields: vec![
                    GeneratedField {
                        field: BinField::Start,
                        kind: GeneratedKind::Float64,
                    },
                    GeneratedField {
                        field: BinField::End,
                        kind: GeneratedKind::Float64,
                    },
                    GeneratedField {
                        field: BinField::Midpoint,
                        kind: GeneratedKind::Float64,
                    },
                    GeneratedField {
                        field: BinField::Count,
                        kind: GeneratedKind::UInt64,
                    },
                ],
            };
            (spec.grouping.clone(), spec.space.clone())
        }
    };
    counts.output = table.rows.len();
    table.operations.push(OperationRecord {
        operation: stat.operation.clone(),
        parameters: stat.parameters.clone(),
        input: table.input,
        filters: filters.to_vec(),
        grouping,
        space,
        counts,
    });
    Ok(Arc::new(table))
}
fn add_group(
    group: GroupValue,
    n: usize,
    groups: &mut Vec<(GroupValue, Vec<Vec<RowKey>>)>,
    index: &mut BTreeMap<GroupValue, usize>,
    limits: CompileLimits,
) -> ChartResult<usize> {
    if groups.len() >= limits.max_groups
        || groups
            .len()
            .checked_add(1)
            .and_then(|g| g.checked_mul(n))
            .is_none_or(|rows| rows > limits.max_prepared_rows)
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Bin group/output row budget exceeded.",
        ));
    }
    let i = groups.len();
    index.insert(group.clone(), i);
    groups.push((group, vec![vec![]; n]));
    Ok(i)
}
