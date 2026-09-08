//! Thin owned primary handles; all grammar and runtime behavior remains in shared Rust.
mod data;
mod output;
mod runtime;
use super::failure;
use chart_core::{
    plot::{
        self,
        host::{Component, Draft},
    },
    portable,
};
use data::{DataHandle, FieldHandle};
use pyo3::prelude::*;

fn disposed() -> PyErr {
    failure(chart_core::Diagnostic::error(
        chart_core::DiagnosticCode::DisposedHandle,
        "This primary handle has been disposed.",
        "Create a new handle.",
    ))
}
macro_rules! handle {
    ($name:ident, $python:literal, $ty:ty) => {
        #[pyclass(name = $python, module = "chart_python")]
        pub(super) struct $name {
            inner: Option<$ty>,
        }
        impl $name {
            pub(super) fn get(&self) -> PyResult<&$ty> {
                self.inner.as_ref().ok_or_else(disposed)
            }
            pub(super) fn wrap(inner: $ty) -> Self {
                Self { inner: Some(inner) }
            }
        }
    };
}
pub(super) use handle;
handle!(ComponentHandle, "_Component", Component);
#[pymethods]
impl ComponentHandle {
    #[new]
    fn new(name: &str, arguments: &str) -> PyResult<Self> {
        Component::new(name, arguments)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn set(&self, method: &str, arguments: &str) -> PyResult<Self> {
        self.get()?
            .set(method, arguments)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn with_component(&self, method: &str, other: &Self) -> PyResult<Self> {
        self.get()?
            .with(method, other.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn field(&self, method: &str, field: &FieldHandle) -> PyResult<Self> {
        self.get()?
            .field(method, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn field_parameter(&self, name: &str, field: &FieldHandle) -> PyResult<Self> {
        self.get()?
            .field_parameter(name, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn data(&self, data: &DataHandle) -> PyResult<Self> {
        self.get()?
            .data(data.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn transform(name: &str, stat: &Self) -> PyResult<Self> {
        Component::transform(name, stat.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn theme_layer(&self, layer: &Self, style: &Self) -> PyResult<Self> {
        self.get()?
            .theme_layer(layer.get()?, style.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn candle_volume(&self, candle: &Self, volume: &FieldHandle) -> PyResult<Self> {
        self.get()?
            .candle_volume(candle.get()?, *volume.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn link_axis(&self, source: &Self, target: &Self) -> PyResult<Self> {
        self.get()?
            .link_axis(source.get()?, target.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(DraftHandle, "_Draft", Draft);
#[pymethods]
impl DraftHandle {
    #[cfg(feature = "extension-proof")]
    fn with_example_extensions(&self) -> PyResult<Self> {
        self.get()?
            .extensions(chart_extension_example::registry().map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }
    #[new]
    fn new(data: &DataHandle) -> PyResult<Self> {
        Ok(Self::wrap(Draft::new(data.get()?)))
    }
    fn with_component(&self, slot: &str, value: &ComponentHandle) -> PyResult<Self> {
        self.get()?
            .with(slot, value.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn dataset(&self, data: &DataHandle) -> PyResult<Self> {
        self.get()?
            .dataset(data.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn layer(&self, name: &str, value: &ComponentHandle) -> PyResult<Self> {
        self.get()?
            .layer(name, value.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn set(&self, method: &str, arguments: &str) -> PyResult<Self> {
        self.get()?
            .set(method, arguments)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn build(&self, py: Python<'_>) -> PyResult<PlotHandle> {
        let draft = self.get()?;
        py.detach(|| draft.build())
            .map(PlotHandle::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(PlotHandle, "_Plot", plot::Plot);
#[pymethods]
impl PlotHandle {
    fn edit(&self) -> PyResult<DraftHandle> {
        Ok(DraftHandle::wrap(Draft::edit(self.get()?)))
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        let plot = self.get()?;
        py.detach(|| plot.to_json()).map_err(failure)
    }
    #[cfg(feature = "extension-proof")]
    #[staticmethod]
    fn from_json_with_example_extensions(input: &str) -> PyResult<Self> {
        plot::Plot::from_json_with_extensions(
            input,
            chart_extension_example::registry().map_err(failure)?,
        )
        .map(Self::wrap)
        .map_err(failure)
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, input: String) -> PyResult<Self> {
        py.detach(|| plot::Plot::from_json(&input))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ComponentHandle>()?;
    module.add_class::<DraftHandle>()?;
    module.add_class::<PlotHandle>()?;
    data::register(module)?;
    runtime::register(module)?;
    output::register(module)
}
