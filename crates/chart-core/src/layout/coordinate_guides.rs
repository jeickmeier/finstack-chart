//! Coordinate guide geometry over the existing selected ticks and measured labels.
use super::{coordinate_map::CoordinateMap, *};
use crate::grammar::{CoordinateSpec, RadialAxisPlacement, RadialMode, ThetaAxis};
use crate::{ChartResult, Point, Rect, ScaleId};
use std::collections::BTreeMap;

pub(super) fn resolve(
    chart: &crate::grammar::PreparedChart,
    axes: &BTreeMap<ScaleId, ResolvedAxis>,
    plot: Rect,
) -> ChartResult<Option<CoordinateMap>> {
    let Some(spec) = &chart.definition().coordinate else {
        return Ok(None);
    };
    let primary = |horizontal| {
        axes.values().find(|a| {
            a.spec.side.horizontal() == horizontal
                && !matches!(
                    a.scale,
                    ResolvedScale::Secondary { .. }
                        | ResolvedScale::SecondaryDiscrete { .. }
                        | ResolvedScale::SecondaryTime { .. }
                )
        })
    };
    let (Some(x), Some(y)) = (primary(true), primary(false)) else {
        return Ok(None);
    };
    super::coordinate_resolve::resolve_with_windows(
        spec,
        [x, y],
        plot,
        Some(&chart.state().axis_windows()),
    )
    .and_then(|map| map.with_chart_resources(chart))
    .map(Some)
}
/// Fit an aspect in the measured available panel, keeping its center and margins.
pub(super) fn fit_aspect(plot: Rect, aspect: Option<f64>) -> ChartResult<Rect> {
    let Some(aspect) = aspect else {
        return Ok(plot);
    };
    if !aspect.is_finite() || aspect <= 0. {
        return Err(crate::scales::error(
            crate::DiagnosticCode::NumericalDomain,
            "Coordinate aspect must be finite and positive.",
        ));
    }
    let width = plot.width().min(plot.height() / aspect);
    let height = width * aspect;
    Rect::new(
        plot.origin().x() + (plot.width() - width) / 2.,
        plot.origin().y() + (plot.height() - height) / 2.,
        width,
        height,
    )
}
fn rescale(value: f64, from: [f64; 2], to: [f64; 2]) -> f64 {
    if from[0] == from[1] {
        to[0].midpoint(to[1])
    } else {
        to[0] + (value - from[0]) / (from[1] - from[0]) * (to[1] - to[0])
    }
}
fn old_point(map: &CoordinateMap, theta: usize, angular: f64, radial: f64) -> ChartResult<Point> {
    let mut values = [0.; 2];
    values[theta] = angular;
    values[1 - theta] = radial;
    Point::new(
        rescale(values[0], map.axes[0].viewport, map.axes[0].range),
        rescale(values[1], map.axes[1].viewport, map.axes[1].range),
    )
}
fn path(
    map: &CoordinateMap,
    from: Point,
    to: Point,
    r: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Vec<crate::scene::PathCommand>> {
    let mut path = crate::path::Path::new();
    path.move_to(from.x(), from.y())?;
    path.line_to(to.x(), to.y())?;
    super::coordinate_path::project(
        &path.geometry(),
        map,
        super::coordinate_path::tolerance(r),
        remaining,
    )?
    .lower(
        super::coordinate_path::tolerance(r),
        r.limits.max_path_commands,
    )
}
/// Radial guide endpoint and outward tick direction in destination units.
fn placement(
    map: &CoordinateMap,
    theta: usize,
    angular: f64,
    radial: f64,
    secondary: bool,
) -> ChartResult<(Point, [f64; 2], f64)> {
    let angle = angular_angle(map, theta, angular);
    let mut values = [0.; 2];
    values[theta] = angular;
    values[1 - theta] = radial;
    let point = map.project_values(values)?.ok_or_else(|| {
        crate::scales::error(
            crate::DiagnosticCode::NumericalDomain,
            "Coordinate guide position is undefined.",
        )
    })?;
    let angle = angle + if secondary { std::f64::consts::PI } else { 0. };
    Ok((point, [libm::sin(angle), -libm::cos(angle)], angle))
}

fn angular_angle(map: &CoordinateMap, theta: usize, value: f64) -> f64 {
    let direction = match &map.spec {
        CoordinateSpec::Radial(spec) if spec.mode == RadialMode::Polar => f64::from(spec.direction),
        _ => 1.,
    };
    rescale(value, map.view[theta], map.arc).rem_euclid(std::f64::consts::TAU) * direction
}

fn outside_placement(
    map: &CoordinateMap,
    radius: f64,
    secondary: bool,
    polar: bool,
) -> ChartResult<(Point, [f64; 2], f64)> {
    let arc_start = map.arc[0].min(map.arc[1]);
    let span = (map.arc[1] - map.arc[0]).abs();
    let angle = if polar {
        0.
    } else {
        [
            0.,
            std::f64::consts::PI,
            std::f64::consts::FRAC_PI_2,
            3. * std::f64::consts::FRAC_PI_2,
        ]
        .into_iter()
        .find(|angle| (angle - arc_start).rem_euclid(std::f64::consts::TAU) <= span + 1e-12)
        .unwrap_or(arc_start)
    };
    let vertical = libm::cos(angle).abs() > 0.5;
    let sign = if secondary { 1. } else { -1. };
    if vertical {
        let y = map.plot.max_y()
            - rescale(0.5 + radius * libm::cos(angle), map.bbox[1], [0., 1.]) * map.plot.height();
        Ok((
            Point::new(
                if secondary {
                    map.plot.max_x()
                } else {
                    map.plot.origin().x()
                },
                y,
            )?,
            [sign, 0.],
            sign * std::f64::consts::FRAC_PI_2,
        ))
    } else {
        let x = map.plot.origin().x()
            + rescale(0.5 + radius * libm::sin(angle), map.bbox[0], [0., 1.]) * map.plot.width();
        Ok((
            Point::new(
                x,
                if secondary {
                    map.plot.origin().y()
                } else {
                    map.plot.max_y()
                },
            )?,
            [0., -sign],
            if secondary { 0. } else { std::f64::consts::PI },
        ))
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn radial(
    map: &CoordinateMap,
    scales: &BTreeMap<ScaleId, ResolvedAxis>,
    guides: &mut BTreeMap<crate::GuideId, ResolvedGuide>,
    labels: &BTreeMap<crate::GuideId, Vec<super::engine::Label>>,
    r: &LayoutRequest,
    measurer: &dyn crate::services::TextMeasurer,
    out: &mut super::project::Output,
) -> ChartResult<bool> {
    use crate::scene::{GuideRole, Primitive, SceneItem};
    let CoordinateSpec::Radial(spec) = &map.spec else {
        return Ok(false);
    };
    let theta = usize::from(spec.theta == ThetaAxis::Y);
    let radius = 1 - theta;
    let color = r
        .host_theme
        .foreground
        .map(crate::color::Paint::resolve)
        .unwrap_or(crate::scene::Color {
            red: 55,
            green: 60,
            blue: 65,
            alpha: 255,
        });
    let mut remaining = r.limits.max_path_commands;
    for (id, guide) in guides {
        if !guide.spec.visible {
            continue;
        }
        let angular = usize::from(!guide.spec.side.horizontal()) == theta;
        let secondary = matches!(
            scales[&guide.spec.scale].scale,
            ResolvedScale::Secondary { .. }
                | ResolvedScale::SecondaryDiscrete { .. }
                | ResolvedScale::SecondaryTime { .. }
        ) || (guide.spec.id != scales[&guide.spec.scale].spec.default_guide().id
            && guide.spec.side != scales[&guide.spec.scale].spec.side);
        if spec.mode == RadialMode::Polar && angular && secondary {
            guide.ticks.clear();
            guide.tick_indices.clear();
            continue;
        }
        let components = guide.spec.components.clone().unwrap_or_default();
        let geom = super::guide_geometry::Geometry::resolve(&guide.spec.style, r);
        let axis = if angular { theta } else { radius };
        let mut selected = Vec::new();
        for label in &labels[id] {
            let value = super::coordinate_resolve::limit(
                &scales[&guide.spec.scale],
                map.axes[axis],
                &label.tick.value,
            )?;
            if in_view(value, map.guide_view(axis)) {
                selected.push(label.clone());
            }
        }
        let mut angular_values = if angular {
            selected
                .iter()
                .map(|label| {
                    super::coordinate_resolve::limit(
                        &scales[&guide.spec.scale],
                        map.axes[theta],
                        &label.tick.value,
                    )
                })
                .collect::<ChartResult<Vec<_>>>()?
        } else {
            Vec::new()
        };
        if angular && selected.len() > 1 {
            let delta = (angular_angle(map, theta, *angular_values.last().unwrap())
                - angular_angle(map, theta, angular_values[0]))
            .rem_euclid(std::f64::consts::TAU);
            if delta < 0.05 {
                let merged = format!(
                    "{}/{}",
                    selected[0].tick.label,
                    selected.last().unwrap().tick.label
                );
                selected.last_mut().unwrap().tick.label = merged;
                selected.remove(0);
                angular_values.remove(0);
            }
        }
        guide.ticks.clear();
        guide.tick_indices.clear();
        let outside = spec.mode == RadialMode::Polar
            || match &spec.radial_axis {
                Some(RadialAxisPlacement::Outside) => true,
                Some(_) => false,
                None => (map.arc[1] - map.arc[0]).abs() >= std::f64::consts::TAU - 1e-10,
            };
        let r_index = if secondary {
            usize::from(map.radii[0] > map.radii[1])
        } else {
            usize::from(map.radii[0] < map.radii[1])
        };
        let radial_value = map.view[radius][r_index];
        let theta_value = match &spec.radial_axis {
            Some(RadialAxisPlacement::At(values)) => {
                let value = &values[usize::from(secondary).min(values.len() - 1)];
                let primary = scales
                    .values()
                    .find(|a| {
                        usize::from(!a.spec.side.horizontal()) == theta
                            && !matches!(
                                a.scale,
                                ResolvedScale::Secondary { .. }
                                    | ResolvedScale::SecondaryDiscrete { .. }
                                    | ResolvedScale::SecondaryTime { .. }
                            )
                    })
                    .unwrap();
                super::coordinate_resolve::limit(primary, map.axes[theta], value)?.clamp(
                    map.view[theta][0].min(map.view[theta][1]),
                    map.view[theta][0].max(map.view[theta][1]),
                )
            }
            _ => map.view[theta][usize::from(secondary)],
        };
        if components.domain.visible != Some(false) && !(spec.mode == RadialMode::Polar && angular)
        {
            let primitive = if !angular && outside {
                Primitive::Rule {
                    from: outside_placement(
                        map,
                        map.radii[0],
                        secondary,
                        spec.mode == RadialMode::Polar,
                    )?
                    .0,
                    to: outside_placement(
                        map,
                        map.radii[1],
                        secondary,
                        spec.mode == RadialMode::Polar,
                    )?
                    .0,
                    stroke: components.domain.stroke(color),
                }
            } else {
                let (from, to) = if angular {
                    (
                        old_point(map, theta, map.view[theta][0], radial_value)?,
                        old_point(map, theta, map.view[theta][1], radial_value)?,
                    )
                } else {
                    (
                        old_point(map, theta, theta_value, map.view[radius][0])?,
                        old_point(map, theta, theta_value, map.view[radius][1])?,
                    )
                };
                Primitive::Path {
                    commands: path(map, from, to, r, &mut remaining)?,
                    stroke: components.domain.stroke(color),
                }
            };
            out.push(
                SceneItem {
                    guide: super::engine::component(&guide.spec, GuideRole::Domain, None),
                    layer: None,
                    clip: None,
                    primitive,
                },
                vec![],
                r,
            )?;
        }
        let anchor_for = |value: f64| -> ChartResult<(Point, [f64; 2], f64)> {
            if angular {
                placement(map, theta, value, radial_value, secondary)
            } else if outside {
                outside_placement(
                    map,
                    rescale(value, map.view[radius], map.radii),
                    secondary,
                    spec.mode == RadialMode::Polar,
                )
            } else {
                let (point, _, angle) = placement(map, theta, theta_value, value, false)?;
                let sign = if secondary { 1. } else { -1. };
                Ok((
                    point,
                    [sign * libm::cos(angle), sign * libm::sin(angle)],
                    angle,
                ))
            }
        };
        for (index, label) in selected.iter().enumerate() {
            guide.ticks.push(label.tick.clone());
            guide.tick_indices.push(label.index);
            if !label.visible {
                continue;
            }
            let value = if angular {
                angular_values[index]
            } else {
                super::coordinate_resolve::limit(
                    &scales[&guide.spec.scale],
                    map.axes[radius],
                    &label.tick.value,
                )?
            };
            let (anchor, normal, angle) = anchor_for(value)?;
            if label.line_style.visible != Some(false)
                && !(spec.mode == RadialMode::Polar && angular)
            {
                out.push(
                    SceneItem {
                        guide: super::engine::component(&guide.spec, GuideRole::Line, Some(label)),
                        layer: None,
                        clip: None,
                        primitive: super::engine::styled_line(
                            Primitive::Rule {
                                from: anchor,
                                to: Point::new(
                                    anchor.x() + normal[0] * geom.inner,
                                    anchor.y() + normal[1] * geom.inner,
                                )?,
                                stroke: label.line_style.stroke(color),
                            },
                            &label.line_style,
                        ),
                    },
                    vec![],
                    r,
                )?;
            }
            if label.text_style.visible == Some(false) || label.tick.label.is_empty() {
                continue;
            }
            let mut request = r.clone();
            request.font_size = label.font_size;
            let rotation = if angular {
                label
                    .text_style
                    .rotation
                    .map(|rotation| upright(rotation + angle.to_degrees()))
                    .unwrap_or(guide.spec.label_rotation)
            } else {
                label
                    .text_style
                    .rotation
                    .unwrap_or(guide.spec.label_rotation)
            };
            let mut run = label
                .text_style
                .typography
                .as_ref()
                .or(guide.spec.typography.as_ref())
                .cloned()
                .unwrap_or_else(|| crate::typography::RichRun::new(""));
            run.replace_text(label.tick.label.clone(), r.limits)?;
            request.font_size = label.font_size / run.size;
            if let Some(color) = label.text_style.color {
                run.color = Some(color);
            }
            let block = if rotation != 0.
                || label.text_style.typography.is_some()
                || guide.spec.typography.is_some()
            {
                super::text::measure(
                    &crate::typography::RichText {
                        lines: vec![vec![run]],
                        line_spacing: 1.2,
                        rotation,
                    },
                    &request,
                    measurer,
                    color,
                )?
            } else {
                super::text::plain_lines(
                    &label.tick.label,
                    &request,
                    measurer,
                    label
                        .text_style
                        .color
                        .map(crate::color::Paint::resolve)
                        .unwrap_or(color),
                )?
            };
            let gap = if angular && spec.mode == RadialMode::Polar {
                0.05 * map.plot.width()
            } else {
                geom.inner.max(0.) + r.label_gap + label.dodge_offset
            };
            let center = angular && spec.mode == RadialMode::Polar;
            let (x, y) = if angular && !center && rotation != 0. {
                let mut metrics_request = request.clone();
                metrics_request.font_size = label.font_size;
                let metrics = crate::services::measure_text(
                    measurer,
                    super::engine::text_request(&metrics_request, &label.tick.label),
                    r.limits,
                )?;
                let radians = rotation.to_radians();
                let local = angle - radians;
                let dx = metrics.width() * libm::sin(local) / 2.;
                let dy = -metrics.height() * libm::cos(local) / 2.;
                (
                    anchor.x() + gap * normal[0] + libm::cos(radians) * dx
                        - libm::sin(radians) * dy
                        - block.bounds.width() / 2.,
                    anchor.y()
                        + gap * normal[1]
                        + libm::sin(radians) * dx
                        + libm::cos(radians) * dy
                        - block.bounds.height() / 2.,
                )
            } else {
                let horizontal = if center { 0.5 } else { 0.5 - normal[0] / 2. };
                let vertical = if center { 0.5 } else { 0.5 - normal[1] / 2. };
                (
                    anchor.x() + gap * normal[0] - horizontal * block.bounds.width(),
                    anchor.y() + gap * normal[1] - vertical * block.bounds.height(),
                )
            };
            for mut item in block.items_at(x, y, r.bounds)? {
                item.guide = super::engine::component(&guide.spec, GuideRole::Label, Some(label));
                out.push(item, vec![], r)?;
            }
            out.diagnostics.extend(block.diagnostics);
        }
        if guide
            .spec
            .ggplot_axis
            .as_ref()
            .is_some_and(|options| options.minor_ticks)
            && components.ticks.visible != Some(false)
            && !(angular && spec.mode == RadialMode::Polar)
        {
            let mut occurrences = BTreeMap::<String, usize>::new();
            for (index, tick) in guide.minor_ticks.iter().enumerate() {
                let axis = if angular { theta } else { radius };
                let value = if let Some(value) = &tick.value {
                    super::coordinate_resolve::limit(
                        &scales[&guide.spec.scale],
                        map.axes[axis],
                        value,
                    )?
                } else {
                    rescale(tick.position, map.axes[axis].range, map.axes[axis].viewport)
                };
                if !in_view(value, map.guide_view(axis)) {
                    continue;
                }
                let (anchor, normal, _) = anchor_for(value)?;
                let identity = tick.value.as_ref().map(|value| {
                    let count = occurrences
                        .entry(serde_json::to_string(value).expect("typed guide value"))
                        .or_default();
                    let occurrence = *count;
                    *count += 1;
                    crate::scene::GuideTickIdentity {
                        value: value.clone(),
                        occurrence,
                    }
                });
                out.push(
                    SceneItem {
                        guide: Some(crate::scene::GuideComponent {
                            animation: None,
                            scope: vec!["minor".into()],
                            guide: guide.spec.id,
                            role: GuideRole::Line,
                            side: guide.spec.side,
                            tick: identity,
                            index: Some(index),
                            label: Some(String::new()),
                        }),
                        layer: None,
                        clip: None,
                        primitive: super::engine::styled_line(
                            Primitive::Rule {
                                from: anchor,
                                to: Point::new(
                                    anchor.x() + normal[0] * geom.inner / 2.,
                                    anchor.y() + normal[1] * geom.inner / 2.,
                                )?,
                                stroke: components.ticks.stroke(color),
                            },
                            &components.ticks,
                        ),
                    },
                    vec![],
                    r,
                )?;
            }
        }
    }
    Ok(true)
}
fn upright(angle: f64) -> f64 {
    let angle = angle.rem_euclid(360.);
    if angle > 90. && angle < 270. {
        angle - 180.
    } else {
        angle
    }
}

pub(super) fn grid(
    map: &CoordinateMap,
    horizontal: bool,
    position: f64,
    request: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Vec<crate::scene::PathCommand>> {
    if matches!(map.spec, CoordinateSpec::Geographic(_)) {
        return super::geographic_guides::path(map, horizontal, position, request, remaining);
    }
    let axis = usize::from(!horizontal);
    let value = rescale(position, map.axes[axis].range, map.axes[axis].viewport);
    let CoordinateSpec::Radial(spec) = &map.spec else {
        let mut a = map.view.map(|v| v[0]);
        let mut b = map.view.map(|v| v[1]);
        a[axis] = value;
        b[axis] = value;
        let old = |v: [f64; 2]| {
            Point::new(
                rescale(v[0], map.axes[0].viewport, map.axes[0].range),
                rescale(v[1], map.axes[1].viewport, map.axes[1].range),
            )
        };
        return path(map, old(a)?, old(b)?, request, remaining);
    };
    let theta = usize::from(spec.theta == ThetaAxis::Y);
    let radial = 1 - theta;
    let (a, b) = if axis == theta {
        let end = if spec.mode == RadialMode::Polar {
            rescale(0.45, map.radii, map.view[radial])
        } else {
            map.view[radial][1]
        };
        (
            old_point(map, theta, value, map.view[radial][0])?,
            old_point(map, theta, value, end)?,
        )
    } else {
        (
            old_point(map, theta, map.view[theta][0], value)?,
            old_point(map, theta, map.view[theta][1], value)?,
        )
    };
    path(map, a, b, request, remaining)
}
pub(super) fn polar_outer_grid(
    map: &CoordinateMap,
    request: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Option<Vec<crate::scene::PathCommand>>> {
    let CoordinateSpec::Radial(spec) = &map.spec else {
        return Ok(None);
    };
    if spec.mode != RadialMode::Polar {
        return Ok(None);
    };
    let theta = usize::from(spec.theta == ThetaAxis::Y);
    let radial = rescale(0.45, map.radii, map.view[1 - theta]);
    path(
        map,
        old_point(map, theta, map.view[theta][0], radial)?,
        old_point(map, theta, map.view[theta][1], radial)?,
        request,
        remaining,
    )
    .map(Some)
}

fn flipped(map: &CoordinateMap) -> bool {
    match &map.spec {
        CoordinateSpec::Cartesian(v) => v.flip,
        CoordinateSpec::Transformed(v) => v.view.flip,
        _ => false,
    }
}
pub(super) fn side(map: &CoordinateMap, side: AxisSide) -> AxisSide {
    if !flipped(map) {
        return side;
    }
    match side {
        AxisSide::Bottom => AxisSide::Left,
        AxisSide::Top => AxisSide::Right,
        AxisSide::Left => AxisSide::Bottom,
        AxisSide::Right => AxisSide::Top,
    }
}
pub(super) fn tick_position(
    map: &CoordinateMap,
    source_side: AxisSide,
    position: f64,
) -> ChartResult<Option<f64>> {
    let axis = usize::from(!source_side.horizontal());
    let other = 1 - axis;
    let edge = usize::from(matches!(source_side, AxisSide::Top | AxisSide::Right));
    let perpendicular = rescale(
        map.view[other][edge],
        map.axes[other].viewport,
        map.axes[other].range,
    );
    let anchor = if axis == 0 {
        Point::new(position, perpendicular)?
    } else {
        Point::new(perpendicular, position)?
    };
    Ok(map.project(anchor)?.map(|point| {
        if side(map, source_side).horizontal() {
            point.x()
        } else {
            point.y()
        }
    }))
}
/// Reposition existing selected guide values without mutating mark scales or identities.
pub(super) fn relocate(
    map: &CoordinateMap,
    guides: &mut BTreeMap<crate::GuideId, ResolvedGuide>,
    request: &LayoutRequest,
) -> ChartResult<()> {
    if matches!(map.spec, CoordinateSpec::Geographic(_)) {
        return super::geographic_guides::relocate(map, guides, request);
    }

    if matches!(map.spec, CoordinateSpec::Radial(_)) {
        return Ok(());
    }
    for guide in guides.values_mut() {
        let source = guide.spec.side;
        guide.spec.side = side(map, source);
        let mut ticks = Vec::with_capacity(guide.ticks.len());
        for mut tick in std::mem::take(&mut guide.ticks) {
            if let Some(position) = tick_position(map, source, tick.position)?
                && in_view(
                    position,
                    if guide.spec.side.horizontal() {
                        [map.plot.origin().x(), map.plot.max_x()]
                    } else {
                        [map.plot.origin().y(), map.plot.max_y()]
                    },
                )
            {
                tick.position = position;
                ticks.push(tick);
            }
        }
        guide.ticks = ticks;
        let mut minor = Vec::with_capacity(guide.minor_ticks.len());
        for mut tick in std::mem::take(&mut guide.minor_ticks) {
            if let Some(position) = tick_position(map, source, tick.position)?
                && in_view(
                    position,
                    if guide.spec.side.horizontal() {
                        [map.plot.origin().x(), map.plot.max_x()]
                    } else {
                        [map.plot.origin().y(), map.plot.max_y()]
                    },
                )
            {
                tick.position = position;
                minor.push(tick);
            }
        }
        guide.minor_ticks = minor;
        if flipped(map) {
            guide.spec.translation.swap(0, 1);
        }
    }
    Ok(())
}

fn in_view(value: f64, view: [f64; 2]) -> bool {
    let tolerance = 16. * f64::EPSILON * view[0].abs().max(view[1].abs()).max(1.);
    value >= view[0].min(view[1]) - tolerance && value <= view[0].max(view[1]) + tolerance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{RadialCoordinate, RadialReverse};
    #[test]
    fn source_angular_guide_anchors_outer_inner_reversal_and_partial_arcs() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/coordinate-guide-controls.json"
        ))
        .unwrap();
        let mut checked = 0;
        for case in fixture["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let mut spec = RadialCoordinate {
                inner_radius: 0.3,
                expand: false,
                ..Default::default()
            };
            if name == "partial" {
                spec.start = std::f64::consts::FRAC_PI_4;
                spec.end = Some(1.5 * std::f64::consts::PI);
            }
            if name == "reverse" {
                spec.reverse = RadialReverse::Both;
            }
            if name == "theta-y" {
                spec.theta = ThetaAxis::Y;
            }
            if matches!(name, "inside" | "at" | "outside-partial") {
                spec.inner_radius = 0.;
            }
            if name == "outside-partial" {
                spec.start = std::f64::consts::FRAC_PI_4;
                spec.end = Some(3. * std::f64::consts::FRAC_PI_4);
            }
            let axes = [
                super::super::coordinate_map::CoordinateDomain {
                    domain: [0., 4.],
                    viewport: [0., 4.],
                    range: [0., 600.],
                },
                super::super::coordinate_map::CoordinateDomain {
                    domain: [0., 4.],
                    viewport: [0., 4.],
                    range: [600., 0.],
                },
            ];
            let map = CoordinateMap::new(
                CoordinateSpec::Radial(spec.clone()),
                axes,
                [[0., 4.], [0., 4.]],
                Rect::new(0., 0., 600., 600.).unwrap(),
            )
            .unwrap();
            let theta = usize::from(spec.theta == ThetaAxis::Y);
            let radial = 1 - theta;
            for (guide, secondary) in [("theta", false), ("theta.sec", true)] {
                let Some(key) = case["guides"][guide]["key"].as_array() else {
                    continue;
                };
                for row in key {
                    let r_index = if secondary {
                        usize::from(map.radii[0] > map.radii[1])
                    } else {
                        usize::from(map.radii[0] < map.radii[1])
                    };
                    let (p, _, _) = placement(
                        &map,
                        theta,
                        row[".value"].as_f64().unwrap(),
                        map.view[radial][r_index],
                        secondary,
                    )
                    .unwrap();
                    assert!(
                        (p.x() / 600. - row["x"].as_f64().unwrap()).abs() < 1e-12,
                        "{name}/{guide}/x"
                    );
                    assert!(
                        (1. - p.y() / 600. - row["y"].as_f64().unwrap()).abs() < 1e-12,
                        "{name}/{guide}/y"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked >= 35);
    }
    #[test]
    fn source_explicit_angular_text_rotation_uses_upright_convention() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/coordinate-guide-controls.json"
        ))
        .unwrap();
        let case = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|v| v["name"] == "angle")
            .unwrap();
        let guide = &case["guides"]["theta"];
        for (key, expected) in guide["key"]
            .as_array()
            .unwrap()
            .iter()
            .zip(guide["label_angle_ccw"].as_array().unwrap())
        {
            let actual = upright(
                key["theta"].as_f64().unwrap().to_degrees() - guide["angle"].as_f64().unwrap(),
            );
            let expected = (-expected.as_f64().unwrap()).rem_euclid(360.);
            assert!((actual.rem_euclid(360.) - expected).abs() < 1e-12);
        }
    }
    #[test]
    fn measured_aspect_preserves_center_and_available_bounds() {
        let plot = Rect::new(20., 30., 500., 200.).unwrap();
        let fitted = fit_aspect(plot, Some(2.)).unwrap();
        assert_eq!(fitted, Rect::new(220., 30., 100., 200.).unwrap());
        let source_partial = 0.8535533905932737;
        let fitted = fit_aspect(plot, Some(source_partial)).unwrap();
        assert!((fitted.height() / fitted.width() - source_partial).abs() < 1e-14);
        assert!(fitted.origin().x() >= plot.origin().x() && fitted.max_x() <= plot.max_x());
    }
}
