//! Bounded implementation of the pinned comma-form CSS grammar, without a host parser.
use super::*;
#[path = "names.rs"]
mod names;
/// Maximum CSS bytes accepted by the default standalone parser.
pub const MAX_CSS_BYTES: usize = 4096;
fn whitespace(c: char) -> bool {
    crate::number::ecma_whitespace(c)
}
fn trim(s: &str) -> &str {
    s.trim_matches(whitespace)
}
fn numeric(s: &str, integer: bool) -> Option<f64> {
    let s = trim(s);
    let b = s.as_bytes();
    let mut i = usize::from(matches!(b.first(), Some(b'+' | b'-')));
    let start = i;
    while b.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let before = i - start;
    if !integer && b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return None;
        }
    } else if before == 0 {
        return None;
    }
    if !integer && matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return None;
        }
    }
    if i != b.len() {
        return None;
    }
    s.parse().ok()
}
fn percent(s: &str) -> Option<f64> {
    let s = trim(s).strip_suffix('%')?;
    // The percent suffix is adjacent to the number in the upstream regex.
    if s.chars().last().is_some_and(whitespace) {
        return None;
    }
    numeric(s, false)
}
fn rgba(r: f64, g: f64, b: f64, a: f64) -> ColorValue {
    if a <= 0. {
        rgb(f64::NAN, f64::NAN, f64::NAN).opacity(a).into()
    } else {
        rgb(r, g, b).opacity(a).into()
    }
}
fn hsla(mut h: f64, mut s: f64, mut l: f64, a: f64) -> ColorValue {
    if a <= 0. {
        h = f64::NAN;
        s = f64::NAN;
        l = f64::NAN;
    } else if l <= 0. || l >= 1. {
        h = f64::NAN;
        s = f64::NAN;
    } else if s <= 0. {
        h = f64::NAN;
    }
    hsl(h, s, l).opacity(a).into()
}
fn named(n: u32) -> ColorValue {
    rgb(
        f64::from(n >> 16 & 255),
        f64::from(n >> 8 & 255),
        f64::from(n & 255),
    )
    .into()
}
fn parse_inner(css: &str) -> Option<ColorValue> {
    if let Some(hex) = css.strip_prefix('#') {
        if ![3, 4, 6, 8].contains(&hex.len()) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let n = u32::from_str_radix(hex, 16).ok()?;
        return Some(match hex.len() {
            3 => rgb(
                f64::from((n >> 8 & 15) * 17),
                f64::from((n >> 4 & 15) * 17),
                f64::from((n & 15) * 17),
            )
            .into(),
            4 => rgba(
                f64::from((n >> 12 & 15) * 17),
                f64::from((n >> 8 & 15) * 17),
                f64::from((n >> 4 & 15) * 17),
                f64::from((n & 15) * 17) / 255.,
            ),
            6 => named(n),
            8 => rgba(
                f64::from(n >> 24 & 255),
                f64::from(n >> 16 & 255),
                f64::from(n >> 8 & 255),
                f64::from(n & 255) / 255.,
            ),
            _ => unreachable!(),
        });
    }
    if css == "transparent" {
        return Some(rgb(f64::NAN, f64::NAN, f64::NAN).opacity(0.).into());
    }
    if let Ok(i) = names::NAMES.binary_search_by_key(&css, |(name, _)| name) {
        return Some(named(names::NAMES[i].1));
    }
    let (function, args) = css.split_once('(')?;
    let args = args.strip_suffix(')')?;
    let args: Vec<_> = args.split(',').collect();
    match (function, args.len()) {
        ("rgb", 3) | ("rgba", 4) => {
            let opacity = if args.len() == 4 {
                numeric(args[3], false)?
            } else {
                1.
            };
            let channels = if let (Some(r), Some(g), Some(b)) = (
                numeric(args[0], true),
                numeric(args[1], true),
                numeric(args[2], true),
            ) {
                [r, g, b]
            } else {
                [
                    percent(args[0])? * 255. / 100.,
                    percent(args[1])? * 255. / 100.,
                    percent(args[2])? * 255. / 100.,
                ]
            };
            Some(rgba(channels[0], channels[1], channels[2], opacity))
        }
        ("hsl", 3) | ("hsla", 4) => Some(hsla(
            numeric(args[0], false)?,
            percent(args[1])? / 100.,
            percent(args[2])? / 100.,
            if args.len() == 4 {
                numeric(args[3], false)?
            } else {
                1.
            },
        )),
        _ => None,
    }
}
/// Parse CSS using the default bounded compatibility grammar.
pub fn parse(css: &str) -> ChartResult<Option<ColorValue>> {
    parse_with_limit(css, MAX_CSS_BYTES)
}
/// Parse with a caller-supplied byte budget; no per-mark parsing is required.
pub fn parse_with_limit(css: &str, max_bytes: usize) -> ChartResult<Option<ColorValue>> {
    if css.len() > max_bytes {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "CSS color exceeds its byte budget.",
        ));
    }
    Ok(parse_inner(&trim(css).to_lowercase()))
}
