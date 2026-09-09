//! Tidy adaptation only: all rank, baseline and normalization arithmetic lives in shape::Stack.
use super::{
    compiler::{EncodedRow, charge, include_geometry, run_style},
    *,
};
use crate::{
    ChartResult, DiagnosticCode, Point,
    shape::{Stack, StackLimits, StackSeries},
};
use std::{collections::BTreeMap, sync::Arc};

pub(super) struct StackLayout {
    groups: Vec<GroupValue>,
    samples: Vec<(f64, f64)>,
    series: Vec<StackSeries<usize>>,
    sources: Vec<Vec<Option<usize>>>,
}
fn kernel(s: &ShapeStackSpec, limits: CompileLimits) -> Stack {
    Stack::new()
        .keys((0..s.groups.len()).map(|i| i.to_string()).collect())
        .order(s.order.clone())
        .offset(s.offset)
        .missing(s.missing)
        .limits(StackLimits {
            max_series: limits.max_groups.min(4096),
            max_cells: limits.max_prepared_rows.min(1_000_000),
            ..StackLimits::default()
        })
}
pub(super) fn validate(
    layer: &Layer,
    domains: &DomainContributions,
    s: &ShapeStackSpec,
    limits: CompileLimits,
) -> ChartResult<()> {
    if !matches!(
        layer.geom,
        Geom::Bar { .. } | Geom::Rectangle | Geom::ShapeArea { .. }
    ) || !domains
        .y_space
        .as_ref()
        .is_none_or(super::positions::additive_space)
        || layer.geometry_extension.is_some()
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Shape stacks require bars, rectangles or shape areas with zero-preserving numeric heights.",
        ));
    }
    if matches!(
        layer.geom,
        Geom::ShapeArea {
            connect_gaps: true,
            ..
        }
    ) {
        return Err(error(
            DiagnosticCode::Validation,
            "Shape stack missing-cell policy controls gaps; connect_gaps must be false.",
        ));
    }
    kernel(s, limits).validate()
}
fn key(x: f64) -> u64 {
    if x == 0. { 0 } else { x.to_bits() }
}
pub(super) fn apply(
    layer: &Layer,
    rows: &mut [EncodedRow],
    s: &ShapeStackSpec,
    limits: CompileLimits,
    shapes: &super::shape_extensions::ResolvedShapes,
) -> ChartResult<StackLayout> {
    let groups: BTreeMap<_, _> = s.groups.iter().enumerate().map(|(i, g)| (g, i)).collect();
    let mut cells = BTreeMap::<(u64, u64), Vec<Option<usize>>>::new();
    let max_cells = limits.max_prepared_rows.min(1_000_000);
    for (i, r) in rows.iter().enumerate() {
        let Some(g) = &r.group else { continue };
        let Some(&group) = groups.get(g) else {
            return Err(error(
                DiagnosticCode::Validation,
                "Shape stack group is absent from its declared catalog.",
            ));
        };
        let (Some(x), Some(x2)) = (r.x, r.x2) else {
            continue;
        };
        if matches!(layer.geom, Geom::ShapeArea { .. }) && x != x2 {
            return Err(error(
                DiagnosticCode::Validation,
                "Shape stack area boundaries must address the same x sample.",
            ));
        }
        if r.y2 != Some(0.) {
            return Err(error(
                DiagnosticCode::Validation,
                "Shape stack input requires an explicit zero baseline; y is signed height.",
            ));
        }
        let k = (key(x), key(x2));
        if !cells.contains_key(&k) {
            if cells
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(s.groups.len()))
                .is_none_or(|n| n > max_cells)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Tidy stack matrix exceeds the prepared-cell budget.",
                ));
            }
            cells.insert(k, vec![None; s.groups.len()]);
        }
        let cell = &mut cells.get_mut(&k).expect("inserted cell")[group];
        if cell.replace(i).is_some() {
            return Err(error(
                DiagnosticCode::Validation,
                "Shape stack requires one row per group/sample; aggregate duplicates explicitly.",
            ));
        }
    }
    let mut cells: Vec<_> = cells.into_iter().collect();
    cells.sort_by(|a, b| {
        f64::from_bits(a.0.0)
            .total_cmp(&f64::from_bits(b.0.0))
            .then_with(|| f64::from_bits(a.0.1).total_cmp(&f64::from_bits(b.0.1)))
    });
    let samples = cells
        .iter()
        .map(|(k, _)| (f64::from_bits(k.0), f64::from_bits(k.1)))
        .collect();
    let values: Vec<Vec<_>> = cells
        .iter()
        .map(|(_, cell)| cell.iter().map(|r| r.and_then(|i| rows[i].y)).collect())
        .collect();
    let order: &dyn crate::shape::StackOrdering = shapes.stack_order().map_or(&s.order, |p| p);
    let offset: &dyn crate::shape::StackOffsetting = shapes.stack_offset().map_or(&s.offset, |p| p);
    let series = kernel(s, limits).generate_with(
        &(0..cells.len()).collect::<Vec<_>>(),
        |_, key, sample, _| Ok(values[sample][key]),
        order,
        offset,
    )?;
    let mut sources = vec![vec![None; cells.len()]; s.groups.len()];
    for (group, series) in series.iter().enumerate() {
        for (sample, p) in series.points.iter().enumerate() {
            if let Some(i) = cells[sample].1[group] {
                let r = &mut rows[i];
                // Missing values are geometry-only even with explicit zero filling.
                if r.y.is_some() {
                    sources[group][sample] = Some(i);
                    r.y = Some(p.y1.0);
                    r.y2 = Some(p.y0.0);
                }
            }
        }
    }
    Ok(StackLayout {
        groups: s.groups.clone(),
        samples,
        series,
        sources,
    })
}
pub(super) fn emit(
    prepared: &mut PreparedLayer,
    layer: &Layer,
    rows: &[EncodedRow],
    stack: StackLayout,
    vertices: &mut usize,
) -> ChartResult<()> {
    for (group, series) in stack.series.iter().enumerate() {
        let style = run_style(
            layer,
            stack.sources[group].iter().flatten().map(|&i| &rows[i]),
        )?;
        let (mut lower, mut upper, mut targets, mut sources) = (vec![], vec![], vec![], vec![]);
        let flush = |prepared: &mut PreparedLayer,
                     lower: &mut Vec<Point>,
                     upper: &mut Vec<Point>,
                     targets: &mut Vec<crate::provenance::Target>,
                     sources: &mut Vec<Option<usize>>| {
            if targets.is_empty() {
                lower.clear();
                upper.clear();
                sources.clear();
                return;
            }
            let geometry = PreparedGeometry::StackBandRun {
                lower: std::mem::take(lower),
                upper: std::mem::take(upper),
                sources: std::mem::take(sources),
            };
            include_geometry(&mut prepared.domains, &geometry);
            Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                geometry,
                targets: std::mem::take(targets),
                group: stack.groups[group].clone(),
                style,
            });
        };
        for (sample, p) in series.points.iter().enumerate() {
            if p.y1.0.is_nan() {
                flush(prepared, &mut lower, &mut upper, &mut targets, &mut sources);
                continue;
            }
            charge(vertices, 2, "stack boundary vertex")?;
            let (x, x2) = stack.samples[sample];
            lower.push(Point::new(x, p.y0.0)?);
            upper.push(Point::new(x2, p.y1.0)?);
            sources.push(stack.sources[group][sample].map(|i| {
                let index = targets.len();
                targets.push(rows[i].target.clone());
                index
            }));
        }
        flush(prepared, &mut lower, &mut upper, &mut targets, &mut sources);
    }
    Ok(())
}
