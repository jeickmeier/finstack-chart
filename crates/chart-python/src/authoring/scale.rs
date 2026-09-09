//! Owned scale transport. The core owns all dispatch, validation and numerical behavior.
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
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ScaleHandle>()
}
