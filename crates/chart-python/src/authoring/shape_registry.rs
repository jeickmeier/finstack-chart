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
    fn materialize(&self, operation: &str, payload: &str) -> PyResult<super::data::DataHandle> {
        self.get()?
            .materialize(
                &portable::decode(operation).map_err(failure)?,
                &portable::decode(payload).map_err(failure)?,
                Default::default(),
                true,
            )
            .map(super::data::DataHandle::wrap)
            .map_err(failure)
    }

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
    fn interpolation_factory_json(&self, operation: &str, parameters: &str) -> PyResult<String> {
        let f = self
            .get()?
            .interpolation_factory(
                portable::decode(operation).map_err(failure)?,
                portable::decode(parameters).map_err(failure)?,
            )
            .map_err(failure)?;
        portable::encode(&f).map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
