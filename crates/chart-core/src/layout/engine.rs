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
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect, ScaleId, SceneStamp, Severity};
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

fn pressure(message: &str) -> Diagnostic {
    let mut d = error(DiagnosticCode::LayoutPressure, message);
    d.severity = Severity::Warning;
    d
}
fn text_request<'a>(r: &'a LayoutRequest, text: &'a str) -> TextRequest<'a> {
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
    let mut ids = BTreeSet::new();
    let mut sides = BTreeSet::new();
    for a in &r.axes {
        if !ids.insert(a.id) || (a.visible && !sides.insert(a.side)) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Axes need unique IDs and one visible guide per side.",
            ));
        }
        if let Some(v) = a.viewport {
            v.distinct()?;
        }
        if let Some(v) = a.range {
            v.distinct()?;
        }
        if let AxisScale::Band(o) = &a.scale
            && let Some(d) = &o.domain
        {
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
    let mut vertices = r.max_vertices;
    let mut paths = r.limits.max_path_commands;
    let mut items = r.limits.max_items;
    for l in chart.layers().iter().filter(|l| l.visible()) {
        for m in l.marks() {
            let n = match &m.geometry {
                PreparedGeometry::Point(_) => 1,
                PreparedGeometry::LineRun(p) => p.len(),
                _ => 2,
            };
            require_within(n <= vertices, "layout vertex")?;
            vertices -= n;
            // Omission can split a line into at most one singleton per input vertex.
            let count = if matches!(m.geometry, PreparedGeometry::LineRun(_)) {
                n
            } else {
                1
            };
            require_within(count <= items, "layout potential item")?;
            items -= count;
            if matches!(m.geometry, PreparedGeometry::LineRun(_)) {
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
                crate::grammar::PreparedRows::Source(_) => !l.marks().is_empty(),
            });
    if no_population {
        potential_items = potential_items.checked_add(1).ok_or_else(|| {
            error(
                DiagnosticCode::ResourceLimit,
                "Compact scene item count overflow.",
            )
        })?;
    }
    for spec in &r.axes {
        let axis = resolve_axis(chart, r, spec, Rect::new(0., 0., 1., 1.)?)?;
        if spec.visible {
            potential_items = potential_items
                .checked_add(1 + axis.ticks.len() * 2)
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
}
fn measure_axes(
    axes: &BTreeMap<ScaleId, ResolvedAxis>,
    r: &LayoutRequest,
    measurer: &dyn TextMeasurer,
) -> ChartResult<BTreeMap<ScaleId, Vec<Label>>> {
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
            measured.push(Label {
                tick: t.clone(),
                metrics: measure_text(measurer, text_request(r, &t.label), r.limits)?,
            });
        }
        labels.insert(*id, measured);
    }
    Ok(labels)
}
// [left, right, top, bottom]. Monotonic margins avoid tick-count oscillation.
fn margins(
    axes: &BTreeMap<ScaleId, ResolvedAxis>,
    labels: &BTreeMap<ScaleId, Vec<Label>>,
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
            m[side] = m[side].max(r.padding + r.tick_length + r.label_gap + size);
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
    axes: &mut BTreeMap<ScaleId, ResolvedAxis>,
    labels: &BTreeMap<ScaleId, Vec<Label>>,
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
                        color: INK,
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
                || !inside(bounds, r.bounds)
                || placed.iter().any(|b| overlaps(bounds, *b, r.label_gap))
                || !seen.insert(l.tick.label.clone())
            {
                pressure = true;
                continue;
            }
            placed.push(bounds);
            a.ticks.push(l.tick.clone());
            out.push(
                SceneItem {
                    layer: None,
                    clip: None,
                    primitive: Primitive::Rule {
                        from,
                        to,
                        stroke: Stroke {
                            color: INK,
                            width: 1.,
                        },
                    },
                },
                vec![],
                r,
            )?;
            out.push(
                SceneItem {
                    layer: None,
                    clip: None,
                    primitive: Primitive::Text {
                        origin,
                        text: l.tick.label.clone(),
                        font: r.font.id,
                        font_size: r.font_size,
                        color: INK,
                    },
                },
                vec![],
                r,
            )?;
        }
        if matches!(&a.scale,ResolvedScale::Band(s) if s.domain().len()>labels[id].len()) {
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
                    color: INK,
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
    let stamp = SceneStamp {
        definition: prepared.definition_revision(),
        store: prepared.source().get()?.revision(),
        layout: request.revision,
        state: prepared.state().revision(),
        viewport: prepared.state().viewport_revision(),
    };
    layout_inner(prepared, request, measurer, stamp).map_err(|mut e| {
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
    preflight(&prepared, request)?;
    let mut m = [request.padding; 4];
    let mut axes = BTreeMap::new();
    let mut labels = BTreeMap::new();
    let mut diagnostics = prepared.diagnostics().to_vec();
    let mut passes = 0;
    let mut final_plot = None;
    for pass in 0..MAX_LAYOUT_PASSES {
        let Some(p) = plot(request, m)? else {
            break;
        };
        axes = request
            .axes
            .iter()
            .map(|s| Ok((s.id, resolve_axis(&prepared, request, s, p)?)))
            .collect::<ChartResult<_>>()?;
        labels = measure_axes(&axes, request, measurer)?;
        passes = pass + 1;
        let next = margins(&axes, &labels, request, m);
        final_plot = Some(p);
        if next == m {
            break;
        }
        if pass + 1 == MAX_LAYOUT_PASSES {
            diagnostics.push(pressure("Margin solver reached its four-pass cap; labels are deterministically thinned to fit."));
            break;
        }
        m = next;
        final_plot = None;
    }
    let (mut output, status) = if let Some(p) = final_plot {
        let mut output = project::project(&prepared, &axes, p, request)?;
        let has_population =
            prepared
                .layers()
                .iter()
                .filter(|l| l.visible())
                .any(|l| match l.table().rows() {
                    crate::grammar::PreparedRows::Binned(bins) => bins.iter().any(|b| b.count > 0),
                    crate::grammar::PreparedRows::Source(_) => !l.marks().is_empty(),
                });
        let status = if output.items.is_empty() || !has_population {
            LayoutStatus::NoData
        } else {
            LayoutStatus::Ready
        };
        if guides(&mut axes, &labels, p, request, &mut output)? {
            diagnostics.push(pressure("Overlapping, duplicate or out-of-figure tick labels were deterministically thinned."));
        }
        (output, status)
    } else {
        axes.clear();
        let mut output = Output {
            items: vec![],
            targets: vec![],
            omitted: 0,
        };
        compact("Not enough space", request, measurer, &mut output)?;
        diagnostics.push(pressure(
            "Bounds and destination text metrics cannot accommodate the minimum useful plot.",
        ));
        (output, LayoutStatus::NoSpace)
    };
    if status == LayoutStatus::NoData {
        compact("No data", request, measurer, &mut output)?;
    }
    if output.omitted > 0 {
        diagnostics.push(pressure(&format!(
            "Explicit scale policies omitted {} marks/vertices; source statistics are unchanged.",
            output.omitted
        )));
    }
    let scene = Scene::new(
        stamp,
        request.units,
        request.bounds,
        &output.items,
        &[request.font],
        request.limits,
    )?;
    Ok(LaidOutChart {
        prepared,
        scene,
        plot: final_plot,
        axes,
        targets: output.targets,
        diagnostics,
        status,
        passes,
    })
}
