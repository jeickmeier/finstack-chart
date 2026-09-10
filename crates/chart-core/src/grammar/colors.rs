use super::compiler::EncodedRow;
use super::*;
use crate::data::{DatasetSnapshot, ValueRef};
use crate::scales::{ColorLegend, ColorScale};
use crate::{ChartResult, DiagnosticCode, FieldId, ScaleId};
use std::collections::{BTreeMap, BTreeSet};

/// Color reads are explicitly bound to source, generated values, or stable groups.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ColorInput {
    /// Source category/UTF-8/integer/bool field.
    Category(FieldId),
    /// Finite source numeric mapping, or a literal at any prepared stage.
    Numeric(Numeric),
    /// Prepared stable group identity.
    Group,
    /// One exact field component of an inferred statistical interaction.
    GroupField(FieldId),
    /// Generated numeric statistical field.
    Statistical(StatField),
}
/// Named portable color mapping; it never trains positional domains.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ColorEncoding {
    /// Human-readable legend title; absent uses a generic color label, empty omits the title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Shared color scale identity.
    pub id: ScaleId,
    /// Stage-specific value.
    pub input: ColorInput,
    /// Exact palette, domain, null and outside policy.
    pub scale: ColorScale<crate::color::Paint>,
}

pub(super) fn preflight(
    encoding: &ColorEncoding,
    data: &DatasetSnapshot,
    source: bool,
    fields: Option<&[StatColumn]>,
    limits: CompileLimits,
    registry: &ExtensionRegistry,
) -> ChartResult<()> {
    if encoding.title.as_ref().is_some_and(|t| t.len() > 4096) {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Color legend title exceeds 4096 bytes.",
        ));
    }
    encoding.scale.validate_with_registry(registry)?;
    let categorical = matches!(
        encoding.input,
        ColorInput::Category(_) | ColorInput::Group | ColorInput::GroupField(_)
    );
    if categorical != encoding.scale.is_categorical()
        && !(matches!(encoding.scale, ColorScale::Mapped { .. })
            && encoding.scale.is_categorical()
            && !categorical)
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Color input and scale family disagree.",
        ));
    }
    let (palette, domain) = match &encoding.scale {
        ColorScale::Discrete {
            domain, palette, ..
        } => (palette.len(), domain.as_ref().map_or(0, Vec::len)),
        ColorScale::Continuous { palette, .. } => (palette.len(), 0),
        ColorScale::Mapped { scale, .. } => (scale.guide_entries(), 0),
    };
    if palette > limits.max_groups || domain > limits.max_groups {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Color catalog/palette exceeds category budget.",
        ));
    }
    match &encoding.input {
        ColorInput::Category(field) if source => {
            if matches!(encoding.scale, ColorScale::Mapped { .. }) {
                validate_key(data, *field)?;
            } else {
                super::stats::validate_group(data, &Grouping::Field(*field))?;
            }
        }
        ColorInput::Numeric(value) if source || matches!(value, Numeric::Literal(_)) => {
            super::stats::numeric_space(data, value)?;
        }
        ColorInput::Group => {}
        ColorInput::GroupField(field) => {
            super::stats::validate_group(data, &Grouping::Field(*field))?;
        }
        ColorInput::Statistical(field)
            if fields.is_some_and(|fields| {
                fields.iter().any(|c| {
                    &c.field == field && !matches!(c.space, ValueSpace::Categorical { .. })
                })
            }) => {}
        _ => {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Color input is absent or incompatible with its source/generated output stage.",
            ));
        }
    }
    Ok(())
}

