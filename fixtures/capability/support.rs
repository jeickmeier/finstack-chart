//! Shared proof-only resource loading. This SVG is a capability fixture, not a chart API.

use chart_core::services::{ResourceDescriptor, ResourceKind, ResourceProvider, resolve_resource};
use chart_core::{ChartResult, Diagnostic, DiagnosticCode, Limits, ResourceId, Revision};
use resvg::usvg;
use std::sync::Arc;

pub const SVG: &str = include_str!("scene.svg");
pub const FONTS: [(&str, &[u8]); 2] = [
    ("Noto Sans", include_bytes!("fonts/NotoSans-Regular.ttf")),
    ("Fira Mono", include_bytes!("fonts/FiraMono-Medium.ttf")),
];

struct Fonts;
impl ResourceProvider for Fonts {
    fn resolve(&self, resource: &ResourceDescriptor) -> ChartResult<&[u8]> {
        if resource.revision != Revision::INITIAL || resource.kind != ResourceKind::Font {
            return Err(font_error("Unsupported resource revision or kind"));
        }
        usize::try_from(resource.id.get())
            .ok()
            .and_then(|index| FONTS.get(index))
            .map(|(_, bytes)| *bytes)
            .ok_or_else(|| font_error("Unknown fixture font resource"))
    }
}

fn font_error(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::InvalidResource,
        message,
        "Supply an explicitly permitted font resource covering the requested glyphs.",
    )
}

pub fn validate_font(bytes: &[u8], text: &str) -> Result<(), String> {
    let face = ttf_parser::Face::parse(bytes, 0).map_err(|e| format!("invalid font: {e}"))?;
    let os2 = face
        .tables()
        .os2
        .ok_or("missing font permission metadata")?;
    if os2.permissions().is_none()
        || os2.permissions() == Some(ttf_parser::Permissions::Restricted)
        || !os2.is_subsetting_allowed()
        || !os2.is_outline_embedding_allowed()
    {
        return Err("font permissions do not permit this subset/outline proof".into());
    }
    for ch in text.chars().filter(|c| !c.is_whitespace()) {
        if face.glyph_index(ch).is_none() {
            return Err(format!("missing glyph U+{:04X}", ch as u32));
        }
    }
    Ok(())
}

pub fn tree() -> Result<usvg::Tree, Box<dyn std::error::Error>> {
    let mut fonts = usvg::fontdb::Database::new();
    for (i, (name, bytes)) in FONTS.iter().enumerate() {
        let resource = ResourceDescriptor {
            id: ResourceId::new(i as u64),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: bytes.len() as u64,
        };
        let resolved = resolve_resource(&Fonts, &resource, Limits::default())?;
        validate_font(
            resolved,
            if i == 0 {
                "Native café naïve Ω −12.5% e\u{301} × ·"
            } else {
                "12,345.67"
            },
        )
        .map_err(|error| format!("resource {i} ({name}), revision 0: {error}"))?;
        fonts.load_font_data(resolved.to_vec());
    }
    let options = usvg::Options {
        dpi: 72.0,
        font_family: "Noto Sans".into(),
        fontdb: Arc::new(fonts),
        ..Default::default()
    };
    Ok(usvg::Tree::from_str(SVG, &options)?)
}
