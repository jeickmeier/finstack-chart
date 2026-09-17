//! Color, numeric and position application after aesthetic encoding.
use super::compiler::{EncodedLayer, GeometryBudget, PositionedLayer};
use super::*;
use crate::ChartResult;
use crate::data::DatasetSnapshot;
use crate::state::ChartState;
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn position_layer(
    layer: &Layer,
    table: Arc<PreparedTable>,
    data: &DatasetSnapshot,
    encoding: EncodedLayer,
    state: &ChartState,
    budget: &mut GeometryBudget<'_>,
) -> ChartResult<PositionedLayer> {
    let EncodedLayer {
        mut domains,
        mut encoded,
        hierarchy,
        mapped_size,
    } = encoding;
    let extensions = budget.extensions;
    let shape_protocols = super::shape_extensions::resolve_layer(layer, extensions)?;
    let limits = budget.limits;
    let vertices = &mut budget.vertices;
    super::scale_stage::generated_rows(layer, budget.population_axes, &mut domains, &mut encoded)?;
    for row in &mut encoded {
        row.outlier_anchor_y = row.y;
    }
    let catalog = layer
        .color
        .as_ref()
        .and_then(|c| budget.color_domains.get(&c.id));
    let color_legend = super::colors::apply(
        layer.color.as_ref(),
        None,
        data,
        &table,
        &mut encoded,
        super::colors::ColorContext {
            layer: layer.id,
            limits,
            registry: extensions,
            shared: catalog.map(Vec::as_slice),
            samples: layer
                .color
                .as_ref()
                .and_then(|c| budget.color_samples.get(&c.id)),
        },
    )?;
    let mut paint_legends = BTreeMap::new();
    for (channel, encoding) in &layer.paint_scales {
        if match channel {
            PaintAesthetic::Fill => layer.style.fill.is_some(),
            PaintAesthetic::Stroke => layer.style.stroke.is_some(),
        } {
            continue;
        }
        if let Some(legend) = super::colors::apply(
            Some(encoding),
            Some(*channel),
            data,
            &table,
            &mut encoded,
            super::colors::ColorContext {
                layer: layer.id,
                limits,
                registry: extensions,
                shared: budget.color_domains.get(&encoding.id).map(Vec::as_slice),
                samples: budget.color_samples.get(&encoding.id),
            },
        )? {
            paint_legends.insert(*channel, legend);
        }
    }
    let mut value_guides = super::style_channels::ValueGuides::default();
    let numeric_scales = super::numeric_aesthetics::apply(
        layer,
        data,
        &table,
        &mut encoded,
        super::numeric_aesthetics::NumericContext {
            limits,
            samples: budget.color_samples,
            registry: extensions,
            profile: budget.profile,
            guides: &mut value_guides,
        },
    )?;
    let value_scales = super::style_channels::apply(
        layer,
        data,
        &table,
        &mut encoded,
        super::numeric_aesthetics::NumericContext {
            limits,
            samples: budget.color_samples,
            registry: extensions,
            profile: budget.profile,
            guides: &mut value_guides,
        },
    )?;
    let mut symbol_legends = super::symbols::apply(
        layer,
        data,
        &table,
        &mut encoded,
        limits,
        budget.color_samples,
        extensions,
    )?;
    if let Some(symbol) = shape_protocols.symbol() {
        super::symbols::custom_glyphs(&mut symbol_legends, symbol, limits, vertices)?;
    }
    let area_size = layer.geom == Geom::Point
        && layer
            .numeric_scales
            .contains_key(&NumericAesthetic::AreaSize);
    let mapped_size =
        mapped_size || area_size || layer.numeric_scales.contains_key(&NumericAesthetic::Size);
    super::recipe_emit::setup(layer, &mut encoded, limits)?;
    let stack = super::positions::apply(layer, &domains, &mut encoded, limits, &shape_protocols)?;
    super::positions::output_space(layer, &mut domains);
    let prepared = PreparedLayer {
        geography_vertices: Vec::new(),
        hierarchy,
        shape_protocols,
        orientation: layer.orientation,
        interactions: BTreeMap::new(),
        color_legend,
        paint_legends,
        numeric_scales,
        value_scales,
        value_guides,
        symbol_legends,
        position: layer.position.clone(),
        id: layer.id,
        scales: layer.scales,
        clip: layer.clip,
        table,
        marks: Arc::new(vec![]),
        domains,
        unpainted_categories: None,
        invalid_geometry: 0,
        visible: state.is_visible(layer.id),
    };
    Ok(PositionedLayer {
        raw_training: vec![],
        prepared,
        encoded,
        mapped_size,
        area_size,
        stack,
    })
}
