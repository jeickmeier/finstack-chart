//! Bounded immutable hierarchy topology, node operations and standalone numeric layouts.
//!
//! ADR-022 defines the checked d3-hierarchy 3.1.2 profile. Algorithms adapted from
//! D3 retain its ISC notice in `LICENSE`. No renderer, clock or I/O enters this owner.
mod construct;
mod membership;
pub use membership::{HierarchyMembership, HierarchyTargetKey};
mod portable;
mod session;
pub use portable::*;
pub use session::*;
mod layout;
mod pack;
mod partition;
pub use pack::{Circle, PackAccessors, PackOptions, PackRadius, pack_enclose, pack_siblings};
mod topology;
mod tree;
mod treemap;
pub use crate::identity::{HierarchyId, HierarchyNodeId};
use crate::{ChartResult, Diagnostic, DiagnosticCode};
pub use construct::{GroupedEntry, NestedNode, StratifyOptions};
pub use layout::{HierarchyLayout, NodeGeometry, Separation, TreeOptions, TreeSize};
pub use partition::PartitionOptions;
use serde::{Deserialize, Serialize};
pub use topology::{Hierarchy, HierarchyNode, NodeHandle, NodeInput, NodeView, VisitOrder};
pub use treemap::{
    GOLDEN_RATIO, PaddingSide, Tiler, TreemapAccessors, TreemapHistory, TreemapOptions,
};

/// Per-operation topology, payload and algorithmic work budgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HierarchyLimits {
    /// Maximum retained occurrences (synthetic ancestors included).
    pub max_nodes: usize,
    /// Maximum edge depth; flat inputs do not consume JSON nesting depth.
    pub max_depth: usize,
    /// Maximum algorithmic visits/comparisons/candidate placements per operation.
    pub max_work: usize,
    /// Maximum aggregate payload/label bytes retained by a constructed hierarchy.
    pub max_payload_bytes: usize,
}
impl Default for HierarchyLimits {
    fn default() -> Self {
        Self {
            max_nodes: 100_000,
            max_depth: 100_000,
            max_work: 10_000_000,
            max_payload_bytes: 16 * 1024 * 1024,
        }
    }
}
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Supply valid hierarchy topology, finite checked values and sufficient explicit hierarchy budgets.",
    )
}
fn invalid(message: impl Into<String>) -> Diagnostic {
    error(DiagnosticCode::Validation, message)
}
fn numerical(message: impl Into<String>) -> Diagnostic {
    error(DiagnosticCode::NumericalDomain, message)
}
fn within(ok: bool, message: &str) -> ChartResult<()> {
    if ok {
        Ok(())
    } else {
        Err(error(DiagnosticCode::ResourceLimit, message))
    }
}
#[derive(Debug)]
struct Work {
    left: usize,
}
impl Work {
    fn new(limits: HierarchyLimits) -> Self {
        Self {
            left: limits.max_work,
        }
    }
    fn charge(&mut self, n: usize) -> ChartResult<()> {
        self.left = self.left.checked_sub(n).ok_or_else(|| {
            error(
                DiagnosticCode::ResourceLimit,
                "Hierarchy operation exceeds its work budget.",
            )
        })?;
        Ok(())
    }
}
fn weight(value: f64) -> ChartResult<f64> {
    if value.is_finite() && value >= 0. {
        Ok(value)
    } else {
        Err(numerical(
            "Hierarchy weight must be finite and nonnegative.",
        ))
    }
}
