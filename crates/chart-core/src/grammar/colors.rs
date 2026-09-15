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
/// Independent paint channels; the legacy color channel remains a shared fallback.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum PaintAesthetic {
    /// Interior paint.
    Fill,
    /// Outline and line paint.
    Stroke,
}

pub(super) fn encodings(layer: &Layer) -> impl Iterator<Item = &ColorEncoding> {
    layer.color.iter().chain(
        layer
            .paint_scales
            .iter()
            .filter(|(channel, _)| match channel {
                PaintAesthetic::Fill => layer.style.fill.is_none(),
                PaintAesthetic::Stroke => layer.style.stroke.is_none(),
            })
            .map(|(_, encoding)| encoding),
    )
}

/// Named portable color mapping; it never trains positional domains.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ColorEncoding {
    /// Reuse the ggplot default scale for this aesthetic when authoring later layers.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub automatic: bool,
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
                    &c.field == field
                        && !matches!(
                            c.space,
                            ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. }
                        )
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
    pub layer: crate::LayerId,
    pub limits: CompileLimits,
    pub registry: &'a ExtensionRegistry,
    pub shared: Option<&'a [String]>,
    pub samples: Option<&'a crate::scales::ScalePopulation>,
}
pub(super) fn apply(
    encoding: Option<&ColorEncoding>,
    channel: Option<PaintAesthetic>,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    context: ColorContext<'_>,
) -> ChartResult<Option<ColorLegend>> {
    let ColorContext {
        layer,
        limits,
        registry,
        shared,
        samples,
    } = context;
    let Some(encoding) = encoding else {
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
    } = read_inputs(
        &encoding.input,
        data,
        table,
        rows,
        limits,
        shared,
        matches!(&encoding.scale, ColorScale::Mapped { scale, .. } if scale.has_ggplot()),
    )?;
    let trained;
    let scale = if let ColorScale::Mapped { scale, missing } = &encoding.scale {
        trained = ColorScale::Mapped {
            scale: scale.trained_population(samples, registry)?,
            missing: *missing,
        };
        &trained
    } else {
        &encoding.scale
    };
    if let ColorScale::Mapped { scale, .. } = scale
        && scale.guide_entries() > limits.max_groups
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Trained color scale exceeds category budget.",
        ));
    }
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
    let mut legend = prepared_scale.legend(encoding.id, &labels)?;
    let source = match scale {
        ColorScale::Mapped { scale, .. } => layer_batch(scale, samples, layer, &encoding.input),
        _ => None,
    };
    let batch = sample_layer_batch(source, table, rows, &values, |inputs| {
        prepared_scale.palette_paints(inputs)
    })?;
    let missing_batch_paint = crate::scene::Color {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
    }
    .into();
    let suppress_missing = matches!(scale,ColorScale::Mapped{scale,..} if scale.preserves_palette_missing() || scale.missing_paint_is_na || matches!(scale.ggplot.as_deref(),Some(crate::scales::GgplotScalePolicy::Discrete{na_translate:false,..})) || matches!(scale.function, crate::scales::ScaleFunctionSpec::GgplotDiscreteIdentity(_)));
    for (i, row) in rows.iter_mut().enumerate() {
        if batch.as_ref().is_some_and(|batch| batch.values.is_none()) {
            continue;
        }
        let mut paint = Some(if let Some(batch) = &batch {
            batch.values.as_ref().unwrap()[batch.indices[i]].unwrap_or(missing_batch_paint)
        } else if let Some(catalog) = &categorical_map {
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
        if suppress_missing {
            let key = keys[i].clone().or_else(|| {
                categories[i]
                    .as_ref()
                    .map(|s| crate::scales::ScaleKey::Text(s.clone()))
            });
            let is_missing = batch.as_ref().map_or_else(
                || prepared_scale.missing_paint(values[i], key.as_ref()),
                |batch| batch.values.as_ref().unwrap()[batch.indices[i]].is_none(),
            );
            if is_missing {
                let aesthetic = match channel {
                    None => AfterScaleAesthetic::Color,
                    Some(PaintAesthetic::Fill) => AfterScaleAesthetic::Fill,
                    Some(PaintAesthetic::Stroke) => AfterScaleAesthetic::Stroke,
                };
                row.set_missing(aesthetic, true);
                paint = Some(
                    crate::scene::Color {
                        red: 0,
                        green: 0,
                        blue: 0,
                        alpha: 0,
                    }
                    .into(),
                );
            }
        }
        match channel {
            None => row.color = paint,
            Some(PaintAesthetic::Fill) => row.fill = paint,
            Some(PaintAesthetic::Stroke) => row.stroke = paint,
        }
    }
    if matches!(&encoding.scale, ColorScale::Mapped { scale, .. } if matches!(&scale.function, crate::scales::ScaleFunctionSpec::GgplotDiscreteIdentity(s) if !s.guide) || matches!(scale.guide.as_deref(),Some(crate::scales::GgplotScaleGuide::Hidden)))
    {
        return Ok(None);
    }
    if legend.entries.len().saturating_add(legend.colorbar.len()) > limits.max_groups {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Prepared color guide exceeds the category budget.",
        ));
    }
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
    reference: bool,
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
                let Some(r) = row.key.and_then(|key| source.get(&key)) else {
                    continue;
                };
                keys[i] = r.value(*field).map(scale_key);
                categories[i] = match r.value(*field) {
                    None => None,
                    Some(ValueRef::Category(s) | ValueRef::Utf8(s)) => Some(s.to_owned()),
                    _ => super::stats::group_value(*r, &Grouping::Field(*field))
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
                values[i] = row.key.and_then(|key| source.get(&key)).and_then(|r| {
                    if reference {
                        super::stats::raw_number(*r, value)
                    } else {
                        super::stats::number(*r, value)
                    }
                });
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
            if !fields.iter().any(|c| {
                &c.field == field
                    && !matches!(
                        c.space,
                        ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. }
                    )
            }) {
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
    for (layer, encoding) in definition
        .layers
        .iter()
        .flat_map(|l| encodings(l).map(move |e| (l, e)))
    {
        if tables.contains_key(&layer.id)
            && let ColorEncoding {
                id,
                scale: ColorScale::Discrete { domain: None, .. },
                ..
            } = encoding
        {
            *counts.entry(*id).or_default() += 1;
        }
    }
    let mut catalogs = BTreeMap::<ScaleId, Vec<String>>::new();
    let mut seen = BTreeMap::<ScaleId, BTreeSet<String>>::new();
    for (layer, encoding) in definition
        .layers
        .iter()
        .flat_map(|l| encodings(l).map(move |e| (l, e)))
    {
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
    reference: bool,
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
                .map(|r| {
                    if reference {
                        super::stats::raw_number(index[&r.key], value)
                    } else {
                        super::stats::number(index[&r.key], value)
                    }
                    .map(Number)
                })
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
    include_missing: bool,
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
            .filter_map(|r| {
                index[&r.key]
                    .value(*field)
                    .map(scale_key)
                    .or_else(|| include_missing.then_some(ScaleKey::Null))
            })
            .collect());
    }
    if matches!(input, ColorInput::Numeric(_) | ColorInput::Statistical(_)) {
        return Ok(numeric_population(input, data, table, include_missing)?
            .into_iter()
            .flatten()
            .map(ScaleKey::Number)
            .collect());
    }
    let key = |group: &GroupValue| {
        (match input {
            ColorInput::Group => Some(ScaleKey::Text(super::statistics::group_label(group))),
            ColorInput::GroupField(field) => group
                .component(*field)
                .filter(|g| **g != GroupValue::Missing)
                .map(|v| ScaleKey::Text(v.label())),
            _ => None,
        })
        .or_else(|| include_missing.then_some(ScaleKey::Null))
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
            .iter()
            .filter(|(a, _)| **a != NumericAesthetic::Alpha || layer.style.alpha.is_none())
            .map(|(_, e)| (e.id, &e.input, &e.scale))
            .chain(
                layer
                    .value_scales
                    .iter()
                    .filter(|(channel, _)| {
                        !layer.aesthetic_values.contains_key(channel)
                            && (**channel != ValueAesthetic::LineType
                                || layer.style.line_type.is_none())
                    })
                    .map(|(_, e)| (e.id, &e.input, &e.scale)),
            )
            .collect();
        for encoding in encodings(layer) {
            if let ColorScale::Mapped { scale, .. } = &encoding.scale {
                mappings.push((encoding.id, &encoding.input, scale));
            }
        }
        for (id, input, scale) in mappings {
            if scale.training != crate::scales::ScaleTraining::Eligible {
                continue;
            }
            scale.validate_training()?;
            if matches!(
                scale.function,
                ScaleFunctionSpec::Ordinal(_) | ScaleFunctionSpec::GgplotDiscreteIdentity(_)
            ) {
                let values = key_population(
                    input,
                    layer,
                    definition,
                    source.dataset(table.input.dataset)?,
                    table,
                    scale.has_ggplot(),
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
            let values = numeric_population(
                input,
                source.dataset(table.input.dataset)?,
                table,
                scale.has_ggplot(),
            )?;
            let population = samples
                .entry(id)
                .or_insert_with(|| ScalePopulation::Numbers {
                    values: vec![],
                    batches: vec![],
                    sources: vec![],
                });
            let ScalePopulation::Numbers {
                values: population,
                batches,
                sources,
            } = population
            else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Shared scale population kinds disagree.",
                ));
            };
            if population
                .len()
                .saturating_add(sources.iter().map(|s| s.rows.len()).sum::<usize>())
                .saturating_add(values.len())
                > limits.max_prepared_rows.min(crate::interpolate::MAX_VALUES)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Shared quantile population exceeds its value budget.",
                ));
            }
            if scale.ggplot_transform().is_some_and(|t| !t.is_pointwise()) {
                let index = sources
                    .iter()
                    .position(|batch| batch.layer == layer.id && batch.input == *input)
                    .unwrap_or_else(|| {
                        sources.push(crate::scales::ScaleLayerBatch {
                            layer: layer.id,
                            input: input.clone(),
                            rows: BTreeMap::new(),
                            generated_panels: BTreeSet::new(),
                        });
                        sources.len() - 1
                    });
                if let PreparedRows::Source(rows) = &table.rows {
                    let order = source
                        .dataset(table.input.dataset)?
                        .rows()
                        .enumerate()
                        .map(|(i, row)| (row.key(), i))
                        .collect::<BTreeMap<_, _>>();
                    for (row, value) in rows.iter().zip(&values) {
                        sources[index].rows.insert(
                            order[&row.key],
                            (crate::scales::ScaleRowIdentity::Source(row.key), *value),
                        );
                    }
                } else {
                    let panel = table.population_operation().and_then(|op| op.panel.clone());
                    if sources[index].generated_panels.insert(panel.clone()) {
                        let offset = sources[index].rows.len();
                        for (ordinal, value) in values.iter().enumerate() {
                            sources[index].rows.insert(
                                offset + ordinal,
                                (
                                    crate::scales::ScaleRowIdentity::Generated(
                                        panel.clone(),
                                        ordinal as u64,
                                    ),
                                    *value,
                                ),
                            );
                        }
                    }
                }
            } else {
                batches.push(values.len());
                population.extend(values);
            }
        }
    }
    for population in samples.values_mut() {
        if let ScalePopulation::Numbers {
            values,
            batches,
            sources,
        } = population
        {
            for source in sources {
                if values.len().saturating_add(source.rows.len())
                    > limits.max_prepared_rows.min(crate::interpolate::MAX_VALUES)
                {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Shared numeric population exceeds its value budget.",
                    ));
                }
                batches.push(source.rows.len());
                values.extend(source.rows.values().map(|(_, value)| *value));
            }
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

