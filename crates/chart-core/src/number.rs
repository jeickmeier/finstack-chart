//! Shared ECMAScript binary64 string boundary for D3-compatible operations.
//! The pinned safe formatter handles shortest-decimal ties, exponents and exceptional values.
pub(crate) fn ecmascript(value: f64) -> String {
    ryu_js::Buffer::new().format(value).to_owned()
}

/// Positive finite nonzero decimal coefficient and exponent. Explicit precision rounds
/// the exact binary value, with decimal ties upward as required by ECMAScript.
pub(crate) fn decimal_parts(value: f64, precision: Option<usize>) -> (String, i32) {
    debug_assert!(value > 0. && value.is_finite());
    if let Some(p) = precision.filter(|p| *p > 0) {
        let (digits, exponent) = exact_decimal(value);
        round_decimal(digits, exponent, p as i32)
    } else {
        let text = ecmascript(value);
        let (coefficient, power) = text.split_once('e').map_or((text.as_str(), 0), |(s, e)| {
            (s, e.parse::<i32>().expect("decimal exponent"))
        });
        let point = coefficient.find('.').unwrap_or(coefficient.len());
        let all: String = coefficient.chars().filter(|c| *c != '.').collect();
        let leading = all.bytes().take_while(|c| *c == b'0').count();
        let digits = all[leading..].trim_end_matches('0').to_owned();
        (digits, point as i32 - leading as i32 - 1 + power)
    }
}

fn exact_decimal(value: f64) -> (Vec<u8>, i32) {
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 2047) as i32;
    let mantissa = (bits & ((1_u64 << 52) - 1)) | if biased == 0 { 0 } else { 1_u64 << 52 };
    let binary = if biased == 0 { -1074 } else { biased - 1075 };
    let mut digits: Vec<u8> = mantissa
        .to_string()
        .bytes()
        .rev()
        .map(|c| c - b'0')
        .collect();
    let factor = if binary < 0 { 5 } else { 2 };
    for _ in 0..binary.unsigned_abs() {
        let mut carry = 0;
        for d in &mut digits {
            let n = *d * factor + carry;
            *d = n % 10;
            carry = n / 10;
        }
        if carry != 0 {
            digits.push(carry);
        }
    }
    digits.reverse();
    let exponent = digits.len() as i32 - (-binary).max(0) - 1;
    (digits, exponent)
}
fn round_decimal(mut digits: Vec<u8>, mut exponent: i32, p: i32) -> (String, i32) {
    if p <= 0 {
        return if p == 0 && digits[0] >= 5 {
            ("1".into(), exponent + 1)
        } else {
            ("0".into(), 0)
        };
    }
    let p = p as usize;
    if digits.len() > p {
        let round = digits[p] >= 5;
        digits.truncate(p);
        if round {
            let mut carry = true;
            for d in digits.iter_mut().rev() {
                if *d < 9 {
                    *d += 1;
                    carry = false;
                    break;
                }
                *d = 0;
            }
            if carry {
                digits[0] = 1;
                exponent += 1;
            }
        }
    } else {
        digits.resize(p, 0);
    }
    (
        digits.into_iter().map(|d| char::from(d + b'0')).collect(),
        exponent,
    )
}
/// Exact positive fixed formatting; shares binary-to-decimal rounding with significant formats.
/// The shortest formatter's optional fixed conversion misformats 1e-25 at 20 places.
pub(crate) fn decimal_fixed(value: f64, places: usize) -> String {
    if !value.is_finite() || value >= 1e21 {
        return ecmascript(value);
    }
    let (digits, e) = if value == 0. {
        ("0".into(), 0)
    } else {
        let (digits, e) = exact_decimal(value);
        round_decimal(digits, e, e + places as i32 + 1)
    };
    let mut text = decimal_expansion(&digits, e);
    if places > 0 {
        let existing = text.find('.').map_or(0, |i| text.len() - i - 1);
        if !text.contains('.') {
            text.push('.');
        }
        text.push_str(&"0".repeat(places.saturating_sub(existing)));
    }
    text
}

