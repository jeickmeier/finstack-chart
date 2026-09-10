//! Owned stack layout adapter; numerical policy is canonical Rust.
use super::shape_registry::ShapeRegistryHandle;
use super::{disposed, failure, handle};
use chart_core::{portable, shape::Stack};
use pyo3::prelude::*;
handle!(ShapeStackHandle, "_ShapeStack", Stack);
#[pymethods]
impl ShapeStackHandle {
    fn layout_registered_json(
        &self,
        data: &str,
        values: &str,
        registry: &ShapeRegistryHandle,
        order: &str,
        offset: &str,
    ) -> PyResult<String> {
        portable::stack_layout_registered_json(
            self.get()?,
            data,
            values,
            registry.get()?.as_ref(),
            order,
            offset,
        )
        .map_err(failure)
    }

    #[new]
    fn new(config: &str) -> PyResult<Self> {
        let generator: Stack = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn config_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
    fn layout_json(&self, data: &str, values: &str) -> PyResult<String> {
        portable::stack_layout_json(self.get()?, data, values).map_err(failure)
    }
}
