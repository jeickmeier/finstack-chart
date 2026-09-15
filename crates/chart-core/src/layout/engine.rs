use super::{
    axes::resolve_axis,
    project::{self, Output},
    *,
};
use crate::grammar::{PreparedChart, PreparedGeometry, ValueSpace};
use crate::limits::require_within;
use crate::scales::*;
use crate::scene::{
    Color, GuideComponent, GuideRole, GuideTickIdentity, PathCommand, Primitive, Scene, SceneItem,
};
use crate::services::{TextMeasurer, TextMetrics, TextRequest, measure_text, validate_text};
use crate::{
    ChartResult, Diagnostic, DiagnosticCode, GuideId, Point, Rect, ScaleId, SceneStamp, Severity,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
const INK: Color = Color {
    red: 55,
    green: 60,
    blue: 65,
    alpha: 255,
};

pub(super) fn pressure(message: &str) -> Diagnostic {
    let mut d = error(DiagnosticCode::LayoutPressure, message);
    d.severity = Severity::Warning;
    d
}
pub(super) fn text_request<'a>(r: &'a LayoutRequest, text: &'a str) -> TextRequest<'a> {
    TextRequest {
        text,
        font: &r.font,
        font_size: r.font_size,
        units: r.units,
    }
}
fn preflight(chart: &PreparedChart, r: &LayoutRequest) -> ChartResult<()> {
    require_within(
        r.hierarchy_history.len() <= r.limits.max_items && r.hierarchy_scope.len() <= 32,
        "hierarchy history scope",
    )?;
    let mut history_members = 0usize;
    let mut history_scopes = std::collections::BTreeSet::new();
    for entry in r.hierarchy_history.iter() {
        if !history_scopes.insert((&entry.scope, entry.layer)) {
            return Err(error(
                DiagnosticCode::Validation,
                "Hierarchy history contains a duplicate destination scope.",
            ));
        }
        history_members = history_members.saturating_add(entry.history.membership_count());
        require_within(
            entry.scope.len() <= 32 && history_members <= r.max_vertices,
            "hierarchy retained history member",
        )?;
    }
    if r.device_scale
        .is_some_and(|scale| !scale.is_finite() || scale <= 0.)
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Device scale must be finite and positive.",
        ));
    }
    require_within(r.axes.len() <= 4, "independent axis count (four)")?;
    if !(2..=128).contains(&r.target_ticks)
        || !(2..=4096).contains(&r.max_ticks)
        || r.target_ticks > r.max_ticks
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Tick target must be 2..128 and no larger than max_ticks (2..4096).",
        ));
    }
    for v in [r.padding, r.tick_length, r.label_gap] {
        if !v.is_finite() || v < 0. {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Layout spacing must be finite and nonnegative.",
            ));
        }
    }
    for v in [r.minimum_plot.0, r.minimum_plot.1] {
        crate::geometry::positive(v, "Minimum plot dimensions must be finite and positive.")?;
    }
    validate_text(text_request(r, ""), r.limits)?;
    require_within(
        r.limits.max_resources >= 1 && r.font.byte_len <= r.limits.max_total_resource_bytes,
        "layout font resource",
    )?;
    let mut category_bytes = r.limits.max_text_bytes;
    let mut charge_categories = |labels: &[String]| -> ChartResult<()> {
        for label in labels {
            require_within(label.len() <= category_bytes, "layout category text byte")?;
            category_bytes -= label.len();
        }
        Ok(())
    };
    super::axes::validate_specs(&r.axes, r.limits)?;
    super::axes::validate_guides(&r.axes, &r.guides, r.limits)?;
    let ids: BTreeSet<_> = r.axes.iter().map(|a| a.id).collect();
    for a in &r.axes {
        let explicit_categories = match &a.scale {
            AxisScale::Band(o) => o.domain.as_ref(),
            AxisScale::Point(o) => o.domain.as_ref(),
            AxisScale::D3Band(o) => o.domain.as_ref(),
            AxisScale::D3Point(o) => o.domain.as_ref(),
            _ => None,
        };
        if let Some(d) = explicit_categories {
            require_within(d.len() <= r.max_categories, "explicit category")?;
            charge_categories(d)?;
        }
        if let Some(d) = chart.scale_domains().get(&a.id) {
            if a.side.horizontal() != d.x_space.is_some() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Guide side disagrees with the named scale orientation.",
                ));
            }
            for space in [&d.x_space, &d.y_space].into_iter().flatten() {
                if let ValueSpace::NullableCategorical { categories } = space {
                    require_within(categories.len() <= r.max_categories, "trained category")?;
                    charge_categories(&categories.iter().flatten().cloned().collect::<Vec<_>>())?;
                }
                if let ValueSpace::Categorical { categories } = space {
                    require_within(categories.len() <= r.max_categories, "trained category")?;
                    charge_categories(categories)?;
                }
            }
        }
    }
    if chart.scale_domains().keys().any(|id| !ids.contains(id)) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Every layer scale binding requires an AxisSpec, including hidden guides.",
        ));
    }
    let mut paths = r.limits.max_path_commands;
    let mut items = r.limits.max_items;
    for l in chart.layers().iter().filter(|l| l.visible()) {
        for m in l.marks() {
            let n = super::work::vertices(&m.geometry);
            if matches!(
                m.geometry,
                PreparedGeometry::HierarchyNode(_) | PreparedGeometry::HierarchyLink { .. }
            ) {
                // Arc/circle/rectangle/link lowering emits at most sixteen bounded commands.
                require_within(16 <= paths, "hierarchy potential path command")?;
                paths -= 16;
            }

            // Omission can split a line into at most one singleton per input vertex.
            let count = if matches!(
                m.geometry,
                PreparedGeometry::LineRun(_)
                    | PreparedGeometry::BandRun { .. }
                    | PreparedGeometry::StackBandRun { .. }
            ) {
                n
            } else {
                1
            };
            require_within(count <= items, "layout potential item")?;
            items -= count;
            if matches!(
                m.geometry,
                PreparedGeometry::LineRun(_)
                    | PreparedGeometry::BandRun { .. }
                    | PreparedGeometry::StackBandRun { .. }
            ) {
                let n = if matches!(
                    m.geometry,
                    PreparedGeometry::BandRun { .. } | PreparedGeometry::StackBandRun { .. }
                ) {
                    n.saturating_mul(2)
                } else {
                    n
                };
                require_within(n <= paths, "layout path command")?;
                paths -= n;
            }
        }
    }
    let mut potential_items = r.limits.max_items - items;
    let no_population =
        !chart
            .layers()
            .iter()
            .filter(|l| l.visible())
            .any(|l| match l.table().rows() {
                crate::grammar::PreparedRows::Binned(bins) => bins.iter().any(|b| b.count > 0),
                crate::grammar::PreparedRows::Source(_)
                | crate::grammar::PreparedRows::Statistical(_) => !l.marks().is_empty(),
            });
    if no_population {
        potential_items = potential_items.checked_add(1).ok_or_else(|| {
            error(
                DiagnosticCode::ResourceLimit,
                "Compact scene item count overflow.",
            )
        })?;
    }
    let mut major_values = BTreeMap::new();
    let scales = r
        .axes
        .iter()
        .map(|spec| {
            Ok((
                spec.id,
                resolve_axis(
                    chart,
                    r,
                    spec,
                    Rect::new(0., 0., 1., 1.)?,
                    major_values.entry(spec.id).or_default(),
                )?,
            ))
        })
        .collect::<ChartResult<BTreeMap<_, _>>>()?;
    for guide in resolve_guides(chart, &scales, &major_values, r)?.values() {
        if guide.spec.visible {
            if guide.spec.profile == GuideProfile::D3_3_0_0 || guide.spec.geometry.is_some() {
                require_within(paths >= 4, "guide domain path command")?;
                paths -= 4;
            }
            potential_items = potential_items
                .checked_add(1 + guide.ticks.len() * 2)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::ResourceLimit,
                        "Guide scene item count overflow.",
                    )
                })?;
        }
    }
    require_within(
        potential_items <= r.limits.max_items,
        "layout potential mark/guide item",
    )?;
    Ok(())
}

