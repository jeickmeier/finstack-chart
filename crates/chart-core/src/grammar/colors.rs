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
    /// Finite source numeric mapping.
    Numeric(Numeric),
    /// Prepared stable group identity.
    Group,
    /// Generated numeric statistical field.
    Statistical(StatField),
}
/// Named portable color mapping; it never trains positional domains.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ColorEncoding {
    /// Human-readable legend title; absent uses a generic color label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Shared color scale identity.
    pub id: ScaleId,
    /// Stage-specific value.
    pub input: ColorInput,
    /// Exact palette, domain, null and outside policy.
    pub scale: ColorScale,
}

pub(super) fn apply(
    layer: &Layer,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
) -> ChartResult<Option<ColorLegend>> {
    let Some(encoding) = &layer.color else {
        return Ok(None);
    };
    if encoding
        .title
        .as_ref()
        .is_some_and(|title| title.len() > 4096)
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Color legend title exceeds 4096 bytes.",
        ));
    }
    let palette_len = match &encoding.scale {
        ColorScale::Discrete {
            domain, palette, ..
        } => {
            if domain.as_ref().is_some_and(|d| d.len() > limits.max_groups) {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Color domain exceeds category budget.",
                ));
            }
            palette.len()
        }
        ColorScale::Continuous { palette, .. } => palette.len(),
    };
    if palette_len > limits.max_groups {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Color catalog/palette exceeds category budget.",
        ));
    }
    encoding.scale.validate()?;
    let mut labels = vec![];
    let mut categories = vec![None; rows.len()];
    let mut values = vec![None; rows.len()];
    match &encoding.input {
        ColorInput::Group => {
            for (i, row) in rows.iter().enumerate() {
                categories[i] = row.group.as_ref().map(super::statistics::group_label);
            }
        }
        ColorInput::Category(field) => {
            if !matches!(table.rows, PreparedRows::Source(_)) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Source color fields require source rows.",
                ));
            }
            super::stats::validate_group(data, &Grouping::Field(*field))?;
            let catalog = data.categories(*field).unwrap_or_default();
            if catalog.len() > limits.max_groups {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Color catalog exceeds category budget.",
                ));
            }
            labels = catalog.to_vec();
            let source: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
            for (i, row) in rows.iter().enumerate() {
                let r = source[&row.key.expect("source row")];
                categories[i] = match r.value(*field) {
                    None => None,
                    Some(ValueRef::Category(s) | ValueRef::Utf8(s)) => Some(s.to_owned()),
                    _ => super::stats::group_value(r, &Grouping::Field(*field))
                        .map(|g| super::statistics::group_label(&g)),
                };
            }
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
    let categorical = matches!(encoding.input, ColorInput::Group | ColorInput::Category(_));
    if categorical != matches!(encoding.scale, ColorScale::Discrete { .. }) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Color input and scale family disagree.",
        ));
    }
    let categorical_map = if let ColorScale::Discrete {
        domain,
        palette,
        missing,
    } = &encoding.scale
    {
        Some((
            domain
                .as_deref()
                .unwrap_or(&labels)
                .iter()
                .enumerate()
                .map(|(i, label)| (label.as_str(), palette[i % palette.len()]))
                .collect::<BTreeMap<_, _>>(),
            *missing,
        ))
    } else {
        None
    };
    for (i, row) in rows.iter_mut().enumerate() {
        row.color = Some(if let Some((catalog, missing)) = &categorical_map {
            categories[i]
                .as_deref()
                .and_then(|label| catalog.get(label))
                .copied()
                .unwrap_or(*missing)
        } else {
            encoding.scale.numeric_validated(values[i])?
        });
    }
    let mut legend = encoding.scale.legend(encoding.id, &labels)?;
    legend.title.clone_from(&encoding.title);
    Ok(Some(legend))
}
