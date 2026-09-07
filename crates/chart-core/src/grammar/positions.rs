use super::compiler::EncodedRow;
use super::*;
use crate::provenance::Target;
use crate::{ChartResult, DiagnosticCode};
use std::collections::{BTreeMap, BTreeSet};

fn order(order: &[GroupValue], limits: CompileLimits) -> ChartResult<()> {
    if order.len() > limits.max_groups {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Position group order exceeds budget.",
        ));
    }
    if order.is_empty() || order.iter().collect::<BTreeSet<_>>().len() != order.len() {
        return Err(error(
            DiagnosticCode::Validation,
            "Position order must be nonempty and contain distinct stable groups.",
        ));
    }
    Ok(())
}
fn additive_space(space: &ValueSpace) -> bool {
    match space {
        ValueSpace::Data => true,
        ValueSpace::Transformed { input, transform } => {
            transform.offset == 0. && additive_space(input)
        }
        _ => false,
    }
}
pub(super) fn validate(
    layer: &Layer,
    domains: &DomainContributions,
    limits: CompileLimits,
) -> ChartResult<()> {
    match &layer.position {
        Position::Identity => {}
        Position::Stack(s) => {
            order(&s.order, limits)?;
            if !matches!(layer.geom, Geom::Rectangle | Geom::Rule)
                || !domains.y_space.as_ref().is_none_or(additive_space)
            {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Additive stacks require interval geometry and zero-preserving linear numeric height encodings.",
                ));
            }
        }
        Position::Dodge(s) => {
            order(&s.order, limits)?;
            if !s.width.is_finite()
                || s.width <= 0.
                || s.width > 1.
                || !matches!(domains.x_space, Some(ValueSpace::Categorical { .. }))
            {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Dodge requires a categorical x band and width in (0,1].",
                ));
            }
        }
        Position::Jitter(s) => {
            if !s.x.is_finite() || !s.y.is_finite() || s.x < 0. || s.y < 0. {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Jitter half-widths must be finite and nonnegative.",
                ));
            }
            if s.units == JitterUnits::Data
                && ((s.x != 0.
                    && !matches!(
                        domains.x_space,
                        None | Some(ValueSpace::Data | ValueSpace::Transformed { .. })
                    ))
                    || (s.y != 0.
                        && !matches!(
                            domains.y_space,
                            None | Some(ValueSpace::Data | ValueSpace::Transformed { .. })
                        )))
            {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Data jitter requires numeric calculation coordinates; use display units for categories/timestamps.",
                ));
            }
        }
    }
    Ok(())
}
fn numeric_key(v: f64) -> u64 {
    if v == 0. { 0 } else { v.to_bits() }
}
pub(super) fn apply(
    layer: &Layer,
    domains: &DomainContributions,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
) -> ChartResult<()> {
    validate(layer, domains, limits)?;
    let ordered = match &layer.position {
        Position::Stack(s) => Some(&s.order),
        Position::Dodge(s) => Some(&s.order),
        _ => None,
    };
    if let Some(order) = ordered {
        if order.len() > limits.max_groups {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Position group order exceeds budget.",
            ));
        }
        if rows
            .iter()
            .filter_map(|r| r.group.as_ref())
            .any(|g| !order.contains(g))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Position group is absent from declared order.",
            ));
        }
    }
    match &layer.position {
        Position::Dodge(_) => {
            if matches!(layer.geom, Geom::Rectangle | Geom::Rule)
                && rows
                    .iter()
                    .any(|r| matches!((r.x,r.x2),(Some(x),Some(x2)) if x!=x2))
            {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Dodged intervals must address the same category at both horizontal endpoints.",
                ));
            }
        }
        Position::Stack(s) => {
            let mut cells: BTreeMap<(u64, u64), Vec<usize>> = BTreeMap::new();
            for (i, r) in rows.iter().enumerate() {
                if let (Some(x), Some(x2), Some(_), Some(base), Some(_)) =
                    (r.x, r.x2, r.y, r.y2, &r.group)
                {
                    if base != 0. {
                        return Err(error(
                            DiagnosticCode::Validation,
                            "Stack input baseline must be explicit zero; y is signed height.",
                        ));
                    }
                    if r.size.is_some_and(|v| v <= 0.) {
                        continue;
                    }
                    cells
                        .entry((numeric_key(x), numeric_key(x2)))
                        .or_default()
                        .push(i);
                }
            }
            for indexes in cells.values_mut() {
                indexes.sort_by(|a, b| {
                    let a = &rows[*a];
                    let b = &rows[*b];
                    s.order
                        .iter()
                        .position(|g| Some(g) == a.group.as_ref())
                        .cmp(&s.order.iter().position(|g| Some(g) == b.group.as_ref()))
                        .then_with(|| {
                            stable_key(&a.target, &a.group).cmp(&stable_key(&b.target, &b.group))
                        })
                });
                let pos_scale = indexes
                    .iter()
                    .map(|i| rows[*i].y.unwrap().max(0.))
                    .fold(0_f64, f64::max);
                let neg_scale = indexes
                    .iter()
                    .map(|i| (-rows[*i].y.unwrap()).max(0.))
                    .fold(0_f64, f64::max);
                let pos_total: f64 = indexes
                    .iter()
                    .map(|i| {
                        if pos_scale > 0. {
                            rows[*i].y.unwrap().max(0.) / pos_scale
                        } else {
                            0.
                        }
                    })
                    .sum();
                let neg_total: f64 = indexes
                    .iter()
                    .map(|i| {
                        if neg_scale > 0. {
                            (-rows[*i].y.unwrap()).max(0.) / neg_scale
                        } else {
                            0.
                        }
                    })
                    .sum();
                let (mut positive, mut negative) = (0., 0.);
                for i in indexes {
                    let r = &mut rows[*i];
                    let mut height = r.y.unwrap();
                    if s.normalize {
                        height = if height > 0. {
                            (height / pos_scale) / pos_total
                        } else if height < 0. {
                            (height / neg_scale) / neg_total
                        } else {
                            0.
                        };
                    }
                    let cursor = if height < 0. {
                        &mut negative
                    } else {
                        &mut positive
                    };
                    let next = *cursor + height;
                    if !next.is_finite() {
                        return Err(error(
                            DiagnosticCode::PrecisionLoss,
                            "Stack endpoint is not representable.",
                        ));
                    }
                    r.y2 = Some(*cursor);
                    r.y = Some(next);
                    *cursor = next;
                }
            }
        }
        Position::Jitter(s) if s.units == JitterUnits::Data => {
            for r in rows {
                let (dx, dy) = jitter(s, &r.target, &r.group);
                for (value, delta) in [
                    (&mut r.x, dx),
                    (&mut r.x2, dx),
                    (&mut r.y, dy),
                    (&mut r.y2, dy),
                ] {
                    if let Some(v) = value {
                        *v += delta;
                        if !v.is_finite() {
                            return Err(error(
                                DiagnosticCode::PrecisionLoss,
                                "Jitter coordinate is not representable.",
                            ));
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
fn stable_key(target: &Target, group: &Option<GroupValue>) -> String {
    // Deliberate stable encoding excludes changing membership and input revisions.
    let target = match target {
        Target::Source(s) => format!("source:{}/{}", s.dataset.get(), s.key.get()),
        Target::Aggregate { id, group, .. } => format!("aggregate:{}/{group}", id.get()),
        Target::Derived {
            id,
            model,
            model_version,
            ..
        } => format!("derived:{}/{}/{model}", id.get(), model_version.get()),
    };
    format!("{target}/{group:?}")
}
/// Version-one FNV-1a key hash followed by SplitMix64, no process-random hashing.
pub(crate) fn jitter(spec: &JitterSpec, target: &Target, group: &Option<GroupValue>) -> (f64, f64) {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in stable_key(target, group).bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    fn sample(mut z: u64) -> f64 {
        z = z.wrapping_add(0x9e3779b97f4a7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^= z >> 31;
        (z >> 11) as f64 / (1_u64 << 53) as f64 * 2. - 1.
    }
    (
        sample(hash ^ spec.seed) * spec.x,
        sample(hash ^ spec.seed ^ 0xd1b54a32d192ed03) * spec.y,
    )
}

// Normalization produces dimensionless fractions, independent of input height units.
pub(super) fn output_space(layer: &Layer, domains: &mut DomainContributions) {
    if matches!(
        &layer.position,
        Position::Stack(StackSpec {
            normalize: true,
            ..
        })
    ) {
        domains.y_space = Some(ValueSpace::Data);
    }
}
