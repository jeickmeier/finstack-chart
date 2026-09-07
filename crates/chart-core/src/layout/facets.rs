use super::{
    engine::{pressure, solve_panels, text_request},
    *,
};
use crate::grammar::{GroupValue, PreparedChart};
use crate::limits::require_within;
use crate::scales::ColorLegend;
use crate::scene::{Color, Primitive, Scene, SceneItem};
use crate::services::{TextMeasurer, TextMetrics, measure_text};
use crate::{ChartResult, Diagnostic, Point, Rect, SceneStamp};
use std::sync::Arc;

const INK: Color = Color {
    red: 55,
    green: 60,
    blue: 65,
    alpha: 255,
};

struct Label {
    text: String,
    metrics: TextMetrics,
    color: Option<Color>,
}

fn panel_label(key: &crate::grammar::PanelKey) -> String {
    key.values
        .iter()
        .map(|v| match v {
            GroupValue::All => String::new(),
            GroupValue::Text(v) => v.clone(),
            GroupValue::Int(v) => v.to_string(),
            GroupValue::UInt(v) => v.to_string(),
            GroupValue::Boolean(v) => v.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn legends(chart: &PreparedChart) -> Vec<ColorLegend> {
    if !chart.state().legend_visible() {
        return vec![];
    }
    let mut legends = vec![];
    for legend in chart.layers().iter().filter_map(|l| l.color_legend()) {
        // Identity, full domain/palette, continuous/discrete semantics and missing policy agree.
        if !legends.contains(legend) {
            legends.push(legend.clone());
        }
    }
    legends
}

fn measure_labels(
    values: Vec<(String, Option<Color>)>,
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    remaining: &mut usize,
) -> ChartResult<Vec<Label>> {
    for (text, _) in &values {
        require_within(text.len() <= *remaining, "figure furniture text byte")?;
        *remaining -= text.len();
    }
    values
        .into_iter()
        .map(|(text, color)| {
            let metrics = measure_text(measurer, text_request(request, &text), request.limits)?;
            Ok(Label {
                text,
                metrics,
                color,
            })
        })
        .collect()
}

fn legend_values(legends: &[ColorLegend]) -> Vec<(String, Option<Color>)> {
    legends
        .iter()
        .flat_map(|l| {
            std::iter::once((
                l.title.clone().unwrap_or_else(|| {
                    if l.continuous {
                        "Value".into()
                    } else {
                        "Color".into()
                    }
                }),
                None,
            ))
            .chain(
                l.entries
                    .iter()
                    .map(|(label, color)| (label.clone(), Some(*color))),
            )
        })
        .collect()
}

fn push_text(
    items: &mut Vec<SceneItem>,
    label: &Label,
    x: f64,
    y: f64,
    clip: Rect,
    r: &LayoutRequest,
) -> ChartResult<()> {
    require_within(items.len() < r.limits.max_items, "figure scene item")?;
    items.push(SceneItem {
        layer: None,
        clip: Some(clip),
        primitive: Primitive::Text {
            origin: Point::new(x, y + label.metrics.ascent())?,
            text: label.text.clone(),
            font: r.font.id,
            font_size: r.font_size,
            color: r.host_theme.foreground.unwrap_or(INK),
        },
    });
    Ok(())
}

fn paint_legend(
    items: &mut Vec<SceneItem>,
    labels: &[Label],
    bounds: Rect,
    r: &LayoutRequest,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<()> {
    let mut y = bounds.origin().y() + r.padding;
    let mut constrained = false;
    for label in labels {
        let height = label.metrics.height().max(r.font_size);
        if y + height > bounds.max_y() - r.padding {
            constrained = true;
            break;
        }
        let swatch = if let Some(color) = label.color {
            require_within(items.len() < r.limits.max_items, "legend swatch item")?;
            let size = r.font_size.min(bounds.width() / 3.);
            if size > 0. {
                items.push(SceneItem {
                    layer: None,
                    clip: Some(bounds),
                    primitive: Primitive::Rectangle {
                        bounds: Rect::new(bounds.origin().x(), y, size, size)?,
                        fill: color,
                    },
                });
            }
            r.font_size + r.label_gap
        } else {
            0.
        };
        let x = bounds.origin().x() + swatch;
        constrained |= x + label.metrics.width() > bounds.max_x();
        push_text(items, label, x, y, bounds, r)?;
        y += height + r.label_gap;
    }
    if constrained {
        diagnostics.push(pressure("Legend pressure clipped long labels or omitted trailing swatches; complete legend metadata remains in preparation."));
    }
    Ok(())
}

pub(super) fn layout_facets(
    prepared: Arc<PreparedChart>,
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    stamp: SceneStamp,
) -> ChartResult<LaidOutChart> {
    let spec = prepared.definition().facets.as_ref().ok_or_else(|| {
        crate::scales::error(
            crate::DiagnosticCode::Validation,
            "Missing facet specification.",
        )
    })?;
    let rows = prepared
        .panels()
        .iter()
        .map(|p| p.row + 1)
        .max()
        .unwrap_or(1);
    let columns = prepared
        .panels()
        .iter()
        .map(|p| p.column + 1)
        .max()
        .unwrap_or(1);
    let mut remaining = request.limits.max_text_bytes;
    let labels = measure_labels(
        prepared
            .panels()
            .iter()
            .map(|p| (panel_label(&p.key), None))
            .collect(),
        request,
        measurer,
        &mut remaining,
    )?;
    let header_height = labels
        .iter()
        .map(|l| l.metrics.height())
        .fold(0_f64, f64::max)
        + request.label_gap;
    let shared_legends = if spec.collect_guides {
        legends(&prepared)
    } else {
        vec![]
    };
    let shared_labels = measure_labels(
        legend_values(&shared_legends),
        request,
        measurer,
        &mut remaining,
    )?;
    let mut local_labels = vec![];
    for panel in prepared.panels() {
        local_labels.push(if spec.collect_guides {
            vec![]
        } else {
            measure_labels(
                legend_values(&legends(&panel.chart)),
                request,
                measurer,
                &mut remaining,
            )?
        });
    }
    let legend_width = |labels: &[Label], width: f64| -> f64 {
        if labels.is_empty() {
            0.
        } else {
            (labels
                .iter()
                .map(|l| l.metrics.width() + request.font_size + request.label_gap)
                .fold(0_f64, f64::max)
                + request.padding)
                .min(width * 0.3)
        }
    };
    let shared_width = legend_width(&shared_labels, request.bounds.width());
    let total_width = request.bounds.width() - shared_width;
    let cell_width = (total_width - spec.gap * (columns - 1) as f64) / columns as f64;
    let cell_height = (request.bounds.height() - spec.gap * (rows - 1) as f64) / rows as f64;
    if cell_width <= 0. || cell_height <= header_height || prepared.panels().is_empty() {
        // Reuse the ordinary compact-state route with the original snapshot retained afterwards.
        let mut empty = (*prepared).clone();
        let mut definition = empty.definition().clone();
        definition.facets = None;
        empty.definition = Arc::new(definition);
        empty.layers.clear();
        empty.scale_domains.clear();
        empty.panels.clear();
        let mut compact_request = request.clone();
        if !prepared.panels().is_empty() {
            compact_request.minimum_plot =
                (request.bounds.width() + 1., request.bounds.height() + 1.);
        }
        let mut result =
            solve_panels(vec![(Arc::new(empty), compact_request)], measurer, stamp)?.remove(0);
        result.prepared = prepared;
        return Ok(result);
    }
    let local_width = local_labels
        .iter()
        .map(|l| legend_width(l, cell_width))
        .fold(0_f64, f64::max);
    let mut cells = vec![];
    let mut inputs = vec![];
    for panel in prepared.panels() {
        let cell = Rect::new(
            request.bounds.origin().x() + panel.column as f64 * (cell_width + spec.gap),
            request.bounds.origin().y() + panel.row as f64 * (cell_height + spec.gap),
            cell_width,
            cell_height,
        )?;
        let mut r = request.clone();
        r.figure_bounds = Some(request.figure_bounds.unwrap_or(request.bounds));
        r.bounds = Rect::new(
            cell.origin().x(),
            cell.origin().y() + header_height,
            cell.width() - local_width,
            cell.height() - header_height,
        )?;
        let mut chart = (*panel.chart).clone();
        for (id, domain) in prepared.scale_domains() {
            chart.scale_domains.insert(*id, domain.clone());
        }
        chart.shared_training = Some(prepared.clone());
        inputs.push((Arc::new(chart), r));
        cells.push(cell);
    }
    let resolved = solve_panels(inputs, measurer, stamp)?;
    let mut interactions = std::collections::BTreeMap::new();
    let mut items = vec![];
    let mut targets = vec![];
    let mut item_panels = vec![];
    let mut panels = vec![];
    let mut diagnostics = prepared.diagnostics().to_vec();
    let mut passes = 0;
    let mut status = LayoutStatus::NoSpace;
    for (index, chart) in resolved.into_iter().enumerate() {
        let panel = &prepared.panels()[index];
        passes = passes.max(chart.passes());
        status = match (status, chart.status()) {
            (LayoutStatus::Ready, _) | (_, LayoutStatus::Ready) => LayoutStatus::Ready,
            (LayoutStatus::NoData, _) | (_, LayoutStatus::NoData) => LayoutStatus::NoData,
            _ => LayoutStatus::NoSpace,
        };
        diagnostics.extend_from_slice(chart.diagnostics());
        require_within(
            chart.scene().items().len() <= request.limits.max_items.saturating_sub(items.len()),
            "figure scene item",
        )?;
        interactions.extend(
            chart
                .interactions()
                .iter()
                .map(|(i, v)| (i + items.len(), v.clone())),
        );
        items.extend_from_slice(chart.scene().items());
        targets.extend_from_slice(chart.targets());
        item_panels.extend(std::iter::repeat_n(
            Some(panel.key.clone()),
            chart.scene().items().len(),
        ));
        let before = items.len();
        if labels[index].metrics.width() + request.padding > cells[index].width() {
            diagnostics.push(pressure(
                "Facet header exceeds its cell and is clipped; its full logical text is retained.",
            ));
        }
        push_text(
            &mut items,
            &labels[index],
            cells[index].origin().x() + request.padding,
            cells[index].origin().y(),
            cells[index],
            request,
        )?;
        if local_width > 0. {
            paint_legend(
                &mut items,
                &local_labels[index],
                Rect::new(
                    cells[index].max_x() - local_width,
                    cells[index].origin().y() + header_height,
                    local_width,
                    cells[index].height() - header_height,
                )?,
                request,
                &mut diagnostics,
            )?;
        }
        targets.extend(std::iter::repeat_n(vec![], items.len() - before));
        item_panels.extend(std::iter::repeat_n(
            Some(panel.key.clone()),
            items.len() - before,
        ));
        panels.push(LaidOutPanel {
            key: panel.key.clone(),
            row: panel.row,
            column: panel.column,
            bounds: cells[index],
            chart: Arc::new(chart),
        });
    }
    let before = items.len();
    if shared_width > 0. {
        paint_legend(
            &mut items,
            &shared_labels,
            Rect::new(
                request.bounds.max_x() - shared_width,
                request.bounds.origin().y(),
                shared_width,
                request.bounds.height(),
            )?,
            request,
            &mut diagnostics,
        )?;
    }
    targets.extend(std::iter::repeat_n(vec![], items.len() - before));
    item_panels.extend(std::iter::repeat_n(None, items.len() - before));
    let scene = Scene::new(
        stamp,
        request.units,
        request.bounds,
        &items,
        &super::text::resources(&items, request)?,
        request.limits,
    )?;
    Ok(LaidOutChart {
        interactions,
        insets: vec![],
        prepared,
        scene,
        plot: None,
        axes: Default::default(),
        targets,
        item_panels,
        panels,
        diagnostics,
        status,
        passes,
    })
}
