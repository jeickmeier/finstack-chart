use super::shape_registry::_ShapeRegistry;
// Owned Cartesian generator adapters; all controls and geometry run in chart-core.
use super::{disposed, failure, handle, path::_Path};
use chart_core::{
    portable,
    shape::{Area, AreaBoundary, Line},
};
use wasm_bindgen::prelude::*;
handle!(_ShapeLine, _ShapeLine, Line);
handle!(_ShapeArea, _ShapeArea, Area);
#[wasm_bindgen]
impl _ShapeLine {
    pub fn generate_registered(
        &self,
        data: &str,
        registry: &_ShapeRegistry,
        selection: &str,
    ) -> Result<_Path, JsError> {
        let selection = portable::decode(selection).map_err(failure)?;
        let data: Vec<Vec<f64>> = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate_registered(&data, registry.get()?.as_ref(), &selection)
            .map(_Path::wrap)
            .map_err(failure)
    }

    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> Result<Self, JsError> {
        let generator: Line = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn generate(&self, rows: &str) -> Result<_Path, JsError> {
        let data: Vec<Vec<f64>> = portable::decode(rows).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(_Path::wrap)
            .map_err(failure)
    }
    pub fn config_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
#[wasm_bindgen]
impl _ShapeArea {
    pub fn generate_registered(
        &self,
        data: &str,
        registry: &_ShapeRegistry,
        selection: &str,
    ) -> Result<_Path, JsError> {
        let selection = portable::decode(selection).map_err(failure)?;
        let data: Vec<Vec<f64>> = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate_registered(&data, registry.get()?.as_ref(), &selection)
            .map(_Path::wrap)
            .map_err(failure)
    }

    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> Result<Self, JsError> {
        let generator: Area = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn generate(&self, rows: &str) -> Result<_Path, JsError> {
        let data: Vec<Vec<f64>> = portable::decode(rows).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(_Path::wrap)
            .map_err(failure)
    }
    pub fn config_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn boundary(&self, boundary: &str) -> Result<_ShapeLine, JsError> {
        let boundary: AreaBoundary = portable::decode(boundary).map_err(failure)?;
        Ok(_ShapeLine::wrap(self.get()?.boundary(boundary)))
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