pub(super) struct ColorContext<'a> {
    pub limits: CompileLimits,
    pub registry: &'a ExtensionRegistry,
    pub shared: Option<&'a [String]>,
    pub samples: Option<&'a crate::scales::ScalePopulation>,
}
pub(super) fn apply(
    layer: &Layer,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    context: ColorContext<'_>,
) -> ChartResult<Option<ColorLegend>> {
    let ColorContext {
        limits,
        registry,
        shared,
        samples,
    } = context;
    let Some(encoding) = &layer.color else {
        return Ok(None);
    };
    let fields = match &table.schema {
        OutputSchema::Statistical { fields, .. } | OutputSchema::Custom { fields, .. } => {
            Some(fields.as_slice())
        }
        _ => None,
    };
    preflight(
        encoding,
        data,
        matches!(table.rows, PreparedRows::Source(_)),
        fields,
        limits,
        registry,
    )?;
    let AestheticInputs {
        labels,
        categories,
        keys,
        values,
    } = read_inputs(&encoding.input, data, table, rows, limits, shared)?;
    let trained;
    let scale = if let ColorScale::Mapped { scale, missing } = &encoding.scale {
        trained = ColorScale::Mapped {
            scale: scale.trained_population(samples)?,
            missing: *missing,
        };
        &trained
    } else {
        &encoding.scale
    };
    let prepared_scale = scale.prepare_with_registry(registry)?;
    let categorical_map = if let ColorScale::Discrete {
        domain,
        palette,
        missing,
    } = scale
    {
        Some(crate::scales::OrdinalScale::new(
            crate::scales::OrdinalSpec {
                domain: domain.clone().unwrap_or_else(|| labels.clone()),
                range: palette.clone(),
                unknown: crate::scales::OrdinalUnknown::Explicit(Some(*missing)),
            },
        ))
    } else {
        None
    };
    for (i, row) in rows.iter_mut().enumerate() {
        row.color = Some(if let Some(catalog) = &categorical_map {
            categories[i]
                .as_ref()
                .and_then(|label| catalog.map(label))
                .copied()
                .unwrap_or_else(|| match catalog.spec().unknown {
                    crate::scales::OrdinalUnknown::Explicit(Some(c)) => c,
                    _ => unreachable!(),
                })
        } else if matches!(
            encoding.input,
            ColorInput::Category(_) | ColorInput::Group | ColorInput::GroupField(_)
        ) {
            let key = keys[i].clone().or_else(|| {
                categories[i]
                    .as_ref()
                    .map(|s| crate::scales::ScaleKey::Text(s.clone()))
            });
            prepared_scale.category_paint(key.as_ref())?
        } else {
            prepared_scale.numeric_paint(values[i])?
        });
    }
    let mut legend = prepared_scale.legend(encoding.id, &labels)?;
    legend.title.clone_from(&encoding.title);
    Ok(Some(legend))
}

pub(super) struct AestheticInputs {
    pub labels: Vec<String>,
    pub categories: Vec<Option<String>>,
    pub keys: Vec<Option<crate::scales::ScaleKey>>,
    pub values: Vec<Option<f64>>,
}
pub(super) fn read_inputs(
    input: &ColorInput,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &[EncodedRow],
    limits: CompileLimits,
    shared: Option<&[String]>,
) -> ChartResult<AestheticInputs> {
    let mut labels = shared.unwrap_or_default().to_vec();
    let mut categories = vec![None; rows.len()];
    let mut keys = vec![None; rows.len()];
    let mut values = vec![None; rows.len()];
    match input {
        ColorInput::Group => {
            for (i, row) in rows.iter().enumerate() {
                categories[i] = row.group.as_ref().map(super::statistics::group_label);
            }
        }
        ColorInput::GroupField(field) => {
            for (i, row) in rows.iter().enumerate() {
                categories[i] = row
                    .group
                    .as_ref()
                    .and_then(|g| g.component(*field))
                    .filter(|g| **g != GroupValue::Missing)
                    .map(GroupValue::label);
            }
        }
        ColorInput::Category(field) => {
            if !matches!(table.rows, PreparedRows::Source(_)) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Source color fields require source rows.",
                ));
            }
            validate_key(data, *field)?;
            let catalog = data.categories(*field).unwrap_or_default();
            if catalog.len() > limits.max_groups {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Color catalog exceeds category budget.",
                ));
            }
            if shared.is_none() {
                labels = catalog.to_vec();
            }
            let source: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
            for (i, row) in rows.iter().enumerate() {
                let r = source[&row.key.expect("source row")];
                keys[i] = r.value(*field).map(scale_key);
                categories[i] = match r.value(*field) {
                    None => None,
                    Some(ValueRef::Category(s) | ValueRef::Utf8(s)) => Some(s.to_owned()),
                    _ => super::stats::group_value(r, &Grouping::Field(*field))
                        .map(|g| super::statistics::group_label(&g)),
                };
            }
        }
        ColorInput::Numeric(value @ Numeric::Literal(number)) => {
            super::stats::numeric_space(data, value)?;
            values.fill(Some(*number));
        }
        ColorInput::Numeric(value) => {
            if !matches!(table.rows, PreparedRows::Source(_)) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Source color mappings require source rows.",
                ));
            }
            super::stats::numeric_space(data, value)?;
            let source: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
            for (i, row) in rows.iter().enumerate() {
                values[i] = super::stats::number(source[&row.key.expect("source row")], value);
            }
        }
        ColorInput::Statistical(field) => {
            let (
                PreparedRows::Statistical(source),
                OutputSchema::Statistical { fields, .. } | OutputSchema::Custom { fields, .. },
            ) = (&table.rows, &table.schema)
            else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Generated color mapping requires statistical rows.",
                ));
            };
            if !fields
                .iter()
                .any(|c| &c.field == field && !matches!(c.space, ValueSpace::Categorical { .. }))
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Generated numeric color field is absent or categorical.",
                ));
            }
            for (i, row) in source.iter().enumerate() {
                values[i] = row.value(field);
            }
        }
    }
    let mut seen: BTreeSet<_> = labels.iter().cloned().collect();
    for label in categories.iter().flatten() {
        if seen.insert(label.clone()) {
            if labels.len() >= limits.max_groups {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Color catalog exceeds category budget.",
                ));
            }
            labels.push(label.clone());
        }
    }
    Ok(AestheticInputs {
        labels,
        categories,
        keys,
        values,
    })
}

