//! Owned floating color values; every operation executes in the shared core.
use super::{disposed, failure, handle};
use chart_core::color::{ColorDescriptor, ColorSpace, ColorValue, parse};
use pyo3::prelude::*;
handle!(ColorHandle, "_Color", ColorValue);
#[pymethods]
impl ColorHandle {
    #[new]
    fn new(name: &str, args: Vec<f64>) -> PyResult<Self> {
        ColorValue::construct(name, &args)
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn parse(css: &str) -> PyResult<Option<Self>> {
        parse(css).map(|v| v.map(Self::wrap)).map_err(failure)
    }
    #[staticmethod]
    fn from_css(css: &str, space: &str) -> PyResult<Self> {
        ColorValue::from_css(css, ColorSpace::from_name(space).map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn from_json(input: &str) -> PyResult<Self> {
        ColorDescriptor::from_json(input)
            .map(|v| Self::wrap(v.value()))
            .map_err(failure)
    }
    fn to_json(&self) -> PyResult<String> {
        ColorDescriptor::new(*self.get()?)
            .to_json()
            .map_err(failure)
    }
    fn value_json(&self) -> PyResult<String> {
        self.get()?.value_json().map_err(failure)
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(*self.get()?))
    }
    fn convert(&self, space: &str) -> PyResult<Self> {
        Ok(Self::wrap(
            self.get()?
                .convert(ColorSpace::from_name(space).map_err(failure)?),
        ))
    }
    fn channel(&self, name: &str) -> PyResult<f64> {
        self.get()?.channel(name).map_err(failure)
    }
    fn with_channel(&self, name: &str, value: f64) -> PyResult<Self> {
        self.get()?
            .with_channel(name, value)
            .map(Self::wrap)
            .map_err(failure)
    }
    #[pyo3(signature=(k=None))]
    fn brighter(&self, k: Option<f64>) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.brighter(k)))
    }
    #[pyo3(signature=(k=None))]
    fn darker(&self, k: Option<f64>) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.darker(k)))
    }
    fn displayable(&self) -> PyResult<bool> {
        Ok(self.get()?.displayable())
    }
    fn clamp(&self) -> PyResult<Self> {
        self.get()?.checked_clamp().map(Self::wrap).map_err(failure)
    }
    fn format(&self, method: &str) -> PyResult<String> {
        self.get()?.format(method).map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