#[derive(Clone)]
struct Label {
    dodge_offset: f64,
    tick: GuideTick,
    index: usize,
    identity: GuideTickIdentity,
    metrics: TextMetrics,
    rich: Option<super::text::Block>,
    font_size: f64,
    visible: bool,
    line_style: GuideLineStyle,
    text_style: GuideTextStyle,
}
fn exposed(spec: &GuideStyle) -> bool {
    spec.ggplot_axis.is_some()
        || spec.profile != GuideProfile::LibraryV1
        || spec.geometry.is_some()
        || spec.components.is_some()
}
fn component(spec: &GuideSpec, role: GuideRole, label: Option<&Label>) -> Option<GuideComponent> {
    exposed(&spec.style).then(|| GuideComponent {
        animation: None,
        scope: vec![],
        guide: spec.id,
        role,
        side: spec.side,
        tick: label.map(|l| l.identity.clone()),
        index: label.map(|l| l.index),
        label: label.map(|l| l.tick.label.clone()),
    })
}
fn measure_guides(
    axes: &BTreeMap<GuideId, ResolvedGuide>,
    r: &LayoutRequest,
    measurer: &dyn TextMeasurer,
) -> ChartResult<BTreeMap<GuideId, Vec<Label>>> {
    // Charge logical component metadata as well as displayed text before callbacks.
    let mut remaining = r.limits.max_text_bytes;
    for a in axes.values() {
        for t in &a.ticks {
            let bytes = if exposed(&a.spec.style) {
                t.label
                    .len()
                    .saturating_mul(3)
                    .saturating_add(match &t.value {
                        crate::composition::ScaleValue::Category(s) => s.len().saturating_mul(2),
                        _ => 0,
                    })
            } else {
                t.label.len()
            };
            require_within(
                bytes <= remaining,
                "layout measured guide and metadata byte",
            )?;
            remaining -= bytes;
        }
    }
    let mut labels = BTreeMap::new();
    for (id, a) in axes {
        let components = a.spec.components.clone().unwrap_or_default();
        let overrides = components.overrides();
        let mut occurrences = BTreeMap::<String, usize>::new();
        let mut measured = vec![];
        for (index, t) in a.ticks.iter().enumerate() {
            let key = if matches!(t.value, crate::composition::ScaleValue::Number(n) if n == 0.) {
                "number:zero".to_owned()
            } else {
                serde_json::to_string(&t.value)
                    .map_err(|_| error(DiagnosticCode::Validation, "Invalid guide identity."))?
            };
            let occurrence = occurrences.entry(key).or_default();
            let identity = GuideTickIdentity {
                value: t.value.clone(),
                occurrence: *occurrence,
            };
            *occurrence += 1;
            let (visible, line_style, text_style) = components.tick(overrides.get(&index).copied());
            let mut guide_request = r.clone();
            guide_request.font_size =
                text_style
                    .font_size
                    .unwrap_or(if a.spec.profile == GuideProfile::D3_3_0_0 {
                        10.
                    } else {
                        r.font_size
                    });
            let r = &guide_request;
            if t.label.is_empty() || !visible || text_style.visible == Some(false) {
                measured.push(Label {
                    dodge_offset: 0.,
                    tick: t.clone(),
                    index,
                    identity,
                    metrics: TextMetrics::new(0., 0., 0.)?,
                    rich: None,
                    font_size: r.font_size,
                    visible,
                    line_style,
                    text_style,
                });
                continue;
            }
            let typography = text_style
                .typography
                .as_ref()
                .or(a.spec.typography.as_ref());
            let rotation = text_style.rotation.unwrap_or(a.spec.label_rotation);
            let font_size = r.font_size * typography.map_or(1., |run| run.size);
            let rich = if typography.is_some() || rotation != 0. {
                let mut run = typography
                    .cloned()
                    .unwrap_or_else(|| crate::typography::RichRun::new(""));
                run.text.clear();
                if let Some(color) = text_style.color {
                    run.color = Some(color);
                }
                Some(super::text::measure(
                    &crate::typography::RichText {
                        lines: t
                            .label
                            .split('\n')
                            .map(|line| {
                                let mut run = run.clone();
                                run.text = line.into();
                                vec![run]
                            })
                            .collect(),
                        line_spacing: 1.2,
                        rotation,
                    },
                    r,
                    measurer,
                    r.host_theme
                        .foreground
                        .map(crate::color::Paint::resolve)
                        .unwrap_or(INK),
                )?)
            } else if t.label.contains('\n') {
                Some(super::text::plain_lines(
                    &t.label,
                    r,
                    measurer,
                    text_style
                        .color
                        .map(crate::color::Paint::resolve)
                        .or_else(|| r.host_theme.foreground.map(crate::color::Paint::resolve))
                        .unwrap_or(INK),
                )?)
            } else {
                None
            };
            let metrics = if let Some(b) = &rich {
                TextMetrics::new(b.bounds.width(), b.bounds.height(), 0.)?
            } else {
                measure_text(measurer, text_request(r, &t.label), r.limits)?
            };
            measured.push(Label {
                dodge_offset: 0.,
                tick: t.clone(),
                index,
                identity,
                metrics,
                rich,
                font_size,
                visible,
                line_style,
                text_style,
            });
        }
        if let Some(options) = &a.spec.ggplot_axis {
            let rows = options.n_dodge.min(measured.len());
            let mut sizes = vec![0_f64; rows];
            for l in &measured {
                let size = if a.spec.side.horizontal() {
                    l.metrics.height()
                } else {
                    l.metrics.width()
                };
                sizes[l.index % rows] = sizes[l.index % rows].max(size);
            }
            let mut offsets = vec![0.; rows];
            for row in 1..rows {
                offsets[row] = offsets[row - 1] + sizes[row - 1] + r.label_gap;
            }
            for l in &mut measured {
                l.dodge_offset = offsets[l.index % rows];
            }
        }
        labels.insert(*id, measured);
    }
    Ok(labels)
}
fn guide_extent(a: &ResolvedGuide, labels: &[Label], r: &LayoutRequest) -> f64 {
    let geometry = super::guide_geometry::Geometry::resolve(&a.spec.style, r);
    let labels = labels
        .iter()
        .map(|l| {
            l.dodge_offset
                + if a.spec.side.horizontal() {
                    l.metrics.height()
                } else {
                    l.metrics.width()
                }
        })
        .fold(0., f64::max);
    let tick = a
        .spec
        .ggplot_axis
        .as_ref()
        .and_then(|o| o.logticks.as_ref())
        .map_or(geometry.inner, |o| {
            o.lengths
                .iter()
                .map(|factor| factor * geometry.inner)
                .fold(0., f64::max)
        });
    geometry
        .outer
        .max(tick)
        .max(geometry.spacing() + labels)
        .max(0.)
}
fn stack_guides(
    axes: &mut BTreeMap<GuideId, ResolvedGuide>,
    labels: &BTreeMap<GuideId, Vec<Label>>,
    titles: &BTreeMap<GuideId, super::text::Block>,
    r: &LayoutRequest,
) {
    let mut ordered = axes
        .values()
        .filter_map(|a| {
            a.spec
                .ggplot_axis
                .as_ref()?
                .stack_order
                .map(|order| (order, a.spec.id))
        })
        .collect::<Vec<_>>();
    ordered.sort();
    let mut offsets = BTreeMap::new();
    for (_, id) in ordered {
        let a = axes.get_mut(&id).unwrap();
        if !a.spec.visible {
            continue;
        }
        let side = match a.spec.side {
            AxisSide::Left => 0,
            AxisSide::Right => 1,
            AxisSide::Top => 2,
            AxisSide::Bottom => 3,
        };
        let offset = offsets.entry((a.spec.scale, side)).or_insert(0.);
        let sign = if matches!(a.spec.side, AxisSide::Left | AxisSide::Top) {
            -1.
        } else {
            1.
        };
        a.spec.translation[usize::from(a.spec.side.horizontal())] += sign * *offset;
        let title = titles.get(&id).map_or(0., |t| {
            r.label_gap
                + if a.spec.side.horizontal() {
                    t.bounds.height()
                } else {
                    t.bounds.width()
                }
        });
        *offset += guide_extent(a, &labels[&id], r)
            + title
            + a.spec.ggplot_axis.as_ref().unwrap().stack_spacing;
    }
}
// [left, right, top, bottom]. Monotonic margins avoid tick-count oscillation.
fn margins(
    axes: &BTreeMap<GuideId, ResolvedGuide>,
    labels: &BTreeMap<GuideId, Vec<Label>>,
    titles: &BTreeMap<GuideId, super::text::Block>,
    r: &LayoutRequest,
    mut m: [f64; 4],
) -> [f64; 4] {
    for (id, a) in axes {
        if !a.spec.visible {
            continue;
        }
        let geometry = super::guide_geometry::Geometry::resolve(&a.spec.style, r);
        if a.spec.ggplot_axis.is_some() {
            let (side, outward) = match a.spec.side {
                AxisSide::Left => (0, -a.spec.translation[0]),
                AxisSide::Right => (1, a.spec.translation[0]),
                AxisSide::Top => (2, -a.spec.translation[1]),
                AxisSide::Bottom => (3, a.spec.translation[1]),
            };
            m[side] = m[side].max(r.padding + guide_extent(a, &labels[id], r) + outward.max(0.));
        }
        if a.spec.profile == GuideProfile::D3_3_0_0 || a.spec.geometry.is_some() {
            let (side, outward) = match a.spec.side {
                AxisSide::Left => (0, -a.spec.translation[0]),
                AxisSide::Right => (1, a.spec.translation[0]),
                AxisSide::Top => (2, -a.spec.translation[1]),
                AxisSide::Bottom => (3, a.spec.translation[1]),
            };
            m[side] = m[side].max(r.padding + geometry.outer.max(0.) + outward.max(0.));
        }
        for l in &labels[id] {
            let w = l.metrics.width();
            let h = l.metrics.height();
            let (side, size) = match a.spec.side {
                AxisSide::Left => (0, w),
                AxisSide::Right => (1, w),
                AxisSide::Top => (2, h),
                AxisSide::Bottom => (3, h),
            };
            let title = titles.get(id).map_or(0., |t| {
                r.label_gap
                    + if a.spec.side.horizontal() {
                        t.bounds.height()
                    } else {
                        t.bounds.width()
                    }
            });
            let outward = match a.spec.side {
                AxisSide::Left => -a.spec.translation[0],
                AxisSide::Right => a.spec.translation[0],
                AxisSide::Top => -a.spec.translation[1],
                AxisSide::Bottom => a.spec.translation[1],
            };
            m[side] = m[side].max(
                r.padding
                    + geometry
                        .outer
                        .max(geometry.spacing() + size + l.dodge_offset + title)
                        .max(0.)
                    + outward.max(0.),
            );
            if a.spec.side.horizontal() {
                m[0] = m[0].max(r.padding + w / 2.);
                m[1] = m[1].max(r.padding + w / 2.);
            } else {
                m[2] = m[2].max(r.padding + h / 2.);
                m[3] = m[3].max(r.padding + h / 2.);
            }
        }
    }
    m
}
fn plot(r: &LayoutRequest, m: [f64; 4]) -> ChartResult<Option<Rect>> {
    let w = r.bounds.width() - m[0] - m[1];
    let h = r.bounds.height() - m[2] - m[3];
    if w < r.minimum_plot.0 || h < r.minimum_plot.1 {
        return Ok(None);
    }
    Ok(Some(Rect::new(
        r.bounds.origin().x() + m[0],
        r.bounds.origin().y() + m[2],
        w,
        h,
    )?))
}
fn base_label_geometry(
    spec: &GuideSpec,
    l: &Label,
    p: Rect,
    r: &LayoutRequest,
) -> ChartResult<(Point, Rect, Point, Point)> {
    let side = spec.side;
    let geometry = super::guide_geometry::Geometry::resolve(&spec.style, r);
    let v = l.tick.position;
    if spec.profile == GuideProfile::D3_3_0_0 || spec.geometry.is_some() {
        let anchor = super::guide_geometry::point(side, v, geometry.spacing(), p)?;
        let dy = match side {
            AxisSide::Bottom => 0.71,
            AxisSide::Top => 0.,
            _ => 0.32,
        };
        let size = l.font_size;
        let x = match side {
            AxisSide::Top | AxisSide::Bottom => anchor.x() - l.metrics.width() / 2.,
            AxisSide::Left => anchor.x() - l.metrics.width(),
            AxisSide::Right => anchor.x(),
        };
        let baseline = anchor.y() + dy * size;
        let top = if l.rich.is_some() && l.tick.label.contains('\n') {
            // Multiline blocks align as a whole outside horizontal axes and around
            // the tick center on vertical axes, independent of the line count.
            match side {
                AxisSide::Bottom => anchor.y(),
                AxisSide::Top => anchor.y() - l.metrics.height(),
                _ => anchor.y() - l.metrics.height() / 2.,
            }
        } else {
            baseline - l.metrics.ascent()
        };
        return Ok((
            Point::new(x, baseline)?,
            Rect::new(x, top, l.metrics.width(), l.metrics.height())?,
            super::guide_geometry::point(side, v, 0., p)?,
            super::guide_geometry::point(side, v, geometry.inner, p)?,
        ));
    }
    let w = l.metrics.width();
    let h = l.metrics.height();
    let gap = r.tick_length + r.label_gap;
    let (x, y, from, to) = match side {
        AxisSide::Bottom => (
            v - w / 2.,
            p.max_y() + gap,
            Point::new(v, p.max_y())?,
            Point::new(v, p.max_y() + r.tick_length)?,
        ),
        AxisSide::Top => (
            v - w / 2.,
            p.origin().y() - gap - h,
            Point::new(v, p.origin().y())?,
            Point::new(v, p.origin().y() - r.tick_length)?,
        ),
        AxisSide::Left => (
            p.origin().x() - gap - w,
            v - h / 2.,
            Point::new(p.origin().x(), v)?,
            Point::new(p.origin().x() - r.tick_length, v)?,
        ),
        AxisSide::Right => (
            p.max_x() + gap,
            v - h / 2.,
            Point::new(p.max_x(), v)?,
            Point::new(p.max_x() + r.tick_length, v)?,
        ),
    };
    Ok((
        Point::new(x, y + l.metrics.ascent())?,
        Rect::new(x, y, w, h)?,
        from,
        to,
    ))
}
fn label_geometry(
    spec: &GuideSpec,
    l: &Label,
    p: Rect,
    r: &LayoutRequest,
) -> ChartResult<(Point, Rect, Point, Point)> {
    let (origin, bounds, from, to) = base_label_geometry(spec, l, p, r)?;
    let (dx, dy) = match spec.side {
        AxisSide::Left => (-l.dodge_offset, 0.),
        AxisSide::Right => (l.dodge_offset, 0.),
        AxisSide::Top => (0., -l.dodge_offset),
        AxisSide::Bottom => (0., l.dodge_offset),
    };
    Ok((
        Point::new(origin.x() + dx, origin.y() + dy)?,
        Rect::new(
            bounds.origin().x() + dx,
            bounds.origin().y() + dy,
            bounds.width(),
            bounds.height(),
        )?,
        from,
        to,
    ))
}
fn inside(a: Rect, b: Rect) -> bool {
    a.origin().x() >= b.origin().x()
        && a.origin().y() >= b.origin().y()
        && a.max_x() <= b.max_x()
        && a.max_y() <= b.max_y()
}
fn overlaps(a: Rect, b: Rect, gap: f64) -> bool {
    a.origin().x() < b.max_x() + gap
        && b.origin().x() < a.max_x() + gap
        && a.origin().y() < b.max_y() + gap
        && b.origin().y() < a.max_y() + gap
}
fn styled_line(primitive: Primitive, style: &GuideLineStyle) -> Primitive {
    match (primitive, style.dashes.as_deref()) {
        (Primitive::Path { commands, stroke }, Some(dashes)) if !dashes.is_empty() => {
            Primitive::DashedPath {
                commands,
                stroke,
                dashes: dashes.to_vec(),
            }
        }
        (Primitive::Rule { from, to, stroke }, Some(dashes)) if !dashes.is_empty() => {
            Primitive::DashedPath {
                commands: vec![PathCommand::MoveTo(from), PathCommand::LineTo(to)],
                stroke,
                dashes: dashes.to_vec(),
            }
        }
        (primitive, _) => primitive,
    }
}
fn guides(
    scales: &BTreeMap<ScaleId, ResolvedAxis>,
    axes: &mut BTreeMap<GuideId, ResolvedGuide>,
    labels: &BTreeMap<GuideId, Vec<Label>>,
    p: Rect,
    r: &LayoutRequest,
    out: &mut Output,
) -> ChartResult<bool> {
    let mut pressure = false;
    let mut placed = vec![];
    let color = r
        .host_theme
        .foreground
        .map(crate::color::Paint::resolve)
        .unwrap_or(INK);
    for (id, a) in axes {
        if !a.spec.visible {
            continue;
        }
        let components = a.spec.components.clone().unwrap_or_default();
        let p = Rect::new(
            p.origin().x() + a.spec.translation[0],
            p.origin().y() + a.spec.translation[1],
            p.width(),
            p.height(),
        )?;
        let (mut from, mut to) = match a.spec.side {
            AxisSide::Bottom => (
                Point::new(p.origin().x(), p.max_y())?,
                Point::new(p.max_x(), p.max_y())?,
            ),
            AxisSide::Top => (p.origin(), Point::new(p.max_x(), p.origin().y())?),
            AxisSide::Left => (p.origin(), Point::new(p.origin().x(), p.max_y())?),
            AxisSide::Right => (
                Point::new(p.max_x(), p.origin().y())?,
                Point::new(p.max_x(), p.max_y())?,
            ),
        };
        let logticks = super::ggplot_axis::log_ticks(&scales[&a.spec.scale], &a.spec, r)?;
        if let Some(options) = &a.spec.ggplot_axis {
            let positions = if options.logticks.is_some() {
                logticks.iter().map(|t| t.1).collect::<Vec<_>>()
            } else {
                a.ticks.iter().map(|t| t.position).collect()
            };
            let min = positions.iter().copied().reduce(f64::min);
            let max = positions.iter().copied().reduce(f64::max);
            if let (Some(min), Some(max)) = (min, max) {
                if matches!(options.cap, AxisCap::Lower | AxisCap::Both) {
                    from = if a.spec.side.horizontal() {
                        Point::new(min, from.y())?
                    } else {
                        Point::new(from.x(), min)?
                    };
                }
                if matches!(options.cap, AxisCap::Upper | AxisCap::Both) {
                    to = if a.spec.side.horizontal() {
                        Point::new(max, to.y())?
                    } else {
                        Point::new(to.x(), max)?
                    };
                }
            }
        }
        let geometry = super::guide_geometry::Geometry::resolve(&a.spec.style, r);
        let clip = (geometry.overflow == GuideOverflow::Clip).then_some(r.bounds);
        if components.domain.visible != Some(false) {
            let stroke = components.domain.stroke(color);
            let primitive = if a.spec.ggplot_axis.is_none()
                && (a.spec.profile == GuideProfile::D3_3_0_0 || a.spec.geometry.is_some())
            {
                Primitive::Path {
                    commands: super::guide_geometry::domain(
                        &a.spec,
                        super::guide_geometry::range(&scales[&a.spec.scale], scales),
                        p,
                        geometry,
                    )?,
                    stroke,
                }
            } else {
                Primitive::Rule { from, to, stroke }
            };
            out.push(
                SceneItem {
                    guide: component(&a.spec, GuideRole::Domain, None),
                    layer: None,
                    clip,
                    primitive: styled_line(primitive, &components.domain),
                },
                vec![],
                r,
            )?;
        }
        a.ticks.clear();
        a.tick_indices.clear();
        let mut seen = BTreeSet::new();
        let mut overlap_hidden = BTreeSet::new();
        if let Some(options) = a.spec.ggplot_axis.as_ref().filter(|o| o.check_overlap) {
            let mut occupied = vec![];
            for i in super::ggplot_axis::label_priority(labels[id].len(), options.n_dodge) {
                let l = &labels[id][i];
                if !l.visible || l.text_style.visible == Some(false) || l.tick.label.is_empty() {
                    continue;
                }
                let (_, bounds, _, _) = label_geometry(&a.spec, l, p, r)?;
                if occupied.iter().any(|other| overlaps(bounds, *other, 0.)) {
                    overlap_hidden.insert(i);
                } else {
                    occupied.push(bounds);
                }
            }
        }
        for l in &labels[id] {
            let (origin, bounds, from, to) = label_geometry(&a.spec, l, p, r)?;
            let along = if a.spec.side.horizontal() {
                l.tick.position >= p.origin().x() && l.tick.position <= p.max_x()
            } else {
                l.tick.position >= p.origin().y() && l.tick.position <= p.max_y()
            };
            let shown_label =
                l.visible && l.text_style.visible != Some(false) && !l.tick.label.is_empty();
            let collision = !along
                || shown_label
                    && (!inside(bounds, r.bounds)
                        || placed.iter().any(|b| overlaps(bounds, *b, r.label_gap))
                        || !seen.insert(l.tick.label.clone()));
            let hide_label = overlap_hidden.contains(&l.index)
                || collision && geometry.labels == GuideLabelPolicy::HideLabels;
            if collision {
                pressure = true;
                if geometry.labels == GuideLabelPolicy::ThinTicks {
                    continue;
                }
            }
            if shown_label && !hide_label {
                placed.push(bounds);
            }
            a.ticks.push(l.tick.clone());
            a.tick_indices.push(l.index);
            if !l.visible {
                continue;
            }
            if l.line_style.visible != Some(false)
                && a.spec
                    .ggplot_axis
                    .as_ref()
                    .is_none_or(|o| o.logticks.is_none())
            {
                out.push(
                    SceneItem {
                        guide: component(&a.spec, GuideRole::Line, Some(l)),
                        layer: None,
                        clip: if geometry.clip_ticks { Some(p) } else { clip },
                        primitive: styled_line(
                            Primitive::Rule {
                                from,
                                to,
                                stroke: l.line_style.stroke(color),
                            },
                            &l.line_style,
                        ),
                    },
                    vec![],
                    r,
                )?;
            }
            if !shown_label || hide_label {
                continue;
            }
            if let Some(block) = &l.rich {
                for mut item in
                    block.items_at(bounds.origin().x(), bounds.origin().y(), r.bounds)?
                {
                    if exposed(&a.spec.style) {
                        item.clip = clip;
                    }
                    item.guide = component(&a.spec, GuideRole::Label, Some(l));
                    out.push(item, vec![], r)?;
                }
            } else {
                out.push(
                    SceneItem {
                        guide: component(&a.spec, GuideRole::Label, Some(l)),
                        layer: None,
                        clip,
                        primitive: Primitive::Text {
                            origin,
                            text: l.tick.label.clone(),
                            font: r.font.id,
                            font_size: l.font_size,
                            color: l
                                .text_style
                                .color
                                .map(crate::color::Paint::resolve)
                                .unwrap_or(color),
                        },
                    },
                    vec![],
                    r,
                )?;
            }
        }
        for (index, (value, position, factor)) in logticks.into_iter().enumerate() {
            let style = &components.ticks;
            if style.visible == Some(false) {
                continue;
            }
            let from = super::guide_geometry::point(a.spec.side, position, 0., p)?;
            let to =
                super::guide_geometry::point(a.spec.side, position, geometry.inner * factor, p)?;
            let guide = Some(GuideComponent {
                animation: None,
                scope: vec!["logtick".into()],
                guide: a.spec.id,
                role: GuideRole::Line,
                side: a.spec.side,
                tick: Some(GuideTickIdentity {
                    value,
                    occurrence: 0,
                }),
                index: Some(index),
                label: Some(String::new()),
            });
            out.push(
                SceneItem {
                    guide,
                    layer: None,
                    clip,
                    primitive: styled_line(
                        Primitive::Rule {
                            from,
                            to,
                            stroke: style.stroke(color),
                        },
                        style,
                    ),
                },
                vec![],
                r,
            )?;
        }
        if a.spec
            .ggplot_axis
            .as_ref()
            .is_some_and(|o| o.minor_ticks && o.logticks.is_none())
        {
            let mut occurrences = std::collections::BTreeMap::<String, usize>::new();
            for (index, tick) in a.minor_ticks.iter().enumerate() {
                let from = super::guide_geometry::point(a.spec.side, tick.position, 0., p)?;
                let to = super::guide_geometry::point(
                    a.spec.side,
                    tick.position,
                    geometry.inner / 2.,
                    p,
                )?;
                let style = &components.ticks;
                if style.visible == Some(false) {
                    continue;
                }
                let occurrence = if let Some(value) = &tick.value {
                    let count = occurrences
                        .entry(serde_json::to_string(value).expect("scale value"))
                        .or_default();
                    let occurrence = *count;
                    *count += 1;
                    occurrence
                } else {
                    0
                };
                let guide = tick.value.as_ref().map(|value| GuideComponent {
                    animation: None,
                    scope: vec!["minor".into()],
                    guide: a.spec.id,
                    role: GuideRole::Line,
                    side: a.spec.side,
                    tick: Some(GuideTickIdentity {
                        value: value.clone(),
                        occurrence,
                    }),
                    index: Some(index),
                    label: Some(String::new()),
                });
                out.push(
                    SceneItem {
                        guide,
                        layer: None,
                        clip,
                        primitive: styled_line(
                            Primitive::Rule {
                                from,
                                to,
                                stroke: style.stroke(color),
                            },
                            style,
                        ),
                    },
                    vec![],
                    r,
                )?;
            }
        }
        if matches!(&scales[&a.spec.scale].scale,ResolvedScale::Band(s) if s.domain().len()>labels[id].len())
        {
            pressure = true;
        }
    }
    Ok(pressure)
}
fn compact(
    text: &str,
    r: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    out: &mut Output,
) -> ChartResult<()> {
    let m = measure_text(measurer, text_request(r, text), r.limits)?;
    if m.width() <= r.bounds.width() && m.height() <= r.bounds.height() {
        out.push(
            SceneItem {
                guide: None,
                layer: None,
                clip: None,
                primitive: Primitive::Text {
                    origin: Point::new(
                        r.bounds.origin().x() + (r.bounds.width() - m.width()) / 2.,
                        r.bounds.origin().y() + (r.bounds.height() - m.height()) / 2. + m.ascent(),
                    )?,
                    text: text.into(),
                    font: r.font.id,
                    font_size: r.font_size,
                    color: r
                        .host_theme
                        .foreground
                        .map(crate::color::Paint::resolve)
                        .unwrap_or(INK),
                },
            },
            vec![],
            r,
        )?;
    }
    Ok(())
}

