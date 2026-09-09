use super::shape_registry::ShapeRegistryHandle;
// Radial and link ownership adapters; coordinate arithmetic stays in chart-core.
use super::{disposed, failure, handle, path::PathHandle};
use chart_core::{
    portable,
    shape::{AreaRadial, LineRadial, Link, LinkDatum, LinkRadial, RadialBoundary},
};
use pyo3::prelude::*;
handle!(ShapeLineRadialHandle, "_ShapeLineRadial", LineRadial);
#[pymethods]
impl ShapeLineRadialHandle {
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
        let generator: LineRadial = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn config_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn generate(&self, data: &str) -> PyResult<PathHandle> {
        let data: Vec<Vec<f64>> = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(PathHandle::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(ShapeAreaRadialHandle, "_ShapeAreaRadial", AreaRadial);
#[pymethods]
impl ShapeAreaRadialHandle {
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
        let generator: AreaRadial = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn config_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn generate(&self, data: &str) -> PyResult<PathHandle> {
        let data: Vec<Vec<f64>> = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(PathHandle::wrap)
            .map_err(failure)
    }
    fn boundary(&self, boundary: &str) -> PyResult<ShapeLineRadialHandle> {
        let boundary: RadialBoundary = portable::decode(boundary).map_err(failure)?;
        Ok(ShapeLineRadialHandle::wrap(self.get()?.boundary(boundary)))
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(ShapeLinkHandle, "_ShapeLink", Link);
#[pymethods]
impl ShapeLinkHandle {
    fn generate_registered(
        &self,
        data: &str,
        registry: &ShapeRegistryHandle,
        selection: &str,
    ) -> PyResult<PathHandle> {
        let selection = portable::decode(selection).map_err(failure)?;
        let data: LinkDatum = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate_registered(&data, registry.get()?.as_ref(), &selection)
            .map(PathHandle::wrap)
            .map_err(failure)
    }

    #[new]
    fn new(config: &str) -> PyResult<Self> {
        let generator: Link = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn config_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn generate(&self, data: &str) -> PyResult<PathHandle> {
        let data: LinkDatum = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(PathHandle::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(ShapeLinkRadialHandle, "_ShapeLinkRadial", LinkRadial);
#[pymethods]
impl ShapeLinkRadialHandle {
    #[new]
    fn new(config: &str) -> PyResult<Self> {
        let generator: LinkRadial = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn config_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn generate(&self, data: &str) -> PyResult<PathHandle> {
        let data: LinkDatum = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(PathHandle::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
#[pyfunction]
fn _point_radial(angle: f64, radius: f64) -> PyResult<(f64, f64)> {
    let p = chart_core::shape::point_radial(angle, radius).map_err(failure)?;
    Ok((p[0], p[1]))
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ShapeLineRadialHandle>()?;
    module.add_class::<ShapeAreaRadialHandle>()?;
    module.add_class::<ShapeLinkHandle>()?;
    module.add_class::<ShapeLinkRadialHandle>()?;
    module.add_function(wrap_pyfunction!(_point_radial, module)?)
}
