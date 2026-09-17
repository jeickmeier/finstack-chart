//! ggplot position controls; legacy position variants retain their original contracts.
use super::compiler::EncodedRow;
use super::*;
use crate::ChartResult;
use std::collections::{BTreeMap, BTreeSet};

/// Whether collision slots preserve their combined or individual width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DodgePreserve {
    /// Recompute the group count at each collision.
    #[default]
    Total,
    /// Use the maximum collision population across the panel.
    Single,
}
/// Reference dodge controls in independent-axis calculation units.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotDodgeSpec {
    /// Collision width; absent uses the first nonzero encoded interval width.
    pub width: Option<f64>,
    /// Total or single element width preservation.
    pub preserve: DodgePreserve,
    /// Reverse group order.
    pub reverse: bool,
    /// Fraction of each dodge2 interval removed as padding.
    pub padding: f64,
    /// Preparation-only maximum shared by existing facet panels; never serialized.
    #[doc(hidden)]
    #[serde(skip)]
    pub resolved_count: Option<usize>,
}
impl Default for GgplotDodgeSpec {
    fn default() -> Self {
        Self {
            width: None,
            preserve: DodgePreserve::Total,
            reverse: false,
            padding: 0.1,
            resolved_count: None,
        }
    }
}
/// Reference separate-sign stack/fill controls. Orientation is the layer orientation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotStackSpec {
    /// Divide sign-side cumulative heights by their absolute total.
    pub fill: bool,
    /// Ascending group order instead of the reference descending default.
    pub reverse: bool,
    /// Point/text anchor within the resulting interval; interval geometry retains both bounds.
    pub vjust: f64,
}
impl Default for GgplotStackSpec {
    fn default() -> Self {
        Self {
            fill: false,
            reverse: false,
            vjust: 1.,
        }
    }
}
/// Constant displacement in axis calculation units, including category steps.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NudgeSpec {
    /// Horizontal displacement.
    pub x: f64,
    /// Vertical displacement.
    pub y: f64,
}
/// Dodge followed by stable-key jitter, using the existing FNV-1a/SplitMix64 policy.
/// This explicitly does not emulate R's mutable RNG or row-order draw sequence.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JitterDodgeSpec {
    /// Dodge controls (padding does not apply to ordinary dodge).
    pub dodge: GgplotDodgeSpec,
    /// Data-unit half widths; x divides by maximum collision population plus two.
    pub jitter: JitterSpec,
    /// Use 0.4 times the independent-axis resolution until displacement is explicit.
    pub auto_width: bool,
    /// Preparation-only maximum collision group count; never serialized.
    #[doc(hidden)]
    #[serde(skip)]
    pub resolved_dodge_count: Option<usize>,
    /// Preparation-only whole-layer independent-axis resolution.
    #[doc(hidden)]
    #[serde(skip)]
    pub resolved_resolution: Option<f64>,
}

