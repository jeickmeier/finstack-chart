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
    glyph: Option<LegendGlyph>,
}

#[derive(Clone, PartialEq)]
enum Legend<'a> {
    Color(&'a ColorLegend),
    Symbol(&'a crate::grammar::SymbolLegend),
}
enum LegendGlyph {
    Color(Color),
    Symbol {
        geometry: crate::path::PathGeometry,
        bounds: Rect,
        fill: Option<Color>,
        stroke: Option<crate::scene::Stroke>,
    },
}
impl LegendGlyph {
    fn dimensions(&self, r: &LayoutRequest) -> (f64, f64) {
        match self {
            Self::Color(_) => (r.font_size, r.font_size),
            Self::Symbol { bounds, .. } => (bounds.width(), bounds.height()),
        }
    }
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
            GroupValue::Missing | GroupValue::Interaction(_) | GroupValue::Number(_) => v.label(),
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn legends(chart: &PreparedChart) -> Vec<Legend<'_>> {
    if !chart.state().legend_visible() {
        return vec![];
    }
    let mut legends = vec![];
    for layer in chart.layers() {
        if let Some(legend) = layer.color_legend().filter(|l| !l.entries.is_empty()) {
            let legend = Legend::Color(legend);
            if !legends.contains(&legend) {
                legends.push(legend);
            }
        }
        for legend in layer
            .symbol_legends()
            .iter()
            .filter(|l| !l.entries.is_empty())
        {
            let legend = Legend::Symbol(legend);
            if !legends.contains(&legend) {
                legends.push(legend);
            }
        }
    }
    legends
}

fn measure_labels(
    values: Vec<(String, Option<LegendGlyph>)>,
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
        .map(|(text, glyph)| {
            let metrics = measure_text(measurer, text_request(request, &text), request.limits)?;
            Ok(Label {
                text,
                metrics,
                glyph,
            })
        })
        .collect()
}

fn legend_values(
    legends: &[Legend<'_>],
    r: &LayoutRequest,
) -> ChartResult<Vec<(String, Option<LegendGlyph>)>> {
    let mut result = vec![];
    for legend in legends {
        match legend {
            Legend::Color(l) => {
                let title = l.title.clone().unwrap_or_else(|| {
                    if l.continuous {
                        "Value".into()
                    } else {
                        "Color".into()
                    }
                });
                if !title.is_empty() {
                    result.push((title, None));
                }
                result.extend(
                    l.entries
                        .iter()
                        .map(|(label, color)| (label.clone(), Some(LegendGlyph::Color(*color)))),
                );
            }
            Legend::Symbol(l) => {
                if !l.title.is_empty() {
                    result.push((l.title.clone(), None));
                }
                for entry in &l.entries {
                    let geometry = match &entry.geometry {
                        Some(geometry) => geometry.clone(),
                        None => crate::shape::Symbol::new()
                            .kind(entry.kind)
                            .size(entry.size)
                            .generate()?
                            .geometry(),
                    };
                    let color = r
                        .host_theme
                        .mark
                        .map(crate::color::Paint::resolve)
                        .unwrap_or(l.color);
                    let fill = entry.paint.fills().then_some(color);
                    let stroke = entry.paint.strokes().then_some(crate::scene::Stroke {
                        color,
                        width: l.stroke_width,
                    });
                    let base = geometry
                        .bounds(0.01, r.limits.max_path_commands)?
                        .unwrap_or(Rect::new(0., 0., 0., 0.)?);
                    let half = stroke.map_or(0., |s| s.width / 2.);
                    let bounds = Rect::new(
                        base.origin().x() - half,
                        base.origin().y() - half,
                        base.width() + 2. * half,
                        base.height() + 2. * half,
                    )?;
                    result.push((
                        entry.label.clone(),
                        Some(LegendGlyph::Symbol {
                            geometry,
                            bounds,
                            fill,
                            stroke,
                        }),
                    ));
                }
            }
        }
    }
    Ok(result)
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
        guide: None,
        layer: None,
        clip: Some(clip),
        primitive: Primitive::Text {
            origin: Point::new(x, y + label.metrics.ascent())?,
            text: label.text.clone(),
            font: r.font.id,
            font_size: r.font_size,
            color: r
                .host_theme
                .foreground
                .map(crate::color::Paint::resolve)
                .unwrap_or(INK),
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
        let height = label
            .metrics
            .height()
            .max(label.glyph.as_ref().map_or(0., |g| g.dimensions(r).1))
            .max(r.font_size);
        if y + height > bounds.max_y() - r.padding {
            constrained = true;
            break;
        }
        let swatch = if let Some(glyph) = &label.glyph {
            require_within(items.len() < r.limits.max_items, "legend swatch item")?;
            let (width, glyph_height) = glyph.dimensions(r);
            match glyph {
                LegendGlyph::Color(color) => {
                    let size = r.font_size.min(bounds.width() / 3.);
                    if size > 0. {
                        items.push(SceneItem {
                            guide: None,
                            layer: None,
                            clip: Some(bounds),
                            primitive: Primitive::Rectangle {
                                bounds: Rect::new(bounds.origin().x(), y, size, size)?,
                                fill: *color,
                            },
                        });
                    }
                }
                LegendGlyph::Symbol {
                    geometry,
                    bounds: local,
                    fill,
                    stroke,
                } => {
                    let map = crate::path::Affine::new([
                        1.,
                        0.,
                        0.,
                        1.,
                        bounds.origin().x() - local.origin().x(),
                        y + (height - glyph_height) / 2. - local.origin().y(),
                    ])?;
                    if geometry.has_segments() {
                        items.push(SceneItem {
                            guide: None,
                            layer: None,
                            clip: Some(bounds),
                            primitive: Primitive::VectorPath {
                                dashes: vec![],
                                geometry: geometry.transformed(
                                    map,
                                    0.01,
                                    r.limits.max_path_commands,
                                )?,
                                fill: *fill,
                                stroke: *stroke,
                            },
                        });
                    }
                    constrained |= width > bounds.width();
                }
            }
            width + r.label_gap
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

pub(super) struct SingleLegend {
    pub content: Rect,
    bounds: Rect,
    labels: Vec<Label>,
}

fn legend_width(labels: &[Label], width: f64, request: &LayoutRequest) -> f64 {
    if labels.is_empty() {
        0.
    } else {
        (labels
            .iter()
            .map(|l| {
                l.metrics.width()
                    + l.glyph
                        .as_ref()
                        .map_or(request.font_size + request.label_gap, |g| {
                            g.dimensions(request).0.max(request.font_size) + request.label_gap
                        })
            })
            .fold(0_f64, f64::max)
            + request.padding)
            .min(width * 0.3)
    }
}

pub(super) fn prepare_single_legend(
    prepared: &PreparedChart,
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
) -> ChartResult<Option<SingleLegend>> {
    if prepared.definition().facets.is_some() {
        return Ok(None);
    }
    let mut remaining = request.limits.max_text_bytes;
    let labels = measure_labels(
        legend_values(&legends(prepared), request)?,
        request,
        measurer,
        &mut remaining,
    )?;
    if labels.is_empty() {
        return Ok(None);
    }
    let width = legend_width(&labels, request.bounds.width(), request);
    let content = Rect::new(
        request.bounds.origin().x(),
        request.bounds.origin().y(),
        request.bounds.width() - width,
        request.bounds.height(),
    )?;
    let bounds = Rect::new(
        content.max_x(),
        content.origin().y(),
        width,
        content.height(),
    )?;
    Ok(Some(SingleLegend {
        content,
        bounds,
        labels,
    }))
}

pub(super) fn finish_single_legend(
    chart: &mut LaidOutChart,
    request: &LayoutRequest,
    legend: SingleLegend,
) -> ChartResult<()> {
    let mut items = chart.scene.items().to_vec();
    paint_legend(
        &mut items,
        &legend.labels,
        legend.bounds,
        request,
        &mut chart.diagnostics,
    )?;
    chart.targets.resize(items.len(), vec![]);
    chart.item_panels.resize(items.len(), None);
    chart.scene = Scene::new(
        chart.scene.stamp(),
        request.units,
        chart.scene.bounds(),
        &items,
        chart.scene.resources(),
        request.limits,
    )?;
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
        legend_values(&shared_legends, request)?,
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
                legend_values(&legends(&panel.chart), request)?,
                request,
                measurer,
                &mut remaining,
            )?
        });
    }
    let shared_width = legend_width(&shared_labels, request.bounds.width(), request);
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
        .map(|l| legend_width(l, cell_width, request))
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
        r.hierarchy_scope
            .push(super::GuideScope::Panel(panel.key.clone()));
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
        let scope = format!(
            "panel:{}",
            serde_json::to_string(&panel.key).map_err(|_| crate::scales::error(
                crate::DiagnosticCode::Validation,
                "Panel scope cannot be encoded."
            ))?
        );
        items.extend(chart.scene().items().iter().cloned().map(|mut item| {
            if let Some(guide) = &mut item.guide {
                guide.scope.insert(0, scope.clone());
            }
            item
        }));
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
        hierarchies: Default::default(),
        guide_frames: Default::default(),
        guide_presentation: None,
        guides: Default::default(),
        paint_themes: std::collections::BTreeMap::new(),
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
