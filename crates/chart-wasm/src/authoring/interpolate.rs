//! Owned standalone interpolation, without JavaScript mathematical implementations.
use super::color::_Color;
use super::{disposed, failure, handle};
use chart_core::interpolate::{FactoryKind, InterpolationFactory, Interpolator, Value};
use wasm_bindgen::prelude::*;
handle!(_Interpolator, _Interpolator, Interpolator);
#[wasm_bindgen]
impl _Interpolator {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, args: &str, options: &str) -> Result<Self, JsError> {
        Interpolator::construct_json(name, args, options)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn from_json(input: &str) -> Result<Self, JsError> {
        Interpolator::from_json(input)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn to_json(&self) -> Result<String, JsError> {
        self.get()?.to_json().map_err(failure)
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn sample_json(&self, t: f64) -> Result<String, JsError> {
        self.get()?
            .sample(t)
            .and_then(|v| v.to_json())
            .map_err(failure)
    }
    pub fn quantize_json(&self, count: usize) -> Result<String, JsError> {
        Value::Array(self.get()?.quantize(count).map_err(failure)?)
            .to_json()
            .map_err(failure)
    }
    pub fn sample_color(&self, t: f64) -> Result<_Color, JsError> {
        self.get()?
            .sample_color(t)
            .map(_Color::wrap)
            .map_err(failure)
    }
    pub fn sample_transform_json(&self, t: f64) -> Result<String, JsError> {
        chart_core::portable::encode(
            &self
                .get()?
                .sample_transform(t)
                .map_err(failure)?
                .coefficients(),
        )
        .map_err(failure)
    }
    pub fn duration_ms(&self) -> Result<Option<f64>, JsError> {
        Ok(self.get()?.duration_ms())
    }
    pub fn scheduling_duration_ms(&self) -> Result<Option<f64>, JsError> {
        Ok(self.get()?.scheduling_duration_ms())
    }
    pub fn normalize_value(input: &str) -> Result<String, JsError> {
        Value::from_json(input)
            .and_then(|v| v.to_json())
            .map_err(failure)
    }
    pub fn date_value(ms: f64) -> Result<String, JsError> {
        Value::date(ms).to_json().map_err(failure)
    }
    pub fn factory(name: &str, gamma: Option<f64>) -> Result<String, JsError> {
        let mut f = InterpolationFactory::new(FactoryKind::from_name(name).map_err(failure)?);
        if let Some(gamma) = gamma {
            f = f.with_gamma(gamma).map_err(failure)?;
        }
        chart_core::portable::encode(&f).map_err(failure)
    }
    pub fn chromatic(name: &str, reverse: bool) -> Result<Self, JsError> {
        let id = name.parse().map_err(failure)?;
        Interpolator::new(chart_core::interpolate::InterpolationSpec::Chromatic {
            spec: chart_core::scales::chromatic::ChromaticSpec { id, reverse },
        })
        .map(Self::wrap)
        .map_err(failure)
    }
    pub fn chromatic_catalog() -> Result<String, JsError> {
        chart_core::scales::chromatic::catalog_json().map_err(failure)
    }
    pub fn chromatic_scheme(input: &str) -> Result<String, JsError> {
        let spec: chart_core::scales::chromatic::SchemeSpec =
            chart_core::portable::decode(input).map_err(failure)?;
        Value::Array(spec.values().map_err(failure)?)
            .to_json()
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