pub(super) fn validate(position: &Position) -> ChartResult<()> {
    let valid = match position {
        Position::Nudge(s) => s.x.is_finite() && s.y.is_finite(),
        Position::GgplotStack(s) => s.vjust.is_finite(),
        Position::GgplotDodge(s) | Position::GgplotDodge2(s) => valid_dodge(s),
        Position::JitterDodge(s) => {
            valid_dodge(&s.dodge)
                && s.jitter.units == JitterUnits::Data
                && s.jitter.x.is_finite()
                && s.jitter.y.is_finite()
                && s.jitter.x >= 0.
                && s.jitter.y >= 0.
        }
        _ => true,
    };
    if valid {
        Ok(())
    } else {
        Err(error(
            DiagnosticCode::Validation,
            "Position controls require finite widths/displacements, positive dodge width, padding in [0,1], and data-unit jitter.",
        ))
    }
}
fn valid_dodge(s: &GgplotDodgeSpec) -> bool {
    s.width.is_none_or(|w| w.is_finite() && w > 0.)
        && s.padding.is_finite()
        && (0. ..=1.).contains(&s.padding)
}
fn shift(r: &mut EncodedRow, dx: f64, dy: f64) -> ChartResult<()> {
    for (v, d) in [
        (&mut r.x, dx),
        (&mut r.x2, dx),
        (&mut r.y, dy),
        (&mut r.y2, dy),
    ] {
        if let Some(v) = v {
            *v += d;
            if !v.is_finite() {
                return Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Position coordinate overflow.",
                ));
            }
        }
    }
    Ok(())
}
pub(super) fn apply(position: &Position, rows: &mut [EncodedRow]) -> ChartResult<()> {
    match position {
        Position::Nudge(s) => {
            for r in rows {
                shift(r, s.x, s.y)?;
            }
        }
        Position::GgplotStack(s) => stack(rows, s)?,
        Position::GgplotDodge(s) => dodge(rows, s, false)?,
        Position::GgplotDodge2(s) => dodge(rows, s, true)?,
        Position::JitterDodge(s) => {
            let n = collision_population(position, rows);
            let width = if s.auto_width {
                0.4 * s
                    .resolved_resolution
                    .unwrap_or_else(|| resolution(rows.iter().filter_map(|r| r.x).collect()))
            } else {
                s.jitter.x
            };
            dodge(rows, &s.dodge, false)?;
            let mut jitter = s.jitter.clone();
            jitter.x = width / (s.resolved_dodge_count.unwrap_or(n) as f64 + 2.);
            for r in rows {
                let (dx, dy) = super::positions::jitter(&jitter, &r.target, &r.group);
                shift(r, dx, dy)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn stack(rows: &mut [EncodedRow], s: &GgplotStackSpec) -> ChartResult<()> {
    if !s.fill && s.vjust == 1. {
        let mut xs = BTreeSet::new();
        if rows
            .iter()
            .filter_map(|r| r.x)
            .all(|x| xs.insert(if x == 0. { 0 } else { x.to_bits() }))
        {
            return Ok(());
        }
    }
    let mut cells = BTreeMap::<(u64, bool), Vec<usize>>::new();
    for (i, r) in rows.iter().enumerate() {
        if let (Some(x), Some(y)) = (r.x, r.y) {
            cells
                .entry((if x == 0. { 0 } else { x.to_bits() }, y < 0.))
                .or_default()
                .push(i);
        }
    }
    for indexes in cells.values_mut() {
        indexes.sort_by(|a, b| {
            let cmp = rows[*a].group.cmp(&rows[*b].group);
            if s.reverse { cmp } else { cmp.reverse() }
        });
        let total = indexes
            .iter()
            .map(|i| rows[*i].y.unwrap())
            .sum::<f64>()
            .abs();
        if !total.is_finite() {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Stack total overflow.",
            ));
        }
        let divisor = if s.fill && total > f64::EPSILON.sqrt() {
            total
        } else {
            1.
        };
        let mut cursor = 0.;
        for &i in indexes.iter() {
            let r = &mut rows[i];
            let next = cursor + r.y.unwrap() / divisor;
            let (lo, hi) = (cursor.min(next), cursor.max(next));
            if r.y2.is_some() {
                r.y = Some(hi);
                r.y2 = Some(lo);
            } else {
                r.y = Some((1. - s.vjust) * lo + s.vjust * hi);
            }
            cursor = next;
        }
    }
    Ok(())
}
#[derive(Clone)]
struct Interval {
    row: usize,
    center: f64,
    lo: f64,
    hi: f64,
    interval: bool,
}
fn dodge(rows: &mut [EncodedRow], s: &GgplotDodgeSpec, second: bool) -> ChartResult<()> {
    let width = s
        .width
        .or_else(|| {
            rows.iter().find_map(|r| match (r.x, r.x2) {
                (Some(a), Some(b)) if a != b => Some((b - a).abs()),
                _ => None,
            })
        })
        .unwrap_or(0.);
    let mut intervals: Vec<_> = rows
        .iter()
        .enumerate()
        .filter_map(|(i, r)| {
            r.x.map(|x| {
                let interval = r.x2.is_some_and(|v| v != x);
                let (lo, hi) = if interval {
                    (x.min(r.x2.unwrap()), x.max(r.x2.unwrap()))
                } else {
                    (x - width / 2., x + width / 2.)
                };
                Interval {
                    row: i,
                    center: (lo + hi) / 2.,
                    lo,
                    hi,
                    interval: r.x2.is_some(),
                }
            })
        })
        .collect();
    intervals.sort_by(|a, b| {
        (if second {
            a.center.total_cmp(&b.center)
        } else {
            a.lo.total_cmp(&b.lo)
        })
        .then_with(|| {
            let cmp = rows[a.row].group.cmp(&rows[b.row].group);
            if s.reverse { cmp.reverse() } else { cmp }
        })
    });
    let mut cells: Vec<Vec<Interval>> = vec![];
    let mut end = f64::NEG_INFINITY;
    for i in intervals {
        let new = if second {
            i.lo > end || (i.lo == end && i.hi != i.lo)
        } else {
            cells.last().is_none_or(|c| c[0].lo != i.lo)
        };
        if new || cells.is_empty() {
            cells.push(vec![]);
            end = i.hi;
        } else {
            end = end.max(i.hi);
        }
        cells.last_mut().unwrap().push(i);
    }
    let maximum = cells
        .iter()
        .map(|c| {
            if second {
                c.len()
            } else {
                c.iter()
                    .map(|i| &rows[i.row].group)
                    .collect::<BTreeSet<_>>()
                    .len()
            }
        })
        .max()
        .unwrap_or(1)
        .max(1);
    let any_overlap = cells.iter().any(|c| c.len() > 1);
    for cell in cells {
        let groups: Vec<_> = cell
            .iter()
            .map(|i| rows[i.row].group.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let n = if s.preserve == DodgePreserve::Single {
            s.resolved_count.unwrap_or(maximum)
        } else if second {
            cell.len()
        } else {
            groups.len()
        }
        .max(1) as f64;
        let maxwidth = cell.iter().map(|i| i.hi - i.lo).fold(0_f64, f64::max);
        let center = (cell.iter().map(|i| i.lo).fold(f64::INFINITY, f64::min)
            + cell.iter().map(|i| i.hi).fold(f64::NEG_INFINITY, f64::max))
            / 2.;
        let mut cursor = center - cell.iter().map(|i| (i.hi - i.lo) / n).sum::<f64>() / 2.;
        for i in cell {
            let (x, w) = if second {
                let w = (i.hi - i.lo) / n;
                let x = cursor + w / 2.;
                cursor += w;
                (x, w * if any_overlap { 1. - s.padding } else { 1. })
            } else if n == 1. {
                (i.center, i.hi - i.lo)
            } else {
                let rank = groups.binary_search(&rows[i.row].group).unwrap();
                let rank = if s.reverse {
                    n as usize - 1 - rank
                } else {
                    rank
                };
                (
                    i.center + width * ((rank as f64 + 0.5) / n - 0.5),
                    maxwidth / n,
                )
            };
            if !x.is_finite() || !w.is_finite() {
                return Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Dodge interval overflow.",
                ));
            }
            let r = &mut rows[i.row];
            if i.interval {
                r.x = Some(x - w / 2.);
                r.x2 = Some(x + w / 2.);
            } else {
                r.x = Some(x);
            }
        }
    }
    Ok(())
}

pub(super) fn resolution(mut values: Vec<f64>) -> f64 {
    values.retain(|v| v.is_finite());
    values.sort_by(f64::total_cmp);
    values.dedup();
    values
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|d| *d > 0.)
        .reduce(f64::min)
        .unwrap_or(1.)
}
pub(super) fn collision_population(position: &Position, rows: &[EncodedRow]) -> usize {
    let second = matches!(position, Position::GgplotDodge2(_));
    let mut cells = BTreeMap::<u64, Vec<&Option<GroupValue>>>::new();
    for r in rows {
        if let Some(x) = r.x {
            let x = if second || matches!(position, Position::JitterDodge(_)) {
                r.x2.map_or(x, |b| (x + b) / 2.)
            } else {
                r.x2.map_or(x, |b| x.min(b))
            };
            cells
                .entry(if x == 0. { 0 } else { x.to_bits() })
                .or_default()
                .push(&r.group);
        }
    }
    cells
        .values()
        .map(|v| {
            if second {
                v.len()
            } else {
                v.iter().copied().collect::<BTreeSet<_>>().len()
            }
        })
        .max()
        .unwrap_or(1)
}
pub(super) fn resolve_population(position: &mut Position, count: usize, resolution: f64) {
    match position {
        Position::GgplotDodge(s) | Position::GgplotDodge2(s) => s.resolved_count = Some(count),
        Position::JitterDodge(s) => {
            s.dodge.resolved_count = Some(count);
            s.resolved_dodge_count = Some(count);
            s.resolved_resolution = Some(resolution);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{SourceRef, Target};
    use crate::{DatasetId, RowKey};
    fn row(x: f64, y: f64, group: i64) -> EncodedRow {
        EncodedRow {
            geo_feature: None,
            stat_outliers: vec![],
            outlier_anchor_y: None,
            missing_aesthetics: 0,
            recipe_values: BTreeMap::new(),
            values: BTreeMap::new(),
            shape: None,
            x: Some(x),
            y: Some(y),
            x2: None,
            y2: None,
            color: None,
            fill: None,
            stroke: None,
            opacity: None,
            alpha: None,
            stroke_width: None,
            low: None,
            high: None,
            size: None,
            group: Some(GroupValue::Int(group)),
            ordinal: group as u64,
            target: Target::Source(SourceRef {
                dataset: DatasetId::new(1),
                key: RowKey::new(group as u64),
            }),
            key: Some(RowKey::new(group as u64)),
        }
    }
    #[test]
    fn reference_position_controls() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/position-controls.json"
        ))
        .unwrap();
        let cases = f["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 32);
        for c in cases {
            let kind = c["kind"].as_str().unwrap();
            let mut rows: Vec<_> = c["input"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| {
                    let mut r = row(
                        v["x"].as_f64().unwrap(),
                        v["y"].as_f64().unwrap(),
                        v["group"].as_i64().unwrap(),
                    );
                    if kind != "stack" {
                        r.x = v["xmin"].as_f64();
                        r.x2 = v["xmax"].as_f64();
                    }
                    r
                })
                .collect();
            let position = if kind == "stack" {
                Position::GgplotStack(GgplotStackSpec {
                    fill: c["fill"].as_bool().unwrap(),
                    reverse: c["reverse"].as_bool().unwrap(),
                    vjust: c["vjust"].as_f64().unwrap(),
                })
            } else {
                let s = GgplotDodgeSpec {
                    width: Some(0.8),
                    preserve: if c["preserve"] == "single" {
                        DodgePreserve::Single
                    } else {
                        DodgePreserve::Total
                    },
                    reverse: c["reverse"].as_bool().unwrap(),
                    padding: 0.1,
                    resolved_count: None,
                };
                if kind == "dodge" {
                    Position::GgplotDodge(s)
                } else {
                    Position::GgplotDodge2(s)
                }
            };
            if c["panels"] == true {
                let mut position = position.clone();
                resolve_population(&mut position, c["count"].as_u64().unwrap() as usize, 1.);
                let (first, second) = rows.split_at_mut(2);
                apply(&position, first).unwrap();
                apply(&position, second).unwrap();
            } else {
                apply(&position, &mut rows).unwrap();
            }
            for (r, v) in rows.iter().zip(c["output"].as_array().unwrap()) {
                let fields = if kind == "stack" {
                    vec![(r.y.unwrap(), v["y"].as_f64().unwrap())]
                } else {
                    vec![
                        (r.x.unwrap(), v["xmin"].as_f64().unwrap()),
                        (r.x2.unwrap(), v["xmax"].as_f64().unwrap()),
                    ]
                };
                for (a, b) in fields {
                    assert!((a - b).abs() < 1e-12, "{c}: {a} != {b}");
                }
            }
        }
    }
    #[test]
    fn reference_jitter_dodge_resolution_and_collision_setup() {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/jitter-dodge-setup.json"
        ))
        .unwrap();
        for c in f["cases"].as_array().unwrap() {
            let rows = c["x"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, x)| row(x.as_f64().unwrap(), 1., i as i64 + 1))
                .collect::<Vec<_>>();
            let s = Position::JitterDodge(JitterDodgeSpec {
                dodge: Default::default(),
                jitter: JitterSpec {
                    seed: 42,
                    x: 0.3,
                    y: 0.,
                    units: JitterUnits::Data,
                },
                auto_width: !c["explicit"].as_bool().unwrap(),
                resolved_dodge_count: None,
                resolved_resolution: None,
            });
            let n = collision_population(&s, &rows);
            let width = if c["explicit"] == true {
                0.3
            } else {
                0.4 * resolution(rows.iter().filter_map(|r| r.x).collect())
            } / (n as f64 + 2.);
            assert!((width - c["width"].as_f64().unwrap()).abs() < 1e-12, "{c}");
            let expected = rows
                .iter()
                .map(|r| {
                    let (dx, _) = super::super::positions::jitter(
                        &JitterSpec {
                            seed: 42,
                            x: c["width"].as_f64().unwrap(),
                            y: 0.,
                            units: JitterUnits::Data,
                        },
                        &r.target,
                        &r.group,
                    );
                    r.x.unwrap() + dx
                })
                .collect::<Vec<_>>();
            let mut rows = rows;
            apply(&s, &mut rows).unwrap();
            for (r, x) in rows.iter().zip(expected) {
                assert!((r.x.unwrap() - x).abs() < 1e-12, "{c}");
            }
        }
    }
    #[test]
    fn nudge_and_jitter_dodge_stable_identity() {
        let original = || vec![row(1., 1., 1), row(1., 2., 2), row(2., 3., 3)];
        let mut nudged = original();
        apply(&Position::Nudge(NudgeSpec { x: 2., y: -1. }), &mut nudged).unwrap();
        assert_eq!(nudged[0].x, Some(3.));
        assert_eq!(nudged[0].y, Some(0.));
        let s = Position::JitterDodge(JitterDodgeSpec {
            resolved_dodge_count: None,
            resolved_resolution: None,
            auto_width: false,
            dodge: GgplotDodgeSpec {
                width: Some(0.8),
                ..Default::default()
            },
            jitter: JitterSpec {
                seed: 42,
                x: 0.4,
                y: 0.2,
                units: JitterUnits::Data,
            },
        });
        let mut a = original();
        let mut b = original();
        b.reverse();
        apply(&s, &mut a).unwrap();
        apply(&s, &mut b).unwrap();
        b.reverse();
        for (x, y) in a.iter().zip(&b) {
            assert_eq!(x.x, y.x);
            assert_eq!(x.y, y.y);
        }
    }
}
