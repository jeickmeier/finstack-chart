use super::{
    engine::{pressure, solve_panels},
    text::{self, Block},
    *,
};
use crate::composition::{Anchor, Collision};
use crate::grammar::{PanelKey, PreparedChart, PreparedGeometry};
use crate::scene::{Color, Primitive, Scene, SceneItem, Stroke};
use crate::services::TextMeasurer;
use crate::theme::ThemePatch;
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect};
use std::sync::Arc;

pub(super) struct Furniture {
    pub content: Rect,
    top: Vec<Block>,
    bottom: Vec<Block>,
}
fn ink(r: &LayoutRequest) -> Color {
    r.host_theme
        .annotation
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
    let Some(f) = &chart.definition().figure else {
        return Ok(None);
    };
    f.validate(r.limits)?;
    // Charge inset projection work before cloning layer geometry or calling font services.
    let vertices = |p: &PreparedChart, ids: Option<&[crate::LayerId]>| {
        p.layers()
            .iter()
            .filter(|l| ids.is_none_or(|ids| ids.contains(&l.id())))
            .flat_map(|l| l.marks())
            .map(|m| match &m.geometry {
                PreparedGeometry::Point(_) => 1,
                PreparedGeometry::LineRun(p) => p.len(),
                PreparedGeometry::BandRun { lower, upper } => lower.len() + upper.len(),
                _ => 2,
            })
            .sum::<usize>()
    };
    let mut cost = vertices(chart, None);
    for inset in &f.insets {
        let p = match &inset.panel {
            Some(key) => chart
                .panels()
                .iter()
                .find(|p| &p.key == key)
                .map(|p| p.chart.as_ref())
                .ok_or_else(|| invalid("Inset names an absent prepared panel."))?,
            None if chart.panels().is_empty() => chart,
            None => return Err(invalid("A faceted inset must select a parent panel.")),
        };
        if inset
            .layers
            .iter()
            .any(|id| !p.layers().iter().any(|l| l.id() == *id))
        {
            return Err(invalid("Inset names a layer absent from its parent panel."));
        }
        cost = cost
            .checked_add(vertices(p, Some(&inset.layers)))
            .ok_or_else(|| invalid("Inset geometry budget overflows."))?;
    }
    crate::limits::require_within(cost <= r.max_vertices, "figure plus inset projected vertex")?;
    let top = f
        .title
        .iter()
        .chain(&f.subtitle)
        .map(|t| text::measure(t, r, m, ink(r)))
        .collect::<ChartResult<Vec<_>>>()?;
    let bottom = f
        .caption
        .iter()
        .chain(&f.source_notes)
        .chain(&f.footnotes)
        .map(|t| text::measure(t, r, m, ink(r)))
        .collect::<ChartResult<Vec<_>>>()?;
    let top_height = top
        .iter()
        .map(|b| b.bounds.height() + r.label_gap)
        .sum::<f64>();
    let bottom_height = bottom
        .iter()
        .map(|b| b.bounds.height() + r.label_gap)
        .sum::<f64>();
    let start = (top_height + r.padding).min(r.bounds.height());
    let height = (r.bounds.height() - start - bottom_height - r.padding).max(0.);
    let content = Rect::new(
        r.bounds.origin().x(),
        r.bounds.origin().y() + start,
        r.bounds.width(),
        height,
    )?;
    Ok(Some(Furniture {
        content,
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

fn anchor(chart: &LaidOutChart, a: &Anchor, figure: Rect) -> ChartResult<Option<(Point, Rect)>> {
    let relative = |b: Rect, x: f64, y: f64| -> ChartResult<Option<(Point, Rect)>> {
        if !x.is_finite() || !y.is_finite() || !(0. ..=1.).contains(&x) || !(0. ..=1.).contains(&y)
        {
            return Err(invalid(
                "Relative furniture coordinates must be finite fractions in [0,1].",
            ));
        }
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
            Ok(Some((Point::new(x, y)?, plot)))
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
        .definition()
        .figure
        .clone()
        .ok_or_else(|| invalid("Missing captured figure furniture."))?;
    let mut items = chart.scene.items().to_vec();
    let mut occupied = vec![];
    let mut y = r.bounds.origin().y() + r.padding;
    for b in &furniture.top {
        occupied.push(append(
            chart,
            &mut items,
            b,
            r.bounds.origin().x() + r.padding,
            y,
            r.bounds,
        )?);
        y += b.bounds.height() + r.label_gap;
    }
    let h = furniture
        .bottom
        .iter()
        .map(|b| b.bounds.height() + r.label_gap)
        .sum::<f64>();
    y = r.bounds.max_y() - r.padding - h;
    for b in &furniture.bottom {
        occupied.push(append(
            chart,
            &mut items,
            b,
            r.bounds.origin().x() + r.padding,
            y,
            r.bounds,
        )?);
        y += b.bounds.height() + r.label_gap;
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
        items.extend_from_slice(view.scene.items());
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
                let from = Point::new(
                    to.x().clamp(candidate.origin().x(), candidate.max_x()),
                    to.y().clamp(candidate.origin().y(), candidate.max_y()),
                )?;
                items.push(SceneItem {
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
