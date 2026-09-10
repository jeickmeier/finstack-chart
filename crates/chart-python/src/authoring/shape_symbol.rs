//! Owned symbol generators and palettes; geometry is implemented in chart-core.
use super::shape_registry::ShapeRegistryHandle;
use super::{disposed, failure, handle, path::PathHandle};
use chart_core::{
    portable,
    shape::{SYMBOLS_FILL, SYMBOLS_STROKE, Symbol},
};
use pyo3::prelude::*;
handle!(ShapeSymbolHandle, "_ShapeSymbol", Symbol);
#[pymethods]
impl ShapeSymbolHandle {
    fn generate_registered(
        &self,
        registry: &ShapeRegistryHandle,
        selection: &str,
    ) -> PyResult<PathHandle> {
        let selection = portable::decode(selection).map_err(failure)?;
        self.get()?
            .generate_registered(registry.get()?.as_ref(), &selection)
            .map(PathHandle::wrap)
            .map_err(failure)
    }

    #[new]
    fn new(config: &str) -> PyResult<Self> {
        let generator: Symbol = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn config_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn generate(&self) -> PyResult<PathHandle> {
        self.get()?
            .generate()
            .map(PathHandle::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn palettes_json() -> PyResult<String> {
        portable::encode(&(SYMBOLS_FILL, SYMBOLS_STROKE)).map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
