//! Exact per-chunk contributions and reversible membership counts for explicit bins.
//! Publishing immutable membership vectors still costs O(retained members); value projection
//! and bin classification only revisit new/replaced chunks. Other statistics remain batch.
use super::*;
use crate::data::{DataChunk, DatasetSnapshot, RowView};
use crate::{ChartResult, DiagnosticCode, RowKey};
use std::collections::{BTreeMap, BTreeSet};

/// Work performed by the last compiler preparation, not elapsed time or an RSS measurement.
#[derive(serde::Serialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StatUpdateMetrics {
    /// Rows whose numeric/group values were evaluated by explicit-bin preparation.
    pub evaluated_rows: usize,
    /// Rows whose immutable chunk contributions were reused.
    pub reused_rows: usize,
    /// Explicit-bin operations with compatible previous contribution state.
    pub updated_operations: usize,
    /// Explicit-bin operations built from scratch (including unsupported incremental scopes).
    pub batch_operations: usize,
    /// Current cached immutable chunks across active explicit-bin operations.
    pub retained_chunks: usize,
}
#[derive(Clone, Debug)]
struct Contribution {
    key: RowKey,
    group: Option<GroupValue>,
    bin: Option<usize>,
    invalid: bool,
    below: bool,
    above: bool,
}
struct Group {
    observations: usize,
    bins: Vec<BTreeSet<RowKey>>,
}
#[derive(Default)]
pub(super) struct Accumulator {
    groups: BTreeMap<GroupValue, Group>,
    invalid: BTreeSet<RowKey>,
    below: usize,
    above: usize,
}
impl Accumulator {
    fn apply(
        &mut self,
        c: &Contribution,
        add: bool,
        n: usize,
        limits: CompileLimits,
    ) -> ChartResult<()> {
        if let Some(group) = &c.group {
            if add && !self.groups.contains_key(group) {
                if self.groups.len() >= limits.max_groups
                    || (self.groups.len() + 1).saturating_mul(n) > limits.max_prepared_rows
                {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Explicit-bin group/output budget exceeded.",
                    ));
                }
                self.groups.insert(
                    group.clone(),
                    Group {
                        observations: 0,
                        bins: (0..n).map(|_| BTreeSet::new()).collect(),
                    },
                );
            }
            let g = self
                .groups
                .get_mut(group)
                .expect("removal contribution has a retained group");
            if add {
                g.observations += 1;
                if let Some(bin) = c.bin {
                    g.bins[bin].insert(c.key);
                }
            } else {
                g.observations -= 1;
                if let Some(bin) = c.bin {
                    g.bins[bin].remove(&c.key);
                }
                if g.observations == 0 {
                    self.groups.remove(group);
                }
            }
        }
        if c.invalid {
            if add {
                self.invalid.insert(c.key);
            } else {
                self.invalid.remove(&c.key);
            }
        }
        if c.below {
            if add {
                self.below += 1;
            } else {
                self.below -= 1;
            }
        }
        if c.above {
            if add {
                self.above += 1;
            } else {
                self.above -= 1;
            }
        }
        Ok(())
    }
    pub(super) fn counts(&self) -> (usize, usize, usize) {
        (self.invalid.len(), self.below, self.above)
    }
    pub(super) fn invalid_samples(&self, rows: &[SourceRow]) -> Vec<RowKey> {
        if self.invalid.is_empty() {
            return vec![];
        }
        rows.iter()
            .filter(|r| self.invalid.contains(&r.key))
            .take(32)
            .map(|r| r.key)
            .collect()
    }
    pub(super) fn groups(
        &self,
        data: &DatasetSnapshot,
        spec: &BinSpec,
    ) -> Vec<(GroupValue, Vec<Vec<RowKey>>)> {
        let mut groups: Vec<_> = self
            .groups
            .iter()
            .map(|(group, g)| {
                (
                    group.clone(),
                    g.bins.iter().map(|b| b.iter().copied().collect()).collect(),
                )
            })
            .collect();
        if spec.grouping == Grouping::All && groups.is_empty() {
            groups.push((GroupValue::All, vec![vec![]; spec.edges.len() - 1]));
        }
        if let Grouping::Field(field) = &spec.grouping
            && let Some(order) = data.categories(*field)
        {
            let ranks: BTreeMap<_, _> = order
                .iter()
                .enumerate()
                .map(|(i, label)| (label, i))
                .collect();
            groups.sort_by_key(|(g, _)| match g {
                GroupValue::Text(label) => ranks.get(label).copied().unwrap_or(usize::MAX),
                _ => usize::MAX,
            });
        }
        groups
    }
}
fn evaluate(row: RowView<'_>, spec: &BinSpec, edges: &[f64]) -> Contribution {
    let group = stats::group_value(row, &spec.grouping);
    let value = stats::number(row, &spec.input).and_then(|v| match &spec.space {
        StatSpace::Data => Some(v),
        StatSpace::Transformed(t) => {
            let v = t.factor * v + t.offset;
            v.is_finite().then_some(v)
        }
    });
    let invalid = group.is_none() || value.is_none();
    let n = spec.edges.len() - 1;
    let below = !invalid && value.is_some_and(|v| v < edges[0]);
    let above = !invalid && value.is_some_and(|v| v > edges[n]);
    let bin = if invalid || ((below || above) && spec.outliers != OutlierPolicy::Overflow) {
        None
    } else {
        value.map(|v| {
            edges
                .partition_point(|edge| {
                    if spec
                        .ggplot
                        .as_ref()
                        .is_some_and(|s| s.closed == BinClosure::Right)
                    {
                        *edge < v
                    } else {
                        *edge <= v
                    }
                })
                .saturating_sub(1)
                .min(n - 1)
        })
    };
    Contribution {
        key: row.key(),
        group,
        bin,
        invalid,
        below,
        above,
    }
}
struct CachedChunk {
    chunk: DataChunk,
    contributions: Vec<Contribution>,
}
struct Entry {
    dataset: crate::DatasetId,
    schema: std::sync::Arc<crate::data::Schema>,
    spec: BinSpec,
    limits: CompileLimits,
    chunks: BTreeMap<usize, CachedChunk>,
    accumulated: Accumulator,
}
/// Entries exist only for operations used by the latest definition/prepare attempt.
#[derive(Default)]
pub(super) struct BinCache {
    entries: BTreeMap<String, Entry>,
    visited: BTreeSet<String>,
    row_capacity: usize,
    pub metrics: StatUpdateMetrics,
}
impl BinCache {
    pub fn begin(&mut self, row_capacity: usize) {
        self.visited.clear();
        self.row_capacity = row_capacity;
        self.metrics = StatUpdateMetrics::default();
    }
    pub fn finish(&mut self) {
        self.entries.retain(|key, _| self.visited.contains(key));
        self.metrics.retained_chunks = self.entries.values().map(|e| e.chunks.len()).sum();
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    // The returned owned accumulator is restored only after a successful materialization.
    // Invalid/budget failures drop the candidate, so no partial contribution update can be reused.
    pub fn run(
        &mut self,
        data: &DatasetSnapshot,
        rows: &[SourceRow],
        spec: &BinSpec,
        scope: &str,
        eligible: bool,
        limits: CompileLimits,
    ) -> ChartResult<Accumulator> {
        let fuzzy = spec
            .ggplot
            .as_ref()
            .map(|s| super::ggplot_stats::fuzzy_edges(&spec.edges, s.closed));
        let edges = fuzzy.as_deref().unwrap_or(&spec.edges);
        let cached_elsewhere: usize = self
            .entries
            .iter()
            .filter(|(key, _)| key.as_str() != scope)
            .map(|(_, e)| {
                e.chunks
                    .values()
                    .map(|c| c.contributions.len())
                    .sum::<usize>()
            })
            .sum();
        if !eligible
            || data.len() > self.row_capacity.saturating_sub(cached_elsewhere)
            || rows.len() != data.len()
            || !rows.iter().zip(data.rows()).all(|(a, b)| a.key == b.key())
        {
            self.entries.remove(scope);
            self.metrics.batch_operations += 1;
            self.metrics.evaluated_rows += rows.len();
            let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
            let mut a = Accumulator::default();
            for r in rows {
                a.apply(
                    &evaluate(index[&r.key], spec, edges),
                    true,
                    spec.edges.len() - 1,
                    limits,
                )?;
            }
            return Ok(a);
        }
        self.visited.insert(scope.into());
        let previous = self.entries.remove(scope).filter(|e| {
            e.dataset == data.version().dataset
                && e.schema == *data.schema()
                && e.spec == *spec
                && e.limits == limits
        });
        let mut entry = if let Some(e) = previous {
            self.metrics.updated_operations += 1;
            e
        } else {
            self.metrics.batch_operations += 1;
            Entry {
                dataset: data.version().dataset,
                schema: data.schema().clone(),
                spec: spec.clone(),
                limits,
                chunks: BTreeMap::new(),
                accumulated: Accumulator::default(),
            }
        };
        let current: BTreeSet<_> = data
            .chunks()
            .iter()
            .map(DataChunk::cache_identity)
            .collect();
        let removed: Vec<_> = entry
            .chunks
            .keys()
            .filter(|id| !current.contains(id))
            .copied()
            .collect();
        for id in removed {
            let old = entry.chunks.remove(&id).expect("collected retained chunk");
            for c in old.contributions {
                entry
                    .accumulated
                    .apply(&c, false, spec.edges.len() - 1, limits)?;
            }
        }
        // RowView is borrowed from this dataset. Skip every value access in unchanged chunks.
        for chunk in data.chunks() {
            let id = chunk.cache_identity();
            if let Some(old) = entry.chunks.get(&id) {
                debug_assert!(old.chunk.shares(chunk));
                self.metrics.reused_rows += chunk.batch().len();
                continue;
            }
            let mut contributions = Vec::with_capacity(chunk.batch().len());
            for row in chunk.rows() {
                let c = evaluate(row, spec, edges);
                entry
                    .accumulated
                    .apply(&c, true, spec.edges.len() - 1, limits)?;
                contributions.push(c);
            }
            self.metrics.evaluated_rows += chunk.batch().len();
            entry.chunks.insert(
                id,
                CachedChunk {
                    chunk: chunk.clone(),
                    contributions,
                },
            );
        }
        let result = std::mem::take(&mut entry.accumulated);
        self.entries.insert(scope.into(), entry);
        Ok(result)
    }
    pub fn restore(&mut self, scope: &str, accumulated: Accumulator) {
        if let Some(entry) = self.entries.get_mut(scope) {
            entry.accumulated = accumulated;
        }
    }
}
