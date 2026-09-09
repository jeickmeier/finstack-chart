use super::shape_registry::_ShapeRegistry;
// Owned stack layout adapter; numerical policy is canonical Rust.
use super::{disposed, failure, handle};
use chart_core::{portable, shape::Stack};
use wasm_bindgen::prelude::*;
handle!(_ShapeStack, _ShapeStack, Stack);
#[wasm_bindgen]
impl _ShapeStack {
    pub fn layout_registered_json(
        &self,
        data: &str,
        values: &str,
        registry: &_ShapeRegistry,
        order: &str,
        offset: &str,
    ) -> Result<String, JsError> {
        portable::stack_layout_registered_json(
            self.get()?,
            data,
            values,
            registry.get()?.as_ref(),
            order,
            offset,
        )
        .map_err(failure)
    }

    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> Result<Self, JsError> {
        let generator: Stack = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn config_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
    pub fn layout_json(&self, data: &str, values: &str) -> Result<String, JsError> {
        portable::stack_layout_json(self.get()?, data, values).map_err(failure)
    }
}