/// Retain the original layer vector when a transform couples observations across panels.
pub(super) fn layer_batch<'a>(
    scale: &crate::scales::MappedScaleSpec,
    samples: Option<&'a crate::scales::ScalePopulation>,
    layer: crate::LayerId,
    input: &ColorInput,
) -> Option<&'a crate::scales::ScaleLayerBatch> {
    if !scale.ggplot_transform().is_some_and(|t| !t.is_pointwise()) {
        return None;
    }
    match samples {
        Some(crate::scales::ScalePopulation::Numbers { sources, .. }) => sources
            .iter()
            .find(|batch| batch.layer == layer && &batch.input == input),
        _ => None,
    }
}
/// Sample through the common scale engine, then select the current panel's rows.
pub(super) fn sample_layer_batch<T>(
    source: Option<&crate::scales::ScaleLayerBatch>,
    table: &PreparedTable,
    rows: &[EncodedRow],
    values: &[Option<f64>],
    sample: impl FnOnce(&[Option<f64>]) -> ChartResult<Option<crate::scales::PaletteBatch<T>>>,
) -> ChartResult<Option<crate::scales::PaletteBatch<T>>> {
    let Some(source) = source else {
        return sample(values)?
            .map(|batch| batch.for_rows(rows.len()))
            .transpose();
    };
    let inputs = source
        .rows
        .values()
        .map(|(_, value)| value.map(|v| v.0))
        .collect::<Vec<_>>();
    let mut batch = sample(&inputs)?
        .map(|batch| batch.for_rows(inputs.len()))
        .transpose()?;
    if let Some(batch) = &mut batch
        && batch.values.is_some()
    {
        let positions = source
            .rows
            .values()
            .enumerate()
            .map(|(i, (key, _))| (key.clone(), i))
            .collect::<BTreeMap<_, _>>();
        batch.indices = rows
            .iter()
            .map(|row| {
                let identity = row
                    .key
                    .map(crate::scales::ScaleRowIdentity::Source)
                    .unwrap_or_else(|| {
                        crate::scales::ScaleRowIdentity::Generated(
                            table.population_operation().and_then(|op| op.panel.clone()),
                            row.ordinal,
                        )
                    });
                positions
                    .get(&identity)
                    .map(|i| batch.indices[*i])
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::SchemaConflict,
                            "Transformed aesthetic row is absent from its population.",
                        )
                    })
            })
            .collect::<ChartResult<Vec<_>>>()?;
    }
    Ok(batch)
}

/// Per-layer observed legend keys, using the same prepared population as scale training.
pub(crate) fn guide_key_population(
    input: &ColorInput,
    layer: &Layer,
    chart: &PreparedChart,
    table: &PreparedTable,
) -> ChartResult<std::collections::BTreeSet<crate::scales::ScaleKey>> {
    let source = chart.source().get()?;
    let data = source.dataset(table.input().dataset)?;
    key_population(input, layer, chart.definition(), data, table, true)
        .map(|keys| keys.into_iter().collect())
}
