//! Owned standalone operations, forwarding to canonical bounded core descriptors.
use super::color::ColorHandle;
use super::shape_registry::ShapeRegistryHandle;
use super::{disposed, failure, handle};
use chart_core::interpolate::{FactoryKind, InterpolationFactory, Interpolator, Value};
use pyo3::prelude::*;
handle!(InterpolatorHandle, "_Interpolator", Interpolator);
#[pymethods]
impl InterpolatorHandle {
    #[new]
    fn new(py: Python<'_>, name: String, args: String, options: String) -> PyResult<Self> {
        py.detach(|| Interpolator::construct_json(&name, &args, &options))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, input: String) -> PyResult<Self> {
        py.detach(|| Interpolator::from_json(&input))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn from_json_registered(
        py: Python<'_>,
        input: String,
        registry: &ShapeRegistryHandle,
    ) -> PyResult<Self> {
        let registry = registry.get()?.clone();
        py.detach(|| Interpolator::from_json_with_registry(&input, &registry))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn from_spec(py: Python<'_>, input: String, registry: &ShapeRegistryHandle) -> PyResult<Self> {
        let registry = registry.get()?.clone();
        py.detach(|| {
            Interpolator::new_with_registry(chart_core::portable::decode(&input)?, &registry)
        })
        .map(Self::wrap)
        .map_err(failure)
    }
    fn descriptor_json(&self, py: Python<'_>) -> PyResult<String> {
        let f = self.get()?;
        py.detach(|| f.descriptor_json()).map_err(failure)
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        let f = self.get()?;
        py.detach(|| f.to_json()).map_err(failure)
    }
    fn copy(&self) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    fn sample_json(&self, py: Python<'_>, t: f64) -> PyResult<String> {
        let f = self.get()?;
        py.detach(|| f.sample(t)?.to_json()).map_err(failure)
    }
    fn quantize_json(&self, py: Python<'_>, count: usize) -> PyResult<String> {
        let f = self.get()?;
        py.detach(|| Value::Array(f.quantize(count)?).to_json())
            .map_err(failure)
    }
    fn sample_color(&self, t: f64) -> PyResult<ColorHandle> {
        self.get()?
            .sample_color(t)
            .map(ColorHandle::wrap)
            .map_err(failure)
    }
    fn sample_transform_json(&self, t: f64) -> PyResult<String> {
        chart_core::portable::encode(
            &self
                .get()?
                .sample_transform(t)
                .map_err(failure)?
                .coefficients(),
        )
        .map_err(failure)
    }
    fn duration_ms(&self) -> PyResult<Option<f64>> {
        Ok(self.get()?.duration_ms())
    }
    fn scheduling_duration_ms(&self) -> PyResult<Option<f64>> {
        Ok(self.get()?.scheduling_duration_ms())
    }
    #[staticmethod]
    fn normalize_value(input: &str) -> PyResult<String> {
        Value::from_json(input)
            .and_then(|v| v.to_json())
            .map_err(failure)
    }
    #[staticmethod]
    fn date_value(ms: f64) -> PyResult<String> {
        Value::date(ms).to_json().map_err(failure)
    }
    #[staticmethod]
    #[pyo3(signature=(name,gamma=None))]
    fn factory(name: &str, gamma: Option<f64>) -> PyResult<String> {
        let mut f = InterpolationFactory::new(FactoryKind::from_name(name).map_err(failure)?);
        if let Some(gamma) = gamma {
            f = f.with_gamma(gamma).map_err(failure)?;
        }
        chart_core::portable::encode(&f).map_err(failure)
    }
    #[staticmethod]
    fn chromatic(py: Python<'_>, name: String, reverse: bool) -> PyResult<Self> {
        py.detach(|| {
            Interpolator::new(chart_core::interpolate::InterpolationSpec::Chromatic {
                spec: chart_core::scales::chromatic::ChromaticSpec {
                    id: name.parse()?,
                    reverse,
                },
            })
        })
        .map(Self::wrap)
        .map_err(failure)
    }
    #[staticmethod]
    fn chromatic_catalog(py: Python<'_>) -> PyResult<String> {
        py.detach(chart_core::scales::chromatic::catalog_json)
            .map_err(failure)
    }
    #[staticmethod]
    fn chromatic_scheme(py: Python<'_>, input: String) -> PyResult<String> {
        py.detach(|| {
            let spec: chart_core::scales::chromatic::SchemeSpec =
                chart_core::portable::decode(&input)?;
            Value::Array(spec.values()?).to_json()
        })
        .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
