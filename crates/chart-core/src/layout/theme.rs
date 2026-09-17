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
        r.host_theme.resolve()
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
    t.overlay(&r.output_theme.resolve());
    Ok(t)
}
pub(super) fn configure(t: &ThemePatch, r: &mut LayoutRequest) {
    r.host_theme = t.clone().map_colors(Into::into);
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
    let elements = r.resolved_theme.as_deref();
    chart.paint_themes.clear();
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
            guide: None,
            layer: None,
            clip,
            primitive,
        });
        targets.push(vec![]);
        panels.push(None);
    };
    let mut coordinate_remaining = r.limits.max_path_commands;
    if let Some(elements) = &elements {
        for primitive in super::theme_elements::rectangle(
            elements,
            "plot.background",
            chart.scene.bounds(),
            r,
            &mut coordinate_remaining,
        )? {
            decoration(primitive, None);
        }
    } else if let Some(fill) = t.background {
        decoration(
            Primitive::Rectangle {
                bounds: chart.scene.bounds(),
                fill,
            },
            None,
        );
    }
    let plots = if chart.panels.is_empty() {
        vec![(chart.plot, &chart.axes, &chart.guides, &chart.prepared)]
    } else {
        chart
            .panels
            .iter()
            .map(|p| {
                (
                    p.chart.plot,
                    &p.chart.axes,
                    &p.chart.guides,
                    &p.chart.prepared,
                )
            })
            .collect()
    };
    for (plot, axes, guides, prepared) in plots {
        if let Some(plot) = plot {
            let coordinate = super::coordinate_guides::resolve(prepared, axes, plot)?;
            if let Some(gradient) = t.gradient {
                let primitive = Primitive::GradientRectangle {
                    bounds: plot,
                    gradient,
                };
                if let Some(map)=coordinate.as_ref().filter(|map|matches!(&map.spec,crate::grammar::CoordinateSpec::Radial(spec) if spec.mode==crate::grammar::RadialMode::Radial)) {
                    for primitive in super::coordinate_raster::panel_gradient(&primitive,map,r,&mut coordinate_remaining)?{decoration(primitive,Some(plot));}
                }else{decoration(primitive,Some(plot));}
            } else if let Some(fill) = t.panel {
                let primitive = Primitive::Rectangle { bounds: plot, fill };
                if let Some(map)=coordinate.as_ref().filter(|map|matches!(&map.spec,crate::grammar::CoordinateSpec::Radial(spec) if spec.mode==crate::grammar::RadialMode::Radial)) {
                    for primitive in super::coordinate_clip::panel(primitive,map,super::coordinate_path::tolerance(r),&mut coordinate_remaining)?{decoration(primitive,Some(plot));}
                }else{decoration(primitive,Some(plot));}
            }
            if t.grid.is_some_and(|c| c.alpha > 0) || elements.is_some() {
                for a in axes.values().filter(|a| a.spec.visible) {
                    for minor in [true, false] {
                        if minor && elements.is_none() {
                            continue;
                        }
                        let name = format!(
                            "panel.grid.{}.{}",
                            if minor { "minor" } else { "major" },
                            if a.spec.side.horizontal() { "x" } else { "y" }
                        );
                        if elements.as_ref().is_some_and(|e| e.blank(&name)) {
                            continue;
                        }
                        let color = if let Some(e) = &elements {
                            e.paint(&name, "colour")?.map(crate::color::Paint::resolve)
                        } else {
                            t.grid
                        };
                        let Some(color) = color.filter(|c| c.alpha > 0) else {
                            continue;
                        };
                        let width = elements
                            .as_ref()
                            .and_then(|e| e.number(&name, "linewidth"))
                            .map(|v| crate::grammar::reference_linewidth(v, r.units))
                            .unwrap_or(t.stroke_width.unwrap_or(1.));
                        if width <= 0. {
                            continue;
                        }
                        let positions: Vec<f64> = if let Some(map) =
                            coordinate.as_ref().filter(|m| {
                                matches!(m.spec, crate::grammar::CoordinateSpec::Geographic(_))
                            }) {
                            if minor {
                                vec![]
                            } else {
                                super::geographic_guides::levels(map, a.spec.side.horizontal(), r)?
                            }
                        } else if minor {
                            guides
                                .values()
                                .filter(|g| g.spec.scale == a.spec.id)
                                .flat_map(|g| g.minor_ticks.iter().map(|t| t.position))
                                .collect()
                        } else {
                            a.ticks.iter().map(|t| t.position).collect()
                        };
                        for position in positions {
                            let (from, to) = if a.spec.side.horizontal() {
                                (
                                    Point::new(position, plot.origin().y())?,
                                    Point::new(position, plot.max_y())?,
                                )
                            } else {
                                (
                                    Point::new(plot.origin().x(), position)?,
                                    Point::new(plot.max_x(), position)?,
                                )
                            };
                            let stroke = Stroke { color, width };
                            let primitive = if let Some(map) = &coordinate {
                                Primitive::Path {
                                    commands: super::coordinate_guides::grid(
                                        map,
                                        a.spec.side.horizontal(),
                                        position,
                                        r,
                                        &mut coordinate_remaining,
                                    )?,
                                    stroke,
                                }
                            } else {
                                Primitive::Rule { from, to, stroke }
                            };
                            if let Some(elements) = &elements {
                                let commands = match primitive {
                                    Primitive::Path { commands, .. } => commands,
                                    Primitive::Rule { from, to, .. } => {
                                        vec![PathCommand::MoveTo(from), PathCommand::LineTo(to)]
                                    }
                                    _ => unreachable!(),
                                };
                                for primitive in super::theme_elements::line_primitives(
                                    elements,
                                    &name,
                                    commands,
                                    r,
                                    &mut coordinate_remaining,
                                )? {
                                    decoration(primitive, Some(plot));
                                }
                            } else {
                                decoration(primitive, Some(plot));
                            }
                        }
                    }
                }
                let color = t.grid.unwrap_or(crate::theme::rgb(0, 0, 0));
                if let Some(map) = &coordinate
                    && let Some(commands) = super::coordinate_guides::polar_outer_grid(
                        map,
                        r,
                        &mut coordinate_remaining,
                    )?
                {
                    decoration(
                        Primitive::Path {
                            commands,
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
    let mut overlaid = Vec::new();
    if elements.as_ref().is_some_and(|e| {
        matches!(
            e.value("panel.ontop", ""),
            Some(crate::theme::ThemeValue::Bool(true))
        )
    }) {
        let mut underlaid = Vec::new();
        for item in items.drain(..) {
            if item.clip.is_some() {
                overlaid.push(item);
            } else {
                underlaid.push(item);
            }
        }
        items = underlaid;
        targets.truncate(items.len());
        panels.truncate(items.len());
    }
    let interaction_offset = items.len();
    let output_theme = r.output_theme.resolve();
    for layer in &chart.prepared.definition().layers {
        let mut local = t.clone();
        if let Some(p) = chart
            .prepared
            .definition()
            .theme
            .as_ref()
            .and_then(|s| s.layers.get(&layer.id))
        {
            local.overlay(&p.resolve());
        }
        if let Some(p) = r.interaction_theme.get(&layer.id) {
            local.overlay(&p.resolve());
        }
        local.overlay(&output_theme);
        chart.paint_themes.insert(layer.id, local);
    }
    for (index, source) in chart.scene.items().iter().enumerate() {
        let mut item = source.clone();
        let local = item
            .layer
            .and_then(|id| chart.paint_themes.get(&id))
            .unwrap_or(t);
        let authored = item.layer.and_then(|id| {
            chart
                .prepared
                .definition()
                .layers
                .iter()
                .find(|l| l.id == id)
        });
        let mapped = authored.is_some_and(|l| {
            l.color.is_some()
                || !l.paint_scales.is_empty()
                || l.style.fill.is_some()
                || l.style.stroke.is_some()
                || l.style.alpha.is_some()
                || l.numeric_scales
                    .contains_key(&crate::grammar::NumericAesthetic::Alpha)
                || l.after_scale.keys().any(|a| {
                    matches!(
                        a,
                        crate::grammar::AfterScaleAesthetic::Color
                            | crate::grammar::AfterScaleAesthetic::Fill
                            | crate::grammar::AfterScaleAesthetic::Stroke
                            | crate::grammar::AfterScaleAesthetic::Alpha
                    )
                })
        });
        let mapped_size = authored.is_some_and(|l| {
            l.numeric_scales
                .contains_key(&crate::grammar::NumericAesthetic::StrokeWidth)
                || l.after_scale
                    .contains_key(&crate::grammar::AfterScaleAesthetic::LineWidth)
                || match &l.mappings {
                    crate::grammar::Mappings::Source(a) => {
                        a.size.is_some()
                            || (l.inherit && chart.prepared.definition().mappings.size.is_some())
                    }
                    crate::grammar::Mappings::Binned(a) => a.size.is_some(),
                    crate::grammar::Mappings::Statistical(a) => a.size.is_some(),
                }
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
            Primitive::VectorPath { fill, stroke, .. }
            | Primitive::ShapePath { fill, stroke, .. } => {
                if let Some(c) = color {
                    if let Some(fill) = fill {
                        *fill = c;
                    }
                    if let Some(stroke) = stroke.as_mut() {
                        stroke.color = c;
                    }
                }
                if !mapped_size
                    && let Some(w) = explicit_width
                    && let Some(stroke) = stroke
                {
                    stroke.width = w;
                }
            }
            Primitive::RasterImage { .. }
            | Primitive::GradientRectangle { .. }
            | Primitive::SampledGradientRectangle { .. } => {}
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
        let mapped_line_type = authored.is_some_and(|l| {
            l.style.line_type.is_some()
                || l.value_scales
                    .contains_key(&crate::grammar::ValueAesthetic::LineType)
                || l.aesthetic_values
                    .contains_key(&crate::grammar::ValueAesthetic::LineType)
        });
        if let Some(dashes) = local
            .dashes
            .as_ref()
            .filter(|d| !d.is_empty() && !mapped_line_type)
        {
            let path = match &item.primitive {
                Primitive::Rule { from, to, stroke } => Some((
                    vec![PathCommand::MoveTo(*from), PathCommand::LineTo(*to)],
                    *stroke,
                )),
                Primitive::Path { commands, stroke } => Some((commands.clone(), *stroke)),
                _ => None,
            };
            if let Primitive::ShapePath {
                dashes: pattern,
                stroke: Some(_),
                ..
            }
            | Primitive::VectorPath {
                dashes: pattern,
                stroke: Some(_),
                ..
            } = &mut item.primitive
            {
                *pattern = dashes.clone();
            }
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
    targets.extend(std::iter::repeat_n(Vec::new(), overlaid.len()));
    panels.extend(std::iter::repeat_n(None, overlaid.len()));
    items.extend(overlaid);
    if let Some(elements) = &elements {
        let borders = if chart.panels.is_empty() {
            chart.plot.into_iter().collect::<Vec<_>>()
        } else {
            chart.panels.iter().filter_map(|p| p.chart.plot).collect()
        };
        for plot in borders {
            for primitive in super::theme_elements::rectangle(
                elements,
                "panel.border",
                plot,
                r,
                &mut coordinate_remaining,
            )? {
                items.push(SceneItem {
                    guide: None,
                    layer: None,
                    clip: Some(plot),
                    primitive,
                });
                targets.push(Vec::new());
                panels.push(None);
            }
        }
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
        Primitive::VectorPath { fill, stroke, .. } | Primitive::ShapePath { fill, stroke, .. } => {
            if let Some(fill) = fill {
                apply(fill);
            }
            if let Some(stroke) = stroke {
                apply(&mut stroke.color);
            }
        }
        Primitive::Rectangle { fill, .. }
        | Primitive::NativePaint { fill, .. }
        | Primitive::Point { fill, .. }
        | Primitive::FilledPath { fill, .. }
        | Primitive::Symbol { fill, .. } => apply(fill),
        Primitive::Rule { stroke, .. }
        | Primitive::Path { stroke, .. }
        | Primitive::DashedPath { stroke, .. } => apply(&mut stroke.color),
        Primitive::Text { color, .. } | Primitive::GlyphRun { color, .. } => apply(color),
        Primitive::RasterImage { raster, .. } => {
            for color in &mut raster.pixels {
                apply(color);
            }
        }
        Primitive::SampledGradientRectangle { colors, .. } => {
            for color in colors {
                apply(color);
            }
        }
        Primitive::GradientRectangle { gradient, .. } => {
            apply(&mut gradient.start);
            apply(&mut gradient.end);
        }
    }
}
