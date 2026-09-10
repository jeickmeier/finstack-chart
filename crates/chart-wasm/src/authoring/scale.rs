//! Owned scale transport. No JavaScript arithmetic or interpreter objects enter core.
use super::shape_registry::_ShapeRegistry;
use super::{disposed, failure, handle};
use chart_core::{
    portable::{self, ScaleChange, ScaleQuery},
    scales::{ScaleConstructor, ScaleOptions, StandaloneScale, StandaloneScaleSpec},
};
use wasm_bindgen::prelude::*;
handle!(_Scale, StandaloneScale);
#[wasm_bindgen]
impl _Scale {
    pub fn create(family: &str, options: &str) -> Result<Self, JsError> {
        portable::decode::<ScaleConstructor>(family)
            .and_then(|f| f.create(portable::decode::<ScaleOptions>(options)?))
            .map(Self::wrap)
            .map_err(failure)
    }
    #[wasm_bindgen(constructor)]
    pub fn new(spec: &str) -> Result<Self, JsError> {
        StandaloneScale::new(portable::decode::<StandaloneScaleSpec>(spec).map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn from_json(text: &str) -> Result<Self, JsError> {
        StandaloneScale::from_json(text)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn create_registered(
        family: &str,
        options: &str,
        registry: &_ShapeRegistry,
    ) -> Result<Self, JsError> {
        let registry = registry.get()?;
        portable::decode::<ScaleConstructor>(family)
            .and_then(|f| f.create_with_registry(portable::decode(options)?, registry))
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn from_spec_registered(spec: &str, registry: &_ShapeRegistry) -> Result<Self, JsError> {
        StandaloneScale::new_with_registry(
            portable::decode(spec).map_err(failure)?,
            registry.get()?,
        )
        .map(Self::wrap)
        .map_err(failure)
    }
    pub fn from_json_registered(text: &str, registry: &_ShapeRegistry) -> Result<Self, JsError> {
        StandaloneScale::from_json_with_registry(text, registry.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn to_json(&self) -> Result<String, JsError> {
        self.get()?.to_json().map_err(failure)
    }
    pub fn query(&self, input: &str) -> Result<String, JsError> {
        let s = self.get()?;
        portable::decode::<ScaleQuery>(input)
            .and_then(|q| q.execute(s))
            .map_err(failure)
    }
    pub fn change(&self, input: &str) -> Result<Self, JsError> {
        let s = self.get()?;
        portable::decode::<ScaleChange>(input)
            .and_then(|q| q.apply(s))
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
