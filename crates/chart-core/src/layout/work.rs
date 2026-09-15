//! One figure-wide projection budget, checked before destination callbacks.
use super::LayoutRequest;
use crate::grammar::{PreparedChart, PreparedGeometry};
use crate::{ChartResult, DiagnosticCode, LayerId};

pub(super) fn vertices(geometry: &PreparedGeometry) -> usize {
    match geometry {
        PreparedGeometry::Point(_) | PreparedGeometry::UnboundedPoint(_) => 1,
        PreparedGeometry::LineRun(points) | PreparedGeometry::Polygon(points) => points.len(),
        PreparedGeometry::BandRun { lower, upper }
        | PreparedGeometry::StackBandRun { lower, upper, .. } => {
            lower.len().saturating_add(upper.len())
        }
        PreparedGeometry::ShapePath { geometry, .. } => geometry.commands().len().saturating_add(1),
        PreparedGeometry::ShapePathRun {
            geometry, anchors, ..
        } => geometry.commands().len().saturating_add(anchors.len()),
        _ => 2,
    }
}
fn add(a: usize, b: usize) -> ChartResult<usize> {
    a.checked_add(b).ok_or_else(|| {
        crate::scales::error(
            DiagnosticCode::ResourceLimit,
            "Figure projection work overflows.",
        )
    })
}
fn cost(chart: &PreparedChart, selected: Option<&[LayerId]>) -> ChartResult<usize> {
    if !chart.panels().is_empty() {
        return chart
            .panels()
            .iter()
            .try_fold(0, |total, panel| add(total, cost(&panel.chart, selected)?));
    }
    chart
        .layers()
        .iter()
        .filter(|layer| layer.visible() && selected.is_none_or(|ids| ids.contains(&layer.id())))
        .flat_map(|layer| layer.marks())
        .try_fold(0, |total, mark| add(total, vertices(&mark.geometry)))
}
pub(super) fn preflight(chart: &PreparedChart, request: &LayoutRequest) -> ChartResult<()> {
    let mut total = cost(chart, None)?;
    let mut commands = custom_commands(chart, None)?;
    if let Some(figure) = chart.state().figure(chart.definition()) {
        figure.validate(request.limits)?;
        for inset in &figure.insets {
            let parent = match &inset.panel {
                Some(key) => chart
                    .panels()
                    .iter()
                    .find(|p| &p.key == key)
                    .map(|p| p.chart.as_ref())
                    .ok_or_else(|| {
                        crate::scales::error(
                            DiagnosticCode::Validation,
                            "Inset names an absent prepared panel.",
                        )
                    })?,
                None if chart.panels().is_empty() => chart,
                None => {
                    return Err(crate::scales::error(
                        DiagnosticCode::Validation,
                        "A faceted inset must select a parent panel.",
                    ));
                }
            };
            if inset
                .layers
                .iter()
                .any(|id| !parent.layers().iter().any(|l| l.id() == *id))
            {
                return Err(crate::scales::error(
                    DiagnosticCode::Validation,
                    "Inset names a layer absent from its parent panel.",
                ));
            }
            total = add(total, cost(parent, Some(&inset.layers))?)?;
            commands = add(commands, custom_commands(parent, Some(&inset.layers))?)?;
        }
    }
    crate::limits::require_within(
        commands <= request.limits.max_path_commands,
        "registered figure plus inset curve command",
    )?;
    crate::limits::require_within(
        total <= request.max_vertices,
        "figure plus inset projected vertex",
    )
}

// Registered Cartesian curves execute after scale layout. Their declared bound is
// checked for the whole figure and all insets before text/destination callbacks.
fn custom_commands(chart: &PreparedChart, selected: Option<&[LayerId]>) -> ChartResult<usize> {
    if !chart.panels().is_empty() {
        return chart.panels().iter().try_fold(0, |total, panel| {
            add(total, custom_commands(&panel.chart, selected)?)
        });
    }
    let mut total = 0;
    for layer in chart
        .layers()
        .iter()
        .filter(|l| l.visible() && selected.is_none_or(|ids| ids.contains(&l.id())))
    {
        let Some(curve) = layer.shape_protocols.curve() else {
            continue;
        };
        let Some(definition) = chart
            .definition()
            .layers
            .iter()
            .find(|d| d.id == layer.id())
        else {
            continue;
        };
        if !matches!(
            definition.geom,
            crate::grammar::Geom::ShapeLine { .. }
                | crate::grammar::Geom::ShapeArea { .. }
                | crate::grammar::Geom::ShapeLink { .. }
        ) {
            continue;
        }
        for mark in layer.marks() {
            let bound = curve
                .command_bound(vertices(&mark.geometry))
                .ok_or_else(|| {
                    crate::scales::error(
                        DiagnosticCode::ResourceLimit,
                        "Registered curve command bound overflows or is unavailable.",
                    )
                })?;
            total = add(total, bound)?;
        }
    }
    Ok(total)
}