/// Train shared automatic catalogs once, before assigning colors to any layer.
/// Explicit domains and single-layer automatic scales retain their existing behavior.
pub(super) fn shared_catalogs(
    definition: &ChartDefinition,
    source: &crate::data::StoreSnapshot,
    tables: &BTreeMap<crate::LayerId, &PreparedTable>,
    limits: CompileLimits,
) -> ChartResult<BTreeMap<ScaleId, Vec<String>>> {
    let mut counts = BTreeMap::<ScaleId, usize>::new();
    for layer in &definition.layers {
        if tables.contains_key(&layer.id)
            && let Some(ColorEncoding {
                id,
                scale: ColorScale::Discrete { domain: None, .. },
                ..
            }) = &layer.color
        {
            *counts.entry(*id).or_default() += 1;
        }
    }
    let mut catalogs = BTreeMap::<ScaleId, Vec<String>>::new();
    let mut seen = BTreeMap::<ScaleId, BTreeSet<String>>::new();
    for layer in &definition.layers {
        let Some(encoding) = &layer.color else {
            continue;
        };
        if counts.get(&encoding.id).copied().unwrap_or_default() < 2 {
            continue;
        }
        let Some(table) = tables.get(&layer.id) else {
            continue;
        };
        let data = source.dataset(table.input.dataset)?;
        let labels = catalogs.entry(encoding.id).or_default();
        let seen = seen.entry(encoding.id).or_default();
        let mut add = |label: String| -> ChartResult<()> {
            if seen.insert(label.clone()) {
                if labels.len() >= limits.max_groups {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Shared color catalog exceeds category budget.",
                    ));
                }
                labels.push(label);
            }
            Ok(())
        };
        match (&encoding.input, &table.rows) {
            (ColorInput::Category(field), PreparedRows::Source(rows)) => {
                for label in data.categories(*field).unwrap_or_default() {
                    add(label.clone())?;
                }
                let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
                for row in rows.iter() {
                    if let Some(group) =
                        super::stats::group_value(index[&row.key], &Grouping::Field(*field))
                    {
                        add(super::statistics::group_label(&group))?;
                    }
                }
            }
            (ColorInput::Group, PreparedRows::Source(rows)) => {
                let Mappings::Source(aes) = &layer.mappings else {
                    continue;
                };
                let field = aes
                    .group
                    .or_else(|| layer.inherit.then_some(definition.mappings.group).flatten());
                let grouping = aes
                    .grouping
                    .clone()
                    .unwrap_or_else(|| field.map_or(Grouping::All, Grouping::Field));
                let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
                for row in rows.iter() {
                    if let Some(group) = super::stats::group_value(index[&row.key], &grouping) {
                        add(super::statistics::group_label(&group))?;
                    }
                }
            }
            (ColorInput::Group, PreparedRows::Binned(rows)) => {
                for row in rows.iter() {
                    add(super::statistics::group_label(&row.group))?;
                }
            }
            (ColorInput::Group, PreparedRows::Statistical(rows)) => {
                for row in rows.iter() {
                    add(super::statistics::group_label(&row.group))?;
                }
            }
            (ColorInput::GroupField(field), PreparedRows::Binned(rows)) => {
                for row in rows.iter() {
                    if let Some(value) = row
                        .group
                        .component(*field)
                        .filter(|g| **g != GroupValue::Missing)
                    {
                        add(value.label())?;
                    }
                }
            }
            (ColorInput::GroupField(field), PreparedRows::Statistical(rows)) => {
                for row in rows.iter() {
                    if let Some(value) = row
                        .group
                        .component(*field)
                        .filter(|g| **g != GroupValue::Missing)
                    {
                        add(value.label())?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(catalogs)
}

fn scale_key(value: ValueRef<'_>) -> crate::scales::ScaleKey {
    use crate::{interpolate::Number, scales::ScaleKey};
    match value {
        ValueRef::Float64(v) => ScaleKey::Number(Number(v)),
        ValueRef::Int64(v) => ScaleKey::Integer(v),
        ValueRef::UInt64(v) => ScaleKey::Unsigned(v),
        ValueRef::Boolean(v) => ScaleKey::Boolean(v),
        ValueRef::Timestamp(v) => ScaleKey::Timestamp(v),
        ValueRef::Utf8(v) | ValueRef::Category(v) => ScaleKey::Text(v.to_owned()),
    }
}
/// Eligible post-stat values. Source table membership is used, never destination visibility.
pub(super) fn numeric_population(
    input: &ColorInput,
    data: &DatasetSnapshot,
    table: &PreparedTable,
) -> ChartResult<Vec<Option<crate::interpolate::Number>>> {
    use crate::interpolate::Number;
    match (input, &table.rows) {
        (ColorInput::Numeric(value @ Numeric::Literal(number)), rows) => {
            super::stats::numeric_space(data, value)?;
            Ok(vec![Some(Number(*number)); rows.len()])
        }
        (ColorInput::Numeric(value), PreparedRows::Source(rows)) => {
            super::stats::numeric_space(data, value)?;
            let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
            Ok(rows
                .iter()
                .map(|r| super::stats::number(index[&r.key], value).map(Number))
                .collect())
        }
        (ColorInput::Statistical(field), PreparedRows::Statistical(rows)) => {
            Ok(rows.iter().map(|r| r.value(field).map(Number)).collect())
        }
        _ => Err(error(
            DiagnosticCode::SchemaConflict,
            "Quantile training requires a numeric source or generated statistical field.",
        )),
    }
}
fn key_population(
    input: &ColorInput,
    layer: &Layer,
    definition: &ChartDefinition,
    data: &DatasetSnapshot,
    table: &PreparedTable,
) -> ChartResult<Vec<crate::scales::ScaleKey>> {
    use crate::scales::ScaleKey;
    if let ColorInput::Category(field) = input {
        let PreparedRows::Source(rows) = &table.rows else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Ordinal source key training requires source rows.",
            ));
        };
        validate_key(data, *field)?;
        let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
        return Ok(rows
            .iter()
            .filter_map(|r| index[&r.key].value(*field).map(scale_key))
            .collect());
    }
    if matches!(input, ColorInput::Numeric(_) | ColorInput::Statistical(_)) {
        return Ok(numeric_population(input, data, table)?
            .into_iter()
            .flatten()
            .map(ScaleKey::Number)
            .collect());
    }
    let key = |group: &GroupValue| match input {
        ColorInput::Group => Some(ScaleKey::Text(super::statistics::group_label(group))),
        ColorInput::GroupField(field) => group
            .component(*field)
            .filter(|g| **g != GroupValue::Missing)
            .map(|v| ScaleKey::Text(v.label())),
        _ => None,
    };
    Ok(match &table.rows {
        PreparedRows::Source(rows) => {
            let Mappings::Source(aes) = &layer.mappings else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Ordinal group training requires source mappings.",
                ));
            };
            let field = aes
                .group
                .or_else(|| layer.inherit.then_some(definition.mappings.group).flatten());
            let grouping = aes
                .grouping
                .clone()
                .unwrap_or_else(|| field.map_or(Grouping::All, Grouping::Field));
            let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
            rows.iter()
                .filter_map(|r| {
                    super::stats::group_value(index[&r.key], &grouping).and_then(|g| key(&g))
                })
                .collect()
        }
        PreparedRows::Binned(rows) => rows.iter().filter_map(|r| key(&r.group)).collect(),
        PreparedRows::Statistical(rows) => rows.iter().filter_map(|r| key(&r.group)).collect(),
    })
}
/// Freeze eligible populations across layers before mapping or viewport work.
pub(super) fn shared_samples<'a>(
    definition: &ChartDefinition,
    source: &crate::data::StoreSnapshot,
    tables: impl IntoIterator<Item = (&'a Layer, &'a PreparedTable)>,
    limits: CompileLimits,
) -> ChartResult<BTreeMap<ScaleId, crate::scales::ScalePopulation>> {
    use crate::scales::{ScaleFunctionSpec, ScalePopulation};
    let mut samples = BTreeMap::<ScaleId, ScalePopulation>::new();
    let mut seen_keys = BTreeMap::<ScaleId, BTreeSet<crate::scales::ScaleKey>>::new();
    // Broadcast/chart-scope tables repeat presentation, not their sample population.
    let mut chart_layers = BTreeSet::new();
    let tables: Vec<_> = tables.into_iter().collect();
    for (layer, table) in definition.layers.iter().flat_map(|layer| {
        tables
            .iter()
            .filter(move |(l, _)| l.id == layer.id)
            .map(move |(_, table)| (layer, *table))
    }) {
        if table.operations.iter().all(|op| op.panel.is_none()) && !chart_layers.insert(layer.id) {
            continue;
        }
        let mut mappings: Vec<_> = layer
            .numeric_scales
            .values()
            .map(|e| (e.id, &e.input, &e.scale))
            .collect();
        if let Some(ColorEncoding {
            id,
            input,
            scale: ColorScale::Mapped { scale, .. },
            ..
        }) = &layer.color
        {
            mappings.push((*id, input, scale));
        }
        for (id, input, scale) in mappings {
            if scale.training != crate::scales::ScaleTraining::Eligible {
                continue;
            }
            scale.validate_training()?;
            if matches!(scale.function, ScaleFunctionSpec::Ordinal(_)) {
                let values = key_population(
                    input,
                    layer,
                    definition,
                    source.dataset(table.input.dataset)?,
                    table,
                )?;
                let population = samples
                    .entry(id)
                    .or_insert_with(|| ScalePopulation::Keys(vec![]));
                let ScalePopulation::Keys(population) = population else {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Shared scale population kinds disagree.",
                    ));
                };
                let seen = seen_keys.entry(id).or_default();
                for value in values {
                    if seen.insert(value.clone()) {
                        if population.len() >= limits.max_groups.min(crate::interpolate::MAX_VALUES)
                        {
                            return Err(error(
                                DiagnosticCode::ResourceLimit,
                                "Shared ordinal catalog exceeds its key budget.",
                            ));
                        }
                        population.push(value);
                    }
                }
                continue;
            }
            let values = numeric_population(input, source.dataset(table.input.dataset)?, table)?;
            let population = samples
                .entry(id)
                .or_insert_with(|| ScalePopulation::Numbers(vec![]));
            let ScalePopulation::Numbers(population) = population else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Shared scale population kinds disagree.",
                ));
            };
            if population.len().saturating_add(values.len())
                > limits.max_prepared_rows.min(crate::interpolate::MAX_VALUES)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Shared quantile population exceeds its value budget.",
                ));
            }
            population.extend(values);
        }
    }
    Ok(samples)
}

fn validate_key(data: &DatasetSnapshot, field: FieldId) -> ChartResult<()> {
    data.schema().field(field).map(|_| ()).ok_or_else(|| {
        error(
            DiagnosticCode::SchemaConflict,
            "Mapped scale key field is absent.",
        )
    })
}
