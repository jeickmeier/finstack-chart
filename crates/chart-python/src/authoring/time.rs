//! Exact owned time handles; calendar math and resource validation remain in core.
use super::{disposed, failure, handle};
use chart_core::{
    portable,
    scales::{CalendarInterval, CalendarTicks, TimeFormat, TimeScale, TimeScaleSpec},
};
use pyo3::prelude::*;
handle!(TimeScaleHandle, "_TimeScale", TimeScale);
#[pymethods]
impl TimeScaleHandle {
    #[new]
    fn new(py: Python<'_>, spec: String) -> PyResult<Self> {
        py.detach(|| TimeScale::new(portable::decode::<TimeScaleSpec>(&spec)?))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, input: String) -> PyResult<Self> {
        py.detach(|| TimeScale::from_json(&input))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        let s = self.get()?;
        py.detach(|| s.to_json()).map_err(failure)
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn map_json(&self, py: Python<'_>, value: Option<i64>) -> PyResult<String> {
        let s = self.get()?;
        py.detach(|| s.map(value)?.to_json()).map_err(failure)
    }
    fn invert(&self, py: Python<'_>, position: f64) -> PyResult<i64> {
        let s = self.get()?;
        py.detach(|| s.invert(position)).map_err(failure)
    }
    fn ticks(&self, py: Python<'_>, selection: String, budget: usize) -> PyResult<Vec<i64>> {
        let s = self.get()?;
        py.detach(|| s.ticks(portable::decode::<CalendarTicks>(&selection)?, budget))
            .map_err(failure)
    }
    fn nice(&self, py: Python<'_>, selection: String) -> PyResult<Self> {
        let s = self.get()?;
        py.detach(|| s.nice(portable::decode::<CalendarTicks>(&selection)?))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn format(&self, py: Python<'_>, value: i64, format: String) -> PyResult<String> {
        let s = self.get()?;
        py.detach(|| {
            s.tick_format(portable::decode::<TimeFormat>(&format)?)?
                .format(value, s.spec().unit)
        })
        .map_err(failure)
    }
    fn floor(&self, py: Python<'_>, value: i64, interval: String) -> PyResult<i64> {
        let s = self.get()?;
        py.detach(|| {
            s.calendar().floor(
                value,
                s.spec().unit,
                portable::decode::<CalendarInterval>(&interval)?,
            )
        })
        .map_err(failure)
    }
    fn ceil(&self, py: Python<'_>, value: i64, interval: String) -> PyResult<i64> {
        let s = self.get()?;
        py.detach(|| {
            s.calendar().ceil(
                value,
                s.spec().unit,
                portable::decode::<CalendarInterval>(&interval)?,
            )
        })
        .map_err(failure)
    }
    fn round(&self, py: Python<'_>, value: i64, interval: String) -> PyResult<i64> {
        let s = self.get()?;
        py.detach(|| {
            s.calendar().round(
                value,
                s.spec().unit,
                portable::decode::<CalendarInterval>(&interval)?,
            )
        })
        .map_err(failure)
    }
    fn offset(&self, py: Python<'_>, value: i64, interval: String, steps: f64) -> PyResult<i64> {
        let s = self.get()?;
        py.detach(|| {
            s.calendar().offset(
                value,
                s.spec().unit,
                portable::decode::<CalendarInterval>(&interval)?,
                steps,
            )
        })
        .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<TimeScaleHandle>()
}
