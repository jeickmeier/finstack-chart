//! Radial and link ownership adapters; coordinate arithmetic stays in chart-core.
use super::shape_registry::_ShapeRegistry;
use super::{disposed, failure, handle, path::_Path};
use chart_core::{
    portable,
    shape::{AreaRadial, LineRadial, Link, LinkDatum, LinkRadial, RadialBoundary},
};
use wasm_bindgen::prelude::*;
handle!(_ShapeLineRadial, LineRadial);
#[wasm_bindgen]
impl _ShapeLineRadial {
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
        let generator: LineRadial = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn config_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn generate(&self, data: &str) -> Result<_Path, JsError> {
        let data: Vec<Vec<f64>> = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(_Path::wrap)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_ShapeAreaRadial, AreaRadial);
#[wasm_bindgen]
impl _ShapeAreaRadial {
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
        let generator: AreaRadial = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn config_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn generate(&self, data: &str) -> Result<_Path, JsError> {
        let data: Vec<Vec<f64>> = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(_Path::wrap)
            .map_err(failure)
    }
    pub fn boundary(&self, boundary: &str) -> Result<_ShapeLineRadial, JsError> {
        let boundary: RadialBoundary = portable::decode(boundary).map_err(failure)?;
        Ok(_ShapeLineRadial::wrap(self.get()?.boundary(boundary)))
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_ShapeLink, Link);
#[wasm_bindgen]
impl _ShapeLink {
    pub fn generate_registered(
        &self,
        data: &str,
        registry: &_ShapeRegistry,
        selection: &str,
    ) -> Result<_Path, JsError> {
        let selection = portable::decode(selection).map_err(failure)?;
        let data: LinkDatum = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate_registered(&data, registry.get()?.as_ref(), &selection)
            .map(_Path::wrap)
            .map_err(failure)
    }

    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> Result<Self, JsError> {
        let generator: Link = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn config_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn generate(&self, data: &str) -> Result<_Path, JsError> {
        let data: LinkDatum = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(_Path::wrap)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_ShapeLinkRadial, LinkRadial);
#[wasm_bindgen]
impl _ShapeLinkRadial {
    #[wasm_bindgen(constructor)]
    pub fn new(config: &str) -> Result<Self, JsError> {
        let generator: LinkRadial = portable::decode(config).map_err(failure)?;
        generator.validate().map_err(failure)?;
        Ok(Self::wrap(generator))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn config_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn generate(&self, data: &str) -> Result<_Path, JsError> {
        let data: LinkDatum = portable::decode(data).map_err(failure)?;
        self.get()?
            .generate(&data)
            .map(_Path::wrap)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
#[wasm_bindgen]
pub fn _point_radial(angle: f64, radius: f64) -> Result<Vec<f64>, JsError> {
    chart_core::shape::point_radial(angle, radius)
        .map(|p| p.to_vec())
        .map_err(failure)
}
