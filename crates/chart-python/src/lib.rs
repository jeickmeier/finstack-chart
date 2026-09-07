//! Minimal owned Python headless proof. No Python callbacks or GPUI initialization.
//! Methods copy inputs/outputs and detach the interpreter for Rust-only work.
use chart_core::Diagnostic;
use chart_export::portable::{PortableChart, diagnostic_json};
use pyo3::{create_exception, exceptions::PyException, prelude::*};
create_exception!(
    chart_python,
    ChartError,
    PyException,
    "Recoverable chart error; args[0] is a structured JSON diagnostic with a stable code."
);
fn failure(error: Diagnostic) -> PyErr {
    ChartError::new_err(diagnostic_json(&error))
}
/// Owned chart, data and publication resources. Call dispose to release them deterministically.
#[pyclass(module = "chart_python")]
pub struct Chart {
    inner: PortableChart,
}
#[pymethods]
impl Chart {
    #[new]
    fn new(
        py: Python<'_>,
        definition: String,
        data: String,
        profile: String,
        font: Vec<u8>,
    ) -> PyResult<Self> {
        py.detach(move || {
            PortableChart::new(&definition, &data, &profile, font).map(|inner| Self { inner })
        })
        .map_err(failure)
    }
    /// Proof-only constructor installing the known compiled example extensions; accepts no code.
    #[cfg(feature = "extension-proof")]
    #[staticmethod]
    fn with_example_extensions(
        py: Python<'_>,
        definition: String,
        data: String,
        profile: String,
        font: Vec<u8>,
    ) -> PyResult<Self> {
        py.detach(move || {
            PortableChart::with_extensions(
                &definition,
                &data,
                &profile,
                font,
                chart_extension_example::registry()?,
            )
            .map(|inner| Self { inner })
        })
        .map_err(failure)
    }
    /// Return owned semantic JSON; long work runs detached from Python.
    fn semantics(&mut self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.semantics()).map_err(failure)
    }
    /// Return the versioned publication scene, including targets and diagnostics.
    fn scene(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.scene()).map_err(failure)
    }
    /// Export svg, pdf or png as independent Python bytes.
    fn export(&self, py: Python<'_>, format: String) -> PyResult<Vec<u8>> {
        py.detach(|| self.inner.export(&format)).map_err(failure)
    }
    /// Apply a versioned transaction; returns applied/replay/rejected/conflict outcome JSON.
    fn transaction(&mut self, py: Python<'_>, input: String) -> PyResult<String> {
        py.detach(|| self.inner.transaction(&input))
            .map_err(failure)
    }
    /// Exact-source-preserving density preview using shared core reduction.
    fn dense_preview(&self, py: Python<'_>, request: String) -> PyResult<String> {
        py.detach(|| self.inner.dense_preview(&request))
            .map_err(failure)
    }
    /// Queue, commit or inspect shared streaming state; queued never means committed.
    fn stream(&mut self, py: Python<'_>, input: String) -> PyResult<String> {
        py.detach(|| self.inner.stream(&input)).map_err(failure)
    }
    /// Apply a versioned action with definition/state revision fences.
    fn action(&mut self, py: Python<'_>, input: String) -> PyResult<String> {
        py.detach(|| self.inner.action(&input)).map_err(failure)
    }
    /// Return the canonical versioned definition.
    fn definition(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.definition()).map_err(failure)
    }
    /// Acknowledge a scene for subsequent input; no window or interpreter objects enter core.
    fn present(&mut self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.present()).map_err(failure)
    }
    /// Query presented geometry or produce typed navigation windows through the common engine.
    fn query(&mut self, py: Python<'_>, input: String) -> PyResult<String> {
        py.detach(|| self.inner.query(&input)).map_err(failure)
    }
    /// Dispatch an origin/revision/scene-fenced shared action and return effective events.
    fn dispatch(&mut self, py: Python<'_>, input: String) -> PyResult<String> {
        py.detach(|| self.inner.dispatch(&input)).map_err(failure)
    }
    /// Return the separate versioned state snapshot.
    fn state(&self, py: Python<'_>) -> PyResult<String> {
        py.detach(|| self.inner.state()).map_err(failure)
    }
    /// Restore state under an exact decimal-string current revision fence.
    fn restore_state(&mut self, py: Python<'_>, input: String, expected: String) -> PyResult<()> {
        py.detach(|| self.inner.restore_state(&input, &expected))
            .map_err(failure)
    }
    /// Idempotently release resources. Later operations raise ChartError/CHART_DISPOSED_HANDLE.
    fn dispose(&mut self, py: Python<'_>) {
        py.detach(|| self.inner.dispose());
    }
}
/// Import the headless extension without starting native or interpreter-owned workers.
#[pymodule]
fn chart_python(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Chart>()?;
    module.add("ChartError", module.py().get_type::<ChartError>())?;
    Ok(())
}
