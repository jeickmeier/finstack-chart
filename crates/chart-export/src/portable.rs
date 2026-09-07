//! One shared publication session for native Rust and the Python/WASM proof adapters.
//! The versioned profile deliberately covers the WP-09 basic publication subset.
use crate::{
    FigureSnapshot, FontResource, FontResources, Format, PageSize, PublicationProfile, TextMode,
    ViewMode, error,
};
use base64::Engine;
use chart_core::{
    portable::{self, Session},
    services::{ResourceDescriptor, ResourceKind},
    *,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

/// Explicitly supported font resource; native handles/paths/closures are not wire resources.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FontWire {
    /// One static font passed as separately owned bytes.
    Font {
        /// Stable font identity.
        id: ResourceId,
        /// Exact font revision.
        revision: Revision,
    },
}
/// An additional explicit font resource, encoded as bounded base64 bytes in the profile.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddedFont {
    /// Stable identity and immutable revision.
    pub resource: FontWire,
    /// Standard padded base64 font bytes. No filesystem paths or font-family lookup.
    pub base64: String,
}
/// Bounded portable publication profile; authored themes/furniture live in ChartDefinition.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileEnvelope {
    /// Optional additional exact faces for rich text and declared fallback.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_fonts: Vec<EmbeddedFont>,
    /// Host cascade tokens, preceding the authored named theme.
    #[serde(default)]
    pub host_theme: chart_core::theme::ThemePatch,
    /// Output-only styling, applied after the authored theme and interaction styling.
    #[serde(default)]
    pub output_theme: chart_core::theme::ThemePatch,
    /// Supported envelope version 1.
    pub version: u32,
    /// Positive physical width in points.
    pub width_pt: f64,
    /// Positive physical height in points.
    pub height_pt: f64,
    /// Positive font size in points.
    pub font_size: f64,
    /// Nonnegative page padding in points.
    pub padding: f64,
    /// Positive raster density; basic SVG layout is independent of it.
    pub dpi: u32,
    /// Exact font identity; bytes supplied separately.
    pub resource: FontWire,
    /// False preserves logical text; true emits vector glyph outlines.
    pub outline: bool,
    /// False captures the visible viewport; true uses the full trained domain.
    pub full_domain: bool,
}
impl ProfileEnvelope {
    fn build(self, bytes: Vec<u8>) -> ChartResult<(PublicationProfile, FontResources)> {
        if self.version != portable::VERSION {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported publication profile version",
            ));
        }
        let FontWire::Font { id, revision } = self.resource;
        let descriptor = ResourceDescriptor {
            id,
            revision,
            kind: ResourceKind::Font,
            byte_len: bytes.len() as u64,
        };
        let font = FontResource::new(descriptor, Arc::from(bytes))?;
        if self.additional_fonts.len() >= Limits::default().max_resources {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Additional font count exceeds the resource budget.",
            ));
        }
        let mut faces = vec![font];
        let mut remaining = Limits::default()
            .max_total_resource_bytes
            .saturating_sub(descriptor.byte_len);
        for embedded in self.additional_fonts {
            let estimated = (embedded.base64.len() as u64).saturating_add(3) / 4 * 3;
            if estimated > Limits::default().max_resource_bytes.saturating_add(2)
                || estimated > remaining.saturating_add(2)
            {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Embedded font exceeds decoded resource budget.",
                ));
            }
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(embedded.base64)
                .map_err(|_| {
                    error(
                        DiagnosticCode::InvalidResource,
                        "Additional font is not valid standard base64.",
                    )
                })?;
            let FontWire::Font { id, revision } = embedded.resource;
            let descriptor = ResourceDescriptor {
                id,
                revision,
                kind: ResourceKind::Font,
                byte_len: bytes.len() as u64,
            };
            remaining = remaining.checked_sub(descriptor.byte_len).ok_or_else(|| {
                error(
                    DiagnosticCode::ResourceLimit,
                    "Additional font resource budget exceeded.",
                )
            })?;
            faces.push(FontResource::new(descriptor, Arc::from(bytes))?);
        }
        let fonts = FontResources::new(faces)?;
        let mut profile =
            PublicationProfile::new(PageSize::points(self.width_pt, self.height_pt)?, descriptor)?;
        profile.dpi = self.dpi;
        profile.layout.host_theme = self.host_theme;
        profile.layout.output_theme = self.output_theme;
        profile.layout.font_size = self.font_size;
        profile.layout.padding = self.padding;
        profile.text = if self.outline {
            TextMode::Outline
        } else {
            TextMode::Preserve
        };
        profile.view = if self.full_domain {
            ViewMode::FullDomain
        } else {
            ViewMode::VisibleView
        };
        profile.validate()?;
        Ok((profile, fonts))
    }
}
struct Inner {
    presented: Option<FigureSnapshot>,
    core: Session,
    profile: PublicationProfile,
    fonts: FontResources,
}
/// Single owned chart handle. Disposal drops data, compiler caches and resources; it is idempotent.
/// Retained output strings/byte arrays are independent copies and survive disposal.
pub struct PortableChart {
    inner: Option<Inner>,
}
impl PortableChart {
    /// Validate copied JSON inputs and one supplied font; initialize without any host runtime.
    pub fn new(chart: &str, data: &str, profile: &str, font: Vec<u8>) -> ChartResult<Self> {
        Self::with_extensions(
            chart,
            data,
            profile,
            font,
            std::sync::Arc::new(chart_core::grammar::ExtensionRegistry::new()),
        )
    }
    /// Resolve known portable extension versions from a host-supplied immutable registry.
    pub fn with_extensions(
        chart: &str,
        data: &str,
        profile: &str,
        font: Vec<u8>,
        extensions: std::sync::Arc<chart_core::grammar::ExtensionRegistry>,
    ) -> ChartResult<Self> {
        let profile: ProfileEnvelope = portable::decode(profile)?;
        let core = Session::with_extensions(chart, data, extensions)?;
        let (profile, fonts) = profile.build(font)?;
        let mut chart = Self {
            inner: Some(Inner {
                presented: None,
                core,
                profile,
                fonts,
            }),
        };
        let figure = chart.capture()?;
        let inner = chart.get_mut()?;
        inner.core.present(figure.layout().clone());
        inner.presented = Some(figure);
        Ok(chart)
    }
    fn get(&self) -> ChartResult<&Inner> {
        self.inner.as_ref().ok_or_else(disposed)
    }
    fn get_mut(&mut self) -> ChartResult<&mut Inner> {
        self.inner.as_mut().ok_or_else(disposed)
    }
    /// Immutable figure using the exact shared exporter; callers may retain it after disposal.
    pub fn capture(&self) -> ChartResult<FigureSnapshot> {
        let i = self.get()?;
        if i.core.reducer().frozen_scene().is_some() {
            return i.presented.clone().ok_or_else(|| {
                error(
                    DiagnosticCode::Validation,
                    "Frozen portable chart has no presented figure.",
                )
            });
        }
        FigureSnapshot::capture_with_extensions(
            i.core.definition(),
            i.core.source(),
            i.core.state(),
            i.fonts.clone(),
            i.profile.clone(),
            i.core.extensions().clone(),
        )
    }
    /// Owned semantic JSON with generated values, domains, exact source payloads and targets.
    pub fn semantics(&mut self) -> ChartResult<String> {
        self.get_mut()?.core.semantics_json()
    }
    /// Owned versioned scene DTO; contains validated primitives/targets, no host objects.
    pub fn scene(&self) -> ChartResult<String> {
        let figure = self.capture()?;
        // The publication adds a background and optional decorations around core items.
        // Preserve one target entry per actual publication item, including decorative empties.
        let targets: Vec<_> = std::iter::once(Vec::new())
            .chain(figure.layout().targets().iter().cloned())
            .chain(
                std::iter::repeat_with(Vec::new).take(figure.metadata().profile.annotations.len()),
            )
            .collect();
        let item_panels: Vec<_> = std::iter::once(None)
            .chain(figure.layout().item_panels().iter().cloned())
            .chain(std::iter::repeat_n(
                None,
                figure.metadata().profile.annotations.len(),
            ))
            .collect();
        let panels = figure.layout().panels().iter().map(|p|json!({"key":p.key,"row":p.row,"column":p.column,"bounds":p.bounds,"plot":p.chart.plot()})).collect::<Vec<_>>();
        let insets = figure
            .layout()
            .insets()
            .iter()
            .map(|i| json!({"id":i.id,"panel":i.panel,"bounds":i.bounds,"plot":i.chart.plot()}))
            .collect::<Vec<_>>();
        let interactions: std::collections::BTreeMap<_, _> = figure
            .layout()
            .interactions()
            .iter()
            .map(|(i, v)| (i + 1, v))
            .collect();
        portable::encode(
            &json!({"interactions":interactions,"insets":insets,"version":portable::VERSION,"stamp":figure.scene().stamp(),"units":figure.scene().units(),"bounds":figure.scene().bounds(),"items":figure.scene().items(),"resources":figure.scene().resources(),"targets":targets,"item_panels":item_panels,"panels":panels,"diagnostics":figure.layout().diagnostics(),"fonts":figure.metadata().fonts.iter().map(|f|json!({"id":f.id,"revision":f.revision,"sha256":f.sha256})).collect::<Vec<_>>() }),
        )
    }
    /// Return bytes for the explicitly requested format; no host file I/O or silent fallback.
    pub fn export(&self, format: &str) -> ChartResult<Vec<u8>> {
        let format = match format {
            "svg" => Format::Svg,
            "pdf" => Format::Pdf,
            "png" => Format::Png,
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Format must be svg, pdf or png",
                ));
            }
        };
        Ok(self.capture()?.export(format)?.bytes)
    }
    /// Apply a transaction, returning its typed applied/replay/conflict/rejection JSON outcome.
    pub fn transaction(&mut self, input: &str) -> ChartResult<String> {
        portable::encode(&self.get_mut()?.core.apply_transaction(input)?)
    }
    /// Apply an exact revision-fenced action and return the typed outcome.
    pub fn action(&mut self, input: &str) -> ChartResult<String> {
        portable::encode(&self.get_mut()?.core.apply_action(input)?)
    }
    /// Acknowledge the scene a headless client chooses to show/use for input. Until this call,
    /// pending state/data changes cannot silently replace the prior interaction basis.
    pub fn present(&mut self) -> ChartResult<String> {
        let figure = self.capture()?;
        let i = self.get_mut()?;
        i.core.present(figure.layout().clone());
        i.presented = Some(figure);
        self.scene()
    }
    /// Full common reducer with explicit origin/state/scene fences. Call present after displaying
    /// a new scene; active gestures retain their original basis across later presentations.
    pub fn dispatch(&mut self, input: &str) -> ChartResult<String> {
        portable::encode(&self.get_mut()?.core.dispatch(input)?)
    }
    /// Canonical chart round-trip representation.
    pub fn definition(&self) -> ChartResult<String> {
        self.get()?.core.chart_json()
    }
    /// Separate current state envelope.
    pub fn state(&self) -> ChartResult<String> {
        self.get()?.core.state_json()
    }
    /// Restore a state snapshot with an exact decimal expected-state fence.
    pub fn restore_state(&mut self, input: &str, expected: &str) -> ChartResult<()> {
        let revision: Revision = portable::decode(&portable::encode(&expected)?)?;
        self.get_mut()?.core.restore_state(input, revision)
    }
    /// Release owned chart/source/font/cache state; repeated calls have no effect.
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
fn disposed() -> Diagnostic {
    error(
        DiagnosticCode::DisposedHandle,
        "This chart handle has been disposed",
    )
}
/// Stable structured JSON for a host exception's payload.
pub fn diagnostic_json(diagnostic: &Diagnostic) -> String {
    // Diagnostic strings/identities are always serializable; retain a controlled fallback.
    portable::encode(diagnostic).unwrap_or_else(|_| {
        "{\"code\":\"CHART_VALIDATION\",\"message\":\"Diagnostic encoding failed\"}".into()
    })
}
