//! Arc and pie ownership adapters; arithmetic and layout live in chart-core.
use super::shape_registry::ShapeRegistryHandle;
use super::{disposed, failure, handle, path::PathHandle};
use chart_core::{
    portable,
    shape::{Arc, ArcDatum, Pie},
};
use pyo3::prelude::*;
handle!(ShapeArcHandle, "_ShapeArc", Arc);
#[pymethods]
impl ShapeArcHandle {
    #[new]
    fn new(config: &str) -> PyResult<Self> {
        let generator: Arc = portable::decode(config).map_err(failure)?;
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
    fn generate(&self, datum: &str) -> PyResult<PathHandle> {
        let datum: ArcDatum = portable::decode(datum).map_err(failure)?;
        self.get()?
            .generate(datum)
            .map(PathHandle::wrap)
            .map_err(failure)
    }
    fn centroid_json(&self, datum: &str) -> PyResult<String> {
        let datum: ArcDatum = portable::decode(datum).map_err(failure)?;
        portable::encode(&self.get()?.centroid(datum).map_err(failure)?).map_err(failure)
    }
}
handle!(ShapePieHandle, "_ShapePie", Pie);
#[pymethods]
impl ShapePieHandle {
    fn layout_registered_json(
        &self,
        data: &str,
        values: &str,
        registry: &ShapeRegistryHandle,
        selection: &str,
    ) -> PyResult<String> {
        portable::pie_layout_registered_json(
            self.get()?,
            data,
            values,
            registry.get()?.as_ref(),
            selection,
        )
        .map_err(failure)
    }

    #[new]
    fn new(config: &str) -> PyResult<Self> {
        let generator: Pie = portable::decode(config).map_err(failure)?;
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
        portable::pie_layout_json(self.get()?, data, values).map_err(failure)
    }
}
