//! Spatial statistical adapter; geometry consumes generated schemas and exact source memberships.
use super::{
    stats::{group_value, number, numeric_space, raw_number, validate_group, warning},
    *,
};
use crate::data::{DatasetSnapshot, InvalidPolicy};
use crate::provenance::Target;
use crate::{
    AggregateId, ChartResult, DerivedId, Diagnostic, DiagnosticCode, Revision, SchemaVersion,
};
use std::{collections::BTreeMap, sync::Arc};
fn message(diagnostics: &mut Vec<Diagnostic>, text: &str) {
    let mut d = invalid(text);
    d.severity = crate::Severity::Warning;
    diagnostics.push(d);
}
fn invalid(s: &str) -> Diagnostic {
    error(DiagnosticCode::NumericalDomain, s)
}
pub(super) fn validate(spec: &SpatialSpec, limits: CompileLimits) -> ChartResult<()> {
    if spec.retained_fields.len() + spec.retained_numeric.len() > 64 {
        return Err(invalid("Spatial retained aesthetic budget exceeded."));
    }
    for space in [&spec.x_space, &spec.y_space, &spec.z_space] {
        statistics::validate_space(space)?;
    }
    if spec.training_ranges.is_some_and(|r| {
        r.iter()
            .any(|r| r.iter().any(|v| !v.is_finite()) || r[0] > r[1])
    }) {
        return Err(invalid(
            "Spatial training ranges must be finite and ordered.",
        ));
    }
    match &spec.kind {
        SpatialKind::Density {
            n,
            adjust,
            bandwidth,
            ..
        } => {
            if n.contains(&0)
                || n[0]
                    .checked_mul(n[1])
                    .is_none_or(|n| n > limits.max_prepared_rows)
                || adjust.iter().any(|v| !v.is_finite() || *v <= 0.)
                || bandwidth.is_some_and(|h| h.iter().any(|v| !v.is_finite() || *v <= 0.))
            {
                return Err(invalid(
                    "Density grid and bandwidth must be positive and bounded.",
                ));
            }
        }
        SpatialKind::Rectangular { axes, summary, .. } => {
            for a in axes {
                a.options.validate()?;
                if a.bins == 0 {
                    return Err(invalid("Spatial bins must be positive."));
                }
            }
            if summary.is_some() && spec.z.is_none() {
                return Err(invalid("Cell summaries require response input."));
            }
        }
        SpatialKind::Hexagonal {
            bins,
            binwidth,
            summary,
            ..
        } => {
            if bins.contains(&0)
                || binwidth.is_some_and(|w| w.iter().any(|v| !v.is_finite() || *v <= 0.))
            {
                return Err(invalid("Hex dimensions must be positive."));
            }
            if summary.is_some() && spec.z.is_none() {
                return Err(invalid("Hex summaries require response input."));
            }
        }
        SpatialKind::Contour { .. } => {
            if spec.z.is_none() {
                return Err(invalid("Contours require response input."));
            }
        }
        SpatialKind::Ellipse {
            segments,
            level,
            kind,
        } => {
            if *segments == 0
                || segments.saturating_add(1) > limits.max_prepared_rows
                || !level.is_finite()
                || *level < 0.
                || (*kind != EllipseKind::Euclidean && *level >= 1.)
            {
                return Err(invalid("Ellipse controls are outside the bounded domain."));
            }
        }
    }
    Ok(())
}
pub(super) fn schema(
    spec: &SpatialSpec,
    data: &DatasetSnapshot,
    limits: CompileLimits,
) -> ChartResult<Vec<StatColumn>> {
    validate(spec, limits)?;
    validate_group(data, &spec.grouping)?;
    for input in spec.numerics() {
        numeric_space(data, input)?;
    }
    let x = statistics::space(data, &spec.x, &spec.x_space)?;
    let y = statistics::space(data, &spec.y, &spec.y_space)?;
    let mut fields = vec![
        statistics::group_column(data, &spec.grouping, limits)?,
        StatColumn {
            field: StatField::Count,
            kind: GeneratedKind::UInt64,
            nullable: false,
            space: ValueSpace::Data,
        },
    ];
    for (field, space) in [
        (StatField::X, x.clone()),
        (StatField::Y, y.clone()),
        (StatField::Width, x),
        (StatField::Height, y),
        (StatField::WeightedCount, ValueSpace::Data),
        (StatField::NormalizedCount, ValueSpace::Data),
        (StatField::Mean, ValueSpace::Data),
        (StatField::Density, ValueSpace::Data),
        (StatField::ScaledDensity, ValueSpace::Data),
        (StatField::DensityCount, ValueSpace::Data),
        (StatField::Level, ValueSpace::Data),
        (StatField::LevelLow, ValueSpace::Data),
        (StatField::LevelHigh, ValueSpace::Data),
        (StatField::LevelMid, ValueSpace::Data),
        (StatField::NormalizedLevel, ValueSpace::Data),
        (StatField::Piece, ValueSpace::Data),
        (StatField::Subgroup, ValueSpace::Data),
    ] {
        fields.push(StatColumn {
            field,
            space,
            kind: GeneratedKind::Float64,
            nullable: true,
        });
    }
    Ok(fields)
}
fn transformed(v: Option<f64>, s: &StatSpace) -> Option<f64> {
    v.and_then(|v| {
        let v = match s {
            StatSpace::Data => v,
            StatSpace::Transformed(t) => t.factor * v + t.offset,
        };
        v.is_finite().then_some(v)
    })
}
#[derive(Clone)]
struct Sample {
    key: crate::RowKey,
    point: [f64; 2],
    z: Option<f64>,
    weight: f64,
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    table: &mut PreparedTable,
    data: &DatasetSnapshot,
    spec: &SpatialSpec,
    policy: InvalidPolicy,
    scope: &str,
    limits: CompileLimits,
    counts: &mut PopulationCounts,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<(Grouping, StatSpace)> {
    let mut fields = schema(spec, data, limits)?;
    let PreparedRows::Source(rows) = &table.rows else {
        return Err(invalid("Spatial statistics require source rows."));
    };
    let index = data
        .rows()
        .map(|r| (r.key(), r))
        .collect::<BTreeMap<_, _>>();
    let mut groups: BTreeMap<GroupValue, Vec<Sample>> = BTreeMap::new();
    let mut excluded = Vec::new();
    let mut dropped = 0;
    for row in rows.iter() {
        let r = index[&row.key];
        let g = group_value(r, &spec.grouping);
        let x = transformed(number(r, &spec.x), &spec.x_space);
        let y = transformed(number(r, &spec.y), &spec.y_space);
        let z = spec
            .z
            .as_ref()
            .and_then(|v| transformed(number(r, v), &spec.z_space));
        let w = if matches!(
            spec.kind,
            SpatialKind::Density { .. }
                | SpatialKind::Contour { .. }
                | SpatialKind::Rectangular {
                    summary: Some(_),
                    ..
                }
                | SpatialKind::Hexagonal {
                    summary: Some(_),
                    ..
                }
        ) {
            1.
        } else {
            spec.weight
                .as_ref()
                .and_then(|w| raw_number(r, w))
                .unwrap_or(if spec.weight.is_some() { f64::NAN } else { 1. })
        };
        if g.is_none()
            || x.is_none()
            || y.is_none()
            || (z.is_none()
                && matches!(
                    spec.kind,
                    SpatialKind::Rectangular {
                        summary: Some(_),
                        ..
                    } | SpatialKind::Hexagonal {
                        summary: Some(_),
                        ..
                    }
                ))
        {
            dropped += 1;
            if excluded.len() < 32 {
                excluded.push(row.key);
            }
            continue;
        }
        groups.entry(g.unwrap()).or_default().push(Sample {
            key: row.key,
            point: [x.unwrap(), y.unwrap()],
            z,
            weight: w,
        });
        if groups.len() > limits.max_groups {
            return Err(invalid("Spatial group budget exceeded."));
        }
    }
    counts.invalid_stat += dropped;
    warning(
        policy,
        dropped,
        excluded,
        "Spatial statistic excluded missing coordinates, groups or required summary responses.",
        diagnostics,
    )?;
    if groups.is_empty() {
        table.schema = OutputSchema::Statistical {
            version: SchemaVersion::new(1),
            fields,
        };
        table.rows = PreparedRows::Statistical(Arc::from([]));
        return Ok((spec.grouping.clone(), spec.x_space.clone()));
    }
    let ranges = spec.training_ranges.unwrap_or_else(|| {
        groups
            .values()
            .flatten()
            .fold([[f64::INFINITY, f64::NEG_INFINITY]; 2], |mut r, s| {
                for (axis, p) in s.point.iter().enumerate() {
                    r[axis][0] = r[axis][0].min(*p);
                    r[axis][1] = r[axis][1].max(*p);
                }
                r
            })
    });
    let mut density_grids = BTreeMap::new();
    let mut contour_range = [f64::INFINITY, f64::NEG_INFINITY];
    if let SpatialKind::Density {
        bandwidth,
        adjust,
        n,
        contour_var,
        ..
    } = &spec.kind
    {
        if n[0]
            .checked_mul(n[1])
            .and_then(|n| n.checked_mul(groups.len()))
            .is_none_or(|n| n > limits.max_prepared_rows)
        {
            return Err(invalid("Shared density grid budget exceeded."));
        }
        for (group, samples) in &groups {
            let points = samples.iter().map(|s| s.point).collect::<Vec<_>>();
            let grid = spatial::density(
                &points,
                *bandwidth,
                *adjust,
                *n,
                ranges,
                limits.max_prepared_rows,
            )?;
            let maximum = grid.values.iter().copied().fold(0., f64::max);
            for value in &grid.values {
                let value = match contour_var {
                    DensityContour::Density => *value,
                    DensityContour::Normalized => value / maximum,
                    DensityContour::Count => value * samples.len() as f64,
                };
                if value.is_finite() {
                    contour_range[0] = contour_range[0].min(value);
                    contour_range[1] = contour_range[1].max(value);
                }
            }
            density_grids.insert(group.clone(), grid);
        }
    } else {
        for value in groups.values().flatten().filter_map(|s| s.z) {
            contour_range[0] = contour_range[0].min(value);
            contour_range[1] = contour_range[1].max(value);
        }
    }
    let common_levels = match &spec.kind {
        SpatialKind::Density {
            contour: Some(levels),
            ..
        }
        | SpatialKind::Contour { levels, .. }
            if contour_range[0].is_finite() =>
        {
            Some(ContourLevels {
                breaks: Some(spatial::contour_levels(
                    contour_range,
                    levels,
                    limits.max_vertices,
                )?),
                bins: None,
                binwidth: None,
            })
        }
        _ => None,
    };
    let mut output = Vec::new();
    for (group, samples) in &groups {
        let points = samples.iter().map(|s| s.point).collect::<Vec<_>>();
        let weights = samples.iter().map(|s| s.weight).collect::<Vec<_>>();
        let mut components = Vec::new();
        let mut retained_numeric = Vec::new();
        for field in &spec.retained_fields {
            let input = Grouping::Interaction(vec![*field]);
            let first = group_value(index[&samples[0].key], &input);
            if samples
                .iter()
                .all(|s| group_value(index[&s.key], &input) == first)
            {
                if let Some(GroupValue::Interaction(mut p)) = first {
                    components.append(&mut p);
                }
            } else {
                message(
                    diagnostics,
                    &format!(
                        "Spatial statistic dropped varying source aesthetic field {}.",
                        field.get()
                    ),
                );
            }
        }
        for input in &spec.retained_numeric {
            let first = number(index[&samples[0].key], input);
            if samples
                .iter()
                .all(|s| number(index[&s.key], input) == first)
            {
                retained_numeric.push((input.clone(), first));
            } else {
                message(
                    diagnostics,
                    "Spatial statistic dropped a varying evaluated numeric aesthetic.",
                );
            }
        }
        let retained = if components.is_empty() {
            GroupValue::All
        } else {
            GroupValue::Interaction(components)
        };
        let all = (0..samples.len()).collect::<Vec<_>>();
        let mut keys = samples.iter().map(|s| s.key).collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();
        let all_members: Arc<[crate::RowKey]> = keys.into();
        let mut emit = |members: &[usize],
                        name: String,
                        derived: bool,
                        row_group: GroupValue,
                        values: Vec<(StatField, Option<f64>)>|
         -> ChartResult<()> {
            if output.len() >= limits.max_prepared_rows {
                return Err(invalid("Spatial generated row budget exceeded."));
            }
            let members: Arc<[crate::RowKey]> = if derived {
                all_members.clone()
            } else {
                let mut keys = members.iter().map(|i| samples[*i].key).collect::<Vec<_>>();
                keys.sort_unstable();
                keys.dedup();
                keys.into()
            };
            let identity = format!("chart.spatial/{scope}/{group:?}/{name}");
            let target = if derived {
                Target::Derived {
                    id: DerivedId::new(0),
                    model: identity,
                    model_version: Revision::INITIAL,
                    inputs: vec![table.input],
                }
            } else {
                Target::Aggregate {
                    id: AggregateId::new(0),
                    group: identity,
                    input: table.input,
                    members: members.clone(),
                }
            };
            output.push(StatisticalRow {
                outliers: vec![],
                group: row_group,
                retained: retained.clone(),
                retained_numeric: retained_numeric.clone(),
                count: members.len() as u64,
                members,
                target,
                values: values
                    .into_iter()
                    .map(|(field, value)| StatValue {
                        field,
                        value: value.filter(|v| v.is_finite()),
                    })
                    .collect(),
            });
            Ok(())
        };
        match &spec.kind {
            SpatialKind::Rectangular { summary, drop, .. }
            | SpatialKind::Hexagonal { summary, drop, .. } => {
                let values = if summary.is_some() {
                    samples
                        .iter()
                        .map(|s| s.z.unwrap_or(f64::NAN))
                        .collect::<Vec<_>>()
                } else {
                    weights.clone()
                };
                let cells = if let SpatialKind::Rectangular { axes, .. } = &spec.kind {
                    spatial::rectangular(
                        &points,
                        &values,
                        axes,
                        ranges,
                        *summary,
                        *drop,
                        limits.max_prepared_rows,
                    )?
                } else {
                    let SpatialKind::Hexagonal { binwidth, bins, .. } = &spec.kind else {
                        unreachable!()
                    };
                    spatial::hexagonal(
                        &points,
                        &values,
                        binwidth.unwrap_or([
                            (ranges[0][1] - ranges[0][0]) / bins[0] as f64,
                            (ranges[1][1] - ranges[1][0]) / bins[1] as f64,
                        ]),
                        *summary,
                        *drop,
                        limits.max_prepared_rows,
                    )?
                };
                let total = cells.iter().filter_map(|c| c.value).sum::<f64>();
                let maximum = cells
                    .iter()
                    .filter_map(|c| c.value)
                    .fold(f64::NEG_INFINITY, f64::max);
                let max_density = cells
                    .iter()
                    .filter_map(|c| c.value.map(|v| v / total))
                    .filter(|v| v.is_finite())
                    .fold(f64::NEG_INFINITY, f64::max);
                for (i, c) in cells.into_iter().enumerate() {
                    emit(
                        &c.members,
                        format!("cell/{i}"),
                        false,
                        group.clone(),
                        vec![
                            (StatField::X, Some(c.center[0])),
                            (StatField::Y, Some(c.center[1])),
                            (StatField::Width, Some(c.size[0])),
                            (StatField::Height, Some(c.size[1])),
                            (
                                StatField::Density,
                                if summary.is_none() {
                                    c.value.map(|v| v / total)
                                } else {
                                    None
                                },
                            ),
                            (
                                StatField::ScaledDensity,
                                if summary.is_none() {
                                    c.value.map(|v| v / total / max_density)
                                } else {
                                    None
                                },
                            ),
                            (
                                StatField::NormalizedCount,
                                if summary.is_none() {
                                    c.value.map(|v| v / maximum)
                                } else {
                                    None
                                },
                            ),
                            (
                                if summary.is_some() {
                                    StatField::Mean
                                } else {
                                    StatField::WeightedCount
                                },
                                c.value,
                            ),
                        ],
                    )?;
                }
            }
            SpatialKind::Ellipse {
                kind,
                level,
                segments,
            } => {
                let path = match spatial::ellipse(
                    &points,
                    &weights,
                    *kind,
                    *level,
                    *segments,
                    limits.max_prepared_rows,
                ) {
                    Ok(path) => path,
                    Err(error) => {
                        warning(
                            policy,
                            samples.len(),
                            samples.iter().take(32).map(|s| s.key).collect(),
                            format!(
                                "Spatial ellipse omitted an invalid group: {}",
                                error.message
                            ),
                            diagnostics,
                        )?;
                        Vec::new()
                    }
                };
                if path.is_empty() && samples.len() < 4 {
                    warning(
                        policy,
                        samples.len(),
                        samples.iter().take(32).map(|s| s.key).collect(),
                        "Spatial ellipse requires at least four observations.",
                        diagnostics,
                    )?;
                }
                for (i, p) in path.into_iter().enumerate() {
                    emit(
                        &all,
                        format!("ellipse/{i}"),
                        true,
                        group.clone(),
                        vec![(StatField::X, Some(p[0])), (StatField::Y, Some(p[1]))],
                    )?;
                }
            }
            SpatialKind::Density {
                contour,
                contour_var,
                filled,
                ..
            } => {
                let grid = &density_grids[group];
                let maximum = grid.values.iter().copied().fold(0., f64::max);
                if let Some(levels) = contour {
                    let values = grid
                        .values
                        .iter()
                        .map(|v| {
                            Some(match contour_var {
                                DensityContour::Density => *v,
                                DensityContour::Normalized => v / maximum,
                                DensityContour::Count => v * samples.len() as f64,
                            })
                        })
                        .collect::<Vec<_>>();
                    emit_contours(
                        &grid.x,
                        &grid.y,
                        &values,
                        common_levels.as_ref().unwrap_or(levels),
                        *filled,
                        0.,
                        limits.max_vertices,
                        group,
                        &all,
                        &mut emit,
                    )?;
                } else {
                    for (i, v) in grid.values.iter().enumerate() {
                        emit(
                            &all,
                            format!("density/{i}"),
                            true,
                            group.clone(),
                            vec![
                                (StatField::X, Some(grid.x[i % grid.x.len()])),
                                (StatField::Y, Some(grid.y[i / grid.x.len()])),
                                (StatField::Width, grid.x.get(1).map(|v| v - grid.x[0])),
                                (StatField::Height, grid.y.get(1).map(|v| v - grid.y[0])),
                                (StatField::Density, Some(*v)),
                                (StatField::ScaledDensity, Some(v / maximum)),
                                (StatField::DensityCount, Some(v * samples.len() as f64)),
                            ],
                        )?;
                    }
                }
            }
            SpatialKind::Contour { levels, filled } => {
                let grid = spatial::grid::grid(
                    &points,
                    &samples.iter().map(|s| s.z).collect::<Vec<_>>(),
                    limits.max_prepared_rows,
                )?;
                emit_contours(
                    &grid.x,
                    &grid.y,
                    &grid.z,
                    common_levels.as_ref().unwrap_or(levels),
                    *filled,
                    grid.angle,
                    limits.max_vertices,
                    group,
                    &all,
                    &mut emit,
                )?;
            }
        }
    }
    if matches!(
        spec.kind,
        SpatialKind::Contour { filled: true, .. } | SpatialKind::Density { filled: true, .. }
    ) {
        let mut intervals = output
            .iter()
            .filter_map(|r| {
                Some((
                    r.value(&StatField::LevelLow)?,
                    r.value(&StatField::LevelHigh)?,
                ))
            })
            .collect::<Vec<_>>();
        intervals.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
        intervals.dedup();
        let categories = spatial::labels::labels(&intervals);
        if categories.len() > limits.max_groups {
            return Err(invalid(
                "Filled contour level catalog exceeds group budget.",
            ));
        }
        for row in &mut output {
            if let (Some(a), Some(b)) = (
                row.value(&StatField::LevelLow),
                row.value(&StatField::LevelHigh),
            ) {
                let index = intervals
                    .binary_search_by(|v| v.0.total_cmp(&a).then(v.1.total_cmp(&b)))
                    .unwrap();
                if let Some(value) = row.values.iter_mut().find(|v| v.field == StatField::Level) {
                    value.value = Some(index as f64);
                }
            }
        }
        if let Some(field) = fields.iter_mut().find(|f| f.field == StatField::Level) {
            field.kind = GeneratedKind::Categorical;
            field.space = ValueSpace::Categorical { categories };
        }
    }
    if matches!(
        spec.kind,
        SpatialKind::Contour { .. }
            | SpatialKind::Density {
                contour: Some(_),
                ..
            }
    ) {
        let groups = output
            .iter()
            .map(|r| r.group.clone())
            .collect::<std::collections::BTreeSet<_>>();
        if groups.len() > limits.max_groups {
            return Err(invalid("Contour group catalog exceeds group budget."));
        }
        if let Some(field) = fields.iter_mut().find(|f| f.field == StatField::Group) {
            field.space = ValueSpace::Categorical {
                categories: groups.iter().map(statistics::group_label).collect(),
            };
        }
    }
    table.schema = OutputSchema::Statistical {
        version: SchemaVersion::new(1),
        fields,
    };
    table.rows = PreparedRows::Statistical(output.into());
    Ok((spec.grouping.clone(), spec.x_space.clone()))
}
type Emit<'a> = dyn FnMut(&[usize], String, bool, GroupValue, Vec<(StatField, Option<f64>)>) -> ChartResult<()>
    + 'a;
#[allow(clippy::too_many_arguments)]
fn emit_contours(
    x: &[f64],
    y: &[f64],
    z: &[Option<f64>],
    levels: &ContourLevels,
    filled: bool,
    angle: f64,
    budget: usize,
    group: &GroupValue,
    members: &[usize],
    emit: &mut Emit<'_>,
) -> ChartResult<()> {
    let range = z
        .iter()
        .flatten()
        .fold([f64::INFINITY, f64::NEG_INFINITY], |r, v| {
            [r[0].min(*v), r[1].max(*v)]
        });
    if !range[0].is_finite() {
        return Ok(());
    }
    let breaks = spatial::contour_levels(range, levels, budget)?;
    let mut contours = Vec::new();
    let mut vertices = 0usize;
    for (i, level) in breaks.iter().enumerate() {
        if filled && i + 1 == breaks.len() {
            break;
        }
        let paths = if filled {
            spatial::isobands(x, y, z, *level, breaks[i + 1], budget)?
        } else {
            spatial::isolines(x, y, z, *level, budget)?
        };
        vertices = vertices.saturating_add(paths.iter().map(|p| p.points.len()).sum::<usize>());
        if vertices > budget {
            return Err(invalid("Contour path vertex budget exceeded."));
        }
        if !paths.is_empty() {
            contours.push((i, *level, paths));
        }
    }
    let maximum = contours
        .iter()
        .map(|(i, level, _)| if filled { breaks[i + 1] } else { *level })
        .fold(f64::NEG_INFINITY, f64::max);
    let mut piece = 0usize;
    for (i, level, paths) in contours {
        for (subgroup, path) in paths.into_iter().enumerate() {
            piece += 1;
            let row_group = GroupValue::Text(format!(
                "{group:?}/level/{i}/{}",
                if filled { 0 } else { piece }
            ));
            for (j, p) in spatial::grid::rotate(&path.points, angle)
                .into_iter()
                .enumerate()
            {
                let mut values = vec![
                    (StatField::X, Some(p[0])),
                    (StatField::Y, Some(p[1])),
                    (StatField::Level, Some(level)),
                    (StatField::NormalizedLevel, {
                        let value = if filled { breaks[i + 1] } else { level };
                        let normalized = value / maximum;
                        normalized.is_finite().then_some(normalized)
                    }),
                    (StatField::Piece, Some(piece as f64)),
                    (StatField::Subgroup, Some(subgroup as f64 + 1.)),
                ];
                if filled {
                    values.push((StatField::LevelLow, Some(level)));
                    values.push((StatField::LevelHigh, Some(breaks[i + 1])));
                    values.push((StatField::LevelMid, Some(0.5 * (level + breaks[i + 1]))));
                }
                emit(
                    members,
                    format!("contour/{i}/{subgroup}/{j}"),
                    true,
                    row_group.clone(),
                    values,
                )?;
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::*;
    use crate::transaction::DataStore;
    use crate::{DatasetId, FieldId, RowKey, SourceEpoch};
    #[test]
    fn source_cells_and_derived_grid_have_exact_shared_membership() {
        let fields = [FieldId::new(1), FieldId::new(2), FieldId::new(3)];
        let schema = Arc::new(
            Schema::new(
                SchemaVersion::new(1),
                fields
                    .iter()
                    .map(|id| Field {
                        id: *id,
                        name: format!("f{}", id.get()),
                        kind: FieldKind::Float64,
                        nullable: false,
                        unit: None,
                        label: None,
                    })
                    .collect(),
            )
            .unwrap(),
        );
        let batch = NormalizedBatch::new(
            schema,
            (1..=5).map(RowKey::new).collect(),
            [
                vec![0., 0.2, 1., 2., 2.],
                vec![0., 0.5, 1., 2., 0.],
                vec![1., 2., 0., 4., 1.],
            ]
            .into_iter()
            .map(|v| Column::new(ColumnValues::Float64(v), vec![true; 5], None))
            .collect(),
            DataLimits::default(),
        )
        .unwrap();
        let store = DataStore::new(
            SourceEpoch::new(1),
            vec![(DatasetId::new(1), batch)],
            DataLimits::default(),
        )
        .unwrap();
        let snapshot = store.snapshot();
        let snapshot = snapshot.get().unwrap();
        let data = snapshot.dataset(DatasetId::new(1)).unwrap();
        let axis = SummaryBins {
            options: GgplotBinOptions {
                binwidth: Some(1.),
                boundary: Some(0.),
                ..Default::default()
            },
            ..Default::default()
        };
        for kind in [
            SpatialKind::Rectangular {
                axes: [axis.clone(), axis],
                summary: None,
                drop: false,
            },
            SpatialKind::Density {
                bandwidth: Some([1., 1.]),
                adjust: [1., 1.],
                n: [3, 3],
                contour: None,
                contour_var: DensityContour::Density,
                filled: false,
            },
        ] {
            let derived = matches!(kind, SpatialKind::Density { .. });
            let spec = SpatialSpec {
                x: Numeric::Field(fields[0]),
                y: Numeric::Field(fields[1]),
                z: None,
                weight: Some(Numeric::Field(fields[2])),
                grouping: Grouping::All,
                x_space: StatSpace::Data,
                y_space: StatSpace::Data,
                z_space: StatSpace::Data,
                training_ranges: None,
                retained_fields: vec![],
                retained_numeric: vec![],
                kind,
            };
            let mut table = PreparedTable {
                schema: OutputSchema::Source(data.schema().clone()),
                rows: PreparedRows::Source(
                    data.rows()
                        .map(|r| SourceRow {
                            key: r.key(),
                            ordinal: r.ordinal(),
                        })
                        .collect::<Vec<_>>()
                        .into(),
                ),
                input: data.version(),
                space: ValueSpace::Data,
                operations: vec![],
            };
            run(
                &mut table,
                data,
                &spec,
                InvalidPolicy::Strict,
                "test",
                CompileLimits::default(),
                &mut PopulationCounts::default(),
                &mut vec![],
            )
            .unwrap();
            let PreparedRows::Statistical(rows) = table.rows else {
                panic!()
            };
            assert!(!rows.is_empty());
            if derived {
                assert_eq!(rows.len(), 9);
                for r in rows.iter() {
                    assert_eq!(r.count, 5);
                    assert!(matches!(r.target, Target::Derived { .. }));
                    assert!(Arc::ptr_eq(&rows[0].members, &r.members));
                }
            } else {
                assert_eq!(rows.iter().map(|r| r.count).sum::<u64>(), 5);
                assert!(
                    rows.iter()
                        .all(|r| matches!(r.target, Target::Aggregate { .. }))
                );
            }
        }
    }
}
