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
    hidden: bool,
    rich: Option<super::text::Block>,
    component: Option<crate::scene::GuideComponent>,
    text: String,
    metrics: TextMetrics,
    glyph: Option<LegendGlyph>,
}

enum LegendBlock {
    GuideBox {
        groups: Vec<Vec<LegendBlock>>,
        horizontal: bool,
    },
    Titled {
        title: Box<Label>,
        body: Vec<LegendBlock>,
        left: bool,
    },
    Bins {
        key: Box<super::legend_keys::KeyLegend>,
        labels: Vec<Label>,
    },
    Custom(Box<crate::grammar::CustomLegend>),
    Row(Box<Label>),
    Grid {
        labels: Vec<Label>,
        columns: usize,
        by_row: bool,
    },
    Colorbar(super::legend_colorbar::Colorbar),
}

#[derive(Clone, PartialEq)]
enum Legend<'a> {
    Custom(Box<std::borrow::Cow<'a, crate::grammar::CustomLegend>>),
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
        let (w, h) = match self {
            Self::Combined(glyphs) => {
                glyphs.iter().fold((r.font_size, r.font_size), |(w, h), g| {
                    if let Some(custom) = &g.custom {
                        return (w.max(custom.size[0]), h.max(custom.size[1]));
                    }
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
        };
        let key = |name| {
            r.resolved_theme
                .as_ref()
                .and_then(|e| e.destination_length(name, "", 0))
                .unwrap_or(0.)
        };
        (
            w.max(key("legend.key.width")),
            h.max(key("legend.key.height")),
        )
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
    let reference = chart
        .definition()
        .legends
        .values()
        .any(|o| o.registered.is_some())
        || chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        || chart.definition().layers.iter().any(|l| {
            l.legend
                .as_ref()
                .is_some_and(|l| l.registered_key.is_some())
        });
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
    if let Some(elements) = &request.resolved_theme {
        let position = super::theme_elements::legend_position(elements);
        let direction = match elements.value("legend.direction", "") {
            Some(crate::theme::ThemeValue::Text(v)) if v == "horizontal" => {
                Some(crate::scene::GradientDirection::Horizontal)
            }
            Some(crate::theme::ThemeValue::Text(v)) if v == "vertical" => {
                Some(crate::scene::GradientDirection::Vertical)
            }
            _ => None,
        };
        for legend in &mut legends {
            if let Legend::Keys(keys) = legend {
                keys.options.position = keys.options.position.clone().or(position.clone());
                keys.options.direction = keys.options.direction.or(direction);
            }
        }
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
    for legend in &mut legends {
        let (id, options, title, labels, values, colors, color_guide) = match &*legend {
            Legend::Keys(k) => (
                k.id,
                Some(&k.options),
                k.title.clone(),
                k.labels.clone(),
                k.values.clone(),
                k.glyphs
                    .iter()
                    .map(|g| g.first().map_or(crate::theme::rgb(0, 0, 0), |g| g.color))
                    .collect::<Vec<_>>(),
                None,
            ),
            Legend::Color(c) => (
                c.id,
                chart.definition().legends.get(&c.id),
                c.title.clone().unwrap_or_default(),
                c.entries.iter().map(|(l, _)| l.clone()).collect(),
                vec![],
                c.entries.iter().map(|(_, c)| *c).collect(),
                Some(c.as_ref().as_ref()),
            ),
            _ => continue,
        };
        if let Some(options) = options
            && let Some(selection) = &options.registered
        {
            let custom = chart.guide_drawing.draw(
                selection,
                crate::grammar::GuideDrawingInput {
                    scale: id,
                    title: &title,
                    labels: &labels,
                    values: &values,
                    colors: &colors,
                    color_guide,
                    parameters: &selection.parameters,
                    units: request.units,
                    font_size: request.font_size,
                    limits: request.limits,
                },
                options.clone(),
            )?;
            *legend = Legend::Custom(Box::new(std::borrow::Cow::Owned(custom)));
        }
    }
    legends.extend(
        chart
            .definition()
            .custom_legends
            .iter()
            .map(|c| Legend::Custom(Box::new(std::borrow::Cow::Borrowed(c)))),
    );
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
                hidden: false,
                rich: None,
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
    definition: &crate::grammar::ChartDefinition,
    legends: &[Legend<'_>],
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    remaining: &mut usize,
) -> ChartResult<Vec<LegendBlock>> {
    let elements = request.resolved_theme.as_deref();
    let mut blocks = vec![];
    let mut guide_groups = Vec::new();
    let box_layout = elements.is_some() && legends.len() > 1;
    for legend in legends {
        let start = blocks.len();
        let math = match legend {
            Legend::Custom(c) => c.options.math.as_ref(),
            Legend::Keys(k) => k.options.math.as_ref(),
            Legend::Color(c) => definition.legends.get(&c.id).and_then(|o| o.math.as_ref()),
            Legend::Symbol(_) => None,
        };
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
                    blocks.push(LegendBlock::Row(Box::new(row)));
                }
            }
            blocks.push(LegendBlock::Custom(Box::new(
                custom.as_ref().as_ref().clone(),
            )));
        } else if let Legend::Color(color) = legend
            && (!color.colorbar.is_empty() || !color.colorsteps.is_empty())
        {
            let title = color.title.clone().unwrap_or_else(|| "Value".into());
            if !title.is_empty() {
                blocks.extend(
                    measure_labels(vec![(title, None)], request, measurer, remaining)?
                        .into_iter()
                        .map(|row| LegendBlock::Row(Box::new(row))),
                );
            }
            blocks.push(LegendBlock::Colorbar(
                super::legend_colorbar::Colorbar::measure(
                    color, request, measurer, remaining, math,
                )?,
            ));
        } else if let Legend::Keys(keys) = legend {
            let mut labels = measure_labels(
                legend_values(std::slice::from_ref(legend), request)?,
                request,
                measurer,
                remaining,
            )?;
            if !keys.title.is_empty() {
                blocks.push(LegendBlock::Row(Box::new(labels.remove(0))));
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
                    by_row: keys.options.by_row
                        || request.resolved_theme.as_ref().is_some_and(|e| {
                            matches!(
                                e.value("legend.byrow", ""),
                                Some(crate::theme::ThemeValue::Bool(true))
                            )
                        }),
                });
            } else {
                blocks.extend(
                    labels
                        .into_iter()
                        .map(|row| LegendBlock::Row(Box::new(row))),
                );
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
                .map(|row| LegendBlock::Row(Box::new(row))),
            );
        }
        if math.is_some() || elements.is_some() {
            for row in blocks[start..].iter_mut().flat_map(|block| match block {
                LegendBlock::Row(row) => std::slice::from_mut(row.as_mut()),
                LegendBlock::Grid { labels, .. } | LegendBlock::Bins { labels, .. } => {
                    labels.as_mut_slice()
                }
                _ => &mut [],
            }) {
                let text = if let Some(fonts) = math {
                    crate::typography::RichText::math(&row.text, fonts.clone())?
                } else {
                    crate::typography::RichText::plain(&row.text)
                };
                let text = if let Some(elements) = &elements {
                    super::theme_elements::text_style(
                        elements,
                        if row.glyph.is_none() {
                            "legend.title"
                        } else {
                            "legend.text"
                        },
                        &text,
                        request,
                    )?
                } else {
                    Some(text)
                };
                let Some(text) = text else {
                    row.hidden = true;
                    row.metrics = TextMetrics::new(0., 0., 0.)?;
                    continue;
                };
                let mut block = super::text::measure(
                    &text,
                    request,
                    measurer,
                    request
                        .host_theme
                        .foreground
                        .map(crate::color::Paint::resolve)
                        .unwrap_or(INK),
                )?;
                if let Some(elements) = &elements {
                    super::theme_elements::margins(
                        elements,
                        if row.glyph.is_none() {
                            "legend.title"
                        } else {
                            "legend.text"
                        },
                        &mut block,
                        request,
                    )?;
                }
                row.metrics = TextMetrics::new(block.bounds.width(), block.bounds.height(), 0.)?;
                row.rich = Some(block);
            }
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
                LegendBlock::Row(row) => std::slice::from_mut(row.as_mut()),
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
        if blocks.len() > start + 1
            && matches!(&blocks[start],LegendBlock::Row(l) if l.glyph.is_none()&&!l.hidden)
        {
            let position = elements
                .as_ref()
                .and_then(|e| e.value("legend.title.position", ""));
            if matches!(position,Some(crate::theme::ThemeValue::Text(v)) if v=="bottom") {
                let title = blocks.remove(start);
                blocks.push(title);
            } else if let Some(crate::theme::ThemeValue::Text(v)) = position
                && matches!(v.as_str(), "left" | "right")
            {
                let mut group = blocks.split_off(start);
                let LegendBlock::Row(title) = group.remove(0) else {
                    unreachable!()
                };
                blocks.push(LegendBlock::Titled {
                    title,
                    body: group,
                    left: v == "left",
                });
            }
        }
        blocks.retain(
            |block| !matches!(block,LegendBlock::Row(row) if row.hidden && row.glyph.is_none()),
        );
        if box_layout {
            guide_groups.push(blocks.split_off(start));
        }
    }
    if box_layout {
        let horizontal=elements.is_some_and(|e|match e.value("legend.box","") {Some(crate::theme::ThemeValue::Text(v))=>v=="horizontal",_=>matches!(e.value("legend.position",""),Some(crate::theme::ThemeValue::Text(v)) if v=="top"||v=="bottom")});
        blocks.push(LegendBlock::GuideBox {
            groups: guide_groups,
            horizontal,
        });
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
    if label.hidden {
        return Ok(());
    }
    if let Some(block) = &label.rich {
        let added = block.items_at(x, y, clip)?;
        require_within(
            items.len().saturating_add(added.len()) <= r.limits.max_items,
            "figure scene items",
        )?;
        items.extend(added.into_iter().map(|mut item| {
            item.guide = label.component.clone();
            item
        }));
        return Ok(());
    }
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

fn box_margins(r: &LayoutRequest) -> [f64; 4] {
    std::array::from_fn(|i| {
        r.resolved_theme
            .as_ref()
            .and_then(|e| e.destination_length("legend.box.margin", "", i))
            .unwrap_or(0.)
    })
}
fn outer_legend_margins(labels: &[LegendBlock], r: &LayoutRequest) -> [f64; 4] {
    if matches!(labels, [LegendBlock::GuideBox { .. }]) {
        box_margins(r)
    } else {
        legend_margins(r)
    }
}
fn guide_spacing(r: &LayoutRequest, horizontal: bool) -> f64 {
    r.resolved_theme
        .as_ref()
        .and_then(|e| {
            e.destination_length(
                if horizontal {
                    "legend.spacing.x"
                } else {
                    "legend.spacing.y"
                },
                "",
                0,
            )
        })
        .unwrap_or(r.label_gap)
}
fn group_dimensions(blocks: &[LegendBlock], r: &LayoutRequest) -> (f64, f64) {
    let m = legend_margins(r);
    let (w, h) = blocks
        .iter()
        .map(|b| block_dimensions(b, r))
        .fold((0_f64, 0.), |(w, h), (bw, bh)| {
            (w.max(bw), h + bh + r.label_gap)
        });
    (w + m[1] + m[3], h + m[0] + m[2])
}
fn legend_margins(r: &LayoutRequest) -> [f64; 4] {
    std::array::from_fn(|i| {
        r.resolved_theme
            .as_ref()
            .and_then(|e| e.destination_length("legend.margin", "", i))
            .unwrap_or(if i == 0 || i == 2 { r.padding } else { 0. })
    })
}
fn paint_legend(
    items: &mut Vec<SceneItem>,
    labels: &[LegendBlock],
    bounds: Rect,
    r: &LayoutRequest,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<()> {
    paint_legend_inner(items, labels, bounds, r, diagnostics, true)
}
fn paint_legend_inner(
    items: &mut Vec<SceneItem>,
    labels: &[LegendBlock],
    bounds: Rect,
    r: &LayoutRequest,
    diagnostics: &mut Vec<Diagnostic>,
    inset: bool,
) -> ChartResult<()> {
    let margins = if inset {
        outer_legend_margins(labels, r)
    } else {
        [0.; 4]
    };
    let bounds = if r.resolved_theme.is_some() {
        Rect::new(
            bounds.origin().x() + margins[3],
            bounds.origin().y() + margins[0],
            (bounds.width() - margins[1] - margins[3]).max(0.),
            (bounds.height() - margins[0] - margins[2]).max(0.),
        )?
    } else {
        bounds
    };
    let mut y = bounds.origin().y()
        + if r.resolved_theme.is_none() {
            r.padding
        } else {
            0.
        };
    let mut constrained = false;
    for block in labels {
        let label = match block {
            LegendBlock::GuideBox { groups, horizontal } => {
                let dimensions = block_dimensions(block, r);
                let mut advance = 0.;
                let justification = r
                    .resolved_theme
                    .as_ref()
                    .map(|e| super::theme_elements::justification(e, "legend.box.just"))
                    .unwrap_or([0.5, 0.5]);
                for group in groups {
                    let (width, height) = group_dimensions(group, r);
                    let x = bounds.origin().x()
                        + if *horizontal {
                            advance
                        } else {
                            (dimensions.0 - width) * justification[0]
                        };
                    let top = y + if *horizontal {
                        (dimensions.1 - height) * (1. - justification[1])
                    } else {
                        advance
                    };
                    let region = Rect::new(
                        x,
                        top,
                        width.min((bounds.max_x() - x).max(0.)),
                        height.min((bounds.max_y() - top).max(0.)),
                    )?;
                    if let Some(elements) = &r.resolved_theme {
                        let mut remaining = r.limits.max_path_commands;
                        for primitive in super::theme_elements::rectangle(
                            elements,
                            "legend.background",
                            region,
                            r,
                            &mut remaining,
                        )? {
                            require_within(
                                items.len() < r.limits.max_items,
                                "legend group background",
                            )?;
                            items.push(SceneItem {
                                guide: None,
                                layer: None,
                                clip: Some(bounds),
                                primitive,
                            });
                        }
                    }
                    paint_legend_inner(items, group, region, r, diagnostics, true)?;
                    advance += if *horizontal { width } else { height };
                    advance += guide_spacing(r, *horizontal);
                }
                y += dimensions.1;
                continue;
            }
            LegendBlock::Titled { title, body, left } => {
                let width = title.metrics.width() + r.label_gap;
                let height = block_dimensions(block, r).1;
                let body_x = bounds.origin().x() + if *left { width } else { 0. };
                let title_x = if *left {
                    bounds.origin().x()
                } else {
                    bounds.max_x() - title.metrics.width()
                };
                push_text(items, title, title_x, y, bounds, r)?;
                let body_bounds = Rect::new(
                    body_x,
                    y,
                    (bounds.width() - width).max(0.),
                    height.min((bounds.max_y() - y).max(0.)),
                )?;
                let mut inner = r.clone();
                inner.padding = 0.;
                paint_legend_inner(items, body, body_bounds, &inner, diagnostics, false)?;
                y += height + r.label_gap;
                continue;
            }
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
                let cell_height = labels.iter().map(|l| row_height(l, r)).fold(0., f64::max)
                    + key_spacing(r, false);
                let cell_width = labels.iter().map(|l| row_width(l, r)).fold(0., f64::max)
                    + key_spacing(r, true);
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
                    paint_legend_inner(
                        items,
                        &[LegendBlock::Row(Box::new(label.clone()))],
                        cell,
                        &inner,
                        diagnostics,
                        false,
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
                if y + height
                    > bounds.max_y()
                        - if r.resolved_theme.is_none() {
                            r.padding
                        } else {
                            0.
                        }
                {
                    constrained = true;
                    break;
                }
                constrained |= bar.paint(items, bounds, y, r)?;
                y += height + r.label_gap;
                continue;
            }
        };
        let height = row_height(label, r);
        if y + height
            > bounds.max_y()
                - if r.resolved_theme.is_none() {
                    r.padding
                } else {
                    0.
                }
        {
            constrained = true;
            break;
        }
        let swatch = if let Some(glyph) = &label.glyph {
            require_within(items.len() < r.limits.max_items, "legend swatch item")?;
            let (width, glyph_height) = glyph.dimensions(r);
            let position = legend_text_position(r);
            let y = if position == "top" {
                y + label.metrics.height() + r.label_gap
            } else {
                y
            };
            let bounds = if position == "left" {
                Rect::new(
                    bounds.origin().x() + label.metrics.width() + r.label_gap,
                    bounds.origin().y(),
                    (bounds.width() - label.metrics.width() - r.label_gap).max(0.),
                    bounds.height(),
                )?
            } else {
                bounds
            };
            let height = if matches!(position, "top" | "bottom") {
                glyph_height
            } else {
                height
            };
            let glyph_start = items.len();
            if let Some(elements) = &r.resolved_theme {
                let mut remaining = r.limits.max_path_commands;
                let key = Rect::new(
                    bounds.origin().x(),
                    y + (height - glyph_height) / 2.,
                    width,
                    glyph_height,
                )?;
                for primitive in super::theme_elements::rectangle(
                    elements,
                    "legend.key",
                    key,
                    r,
                    &mut remaining,
                )? {
                    require_within(items.len() < r.limits.max_items, "legend key background")?;
                    items.push(SceneItem {
                        guide: None,
                        layer: None,
                        clip: Some(bounds),
                        primitive,
                    });
                }
            }

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
        let position = legend_text_position(r);
        let x = bounds.origin().x()
            + if label.glyph.is_some() && position == "right" {
                swatch
            } else {
                0.
            };
        let text_y = if let Some(glyph) = &label.glyph
            && position == "bottom"
        {
            y + glyph.dimensions(r).1 + r.label_gap
        } else {
            y
        };
        constrained |= x + label.metrics.width() > bounds.max_x();
        push_text(items, label, x, text_y, bounds, r)?;
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

fn legend_text_position(r: &LayoutRequest) -> &str {
    match r
        .resolved_theme
        .as_ref()
        .and_then(|e| e.value("legend.text.position", ""))
    {
        Some(crate::theme::ThemeValue::Text(v)) => v,
        _ => "right",
    }
}
fn key_spacing(r: &LayoutRequest, horizontal: bool) -> f64 {
    r.resolved_theme
        .as_ref()
        .and_then(|e| {
            e.destination_length(
                if horizontal {
                    "legend.key.spacing.x"
                } else {
                    "legend.key.spacing.y"
                },
                "",
                0,
            )
        })
        .unwrap_or(if horizontal { r.padding } else { r.label_gap })
}
fn row_height(l: &Label, r: &LayoutRequest) -> f64 {
    let glyph = l.glyph.as_ref().map_or(0., |g| g.dimensions(r).1);
    if glyph > 0. && matches!(legend_text_position(r), "top" | "bottom") {
        glyph + l.metrics.height() + r.label_gap
    } else {
        l.metrics.height().max(glyph).max(r.font_size)
    }
}
fn row_width(l: &Label, r: &LayoutRequest) -> f64 {
    let glyph = l
        .glyph
        .as_ref()
        .map_or(0., |g| g.dimensions(r).0.max(r.font_size));
    if glyph > 0. && matches!(legend_text_position(r), "top" | "bottom") {
        l.metrics.width().max(glyph)
    } else {
        l.metrics.width() + if glyph > 0. { glyph + r.label_gap } else { 0. }
    }
}
fn grid_height(labels: &[Label], columns: usize, r: &LayoutRequest) -> f64 {
    (labels.iter().map(|l| row_height(l, r)).fold(0., f64::max) + key_spacing(r, false))
        * labels.len().div_ceil(columns) as f64
}
fn block_dimensions(block: &LegendBlock, r: &LayoutRequest) -> (f64, f64) {
    match block {
        LegendBlock::GuideBox { groups, horizontal } => {
            let mut size = (0_f64, 0_f64);
            for group in groups {
                let (w, h) = group_dimensions(group, r);
                if *horizontal {
                    size.0 += w;
                    size.1 = size.1.max(h);
                } else {
                    size.0 = size.0.max(w);
                    size.1 += h;
                }
            }
            let spacing = guide_spacing(r, *horizontal) * groups.len().saturating_sub(1) as f64;
            if *horizontal {
                size.0 += spacing;
            } else {
                size.1 += spacing;
            }
            size
        }
        LegendBlock::Titled { title, body, .. } => {
            let (w, h) = body
                .iter()
                .map(|b| block_dimensions(b, r))
                .fold((0_f64, 0.), |(w, h), (bw, bh)| {
                    (w.max(bw), h + bh + r.label_gap)
                });
            (
                title.metrics.width() + r.label_gap + w,
                title.metrics.height().max(h),
            )
        }
        LegendBlock::Bins { key, labels } => bins_dimensions(key, labels, r),
        LegendBlock::Custom(c) => (c.bounds[2], c.bounds[3]),
        LegendBlock::Colorbar(b) => (b.width(r), b.height(r)),
        LegendBlock::Grid {
            labels, columns, ..
        } => (
            (labels.iter().map(|l| row_width(l, r)).fold(0., f64::max) + key_spacing(r, true))
                * *columns as f64,
            grid_height(labels, *columns, r),
        ),
        LegendBlock::Row(l) => (
            if r.resolved_theme.is_none() {
                l.metrics.width()
                    + l.glyph.as_ref().map_or(r.font_size + r.label_gap, |g| {
                        g.dimensions(r).0.max(r.font_size) + r.label_gap
                    })
            } else {
                row_width(l, r)
            },
            row_height(l, r),
        ),
    }
}
fn legend_width(labels: &[LegendBlock], width: f64, request: &LayoutRequest) -> f64 {
    if labels.is_empty() {
        0.
    } else {
        // Horizontal bars need their measured span even in local facet guides.
        // Preserve the legacy column cap for other legends and leave the common
        // solver the requested minimum plot span plus its outer padding.
        let cap = if labels.iter().any(|block| {
            matches!(
                block,
                LegendBlock::Grid { .. }
                    | LegendBlock::GuideBox {
                        horizontal: true,
                        ..
                    }
            ) || matches!(block, LegendBlock::Colorbar(bar) if bar.horizontal())
        }) {
            (width - request.minimum_plot.0 - 2. * request.padding).max(0.)
        } else {
            width * 0.3
        };
        (labels
            .iter()
            .map(|block| block_dimensions(block, request).0)
            .fold(0_f64, f64::max)
            + if request.resolved_theme.is_some() {
                let m = outer_legend_margins(labels, request);
                m[1] + m[3]
            } else {
                request.padding
            })
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
        let override_position = match legend {
            Legend::Custom(c) => c.options.position.clone(),
            Legend::Keys(k) => k.options.position.clone(),
            Legend::Color(c) => definition
                .legends
                .get(&c.id)
                .and_then(|o| o.position.clone()),
            _ => None,
        };
        let position = override_position
            .clone()
            .or_else(|| {
                request
                    .resolved_theme
                    .as_ref()
                    .and_then(|e| super::theme_elements::legend_position(e))
            })
            .unwrap_or(P::Right);
        if override_position.is_none() && request.resolved_theme.as_ref().is_some_and(|e|matches!(e.value("legend.position",""),Some(crate::theme::ThemeValue::Text(v))if v=="none")) {continue;}
        if let Some((_, group)) = groups.iter_mut().find(|(p, _)| *p == position) {
            group.push(legend.clone());
        } else {
            groups.push((position, vec![legend.clone()]));
        }
    }
    let mut content = region;
    let mut boxes = vec![];
    for (position, group) in groups {
        let labels = measure_legends(definition, &group, request, measurer, remaining)?;
        if labels.is_empty() {
            continue;
        }
        let width = legend_width(&labels, region.width(), request).min(content.width());
        let desired_height = {
            let m = outer_legend_margins(&labels, request);
            m[0] + m[2]
        } + labels
            .iter()
            .map(|b| block_dimensions(b, request).1 + request.label_gap)
            .sum::<f64>();
        let height = desired_height.min(content.height() * 0.4);
        let box_spacing = request
            .resolved_theme
            .as_ref()
            .and_then(|e| e.destination_length("legend.box.spacing", "", 0))
            .unwrap_or(0.)
            .max(0.);
        let bounds = match position {
            P::Right => {
                let b = Rect::new(
                    content.max_x() - width,
                    content.origin().y()
                        + request.resolved_theme.as_ref().map_or(0., |e| {
                            (content.height() - desired_height.min(content.height()))
                                * (1.
                                    - super::theme_elements::justification(
                                        e,
                                        if matches!(position, P::Left) {
                                            "legend.justification.left"
                                        } else {
                                            "legend.justification.right"
                                        },
                                    )[1])
                        }),
                    width,
                    if request.resolved_theme.is_some() {
                        desired_height.min(content.height())
                    } else {
                        content.height()
                    },
                )?;
                content = Rect::new(
                    content.origin().x(),
                    content.origin().y(),
                    (content.width() - width - box_spacing).max(0.),
                    content.height(),
                )?;
                b
            }
            P::Left => {
                let b = Rect::new(
                    content.origin().x(),
                    content.origin().y()
                        + request.resolved_theme.as_ref().map_or(0., |e| {
                            (content.height() - desired_height.min(content.height()))
                                * (1.
                                    - super::theme_elements::justification(
                                        e,
                                        if matches!(position, P::Left) {
                                            "legend.justification.left"
                                        } else {
                                            "legend.justification.right"
                                        },
                                    )[1])
                        }),
                    width,
                    if request.resolved_theme.is_some() {
                        desired_height.min(content.height())
                    } else {
                        content.height()
                    },
                )?;
                content = Rect::new(
                    content.origin().x() + width + box_spacing,
                    content.origin().y(),
                    (content.width() - width - box_spacing).max(0.),
                    content.height(),
                )?;
                b
            }
            P::Top => {
                let b = Rect::new(
                    content.origin().x()
                        + request.resolved_theme.as_ref().map_or(0., |e| {
                            (content.width() - width)
                                * super::theme_elements::justification(
                                    e,
                                    "legend.justification.top",
                                )[0]
                        }),
                    content.origin().y(),
                    if request.resolved_theme.is_some() {
                        width
                    } else {
                        content.width()
                    },
                    height,
                )?;
                content = Rect::new(
                    content.origin().x(),
                    content.origin().y() + height + box_spacing,
                    content.width(),
                    (content.height() - height - box_spacing).max(0.),
                )?;
                b
            }
            P::Bottom => {
                let b = Rect::new(
                    content.origin().x()
                        + request.resolved_theme.as_ref().map_or(0., |e| {
                            (content.width() - width)
                                * super::theme_elements::justification(
                                    e,
                                    "legend.justification.bottom",
                                )[0]
                        }),
                    content.max_y() - height,
                    if request.resolved_theme.is_some() {
                        width
                    } else {
                        content.width()
                    },
                    height,
                )?;
                content = Rect::new(
                    content.origin().x(),
                    content.origin().y(),
                    content.width(),
                    (content.height() - height - box_spacing).max(0.),
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
                plot.origin().x()
                    + if let Some(e) = &request.resolved_theme {
                        plot.width() * x
                            - width
                                * super::theme_elements::justification(
                                    e,
                                    "legend.justification.inside",
                                )[0]
                    } else {
                        (plot.width() - width) * x
                    },
                plot.origin().y()
                    + if let Some(e) = &request.resolved_theme {
                        plot.height() * y
                            - height
                                * (1.
                                    - super::theme_elements::justification(
                                        e,
                                        "legend.justification.inside",
                                    )[1])
                    } else {
                        (plot.height() - height) * y
                    },
                width,
                height,
            )?
        } else {
            b.bounds
        };
        let start = items.len();
        if let Some(elements) = &request.resolved_theme {
            let mut remaining = request.limits.max_path_commands;
            for name in ["legend.box.background", "legend.background"] {
                if name == "legend.background"
                    && matches!(b.labels.as_slice(), [LegendBlock::GuideBox { .. }])
                {
                    continue;
                }
                for primitive in super::theme_elements::rectangle(
                    elements,
                    name,
                    bounds,
                    request,
                    &mut remaining,
                )? {
                    require_within(items.len() < request.limits.max_items, "legend background")?;
                    items.push(SceneItem {
                        guide: None,
                        layer: None,
                        clip: Some(bounds),
                        primitive,
                    });
                }
            }
        }
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
    if prepared.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
        && (spec.scales.free_x || spec.scales.free_y)
        && matches!(&prepared.definition().coordinate,Some(crate::grammar::CoordinateSpec::Cartesian(coordinate)) if coordinate.ratio.is_some())
    {
        return Err(crate::scales::error(
            crate::DiagnosticCode::UnsupportedCapability,
            "Reference facets cannot combine free scales with a fixed Cartesian coordinate ratio.",
        ));
    }
    if (spec.scales.free_x || spec.scales.free_y)
        && matches!(
            &prepared.definition().coordinate,
            Some(crate::grammar::CoordinateSpec::Geographic(_))
        )
    {
        return Err(crate::scales::error(
            crate::DiagnosticCode::UnsupportedCapability,
            "Geographic facets require fixed positional scales.",
        ));
    }
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
    let mut strip_insets = super::facet_policy::insets(&strips, request.label_gap);
    for side in [
        AxisSide::Left,
        AxisSide::Right,
        AxisSide::Top,
        AxisSide::Bottom,
    ] {
        let i = super::facet_policy::side_index(side);
        if strip_insets[i] > 0. && super::facet_policy::outside(request, side) {
            strip_insets[i] += super::facet_policy::switch_padding(request, &spec.layout);
        }
    }

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
    let gap_x = request
        .resolved_theme
        .as_ref()
        .and_then(|e| e.destination_length("panel.spacing.x", "", 0))
        .unwrap_or(spec.gap);
    let gap_y = request
        .resolved_theme
        .as_ref()
        .and_then(|e| e.destination_length("panel.spacing.y", "", 0))
        .unwrap_or(spec.gap);
    let cell_width = (content.width() - gap_x * (columns - 1) as f64) / columns as f64;
    let cell_height = (content.height() - gap_y * (rows - 1) as f64) / rows as f64;
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
    let explicit_widths = request
        .resolved_theme
        .as_ref()
        .map(|e| {
            e.panel_sizes(
                "panel.widths",
                columns,
                content.width() - gap_x * (columns - 1) as f64,
            )
        })
        .transpose()?
        .flatten();
    let explicit_heights = request
        .resolved_theme
        .as_ref()
        .map(|e| {
            e.panel_sizes(
                "panel.heights",
                rows,
                content.height() - gap_y * (rows - 1) as f64,
            )
        })
        .transpose()?
        .flatten();

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
        .is_some_and(|p| p.space != crate::grammar::FacetSpace::Fixed)
        || explicit_widths.is_some()
        || explicit_heights.is_some();
    for pass in 0..if weighted { 2 } else { 1 } {
        cells.clear();
        local_boxes.clear();
        let mut inputs = vec![];
        for panel in prepared.panels() {
            let cell = Rect::new(
                content.origin().x()
                    + widths[..panel.column].iter().sum::<f64>()
                    + panel.column as f64 * gap_x,
                content.origin().y()
                    + heights[..panel.row].iter().sum::<f64>()
                    + panel.row as f64 * gap_y,
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
                    if !super::facet_policy::outside(request, strip.side) {
                        offsets[side] = strip_insets[side];
                    }
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
            if let Some(elements) = &request.resolved_theme {
                if explicit_widths.is_some() {
                    widths = elements
                        .panel_sizes(
                            "panel.widths",
                            columns,
                            (content.width()
                                - gap_x * (columns - 1) as f64
                                - margin[0] * columns as f64)
                                .max(0.),
                        )?
                        .unwrap()
                        .into_iter()
                        .map(|v| v + margin[0])
                        .collect();
                }
                if explicit_heights.is_some() {
                    heights = elements
                        .panel_sizes(
                            "panel.heights",
                            rows,
                            (content.height()
                                - gap_y * (rows - 1) as f64
                                - margin[1] * rows as f64)
                                .max(0.),
                        )?
                        .unwrap()
                        .into_iter()
                        .map(|v| v + margin[1])
                        .collect();
                }
            }
            if let Some(policy) = &spec.reference {
                if policy.space.free(true) && explicit_widths.is_none() {
                    let usable = (content.width()
                        - gap_x * (columns - 1) as f64
                        - margin[0] * columns as f64)
                        .max(0.);
                    let total = xweights.iter().sum::<f64>();
                    widths = xweights
                        .iter()
                        .map(|w| margin[0] + usable * w / total)
                        .collect();
                }
                if policy.space.free(false) && explicit_heights.is_none() {
                    let usable =
                        (content.height() - gap_y * (rows - 1) as f64 - margin[1] * rows as f64)
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
                let outside = super::facet_policy::outside(request, side);
                let pad = if outside {
                    super::facet_policy::switch_padding(request, &spec.layout)
                } else {
                    0.
                };
                let size = (strip_insets[i] - pad).max(0.);
                let plot = chart.plot().unwrap_or(cell);
                let bounds = if outside {
                    match side {
                        AxisSide::Top => Rect::new(
                            plot.origin().x(),
                            if grid_reference {
                                cell.origin().y() - size - pad
                            } else {
                                cell.origin().y()
                            },
                            plot.width(),
                            size,
                        )?,
                        AxisSide::Bottom => Rect::new(
                            plot.origin().x(),
                            if grid_reference {
                                cell.max_y() + pad
                            } else {
                                cell.max_y() - size
                            },
                            plot.width(),
                            size,
                        )?,
                        AxisSide::Left => Rect::new(
                            if grid_reference {
                                cell.origin().x() - size - pad
                            } else {
                                cell.origin().x()
                            },
                            plot.origin().y(),
                            size,
                            plot.height(),
                        )?,
                        AxisSide::Right => Rect::new(
                            if grid_reference {
                                cell.max_x() + pad
                            } else {
                                cell.max_x() - size
                            },
                            plot.origin().y(),
                            size,
                            plot.height(),
                        )?,
                    }
                } else {
                    match side {
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
                    }
                };
                if let Some(elements) = request.resolved_theme.as_deref() {
                    let name = if side.horizontal() {
                        "strip.background.x"
                    } else {
                        "strip.background.y"
                    };
                    let mut remaining = request.limits.max_path_commands;
                    for primitive in super::theme_elements::rectangle(
                        elements,
                        name,
                        bounds,
                        request,
                        &mut remaining,
                    )? {
                        require_within(
                            items.len() < request.limits.max_items,
                            "facet strip background",
                        )?;
                        items.push(SceneItem {
                            guide: None,
                            layer: None,
                            clip: Some(bounds),
                            primitive,
                        });
                    }
                } else {
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
                }
                let mut text = strip.block.items_at(
                    bounds.origin().x() + (bounds.width() - strip.block.bounds.width()) / 2.,
                    bounds.origin().y() + (bounds.height() - strip.block.bounds.height()) / 2.,
                    bounds,
                )?;
                if request.resolved_theme.as_ref().is_some_and(|e|matches!(e.value("strip.clip",""),Some(crate::theme::ThemeValue::Text(v)) if v=="off")) {for item in &mut text {item.clip=None;}}
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
