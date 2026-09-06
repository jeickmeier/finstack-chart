//! Explicit synchronous host services. Implementations need not be `Send` or `Sync`.
//!
//! Resource retrieval, font parsing/shaping and destination-specific measurement belong
//! to the caller. Core never searches system fonts, opens files or starts worker threads.

use crate::geometry::{numeric_error, positive};
use crate::limits::require_within;
use crate::{ChartResult, Diagnostic, DiagnosticCode, Limits, ResourceId, Revision};

/// Unit convention shared by a scene and its destination text service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Units {
    /// Native logical pixels, before device-pixel scaling.
    LogicalPixels,
    /// Publication points, with 72 points per inch.
    Points,
}

/// Resource representation the host is expected to supply.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    /// Font bytes; actual parsing, glyph support and permissions are host responsibilities.
    Font,
    /// Image bytes; image scene primitives are not implemented in WP-02.
    Image,
}

/// Identity of immutable host-owned bytes. Changing bytes requires a new revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceDescriptor {
    /// Stable resource identity.
    pub id: ResourceId,
    /// Revision of the exact bytes; providers must honor this snapshot identity.
    pub revision: Revision,
    /// Expected representation.
    pub kind: ResourceKind,
    /// Expected byte length, preserved without pointer-width narrowing.
    pub byte_len: u64,
}

impl ResourceDescriptor {
    pub(crate) fn validate(self, limits: Limits) -> ChartResult<()> {
        let result = if self.byte_len == 0 {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidResource,
                "A resource descriptor cannot describe empty bytes.",
                "Supply a nonempty resource and its exact byte length.",
            ))
        } else {
            require_within(self.byte_len <= limits.max_resource_bytes, "resource byte")
        };
        result.map_err(|mut error| {
            error.context.resource = Some(self.id);
            error.context.resource_revision = Some(self.revision);
            error
        })
    }
}

/// Supplies immutable bytes for the requested identity and revision, or a typed error.
pub trait ResourceProvider {
    /// Returned bytes stay borrowed from this provider. No filesystem or thread policy is implied.
    fn resolve(&self, resource: &ResourceDescriptor) -> ChartResult<&[u8]>;
}

/// Resolve only after checking the declared budget, then check the returned byte length.
/// Content identity, parsing and font permissions remain the provider's responsibility.
pub fn resolve_resource<'a>(
    provider: &'a dyn ResourceProvider,
    resource: &ResourceDescriptor,
    limits: Limits,
) -> ChartResult<&'a [u8]> {
    resource.validate(limits)?;
    let result = provider.resolve(resource).and_then(|bytes| {
        if bytes.len() as u64 != resource.byte_len {
            return Err(Diagnostic::error(
                DiagnosticCode::InvalidResource,
                "Resolved bytes disagree with the resource descriptor.",
                "Resolve the requested identity/revision and provide its exact length.",
            ));
        }
        Ok(bytes)
    });
    result.map_err(|mut error| {
        error.context.resource = Some(resource.id);
        error.context.resource_revision = Some(resource.revision);
        error
    })
}

/// One plain text run; rich shaping descriptors will follow the WP-03 typography proof.
#[derive(Clone, Copy, Debug)]
pub struct TextRequest<'a> {
    /// Logical UTF-8 text; never replaced by glyph indices in this contract.
    pub text: &'a str,
    /// Explicit font identity and revision.
    pub font: &'a ResourceDescriptor,
    /// Positive font size in the declared destination units.
    pub font_size: f64,
    /// Destination convention; host measurement and painting must agree.
    pub units: Units,
}

/// Finite advance width and nonnegative baseline metrics in the requested units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    width: f64,
    ascent: f64,
    descent: f64,
}

impl TextMetrics {
    /// Reject invalid host metrics, including overflow of ascent plus descent.
    pub fn new(width: f64, ascent: f64, descent: f64) -> ChartResult<Self> {
        if [width, ascent, descent]
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
            || !(ascent + descent).is_finite()
        {
            return Err(numeric_error(
                "Text metrics must be finite and nonnegative.",
            ));
        }
        Ok(Self {
            width,
            ascent,
            descent,
        })
    }

    /// Horizontal advance; this is not a glyph-ink bounding box.
    pub const fn width(self) -> f64 {
        self.width
    }
    /// Distance above the baseline.
    pub const fn ascent(self) -> f64 {
        self.ascent
    }
    /// Distance below the baseline.
    pub const fn descent(self) -> f64 {
        self.descent
    }
    /// Finite sum of ascent and descent.
    pub fn height(self) -> f64 {
        self.ascent + self.descent
    }
}

/// A synchronous destination-specific measurer; no font fallback is performed by core.
pub trait TextMeasurer {
    /// Measure with the exact requested font revision, size and units or return diagnostics.
    fn measure(&self, request: TextRequest<'_>) -> ChartResult<TextMetrics>;
}

/// Validate a request before invoking the host, preserving resource context on errors.
pub fn measure_text(
    measurer: &dyn TextMeasurer,
    request: TextRequest<'_>,
    limits: Limits,
) -> ChartResult<TextMetrics> {
    validate_text(request, limits)?;
    measurer.measure(request).map_err(|mut error| {
        error.context.resource = Some(request.font.id);
        error.context.resource_revision = Some(request.font.revision);
        error
    })
}

pub(crate) fn validate_text(request: TextRequest<'_>, limits: Limits) -> ChartResult<()> {
    let result = (|| {
        require_within(
            request.text.len() <= limits.max_text_bytes,
            "UTF-8 text byte",
        )?;
        request.font.validate(limits)?;
        if request.font.kind != ResourceKind::Font {
            return Err(Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Text requires a font resource.",
                "Reference a font descriptor instead of an image resource.",
            ));
        }
        positive(request.font_size, "Font size must be finite and positive.")
    })();
    result.map_err(|mut error| {
        error.context.resource = Some(request.font.id);
        error.context.resource_revision = Some(request.font.revision);
        error
    })
}
