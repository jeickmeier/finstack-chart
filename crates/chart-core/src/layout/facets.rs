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

#[derive(Clone)]
struct Label {
    component: Option<crate::scene::GuideComponent>,
    text: String,
    metrics: TextMetrics,
    glyph: Option<LegendGlyph>,
}

enum LegendBlock {
    Bins {
        key: Box<super::legend_keys::KeyLegend>,
        labels: Vec<Label>,
    },
    Custom(crate::grammar::CustomLegend),
    Row(Label),
    Grid {
        labels: Vec<Label>,
        columns: usize,
        by_row: bool,
    },
    Colorbar(super::legend_colorbar::Colorbar),
}

#[derive(Clone, PartialEq)]
enum Legend<'a> {
    Custom(&'a crate::grammar::CustomLegend),
    Color(Box<std::borrow::Cow<'a, ColorLegend>>),
    Symbol(&'a crate::grammar::SymbolLegend),
    Keys(Box<super::legend_keys::KeyLegend>),
}
#[derive(Clone)]
enum LegendGlyph {
    Combined(Vec<super::legend_keys::KeyGlyph>),
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
            Self::Combined(glyphs) => {
                glyphs.iter().fold((r.font_size, r.font_size), |(w, h), g| {
                    (
                        w.max(if g.line.is_some() {
                            2. * r.font_size
                        } else {
                            2. * g.size + g.width
                        }),
                        h.max(2. * g.size + g.width),
                    )
                })
            }
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

fn step_draw_requires_cells(color: &ColorLegend) -> bool {
    use crate::scales::*;
    let Some(mapping) = &color.mapping else {
        return false;
    };
    let default_binned = matches!(
        mapping.ggplot.as_deref(),
        Some(GgplotScalePolicy::Binned(_))
    ) && matches!(
        mapping.guide.as_deref(),
        None | Some(GgplotScaleGuide::Binned(_))
    );
    if !default_binned
        && !matches!(
            mapping.guide.as_deref(),
            Some(
                GgplotScaleGuide::BinnedSteps(_)
                    | GgplotScaleGuide::ContinuousSteps(_)
                    | GgplotScaleGuide::TemporalSteps(_)
            )
        )
    {
        return false;
    }
    if !color.numeric_breaks.is_empty() {
        return true;
    }
    mapping
        .colorbar_options
        .as_deref()
        .is_some_and(|o| o.show_limits)
        && matches!(&mapping.function, ScaleFunctionSpec::Interpolated(s) if matches!(s.normalization, NormalizationSpec::Ggplot{domain,..} if domain[0] == domain[1]))
        && matches!(mapping.guide.as_deref(), Some(GgplotScaleGuide::ContinuousSteps(g)) if g.breaks.is_none())
}

fn legends<'a>(chart: &'a PreparedChart, request: &LayoutRequest) -> ChartResult<Vec<Legend<'a>>> {
    if !chart.state().legend_visible() {
        return Ok(vec![]);
    }
    let reference = chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3;
    let mut legends = vec![];
    for layer in chart.layers() {
        if chart.definition().layers.iter().any(|definition| {
            definition.id == layer.id() && definition.geom == crate::grammar::Geom::Blank
        }) {
            continue;
        }
        let policy = chart
            .definition()
            .layers
            .iter()
            .find(|l| l.id == layer.id())
            .and_then(|l| l.legend.as_ref());
        if let Some(legend) = layer
            .color_legend()
            .filter(|_| policy.is_none_or(|p| p.includes(crate::grammar::LegendAesthetic::Color)))
            .filter(|l| !l.entries.is_empty() || step_draw_requires_cells(l))
        {
            let legend = Legend::Color(Box::new(std::borrow::Cow::Borrowed(legend)));
            if !legends.contains(&legend) {
                legends.push(legend);
            }
        }
        for legend in layer
            .paint_legends()
            .iter()
            .filter(|(a, _)| {
                policy.is_none_or(|p| {
                    p.includes(if **a == crate::grammar::PaintAesthetic::Fill {
                        crate::grammar::LegendAesthetic::Fill
                    } else {
                        crate::grammar::LegendAesthetic::Stroke
                    })
                })
            })
            .map(|(_, l)| l)
            .filter(|l| !l.entries.is_empty() || step_draw_requires_cells(l))
        {
            let legend = Legend::Color(Box::new(std::borrow::Cow::Borrowed(legend)));
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
    if reference {
        legends.retain(|l| !matches!(l, Legend::Color(c) if c.colorbar.is_empty() && c.colorsteps.is_empty() && !step_draw_requires_cells(c)));
        legends.extend(
            super::legend_keys::collect(chart, request)?
                .into_iter()
                .map(|keys| Legend::Keys(Box::new(keys))),
        );
    }
    for legend in &mut legends {
        if let Legend::Color(color) = legend
            && let Some(options) = chart.definition().legends.get(&color.id)
        {
            let current = color
                .mapping
                .as_ref()
                .and_then(|m| m.colorbar_options.as_ref())
                .is_some_and(|o| o.reverse);
            if options.reverse.is_some_and(|reverse| reverse != current) {
                let c = color.to_mut();
                c.entries.reverse();
                c.colorbar.reverse();
                c.colorsteps.reverse();
                for step in &mut c.colorsteps {
                    std::mem::swap(&mut step.start, &mut step.end);
                }
                for p in c.colorstep_positions.iter_mut().flatten() {
                    p.0 = 1. - p.0;
                }
            }
            if let Some(title) = &options.title {
                color.to_mut().title = Some(title.clone());
            }
            let direction = options.direction.or_else(|| {
                matches!(
                    options.position,
                    Some(
                        crate::grammar::LegendPosition::Top
                            | crate::grammar::LegendPosition::Bottom
                    )
                )
                .then_some(crate::scene::GradientDirection::Horizontal)
            });
            if let Some(direction) = direction
                && let Some(mapping) = &mut color.to_mut().mapping
            {
                mapping
                    .colorbar_options
                    .get_or_insert_with(Default::default)
                    .direction = Some(direction);
            }
        }
    }
    legends.extend(chart.definition().custom_legends.iter().map(Legend::Custom));
    legends.sort_by_key(|legend| {
        let order = match legend {
            Legend::Custom(c) => c.options.order,
            Legend::Keys(k) => k.options.order,
            Legend::Color(c) => chart.definition().legends.get(&c.id).map_or(0, |o| o.order),
            _ => 0,
        };
        if order == 0 { 99 } else { order }
    });
    Ok(legends)
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
                component: None,
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
            Legend::Custom(_) => {}
            Legend::Keys(l) => {
                if !l.title.is_empty() {
                    result.push((l.title.clone(), None));
                }
                result.extend(
                    l.labels.iter().cloned().zip(
                        l.glyphs
                            .iter()
                            .cloned()
                            .map(|g| Some(LegendGlyph::Combined(g))),
                    ),
                );
            }
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

fn measure_legends(
    legends: &[Legend<'_>],
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    remaining: &mut usize,
) -> ChartResult<Vec<LegendBlock>> {
    let mut blocks = vec![];
    for legend in legends {
        let start = blocks.len();
        if let Legend::Color(color) = legend
            && color.colorsteps.is_empty()
            && step_draw_requires_cells(color)
        {
            return Err(crate::scales::error(
                crate::DiagnosticCode::Validation,
                "A stepped color guide requires at least one interval to draw.",
            ));
        }

        if let Legend::Custom(custom) = legend {
            if let Some(title) = &custom.options.title {
                for mut row in
                    measure_labels(vec![(title.clone(), None)], request, measurer, remaining)?
                {
                    let mut component = legend_component(
                        custom.id,
                        crate::scene::GuideRole::LegendTitle,
                        None,
                        Some(title.clone()),
                    );
                    component.scope.push("custom".into());
                    row.component = Some(component);
                    blocks.push(LegendBlock::Row(row));
                }
            }
            blocks.push(LegendBlock::Custom((*custom).clone()));
        } else if let Legend::Color(color) = legend
            && (!color.colorbar.is_empty() || !color.colorsteps.is_empty())
        {
            let title = color.title.clone().unwrap_or_else(|| "Value".into());
            if !title.is_empty() {
                blocks.extend(
                    measure_labels(vec![(title, None)], request, measurer, remaining)?
                        .into_iter()
                        .map(LegendBlock::Row),
                );
            }
            blocks.push(LegendBlock::Colorbar(
                super::legend_colorbar::Colorbar::measure(color, request, measurer, remaining)?,
            ));
        } else if let Legend::Keys(keys) = legend {
            let mut labels = measure_labels(
                legend_values(std::slice::from_ref(legend), request)?,
                request,
                measurer,
                remaining,
            )?;
            if !keys.title.is_empty() {
                blocks.push(LegendBlock::Row(labels.remove(0)));
            }
            let horizontal = keys.options.direction
                == Some(crate::scene::GradientDirection::Horizontal)
                || (keys.options.direction.is_none()
                    && matches!(
                        keys.options.position,
                        Some(
                            crate::grammar::LegendPosition::Top
                                | crate::grammar::LegendPosition::Bottom
                        )
                    ));
            if let (Some(rows), Some(columns)) = (keys.options.nrow, keys.options.ncol)
                && rows.saturating_mul(columns) < labels.len()
            {
                return Err(crate::scales::error(
                    crate::DiagnosticCode::Validation,
                    "Legend rows and columns cannot contain every key.",
                ));
            }
            let columns = keys
                .options
                .ncol
                .unwrap_or_else(|| {
                    keys.options
                        .nrow
                        .map_or(if horizontal { labels.len().max(1) } else { 1 }, |rows| {
                            labels.len().div_ceil(rows).max(1)
                        })
                })
                .min(labels.len().max(1));
            if keys.binned {
                if labels.len() < 2 {
                    return Err(crate::scales::error(
                        crate::DiagnosticCode::Validation,
                        "A binned key guide requires an interval.",
                    ));
                }
                blocks.push(LegendBlock::Bins {
                    key: keys.clone(),
                    labels,
                });
            } else if columns > 1 {
                blocks.push(LegendBlock::Grid {
                    labels,
                    columns,
                    by_row: keys.options.by_row,
                });
            } else {
                blocks.extend(labels.into_iter().map(LegendBlock::Row));
            }
        } else {
            blocks.extend(
                measure_labels(
                    legend_values(std::slice::from_ref(legend), request)?,
                    request,
                    measurer,
                    remaining,
                )?
                .into_iter()
                .map(LegendBlock::Row),
            );
        }
        let id = match legend {
            Legend::Color(c) => Some(c.id),
            Legend::Keys(k) => Some(k.id),
            _ => None,
        };
        if let Some(id) = id {
            let mut index = 0;
            let mut occurrences = std::collections::BTreeMap::<String, usize>::new();
            for row in blocks[start..].iter_mut().flat_map(|block| match block {
                LegendBlock::Row(row) => std::slice::from_mut(row),
                LegendBlock::Grid { labels, .. } | LegendBlock::Bins { labels, .. } => {
                    labels.as_mut_slice()
                }
                _ => &mut [],
            }) {
                {
                    let title = row.glyph.is_none();
                    row.component = Some(legend_component(
                        id,
                        if title {
                            crate::scene::GuideRole::LegendTitle
                        } else {
                            crate::scene::GuideRole::LegendLabel
                        },
                        (!title).then_some(index),
                        Some(row.text.clone()),
                    ));
                    if !title {
                        if let Legend::Keys(keys) = legend {
                            row.component.as_mut().unwrap().tick.as_mut().unwrap().value =
                                keys.values[index].clone();
                        }
                        let identity = serde_json::to_string(
                            &row.component.as_ref().unwrap().tick.as_ref().unwrap().value,
                        )
                        .map_err(|_| {
                            crate::scales::error(
                                crate::DiagnosticCode::Validation,
                                "Invalid legend identity.",
                            )
                        })?;
                        let occurrence = occurrences.entry(identity).or_default();
                        row.component
                            .as_mut()
                            .unwrap()
                            .tick
                            .as_mut()
                            .unwrap()
                            .occurrence = *occurrence;
                        *occurrence += 1;
                        index += 1;
                    }
                }
            }
        }
    }
    Ok(blocks)
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
        guide: label.component.clone(),
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
    labels: &[LegendBlock],
    bounds: Rect,
    r: &LayoutRequest,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<()> {
    let mut y = bounds.origin().y() + r.padding;
    let mut constrained = false;
    for block in labels {
        let label = match block {
            LegendBlock::Bins { key, labels } => {
                constrained |= paint_bins(key, labels, items, bounds, y, r)?;
                y += bins_dimensions(key, labels, r).1 + r.label_gap;
                continue;
            }
            LegendBlock::Grid {
                labels,
                columns,
                by_row,
            } => {
                let rows = labels.len().div_ceil(*columns);
                let cell_height =
                    labels.iter().map(|l| row_height(l, r)).fold(0., f64::max) + r.label_gap;
                let cell_width =
                    labels.iter().map(|l| row_width(l, r)).fold(0., f64::max) + r.padding;
                let mut inner = r.clone();
                inner.padding = 0.;
                for (index, label) in labels.iter().enumerate() {
                    let (row, column) = if *by_row {
                        (index / columns, index % columns)
                    } else {
                        (index % rows, index / rows)
                    };
                    let x = (bounds.origin().x() + column as f64 * cell_width).min(bounds.max_x());
                    let top = (y + row as f64 * cell_height).min(bounds.max_y());
                    let cell = Rect::new(
                        x,
                        top,
                        cell_width.min((bounds.max_x() - x).max(0.)),
                        cell_height.min((bounds.max_y() - top).max(0.)),
                    )?;
                    paint_legend(
                        items,
                        &[LegendBlock::Row(label.clone())],
                        cell,
                        &inner,
                        diagnostics,
                    )?;
                }
                y += rows as f64 * cell_height;
                constrained |=
                    y > bounds.max_y() || cell_width * (*columns as f64) > bounds.width();
                continue;
            }
            LegendBlock::Custom(custom) => {
                let map = crate::path::Affine::new([
                    1.,
                    0.,
                    0.,
                    1.,
                    bounds.origin().x() - custom.bounds[0],
                    y - custom.bounds[1],
                ])?;
                for (index, path) in custom.paths.iter().enumerate() {
                    require_within(items.len() < r.limits.max_items, "custom guide item")?;
                    let mut guide = legend_component(
                        custom.id,
                        crate::scene::GuideRole::LegendKey,
                        Some(index),
                        Some(String::new()),
                    );
                    guide.scope.push("custom".into());
                    items.push(SceneItem {
                        guide: Some(guide),
                        layer: None,
                        clip: Some(bounds),
                        primitive: Primitive::VectorPath {
                            geometry: path.geometry.transformed(
                                map,
                                0.01,
                                r.limits.max_path_commands,
                            )?,
                            fill: path.fill.map(crate::color::Paint::resolve),
                            stroke: path.stroke,
                            dashes: vec![],
                        },
                    });
                }
                constrained |=
                    custom.bounds[2] > bounds.width() || y + custom.bounds[3] > bounds.max_y();
                y += custom.bounds[3] + r.label_gap;
                continue;
            }
            LegendBlock::Row(label) => label,
            LegendBlock::Colorbar(bar) => {
                let height = bar.height(r);
                if y + height > bounds.max_y() - r.padding {
                    constrained = true;
                    break;
                }
                constrained |= bar.paint(items, bounds, y, r)?;
                y += height + r.label_gap;
                continue;
            }
        };
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
            let glyph_start = items.len();
            match glyph {
                LegendGlyph::Combined(glyphs) => {
                    for glyph in glyphs {
                        super::legend_keys::paint(glyph, items, bounds, y, height, width, r)?;
                    }
                }
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
            for item in &mut items[glyph_start..] {
                item.guide = label.component.clone().map(|mut c| {
                    c.role = crate::scene::GuideRole::LegendKey;
                    c
                });
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
        diagnostics.push(pressure("Legend pressure clipped or overlapped labels, or omitted trailing guide content; complete legend metadata remains in preparation."));
    }
    Ok(())
}

pub(super) struct SingleLegend {
    pub content: Rect,
    boxes: Vec<PlacedLegend>,
}
struct PlacedLegend {
    bounds: Rect,
    position: crate::grammar::LegendPosition,
    labels: Vec<LegendBlock>,
}

fn row_height(l: &Label, r: &LayoutRequest) -> f64 {
    l.metrics
        .height()
        .max(l.glyph.as_ref().map_or(0., |g| g.dimensions(r).1))
        .max(r.font_size)
}
fn row_width(l: &Label, r: &LayoutRequest) -> f64 {
    l.metrics.width()
        + l.glyph
            .as_ref()
            .map_or(0., |g| g.dimensions(r).0.max(r.font_size) + r.label_gap)
}
fn grid_height(labels: &[Label], columns: usize, r: &LayoutRequest) -> f64 {
    (labels.iter().map(|l| row_height(l, r)).fold(0., f64::max) + r.label_gap)
        * labels.len().div_ceil(columns) as f64
}
fn legend_width(labels: &[LegendBlock], width: f64, request: &LayoutRequest) -> f64 {
    if labels.is_empty() {
        0.
    } else {
        // Horizontal bars need their measured span even in local facet guides.
        // Preserve the legacy column cap for other legends and leave the common
        // solver the requested minimum plot span plus its outer padding.
        let cap = if labels.iter().any(|block| {
            matches!(block, LegendBlock::Grid { .. })
                || matches!(block, LegendBlock::Colorbar(bar) if bar.horizontal())
        }) {
            (width - request.minimum_plot.0 - 2. * request.padding).max(0.)
        } else {
            width * 0.3
        };
        (labels
            .iter()
            .map(|block| match block {
                LegendBlock::Bins { key, labels } => bins_dimensions(key, labels, request).0,
                LegendBlock::Custom(custom) => custom.bounds[2],
                LegendBlock::Colorbar(bar) => bar.width(request),
                LegendBlock::Grid {
                    labels, columns, ..
                } => {
                    (labels
                        .iter()
                        .map(|l| row_width(l, request))
                        .fold(0., f64::max)
                        + request.padding)
                        * (*columns as f64)
                }
                LegendBlock::Row(l) => {
                    l.metrics.width()
                        + l.glyph
                            .as_ref()
                            .map_or(request.font_size + request.label_gap, |g| {
                                g.dimensions(request).0.max(request.font_size) + request.label_gap
                            })
                }
            })
            .fold(0_f64, f64::max)
            + request.padding)
            .min(cap)
    }
}

fn arrange_legends(
    legends: &[Legend<'_>],
    definition: &crate::grammar::ChartDefinition,
    region: Rect,
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    remaining: &mut usize,
) -> ChartResult<SingleLegend> {
    use crate::grammar::LegendPosition as P;
    let mut groups: Vec<(P, Vec<Legend<'_>>)> = vec![];
    for legend in legends {
        let position = match legend {
            Legend::Custom(c) => c.options.position.clone(),
            Legend::Keys(k) => k.options.position.clone(),
            Legend::Color(c) => definition
                .legends
                .get(&c.id)
                .and_then(|o| o.position.clone()),
            _ => None,
        }
        .unwrap_or(P::Right);
        if let Some((_, group)) = groups.iter_mut().find(|(p, _)| *p == position) {
            group.push(legend.clone());
        } else {
            groups.push((position, vec![legend.clone()]));
        }
    }
    let mut content = region;
    let mut boxes = vec![];
    for (position, group) in groups {
        let labels = measure_legends(&group, request, measurer, remaining)?;
        if labels.is_empty() {
            continue;
        }
        let width = legend_width(&labels, region.width(), request).min(content.width());
        let desired_height = 2. * request.padding
            + labels
                .iter()
                .map(|b| match b {
                    LegendBlock::Bins { key, labels } => {
                        bins_dimensions(key, labels, request).1 + request.label_gap
                    }
                    LegendBlock::Custom(custom) => custom.bounds[3] + request.label_gap,
                    LegendBlock::Colorbar(bar) => bar.height(request) + request.label_gap,
                    LegendBlock::Grid {
                        labels, columns, ..
                    } => grid_height(labels, *columns, request),
                    LegendBlock::Row(l) => {
                        l.metrics
                            .height()
                            .max(l.glyph.as_ref().map_or(0., |g| g.dimensions(request).1))
                            .max(request.font_size)
                            + request.label_gap
                    }
                })
                .sum::<f64>();
        let height = desired_height.min(content.height() * 0.4);
        let bounds = match position {
            P::Right => {
                let b = Rect::new(
                    content.max_x() - width,
                    content.origin().y(),
                    width,
                    content.height(),
                )?;
                content = Rect::new(
                    content.origin().x(),
                    content.origin().y(),
                    content.width() - width,
                    content.height(),
                )?;
                b
            }
            P::Left => {
                let b = Rect::new(
                    content.origin().x(),
                    content.origin().y(),
                    width,
                    content.height(),
                )?;
                content = Rect::new(
                    content.origin().x() + width,
                    content.origin().y(),
                    content.width() - width,
                    content.height(),
                )?;
                b
            }
            P::Top => {
                let b = Rect::new(
                    content.origin().x(),
                    content.origin().y(),
                    content.width(),
                    height,
                )?;
                content = Rect::new(
                    content.origin().x(),
                    content.origin().y() + height,
                    content.width(),
                    content.height() - height,
                )?;
                b
            }
            P::Bottom => {
                let b = Rect::new(
                    content.origin().x(),
                    content.max_y() - height,
                    content.width(),
                    height,
                )?;
                content = Rect::new(
                    content.origin().x(),
                    content.origin().y(),
                    content.width(),
                    content.height() - height,
                )?;
                b
            }
            P::Inside { x, y } => Rect::new(
                region.origin().x() + (region.width() - width) * x,
                region.origin().y() + (region.height() - desired_height.min(region.height())) * y,
                width,
                desired_height.min(region.height()),
            )?,
        };
        boxes.push(PlacedLegend {
            bounds,
            position,
            labels,
        });
    }
    Ok(SingleLegend { content, boxes })
}
fn paint_boxes(
    items: &mut Vec<SceneItem>,
    boxes: &[PlacedLegend],
    inside: Option<Rect>,
    request: &LayoutRequest,
    diagnostics: &mut Vec<Diagnostic>,
    scope: Option<&str>,
) -> ChartResult<()> {
    use crate::grammar::LegendPosition as P;
    for b in boxes {
        let bounds = if let (P::Inside { x, y }, Some(plot)) = (&b.position, inside) {
            let width = b.bounds.width().min(plot.width());
            let height = b.bounds.height().min(plot.height());
            Rect::new(
                plot.origin().x() + (plot.width() - width) * x,
                plot.origin().y() + (plot.height() - height) * y,
                width,
                height,
            )?
        } else {
            b.bounds
        };
        let start = items.len();
        paint_legend(items, &b.labels, bounds, request, diagnostics)?;
        for item in &mut items[start..] {
            if let Some(c) = &mut item.guide {
                c.side = match b.position {
                    P::Left => AxisSide::Left,
                    P::Top => AxisSide::Top,
                    P::Bottom => AxisSide::Bottom,
                    _ => AxisSide::Right,
                };
                if let Some(scope) = scope {
                    c.scope.insert(0, scope.into());
                }
            }
        }
    }
    Ok(())
}
pub(super) fn prepare_single_legend(
    prepared: &PreparedChart,
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
) -> ChartResult<Option<SingleLegend>> {
    if prepared.definition().facets.is_some() {
        return Ok(None);
    }
    let result = arrange_legends(
        &legends(prepared, request)?,
        prepared.definition(),
        request.bounds,
        request,
        measurer,
        &mut request.limits.max_text_bytes.clone(),
    )?;
    Ok((!result.boxes.is_empty()).then_some(result))
}
pub(super) fn finish_single_legend(
    chart: &mut LaidOutChart,
    request: &LayoutRequest,
    legend: SingleLegend,
) -> ChartResult<()> {
    let mut items = chart.scene.items().to_vec();
    paint_boxes(
        &mut items,
        &legend.boxes,
        chart.plot(),
        request,
        &mut chart.diagnostics,
        None,
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
    let labels = if spec.reference.is_none() {
        measure_labels(
            prepared
                .panels()
                .iter()
                .map(|p| (panel_label(&p.key), None))
                .collect(),
            request,
            measurer,
            &mut remaining,
        )?
    } else {
        vec![]
    };
    let strips = if let Some(policy) = &spec.reference {
        prepared
            .panels()
            .iter()
            .map(|p| {
                super::facet_policy::strips(
                    policy,
                    &spec.layout,
                    p,
                    (rows, columns),
                    request,
                    measurer,
                    &mut remaining,
                )
            })
            .collect::<ChartResult<Vec<_>>>()?
    } else {
        vec![]
    };
    let strip_insets = super::facet_policy::insets(&strips, request.label_gap);
    let header_height = if spec.reference.is_none() {
        labels
            .iter()
            .map(|l| l.metrics.height())
            .fold(0_f64, f64::max)
            + request.label_gap
    } else {
        0.
    };
    let shared_legends = if spec.collect_guides {
        legends(&prepared, request)?
    } else {
        vec![]
    };
    let shared = arrange_legends(
        &shared_legends,
        prepared.definition(),
        request.bounds,
        request,
        measurer,
        &mut remaining,
    )?;
    let grid_reference =
        spec.reference.is_some() && matches!(spec.layout, crate::grammar::FacetLayout::Grid);
    let content = if grid_reference {
        super::facet_policy::inset(shared.content, strip_insets)?
    } else {
        shared.content
    };
    let cell_width = (content.width() - spec.gap * (columns - 1) as f64) / columns as f64;
    let cell_height = (content.height() - spec.gap * (rows - 1) as f64) / rows as f64;
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
    let mut widths = vec![cell_width; columns];
    let mut heights = vec![cell_height; rows];
    let mut xweights = vec![1_f64; columns];
    let mut yweights = vec![1_f64; rows];
    if let Some(policy) = &spec.reference {
        for panel in prepared.panels() {
            if policy.space.free(true) {
                xweights[panel.column] = super::facet_policy::span(&panel.chart, request, true)?;
            }
            if policy.space.free(false) {
                yweights[panel.row] = super::facet_policy::span(&panel.chart, request, false)?;
            }
        }
    }
    let mut cells = vec![];
    let mut local_boxes = vec![];
    let mut resolved = vec![];
    let weighted = spec
        .reference
        .as_ref()
        .is_some_and(|p| p.space != crate::grammar::FacetSpace::Fixed);
    for pass in 0..if weighted { 2 } else { 1 } {
        cells.clear();
        local_boxes.clear();
        let mut inputs = vec![];
        for panel in prepared.panels() {
            let cell = Rect::new(
                content.origin().x()
                    + widths[..panel.column].iter().sum::<f64>()
                    + panel.column as f64 * spec.gap,
                content.origin().y()
                    + heights[..panel.row].iter().sum::<f64>()
                    + panel.row as f64 * spec.gap,
                widths[panel.column],
                heights[panel.row],
            )?;
            let mut r = request.clone();
            r.hierarchy_scope
                .push(super::GuideScope::Panel(panel.key.clone()));
            r.figure_bounds = Some(request.figure_bounds.unwrap_or(request.bounds));
            let inset = if spec.reference.is_some() && !grid_reference {
                strip_insets
            } else {
                [0., 0., header_height, 0.]
            };
            let region = super::facet_policy::inset(cell, inset)?;
            let local = arrange_legends(
                &if spec.collect_guides {
                    vec![]
                } else {
                    legends(&panel.chart, request)?
                },
                prepared.definition(),
                region,
                request,
                measurer,
                &mut remaining,
            )?;
            r.bounds = local.content;
            local_boxes.push(local.boxes);
            if let Some(policy) = &spec.reference {
                super::facet_policy::apply_axes(
                    policy,
                    &spec.layout,
                    panel,
                    prepared.panels(),
                    &mut r,
                );
            }
            let mut chart = (*panel.chart).clone();
            for (id, domain) in prepared.scale_domains() {
                chart.scale_domains.insert(*id, domain.clone());
            }
            chart.shared_training = Some(prepared.clone());
            inputs.push((Arc::new(chart), r));
            cells.push(cell);
        }
        let offsets = strips
            .iter()
            .map(|panel| {
                let mut offsets = [0.; 4];
                for strip in panel {
                    let side = super::facet_policy::side_index(strip.side);
                    offsets[side] = strip_insets[side];
                }
                offsets
            })
            .collect::<Vec<_>>();
        resolved =
            super::engine::solve_panels_with_strip_offsets(inputs, &offsets, measurer, stamp)?;
        if weighted && pass == 0 {
            let mut margin = [0_f64; 2];
            for (cell, chart) in cells.iter().zip(&resolved) {
                if let Some(plot) = chart.plot() {
                    margin[0] = margin[0].max(cell.width() - plot.width());
                    margin[1] = margin[1].max(cell.height() - plot.height());
                }
            }
            if let Some(policy) = &spec.reference {
                if policy.space.free(true) {
                    let usable = (content.width()
                        - spec.gap * (columns - 1) as f64
                        - margin[0] * columns as f64)
                        .max(0.);
                    let total = xweights.iter().sum::<f64>();
                    widths = xweights
                        .iter()
                        .map(|w| margin[0] + usable * w / total)
                        .collect();
                }
                if policy.space.free(false) {
                    let usable =
                        (content.height() - spec.gap * (rows - 1) as f64 - margin[1] * rows as f64)
                            .max(0.);
                    let total = yweights.iter().sum::<f64>();
                    heights = yweights
                        .iter()
                        .map(|w| margin[1] + usable * w / total)
                        .collect();
                }
            }
        }
    }
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
        if spec.reference.is_none() {
            if labels[index].metrics.width() + request.padding > cells[index].width() {
                diagnostics.push(pressure("Facet header exceeds its cell and is clipped; its full logical text is retained."));
            }
            push_text(
                &mut items,
                &labels[index],
                cells[index].origin().x() + request.padding,
                cells[index].origin().y(),
                cells[index],
                request,
            )?;
        } else {
            for strip in &strips[index] {
                let cell = cells[index];
                let side = strip.side;
                let i = super::facet_policy::side_index(side);
                let size = strip_insets[i];
                let plot = chart.plot().unwrap_or(cell);
                let bounds = match side {
                    AxisSide::Top => Rect::new(
                        plot.origin().x(),
                        plot.origin().y() - size,
                        plot.width(),
                        size,
                    )?,
                    AxisSide::Bottom => {
                        Rect::new(plot.origin().x(), plot.max_y(), plot.width(), size)?
                    }
                    AxisSide::Left => Rect::new(
                        plot.origin().x() - size,
                        plot.origin().y(),
                        size,
                        plot.height(),
                    )?,
                    AxisSide::Right => {
                        Rect::new(plot.max_x(), plot.origin().y(), size, plot.height())?
                    }
                };
                require_within(
                    items.len() < request.limits.max_items,
                    "facet strip background",
                )?;
                items.push(SceneItem {
                    guide: None,
                    layer: None,
                    clip: Some(bounds),
                    primitive: Primitive::Rectangle {
                        bounds,
                        fill: Color {
                            red: 217,
                            green: 217,
                            blue: 217,
                            alpha: 255,
                        },
                    },
                });
                let text = strip.block.items_at(
                    bounds.origin().x() + (bounds.width() - strip.block.bounds.width()) / 2.,
                    bounds.origin().y() + (bounds.height() - strip.block.bounds.height()) / 2.,
                    bounds,
                )?;
                require_within(
                    text.len() <= request.limits.max_items.saturating_sub(items.len()),
                    "facet strip text",
                )?;
                items.extend(text);
                diagnostics.extend(strip.block.diagnostics.clone());
            }
        }
        paint_boxes(
            &mut items,
            &local_boxes[index],
            chart.plot(),
            request,
            &mut diagnostics,
            Some(&scope),
        )?;
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
    paint_boxes(
        &mut items,
        &shared.boxes,
        Some(shared.content),
        request,
        &mut diagnostics,
        None,
    )?;
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

pub(super) fn legend_component(
    id: crate::ScaleId,
    role: crate::scene::GuideRole,
    index: Option<usize>,
    label: Option<String>,
) -> crate::scene::GuideComponent {
    crate::scene::GuideComponent {
        animation: None,
        scope: vec!["legend".into()],
        guide: crate::GuideId::new(id.get()),
        role,
        side: AxisSide::Right,
        tick: index.map(|_| crate::scene::GuideTickIdentity {
            value: crate::composition::ScaleValue::Category(label.clone().unwrap_or_default()),
            occurrence: 0,
        }),
        index,
        label,
    }
}

fn bins_horizontal(key: &super::legend_keys::KeyLegend) -> bool {
    key.options.direction == Some(crate::scene::GradientDirection::Horizontal)
        || (key.options.direction.is_none()
            && matches!(
                key.options.position,
                Some(crate::grammar::LegendPosition::Top | crate::grammar::LegendPosition::Bottom)
            ))
}
fn bins_cell(key: &super::legend_keys::KeyLegend, r: &LayoutRequest) -> (f64, f64) {
    key.glyphs
        .iter()
        .flatten()
        .fold((r.font_size * 2., r.font_size * 2.), |(w, h), g| {
            (
                w.max(if g.line.is_some() {
                    r.font_size * 2.
                } else {
                    g.size * 2. + g.width
                }),
                h.max(g.size * 2. + g.width),
            )
        })
}
fn bins_dimensions(
    key: &super::legend_keys::KeyLegend,
    labels: &[Label],
    r: &LayoutRequest,
) -> (f64, f64) {
    let (w, h) = bins_cell(key, r);
    let n = labels.len().saturating_sub(1) as f64;
    if bins_horizontal(key) {
        (
            w * n
                + if key.show_limits {
                    (labels[0].metrics.width() + labels[labels.len() - 1].metrics.width()) / 2.
                } else {
                    0.
                },
            h + r.label_gap + labels.iter().map(|l| l.metrics.height()).fold(0., f64::max),
        )
    } else {
        (
            w + r.label_gap + labels.iter().map(|l| l.metrics.width()).fold(0., f64::max),
            h * n + labels.iter().map(|l| l.metrics.height()).fold(0., f64::max),
        )
    }
}
fn paint_bins(
    key: &super::legend_keys::KeyLegend,
    labels: &[Label],
    items: &mut Vec<SceneItem>,
    bounds: Rect,
    y: f64,
    r: &LayoutRequest,
) -> ChartResult<bool> {
    let (w, h) = bins_cell(key, r);
    let horizontal = bins_horizontal(key);
    let n = labels.len() - 1;
    let x = bounds.origin().x()
        + if horizontal && key.show_limits {
            labels[0].metrics.width() / 2.
        } else {
            0.
        };
    let top = y + if horizontal {
        0.
    } else {
        labels.iter().map(|l| l.metrics.height()).fold(0., f64::max) / 2.
    };
    for (i, glyphs) in key.glyphs.iter().take(n).enumerate() {
        let (gx, gy) = if horizontal {
            (x + i as f64 * w, top)
        } else {
            (x, top + (n - 1 - i) as f64 * h)
        };
        let cell = Rect::new(gx, gy, w, h)?;
        for glyph in glyphs {
            let start = items.len();
            super::legend_keys::paint(glyph, items, cell, gy, h, w, r)?;
            for item in &mut items[start..] {
                item.clip = Some(bounds);
                item.guide = labels[i].component.clone().map(|mut c| {
                    c.role = crate::scene::GuideRole::LegendKey;
                    c
                });
            }
        }
    }
    require_within(
        items.len().saturating_add(1 + labels.len() * 2) <= r.limits.max_items,
        "binned guide item",
    )?;
    let axis_from = Point::new(
        if horizontal { x } else { x + w },
        if horizontal { top + h } else { top },
    )?;
    let axis_to = Point::new(
        if horizontal { x + w * n as f64 } else { x + w },
        if horizontal {
            top + h
        } else {
            top + h * n as f64
        },
    )?;
    let mut axis = legend_component(key.id, crate::scene::GuideRole::LegendBar, None, None);
    axis.side = if horizontal {
        AxisSide::Bottom
    } else {
        AxisSide::Right
    };
    items.push(SceneItem {
        guide: Some(axis),
        layer: None,
        clip: Some(bounds),
        primitive: Primitive::Rule {
            from: axis_from,
            to: axis_to,
            stroke: crate::scene::Stroke {
                color: INK,
                width: 0.5,
            },
        },
    });
    let mut constrained = false;
    for (i, label) in labels.iter().enumerate() {
        if !key.show_limits && (i == 0 || i == n) {
            continue;
        }
        let (tx, ty) = if horizontal {
            (x + i as f64 * w, top + h)
        } else {
            (x + w, top + (n - i) as f64 * h)
        };
        let to = Point::new(
            tx + if horizontal { 0. } else { r.tick_length },
            ty + if horizontal { r.tick_length } else { 0. },
        )?;
        items.push(SceneItem {
            guide: label.component.clone().map(|mut c| {
                c.role = crate::scene::GuideRole::LegendTick;
                c
            }),
            layer: None,
            clip: Some(bounds),
            primitive: Primitive::Rule {
                from: Point::new(tx, ty)?,
                to,
                stroke: crate::scene::Stroke {
                    color: INK,
                    width: 0.5,
                },
            },
        });
        let (lx, ly) = if horizontal {
            (tx - label.metrics.width() / 2., ty + r.label_gap)
        } else {
            (tx + r.label_gap, ty - label.metrics.height() / 2.)
        };
        constrained |= lx < bounds.origin().x()
            || lx + label.metrics.width() > bounds.max_x()
            || ly + label.metrics.height() > bounds.max_y();
        push_text(items, label, lx, ly, bounds, r)?;
    }
    Ok(constrained)
}
