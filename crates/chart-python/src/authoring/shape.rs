use super::shape_registry::ShapeRegistryHandle;
// Owned Cartesian generator adapters; all controls and geometry run in chart-core.
use super::{disposed, failure, handle, path::PathHandle};
use chart_core::{
    portable,
    shape::{Area, AreaBoundary, Line},
};
use pyo3::prelude::*;
handle!(ShapeLineHandle, "_ShapeLine", Line);
handle!(ShapeAreaHandle, "_ShapeArea", Area);
#[pymethods]
impl ShapeLineHandle {
    fn generate_registered(
        &self,
        data: &str,
        registry: &ShapeRegistryHandle,
        selection: &str,
    ) -> PyResult<PathHandle> {
        let selection = portable::decode(selection).map_err(failure)?;
        let data: Vec<Vec<f64>> = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate_registered(&data, registry.get()?.as_ref(), &selection)
            .map(PathHandle::wrap)
            .map_err(failure)
    }

    #[new]
    fn new(config: &str) -> PyResult<Self> {
        let generator: Line = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn generate(&self, rows: &str) -> PyResult<PathHandle> {
        let data: Vec<Vec<f64>> = portable::decode(rows).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(PathHandle::wrap)
            .map_err(failure)
    }
    fn config_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
#[pymethods]
impl ShapeAreaHandle {
    fn generate_registered(
        &self,
        data: &str,
        registry: &ShapeRegistryHandle,
        selection: &str,
    ) -> PyResult<PathHandle> {
        let selection = portable::decode(selection).map_err(failure)?;
        let data: Vec<Vec<f64>> = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate_registered(&data, registry.get()?.as_ref(), &selection)
            .map(PathHandle::wrap)
            .map_err(failure)
    }

    #[new]
    fn new(config: &str) -> PyResult<Self> {
        let generator: Area = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn generate(&self, rows: &str) -> PyResult<PathHandle> {
        let data: Vec<Vec<f64>> = portable::decode(rows).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(PathHandle::wrap)
            .map_err(failure)
    }
    fn config_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn boundary(&self, boundary: &str) -> PyResult<ShapeLineHandle> {
        let boundary: AreaBoundary = portable::decode(boundary).map_err(failure)?;
        Ok(ShapeLineHandle::wrap(self.get()?.boundary(boundary)))
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ShapeLineHandle>()?;
    module.add_class::<ShapeAreaHandle>()
}
