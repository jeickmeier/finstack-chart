//! Compose the shared tick join with immutable layout primitives and capture metadata.
use super::{
    guide_geometry::{self, Geometry},
    *,
};
use crate::{
    ChartResult, DiagnosticCode, GuideId, Limits, Point, Rect, ScaleId,
    composition::ScaleValue,
    scene::{GuideAnimation, GuideComponent, GuideRole, PathCommand, Primitive, Scene, SceneItem},
};
use std::{collections::BTreeMap, sync::Arc};

type Key = (Vec<String>, GuideId);
/// One guide's exact displayed geometry, including continuous opacity and exiting ticks.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuidePresentationSnapshot {
    /// Stable nested facet/inset scope, consistent with semantic guide snapshots.
    pub scope: Vec<GuideScope>,
    /// Independent guide identity in that scope.
    pub guide: GuideId,
    /// Coherent sampled geometry, logical labels and lifecycle identities.
    pub frame: GuideTransitionFrame,
}
#[derive(Clone, Debug)]
pub(super) struct Presentation {
    frames: BTreeMap<Key, GuidePresentationSnapshot>,
    pub snapshots: Vec<GuideSnapshot>,
}
impl LaidOutChart {
    /// Displayed guide geometry from this immutable scene, including interrupted states.
    /// Data scales remain those of the current target; only guide decoration interpolates.
    pub fn guide_presentation(&self) -> Vec<GuidePresentationSnapshot> {
        capture(self).into_values().collect()
    }
}
pub(super) fn initial_frames(
    guides: &BTreeMap<GuideId, ResolvedGuide>,
    axes: &BTreeMap<ScaleId, ResolvedAxis>,
    plot: Rect,
    request: &LayoutRequest,
) -> ChartResult<BTreeMap<GuideId, GuideTransitionFrame>> {
    guides
        .values()
        .filter(|g| {
            g.spec.visible
                && (g.spec.ggplot_axis.is_some()
                    || g.spec.profile != GuideProfile::LibraryV1
                    || g.spec.geometry.is_some()
                    || g.spec.components.is_some())
        })
        .map(|guide| {
            let g = Geometry::resolve(&guide.spec.style, request);
            let side = guide.spec.side;
            let horizontal = side.horizontal();
            let sign = if matches!(side, AxisSide::Top | AxisSide::Left) {
                -1.
            } else {
                1.
            };
            let range =
                if guide.spec.profile == GuideProfile::LibraryV1 && guide.spec.geometry.is_none() {
                    if horizontal {
                        crate::scales::Bounds::new(plot.origin().x(), plot.max_x())?
                    } else {
                        crate::scales::Bounds::new(plot.origin().y(), plot.max_y())?
                    }
                } else {
                    guide_geometry::range(&axes[&guide.spec.scale], axes)
                };
            let numbers = match (horizontal, g.outer != 0.) {
                (true, true) => vec![
                    range.start() + g.offset,
                    sign * g.outer,
                    g.offset,
                    range.end() + g.offset,
                    sign * g.outer,
                ],
                (true, false) => vec![range.start() + g.offset, g.offset, range.end() + g.offset],
                (false, true) => vec![
                    sign * g.outer,
                    range.start() + g.offset,
                    g.offset,
                    range.end() + g.offset,
                    sign * g.outer,
                ],
                (false, false) => vec![g.offset, range.start() + g.offset, range.end() + g.offset],
            };
            let mut translation = guide.spec.translation;
            match side {
                AxisSide::Top => translation[1] += plot.origin().y(),
                AxisSide::Bottom => translation[1] += plot.max_y(),
                AxisSide::Left => translation[0] += plot.origin().x(),
                AxisSide::Right => translation[0] += plot.max_x(),
            }
            let ticks = guide
                .ticks
                .iter()
                .zip(&guide.tick_indices)
                .enumerate()
                .map(|(identity, (tick, &index))| GuideTransitionTick {
                    identity,
                    value: tick.value.clone(),
                    index,
                    label: tick.label.clone(),
                    position: tick.position - guide.spec.translation[usize::from(!horizontal)],
                    opacity: 1.,
                    line_end: sign * g.inner,
                    label_offset: sign * g.spacing(),
                })
                .collect();
            Ok((
                guide.spec.id,
                GuideTransitionFrame {
                    side,
                    translation,
                    domain: GuideDomain {
                        horizontal,
                        caps: g.outer != 0.,
                        numbers,
                    },
                    ticks,
                    next_identity: guide.ticks.len(),
                },
            ))
        })
        .collect()
}
fn visit<'a>(
    chart: &'a LaidOutChart,
    scope: &mut Vec<String>,
    typed: &mut Vec<GuideScope>,
    out: &mut BTreeMap<Key, (&'a LaidOutChart, Vec<GuideScope>)>,
) {
    for &id in chart.guide_frames.keys() {
        out.insert((scope.clone(), id), (chart, typed.clone()));
    }
    for panel in &chart.panels {
        scope.push(format!(
            "panel:{}",
            serde_json::to_string(&panel.key).expect("bounded panel key")
        ));
        typed.push(GuideScope::Panel(panel.key.clone()));
        visit(&panel.chart, scope, typed, out);
        typed.pop();
        scope.pop();
    }
    for inset in &chart.insets {
        scope.push(format!("inset:{}", inset.id));
        typed.push(GuideScope::Inset(inset.id.clone()));
        visit(&inset.chart, scope, typed, out);
        typed.pop();
        scope.pop();
    }
}
fn charts(chart: &LaidOutChart) -> BTreeMap<Key, (&LaidOutChart, Vec<GuideScope>)> {
    let mut out = BTreeMap::new();
    visit(chart, &mut vec![], &mut vec![], &mut out);
    out
}
fn capture(chart: &LaidOutChart) -> BTreeMap<Key, GuidePresentationSnapshot> {
    if let Some(p) = &chart.guide_presentation {
        return p.frames.clone();
    }
    charts(chart)
        .into_iter()
        .map(|(key, (chart, scope))| {
            let frame = chart.guide_frames[&key.1].clone();
            let guide = key.1;
            (
                key,
                GuidePresentationSnapshot {
                    scope,
                    guide,
                    frame,
                },
            )
        })
        .collect()
}
fn offset(frame: &GuideTransitionFrame) -> f64 {
    frame.domain.numbers[if frame.domain.caps {
        2
    } else {
        usize::from(frame.domain.horizontal)
    }]
}
fn mapped(
    chart: &LaidOutChart,
    id: GuideId,
    value: &ScaleValue,
    position: bool,
) -> ChartResult<Option<f64>> {
    let guide = &chart.guides[&id];
    let axis = &chart.axes[&guide.spec.scale];
    let pixel_offset = offset(&chart.guide_frames[&id]);
    let result = match (&axis.scale, value) {
        (ResolvedScale::Band(s), ScaleValue::Category(v)) => {
            if position {
                s.guide_position(v, pixel_offset)
            } else {
                s.start(v)
            }
        }
        (ResolvedScale::Provider(s), v) => {
            if position {
                s.guide_position(v, pixel_offset)
            } else {
                s.map(v)
            }
        }
        _ if position => axis.guide_value_position(value),
        _ => axis.map_value(value),
    }?;
    Ok(result.map(|p| p + if position { pixel_offset } else { 0. }))
}
fn key(component: &GuideComponent) -> Key {
    (component.scope.clone(), component.guide)
}
#[derive(Clone, Debug, Default)]
struct Components {
    domain: Vec<SceneItem>,
    ticks: BTreeMap<usize, Vec<SceneItem>>,
}
fn components(
    chart: &LaidOutChart,
    frames: &BTreeMap<Key, GuidePresentationSnapshot>,
) -> BTreeMap<Key, Components> {
    let mut result: BTreeMap<Key, Components> = BTreeMap::new();
    // Index maps are built once; even a large tick list remains O(n log n).
    let indices: BTreeMap<_, BTreeMap<_, _>> = frames
        .iter()
        .map(|(k, f)| {
            (
                k.clone(),
                f.frame
                    .ticks
                    .iter()
                    .map(|t| (t.index, t.identity))
                    .collect(),
            )
        })
        .collect();
    for item in chart.scene.items() {
        if let Some(c) = &item.guide {
            let k = key(c);
            let Some(index) = indices.get(&k) else {
                continue;
            };
            let group = result.entry(k).or_default();
            if c.role == GuideRole::Domain {
                group.domain.push(item.clone());
            } else if let Some(id) = c
                .animation
                .map(|a| a.identity)
                .or_else(|| c.index.and_then(|i| index.get(&i).copied()))
            {
                group.ticks.entry(id).or_default().push(item.clone());
            }
        }
    }
    result
}
#[derive(Clone, Debug)]
struct Animation {
    plan: GuideTransitionPlan,
    templates: Components,
    reference: BTreeMap<usize, GuideTransitionTick>,
    domain_frame: GuideTransitionFrame,
}
/// Interpolate guide decorations while retaining the current immutable data scene.
/// New/removed guides and orientation replacements are immediate. Existing guides
/// retain D3 joins, exit groups and interruption positions without relayout at sample time.
#[derive(Clone, Debug)]
pub struct LayoutGuideTransition {
    target: Arc<LaidOutChart>,
    frames: BTreeMap<Key, GuidePresentationSnapshot>,
    animations: BTreeMap<Key, Animation>,
    limits: Limits,
}
impl LayoutGuideTransition {
    /// Prepare from two immutable layouts. Resource replacement requires immediate
    /// presentation instead, so exiting text never resolves against a changed font.
    pub fn new(
        from: Arc<LaidOutChart>,
        target: Arc<LaidOutChart>,
        limits: Limits,
    ) -> ChartResult<Self> {
        if from.scene.resources() != target.scene.resources()
            || from.scene.units() != target.scene.units()
        {
            return Err(crate::scales::error(
                DiagnosticCode::UnsupportedCapability,
                "Guide animation requires identical resource revisions and units; replace presentation immediately.",
            ));
        }
        crate::limits::require_within(
            from.scene
                .items()
                .len()
                .saturating_add(target.scene.items().len())
                <= limits.max_items,
            "guide transition scene union",
        )?;
        let prior = capture(&from);
        let mut frames = capture(&target);
        let old_components = components(&from, &prior);
        let new_components = components(&target, &frames);
        let old_charts = charts(&from);
        let new_charts = charts(&target);
        let mut animations = BTreeMap::new();
        for (k, current) in frames.clone() {
            let k = &k;
            let Some(old) = prior.get(k) else { continue };
            let Some((old_chart, _)) = old_charts.get(k) else {
                continue;
            };
            let (new_chart, _) = &new_charts[k];
            let plan = GuideTransitionPlan::new(
                old.frame.clone(),
                current.frame.clone(),
                |v| {
                    Ok(mapped(old_chart, k.1, v, true)?.map(|p| {
                        p - offset(&old_chart.guide_frames[&k.1])
                            + offset(&new_chart.guide_frames[&k.1])
                    }))
                },
                |v| mapped(new_chart, k.1, v, false),
                |v| mapped(new_chart, k.1, v, true),
                limits,
            )?;
            let final_frame = plan.sample(1.)?;
            frames.get_mut(k).expect("target guide").frame = final_frame.clone();
            let old_group = old_components.get(k).cloned().unwrap_or_default();
            let new_group = new_components.get(k).cloned().unwrap_or_default();
            let mut templates = Components {
                domain: new_group.domain,
                ticks: BTreeMap::new(),
            };
            let mut reference = BTreeMap::new();
            for (original, tick) in current.frame.ticks.iter().zip(&final_frame.ticks) {
                templates.ticks.insert(
                    tick.identity,
                    new_group
                        .ticks
                        .get(&original.identity)
                        .cloned()
                        .unwrap_or_default(),
                );
                reference.insert(tick.identity, tick.clone());
            }
            let old_ticks: BTreeMap<_, _> =
                old.frame.ticks.iter().map(|t| (t.identity, t)).collect();
            for (id, exit) in plan.identities() {
                if exit {
                    templates
                        .ticks
                        .insert(id, old_group.ticks.get(&id).cloned().unwrap_or_default());
                    // Exit templates are measured relative to the displayed prior frame.
                    let tick = old_ticks[&id];
                    let mut base = tick.clone();
                    let along = usize::from(!old.frame.domain.horizontal);
                    let cross = 1 - along;
                    base.position += old.frame.translation[along] - final_frame.translation[along];
                    base.label_offset +=
                        old.frame.translation[cross] - final_frame.translation[cross];
                    reference.insert(id, base);
                }
            }
            animations.insert(
                k.clone(),
                Animation {
                    plan,
                    templates,
                    reference,
                    domain_frame: final_frame,
                },
            );
        }
        let mut final_target = (*target).clone();
        final_target.guide_presentation = Some(Arc::new(Presentation {
            frames: frames.clone(),
            snapshots: target.guide_snapshots(),
        }));
        Ok(Self {
            target: Arc::new(final_target),
            frames,
            animations,
            limits,
        })
    }
    /// Produce a coherent immutable scene for painting, inspection and publication.
    /// At 1 the original target is returned, preserving exact update-versus-fresh output.
    pub fn sample(&self, fraction: f64) -> ChartResult<Arc<LaidOutChart>> {
        if !fraction.is_finite() || !(0. ..=1.).contains(&fraction) {
            return Err(crate::scales::error(
                DiagnosticCode::Validation,
                "Guide animation fraction must be finite in [0,1].",
            ));
        }
        if fraction == 1. {
            return Ok(self.target.clone());
        }
        let mut frames = self.frames.clone();
        let mut rendered = BTreeMap::new();
        for (k, animation) in &self.animations {
            let frame = animation.plan.sample(fraction)?;
            let mut items = vec![];
            for template in &animation.templates.domain {
                let mut item = template.clone();
                let commands = translated(&frame.domain.commands()?, frame.translation)?;
                match &mut item.primitive {
                    Primitive::Path { commands: c, .. }
                    | Primitive::DashedPath { commands: c, .. } => *c = commands,
                    Primitive::Rule { from, to, .. } => {
                        let PathCommand::MoveTo(a) = commands[0] else {
                            unreachable!()
                        };
                        let PathCommand::LineTo(b) = commands[1] else {
                            unreachable!()
                        };
                        *from = a;
                        *to = b;
                    }
                    _ => unreachable!("guide domain primitive"),
                }
                items.push(item);
            }
            let along = usize::from(!frame.domain.horizontal);
            let cross = 1 - along;
            for tick in &frame.ticks {
                let Some(templates) = animation.templates.ticks.get(&tick.identity) else {
                    continue;
                };
                let reference = &animation.reference[&tick.identity];
                for template in templates {
                    let mut item = template.clone();
                    let component = item.guide.as_mut().expect("guide template");
                    component.animation = Some(GuideAnimation {
                        identity: tick.identity,
                        opacity: tick.opacity,
                    });
                    if component.role == GuideRole::Line {
                        let mut start = frame.translation;
                        start[along] += tick.position;
                        let mut end = start;
                        end[cross] += tick.line_end;
                        let a = Point::new(start[0], start[1])?;
                        let b = Point::new(end[0], end[1])?;
                        match &mut item.primitive {
                            Primitive::Rule { from, to, .. } => {
                                *from = a;
                                *to = b
                            }
                            Primitive::DashedPath { commands, .. } => {
                                *commands = vec![PathCommand::MoveTo(a), PathCommand::LineTo(b)]
                            }
                            _ => unreachable!("guide line primitive"),
                        }
                    } else {
                        let mut delta = [
                            frame.translation[0] - animation.domain_frame.translation[0],
                            frame.translation[1] - animation.domain_frame.translation[1],
                        ];
                        delta[along] += tick.position - reference.position;
                        delta[cross] += tick.label_offset - reference.label_offset;
                        match &mut item.primitive {
                            Primitive::Text { origin, .. } | Primitive::GlyphRun { origin, .. } => {
                                *origin = Point::new(origin.x() + delta[0], origin.y() + delta[1])?
                            }
                            _ => unreachable!("guide label primitive"),
                        }
                    }
                    items.push(item);
                }
            }
            frames.get_mut(k).expect("target guide").frame = frame;
            rendered.insert(k.clone(), items);
        }
        let mut result = (*self.target).clone();
        let mut items = vec![];
        let mut targets = vec![];
        let mut panels = vec![];
        let mut interactions = BTreeMap::new();
        let mut emitted = std::collections::BTreeSet::new();
        for (index, item) in self.target.scene.items().iter().enumerate() {
            if let Some(c) = &item.guide
                && let Some(replacement) = rendered.get(&key(c))
            {
                if emitted.insert(key(c)) {
                    items.extend(replacement.iter().cloned());
                    targets.extend(std::iter::repeat_n(vec![], replacement.len()));
                    panels.extend(std::iter::repeat_n(
                        self.target.item_panels[index].clone(),
                        replacement.len(),
                    ));
                }
            } else {
                if let Some(interaction) = self.target.interactions.get(&index) {
                    interactions.insert(items.len(), interaction.clone());
                }
                items.push(item.clone());
                targets.push(self.target.targets[index].clone());
                panels.push(self.target.item_panels[index].clone());
            }
        }
        result.scene = Scene::new(
            self.target.scene.stamp(),
            self.target.scene.units(),
            self.target.scene.bounds(),
            &items,
            self.target.scene.resources(),
            self.limits,
        )?;
        result.targets = targets;
        result.item_panels = panels;
        result.interactions = interactions;
        let mut snapshots = self.target.guide_snapshots();
        for snapshot in &mut snapshots {
            if let Some(p) = frames
                .values()
                .find(|p| p.scope == snapshot.scope && p.guide == snapshot.spec.id)
            {
                let along = usize::from(!p.frame.domain.horizontal);
                snapshot.ticks = p
                    .frame
                    .ticks
                    .iter()
                    .map(|t| GuideTick {
                        value: t.value.clone(),
                        position: t.position + p.frame.translation[along],
                        label: t.label.clone(),
                    })
                    .collect();
            }
        }
        result.guide_presentation = Some(Arc::new(Presentation { frames, snapshots }));
        Ok(Arc::new(result))
    }
}
fn translated(commands: &[PathCommand], delta: [f64; 2]) -> ChartResult<Vec<PathCommand>> {
    commands
        .iter()
        .map(|c| {
            Ok(match c {
                PathCommand::MoveTo(p) => {
                    PathCommand::MoveTo(Point::new(p.x() + delta[0], p.y() + delta[1])?)
                }
                PathCommand::LineTo(p) => {
                    PathCommand::LineTo(Point::new(p.x() + delta[0], p.y() + delta[1])?)
                }
                _ => unreachable!("compact axis path"),
            })
        })
        .collect()
}