/// Resolve destination ranges, bounded text-aware margins, geometry and plain axes atomically.
/// Retains the supplied prepared snapshot; failures leave prior caller-owned scenes intact.
pub fn layout(
    prepared: Arc<PreparedChart>,
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
) -> ChartResult<LaidOutChart> {
    super::work::preflight(&prepared, request)?;
    let bounded = super::text::BoundedMeasurer::new(measurer, request.limits)?;
    let measurer = &bounded as &dyn TextMeasurer;
    let mut effective = request.clone();
    if !prepared.definition().axes.is_empty() {
        effective.axes.clone_from(&prepared.definition().axes);
    }
    if !prepared.definition().guides.is_empty() {
        effective.guides.clone_from(&prepared.definition().guides);
    }
    if prepared
        .state()
        .axis_windows()
        .keys()
        .any(|id| !effective.axes.iter().any(|a| a.id == *id))
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Navigation window names an absent axis.",
        ));
    }
    let theme = super::theme::tokens(&prepared, &effective)?;
    super::theme::configure(&theme, &mut effective);
    let full_request = effective.clone();
    let furniture = super::composition::prepare(&prepared, &effective, measurer)?;
    if let Some(f) = &furniture {
        effective.bounds = f.content;
        effective.figure_bounds = Some(full_request.bounds);
    }
    let legend = super::facets::prepare_single_legend(&prepared, &effective, measurer)?;
    if let Some(legend) = &legend {
        effective.bounds = legend.content;
    }
    let request = &effective;
    let stamp = SceneStamp {
        definition: prepared.definition_revision(),
        store: prepared.source().get()?.revision(),
        layout: request.revision,
        state: prepared.state().revision(),
        viewport: prepared.state().viewport_revision(),
    };
    layout_inner(prepared, request, measurer, stamp)
        .and_then(|mut chart| {
            chart.scene = Scene::new(
                chart.scene.stamp(),
                request.units,
                full_request.bounds,
                chart.scene.items(),
                chart.scene.resources(),
                request.limits,
            )?;
            if let Some(legend) = legend {
                super::facets::finish_single_legend(&mut chart, &full_request, legend)?;
            }
            super::theme::apply(&mut chart, &full_request, &theme)?;
            if let Some(furniture) = furniture {
                super::composition::finish(&mut chart, &full_request, measurer, &theme, furniture)?;
            }
            Ok(chart)
        })
        .map_err(|mut e| {
            e.context.stamp = Some(stamp);
            e
        })
}

