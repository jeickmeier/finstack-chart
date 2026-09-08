//! Single-threaded WASM proof. JSON and font inputs and all outputs are owned copies.
//! No borrowed WASM memory view is exposed. dispose releases the Rust payload;
//! generated free releases the wrapper and must be called only after final use.
mod authoring;
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
    /// Proof-only constructor installing known compiled extensions; JSON cannot install code.
    #[cfg(feature = "extension-proof")]
    #[wasm_bindgen(js_name=withExampleExtensions)]
    pub fn with_example_extensions(
        definition: &str,
        data: &str,
        profile: &str,
        font: Vec<u8>,
    ) -> Result<Chart, JsError> {
        PortableChart::with_extensions(
            definition,
            data,
            profile,
            font,
            chart_extension_example::registry().map_err(failure)?,
        )
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
    /// Exact-source-preserving density preview using shared core reduction.
    pub fn dense_preview(&self, request: &str) -> Result<String, JsError> {
        self.inner.dense_preview(request).map_err(failure)
    }
    /// Queue, commit or inspect shared streaming state; queued never means committed.
    pub fn stream(&mut self, input: &str) -> Result<String, JsError> {
        self.inner.stream(input).map_err(failure)
    }
    /// Apply a revision-fenced action and return its outcome JSON.
    pub fn action(&mut self, input: &str) -> Result<String, JsError> {
        self.inner.action(input).map_err(failure)
    }
    /// Canonical chart definition round-trip.
    pub fn definition(&self) -> Result<String, JsError> {
        self.inner.definition().map_err(failure)
    }
    /// Acknowledge a scene for subsequent input.
    pub fn present(&mut self) -> Result<String, JsError> {
        self.inner.present().map_err(failure)
    }
    /// Query presented geometry or produce typed navigation windows through the common engine.
    pub fn query(&mut self, input: &str) -> Result<String, JsError> {
        self.inner.query(input).map_err(failure)
    }
    /// Dispatch an origin/revision/scene-fenced shared action with effective events.
    pub fn dispatch(&mut self, input: &str) -> Result<String, JsError> {
        self.inner.dispatch(input).map_err(failure)
    }
    /// Capture/cancel/status for bounded immutable export jobs.
    pub fn export_control(&mut self, input: &str) -> Result<String, JsError> {
        self.inner.export_control(input).map_err(failure)
    }
    /// Run a captured export later; returns owned output bytes.
    pub fn export_job(&mut self, job: &str) -> Result<Vec<u8>, JsError> {
        self.inner.export_job(job).map_err(failure)
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

/// Proof-only allocator high-water observation; not a source/runtime protocol field.
#[cfg(all(feature = "extension-proof", target_arch = "wasm32"))]
#[wasm_bindgen]
pub fn authoring_memory_bytes() -> usize {
    core::arch::wasm32::memory_size::<0>() * 65536
}
