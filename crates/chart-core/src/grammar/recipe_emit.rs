//! Shared row input, emission, validation and budget boundary for built-in recipes.
use super::{compiler::EncodedRow, *};
use crate::{ChartResult, DiagnosticCode};
use std::sync::Arc;
fn value_number(value: &crate::interpolate::Value) -> Option<f64> {
    match value {
        crate::interpolate::Value::Number(v) => Some(v.0),
        _ => None,
    }
}
pub(super) fn number(row: &EncodedRow, channel: RecipeAesthetic) -> Option<f64> {
    row.recipe_values.get(&channel).and_then(value_number)
}
/// Defaults apply only to absent mappings; mapped missing values remain missing.
pub(super) fn number_or(row: &EncodedRow, channel: RecipeAesthetic, default: f64) -> Option<f64> {
    if row.recipe_values.contains_key(&channel) {
        number(row, channel)
    } else {
        Some(default)
    }
}
/// Retain positional training when a nonpositional recipe control suppresses ink.
pub(super) fn retain_positions(prepared: &mut PreparedLayer, row: &EncodedRow) {
    for value in [row.x, row.x2]
        .into_iter()
        .flatten()
        .filter(|v| v.is_finite())
    {
        Extent::include(&mut prepared.domains.x, value);
    }
    for value in [row.y, row.y2, row.low, row.high]
        .into_iter()
        .flatten()
        .filter(|v| v.is_finite())
    {
        Extent::include(&mut prepared.domains.y, value);
    }
}
pub(super) fn resolve(
    layer: &Layer,
    data: &crate::data::DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    domains: &DomainContributions,
    limits: CompileLimits,
) -> ChartResult<()> {
    for (channel, input) in &layer.recipe_aes {
        if super::colors::dropped(input, table) {
            continue;
        }
        let input_space = match input {
            ColorInput::Numeric(n) => Some(super::stats::numeric_space(data, n)?),
            ColorInput::Statistical(field) => match &table.schema {
                OutputSchema::Statistical { fields, .. } | OutputSchema::Custom { fields, .. } => {
                    fields
                        .iter()
                        .find(|f| &f.field == field)
                        .map(|f| f.space.clone())
                }
                _ => None,
            },
            _ => None,
        };
        let input = super::colors::read_inputs(input, data, table, rows, limits, None, true)?;
        for (i, row) in rows.iter_mut().enumerate() {
            let mut value = input.values[i]
                .map(|v| crate::interpolate::Value::Number(crate::interpolate::Number(v)))
                .or_else(|| {
                    input.categories[i]
                        .clone()
                        .map(crate::interpolate::Value::Text)
                })
                .unwrap_or(crate::interpolate::Value::Missing);
            if dependent_channel(*channel) {
                let mut v = value_number(&value);
                if input_space.as_ref() != domains.y_space.as_ref()
                    && let Some(ValueSpace::Scaled { scale, .. }) = &domains.y_space
                {
                    v = scale.project_optional(v);
                }
                if *channel == RecipeAesthetic::Lower {
                    row.low = v;
                } else if *channel == RecipeAesthetic::Upper {
                    row.high = v;
                }
                value = v
                    .map(|v| crate::interpolate::Value::Number(crate::interpolate::Number(v)))
                    .unwrap_or(crate::interpolate::Value::Missing);
            }
            row.recipe_values.insert(*channel, value);
        }
    }
    Ok(())
}
pub(super) fn dependent_channel(channel: RecipeAesthetic) -> bool {
    matches!(
        channel,
        RecipeAesthetic::Lower
            | RecipeAesthetic::Upper
            | RecipeAesthetic::Middle
            | RecipeAesthetic::WhiskerLower
            | RecipeAesthetic::WhiskerUpper
            | RecipeAesthetic::NotchLower
            | RecipeAesthetic::NotchUpper
    )
}
pub(super) fn push(
    prepared: &mut PreparedLayer,
    row: &EncodedRow,
    layer: &Layer,
    geometry: PreparedGeometry,
    vertices: &mut usize,
) -> ChartResult<()> {
    let count = match &geometry {
        PreparedGeometry::Recipe(v) => v.points().len().max(1),
        PreparedGeometry::LineRun(p) | PreparedGeometry::Polygon(p) => p.len(),
        PreparedGeometry::Point(_) => 1,
        _ => 4,
    };
    *vertices = vertices.checked_sub(count).ok_or_else(|| {
        error(
            DiagnosticCode::ResourceLimit,
            "Recipe geometry exceeds vertex budget.",
        )
    })?;
    if matches!(layer.recipe, Some(BuiltinRecipe::Rug(_))) {
        if let Some(v) = row.x {
            Extent::include(&mut prepared.domains.x, v);
        }
        if let Some(v) = row.y {
            Extent::include(&mut prepared.domains.y, v);
        }
    }
    super::compiler::include_geometry(&mut prepared.domains, &geometry);
    let component_style = if matches!(layer.recipe, Some(BuiltinRecipe::Interval(_))) {
        let box_fill = if matches!(
            layer.recipe,
            Some(BuiltinRecipe::Interval(IntervalRecipe {
                kind: IntervalKind::Crossbar,
                ..
            }))
        ) && matches!(geometry, PreparedGeometry::Rectangle { .. })
        {
            Some(super::compiler::row_style(layer, row)?.fill)
        } else {
            None
        };
        let mut component_layer = layer.clone();
        let mut component_row = row.clone();
        if let Some(BuiltinRecipe::Interval(spec)) = &layer.recipe {
            let controls = if spec.kind == IntervalKind::Crossbar
                && matches!(geometry, PreparedGeometry::Rule { .. })
            {
                component_layer.style.alpha = None;
                component_row.alpha = None;
                Some(&spec.middle)
            } else if spec.kind == IntervalKind::Crossbar
                && matches!(geometry, PreparedGeometry::Rectangle { .. })
            {
                component_layer.style.alpha = None;
                component_row.alpha = None;
                Some(&spec.box_style)
            } else {
                None
            };
            if let Some(controls) = controls {
                if let Some(color) = controls.color {
                    component_layer.style.color = color;
                    component_layer.style.stroke = Some(color);
                    component_row.color = None;
                    component_row.stroke = None;
                }
                if let Some(width) = controls.linewidth {
                    component_layer.style.stroke_width = width;
                    component_row.stroke_width = None;
                }
                if let Some(line) = controls.line_type {
                    component_layer.style.line_type = Some(line);
                }
            }
            if spec.kind == IntervalKind::PointRange
                && matches!(geometry, PreparedGeometry::Point(_))
                && let Some(fill) = spec.point.fill
            {
                component_layer.style.fill = Some(fill);
                component_row.fill = None;
            }
        }
        let mut component_style = super::compiler::row_style(&component_layer, &component_row)?;
        if let Some(fill) = box_fill {
            component_style.fill = fill;
        }
        component_style
    } else {
        super::compiler::row_style(layer, row)?
    };
    let mut component_style = component_style;
    if matches!(
        layer.recipe,
        Some(BuiltinRecipe::Interval(IntervalRecipe {
            kind: IntervalKind::PointRange,
            ..
        }))
    ) && matches!(geometry, PreparedGeometry::Point(_))
        && let Some(size) = row.size
    {
        component_style.radius = size;
    }
    Arc::make_mut(&mut prepared.marks).push(PreparedMark {
        geometry,
        style: component_style,
        group: row.group.clone().unwrap_or(GroupValue::All),
        targets: vec![row.target.clone()],
        aesthetics: row.values.clone(),
    });
    Ok(())
}
pub(super) fn resolution(values: impl Iterator<Item = f64>) -> f64 {
    super::ggplot_position::resolution(values.collect())
}
pub(super) fn validate(layer: &Layer) -> ChartResult<()> {
    if let Some(recipe) = &layer.recipe {
        super::recipe_distributions::validate(recipe)?;
        super::recipe_intervals::validate(recipe)?;
        super::recipe_surfaces::validate(recipe)?;
        super::recipe_marks::validate(recipe)?;
    }
    Ok(())
}
pub(super) fn setup(
    layer: &Layer,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
) -> ChartResult<()> {
    super::recipe_distributions::setup(layer, rows, limits)?;
    super::recipe_intervals::setup(layer, rows, limits)?;
    super::recipe_surfaces::setup(layer, rows, limits)?;
    super::recipe_marks::setup(layer, rows, limits)
}
pub(super) fn emit(
    layer: &Layer,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<bool> {
    if matches!(layer.recipe, Some(BuiltinRecipe::Smooth)) {
        super::recipe_models::emit(layer, rows, prepared, vertices)?;
        return Ok(false);
    }
    if layer.recipe.is_none() {
        return Ok(false);
    }
    if super::recipe_distributions::emit(layer, rows, prepared, vertices)? {
        return Ok(true);
    }
    if super::recipe_intervals::emit(layer, rows, prepared, vertices)? {
        return Ok(true);
    }
    if super::recipe_surfaces::emit(layer, rows, prepared, vertices)? {
        return Ok(true);
    }
    super::recipe_marks::emit(layer, rows, prepared, vertices)
}
