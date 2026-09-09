//! Standalone path ownership; every draw and result invokes the shared core implementation.
use super::{ComponentHandle, disposed, failure, handle};
use chart_core::{
    path::{Path, PathLimits, PathOp, Precision},
    plot::host::Component,
    portable,
};
use pyo3::prelude::*;
handle!(PathHandle, "_Path", Path);
#[pymethods]
impl PathHandle {
    #[new]
    #[pyo3(signature = (digits=None, limits=None))]
    fn new(digits: Option<f64>, limits: Option<&str>) -> PyResult<Self> {
        let limits: PathLimits = limits
            .map(portable::decode)
            .transpose()
            .map_err(failure)?
            .unwrap_or_default();
        Path::with_options(
            digits
                .map(Precision::from_digits)
                .transpose()
                .map_err(failure)?
                .unwrap_or_default(),
            limits,
        )
        .map(Self::wrap)
        .map_err(failure)
    }
    #[staticmethod]
    fn from_json(input: &str) -> PyResult<Self> {
        Path::from_json(input).map(Self::wrap).map_err(failure)
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn draw(&mut self, method: &str, values: Vec<f64>, anticlockwise: bool) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(disposed)?
            .draw(method, &values, anticlockwise)
            .map_err(failure)
    }
    fn batch(&mut self, operations: &str) -> PyResult<()> {
        let path = self.inner.as_mut().ok_or_else(disposed)?;
        let ops: Vec<PathOp> = portable::decode(operations).map_err(failure)?;
        path.apply_batch(&ops).map_err(failure)
    }
    fn to_svg(&self) -> PyResult<String> {
        self.get()?.to_svg().map_err(failure)
    }
    fn result_json(&self) -> PyResult<String> {
        self.get()?.result_json().map_err(failure)
    }
    fn replay_json(&self) -> PyResult<String> {
        self.get()?.replay_json().map_err(failure)
    }
    fn annotation(&self, id: &str) -> PyResult<ComponentHandle> {
        Ok(ComponentHandle::wrap(Component::vector_path(
            id,
            self.get()?,
        )))
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PathHandle>()
}