fn layout_inner(
    prepared: Arc<PreparedChart>,
    request: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    stamp: SceneStamp,
) -> ChartResult<LaidOutChart> {
    if prepared.definition().facets.is_some() {
        return super::facets::layout_facets(prepared, request, measurer, stamp);
    }
    solve_panels(vec![(prepared, request.clone())], measurer, stamp)?
        .pop()
        .ok_or_else(|| error(DiagnosticCode::Validation, "No layout panel."))
}

fn guide_specs(r: &LayoutRequest) -> Vec<GuideSpec> {
    r.axes
        .iter()
        .map(AxisSpec::default_guide)
        .chain(r.guides.iter().cloned())
        .collect()
}
fn resolve_guides(
    chart: &PreparedChart,
    axes: &BTreeMap<ScaleId, ResolvedAxis>,
    major_values: &BTreeMap<ScaleId, super::guide_ticks::SelectedGuideValues>,
    r: &LayoutRequest,
) -> ChartResult<BTreeMap<GuideId, ResolvedGuide>> {
    let mut guides = BTreeMap::new();
    for axis in axes.values() {
        let mut spec = axis.spec.default_guide();
        if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
            && spec.profile == GuideProfile::LibraryV1
            && spec.ggplot_axis.is_none()
        {
            spec.ggplot_axis = Some(GgplotAxisOptions::default());
        }

        guides.insert(
            spec.id,
            ResolvedGuide {
                minor_ticks: super::minor_breaks::resolve(
                    chart,
                    axis,
                    &spec.style,
                    &axis.ticks,
                    major_values.get(&axis.spec.id),
                    r,
                )?,
                tick_indices: (0..axis.ticks.len()).collect(),
                spec,
                ticks: axis.ticks.clone(),
            },
        );
    }
    for original in &r.guides {
        let mut spec = original.clone();
        if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
            && spec.profile == GuideProfile::LibraryV1
            && spec.ggplot_axis.is_none()
        {
            spec.ggplot_axis = Some(GgplotAxisOptions::default());
        }
        let spec = &spec;
        let axis = axes
            .get(&spec.scale)
            .ok_or_else(|| error(DiagnosticCode::MissingResource, "Guide scale is absent."))?;
        let mut selected = super::guide_ticks::SelectedGuideValues::default();
        let ticks =
            super::guide_ticks::resolve_with_values(chart, axis, &spec.style, r, &mut selected)?;
        guides.insert(
            spec.id,
            ResolvedGuide {
                minor_ticks: super::minor_breaks::resolve(
                    chart,
                    axis,
                    &spec.style,
                    &ticks,
                    Some(&selected),
                    r,
                )?,
                tick_indices: (0..ticks.len()).collect(),
                spec: spec.clone(),
                ticks,
            },
        );
    }
    for guide in guides.values_mut() {
        let geometry = super::guide_geometry::Geometry::resolve(&guide.spec.style, r);
        let shift = guide.spec.translation[usize::from(!guide.spec.side.horizontal())];
        for tick in &mut guide.minor_ticks {
            tick.position += shift + geometry.offset;
            if !tick.position.is_finite() {
                return Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Minor guide geometry exceeds finite positions.",
                ));
            }
        }
        for tick in &mut guide.ticks {
            if guide.spec.profile == GuideProfile::D3_3_0_0 {
                let axis = &axes[&guide.spec.scale];
                let position = match (&axis.scale, &tick.value) {
                    (ResolvedScale::Provider(scale), value) => {
                        scale.guide_position(value, geometry.offset)?
                    }
                    (
                        ResolvedScale::Band(scale),
                        crate::composition::ScaleValue::Category(value),
                    ) => scale.guide_position(value, geometry.offset)?,
                    _ => Some(tick.position),
                };
                tick.position = position.ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "Retained guide tick lost its mapping.",
                    )
                })?;
            }
            tick.position += shift + geometry.offset;
            if !tick.position.is_finite() {
                return Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Guide geometry exceeds finite positions.",
                ));
            }
        }
    }
    Ok(guides)
}

