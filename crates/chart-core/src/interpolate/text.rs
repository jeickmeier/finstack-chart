use super::scalar::number;
use super::{ChartResult, DiagnosticCode, MAX_VALUE_BYTES, error};
use crate::number::{ecma_whitespace, ecmascript};

#[derive(Clone, Debug)]
enum Part {
    Text(String),
    Number(f64, f64),
}
#[derive(Clone, Debug)]
pub(super) struct TextInterpolator {
    parts: Vec<Part>,
    pub(super) max_bytes: usize,
}
#[derive(Debug)]
struct Token {
    start: usize,
    end: usize,
    value: f64,
}
pub(super) fn number_prefix(s: &str) -> Option<(usize, f64)> {
    token_at(s, 0).map(|token| (token.end, token.value))
}
fn token_at(s: &str, start: usize) -> Option<Token> {
    let b = s.as_bytes();
    let mut i = start;
    if matches!(b.get(i), Some(b'+' | b'-')) {
        i += 1;
    }
    let digits = i;
    while b.get(i).is_some_and(u8::is_ascii_digit) {
        i += 1;
    }
    let before = i - digits;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let after = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if before == 0 && i == after {
            return None;
        }
    } else if before == 0 {
        return None;
    }
    let exponent = i;
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let digits = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if digits == i {
            i = exponent;
        }
    }
    Some(Token {
        start,
        end: i,
        value: s[start..i].parse().unwrap_or(f64::NAN),
    })
}
fn tokens(s: &str) -> ChartResult<Vec<Token>> {
    let mut tokens = vec![];
    let mut i = 0;
    while i < s.len() {
        if let Some(token) = token_at(s, i) {
            i = token.end;
            if tokens.len() == super::MAX_VALUES {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Interpolation text token count exceeds its budget.",
                ));
            }
            tokens.push(token);
        } else {
            i += 1;
        }
    }
    Ok(tokens)
}
impl TextInterpolator {
    pub(super) fn new(a: &str, b: &str) -> ChartResult<Self> {
        if a.len() > MAX_VALUE_BYTES || b.len() > MAX_VALUE_BYTES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Interpolation text input exceeds its byte budget.",
            ));
        }
        let a_tokens = tokens(a)?;
        let b_tokens = tokens(b)?;
        let mut parts = vec![];
        let mut end = 0;
        for (at, bt) in a_tokens.iter().zip(&b_tokens) {
            if bt.start > end {
                parts.push(Part::Text(b[end..bt.start].to_owned()));
            }
            if a[at.start..at.end] == b[bt.start..bt.end] {
                parts.push(Part::Text(b[bt.start..bt.end].to_owned()));
            } else {
                parts.push(Part::Number(at.value, bt.value));
            }
            end = bt.end;
        }
        if end < b.len() {
            parts.push(Part::Text(b[end..].to_owned()));
        }
        // Shortest binary64 spelling is at most 25 ASCII bytes, including exponent/sign.
        let max_bytes = parts
            .iter()
            .map(|p| match p {
                Part::Text(s) => s.len(),
                Part::Number(..) => 25,
            })
            .sum();
        if max_bytes > MAX_VALUE_BYTES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Interpolation text output can exceed its byte budget.",
            ));
        }
        Ok(Self { parts, max_bytes })
    }
    pub(super) fn sample_into(&self, t: f64, out: &mut String) {
        out.clear();
        for part in &self.parts {
            match part {
                Part::Text(s) => out.push_str(s),
                Part::Number(a, b) => out.push_str(&ecmascript(number(*a, *b, t))),
            }
        }
    }
}
pub(super) fn numeric_text(s: &str) -> f64 {
    let s = s.trim_matches(ecma_whitespace);
    if s.is_empty() {
        return 0.;
    }
    if s == "Infinity" || s == "+Infinity" {
        return f64::INFINITY;
    }
    if s == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    if let Some(digits) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        return radix(digits, 16);
    }
    if let Some(digits) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        return radix(digits, 2);
    }
    if let Some(digits) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        return radix(digits, 8);
    }
    token_at(s, 0)
        .filter(|t| t.end == s.len())
        .map_or(f64::NAN, |t| t.value)
}
fn radix(s: &str, base: u32) -> f64 {
    if s.is_empty() {
        return f64::NAN;
    }
    // Accumulate the exact integer bit pattern with one final binary64 rounding.
    // For long inputs the leading 54 bits plus sticky tail implement round-to-even.
    let width = match base {
        2 => 1,
        8 => 3,
        16 => 4,
        _ => unreachable!(),
    };
    let mut started = false;
    let mut bits = 0usize;
    let mut leading = 0u64;
    let mut sticky = false;
    for c in s.chars() {
        let Some(digit) = c.to_digit(base) else {
            return f64::NAN;
        };
        for bit in (0..width).rev() {
            let one = (digit >> bit) & 1;
            if !started && one == 0 {
                continue;
            }
            started = true;
            bits += 1;
            if bits <= 54 {
                leading = (leading << 1) | u64::from(one);
            } else {
                sticky |= one != 0;
            }
        }
    }
    if bits <= 53 {
        return leading as f64;
    }
    let mut mantissa = leading >> 1;
    if leading & 1 != 0 && (sticky || mantissa & 1 != 0) {
        mantissa += 1;
    }
    if bits > 1024 {
        return f64::INFINITY;
    }
    mantissa as f64 * 2_f64.powi((bits - 53) as i32)
}
