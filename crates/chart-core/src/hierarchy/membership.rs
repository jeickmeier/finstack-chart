//! Linear-size shared source membership for nested hierarchy targets.
use super::*;
use crate::RowKey;
use std::sync::Arc;
/// Subtree range into one shared preorder source-key array.
/// Portable target metadata carries only the range; the chart hierarchy snapshot
/// serializes the shared key array once per layer/panel/input scope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HierarchyMembership {
    #[serde(skip)]
    keys: Arc<[RowKey]>,
    /// First source-key index, inclusive.
    start: usize,
    /// End source-key index, exclusive.
    end: usize,
}
impl HierarchyMembership {
    /// Half-open range in the shared source table.
    pub fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }
    pub(crate) fn new(keys: Arc<[RowKey]>, start: usize, end: usize) -> ChartResult<Self> {
        if start > end || end > keys.len() {
            return Err(invalid("Invalid hierarchy membership range."));
        }
        Ok(Self { keys, start, end })
    }
    /// Exact subtree members, including an internal node's own source row.
    pub fn members(&self) -> &[RowKey] {
        &self.keys[self.start..self.end]
    }
    /// Shared array; retained only once across all targets in this hierarchy snapshot.
    pub fn shared_keys(&self) -> &Arc<[RowKey]> {
        &self.keys
    }
}
/// Stable selection identity, including imputed path ancestors across reorders.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum HierarchyTargetKey {
    /// Caller-stable source occurrence.
    Source(RowKey),
    /// Exact normalized path of an imputed ancestor; never a fabricated source key.
    Synthetic(String),
}
