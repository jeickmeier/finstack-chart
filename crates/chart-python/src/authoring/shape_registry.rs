//! Explicit portable shape registration ownership. JSON never installs code.
use super::{disposed, failure, handle};
use chart_core::{
    grammar::{ExtensionRegistry, ShapeFamily, ShapeOperation},
    portable,
};
use pyo3::prelude::*;
use std::sync::Arc;
handle!(
    ShapeRegistryHandle,
    "_ShapeRegistry",
    Arc<ExtensionRegistry>
);
#[pymethods]
impl ShapeRegistryHandle {
    #[new]
    fn new() -> Self {
        Self::wrap(Arc::new(ExtensionRegistry::new()))
    }
    #[staticmethod]
    #[cfg(feature = "extension-proof")]
    fn example() -> PyResult<Self> {
        chart_extension_example::registry()
            .map(Self::wrap)
            .map_err(failure)
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn selection_json(&self, selection: &str, family: &str) -> PyResult<String> {
        let selection: ShapeOperation = portable::decode(selection).map_err(failure)?;
        let family: ShapeFamily = portable::decode(family).map_err(failure)?;
        self.get()?
            .shape_to_json(&selection, family)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ShapeRegistryHandle>()
}
