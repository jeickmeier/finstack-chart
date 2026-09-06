//! Explicit caller-selected budgets, checked before core clones scene payloads or calls hosts.

use crate::{ChartResult, Diagnostic, DiagnosticCode};

/// Per-construction budgets. Callers own input allocation; wire decoding is not implemented.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum scene items, including decorative items.
    pub max_items: usize,
    /// Maximum total numeric path commands across a scene.
    pub max_path_commands: usize,
    /// Maximum total UTF-8 text bytes across a scene, or one measurement request.
    pub max_text_bytes: usize,
    /// Maximum resource descriptors per scene.
    pub max_resources: usize,
    /// Maximum bytes for one resource, before asking a host to resolve it.
    pub max_resource_bytes: u64,
    /// Maximum sum of declared resource bytes in a scene.
    pub max_total_resource_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_items: 100_000,
            max_path_commands: 1_000_000,
            max_text_bytes: 1_048_576,
            max_resources: 256,
            max_resource_bytes: 64 * 1024 * 1024,
            max_total_resource_bytes: 256 * 1024 * 1024,
        }
    }
}

pub(crate) fn require_within(within: bool, budget: &str) -> ChartResult<()> {
    if !within {
        return Err(Diagnostic::error(
            DiagnosticCode::ResourceLimit,
            format!("The {budget} budget would be exceeded."),
            "Reduce the payload or explicitly select a larger budget before retrying.",
        ));
    }
    Ok(())
}
