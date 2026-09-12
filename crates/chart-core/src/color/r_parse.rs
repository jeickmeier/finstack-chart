//! Bounded R color grammar over pinned grDevices color facts.
use super::{Paint, error};
use crate::{ChartResult, DiagnosticCode, scene::Color};
#[path = "r_names.rs"]
mod names;

/// R 4.6.1's fixed default graphics palette. Core never reads the process graphics state.
pub const R_DEFAULT_PALETTE: [Color; 8] = [
    rgb(0x000000),
    rgb(0xdf536b),
    rgb(0x61d04f),
    rgb(0x2297e6),
    rgb(0x28e2e5),
    rgb(0xcd0bbc),
    rgb(0xf5c710),
    rgb(0x9e9e9e),
];
const fn rgb(value: u32) -> Color {
    Color {
        red: (value >> 16) as u8,
        green: (value >> 8) as u8,
        blue: value as u8,
        alpha: 255,
    }
}

/// Parse R color text against the fixed reference palette, retaining RGB at zero alpha.
/// Numeric palette indexes must fit a positive signed 32-bit integer after truncation.
pub fn parse_r(text: &str) -> ChartResult<Paint> {
    parse_r_with_palette(text, &R_DEFAULT_PALETTE)
}
/// Parse R color text against an explicitly supplied palette. Names use ASCII case
/// folding and ignore ordinary spaces; special transparent spellings are exact.
/// The input is bounded by the shared default color-string byte limit.
pub fn parse_r_with_palette(text: &str, palette: &[Color]) -> ChartResult<Paint> {
    crate::limits::require_within(text.len() <= super::MAX_CSS_BYTES, "R color input byte")?;
    if text == "transparent" || text == "NA" {
        return Ok(Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 0,
        }
        .into());
    }
    if let Some(hex) = text.strip_prefix('#') {
        return super::paint::hex_bytes(hex).map(Into::into);
    }
    if text.as_bytes().first().is_some_and(u8::is_ascii_digit) {
        let value = text
            .parse::<f64>()
            .map_err(|_| error(DiagnosticCode::Validation, "Invalid R color palette index."))?;
        let integer = value.trunc();
        if !integer.is_finite() || integer > f64::from(i32::MAX) {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "R color palette indexes must fit a signed 32-bit integer.",
            ));
        }
        if integer < 1. || palette.is_empty() {
            return Err(error(
                DiagnosticCode::Validation,
                "R color palette indexes must be positive and the palette nonempty.",
            ));
        }
        return Ok(palette[(integer as usize - 1) % palette.len()].into());
    }
    let name = text
        .bytes()
        .filter(|c| *c != b' ')
        .map(|c| c.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let index = names::NAMES
        .binary_search_by(|(name_ref, _)| name_ref.as_bytes().cmp(&name))
        .map_err(|_| error(DiagnosticCode::Validation, "Invalid R color name."))?;
    Ok(rgb(names::NAMES[index].1).into())
}