pub(crate) fn decimal_expansion(digits: &str, exponent: i32) -> String {
    let point = exponent + 1;
    if point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), digits)
    } else if point as usize >= digits.len() {
        format!("{}{}", digits, "0".repeat(point as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    }
}

/// ECMAScript whitespace, shared by pinned CSS and explicit numeric text conversion.
pub(crate) fn ecma_whitespace(c: char) -> bool {
    matches!(c,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}')
}

use serde::{Deserialize, Serialize};

/// Exact binary64 value classification at the numerical boundary, including -0.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(try_from = "NumberWire", into = "NumberWire")]
pub struct Number(pub f64);
impl PartialEq for Number {
    fn eq(&self, rhs: &Self) -> bool {
        self.0.to_bits() == rhs.0.to_bits() || self.0.is_nan() && rhs.0.is_nan()
    }
}
impl Eq for Number {}
impl From<f64> for Number {
    fn from(v: f64) -> Self {
        Self(v)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum NumberWire {
    Finite(f64),
    Special(Special),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Special {
    number: SpecialNumber,
}
#[derive(Serialize, Deserialize)]
enum SpecialNumber {
    NaN,
    Infinity,
    #[serde(rename = "-Infinity")]
    NegativeInfinity,
    #[serde(rename = "-0")]
    NegativeZero,
}
impl TryFrom<NumberWire> for Number {
    type Error = String;
    fn try_from(v: NumberWire) -> Result<Self, String> {
        Ok(Self(match v {
            NumberWire::Finite(v) => v,
            NumberWire::Special(s) => match s.number {
                SpecialNumber::NaN => f64::NAN,
                SpecialNumber::Infinity => f64::INFINITY,
                SpecialNumber::NegativeInfinity => f64::NEG_INFINITY,
                SpecialNumber::NegativeZero => -0.,
            },
        }))
    }
}
impl From<Number> for NumberWire {
    fn from(v: Number) -> Self {
        let tag = if v.0.is_nan() {
            Some(SpecialNumber::NaN)
        } else if v.0 == f64::INFINITY {
            Some(SpecialNumber::Infinity)
        } else if v.0 == f64::NEG_INFINITY {
            Some(SpecialNumber::NegativeInfinity)
        } else if v.0 == 0. && v.0.is_sign_negative() {
            Some(SpecialNumber::NegativeZero)
        } else {
            None
        };
        match tag {
            Some(number) => Self::Special(Special { number }),
            None => Self::Finite(v.0),
        }
    }
}

/// ECMAScript power's unit-base exceptional rule differs from IEEE pow.
pub(crate) fn ecma_pow(base: f64, exponent: f64) -> f64 {
    if exponent.is_nan() || base.abs() == 1. && exponent.is_infinite() {
        f64::NAN
    } else {
        pxfm::f_pow(base, exponent)
    }
}

/// Preserve the existing finite f64 wire representation while retaining exceptional
/// values through the canonical Number transport instead of JSON null.
pub(crate) mod finite_or_special {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
        if value.is_finite() {
            value.serialize(serializer)
        } else {
            super::Number(*value).serialize(serializer)
        }
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
        super::Number::deserialize(deserializer).map(|value| value.0)
    }
}

/// C-style general significant formatting for independently formatted numeric labels.
/// Rust's binary-to-decimal formatter supplies nearest-even rounding for each endpoint.
pub(crate) fn general_significant(value: f64, digits: usize) -> String {
    if value == 0. {
        return "0".into();
    }
    let text = format!("{value:.p$e}", p = digits - 1);
    let (mantissa, exponent) = text.split_once('e').expect("scientific formatter");
    let exponent: i32 = exponent.parse().expect("scientific exponent");
    let trim = |s: &str| {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            s.to_owned()
        }
    };
    if exponent < -4 || exponent >= digits as i32 {
        format!("{}e{exponent:+03}", trim(mantissa))
    } else {
        trim(&format!(
            "{value:.p$}",
            p = (digits as i32 - exponent - 1).max(0) as usize
        ))
    }
}
