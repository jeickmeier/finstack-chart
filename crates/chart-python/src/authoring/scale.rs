//! Owned scale transport. The core owns all dispatch, validation and numerical behavior.
use super::shape_registry::ShapeRegistryHandle;
use super::{disposed, failure, handle};
use chart_core::{
    portable::{self, ScaleChange, ScaleQuery},
    scales::{ScaleConstructor, ScaleOptions, StandaloneScale, StandaloneScaleSpec},
};
use pyo3::prelude::*;
handle!(ScaleHandle, "_Scale", StandaloneScale);
#[pymethods]
impl ScaleHandle {
    #[staticmethod]
    fn create(py: Python<'_>, family: String, options: String) -> PyResult<Self> {
        py.detach(|| {
            portable::decode::<ScaleConstructor>(&family)?
                .create(portable::decode::<ScaleOptions>(&options)?)
        })
        .map(Self::wrap)
        .map_err(failure)
    }
    #[new]
    fn new(py: Python<'_>, spec: String) -> PyResult<Self> {
        py.detach(|| StandaloneScale::new(portable::decode::<StandaloneScaleSpec>(&spec)?))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, text: String) -> PyResult<Self> {
        py.detach(|| StandaloneScale::from_json(&text))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn create_registered(
        py: Python<'_>,
        family: String,
        options: String,
        registry: &ShapeRegistryHandle,
    ) -> PyResult<Self> {
        let registry = registry.get()?.clone();
        py.detach(|| {
            portable::decode::<ScaleConstructor>(&family)?
                .create_with_registry(portable::decode(&options)?, &registry)
        })
        .map(Self::wrap)
        .map_err(failure)
    }
    #[staticmethod]
    fn from_spec_registered(
        py: Python<'_>,
        spec: String,
        registry: &ShapeRegistryHandle,
    ) -> PyResult<Self> {
        let registry = registry.get()?.clone();
        py.detach(|| StandaloneScale::new_with_registry(portable::decode(&spec)?, &registry))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn from_json_registered(
        py: Python<'_>,
        text: String,
        registry: &ShapeRegistryHandle,
    ) -> PyResult<Self> {
        let registry = registry.get()?.clone();
        py.detach(|| StandaloneScale::from_json_with_registry(&text, &registry))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        let s = self.get()?;
        py.detach(|| s.to_json()).map_err(failure)
    }
    fn query(&self, py: Python<'_>, input: String) -> PyResult<String> {
        let s = self.get()?;
        py.detach(|| portable::decode::<ScaleQuery>(&input)?.execute(s))
            .map_err(failure)
    }
    fn change(&self, py: Python<'_>, input: String) -> PyResult<Self> {
        let s = self.get()?;
        py.detach(|| portable::decode::<ScaleChange>(&input)?.apply(s))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
