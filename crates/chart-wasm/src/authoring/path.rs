//! Standalone path ownership; no JavaScript geometry or SVG parsing.
use super::{_Component, disposed, failure, handle};
use chart_core::{
    path::{Path, PathLimits, PathOp, Precision},
    plot::host::Component,
    portable,
};
use wasm_bindgen::prelude::*;
handle!(_Path, _Path, Path);
#[wasm_bindgen]
impl _Path {
    #[wasm_bindgen(constructor)]
    pub fn new(digits: Option<f64>, limits: Option<String>) -> Result<Self, JsError> {
        let limits: PathLimits = limits
            .as_deref()
            .map(portable::decode)
            .transpose()
            .map_err(failure)?
            .unwrap_or_default();
        Path::with_options(
            digits
                .map(Precision::from_digits)
                .transpose()
                .map_err(failure)?
                .unwrap_or_default(),
            limits,
        )
        .map(Self::wrap)
        .map_err(failure)
    }
    pub fn from_json(input: &str) -> Result<Self, JsError> {
        Path::from_json(input).map(Self::wrap).map_err(failure)
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn draw(
        &mut self,
        method: &str,
        values: &[f64],
        anticlockwise: bool,
    ) -> Result<(), JsError> {
        self.inner
            .as_mut()
            .ok_or_else(disposed)?
            .draw(method, values, anticlockwise)
            .map_err(failure)
    }
    pub fn batch(&mut self, operations: &str) -> Result<(), JsError> {
        let path = self.inner.as_mut().ok_or_else(disposed)?;
        let ops: Vec<PathOp> = portable::decode(operations).map_err(failure)?;
        path.apply_batch(&ops).map_err(failure)
    }
    pub fn to_svg(&self) -> Result<String, JsError> {
        self.get()?.to_svg().map_err(failure)
    }
    pub fn result_json(&self) -> Result<String, JsError> {
        self.get()?.result_json().map_err(failure)
    }
    pub fn replay_json(&self) -> Result<String, JsError> {
        self.get()?.replay_json().map_err(failure)
    }
    pub fn annotation(&self, id: &str) -> Result<_Component, JsError> {
        Ok(_Component::wrap(Component::vector_path(id, self.get()?)))
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
