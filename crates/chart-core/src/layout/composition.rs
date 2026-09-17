use super::{
    engine::{pressure, solve_panels},
    text::{self, Block},
    *,
};
use crate::composition::{Anchor, Collision};
use crate::grammar::{PanelKey, PreparedChart};
use crate::scene::{Color, Primitive, Scene, SceneItem, Stroke};
use crate::services::TextMeasurer;
use crate::theme::ThemePatch;
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect};
use std::sync::Arc;

pub(super) struct Furniture {
    pub content: Rect,
    inner: Rect,
    tag: Option<Tag>,
    top: Vec<(Block, f64)>,
    bottom: Vec<(Block, f64)>,
}
struct Tag {
    block: Block,
    position: [f64; 2],
    numeric: bool,
    location: String,
    hjust: f64,
    vjust: f64,
}
fn tag_options(
    elements: Option<&crate::theme::ResolvedElements>,
    block: Block,
) -> ChartResult<Tag> {
    use crate::theme::ThemeValue as V;
    let position = elements.and_then(|e| e.value("plot.tag.position", ""));
    let (position, numeric) = match position {
        Some(V::Vector(v)) if v.len() == 2 => {
            let (V::Number(x), V::Number(y)) = (&v[0], &v[1]) else {
                return Err(invalid("Tag numeric position requires two finite numbers."));
            };
            if !x.is_finite() || !y.is_finite() {
                return Err(invalid("Tag position must be finite."));
            }
            ([*x, 1. - *y], true)
        }
        Some(V::Text(name)) => (
            match name.as_str() {
                "topleft" => [0., 0.],
                "top" => [0.5, 0.],
                "topright" => [1., 0.],
                "left" => [0., 0.5],
                "right" => [1., 0.5],
                "bottomleft" => [0., 1.],
                "bottom" => [0.5, 1.],
                "bottomright" => [1., 1.],
                _ => return Err(invalid("Unknown tag position.")),
            },
            false,
        ),
        None | Some(V::Missing) => ([0., 0.], false),
        _ => {
            return Err(invalid(
                "Tag position requires a named edge or two numeric fractions.",
            ));
        }
    };
    let location = match elements.and_then(|e| e.value("plot.tag.location", "")) {
        Some(V::Text(v)) => v.clone(),
        None | Some(V::Missing) => {
            if numeric {
                "plot".into()
            } else {
                "margin".into()
            }
        }
        _ => return Err(invalid("Tag location must be plot, panel or margin.")),
    };
    if !matches!(location.as_str(), "plot" | "panel" | "margin") || numeric && location == "margin"
    {
        return Err(invalid(
            "Numeric tags require plot/panel location; named tags additionally support margin.",
        ));
    }
    Ok(Tag {
        block,
        position,
        numeric,
        location,
        hjust: elements
            .and_then(|e| e.number("plot.tag", "hjust"))
            .unwrap_or(0.5),
        vjust: elements
            .and_then(|e| e.number("plot.tag", "vjust"))
            .unwrap_or(0.5),
    })
}
fn panel_bounds(chart: &LaidOutChart) -> Option<Rect> {
    if chart.panels.is_empty() {
        return chart.plot;
    }
    let bounds = chart
        .panels
        .iter()
        .filter_map(|p| p.chart.plot)
        .collect::<Vec<_>>();
    let first = *bounds.first()?;
    let (mut x0, mut y0, mut x1, mut y1) = (
        first.origin().x(),
        first.origin().y(),
        first.max_x(),
        first.max_y(),
    );
    for b in bounds {
        x0 = x0.min(b.origin().x());
        y0 = y0.min(b.origin().y());
        x1 = x1.max(b.max_x());
        y1 = y1.max(b.max_y());
    }
    Rect::new(x0, y0, x1 - x0, y1 - y0).ok()
}
fn ink(r: &LayoutRequest) -> Color {
    r.host_theme
        .annotation
        .map(crate::color::Paint::resolve)
        .unwrap_or(crate::theme::rgb(55, 60, 65))
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use a current panel/scale/layer identity and a valid explicit furniture position.",
    )
}
pub(super) fn prepare(
    chart: &PreparedChart,
    r: &LayoutRequest,
    m: &dyn TextMeasurer,
) -> ChartResult<Option<Furniture>> {
    let Some(f) = chart.state().figure(chart.definition()) else {
        return Ok(None);
    };
    let elements = r.resolved_theme.as_deref();
    let measured =
        |text: &crate::typography::RichText, node: &str| -> ChartResult<Option<(Block, f64)>> {
            let styled = if let Some(elements) = &elements {
                super::theme_elements::text_style(elements, node, text, r)?
            } else {
                Some(text.clone())
            };
            styled
                .as_ref()
                .map(|text| {
                    let mut block = text::measure(text, r, m, ink(r))?;
                    if let Some(elements) = &elements {
                        super::theme_elements::margins(elements, node, &mut block, r)?;
                    }
                    Ok((
                        block,
                        elements
                            .as_ref()
                            .and_then(|e| e.number(node, "hjust"))
                            .unwrap_or(0.),
                    ))
                })
                .transpose()
        };
    let tag = f
        .tag
        .as_ref()
        .map(|text| measured(text, "plot.tag"))
        .transpose()?
        .flatten()
        .map(|(block, _)| tag_options(elements, block))
        .transpose()?;
    let mut reserve = [0.; 4];
    if let Some(tag) = &tag
        && tag.location == "margin"
    {
        if tag.position[1] == 0. {
            reserve[0] = tag.block.bounds.height();
        }
        if tag.position[0] == 1. {
            reserve[1] = tag.block.bounds.width();
        }
        if tag.position[1] == 1. {
            reserve[2] = tag.block.bounds.height();
        }
        if tag.position[0] == 0. {
            reserve[3] = tag.block.bounds.width();
        }
    }
    let inner = Rect::new(
        r.bounds.origin().x() + reserve[3],
        r.bounds.origin().y() + reserve[0],
        (r.bounds.width() - reserve[1] - reserve[3]).max(0.),
        (r.bounds.height() - reserve[0] - reserve[2]).max(0.),
    )?;
    let mut top = Vec::new();
    if let Some(text) = &f.title
        && let Some(block) = measured(text, "plot.title")?
    {
        top.push(block);
    }
    if let Some(text) = &f.subtitle
        && let Some(block) = measured(text, "plot.subtitle")?
    {
        top.push(block);
    }
    let mut bottom = Vec::new();
    for text in f.caption.iter().chain(&f.source_notes).chain(&f.footnotes) {
        if let Some(block) = measured(text, "plot.caption")? {
            bottom.push(block);
        }
    }
    let top_height = top
        .iter()
        .map(|(b, _)| b.bounds.height() + r.label_gap)
        .sum::<f64>();
    let bottom_height = bottom
        .iter()
        .map(|(b, _)| b.bounds.height() + r.label_gap)
        .sum::<f64>();
    let start = (top_height + r.padding).min(inner.height());
    let height = (inner.height() - start - bottom_height - r.padding).max(0.);
    let content = Rect::new(
        inner.origin().x(),
        inner.origin().y() + start,
        inner.width(),
        height,
    )?;
    Ok(Some(Furniture {
        content,
        inner,
        tag,
        top,
        bottom,
    }))
}
fn parent<'a>(chart: &'a LaidOutChart, key: &Option<PanelKey>) -> ChartResult<&'a LaidOutChart> {
    match key {
        Some(k) => chart
            .panels
            .iter()
            .find(|p| &p.key == k)
            .map(|p| p.chart.as_ref())
            .ok_or_else(|| invalid("Furniture names an absent laid-out facet.")),
        None if chart.panels.is_empty() => Ok(chart),
        None => Err(invalid("Faceted data/panel furniture must name its panel.")),
    }
}

