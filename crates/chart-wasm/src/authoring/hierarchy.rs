//! Owned hierarchy sessions forwarding to the same Rust engine as native/Python.
use super::shape_registry::_ShapeRegistry;
use super::{disposed, failure, handle};
use chart_core::{
    grammar::ExtensionRegistry,
    hierarchy::{HierarchySession, PackingRequest},
};
use wasm_bindgen::prelude::*;
handle!(_Hierarchy, HierarchySession);
#[wasm_bindgen]
impl _Hierarchy {
    #[wasm_bindgen(constructor)]
    pub fn new(input: &str) -> Result<Self, JsError> {
        HierarchySession::from_json(input, &ExtensionRegistry::new())
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn registered(input: &str, registry: &_ShapeRegistry) -> Result<Self, JsError> {
        HierarchySession::from_json(input, registry.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn from_snapshot(input: &str, registry: &_ShapeRegistry) -> Result<Self, JsError> {
        HierarchySession::from_snapshot_json(input, registry.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn apply(&mut self, input: &str) -> Result<(), JsError> {
        self.get_mut()?.apply_json(input).map_err(failure)
    }
    pub fn query(&self, input: &str) -> Result<String, JsError> {
        self.get()?.query_json(input).map_err(failure)
    }
    pub fn tile(&mut self, input: &str) -> Result<String, JsError> {
        self.get_mut()?.tile_json(input).map_err(failure)
    }
    pub fn to_json(&self) -> Result<String, JsError> {
        self.get()?.to_json().map_err(failure)
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn copy_subtree(&self, node: &str, identity: &str) -> Result<Self, JsError> {
        self.get()?
            .copy_subtree(
                chart_core::portable::decode(node).map_err(failure)?,
                chart_core::portable::decode(identity).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn packing(input: &str) -> Result<String, JsError> {
        PackingRequest::execute_json(input).map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner = None;
    }
}
