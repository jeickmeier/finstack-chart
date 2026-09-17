//! Synchronous headless publication from immutable shared-core charts.
//! Fonts and data are supplied as owned snapshots; outputs are bytes plus diagnostics.
//! Construction/encoding need no I/O, system-font scan, GPUI initialization or mandatory workers.
//! Saving encoded bytes is an explicit optional host operation.
mod authoring;
mod save;
pub use save::{
    CustomDevice, DeviceRegistry, FigurePages, SaveDpi, SaveOptions, SavePlan, SaveUnits,
    resolve_save_json,
};
mod devices_metafile;
mod devices_pictex;
mod devices_raster;
mod devices_vector;
mod encode;
pub use devices_raster::{RasterDeviceOptions, TiffCompression};
pub use devices_vector::{VectorAlphaPolicy, VectorColorModel, VectorDeviceOptions};
mod raster;
pub use authoring::{CaptureBasis, ExportOptions, Output, export_options};
mod fonts;
mod jobs;
mod profile;
mod request;
mod snapshot;
mod svg;
use chart_core::{Diagnostic, DiagnosticCode};
pub use fonts::{FontResource, FontResources};
pub use jobs::{
    ExportCancellation, ExportJob, ExportLimits, ExportMetrics, ExportPhase, ExportQueue,
};
pub use profile::{
    ExportCapabilities, Format, PageSize, PublicationProfile, TextMode, TextRepresentation,
    ViewMode,
};
pub use request::FigureRequest;
pub use snapshot::{
    ExportArtifact, FigureSnapshot, FigureTransition, FontManifest, Reproducibility,
};
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Supply a supported point-based publication profile and explicit permitted font bytes, then capture again.",
    )
}
fn resource_error(
    code: DiagnosticCode,
    message: impl Into<String>,
    descriptor: &chart_core::services::ResourceDescriptor,
) -> Diagnostic {
    let mut e = error(code, message);
    e.context.resource = Some(descriptor.id);
    e.context.resource_revision = Some(descriptor.revision);
    e
}

/// Shared typed host-language ownership adapters for primary authoring and capture.
#[doc(hidden)]
pub mod host;
/// Owned host-independent session used by the minimal binding proofs.
pub mod portable;

fn serialize_u64<S: serde::Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}
