//! Captured, versioned hierarchy accessors; no interpreter enters the core loop.
use super::{ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions};
use crate::{ChartResult, DiagnosticCode, hierarchy::*};
use std::{cmp::Ordering, collections::BTreeMap, sync::Arc};

/// Purpose of a scalar callback, including the exact treemap padding slot.
#[derive(Clone, Copy, Debug)]
pub enum HierarchyScalar {
    /// Own aggregation weight.
    Value,
    /// Explicit leaf circle radius.
    Radius,
    /// Packing padding.
    PackPadding,
    /// Inner or directional treemap padding.
    TreemapPadding(PaddingSide),
}
/// Trusted pure native operations selected by one bounded portable descriptor.
/// Implementations may support only the methods they advertise in their parameters.
/// Core checks resulting topology/numbers/rectangles and cannot preempt native code.
pub trait CustomHierarchyOperation: Send + Sync {
    /// Stable identity and portability, captured at registration.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate the bounded declarative configuration.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Custom children with caller-stable occurrence keys.
    fn children(
        &self,
        _key: HierarchyNodeId,
        _data: &Arc<serde_json::Value>,
        _parameters: &serde_json::Value,
    ) -> ChartResult<Vec<(HierarchyNodeId, Arc<serde_json::Value>)>> {
        unsupported()
    }
    /// Optional own value, radius or padding. Missing values are allowed only for sum.
    fn scalar(
        &self,
        _node: NodeView<'_>,
        _purpose: HierarchyScalar,
        _parameters: &serde_json::Value,
    ) -> ChartResult<Option<f64>> {
        unsupported()
    }
    /// Stable sibling ordering.
    fn compare(
        &self,
        _a: NodeView<'_>,
        _b: NodeView<'_>,
        _parameters: &serde_json::Value,
    ) -> ChartResult<Ordering> {
        unsupported()
    }
    /// Signed finite tree or cluster separation.
    fn separation(
        &self,
        _a: NodeView<'_>,
        _b: NodeView<'_>,
        _parameters: &serde_json::Value,
    ) -> ChartResult<f64> {
        unsupported()
    }
    /// Search predicate with breadth-first index and exact subtree root.
    fn predicate(
        &self,
        _node: NodeView<'_>,
        _index: usize,
        _root: NodeView<'_>,
        _parameters: &serde_json::Value,
    ) -> ChartResult<bool> {
        unsupported()
    }
    /// Child rectangles in the supplied parent's current child order.
    fn tile(
        &self,
        _parent: NodeView<'_>,
        _bounds: [f64; 4],
        _parameters: &serde_json::Value,
    ) -> ChartResult<Vec<[f64; 4]>> {
        unsupported()
    }
}
fn unsupported<T>() -> ChartResult<T> {
    Err(super::error(
        DiagnosticCode::UnsupportedCapability,
        "Registered hierarchy operation does not implement this accessor.",
    ))
}
#[derive(Clone)]
pub(crate) struct HierarchyRegistration {
    pub(crate) descriptor: ExtensionDescriptor,
    pub(crate) implementation: Arc<dyn CustomHierarchyOperation>,
}
#[derive(Clone, Default)]
pub(crate) struct HierarchyRegistrations {
    entries: BTreeMap<(String, u64), HierarchyRegistration>,
}
impl ExtensionRegistry {
    /// Install an exact version. Existing sessions retain their captured registration.
    pub fn register_hierarchy(
        &mut self,
        implementation: Arc<dyn CustomHierarchyOperation>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        extensions::validate_descriptor(&descriptor)?;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        let entries = &mut Arc::make_mut(&mut self.hierarchies).entries;
        if entries.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "A hierarchy operation version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "registered hierarchy operation")?;
        entries.insert(
            key,
            HierarchyRegistration {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }
    pub(crate) fn hierarchy_operation(
        &self,
        operation: &OperationRef,
        parameters: &serde_json::Value,
        portable: bool,
    ) -> ChartResult<HierarchyRegistration> {
        extensions::parameter_size(parameters)?;
        let entry = self
            .hierarchies
            .entries
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    "Hierarchy operation version is not registered.",
                )
            })?;
        if portable && !entry.descriptor.portable {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "Native-only hierarchy operation cannot execute through a portable boundary.",
            ));
        }
        entry.implementation.validate(parameters)?;
        Ok(entry.clone())
    }
}
