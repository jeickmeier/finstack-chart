use super::shape_registry::_ShapeRegistry;
// Arc and pie ownership adapters; arithmetic and layout live in chart-core.
use super::{disposed, failure, handle, path::_Path};
use chart_core::{
    portable,
    shape::{Arc, ArcDatum, Pie},
};
use wasm_bindgen::prelude::*;
handle!(_ShapeArc, _ShapeArc, Arc);
#[wasm_bindgen]
impl _ShapeArc {
    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> Result<Self, JsError> {
        let generator: Arc = portable::decode(config).map_err(failure)?;
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
    pub fn generate(&self, datum: &str) -> Result<_Path, JsError> {
        let datum: ArcDatum = portable::decode(datum).map_err(failure)?;
        self.get()?
            .generate(datum)
            .map(_Path::wrap)
            .map_err(failure)
    }
    pub fn centroid_json(&self, datum: &str) -> Result<String, JsError> {
        let datum: ArcDatum = portable::decode(datum).map_err(failure)?;
        portable::encode(&self.get()?.centroid(datum).map_err(failure)?).map_err(failure)
    }
}
handle!(_ShapePie, _ShapePie, Pie);
#[wasm_bindgen]
impl _ShapePie {
    pub fn layout_registered_json(
        &self,
        data: &str,
        values: &str,
        registry: &_ShapeRegistry,
        selection: &str,
    ) -> Result<String, JsError> {
        portable::pie_layout_registered_json(
            self.get()?,
            data,
            values,
            registry.get()?.as_ref(),
            selection,
        )
        .map_err(failure)
    }

    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> Result<Self, JsError> {
        let generator: Pie = portable::decode(config).map_err(failure)?;
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
        portable::pie_layout_json(self.get()?, data, values).map_err(failure)
    }
}
