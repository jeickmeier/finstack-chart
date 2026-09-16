//! Panel-dependent hierarchy layouts with explicitly retained immutable history.
use super::*;
use crate::scales::error;
use crate::{
    ChartResult, LayerId, Point, Rect,
    grammar::{
        HierarchyProjection, HierarchyRadius, PreparedGeometry, PreparedHierarchy, PreparedLayer,
    },
    hierarchy::{HierarchyLayout, LayoutSpec, NodeGeometry, NodeRecord, TreeSize, TreemapHistory},
    path::{Affine, Path, PathGeometry},
    scene::{Primitive, SceneItem, Stroke},
    shape::{Arc as ArcShape, ArcDatum, Link, LinkDatum, LinkRadial, ShapeLimits},
};
use std::{collections::BTreeMap, sync::Arc};
/// One captured resquarify cache, isolated by panel/inset/layer scope.
#[derive(Clone, Debug, serde::Serialize)]
pub struct HierarchyHistoryEntry {
    /// Existing nested destination scope convention.
    pub scope: Vec<GuideScope>,
    /// Layer identity in that destination.
    pub layer: LayerId,
    /// Immutable-by-ownership row cache.
    pub history: TreemapHistory,
}
/// Exact topology, numeric layout and rows retained with a destination scene.
#[derive(Clone, Debug)]
pub struct ResolvedHierarchy {
    pub(crate) prepared: Arc<PreparedHierarchy>,
    pub(crate) layout: HierarchyLayout,
    pub(crate) history: TreemapHistory,
    pub(crate) bounds: Rect,
}
impl ResolvedHierarchy {
    /// Exact immutable numeric layout before recipe projection.
    pub fn layout(&self) -> &HierarchyLayout {
        &self.layout
    }
    /// Original preparation, including shared source membership.
    pub fn prepared(&self) -> &Arc<PreparedHierarchy> {
        &self.prepared
    }
    /// Rows captured after this destination's layout.
    pub fn history(&self) -> &TreemapHistory {
        &self.history
    }
    /// Allocated destination rectangle.
    pub fn bounds(&self) -> Rect {
        self.bounds
    }
}
/// Owned portable chart hierarchy metadata; source membership is serialized once.
#[derive(Clone, Debug, serde::Serialize)]
pub struct HierarchySnapshot {
    /// Nested facet/inset scope.
    pub scope: Vec<GuideScope>,
    /// Exact layer identity.
    pub layer: LayerId,
    /// Authored source selectors and layout controls.
    pub recipe: crate::grammar::HierarchyRecipe,
    /// Exact source revision.
    pub input: crate::data::DatasetVersion,
    /// Destination bounds used for extent fitting and radial projection.
    pub bounds: Rect,
    /// One shared preorder source-key table.
    pub source_keys: Arc<[crate::RowKey]>,
    /// Breadth-first node payloads, values and layout-space geometry.
    pub nodes: Vec<NodeRecord>,
    /// Corresponding stable targets/ranges in the same node order.
    pub targets: Vec<crate::provenance::Target>,
    /// Retained resquarify row count.
    pub history_rows: usize,
    /// Retained child membership count.
    pub history_members: usize,
}
impl LaidOutChart {
    /// Hierarchy layouts local to this panel; nested destinations own separate entries.
    pub fn hierarchies(&self) -> &BTreeMap<LayerId, ResolvedHierarchy> {
        &self.hierarchies
    }
    /// Capture history across all current destinations without retaining previous scenes.
    pub fn hierarchy_history(&self) -> Vec<HierarchyHistoryEntry> {
        fn visit(
            chart: &LaidOutChart,
            scope: &mut Vec<GuideScope>,
            out: &mut Vec<HierarchyHistoryEntry>,
        ) {
            out.extend(
                chart
                    .hierarchies
                    .iter()
                    .map(|(layer, h)| HierarchyHistoryEntry {
                        scope: scope.clone(),
                        layer: *layer,
                        history: h.history.clone(),
                    }),
            );
            for p in &chart.panels {
                scope.push(GuideScope::Panel(p.key.clone()));
                visit(&p.chart, scope, out);
                scope.pop();
            }
            for p in &chart.insets {
                if let Some(panel) = &p.panel {
                    scope.push(GuideScope::Panel(panel.clone()));
                }
                scope.push(GuideScope::Inset(p.id.clone()));
                visit(&p.chart, scope, out);
                scope.pop();
                if p.panel.is_some() {
                    scope.pop();
                }
            }
        }
        let mut out = vec![];
        visit(self, &mut vec![], &mut out);
        out
    }
    /// Exact owned hierarchy metadata, including compact shared membership once per scope.
    pub fn hierarchy_snapshots(&self) -> ChartResult<Vec<HierarchySnapshot>> {
        fn visit(
            chart: &LaidOutChart,
            scope: &mut Vec<GuideScope>,
            out: &mut Vec<HierarchySnapshot>,
        ) -> ChartResult<()> {
            for (layer, h) in &chart.hierarchies {
                let input = chart
                    .prepared
                    .layers()
                    .iter()
                    .find(|l| l.id() == *layer)
                    .expect("resolved prepared layer")
                    .table()
                    .input();
                let nodes = h
                    .layout
                    .nodes()?
                    .map(|(n, _)| NodeRecord::from_node(n, Some(&h.layout)))
                    .collect::<ChartResult<Vec<_>>>()?;
                let targets = nodes
                    .iter()
                    .map(|n| h.prepared.target(n.handle).cloned())
                    .collect::<ChartResult<_>>()?;
                out.push(HierarchySnapshot {
                    scope: scope.clone(),
                    layer: *layer,
                    recipe: h.prepared.recipe().clone(),
                    input,
                    bounds: h.bounds,
                    source_keys: h.prepared.source_keys().clone(),
                    nodes,
                    targets,
                    history_rows: h.history.row_count(),
                    history_members: h.history.membership_count(),
                });
            }
            for p in &chart.panels {
                scope.push(GuideScope::Panel(p.key.clone()));
                visit(&p.chart, scope, out)?;
                scope.pop();
            }
            for p in &chart.insets {
                if let Some(panel) = &p.panel {
                    scope.push(GuideScope::Panel(panel.clone()));
                }
                scope.push(GuideScope::Inset(p.id.clone()));
                visit(&p.chart, scope, out)?;
                scope.pop();
                if p.panel.is_some() {
                    scope.pop();
                }
            }
            Ok(())
        }
        let mut out = vec![];
        visit(self, &mut vec![], &mut out)?;
        Ok(out)
    }
}
impl LayoutRequest {
    /// Seed compatible layout rows from an exact previous scene. No old scene is retained.
    pub fn with_hierarchy_history(mut self, previous: &LaidOutChart) -> Self {
        self.hierarchy_history = Arc::new(previous.hierarchy_history());
        self
    }
}
fn fitted(spec: &LayoutSpec, projection: HierarchyProjection, plot: Rect) -> LayoutSpec {
    let mut result = spec.clone();
    let size = match projection {
        HierarchyProjection::Horizontal => [plot.height(), plot.width()],
        HierarchyProjection::Radial => {
            [std::f64::consts::TAU, plot.width().min(plot.height()) / 2.]
        }
        HierarchyProjection::Sunburst { .. } => [std::f64::consts::TAU, 1.],
        _ => [plot.width(), plot.height()],
    };
    match &mut result {
        LayoutSpec::Tree { options, .. } | LayoutSpec::Cluster { options, .. } => {
            if matches!(options.mode, TreeSize::Extent(_)) {
                options.mode = TreeSize::Extent(size);
            }
        }
        LayoutSpec::Partition(options) => options.size = size,
        LayoutSpec::Treemap { options, .. } => options.size = size,
        LayoutSpec::Pack { options, .. } => options.size = size,
    }
    result
}
fn translate(
    geometry: PathGeometry,
    x: f64,
    y: f64,
    request: &LayoutRequest,
) -> ChartResult<PathGeometry> {
    geometry.transformed(
        Affine::new([1., 0., 0., 1., x, y])?,
        0.01,
        request.limits.max_path_commands,
    )
}
fn point(
    layout: &HierarchyLayout,
    handle: crate::hierarchy::NodeHandle,
    projection: HierarchyProjection,
    plot: Rect,
    fixed: bool,
) -> ChartResult<Point> {
    let NodeGeometry::Point { x, y } = layout.geometry(handle)? else {
        return Err(error(
            crate::DiagnosticCode::Validation,
            "Hierarchy edge requires point-layout endpoints.",
        ));
    };
    let x = if fixed && projection != HierarchyProjection::Radial {
        x + if projection == HierarchyProjection::Horizontal {
            plot.height() / 2.
        } else {
            plot.width() / 2.
        }
    } else {
        x
    };
    let (x, y) = match projection {
        HierarchyProjection::Radial => {
            let [x, y] = crate::shape::point_radial(x, y)?;
            (
                plot.origin().x() + plot.width() / 2. + x,
                plot.origin().y() + plot.height() / 2. + y,
            )
        }
        HierarchyProjection::Horizontal => (plot.origin().x() + y, plot.origin().y() + x),
        _ => (plot.origin().x() + x, plot.origin().y() + y),
    };
    Point::new(x, y)
}
fn circle(center: Point, r: f64, limits: ShapeLimits) -> ChartResult<PathGeometry> {
    ArcShape::new()
        .limits(limits)
        .generate(ArcDatum {
            inner_radius: 0.,
            outer_radius: r,
            start_angle: 0.,
            end_angle: std::f64::consts::TAU,
            pad_angle: 0.,
        })?
        .geometry()
        .transformed(
            Affine::new([1., 0., 0., 1., center.x(), center.y()])?,
            0.01,
            limits.path.max_commands,
        )
}
pub(super) fn project(
    layer: &PreparedLayer,
    plot: Rect,
    request: &LayoutRequest,
    out: &mut super::project::Output,
) -> ChartResult<()> {
    let Some(prepared) = layer.hierarchy() else {
        return Ok(());
    };
    let recipe = prepared.recipe();
    let spec = fitted(&recipe.layout, recipe.projection, plot);
    let mut history = request
        .hierarchy_history
        .iter()
        .find(|h| h.scope == request.hierarchy_scope && h.layer == layer.id())
        .map(|h| h.history.clone())
        .unwrap_or_default();
    if !matches!(spec, LayoutSpec::Treemap { history: true, .. }) {
        history.reset();
    }
    let layout = spec.layout(prepared.hierarchy(), &prepared.registry, &mut history)?;
    let clip = Some(if layer.clip() == crate::grammar::ClipPolicy::Plot {
        plot
    } else {
        request.figure_bounds.unwrap_or(request.bounds)
    });
    let limits = ShapeLimits {
        max_points: request.max_vertices,
        path: crate::path::PathLimits {
            max_commands: request.limits.max_path_commands,
            ..Default::default()
        },
    };
    let fixed = matches!(&spec, LayoutSpec::Tree { options, .. } | LayoutSpec::Cluster { options, .. } if matches!(options.mode, TreeSize::NodeSize(_)));
    let mut commands = 0usize;
    for mark in layer.marks() {
        let (geometry, anchor, fill) = match &mark.geometry {
            PreparedGeometry::HierarchyLink { parent, child } => {
                let a = point(&layout, *parent, recipe.projection, plot, fixed)?;
                let b = point(&layout, *child, recipe.projection, plot, fixed)?;
                let geometry = if recipe.projection == HierarchyProjection::Radial {
                    let (
                        NodeGeometry::Point { x: ax, y: ay },
                        NodeGeometry::Point { x: bx, y: by },
                    ) = (layout.geometry(*parent)?, layout.geometry(*child)?)
                    else {
                        unreachable!()
                    };
                    let path = LinkRadial::new().limits(limits).generate(&LinkDatum {
                        source: vec![ax, ay],
                        target: vec![bx, by],
                    })?;
                    translate(
                        path.geometry(),
                        plot.origin().x() + plot.width() / 2.,
                        plot.origin().y() + plot.height() / 2.,
                        request,
                    )?
                } else {
                    let link = if recipe.projection == HierarchyProjection::Horizontal {
                        Link::horizontal()
                    } else {
                        Link::vertical()
                    };
                    link.limits(limits)
                        .generate(&LinkDatum {
                            source: vec![a.x(), a.y()],
                            target: vec![b.x(), b.y()],
                        })?
                        .geometry()
                };
                (
                    geometry,
                    Point::new(a.x().midpoint(b.x()), a.y().midpoint(b.y()))?,
                    false,
                )
            }
            PreparedGeometry::HierarchyNode(node) => match layout.geometry(*node)? {
                NodeGeometry::Point { .. } => {
                    let p = point(&layout, *node, recipe.projection, plot, fixed)?;
                    (circle(p, mark.style.radius, limits)?, p, true)
                }
                NodeGeometry::Circle { x, y, r } => {
                    if r == 0. {
                        continue;
                    }
                    let p = Point::new(plot.origin().x() + x, plot.origin().y() + y)?;
                    (circle(p, r, limits)?, p, true)
                }
                NodeGeometry::Rectangle { x0, y0, x1, y1 } => {
                    if x0 == x1 || y0 == y1 {
                        continue;
                    }
                    if let HierarchyProjection::Sunburst {
                        inner_radius,
                        radius,
                    } = recipe.projection
                    {
                        let outer = plot.width().min(plot.height()) / 2.;
                        if inner_radius >= outer {
                            continue;
                        }
                        let radius_at = |t: f64| match radius {
                            HierarchyRadius::Linear => inner_radius + (outer - inner_radius) * t,
                            HierarchyRadius::Area => (inner_radius * inner_radius
                                + (outer * outer - inner_radius * inner_radius) * t)
                                .sqrt(),
                        };
                        let datum = ArcDatum {
                            inner_radius: radius_at(y0),
                            outer_radius: radius_at(y1),
                            start_angle: x0,
                            end_angle: x1,
                            pad_angle: 0.,
                        };
                        let arc = ArcShape::new().limits(limits);
                        let anchor = arc.centroid(datum)?;
                        let (cx, cy) = (
                            plot.origin().x() + plot.width() / 2.,
                            plot.origin().y() + plot.height() / 2.,
                        );
                        (
                            translate(arc.generate(datum)?.geometry(), cx, cy, request)?,
                            Point::new(cx + anchor[0], cy + anchor[1])?,
                            true,
                        )
                    } else {
                        let (x0, y0, x1, y1) =
                            if recipe.projection == HierarchyProjection::Horizontal {
                                (y0, x0, y1, x1)
                            } else {
                                (x0, y0, x1, y1)
                            };
                        let mut path = Path::new();
                        path.rect(
                            plot.origin().x() + x0,
                            plot.origin().y() + y0,
                            x1 - x0,
                            y1 - y0,
                        )?;
                        (
                            path.geometry(),
                            Point::new(
                                plot.origin().x() + x0.midpoint(x1),
                                plot.origin().y() + y0.midpoint(y1),
                            )?,
                            true,
                        )
                    }
                }
            },
            _ => {
                return Err(error(
                    crate::DiagnosticCode::Validation,
                    "Hierarchy layer contains unrelated prepared geometry.",
                ));
            }
        };
        commands = commands.saturating_add(geometry.commands().len());
        crate::limits::require_within(
            commands <= request.limits.max_path_commands,
            "hierarchy path command",
        )?;
        if !geometry.has_segments() {
            continue;
        }
        out.push(
            SceneItem {
                guide: None,
                layer: Some(layer.id()),
                clip,
                primitive: Primitive::ShapePath {
                    fill_rule: crate::scene::FillRule::NonZero,
                    geometry,
                    fill: fill.then_some(mark.style.color),
                    stroke: (!fill).then_some(Stroke {
                        color: mark.style.color,
                        width: mark.style.stroke_width,
                    }),
                    dashes: vec![],
                    anchors: vec![anchor],
                },
            },
            mark.targets.clone(),
            request,
        )?;
    }
    out.hierarchies.insert(
        layer.id(),
        ResolvedHierarchy {
            prepared: prepared.clone(),
            layout,
            history,
            bounds: plot,
        },
    );
    Ok(())
}
