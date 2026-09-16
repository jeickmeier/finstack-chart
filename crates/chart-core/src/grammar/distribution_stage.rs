//! Shared statistical adapter for distribution kernels and exact source memberships.
use super::{
    distributions as kernel,
    stats::{group_value, number, numeric_space, raw_number, validate_group, warning},
    *,
};
use crate::data::{DatasetSnapshot, InvalidPolicy};
use crate::provenance::{SourceRef, Target};
use crate::{
    AggregateId, ChartResult, DerivedId, Diagnostic, DiagnosticCode, Revision, RowKey,
    SchemaVersion,
};
use std::{collections::BTreeMap, sync::Arc};
fn invalid(message: &str) -> Diagnostic {
    error(DiagnosticCode::NumericalDomain, message)
}
pub(super) fn validate(spec: &DistributionSpec, limits: CompileLimits) -> ChartResult<()> {
    if spec
        .retained_fields
        .len()
        .saturating_add(spec.retained_numeric.len())
        > 64
    {
        return Err(invalid("Distribution retained aesthetic budget exceeded."));
    }
    super::statistics::validate_space(&spec.space)?;
    super::statistics::validate_space(&spec.position_space)?;
    if spec.width.is_some_and(|v| !v.is_finite() || v < 0.)
        || spec
            .training_range
            .is_some_and(|r| r.iter().any(|v| !v.is_finite()) || r[0] > r[1])
    {
        return Err(invalid(
            "Distribution width and trained range must be finite and ordered.",
        ));
    }
    match &spec.kind {
        DistributionKind::Boxplot {
            coefficient,
            quantile_type,
        } => {
            if !coefficient.is_finite() || *coefficient < 0. || !(1..=9).contains(quantile_type) {
                return Err(invalid(
                    "Boxplot requires nonnegative fences and quantile type1..9.",
                ));
            }
        }
        DistributionKind::Density { controls, .. } | DistributionKind::Violin { controls, .. } => {
            if controls.n == 0
                || controls.n > limits.max_prepared_rows
                || !controls.adjust.is_finite()
                || controls.adjust <= 0.
                || controls.lower.is_some_and(|v| !v.is_finite())
                || controls.upper.is_some_and(|v| !v.is_finite())
                || matches!(controls.bandwidth,Bandwidth::Fixed(v)if !v.is_finite()||v<=0.)
            {
                return Err(invalid(
                    "Density controls require a bounded grid and finite positive bandwidth/adjustment.",
                ));
            }
            if let DistributionKind::Violin { quantiles, .. } = &spec.kind
                && (controls.n != 512
                    || quantiles.iter().any(|q| !(0. ..=1.).contains(q))
                    || quantiles.len() > limits.max_prepared_rows)
            {
                return Err(invalid(
                    "Reference violin uses512density samples and quantiles in[0,1].",
                ));
            }
        }
        DistributionKind::Dotplot {
            bin_width,
            origin,
            method,
            ..
        } => {
            if bin_width
                .is_some_and(|v| !v.is_finite() || *method == DotBinMethod::HistoDot && v <= 0.)
                || origin.is_some_and(|v| !v.is_finite())
            {
                return Err(invalid(
                    "Dot width/origin must be finite; histodot requires positive width.",
                ));
            }
        }
    }
    Ok(())
}
pub(super) fn schema(
    spec: &DistributionSpec,
    data: &DatasetSnapshot,
    limits: CompileLimits,
) -> ChartResult<Vec<StatColumn>> {
    validate(spec, limits)?;
    validate_group(data, &spec.grouping)?;
    if let Some(w) = &spec.weight {
        numeric_space(data, w)?;
    }
    if !spec.retained_fields.is_empty() {
        validate_group(data, &Grouping::Interaction(spec.retained_fields.clone()))?;
    }
    for input in &spec.retained_numeric {
        numeric_space(data, input)?;
    }
    let sample = super::statistics::space(data, &spec.input, &spec.space)?;
    let position = spec
        .position
        .as_ref()
        .map(|p| super::statistics::space(data, p, &spec.position_space))
        .transpose()?
        .unwrap_or(ValueSpace::Data);
    let mut fields = vec![
        super::statistics::group_column(data, &spec.grouping, limits)?,
        StatColumn {
            field: StatField::Count,
            kind: GeneratedKind::UInt64,
            nullable: false,
            space: ValueSpace::Data,
        },
    ];
    let mut add = |field, space, nullable| {
        fields.push(StatColumn {
            field,
            space,
            nullable,
            kind: GeneratedKind::Float64,
        })
    };
    if spec.sample_axis() == 0 {
        add(StatField::X, sample.clone(), true);
        add(StatField::Y, ValueSpace::Data, true);
    } else {
        add(StatField::X, position.clone(), true);
        add(StatField::Y, sample.clone(), true);
    }
    add(StatField::Width, position.clone(), true);
    match &spec.kind {
        DistributionKind::Boxplot { .. } => {
            for field in [
                StatField::Lower,
                StatField::Middle,
                StatField::Upper,
                StatField::WhiskerLower,
                StatField::WhiskerUpper,
                StatField::NotchLower,
                StatField::NotchUpper,
            ] {
                add(field, sample.clone(), true);
            }
            add(StatField::RelativeWidth, ValueSpace::Data, false);
        }
        DistributionKind::Density { .. } | DistributionKind::Violin { .. } => {
            for field in [
                StatField::Density,
                StatField::ScaledDensity,
                StatField::DensityCount,
                StatField::WeightedDensity,
            ] {
                add(field, ValueSpace::Data, true);
            }
            if matches!(spec.kind, DistributionKind::Violin { .. }) {
                add(StatField::ViolinWidth, ValueSpace::Data, true);
                add(StatField::QuantileFlag, ValueSpace::Data, true);
            }
        }
        DistributionKind::Dotplot { .. } => {
            add(StatField::BinWidth, sample, true);
            add(StatField::WeightedCount, ValueSpace::Data, false);
            add(StatField::Proportion, ValueSpace::Data, true);
        }
    }
    Ok(fields)
}
#[derive(Clone)]
struct Sample {
    key: RowKey,
    value: f64,
    position: Option<f64>,
    weight: f64,
}
fn transform(v: Option<f64>, space: &StatSpace) -> Option<f64> {
    v.and_then(|v| {
        let v = match space {
            StatSpace::Data => v,
            StatSpace::Transformed(t) => t.factor * v + t.offset,
        };
        v.is_finite().then_some(v)
    })
}
fn message(diagnostics: &mut Vec<Diagnostic>, text: &str) {
    let mut d = invalid(text);
    d.severity = crate::Severity::Warning;
    diagnostics.push(d);
}
fn center_width(samples: &[Sample], default: f64, explicit: Option<f64>) -> (f64, f64) {
    let range = samples
        .iter()
        .filter_map(|s| s.position)
        .fold(None, |r: Option<(f64, f64)>, v| {
            Some(r.map_or((v, v), |(a, b)| (a.min(v), b.max(v))))
        });
    let (center, width) = range.map_or((0., default), |(a, b)| {
        (a.midpoint(b), if a != b { 0.9 * (b - a) } else { default })
    });
    (center, explicit.unwrap_or(width))
}
type RetainedAesthetics = (GroupValue, Vec<(Numeric, Option<f64>)>);
struct Output<'a> {
    table: &'a PreparedTable,
    scope: &'a str,
    rows: Vec<StatisticalRow>,
    limit: usize,
    memberships: BTreeMap<Vec<RowKey>, Arc<[RowKey]>>,
    retained: BTreeMap<GroupValue, RetainedAesthetics>,
}
impl Output<'_> {
    fn push(
        &mut self,
        group: &GroupValue,
        samples: &[Sample],
        identity: &str,
        derived: bool,
        values: Vec<(StatField, Option<f64>)>,
        outliers: Vec<StatOutlier>,
    ) -> ChartResult<()> {
        if self.rows.len() >= self.limit {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Distribution output row budget exceeded.",
            ));
        }
        let mut keys = samples.iter().map(|s| s.key).collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();
        let members = self
            .memberships
            .entry(keys)
            .or_insert_with_key(|keys| Arc::from(keys.clone()))
            .clone();
        let name = format!("chart.distribution/{}/{group:?}/{identity}", self.scope);
        let target = if derived {
            Target::Derived {
                id: DerivedId::new(0),
                model: name,
                model_version: Revision::INITIAL,
                inputs: vec![self.table.input],
            }
        } else {
            Target::Aggregate {
                id: AggregateId::new(0),
                group: name,
                input: self.table.input,
                members: members.clone(),
            }
        };
        self.rows.push(StatisticalRow {
            outliers,
            retained: self
                .retained
                .get(group)
                .map_or(GroupValue::All, |r| r.0.clone()),
            retained_numeric: self
                .retained
                .get(group)
                .map_or_else(Vec::new, |r| r.1.clone()),
            group: group.clone(),
            count: members.len() as u64,
            values: values
                .into_iter()
                .map(|(field, value)| StatValue {
                    field,
                    value: value.filter(|v| v.is_finite()),
                })
                .collect(),
            members,
            target,
        });
        Ok(())
    }
}
fn density_values(p: &kernel::DensityPoint) -> Vec<(StatField, Option<f64>)> {
    vec![
        (StatField::Density, Some(p.density)),
        (StatField::ScaledDensity, Some(p.scaled)),
        (StatField::DensityCount, Some(p.count)),
        (StatField::WeightedDensity, Some(p.wdensity)),
    ]
}
fn density_warnings(d: &kernel::DensityEstimate, diagnostics: &mut Vec<Diagnostic>) {
    if d.removed_bounds {
        message(
            diagnostics,
            "Density removed observations outside support bounds.",
        );
    }
    if d.too_few {
        message(
            diagnostics,
            "Density requires at least two observations per group.",
        );
    }
    if d.boundary_minimum {
        message(
            diagnostics,
            "Bandwidth minimum occurred at an end of its search interval.",
        );
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    table: &mut PreparedTable,
    data: &DatasetSnapshot,
    spec: &DistributionSpec,
    policy: InvalidPolicy,
    scope: &str,
    limits: CompileLimits,
    counts: &mut PopulationCounts,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<(Grouping, StatSpace)> {
    let fields = schema(spec, data, limits)?;
    let PreparedRows::Source(rows) = &table.rows else {
        return Err(invalid("Distribution statistics require source rows."));
    };
    let index = data
        .rows()
        .map(|r| (r.key(), r))
        .collect::<BTreeMap<_, _>>();
    let mut groups: BTreeMap<GroupValue, Vec<Sample>> = BTreeMap::new();
    let mut excluded = vec![];
    let mut dropped = 0;
    for row in rows.iter() {
        let source = index[&row.key];
        let group = group_value(source, &spec.grouping);
        let value = transform(number(source, &spec.input), &spec.space);
        let position = spec
            .position
            .as_ref()
            .and_then(|p| transform(number(source, p), &spec.position_space));
        let weight = spec
            .weight
            .as_ref()
            .map_or(Some(1.), |w| raw_number(source, w));
        let density = matches!(spec.kind, DistributionKind::Density { .. });
        if group.is_none()
            || value.is_none()
            || spec.position.is_some() && position.is_none()
            || !density && weight.is_none_or(|w| !w.is_finite())
        {
            dropped += 1;
            if excluded.len() < 32 {
                excluded.push(row.key);
            }
            continue;
        }
        groups.entry(group.unwrap()).or_default().push(Sample {
            key: row.key,
            value: value.unwrap(),
            position,
            weight: weight.unwrap_or(f64::NAN),
        });
        if groups.len() > limits.max_groups {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Distribution group budget exceeded.",
            ));
        }
    }
    counts.invalid_stat += dropped;
    warning(
        policy,
        dropped,
        excluded,
        "Distribution excluded missing/nonfinite samples, positions, weights or groups.",
        diagnostics,
    )?;
    let all = groups.values().flatten().cloned().collect::<Vec<_>>();
    let range = spec.training_range.unwrap_or_else(|| {
        all.iter().fold([f64::INFINITY, f64::NEG_INFINITY], |r, s| {
            [r[0].min(s.value), r[1].max(s.value)]
        })
    });
    let default_width = if matches!(spec.kind, DistributionKind::Boxplot { .. }) {
        let mut positions = all.iter().filter_map(|s| s.position).collect::<Vec<_>>();
        positions.sort_by(f64::total_cmp);
        positions.dedup();
        let discrete = spec.position.as_ref().is_some_and(|p| {
            matches!(
                numeric_space(data, p),
                Ok(ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. })
            )
        }) || matches!((&spec.position,&spec.position_space),(Some(Numeric::Field(f)),StatSpace::Data)if data.schema().field(*f).is_some_and(|(_,f)|f.kind==crate::data::FieldKind::Int64));
        let resolution = if discrete || positions.len() < 2 {
            1.
        } else {
            positions.push(0.);
            positions.sort_by(f64::total_cmp);
            positions
                .windows(2)
                .map(|v| v[1] - v[0])
                .filter(|d| *d > libm::sqrt(f64::EPSILON))
                .reduce(f64::min)
                .unwrap_or(1.)
        };
        0.75 * resolution
    } else {
        0.9 * super::recipe_emit::resolution(all.iter().filter_map(|s| s.position))
    };
    let mut retained = BTreeMap::new();
    for (group, samples) in &groups {
        let mut components = vec![];
        let mut numeric = vec![];
        for field in &spec.retained_fields {
            let input = Grouping::Interaction(vec![*field]);
            let first = group_value(index[&samples[0].key], &input);
            if samples
                .iter()
                .all(|s| group_value(index[&s.key], &input) == first)
            {
                if let Some(GroupValue::Interaction(mut parts)) = first {
                    components.append(&mut parts);
                }
            } else {
                message(
                    diagnostics,
                    &format!(
                        "Distribution dropped varying source aesthetic field {}.",
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
                numeric.push((input.clone(), first));
            } else {
                message(
                    diagnostics,
                    "Distribution dropped a varying evaluated numeric aesthetic.",
                );
            }
        }
        retained.insert(
            group.clone(),
            (
                if components.is_empty() {
                    GroupValue::All
                } else {
                    GroupValue::Interaction(components)
                },
                numeric,
            ),
        );
    }
    let mut output = Output {
        table,
        scope,
        rows: vec![],
        limit: limits.max_prepared_rows,
        memberships: BTreeMap::new(),
        retained,
    };
    let mut violins = vec![];
    let mut violin_rows = 0_usize;
    let panel_dots = if all.is_empty() {
        None
    } else if let DistributionKind::Dotplot {
        method: DotBinMethod::DotDensity,
        bin_positions_all: true,
        bin_width,
        ..
    } = spec.kind
    {
        let xs = all.iter().map(|s| s.value).collect::<Vec<_>>();
        let ws = all.iter().map(|s| s.weight).collect::<Vec<_>>();
        Some(kernel::dotdensity(
            &xs,
            Some(&ws),
            bin_width.unwrap_or_else(|| {
                (xs.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                    - xs.iter().copied().fold(f64::INFINITY, f64::min))
                    / 30.
            }),
        )?)
    } else {
        None
    };
    for (group, samples) in &groups {
        let values = samples.iter().map(|s| s.value).collect::<Vec<_>>();
        let ws = samples.iter().map(|s| s.weight).collect::<Vec<_>>();
        let weights = spec.weight.as_ref().map(|_| ws.as_slice());
        let (center, width) = center_width(samples, default_width, spec.width);
        let result = (|| -> ChartResult<()> {
            match &spec.kind {
                DistributionKind::Boxplot {
                    coefficient,
                    quantile_type,
                } => {
                    let b = kernel::boxplot(&values, weights, *coefficient, *quantile_type)?;
                    if b.nonunique {
                        message(
                            diagnostics,
                            "Weighted boxplot quantile solution may be nonunique.",
                        );
                    }
                    let outliers = b
                        .outliers
                        .iter()
                        .map(|i| StatOutlier {
                            value: samples[*i].value,
                            target: Target::Source(SourceRef {
                                dataset: table.input.dataset,
                                key: samples[*i].key,
                            }),
                        })
                        .collect();
                    output.push(
                        group,
                        samples,
                        "box",
                        false,
                        vec![
                            (StatField::X, Some(center)),
                            (StatField::Y, Some(b.middle)),
                            (StatField::Width, Some(width)),
                            (StatField::Lower, Some(b.lower)),
                            (StatField::Middle, Some(b.middle)),
                            (StatField::Upper, Some(b.upper)),
                            (StatField::WhiskerLower, Some(b.ymin)),
                            (StatField::WhiskerUpper, Some(b.ymax)),
                            (StatField::NotchLower, Some(b.notchlower)),
                            (StatField::NotchUpper, Some(b.notchupper)),
                            (StatField::RelativeWidth, Some(b.relvarwidth)),
                        ],
                        outliers,
                    )?;
                }
                DistributionKind::Density { controls, trim } => {
                    let density_range = if *trim {
                        values
                            .iter()
                            .fold([f64::INFINITY, f64::NEG_INFINITY], |r, v| {
                                [r[0].min(*v), r[1].max(*v)]
                            })
                    } else {
                        range
                    };
                    let d = kernel::density(
                        &values,
                        weights,
                        density_range[0],
                        density_range[1],
                        controls,
                        limits.max_vertices,
                    )?;
                    density_warnings(&d, diagnostics);
                    let fitted_samples = samples
                        .iter()
                        .filter(|s| {
                            controls.lower.is_none_or(|v| s.value >= v)
                                && controls.upper.is_none_or(|v| s.value <= v)
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    if d.too_few {
                        output.push(
                            group,
                            &fitted_samples,
                            "density/missing",
                            true,
                            vec![
                                (StatField::X, None),
                                (StatField::Y, None),
                                (StatField::Density, None),
                                (StatField::ScaledDensity, None),
                                (StatField::DensityCount, None),
                                (StatField::WeightedDensity, None),
                            ],
                            vec![],
                        )?;
                    }
                    for p in &d.points {
                        let mut v = density_values(p);
                        v.extend([(StatField::X, Some(p.x)), (StatField::Y, Some(p.density))]);
                        output.push(
                            group,
                            &fitted_samples,
                            &format!("density/{:016x}", p.x.to_bits()),
                            true,
                            v,
                            vec![],
                        )?;
                    }
                }
                DistributionKind::Violin {
                    controls,
                    trim,
                    quantiles,
                    drop,
                    ..
                } => {
                    let v = kernel::violin(
                        &values,
                        weights,
                        controls,
                        *trim,
                        quantiles,
                        limits.max_vertices,
                    )?;
                    density_warnings(&v.density, diagnostics);
                    if v.weighted_quantiles {
                        message(
                            diagnostics,
                            "Violin quantile markers use unweighted sample quantiles even when density weights are mapped.",
                        );
                    }
                    if v.density.too_few && !drop {
                        output.push(
                            group,
                            samples,
                            "violin/singleton",
                            true,
                            vec![
                                (StatField::X, Some(center)),
                                (StatField::Y, None),
                                (StatField::Width, Some(width)),
                            ],
                            vec![],
                        )?;
                    }
                    violin_rows = violin_rows.checked_add(v.points.len()).ok_or_else(|| {
                        error(DiagnosticCode::ResourceLimit, "Violin row budget overflow.")
                    })?;
                    if violin_rows > limits.max_prepared_rows {
                        return Err(error(
                            DiagnosticCode::ResourceLimit,
                            "Violin panel row budget exceeded.",
                        ));
                    }
                    violins.push((group.clone(), samples.clone(), center, width, v));
                }
                DistributionKind::Dotplot {
                    method,
                    bin_axis,
                    bin_width,
                    closed,
                    origin,
                    ..
                } => {
                    let bins = if let Some(panel) = &panel_dots {
                        let indices = samples
                            .iter()
                            .enumerate()
                            .map(|(i, s)| (s.key, i))
                            .collect::<BTreeMap<_, _>>();
                        panel
                            .iter()
                            .filter_map(|b| {
                                let members = b
                                    .members
                                    .iter()
                                    .filter_map(|i| indices.get(&all[*i].key).copied())
                                    .collect::<Vec<_>>();
                                (!members.is_empty()).then(|| kernel::DotBin {
                                    center: b.center,
                                    width: b.width,
                                    count: members.iter().map(|i| samples[*i].weight).sum(),
                                    members,
                                })
                            })
                            .collect()
                    } else {
                        match method {
                            DotBinMethod::DotDensity => kernel::dotdensity(
                                &values,
                                weights,
                                bin_width.unwrap_or((range[1] - range[0]) / 30.),
                            )?,
                            DotBinMethod::HistoDot => kernel::histodot(
                                &values, weights, range, *bin_width, *origin, *closed,
                            )?,
                        }
                    };
                    let maximum = bins.iter().map(|b| b.count.abs()).fold(0., f64::max);
                    for b in bins {
                        let members = b
                            .members
                            .iter()
                            .map(|i| samples[*i].clone())
                            .collect::<Vec<_>>();
                        let (x, y) = if *bin_axis == DotAxis::X {
                            (b.center, 0.)
                        } else {
                            (center, b.center)
                        };
                        output.push(
                            group,
                            &members,
                            &format!("dot/{:016x}", b.center.to_bits()),
                            false,
                            vec![
                                (StatField::X, Some(x)),
                                (StatField::Y, Some(y)),
                                (StatField::Width, Some(width)),
                                (StatField::BinWidth, Some(b.width)),
                                (StatField::WeightedCount, Some(b.count)),
                                (
                                    StatField::Proportion,
                                    (maximum > 0.).then_some(b.count / maximum),
                                ),
                            ],
                            vec![],
                        )?;
                    }
                }
            }
            Ok(())
        })();
        if let Err(e) = result {
            if e.code == DiagnosticCode::ResourceLimit {
                return Err(e);
            }
            counts.invalid_stat += samples.len();
            warning(
                policy,
                samples.len(),
                samples.iter().take(32).map(|s| s.key).collect(),
                format!("Distribution group could not be computed: {}", e.message),
                diagnostics,
            )?;
        }
    }
    if let DistributionKind::Violin { scale, .. } = &spec.kind {
        let mut estimates = violins.iter().map(|v| v.4.clone()).collect::<Vec<_>>();
        kernel::normalize_violin(&mut estimates, *scale);
        for ((group, samples, center, width, _), estimate) in violins.into_iter().zip(estimates) {
            for p in estimate.points {
                let mut values = density_values(&p.density);
                values.extend([
                    (StatField::X, Some(center)),
                    (StatField::Y, Some(p.density.x)),
                    (StatField::Width, Some(width)),
                    (StatField::ViolinWidth, Some(p.violinwidth)),
                    (StatField::QuantileFlag, p.quantile),
                ]);
                output.push(
                    &group,
                    &samples,
                    &format!("violin/{:016x}/{:?}", p.density.x.to_bits(), p.quantile),
                    true,
                    values,
                    vec![],
                )?;
            }
        }
    }
    let result = output.rows;
    table.schema = OutputSchema::Statistical {
        version: SchemaVersion::new(1),
        fields,
    };
    table.rows = PreparedRows::Statistical(result.into());
    Ok((spec.grouping.clone(), spec.space.clone()))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::*;
    use crate::transaction::*;
    use crate::{DatasetId, FieldId, SourceEpoch};
    const DATA: DatasetId = DatasetId::new(1);
    const SAMPLE: FieldId = FieldId::new(1);
    const POSITION: FieldId = FieldId::new(2);
    const WEIGHT: FieldId = FieldId::new(3);
    fn batch(rows: &[(u64, f64)]) -> NormalizedBatch {
        let schema = Arc::new(
            Schema::new(
                SchemaVersion::new(1),
                [SAMPLE, POSITION, WEIGHT]
                    .into_iter()
                    .map(|id| Field {
                        id,
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
        let columns = [
            rows.iter().map(|r| r.1).collect(),
            vec![1.; rows.len()],
            vec![1.; rows.len()],
        ]
        .into_iter()
        .map(|v| Column::new(ColumnValues::Float64(v), vec![true; rows.len()], None))
        .collect();
        NormalizedBatch::new(
            schema,
            rows.iter().map(|r| RowKey::new(r.0)).collect(),
            columns,
            DataLimits::default(),
        )
        .unwrap()
    }
    fn spec(kind: DistributionKind) -> DistributionSpec {
        DistributionSpec {
            training_range: None,
            retained_fields: vec![],
            retained_numeric: vec![],
            input: Numeric::Field(SAMPLE),
            position: Some(Numeric::Field(POSITION)),
            weight: None,
            grouping: Grouping::All,
            space: StatSpace::Data,
            position_space: StatSpace::Data,
            width: None,
            kind,
        }
    }
    fn evaluate(store: &DataStore, spec: &DistributionSpec) -> PreparedTable {
        evaluate_diagnostics(store, spec).0
    }
    fn evaluate_diagnostics(
        store: &DataStore,
        spec: &DistributionSpec,
    ) -> (PreparedTable, Vec<Diagnostic>) {
        let snapshot = store.snapshot();
        let snapshot = snapshot.get().unwrap();
        let data = snapshot.dataset(DATA).unwrap();
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
        let mut diagnostics = vec![];
        run(
            &mut table,
            data,
            spec,
            InvalidPolicy::Strict,
            "test",
            CompileLimits::default(),
            &mut PopulationCounts::default(),
            &mut diagnostics,
        )
        .unwrap();
        (table, diagnostics)
    }
    fn rows(t: &PreparedTable) -> &[StatisticalRow] {
        let PreparedRows::Statistical(rows) = &t.rows else {
            panic!()
        };
        rows
    }
    #[test]
    fn density_trim_and_singleton_placeholder_preserve_reference_contract() {
        let store = DataStore::new(
            SourceEpoch::new(1),
            vec![(DATA, batch(&[(1, 0.), (2, 1.), (3, 2.)]))],
            DataLimits::default(),
        )
        .unwrap();
        for trim in [false, true] {
            let mut spec = spec(DistributionKind::Density {
                trim,
                controls: DensityControls {
                    bandwidth: Bandwidth::Fixed(0.5),
                    n: 16,
                    ..Default::default()
                },
            });
            spec.training_range = Some([-10., 10.]);
            let table = evaluate(&store, &spec);
            assert_eq!(
                rows(&table).first().unwrap().value(&StatField::X),
                Some(if trim { 0. } else { -10. })
            );
            assert_eq!(
                rows(&table).last().unwrap().value(&StatField::X),
                Some(if trim { 2. } else { 10. })
            );
        }
        let singleton = DataStore::new(
            SourceEpoch::new(1),
            vec![(DATA, batch(&[(8, 2.)]))],
            DataLimits::default(),
        )
        .unwrap();
        let (table, diagnostics) = evaluate_diagnostics(
            &singleton,
            &spec(DistributionKind::Density {
                trim: false,
                controls: DensityControls::default(),
            }),
        );
        assert_eq!(rows(&table).len(), 1);
        assert_eq!(rows(&table)[0].count, 1);
        assert_eq!(rows(&table)[0].value(&StatField::X), None);
        assert_eq!(rows(&table)[0].value(&StatField::Density), None);
        assert!(matches!(rows(&table)[0].target, Target::Derived { .. }));
        assert!(!diagnostics.is_empty());
    }
    #[test]
    fn retains_only_proven_constant_group_aesthetics() {
        let store = DataStore::new(
            SourceEpoch::new(1),
            vec![(DATA, batch(&[(1, 0.), (2, 1.), (3, 2.)]))],
            DataLimits::default(),
        )
        .unwrap();
        let mut spec = spec(DistributionKind::Density {
            trim: false,
            controls: DensityControls {
                bandwidth: Bandwidth::Fixed(0.5),
                n: 16,
                ..Default::default()
            },
        });
        spec.retained_fields = vec![POSITION, SAMPLE];
        spec.retained_numeric = vec![Numeric::Literal(7.), Numeric::Field(SAMPLE)];
        let (table, diagnostics) = evaluate_diagnostics(&store, &spec);
        for row in rows(&table) {
            assert_eq!(row.retained.component(POSITION).unwrap().label(), "1");
            assert!(row.retained.component(SAMPLE).is_none());
            assert_eq!(row.retained_numeric, vec![(Numeric::Literal(7.), Some(7.))]);
        }
        assert_eq!(
            diagnostics
                .iter()
                .filter(|d| d.message.contains("dropped"))
                .count(),
            2
        );
        assert!(
            Arc::ptr_eq(&rows(&table)[0].members, &rows(&table)[1].members),
            "fit rows share compact source membership"
        );
    }
    #[test]
    fn observed_outliers_and_bounded_fit_members_are_distinct() {
        let input = [
            (1, 0.),
            (2, 0.),
            (3, 1.),
            (4, 2.),
            (5, 3.),
            (6, 3.),
            (7, 20.),
        ];
        let store = DataStore::new(
            SourceEpoch::new(1),
            vec![(DATA, batch(&input))],
            DataLimits::default(),
        )
        .unwrap();
        let b = evaluate(
            &store,
            &spec(DistributionKind::Boxplot {
                coefficient: 1.5,
                quantile_type: 7,
            }),
        );
        let row = &rows(&b)[0];
        assert_eq!(row.value(&StatField::Middle), Some(2.));
        assert_eq!(row.count, 7);
        assert_eq!(row.value(&StatField::Width), Some(0.75));
        assert!(matches!(row.target, Target::Aggregate { .. }));
        assert_eq!(row.outliers.len(), 1);
        assert_eq!(
            row.outliers[0].target,
            Target::Source(SourceRef {
                dataset: DATA,
                key: RowKey::new(7)
            })
        );
        let d = evaluate(
            &store,
            &spec(DistributionKind::Density {
                trim: false,
                controls: DensityControls {
                    bandwidth: Bandwidth::Fixed(0.5),
                    n: 16,
                    lower: Some(0.),
                    upper: Some(3.),
                    ..Default::default()
                },
            }),
        );
        for row in rows(&d) {
            assert!(matches!(row.target, Target::Derived { .. }));
            assert_eq!(row.count, 6);
            assert!(!row.members.contains(&RowKey::new(7)));
            assert_eq!(
                row.value(&StatField::DensityCount),
                row.value(&StatField::Density).map(|d| d * 6.)
            );
        }
    }
    #[test]
    fn distribution_updates_match_batch_on_actual_upsert() {
        let input = [(1, 0.), (2, 1.), (3, 2.), (4, 3.)];
        let mut store = DataStore::new(
            SourceEpoch::new(1),
            vec![(DATA, batch(&input))],
            DataLimits::default(),
        )
        .unwrap();
        let kinds = [
            DistributionKind::Boxplot {
                coefficient: 1.5,
                quantile_type: 7,
            },
            DistributionKind::Density {
                trim: false,
                controls: DensityControls {
                    bandwidth: Bandwidth::Fixed(0.5),
                    n: 16,
                    ..Default::default()
                },
            },
            DistributionKind::Violin {
                controls: DensityControls {
                    bandwidth: Bandwidth::Fixed(0.5),
                    ..Default::default()
                },
                trim: true,
                scale: ViolinScale::Area,
                quantiles: vec![0.25, 0.5, 0.75],
                drop: true,
            },
            DistributionKind::Dotplot {
                bin_axis: DotAxis::X,
                method: DotBinMethod::DotDensity,
                bin_width: Some(1.),
                bin_positions_all: false,
                closed: BinClosure::Right,
                origin: None,
            },
        ];
        let before = kinds
            .iter()
            .cloned()
            .map(|k| evaluate(&store, &spec(k)))
            .collect::<Vec<_>>();
        let version = store
            .snapshot()
            .get()
            .unwrap()
            .dataset(DATA)
            .unwrap()
            .version();
        let tx = Transaction {
            id: TransactionId::new("gg09-upsert").unwrap(),
            epoch: SourceEpoch::new(1),
            expected: vec![version],
            operations: vec![Operation {
                dataset: DATA,
                mutation: Mutation::UpsertByKey(batch(&[(3, 5.)])),
            }],
        };
        assert!(matches!(store.apply(tx), CommitOutcome::Applied(_)));
        let fresh = DataStore::new(
            SourceEpoch::new(1),
            vec![(DATA, batch(&[(1, 0.), (2, 1.), (3, 5.), (4, 3.)]))],
            DataLimits::default(),
        )
        .unwrap();
        for (i, kind) in kinds.into_iter().enumerate() {
            let spec = spec(kind);
            let updated = evaluate(&store, &spec);
            let batch = evaluate(&fresh, &spec);
            let values = |t: &PreparedTable| {
                rows(t)
                    .iter()
                    .map(|r| {
                        (
                            r.values.clone(),
                            r.members.clone(),
                            r.outliers.iter().map(|v| v.value).collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(values(&updated), values(&batch));
            assert_ne!(values(&updated), values(&before[i]));
        }
    }
}