pub(super) fn anchor(
    chart: &LaidOutChart,
    a: &Anchor,
    figure: Rect,
) -> ChartResult<Option<(Point, Rect)>> {
    a.validate()?;
    let relative = |b: Rect, x: f64, y: f64| -> ChartResult<Option<(Point, Rect)>> {
        Ok(Some((
            Point::new(
                b.origin().x() + b.width() * x,
                b.origin().y() + b.height() * y,
            )?,
            b,
        )))
    };
    match a {
        Anchor::Figure { x, y } => relative(figure, *x, *y),
        Anchor::Output { x, y } => Ok(Some((
            Point::new(figure.origin().x() + x, figure.origin().y() + y)?,
            figure,
        ))),
        Anchor::Panel { panel, x, y } => {
            let Some(plot) = parent(chart, panel)?.plot else {
                return Ok(None);
            };
            relative(plot, *x, *y)
        }
        Anchor::Data {
            panel,
            scales,
            x,
            y,
        } => {
            let p = parent(chart, panel)?;
            let Some(plot) = p.plot else {
                return Ok(None);
            };
            let x_axis = p
                .axes
                .get(&scales.x)
                .filter(|a| a.spec.side.horizontal())
                .ok_or_else(|| invalid("Annotation horizontal scale is absent or vertical."))?;
            let y_axis = p
                .axes
                .get(&scales.y)
                .filter(|a| !a.spec.side.horizontal())
                .ok_or_else(|| invalid("Annotation vertical scale is absent or horizontal."))?;
            let (Some(x), Some(y)) = (x_axis.map_value(x)?, y_axis.map_value(y)?) else {
                return Ok(None);
            };
            let point = Point::new(x, y)?;
            let point = if let Some(spec) = &p.prepared.definition().coordinate {
                super::coordinate_resolve::resolve_with_windows(
                    spec,
                    [x_axis, y_axis],
                    plot,
                    Some(&p.prepared.state().axis_windows()),
                )?
                .with_chart_resources(&p.prepared)?
                .project(point)?
            } else {
                Some(point)
            };
            Ok(point.map(|point| (point, plot)))
        }
    }
}
fn overlap(a: Rect, b: Rect) -> bool {
    a.origin().x() < b.max_x()
        && b.origin().x() < a.max_x()
        && a.origin().y() < b.max_y()
        && b.origin().y() < a.max_y()
}
fn inside(a: Rect, b: Rect) -> bool {
    a.origin().x() >= b.origin().x()
        && a.origin().y() >= b.origin().y()
        && a.max_x() <= b.max_x()
        && a.max_y() <= b.max_y()
}
fn append(
    chart: &mut LaidOutChart,
    items: &mut Vec<SceneItem>,
    block: &Block,
    x: f64,
    y: f64,
    clip: Rect,
) -> ChartResult<Rect> {
    let bounds = Rect::new(x, y, block.bounds.width(), block.bounds.height())?;
    if !inside(bounds, clip) {
        chart.diagnostics.push(pressure(
            "Furniture exceeds its declared clip and is clipped; logical text is retained.",
        ));
    }
    let added = block.items_at(x, y, clip)?;
    chart
        .targets
        .extend(std::iter::repeat_n(vec![], added.len()));
    chart
        .item_panels
        .extend(std::iter::repeat_n(None, added.len()));
    items.extend(added);
    chart.diagnostics.extend_from_slice(&block.diagnostics);
    Ok(bounds)
}
pub(super) fn finish(
    chart: &mut LaidOutChart,
    r: &LayoutRequest,
    m: &dyn TextMeasurer,
    theme: &ThemePatch,
    furniture: Furniture,
) -> ChartResult<()> {
    let f = chart
        .prepared
        .state()
        .figure(chart.prepared.definition())
        .ok_or_else(|| invalid("Missing captured figure furniture."))?;
    let mut items = chart.scene.items().to_vec();
    let elements = r.resolved_theme.as_deref();
    let span = |name: &str| -> ChartResult<Rect> {
        match elements.as_ref().and_then(|e| e.value(name, "")) {
            Some(crate::theme::ThemeValue::Text(v)) if v == "panel" => {
                Ok(panel_bounds(chart).unwrap_or(furniture.inner))
            }
            None | Some(crate::theme::ThemeValue::Missing) => Ok(furniture.inner),
            Some(crate::theme::ThemeValue::Text(v)) if v == "plot" => Ok(furniture.inner),
            _ => Err(invalid("Figure text position must be panel or plot.")),
        }
    };
    let top_span = span("plot.title.position")?;
    let bottom_span = span("plot.caption.position")?;
    let text_padding = if elements.is_some() { 0. } else { r.padding };
    let mut occupied = vec![];
    let mut y = furniture.inner.origin().y() + r.padding;
    for (b, align) in &furniture.top {
        occupied.push(append(
            chart,
            &mut items,
            b,
            top_span.origin().x()
                + text_padding
                + align * (top_span.width() - 2. * text_padding - b.bounds.width()),
            y,
            r.bounds,
        )?);
        y += b.bounds.height() + r.label_gap;
    }
    let h = furniture
        .bottom
        .iter()
        .map(|(b, _)| b.bounds.height() + r.label_gap)
        .sum::<f64>();
    y = furniture.inner.max_y() - r.padding - h;
    for (b, align) in &furniture.bottom {
        occupied.push(append(
            chart,
            &mut items,
            b,
            bottom_span.origin().x()
                + text_padding
                + align * (bottom_span.width() - 2. * text_padding - b.bounds.width()),
            y,
            r.bounds,
        )?);
        y += b.bounds.height() + r.label_gap;
    }
    if let Some(tag) = &furniture.tag {
        let region = if tag.location == "panel" {
            panel_bounds(chart).unwrap_or(furniture.inner)
        } else if tag.location == "plot" {
            furniture.inner
        } else {
            let x = if tag.position[0] == 0. {
                r.bounds.origin().x()
            } else if tag.position[0] == 1. {
                furniture.inner.max_x()
            } else {
                furniture.inner.origin().x()
            };
            let y = if tag.position[1] == 0. {
                r.bounds.origin().y()
            } else if tag.position[1] == 1. {
                furniture.inner.max_y()
            } else {
                furniture.inner.origin().y()
            };
            Rect::new(
                x,
                y,
                if tag.position[0] == 0.5 {
                    furniture.inner.width()
                } else {
                    tag.block.bounds.width()
                },
                if tag.position[1] == 0.5 {
                    furniture.inner.height()
                } else {
                    tag.block.bounds.height()
                },
            )?
        };
        let (w, h) = (tag.block.bounds.width(), tag.block.bounds.height());
        let (x, y) = if tag.numeric {
            (
                region.origin().x() + tag.position[0] * region.width() - tag.hjust * w,
                region.origin().y() + tag.position[1] * region.height() - (1. - tag.vjust) * h,
            )
        } else {
            let x = if tag.position[0] == 0. {
                (1. - 2. * tag.hjust) * w
            } else if tag.position[0] == 1. {
                region.width() - w
            } else {
                tag.hjust * (region.width() - w)
            };
            let y = if tag.position[1] == 0. {
                0.
            } else if tag.position[1] == 1. {
                region.height() - 2. * (1. - tag.vjust) * h
            } else {
                (1. - tag.vjust) * (region.height() - h)
            };
            (region.origin().x() + x, region.origin().y() + y)
        };
        occupied.push(append(chart, &mut items, &tag.block, x, y, r.bounds)?);
    }
    for letter in &f.panel_letters {
        if let Some(plot) = parent(chart, &letter.panel)?.plot {
            let block = text::measure(&letter.text, r, m, ink(r))?;
            occupied.push(append(
                chart,
                &mut items,
                &block,
                plot.origin().x() + r.label_gap,
                plot.origin().y() + r.label_gap,
                plot,
            )?);
        } else {
            chart
                .diagnostics
                .push(pressure("Panel letter has no useful plot and was omitted."));
        }
    }
    for inset in &f.insets {
        let p = parent(chart, &inset.panel)?;
        let Some(plot) = p.plot else {
            chart
                .diagnostics
                .push(pressure("Inset has no useful parent plot and was omitted."));
            continue;
        };
        let bounds = Rect::new(
            plot.origin().x() + inset.rectangle[0] * plot.width(),
            plot.origin().y() + inset.rectangle[1] * plot.height(),
            inset.rectangle[2] * plot.width(),
            inset.rectangle[3] * plot.height(),
        )?;
        let mut prepared = p.prepared.as_ref().clone();
        prepared.layers.retain(|l| inset.layers.contains(&l.id()));
        prepared.panels.clear();
        let mut request = r.clone();
        if let Some(panel) = &inset.panel {
            request
                .hierarchy_scope
                .push(super::GuideScope::Panel(panel.clone()));
        }
        request
            .hierarchy_scope
            .push(super::GuideScope::Inset(inset.id.clone()));
        request.bounds = bounds;
        request.figure_bounds = Some(bounds);
        request.minimum_plot = (8., 8.);
        request.font_size *= 0.7;
        request.target_ticks = 3;
        request.padding = 2.;
        request.label_gap = 2.;
        request.tick_length = 2.;
        request.axes = p
            .axes
            .values()
            .map(|a| {
                let mut spec = a.spec.clone();
                spec.viewport = if spec.side.horizontal() {
                    inset.x_view
                } else {
                    inset.y_view
                };
                spec.visible = inset.guides && spec.visible;
                spec.title = None;
                spec.typography = None;
                spec.label_rotation = 0.;
                spec.range = None;
                spec
            })
            .collect();
        let mut view = solve_panels(
            vec![(Arc::new(prepared), request.clone())],
            m,
            chart.scene.stamp(),
        )?
        .pop()
        .ok_or_else(|| invalid("Inset layout returned no view."))?;
        let mut inset_theme = theme.clone();
        inset_theme.background = Some(theme.panel.unwrap_or(crate::theme::rgb(255, 255, 255)));
        super::theme::apply(&mut view, &request, &inset_theme)?;
        chart.interactions.extend(
            view.interactions
                .iter()
                .map(|(i, v)| (i + items.len(), v.clone())),
        );
        let scope = format!("inset:{}", inset.id);
        items.extend(view.scene.items().iter().cloned().map(|mut item| {
            if let Some(guide) = &mut item.guide {
                guide.scope.insert(0, scope.clone());
            }
            item
        }));
        chart.targets.extend_from_slice(&view.targets);
        chart.item_panels.extend(std::iter::repeat_n(
            inset.panel.clone(),
            view.scene.items().len(),
        ));
        chart.diagnostics.extend_from_slice(&view.diagnostics);
        chart.insets.push(LaidOutInset {
            id: inset.id.clone(),
            panel: inset.panel.clone(),
            bounds,
            chart: Arc::new(view),
        });
        occupied.push(bounds);
    }
    for a in &f.paths {
        let Some((position, clip)) = anchor(chart, &a.anchor, r.bounds)? else {
            chart.diagnostics.push(pressure(
                "Path annotation was omitted by its anchor scale/no-space policy.",
            ));
            continue;
        };
        let geometry = a.geometry.transformed(
            crate::path::Affine::new([1., 0., 0., 1., position.x(), position.y()])?,
            0.01,
            r.limits.max_path_commands,
        )?;
        items.push(SceneItem {
            guide: None,
            layer: None,
            clip: Some(if a.overflow { r.bounds } else { clip }),
            primitive: Primitive::VectorPath {
                dashes: if a.stroke.is_some() {
                    theme.dashes.clone().unwrap_or_default()
                } else {
                    vec![]
                },
                geometry,
                fill: a.fill.map(crate::color::Paint::resolve),
                stroke: a.stroke.map(|s| s.map_color(crate::color::Paint::resolve)),
            },
        });
        chart.targets.push(vec![]);
        chart.item_panels.push(None);
    }
    let mut annotations: Vec<_> = f.annotations.iter().enumerate().collect();
    annotations.sort_by_key(|(index, a)| (std::cmp::Reverse(a.priority), *index));
    for (_, a) in annotations {
        let Some((position, clip)) = anchor(chart, &a.anchor, r.bounds)? else {
            chart.diagnostics.push(pressure(
                "Annotation was omitted by its scale/no-space policy; logical text is retained.",
            ));
            continue;
        };
        let clip = if a.overflow { r.bounds } else { clip };
        let block = text::measure(&a.text, r, m, ink(r))?;
        let mut candidate = Rect::new(
            position.x() + a.offset[0],
            position.y() + a.offset[1],
            block.bounds.width(),
            block.bounds.height(),
        )?;
        if a.collision != Collision::Keep {
            let valid =
                |b: Rect| inside(b, clip) && !occupied.iter().any(|other| overlap(b, *other));
            if !valid(candidate) {
                let step = block.bounds.height() + r.label_gap;
                let shifted = if a.collision == Collision::ShiftThenHide {
                    [
                        (0., -1.),
                        (1., 0.),
                        (0., 1.),
                        (-1., 0.),
                        (0., -2.),
                        (2., 0.),
                        (0., 2.),
                        (-2., 0.),
                    ]
                    .into_iter()
                    .map(|(x, y)| {
                        Rect::new(
                            candidate.origin().x() + x * step,
                            candidate.origin().y() + y * step,
                            candidate.width(),
                            candidate.height(),
                        )
                    })
                    .collect::<ChartResult<Vec<_>>>()?
                    .into_iter()
                    .find(|b| valid(*b))
                } else {
                    None
                };
                if let Some(next) = shifted {
                    candidate = next;
                    chart.diagnostics.push(pressure("Annotation collision moved a lower-priority label to the first valid bounded candidate."));
                } else {
                    chart.diagnostics.push(pressure("Annotation collision exhausted its bounded placement policy; label hidden with logical text retained."));
                    continue;
                }
            }
        }
        if let Some(to) = &a.callout {
            if let Some((to, _)) = anchor(chart, to, r.bounds)? {
                // The nearest point on the label box minimizes leader length deterministically.
                let from = if a.connector_origin == crate::composition::ConnectorOrigin::Anchor {
                    position
                } else {
                    Point::new(
                        to.x().clamp(candidate.origin().x(), candidate.max_x()),
                        to.y().clamp(candidate.origin().y(), candidate.max_y()),
                    )?
                };
                items.push(SceneItem {
                    guide: None,
                    layer: None,
                    clip: Some(clip),
                    primitive: Primitive::Rule {
                        from,
                        to,
                        stroke: Stroke {
                            color: ink(r),
                            width: theme.stroke_width.unwrap_or(1.),
                        },
                    },
                });
                chart.targets.push(vec![]);
                chart.item_panels.push(None);
            } else {
                chart.diagnostics.push(pressure(
                    "Callout endpoint was omitted by its scale/no-space policy.",
                ));
            }
        }
        occupied.push(append(
            chart,
            &mut items,
            &block,
            candidate.origin().x(),
            candidate.origin().y(),
            clip,
        )?);
    }
    // Figure furniture follows the selected final output color policy as well.
    for item in &mut items {
        if item.layer.is_none() {
            super::theme::monochrome(&mut item.primitive, theme.color_mode);
        }
    }
    let resources = text::resources(&items, r)?;
    chart.scene = Scene::new(
        chart.scene.stamp(),
        r.units,
        r.bounds,
        &items,
        &resources,
        r.limits,
    )?;
    Ok(())
}
