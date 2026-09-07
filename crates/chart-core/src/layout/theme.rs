use super::*;
use crate::Rect;
use crate::grammar::PreparedChart;
use crate::scene::{PathCommand, Primitive, Scene, SceneItem, Stroke};
use crate::theme::{Symbol, ThemePatch};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point};
use std::sync::Arc;

pub(super) fn tokens(chart: &PreparedChart, r: &LayoutRequest) -> ChartResult<ThemePatch> {
    let mut t = if let Some(theme) = &chart.definition().theme {
        if theme
            .layers
            .keys()
            .any(|id| !chart.definition().layers.iter().any(|l| &l.id == id))
        {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "Theme names an absent layer.",
                "Use identities from this chart definition.",
            ));
        }
        theme.resolve(&r.host_theme)?
    } else {
        r.host_theme.validate()?;
        r.host_theme.clone()
    };
    for (id, p) in &r.interaction_theme {
        if !chart.definition().layers.iter().any(|l| &l.id == id) {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "Interaction style names an absent layer.",
                "Use current layer identities.",
            ));
        }
        p.validate()?;
    }
    r.output_theme.validate()?;
    t.overlay(&r.output_theme);
    Ok(t)
}
pub(super) fn configure(t: &ThemePatch, r: &mut LayoutRequest) {
    r.host_theme = t.clone();
    if let Some(v) = t.font_size {
        r.font_size = v;
    }
    if let Some(v) = t.padding {
        r.padding = v;
    }
    if let Some(v) = t.gap {
        r.label_gap = v;
    }
    if let Some(v) = t.tick_length {
        r.tick_length = v;
    }
}
pub(super) fn apply(
    chart: &mut LaidOutChart,
    r: &LayoutRequest,
    t: &ThemePatch,
) -> ChartResult<()> {
    // Update panel views as well as the flattened presented scene, keeping both coherent.
    for p in &mut chart.panels {
        apply(Arc::make_mut(&mut p.chart), r, t)?;
    }
    let mut items = vec![];
    let mut targets = vec![];
    let mut panels = vec![];
    let mut decoration = |mut primitive: Primitive, clip: Option<Rect>| {
        monochrome(&mut primitive, t.color_mode);
        items.push(SceneItem {
            layer: None,
            clip,
            primitive,
        });
        targets.push(vec![]);
        panels.push(None);
    };
    if let Some(fill) = t.background {
        decoration(
            Primitive::Rectangle {
                bounds: chart.scene.bounds(),
                fill,
            },
            None,
        );
    }
    let plots = if chart.panels.is_empty() {
        vec![(chart.plot, &chart.axes)]
    } else {
        chart
            .panels
            .iter()
            .map(|p| (p.chart.plot, &p.chart.axes))
            .collect()
    };
    for (plot, axes) in plots {
        if let Some(plot) = plot {
            if let Some(gradient) = t.gradient {
                decoration(
                    Primitive::GradientRectangle {
                        bounds: plot,
                        gradient,
                    },
                    Some(plot),
                );
            } else if let Some(fill) = t.panel {
                decoration(Primitive::Rectangle { bounds: plot, fill }, Some(plot));
            }
            if let Some(color) = t.grid.filter(|c| c.alpha > 0) {
                for a in axes.values().filter(|a| a.spec.visible) {
                    for tick in &a.ticks {
                        let (from, to) = if a.spec.side.horizontal() {
                            (
                                Point::new(tick.position, plot.origin().y())?,
                                Point::new(tick.position, plot.max_y())?,
                            )
                        } else {
                            (
                                Point::new(plot.origin().x(), tick.position)?,
                                Point::new(plot.max_x(), tick.position)?,
                            )
                        };
                        decoration(
                            Primitive::Rule {
                                from,
                                to,
                                stroke: Stroke {
                                    color,
                                    width: t.stroke_width.unwrap_or(1.),
                                },
                            },
                            Some(plot),
                        );
                    }
                }
            }
        }
    }
    let interaction_offset = items.len();
    for (index, source) in chart.scene.items().iter().enumerate() {
        let mut item = source.clone();
        let mut local = t.clone();
        if let Some(id) = item.layer {
            if let Some(p) = chart
                .prepared
                .definition()
                .theme
                .as_ref()
                .and_then(|s| s.layers.get(&id))
            {
                local.overlay(p);
            }
            if let Some(p) = r.interaction_theme.get(&id) {
                local.overlay(p);
            }
            local.overlay(&r.output_theme);
        }
        let authored = item.layer.and_then(|id| {
            chart
                .prepared
                .definition()
                .layers
                .iter()
                .find(|l| l.id == id)
        });
        let mapped = authored.is_some_and(|l| l.color.is_some());
        let mapped_size = authored.is_some_and(|l| match &l.mappings {
            crate::grammar::Mappings::Source(a) => {
                a.size.is_some()
                    || (l.inherit && chart.prepared.definition().mappings.size.is_some())
            }
            crate::grammar::Mappings::Binned(a) => a.size.is_some(),
            crate::grammar::Mappings::Statistical(a) => a.size.is_some(),
        });
        let explicit_width = item.layer.and_then(|id| {
            let authored = chart
                .prepared
                .definition()
                .theme
                .as_ref()
                .and_then(|s| s.layers.get(&id))
                .and_then(|p| p.stroke_width);
            r.output_theme
                .stroke_width
                .or_else(|| r.interaction_theme.get(&id).and_then(|p| p.stroke_width))
                .or(authored)
        });
        let color = if item.layer.is_some() {
            if mapped { None } else { local.mark }
        } else {
            None
        };
        match &mut item.primitive {
            Primitive::GradientRectangle { .. } => {}
            Primitive::Rectangle { fill, .. }
            | Primitive::NativePaint { fill, .. }
            | Primitive::Point { fill, .. }
            | Primitive::FilledPath { fill, .. }
            | Primitive::Symbol { fill, .. } => {
                if item.layer.is_some()
                    && let Some(c) = color
                {
                    *fill = c;
                }
            }
            Primitive::Rule { stroke, .. }
            | Primitive::Path { stroke, .. }
            | Primitive::DashedPath { stroke, .. } => {
                if let Some(c) = color {
                    stroke.color = c;
                }
                if let Some(w) = local.stroke_width {
                    if item.layer.is_none() {
                        stroke.width = w;
                    } else if !mapped_size && let Some(w) = explicit_width {
                        stroke.width = w;
                    }
                }
            }
            Primitive::Text { color: c, .. } | Primitive::GlyphRun { color: c, .. } => {
                if let Some(color) = color {
                    *c = color;
                }
            }
        }
        if let Primitive::Point {
            center,
            radius,
            fill,
        } = &item.primitive
            && local.symbol.is_some_and(|s| s != Symbol::Circle)
        {
            item.primitive = Primitive::Symbol {
                center: *center,
                radius: *radius,
                kind: local.symbol.unwrap_or_default(),
                fill: *fill,
            };
        }
        if let Some(dashes) = local.dashes.as_ref().filter(|d| !d.is_empty()) {
            let path = match &item.primitive {
                Primitive::Rule { from, to, stroke } => Some((
                    vec![PathCommand::MoveTo(*from), PathCommand::LineTo(*to)],
                    *stroke,
                )),
                Primitive::Path { commands, stroke } => Some((commands.clone(), *stroke)),
                _ => None,
            };
            if let Some((commands, stroke)) = path {
                item.primitive = Primitive::DashedPath {
                    commands,
                    stroke,
                    dashes: dashes.clone(),
                };
            }
        }
        monochrome(&mut item.primitive, local.color_mode);
        items.push(item);
        targets.push(chart.targets[index].clone());
        panels.push(chart.item_panels[index].clone());
    }
    chart.scene = Scene::new(
        chart.scene.stamp(),
        chart.scene.units(),
        chart.scene.bounds(),
        &items,
        chart.scene.resources(),
        r.limits,
    )?;
    chart.interactions = std::mem::take(&mut chart.interactions)
        .into_iter()
        .map(|(i, v)| (i + interaction_offset, v))
        .collect();
    chart.targets = targets;
    chart.item_panels = panels;
    Ok(())
}

pub(super) fn monochrome(p: &mut Primitive, mode: Option<crate::theme::ColorMode>) {
    let apply = |c: &mut crate::scene::Color| *c = crate::theme::paint_color(*c, mode);
    match p {
        Primitive::Rectangle { fill, .. }
        | Primitive::NativePaint { fill, .. }
        | Primitive::Point { fill, .. }
        | Primitive::FilledPath { fill, .. }
        | Primitive::Symbol { fill, .. } => apply(fill),
        Primitive::Rule { stroke, .. }
        | Primitive::Path { stroke, .. }
        | Primitive::DashedPath { stroke, .. } => apply(&mut stroke.color),
        Primitive::Text { color, .. } | Primitive::GlyphRun { color, .. } => apply(color),
        Primitive::GradientRectangle { gradient, .. } => {
            apply(&mut gradient.start);
            apply(&mut gradient.end);
        }
    }
}
