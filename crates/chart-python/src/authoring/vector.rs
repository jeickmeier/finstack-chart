use super::*;
use chart_core::plot::{CutOptions, CutResult, CutSpec, ResolutionOptions};
handle!(CutHandle, "_Cut", CutResult);
#[pymethods]
impl CutHandle {
    #[staticmethod]
    fn create(values: Vec<Option<f64>>, spec: &str, options: &str) -> PyResult<Self> {
        plot::cut(
            &values,
            portable::decode::<CutSpec>(spec).map_err(failure)?,
            portable::decode::<CutOptions>(options).map_err(failure)?,
        )
        .map(Self::wrap)
        .map_err(failure)
    }
    #[staticmethod]
    fn resolution(values: Vec<Option<f64>>, options: &str) -> PyResult<f64> {
        plot::resolution(
            &values,
            portable::decode::<ResolutionOptions>(options).map_err(failure)?,
        )
        .map_err(failure)
    }
    #[staticmethod]
    fn summarize(values: Vec<Option<f64>>, helper: &str) -> PyResult<String> {
        let value = plot::summarize(
            &values,
            &portable::decode::<chart_core::grammar::SummaryHelper>(helper).map_err(failure)?,
        )
        .map_err(failure)?;
        portable::encode(&value).map_err(failure)
    }
    fn to_json(&self) -> PyResult<String> {
        portable::encode(self.get()?).map_err(failure)
    }
    fn column(&self) -> PyResult<data::ColumnHandle> {
        Ok(data::ColumnHandle::wrap(self.get()?.column()))
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
