use crate::{Format, TextMode, error};
use chart_core::services::{
    ResourceDescriptor, ResourceKind, TextMeasurer, TextMetrics, TextRequest, Units,
};
use chart_core::{ChartResult, DiagnosticCode, Limits, ResourceId};
use sha2::{Digest, Sha256};
use skrifa::{FontRef, MetadataProvider, raw::TableProvider};
use std::{collections::BTreeMap, sync::Arc};

/// One immutable, explicitly supplied font; no path, download or system font lookup.
#[derive(Clone, Debug)]
pub struct FontResource {
    pub(crate) descriptor: ResourceDescriptor,
    pub(crate) bytes: Arc<[u8]>,
    pub(crate) hash: String,
    subset_allowed: bool,
    editable_allowed: bool,
}
impl FontResource {
    /// Parse a single outline face and its OS/2 permissions. Restricted, bitmap/color and
    /// variable fonts are rejected for this initial publication subset.
    pub fn new(descriptor: ResourceDescriptor, bytes: Arc<[u8]>) -> ChartResult<Self> {
        let result = (|| {
            if descriptor.kind != ResourceKind::Font
                || bytes.is_empty()
                || descriptor.byte_len != bytes.len() as u64
            {
                return Err(error(
                    DiagnosticCode::InvalidResource,
                    "Font descriptor and supplied bytes disagree.",
                ));
            }
            if descriptor.byte_len > Limits::default().max_resource_bytes {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Font exceeds the resource byte budget.",
                ));
            }
            let font = FontRef::new(&bytes).map_err(|e| {
                error(
                    DiagnosticCode::InvalidResource,
                    format!("Invalid font: {e}"),
                )
            })?;
            let flags = font
                .os2()
                .map_err(|_| {
                    error(
                        DiagnosticCode::InvalidResource,
                        "Font lacks embedding permission metadata.",
                    )
                })?
                .fs_type();
            if flags & 0x0002 != 0 || flags & 0x0200 != 0 || flags & !0x030e != 0 {
                return Err(error(
                    DiagnosticCode::ExportFidelity,
                    "Font permissions prohibit this outline/font embedding route.",
                ));
            }
            if font.fvar().is_ok()
                || font.colr().is_ok()
                || font.sbix().is_ok()
                || font.cbdt().is_ok()
                || font.svg().is_ok()
            {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Variable, color, bitmap or SVG-glyph fonts are outside the initial outline-font capability.",
                ));
            }
            if font.glyf().is_err() && font.cff().is_err() {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Font has no supported static TrueType/CFF outlines.",
                ));
            }
            Ok(Self {
                descriptor,
                hash: format!("{:x}", Sha256::digest(&bytes)),
                bytes,
                subset_allowed: flags & 0x0100 == 0,
                editable_allowed: flags & 0x0004 == 0 || flags & 0x0008 != 0,
            })
        })();
        result.map_err(|mut e| {
            e.context.resource = Some(descriptor.id);
            e.context.resource_revision = Some(descriptor.revision);
            e
        })
    }
    /// Exact requested resource identity and revision.
    pub fn descriptor(&self) -> ResourceDescriptor {
        self.descriptor
    }
    pub(crate) fn embedding_type(&self) -> (&'static str, &'static str) {
        if self.bytes.starts_with(b"OTTO") {
            ("font/otf", "opentype")
        } else {
            ("font/ttf", "truetype")
        }
    }
    pub(crate) fn alias(&self) -> String {
        format!(
            "ChartFont-{}-{}",
            self.descriptor.id.get(),
            self.descriptor.revision.get()
        )
    }
    pub(crate) fn validate_text(&self, text: &str) -> ChartResult<()> {
        let font = FontRef::new(&self.bytes)
            .map_err(|e| error(DiagnosticCode::InvalidResource, e.to_string()))?;
        if let Some(c) = text
            .chars()
            .find(|c| c.is_control() || font.charmap().map(*c).is_none())
        {
            let mut e = error(
                DiagnosticCode::MissingResource,
                format!(
                    "Declared font cannot paint plain glyph U+{:04X}; implicit fallback is disabled.",
                    c as u32
                ),
            );
            e.context.resource = Some(self.descriptor.id);
            e.context.resource_revision = Some(self.descriptor.revision);
            return Err(e);
        }
        Ok(())
    }
    pub(crate) fn check_mode(&self, format: Format, mode: TextMode) -> ChartResult<()> {
        if format == Format::Svg && mode == TextMode::Preserve && !self.editable_allowed {
            let mut e = error(
                DiagnosticCode::ExportFidelity,
                "Preview/print-only font embedding cannot be offered as editable SVG text; use outline mode or an editable font.",
            );
            e.context.resource = Some(self.descriptor.id);
            e.context.resource_revision = Some(self.descriptor.revision);
            return Err(e);
        }

        if format == Format::Pdf && mode == TextMode::Preserve && !self.subset_allowed {
            let mut e = error(
                DiagnosticCode::ExportFidelity,
                "Font prohibits subsetting; use permitted outline mode or another font for text-preserving PDF.",
            );
            e.context.resource = Some(self.descriptor.id);
            e.context.resource_revision = Some(self.descriptor.revision);
            return Err(e);
        }
        Ok(())
    }
}
/// Coherent immutable font database with exact resource aliases and no implicit fallback.
#[derive(Clone)]
pub struct FontResources(Arc<Fonts>);
struct Fonts {
    fonts: BTreeMap<ResourceId, FontResource>,
    database: Arc<usvg::fontdb::Database>,
    aliases: BTreeMap<String, usvg::fontdb::ID>,
}
impl FontResources {
    /// Load bounded explicit faces; duplicate identities reject rather than replacing bytes.
    pub fn new(fonts: Vec<FontResource>) -> ChartResult<Self> {
        let limits = Limits::default();
        if fonts.len() > limits.max_resources
            || fonts
                .iter()
                .try_fold(0u64, |sum, f| sum.checked_add(f.descriptor.byte_len))
                .is_none_or(|n| n > limits.max_total_resource_bytes)
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Publication fonts exceed the resource budget.",
            ));
        }
        let mut database = usvg::fontdb::Database::new();
        let mut by_id = BTreeMap::new();
        let mut aliases = BTreeMap::new();
        for font in fonts {
            if by_id.contains_key(&font.descriptor.id) {
                let mut e = error(
                    DiagnosticCode::SchemaConflict,
                    "Duplicate publication font identity.",
                );
                e.context.resource = Some(font.descriptor.id);
                return Err(e);
            }
            let data: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(font.bytes.clone());
            let loaded = database.load_font_source(usvg::fontdb::Source::Binary(data));
            if loaded.len() != 1 {
                return Err(error(
                    DiagnosticCode::InvalidResource,
                    "Expected one supported font face.",
                ));
            }
            aliases.insert(font.alias(), loaded[0]);
            by_id.insert(font.descriptor.id, font);
        }
        Ok(Self(Arc::new(Fonts {
            fonts: by_id,
            database: Arc::new(database),
            aliases,
        })))
    }
    pub(crate) fn get(&self, descriptor: &ResourceDescriptor) -> ChartResult<&FontResource> {
        self.0
            .fonts
            .get(&descriptor.id)
            .filter(|f| f.descriptor == *descriptor)
            .ok_or_else(|| {
                let mut e = error(
                    DiagnosticCode::MissingResource,
                    "Exact publication font resource/revision is unavailable.",
                );
                e.context.resource = Some(descriptor.id);
                e.context.resource_revision = Some(descriptor.revision);
                e
            })
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = &FontResource> {
        self.0.fonts.values()
    }
    pub(crate) fn options(&self) -> usvg::Options<'_> {
        usvg::Options {
            dpi: 72.,
            fontdb: self.0.database.clone(),
            font_resolver: usvg::FontResolver {
                select_font: Box::new(|font, _| {
                    font.families().iter().find_map(|family| match family {
                        usvg::FontFamily::Named(name) => self.0.aliases.get(name).copied(),
                        _ => None,
                    })
                }),
                select_fallback: Box::new(|_, _, _| None),
            },
            ..Default::default()
        }
    }
    pub(crate) fn matches(&self, font: &FontResource, id: usvg::fontdb::ID) -> bool {
        self.0.aliases.get(&font.alias()) == Some(&id)
    }
}
impl TextMeasurer for FontResources {
    fn shape(
        &self,
        r: chart_core::typography::ShapeRequest<'_>,
    ) -> ChartResult<chart_core::typography::ShapedRun> {
        r.run.validate(r.limits)?;
        let primary = r.run.font.as_ref().unwrap_or(r.default_font);
        for (index, descriptor) in std::iter::once(primary).chain(&r.run.fallback).enumerate() {
            let font = self.get(descriptor)?;
            if chart_text::supports(&font.bytes, &r.run.text, r.run.weight)? {
                return chart_text::shape(&font.bytes, *descriptor, r, index > 0);
            }
        }
        let mut e = error(
            DiagnosticCode::MissingResource,
            "No declared face supports the complete rich run and requested weight.",
        );
        e.context.resource = Some(primary.id);
        Err(e)
    }
    fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
        if r.units != Units::Points {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Publication text measurement requires points.",
            ));
        }
        let font = self.get(r.font)?;
        font.validate_text(r.text)?;
        if r.text.is_empty() {
            return TextMetrics::new(0., 0., 0.);
        }
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100000\" height=\"100000\"><text id=\"measure\" x=\"0\" y=\"0\" font-family=\"{}\" font-size=\"{}\" xml:space=\"preserve\">{}</text></svg>",
            font.alias(),
            r.font_size,
            crate::svg::escape(r.text)
        );
        let tree = usvg::Tree::from_str(&svg, &self.options())
            .map_err(|e| error(DiagnosticCode::ExportFidelity, e.to_string()))?;
        let Some(usvg::Node::Text(text)) = tree.node_by_id("measure") else {
            return Err(error(
                DiagnosticCode::ExportFidelity,
                "Publication shaper omitted a nonempty text run.",
            ));
        };
        if text
            .layouted()
            .iter()
            .flat_map(|s| &s.positioned_glyphs)
            .any(|g| g.id.0 == 0 || !self.matches(font, g.font))
        {
            return Err(error(
                DiagnosticCode::MissingResource,
                "Publication shaping substituted a missing/different font glyph.",
            ));
        }
        let bounds = text.bounding_box();
        TextMetrics::new(
            f64::from(bounds.width()),
            f64::from(-bounds.top()),
            f64::from(bounds.bottom()),
        )
    }
}
