//! Owned hierarchy sessions, with every operation performed by the canonical core.
use super::shape_registry::ShapeRegistryHandle;
use super::{disposed, failure, handle};
use chart_core::{
    grammar::ExtensionRegistry,
    hierarchy::{HierarchyId, HierarchySession, NodeHandle, PackingRequest},
};
use pyo3::prelude::*;
handle!(HierarchyHandle, "_Hierarchy", HierarchySession);
#[pymethods]
impl HierarchyHandle {
    #[new]
    fn new(py: Python<'_>, input: String) -> PyResult<Self> {
        py.detach(|| HierarchySession::from_json(&input, &ExtensionRegistry::new()))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn registered(py: Python<'_>, input: String, registry: &ShapeRegistryHandle) -> PyResult<Self> {
        let registry = registry.get()?.clone();
        py.detach(|| HierarchySession::from_json(&input, &registry))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn from_snapshot(
        py: Python<'_>,
        input: String,
        registry: &ShapeRegistryHandle,
    ) -> PyResult<Self> {
        let registry = registry.get()?.clone();
        py.detach(|| HierarchySession::from_snapshot_json(&input, &registry))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn apply(&mut self, py: Python<'_>, input: String) -> PyResult<()> {
        let session = self.get_mut()?;
        py.detach(|| session.apply_json(&input)).map_err(failure)
    }
    fn query(&self, py: Python<'_>, input: String) -> PyResult<String> {
        let session = self.get()?;
        py.detach(|| session.query_json(&input)).map_err(failure)
    }
    fn tile(&mut self, py: Python<'_>, input: String) -> PyResult<String> {
        let session = self.get_mut()?;
        py.detach(|| session.tile_json(&input)).map_err(failure)
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        let session = self.get()?;
        py.detach(|| session.to_json()).map_err(failure)
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn copy_subtree(&self, py: Python<'_>, node: String, identity: String) -> PyResult<Self> {
        let session = self.get()?;
        py.detach(|| {
            session.copy_subtree(
                chart_core::portable::decode::<NodeHandle>(&node)?,
                chart_core::portable::decode::<HierarchyId>(&identity)?,
            )
        })
        .map(Self::wrap)
        .map_err(failure)
    }
    #[staticmethod]
    fn packing(py: Python<'_>, input: String) -> PyResult<String> {
        py.detach(|| PackingRequest::execute_json(&input))
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner = None;
    }
}
