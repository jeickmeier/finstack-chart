//! Recoverable errors with stable codes, contextual identities and corrective action.

use std::fmt;

use crate::identity::{
    DatasetId, FieldId, LayerId, ResourceId, Revision, RowKey, SceneStamp, SchemaVersion,
};

/// Shared result type; user input failures never require a successful empty replacement.
pub type ChartResult<T> = Result<T, Diagnostic>;

/// Severity of a diagnostic; warnings may accompany a usable outcome in later stages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    /// Recoverable failure of the requested operation.
    Error,
    /// Explicit degradation or non-fatal issue.
    Warning,
}

/// Stable error categories. Later operations may add more specific codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
    /// Malformed input or inconsistent primitive structure.
    Validation,
    /// Conflicting schema or duplicate identity.
    SchemaConflict,
    /// Non-finite or invalid numeric domain.
    NumericalDomain,
    /// Required operation or resource kind is unsupported.
    UnsupportedCapability,
    /// A referenced resource is unavailable.
    MissingResource,
    /// A resource disagrees with its declared descriptor.
    InvalidResource,
    /// A configured allocation/work budget would be exceeded.
    ResourceLimit,
    /// Available layout space is insufficient.
    LayoutPressure,
    /// The owner cancelled the operation.
    Cancelled,
    /// A newer incompatible request superseded the operation.
    Superseded,
    /// A runtime handle is no longer usable.
    DisposedHandle,
    /// The requested export representation cannot preserve required fidelity.
    ExportFidelity,
    /// A revision counter cannot advance without wrapping.
    RevisionOverflow,
    /// Expected dataset, schema or source epoch no longer matches.
    RevisionConflict,
    /// A remembered transaction identity was reused with a different payload.
    TransactionReuse,
    /// A numeric conversion cannot preserve the requested source resolution.
    PrecisionLoss,
}

impl DiagnosticCode {
    /// Stable machine-readable spelling, independent of the displayed message.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Validation => "CHART_VALIDATION",
            Self::SchemaConflict => "CHART_SCHEMA_CONFLICT",
            Self::NumericalDomain => "CHART_NUMERICAL_DOMAIN",
            Self::UnsupportedCapability => "CHART_UNSUPPORTED_CAPABILITY",
            Self::MissingResource => "CHART_MISSING_RESOURCE",
            Self::InvalidResource => "CHART_INVALID_RESOURCE",
            Self::ResourceLimit => "CHART_RESOURCE_LIMIT",
            Self::LayoutPressure => "CHART_LAYOUT_PRESSURE",
            Self::Cancelled => "CHART_CANCELLED",
            Self::Superseded => "CHART_SUPERSEDED",
            Self::DisposedHandle => "CHART_DISPOSED_HANDLE",
            Self::ExportFidelity => "CHART_EXPORT_FIDELITY",
            Self::RevisionOverflow => "CHART_REVISION_OVERFLOW",
            Self::RevisionConflict => "CHART_REVISION_CONFLICT",
            Self::TransactionReuse => "CHART_TRANSACTION_REUSE",
            Self::PrecisionLoss => "CHART_PRECISION_LOSS",
        }
    }
}

/// Relevant identities, when the failing boundary has that information.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticContext {
    /// Dataset involved in the failure.
    pub dataset: Option<DatasetId>,
    /// Layer involved in the failure.
    pub layer: Option<LayerId>,
    /// Field involved in the failure.
    pub field: Option<FieldId>,
    /// Resource involved in the failure.
    pub resource: Option<ResourceId>,
    /// Requested resource revision when a resource service has that context.
    pub resource_revision: Option<Revision>,
    /// Compilation revisions supplied to the operation.
    pub stamp: Option<SceneStamp>,
    /// Observed dataset revision at a data boundary.
    pub dataset_revision: Option<Revision>,
    /// Observed schema version at a data boundary.
    pub schema_version: Option<SchemaVersion>,
    /// Total affected rows; bounded samples do not replace this count.
    pub affected_rows: u64,
    /// Bounded sample of affected source keys, never positional identities.
    pub row_samples: Vec<RowKey>,
}

/// Structured diagnostic. Context is boxed to keep successful result values small.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Stable category.
    pub code: DiagnosticCode,
    /// Whether this represents failure or an explicit warning.
    pub severity: Severity,
    /// Human-readable explanation.
    pub message: String,
    /// Action the caller can take to correct the problem.
    pub correction: String,
    /// Identities/revisions available at the failing boundary.
    pub context: Box<DiagnosticContext>,
}

impl Diagnostic {
    /// Construct a recoverable error with actionable text.
    pub fn error(
        code: DiagnosticCode,
        message: impl Into<String>,
        correction: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            correction: correction.into(),
            context: Box::default(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {} {}",
            self.code.as_str(),
            self.message,
            self.correction
        )
    }
}

impl std::error::Error for Diagnostic {}
