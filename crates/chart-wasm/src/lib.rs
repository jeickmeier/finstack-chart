//! Single-threaded WASM proof. JSON and font inputs and all outputs are owned copies.
//! No borrowed WASM memory view is exposed. dispose releases the Rust payload;
//! generated free releases the wrapper and must be called only after final use.
use chart_core::Diagnostic;
use chart_export::portable::{PortableChart, diagnostic_json};
use wasm_bindgen::prelude::*;
fn failure(error: Diagnostic) -> JsError {
    JsError::new(&diagnostic_json(&error))
}
/// Owned chart session; every entry point preserves exact decimal-string wire identities.
#[wasm_bindgen]
pub struct Chart {
    inner: PortableChart,
}
#[wasm_bindgen]
impl Chart {
    /// Copy the supplied JSON/font payloads and construct one headless shared-core chart.
    #[wasm_bindgen(constructor)]
    pub fn new(
        definition: &str,
        data: &str,
        profile: &str,
        font: Vec<u8>,
    ) -> Result<Chart, JsError> {
        PortableChart::new(definition, data, profile, font)
            .map(|inner| Self { inner })
            .map_err(failure)
    }
    /// Return owned semantic JSON.
    pub fn semantics(&mut self) -> Result<String, JsError> {
        self.inner.semantics().map_err(failure)
    }
    /// Return owned versioned scene JSON with numeric primitives and stable targets.
    pub fn scene(&self) -> Result<String, JsError> {
        self.inner.scene().map_err(failure)
    }
    /// Basic SVG proof using the shared publication encoder, returned as an owned Uint8Array.
    pub fn svg(&self) -> Result<Vec<u8>, JsError> {
        self.inner.export("svg").map_err(failure)
    }
    /// Apply a versioned transaction and return its typed outcome JSON.
    pub fn transaction(&mut self, input: &str) -> Result<String, JsError> {
        self.inner.transaction(input).map_err(failure)
    }
    /// Apply a revision-fenced action and return its outcome JSON.
    pub fn action(&mut self, input: &str) -> Result<String, JsError> {
        self.inner.action(input).map_err(failure)
    }
    /// Canonical chart definition round-trip.
    pub fn definition(&self) -> Result<String, JsError> {
        self.inner.definition().map_err(failure)
    }
    /// Exact state snapshot envelope.
    pub fn state(&self) -> Result<String, JsError> {
        self.inner.state().map_err(failure)
    }
    /// Restore state using a canonical decimal current-state revision fence.
    pub fn restore_state(&mut self, input: &str, expected: &str) -> Result<(), JsError> {
        self.inner.restore_state(input, expected).map_err(failure)
    }
    /// Idempotent payload disposal. Further operations return CHART_DISPOSED_HANDLE errors.
    pub fn dispose(&mut self) {
        self.inner.dispose();
    }
}