/// One synchronized four-pass solve. All panels use the maximum required side margins.
pub(super) fn solve_panels(
    inputs: Vec<(Arc<PreparedChart>, LayoutRequest)>,
    measurer: &dyn TextMeasurer,
    stamp: SceneStamp,
) -> ChartResult<Vec<LaidOutChart>> {
    struct Work {
        prepared: Arc<PreparedChart>,
        request: LayoutRequest,
        axes: BTreeMap<ScaleId, ResolvedAxis>,
        guides: BTreeMap<GuideId, ResolvedGuide>,
        labels: BTreeMap<GuideId, Vec<Label>>,
        titles: BTreeMap<GuideId, super::text::Block>,
        plot: Option<Rect>,
        diagnostics: Vec<Diagnostic>,
        passes: usize,
    }
    let mut work = Vec::new();
    let mut m = [0_f64; 4];
    for (prepared, request) in inputs {
        preflight(&prepared, &request)?;
        for side in &mut m {
            *side = side.max(request.padding);
        }
        let specs = guide_specs(&request);
        let titles = specs
            .iter()
            .filter(|a| a.visible)
            .filter_map(|a| a.title.as_ref().map(|t| (a.id, t)))
            .map(|(id, t)| {
                Ok((
                    id,
                    super::text::measure(
                        t,
                        &request,
                        measurer,
                        request
                            .host_theme
                            .foreground
                            .map(crate::color::Paint::resolve)
                            .unwrap_or(INK),
                    )?,
                ))
            })
            .collect::<ChartResult<BTreeMap<_, _>>>()?;
        work.push(Work {
            titles,
            diagnostics: prepared.diagnostics().to_vec(),
            prepared,
            request,
            axes: BTreeMap::new(),
            guides: BTreeMap::new(),
            labels: BTreeMap::new(),
            plot: None,
            passes: 0,
        });
    }
    for pass in 0..MAX_LAYOUT_PASSES {
        let mut next = m;
        for w in &mut work {
            w.plot = plot(&w.request, m)?;
            let Some(p) = w.plot else {
                w.axes.clear();
                w.guides.clear();
                continue;
            };
            let mut major_values = BTreeMap::new();
            w.axes = w
                .request
                .axes
                .iter()
                .map(|s| {
                    Ok((
                        s.id,
                        resolve_axis(
                            &w.prepared,
                            &w.request,
                            s,
                            p,
                            major_values.entry(s.id).or_default(),
                        )?,
                    ))
                })
                .collect::<ChartResult<_>>()?;
            w.guides = resolve_guides(&w.prepared, &w.axes, &major_values, &w.request)?;
            w.labels = measure_guides(&w.guides, &w.request, measurer)?;
            stack_guides(&mut w.guides, &w.labels, &w.titles, &w.request);
            w.passes = pass + 1;
            next = margins(&w.guides, &w.labels, &w.titles, &w.request, next);
        }
        if next == m {
            break;
        }
        if pass + 1 == MAX_LAYOUT_PASSES {
            for w in &mut work {
                w.diagnostics.push(pressure("Margin solver reached its four-pass cap; each guide retains its declared preservation/adaptive and overflow policies."));
            }
            break;
        }
        m = next;
    }
    work.into_iter().map(|mut w| {
        let request = &w.request;
        let (mut output, status) = if let Some(p) = w.plot {
            let mut output = project::project(&w.prepared, &w.axes, p, request)?;
            let has_population = w.prepared.layers().iter().filter(|l| l.visible()).any(|l| match l.table().rows() {
                crate::grammar::PreparedRows::Binned(bins) => bins.iter().any(|b| b.count > 0),
                crate::grammar::PreparedRows::Source(_) | crate::grammar::PreparedRows::Statistical(_) => !l.marks().is_empty(),
            });
            let retained_empty_glyphs = w.prepared.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3 && has_population && output.omitted == 0;
            let status = if !has_population || (output.items.is_empty() && !retained_empty_glyphs) { LayoutStatus::NoData } else { LayoutStatus::Ready };
            if guides(&w.axes, &mut w.guides, &w.labels, p, request, &mut output)? {
                w.diagnostics.push(pressure("Overlapping, duplicate or out-of-figure tick labels were encountered; each guide applied its declared preservation/adaptive policy."));
            }
            for (id,title) in &w.titles {
                let axis=&w.guides[id];
                let (x,y)=match axis.spec.side {
                    AxisSide::Bottom=>(p.origin().x()+(p.width()-title.bounds.width())/2.,request.bounds.max_y()-request.padding-title.bounds.height()),
                    AxisSide::Top=>(p.origin().x()+(p.width()-title.bounds.width())/2.,request.bounds.origin().y()+request.padding),
                    AxisSide::Left=>(request.bounds.origin().x()+request.padding,p.origin().y()+(p.height()-title.bounds.height())/2.),
                    AxisSide::Right=>(request.bounds.max_x()-request.padding-title.bounds.width(),p.origin().y()+(p.height()-title.bounds.height())/2.),
                };
                let (x,y) = if axis.spec.ggplot_axis.is_some() {
                    let extent = guide_extent(axis, &w.labels[id], request) + request.label_gap;
                    match axis.spec.side {
                        AxisSide::Bottom => (p.origin().x()+(p.width()-title.bounds.width())/2., p.max_y()+extent),
                        AxisSide::Top => (p.origin().x()+(p.width()-title.bounds.width())/2., p.origin().y()-extent-title.bounds.height()),
                        AxisSide::Left => (p.origin().x()-extent-title.bounds.width(),p.origin().y()+(p.height()-title.bounds.height())/2.),
                        AxisSide::Right => (p.max_x()+extent,p.origin().y()+(p.height()-title.bounds.height())/2.),
                    }
                } else {(x,y)};
                let (x,y)=(x+axis.spec.translation[0],y+axis.spec.translation[1]);
                if !inside(Rect::new(x,y,title.bounds.width(),title.bounds.height())?,request.bounds){w.diagnostics.push(pressure("Axis title exceeds its panel and is clipped; logical text is retained."));}
                for item in title.items_at(x,y,request.bounds)? {output.push(item,vec![],request)?;}
                w.diagnostics.extend_from_slice(&title.diagnostics);
            }
            (output,status)
        } else {
            w.axes.clear();
            w.guides.clear();
            let mut output = Output {hierarchies:Default::default(),items: vec![],targets:vec![],omitted:0,interactions:Default::default()};
            compact("Not enough space",request,measurer,&mut output)?;
            w.diagnostics.push(pressure("Bounds and destination text metrics cannot accommodate the minimum useful plot."));
            (output,LayoutStatus::NoSpace)
        };
        if status == LayoutStatus::NoData { compact("No data",request,measurer,&mut output)?; }
        if output.omitted > 0 { w.diagnostics.push(pressure(&format!("Explicit scale policies omitted {} marks/vertices; source statistics are unchanged.",output.omitted))); }
        for label in w.labels.values().flatten(){if let Some(block)=&label.rich {for d in &block.diagnostics {if !w.diagnostics.contains(d){w.diagnostics.push(d.clone());}}}}
        let resources=super::text::resources(&output.items,request)?;
        let scene = Scene::new(stamp,request.units,request.bounds,&output.items,&resources,request.limits)?;
        for (id, axis) in &mut w.axes {
            if let Some(guide)=w.guides.get(&GuideId::new(id.get())) {axis.ticks.clone_from(&guide.ticks);}
        }
        let guide_frames = if let Some(plot) = w.plot {super::guide_animation::initial_frames(&w.guides,&w.axes,plot,request)?} else {BTreeMap::new()};
        Ok(LaidOutChart {hierarchies:output.hierarchies,guide_frames,guide_presentation:None,guides:w.guides,paint_themes:BTreeMap::new(),interactions:output.interactions,insets:vec![],prepared:w.prepared,scene,plot:w.plot,axes:w.axes,
            item_panels: vec![None; output.targets.len()], panels: vec![],
            targets:output.targets,diagnostics:w.diagnostics,status,passes:w.passes})
    }).collect()
}
