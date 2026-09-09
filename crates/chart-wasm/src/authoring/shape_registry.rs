//! Explicit portable shape registration ownership. JSON never installs code.
use super::{disposed, failure, handle};
use chart_core::{
    grammar::{ExtensionRegistry, ShapeFamily, ShapeOperation},
    portable,
};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
handle!(_ShapeRegistry, _ShapeRegistry, Arc<ExtensionRegistry>);
#[wasm_bindgen]
impl _ShapeRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::wrap(Arc::new(ExtensionRegistry::new()))
    }

    #[cfg(feature = "extension-proof")]
    pub fn example() -> Result<Self, JsError> {
        chart_extension_example::registry()
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn selection_json(&self, selection: &str, family: &str) -> Result<String, JsError> {
        let selection: ShapeOperation = portable::decode(selection).map_err(failure)?;
        let family: ShapeFamily = portable::decode(family).map_err(failure)?;
        self.get()?
            .shape_to_json(&selection, family)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
impl Default for _ShapeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
