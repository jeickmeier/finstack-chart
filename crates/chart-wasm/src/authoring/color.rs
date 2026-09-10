//! Owned floating color values; no JavaScript math or parsing.
use super::{disposed, failure, handle};
use chart_core::color::{ColorDescriptor, ColorSpace, ColorValue, parse};
use wasm_bindgen::prelude::*;
handle!(_Color, ColorValue);
#[wasm_bindgen]
impl _Color {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, args: &[f64]) -> Result<Self, JsError> {
        ColorValue::construct(name, args)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn parse(css: &str) -> Result<Option<Self>, JsError> {
        parse(css).map(|v| v.map(Self::wrap)).map_err(failure)
    }
    pub fn from_css(css: &str, space: &str) -> Result<Self, JsError> {
        ColorValue::from_css(css, ColorSpace::from_name(space).map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn from_json(input: &str) -> Result<Self, JsError> {
        ColorDescriptor::from_json(input)
            .map(|v| Self::wrap(v.value()))
            .map_err(failure)
    }
    pub fn to_json(&self) -> Result<String, JsError> {
        ColorDescriptor::new(*self.get()?)
            .to_json()
            .map_err(failure)
    }
    pub fn value_json(&self) -> Result<String, JsError> {
        self.get()?.value_json().map_err(failure)
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(*self.get()?))
    }
    pub fn convert(&self, space: &str) -> Result<Self, JsError> {
        Ok(Self::wrap(
            self.get()?
                .convert(ColorSpace::from_name(space).map_err(failure)?),
        ))
    }
    pub fn channel(&self, name: &str) -> Result<f64, JsError> {
        self.get()?.channel(name).map_err(failure)
    }
    pub fn with_channel(&self, name: &str, value: f64) -> Result<Self, JsError> {
        self.get()?
            .with_channel(name, value)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn brighter(&self, k: Option<f64>) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.brighter(k)))
    }
    pub fn darker(&self, k: Option<f64>) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.darker(k)))
    }
    pub fn displayable(&self) -> Result<bool, JsError> {
        Ok(self.get()?.displayable())
    }
    pub fn clamp(&self) -> Result<Self, JsError> {
        self.get()?.checked_clamp().map(Self::wrap).map_err(failure)
    }
    pub fn format(&self, method: &str) -> Result<String, JsError> {
        self.get()?.format(method).map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
