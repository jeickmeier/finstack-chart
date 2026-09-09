use super::{
    axes::resolve_axis,
    project::{self, Output},
    *,
};
use crate::grammar::{PreparedChart, PreparedGeometry, ValueSpace};
use crate::limits::require_within;
use crate::scales::*;
use crate::scene::{Color, Primitive, Scene, SceneItem, Stroke};
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
    let scales = r
        .axes
        .iter()
        .map(|spec| {
            Ok((
                spec.id,
                resolve_axis(chart, r, spec, Rect::new(0., 0., 1., 1.)?)?,
            ))
        })
        .collect::<ChartResult<BTreeMap<_, _>>>()?;
    for guide in resolve_guides(&scales, r)?.values() {
        if guide.spec.visible {
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
    tick: GuideTick,
    metrics: TextMetrics,
    rich: Option<super::text::Block>,
}
fn measure_guides(
    axes: &BTreeMap<GuideId, ResolvedGuide>,
    r: &LayoutRequest,
    measurer: &dyn TextMeasurer,
) -> ChartResult<BTreeMap<GuideId, Vec<Label>>> {
    // Aggregate per-pass text budget is checked for all candidates before host callbacks.
    let mut remaining = r.limits.max_text_bytes;
    for a in axes.values() {
        for t in &a.ticks {
            require_within(t.label.len() <= remaining, "layout measured text byte")?;
            remaining -= t.label.len();
        }
    }
    let mut labels = BTreeMap::new();
    for (id, a) in axes {
        let mut measured = vec![];
        for t in &a.ticks {
            if t.label.is_empty() {
                measured.push(Label {
                    tick: t.clone(),
                    metrics: TextMetrics::new(0., 0., 0.)?,
                    rich: None,
                });
                continue;
            }
            let rich = if a.spec.typography.is_some() || a.spec.label_rotation != 0. {
                let mut run = a
                    .spec
                    .typography
                    .clone()
                    .unwrap_or_else(|| crate::typography::RichRun::new(""));
                run.text = t.label.clone();
                Some(super::text::measure(
                    &crate::typography::RichText {
                        lines: vec![vec![run]],
                        line_spacing: 1.2,
                        rotation: a.spec.label_rotation,
                    },
                    r,
                    measurer,
                    r.host_theme
                        .foreground
                        .map(crate::color::Paint::resolve)
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
                tick: t.clone(),
                metrics,
                rich,
            });
        }
        labels.insert(*id, measured);
    }
    Ok(labels)
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
            m[side] = m[side]
                .max(r.padding + r.tick_length + r.label_gap + size + title + outward.max(0.));
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
fn label_geometry(
    side: AxisSide,
    l: &Label,
    p: Rect,
    r: &LayoutRequest,
) -> ChartResult<(Point, Rect, Point, Point)> {
    let v = l.tick.position;
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
    for (id, a) in axes {
        if !a.spec.visible {
            continue;
        }
        let p = Rect::new(
            p.origin().x() + a.spec.translation[0],
            p.origin().y() + a.spec.translation[1],
            p.width(),
            p.height(),
        )?;
        let (from, to) = match a.spec.side {
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
        out.push(
            SceneItem {
                layer: None,
                clip: None,
                primitive: Primitive::Rule {
                    from,
                    to,
                    stroke: Stroke {
                        color: r
                            .host_theme
                            .foreground
                            .map(crate::color::Paint::resolve)
                            .unwrap_or(INK),
                        width: 1.,
                    },
                },
            },
            vec![],
            r,
        )?;
        a.ticks.clear();
        let mut seen = BTreeSet::new();
        for l in &labels[id] {
            let (origin, bounds, from, to) = label_geometry(a.spec.side, l, p, r)?;
            let along = if a.spec.side.horizontal() {
                l.tick.position >= p.origin().x() && l.tick.position <= p.max_x()
            } else {
                l.tick.position >= p.origin().y() && l.tick.position <= p.max_y()
            };
            if !along
                || !l.tick.label.is_empty()
                    && (!inside(bounds, r.bounds)
                        || placed.iter().any(|b| overlaps(bounds, *b, r.label_gap))
                        || !seen.insert(l.tick.label.clone()))
            {
                pressure = true;
                continue;
            }
            if !l.tick.label.is_empty() {
                placed.push(bounds);
            }
            a.ticks.push(l.tick.clone());
            out.push(
                SceneItem {
                    layer: None,
                    clip: None,
                    primitive: Primitive::Rule {
                        from,
                        to,
                        stroke: Stroke {
                            color: r
                                .host_theme
                                .foreground
                                .map(crate::color::Paint::resolve)
                                .unwrap_or(INK),
                            width: 1.,
                        },
                    },
                },
                vec![],
                r,
            )?;
            if l.tick.label.is_empty() {
                continue;
            }
            if let Some(block) = &l.rich {
                for item in block.items_at(bounds.origin().x(), bounds.origin().y(), r.bounds)? {
                    out.push(item, vec![], r)?;
                }
            } else {
                out.push(
                    SceneItem {
                        layer: None,
                        clip: None,
                        primitive: Primitive::Text {
                            origin,
                            text: l.tick.label.clone(),
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
    axes: &BTreeMap<ScaleId, ResolvedAxis>,
    r: &LayoutRequest,
) -> ChartResult<BTreeMap<GuideId, ResolvedGuide>> {
    let mut guides = BTreeMap::new();
    for axis in axes.values() {
        let spec = axis.spec.default_guide();
        guides.insert(
            spec.id,
            ResolvedGuide {
                spec,
                ticks: axis.ticks.clone(),
            },
        );
    }
    for spec in &r.guides {
        let axis = axes
            .get(&spec.scale)
            .ok_or_else(|| error(DiagnosticCode::MissingResource, "Guide scale is absent."))?;
        let mut ticks = super::guide_ticks::resolve(axis, &spec.style, r)?;
        let offset = spec.translation[usize::from(!spec.side.horizontal())];
        for tick in &mut ticks {
            tick.position += offset;
            if !tick.position.is_finite() {
                return Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Guide translation exceeds finite positions.",
                ));
            }
        }
        guides.insert(
            spec.id,
            ResolvedGuide {
                spec: spec.clone(),
                ticks,
            },
        );
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
            w.axes = w
                .request
                .axes
                .iter()
                .map(|s| Ok((s.id, resolve_axis(&w.prepared, &w.request, s, p)?)))
                .collect::<ChartResult<_>>()?;
            w.guides = resolve_guides(&w.axes, &w.request)?;
            w.labels = measure_guides(&w.guides, &w.request, measurer)?;
            w.passes = pass + 1;
            next = margins(&w.guides, &w.labels, &w.titles, &w.request, next);
        }
        if next == m {
            break;
        }
        if pass + 1 == MAX_LAYOUT_PASSES {
            for w in &mut work {
                w.diagnostics.push(pressure("Margin solver reached its four-pass cap; labels are deterministically thinned to fit."));
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
            let status = if output.items.is_empty() || !has_population { LayoutStatus::NoData } else { LayoutStatus::Ready };
            if guides(&w.axes, &mut w.guides, &w.labels, p, request, &mut output)? {
                w.diagnostics.push(pressure("Overlapping, duplicate or out-of-figure tick labels were deterministically thinned."));
            }
            for (id,title) in &w.titles {
                let axis=&w.guides[id];
                let (x,y)=match axis.spec.side {
                    AxisSide::Bottom=>(p.origin().x()+(p.width()-title.bounds.width())/2.,request.bounds.max_y()-request.padding-title.bounds.height()),
                    AxisSide::Top=>(p.origin().x()+(p.width()-title.bounds.width())/2.,request.bounds.origin().y()+request.padding),
                    AxisSide::Left=>(request.bounds.origin().x()+request.padding,p.origin().y()+(p.height()-title.bounds.height())/2.),
                    AxisSide::Right=>(request.bounds.max_x()-request.padding-title.bounds.width(),p.origin().y()+(p.height()-title.bounds.height())/2.),
                };
                let (x,y)=(x+axis.spec.translation[0],y+axis.spec.translation[1]);
                if !inside(Rect::new(x,y,title.bounds.width(),title.bounds.height())?,request.bounds){w.diagnostics.push(pressure("Axis title exceeds its panel and is clipped; logical text is retained."));}
                for item in title.items_at(x,y,request.bounds)? {output.push(item,vec![],request)?;}
                w.diagnostics.extend_from_slice(&title.diagnostics);
            }
            (output,status)
        } else {
            w.axes.clear();
            w.guides.clear();
            let mut output = Output {items: vec![],targets:vec![],omitted:0,interactions:Default::default()};
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
        Ok(LaidOutChart {guides:w.guides,paint_themes:BTreeMap::new(),interactions:output.interactions,insets:vec![],prepared:w.prepared,scene,plot:w.plot,axes:w.axes,
            item_panels: vec![None; output.targets.len()], panels: vec![],
            targets:output.targets,diagnostics:w.diagnostics,status,passes:w.passes})
    }).collect()
}
