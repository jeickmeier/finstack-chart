//! Owned symbol generators and palettes; geometry is implemented in chart-core.
use super::shape_registry::_ShapeRegistry;
use super::{disposed, failure, handle, path::_Path};
use chart_core::{
    portable,
    shape::{SYMBOLS_FILL, SYMBOLS_STROKE, Symbol},
};
use wasm_bindgen::prelude::*;
handle!(_ShapeSymbol, Symbol);
#[wasm_bindgen]
impl _ShapeSymbol {
    pub fn generate_registered(
        &self,
        registry: &_ShapeRegistry,
        selection: &str,
    ) -> Result<_Path, JsError> {
        let selection = portable::decode(selection).map_err(failure)?;
        self.get()?
            .generate_registered(registry.get()?.as_ref(), &selection)
            .map(_Path::wrap)
            .map_err(failure)
    }

    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> Result<Self, JsError> {
        let generator: Symbol = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn config_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn generate(&self) -> Result<_Path, JsError> {
        self.get()?.generate().map(_Path::wrap).map_err(failure)
    }
    pub fn palettes_json() -> Result<String, JsError> {
        portable::encode(&(SYMBOLS_FILL, SYMBOLS_STROKE)).map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
