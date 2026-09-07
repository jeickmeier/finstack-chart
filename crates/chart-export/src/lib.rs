//! Synchronous headless publication from immutable shared-core charts.
//! Fonts and data are supplied as owned snapshots; outputs are bytes plus diagnostics.
//! No file I/O, system-font scan, GPUI initialization or mandatory workers are used.
mod encode;
mod fonts;
mod profile;
mod snapshot;
mod svg;
use chart_core::{Diagnostic, DiagnosticCode};
pub use fonts::{FontResource, FontResources};
pub use profile::{
    ExportCapabilities, Format, PageSize, PublicationProfile, TextMode, TextRepresentation,
    ViewMode,
};
pub use snapshot::{ExportArtifact, FigureSnapshot, FontManifest, Reproducibility};
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Supply a supported point-based publication profile and explicit permitted font bytes, then capture again.",
    )
}

/// Owned host-independent session used by the minimal binding proofs.
pub mod portable;
