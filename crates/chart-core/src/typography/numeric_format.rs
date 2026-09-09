//! D3-format 3.1.2 semantics; ISC notice in `LICENSE-d3-format`.
use crate::scales::error;
use crate::{
    ChartResult, DiagnosticCode,
    number::{decimal_expansion, decimal_fixed, decimal_parts, ecmascript},
};
use serde::{Deserialize, Serialize};

const PREFIXES: [&str; 17] = [
    "y", "z", "a", "f", "p", "n", "µ", "m", "", "k", "M", "G", "T", "P", "E", "Z", "Y",
];
const MAX_TEXT: usize = 4096;

/// Explicit numeric punctuation and digit substitution; never reads process locale.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct NumericLocale {
    /// Decimal separator.
    pub decimal: String,
    /// Group separator.
    pub thousands: Option<String>,
    /// Repeating group widths from right to left; None disables grouping; an explicit empty list retains D3 empty-group behavior.
    pub grouping: Option<Vec<usize>>,
    /// Currency prefix and suffix.
    pub currency: [String; 2],
    /// Optional ten digit replacements, indexed zero through nine.
    pub numerals: Option<[String; 10]>,
    /// Percentage suffix.
    pub percent: String,
    /// Negative sign.
    pub minus: String,
    /// NaN label.
    pub nan: String,
}
impl Default for NumericLocale {
    fn default() -> Self {
        Self {
            decimal: ".".into(),
            thousands: Some(",".into()),
            grouping: Some(vec![3]),
            currency: ["$".into(), String::new()],
            numerals: None,
            percent: "%".into(),
            minus: "−".into(),
            nan: "NaN".into(),
        }
    }
}
impl NumericLocale {
    /// Check text and grouping resource limits before preparing even an empty guide.
    pub fn validate(&self) -> ChartResult<()> {
        let text = [
            &self.decimal,
            &self.currency[0],
            &self.currency[1],
            &self.percent,
            &self.minus,
            &self.nan,
        ];
        if text
            .into_iter()
            .chain(self.thousands.iter())
            .chain(self.numerals.iter().flatten())
            .any(|s| s.len() > 64 || s.chars().any(char::is_control))
            || self
                .grouping
                .as_ref()
                .is_some_and(|g| g.len() > 32 || g.iter().any(|g| *g > MAX_TEXT))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Numeric locale exceeds printable text or grouping limits.",
            ));
        }
        Ok(())
    }
}

/// Parsed `[[fill]align][sign][symbol][0][width][,][.precision][~][type]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericSpecifier {
    /// Padding character.
    pub fill: char,
    /// `<`, `>`, `=`, or `^` alignment.
    pub align: char,
    /// `+`, `-`, space, or `(` sign handling.
    pub sign: char,
    /// Currency `$` or radix `#` prefix.
    pub symbol: Option<char>,
    /// Pad digits with zeros, after the sign.
    pub zero: bool,
    /// Minimum output width, bounded independently of precision.
    pub width: Option<usize>,
    /// Group integer digits.
    pub comma: bool,
    /// Authored precision; the formatter clamps it for the selected notation.
    pub precision: Option<i32>,
    /// Remove insignificant fractional zeros.
    pub trim: bool,
    /// Numeric type; absent or unknown ASCII letter uses trimmed general notation.
    pub kind: Option<char>,
}
impl NumericSpecifier {
    /// Parse the full portable numeric specifier grammar.
    pub fn parse(text: &str) -> ChartResult<Self> {
        let invalid = || {
            error(
                DiagnosticCode::Validation,
                format!("Invalid numeric format: {text:?}."),
            )
        };
        if text.len() > 128 {
            return Err(invalid());
        }
        let c: Vec<_> = text.chars().collect();
        let mut s = Self {
            fill: ' ',
            align: '>',
            sign: '-',
            symbol: None,
            zero: false,
            width: None,
            comma: false,
            precision: None,
            trim: false,
            kind: None,
        };
        let mut i = 0;
        if c.get(1).is_some_and(|c| "<>=^".contains(*c)) {
            s.fill = c[0];
            s.align = c[1];
            i = 2;
        } else if c.first().is_some_and(|c| "<>=^".contains(*c)) {
            s.align = c[0];
            i = 1;
        }
        if c.get(i).is_some_and(|c| "+-( ".contains(*c)) {
            s.sign = c[i];
            i += 1;
        }
        if c.get(i).is_some_and(|c| "$#".contains(*c)) {
            s.symbol = Some(c[i]);
            i += 1;
        }
        if c.get(i) == Some(&'0') {
            s.zero = true;
            i += 1;
        }
        let mut integer = || -> ChartResult<Option<usize>> {
            let start = i;
            let mut n = 0_usize;
            while c.get(i).is_some_and(char::is_ascii_digit) {
                n = n
                    .checked_mul(10)
                    .and_then(|n| n.checked_add(c[i] as usize - '0' as usize))
                    .ok_or_else(invalid)?;
                i += 1;
            }
            Ok((i > start).then_some(n))
        };
        s.width = integer()?;
        if c.get(i) == Some(&',') {
            s.comma = true;
            i += 1;
        }
        if c.get(i) == Some(&'.') {
            i += 1;
            let start = i;
            let mut p = 0_i32;
            while c.get(i).is_some_and(char::is_ascii_digit) {
                p = p
                    .saturating_mul(10)
                    .saturating_add(c[i] as i32 - '0' as i32);
                i += 1;
            }
            if i == start {
                return Err(invalid());
            }
            s.precision = Some(p);
        }
        if c.get(i) == Some(&'~') {
            s.trim = true;
            i += 1;
        }
        if c.get(i)
            .is_some_and(|c| c.is_ascii_alphabetic() || *c == '%')
        {
            s.kind = Some(c[i]);
            i += 1;
        }
        if i != c.len()
            || s.fill.is_control()
            || s.fill.len_utf16() != 1
            || s.width.is_some_and(|w| w > MAX_TEXT)
        {
            return Err(invalid());
        }
        Ok(s)
    }
}

/// Portable opt-in formatter descriptor; legacy `NumberFormat` remains stable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericFormat {
    /// D3 numeric specifier.
    pub specifier: String,
    /// Explicit punctuation and digit labels.
    #[serde(default)]
    pub locale: NumericLocale,
}
impl NumericFormat {
    /// Prepare the formatter and validate its resource budget.
    pub fn prepare(&self) -> ChartResult<NumericFormatter> {
        NumericFormatter::new(
            NumericSpecifier::parse(&self.specifier)?,
            self.locale.clone(),
        )
    }
    /// Fix the SI prefix from one reference value, using fixed-decimal precision.
    pub fn prepare_prefix(&self, reference: f64) -> ChartResult<NumericFormatter> {
        let mut spec = NumericSpecifier::parse(&self.specifier)?;
        spec.kind = Some('f');
        Ok(NumericFormatter::new(spec, self.locale.clone())?.reference_prefix(reference))
    }
}
/// Prepared numeric formatter, including optional fixed SI scale and log suppression.
#[derive(Clone, Debug)]
pub struct NumericFormatter {
    spec: NumericSpecifier,
    locale: NumericLocale,
    scale: f64,
    suffix: String,
    log: Option<(f64, bool, f64)>,
}
impl NumericFormatter {
    /// Prepare a parsed descriptor with an explicit locale.
    pub fn new(mut spec: NumericSpecifier, locale: NumericLocale) -> ChartResult<Self> {
        locale.validate()?;
        if !"<>=^".contains(spec.align)
            || !"+-( ".contains(spec.sign)
            || spec.symbol.is_some_and(|s| !"$#".contains(s))
            || spec.width.is_some_and(|w| w > MAX_TEXT)
            || spec.fill.is_control()
            || spec.fill.len_utf16() != 1
            || spec
                .kind
                .is_some_and(|c| !c.is_ascii_alphabetic() && c != '%')
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Invalid parsed numeric specifier.",
            ));
        }
        if spec.kind == Some('n') {
            spec.comma = true;
            spec.kind = Some('g');
        } else if !spec.kind.is_some_and(|c| "%bcdefgoprsXx".contains(c)) {
            spec.precision.get_or_insert(12);
            spec.trim = true;
            spec.kind = Some('g');
        }
        if spec.zero || spec.fill == '0' && spec.align == '=' {
            spec.zero = true;
            spec.fill = '0';
            spec.align = '=';
        }
        let significant = spec.kind.is_some_and(|c| "gprs".contains(c));
        spec.precision = Some(
            spec.precision
                .unwrap_or(6)
                .clamp(i32::from(significant), if significant { 21 } else { 20 }),
        );
        Ok(Self {
            spec,
            locale,
            scale: 1.,
            suffix: String::new(),
            log: None,
        })
    }
    pub(crate) fn prefix(mut self, exponent: i32) -> Self {
        self.scale = crate::number::ecma_pow(10., f64::from(-exponent));
        self.suffix = PREFIXES[(8 + exponent / 3) as usize].into();
        self
    }
    fn reference_prefix(mut self, reference: f64) -> Self {
        let e = (exponent(reference) / 3.).floor().clamp(-8., 8.) * 3.;
        if e.is_nan() {
            self.scale = f64::NAN;
            self.suffix.clear();
            self
        } else {
            self.prefix(e as i32)
        }
    }
    pub(crate) fn suppress(mut self, base: f64, negative: bool, threshold: f64) -> Self {
        self.log = Some((base, negative, threshold));
        self
    }
    /// Format a numerical value, preserving IEEE signs and explicit NaN text.
    pub fn format(&self, value: f64) -> String {
        if let Some((base, negative, threshold)) = self.log {
            let mut i = value
                / crate::scales::ticks::log_power(
                    base,
                    negative,
                    crate::interpolate::js_round(crate::scales::ticks::log_value(
                        base, negative, value,
                    )),
                );
            if i * base < base - 0.5 {
                i *= base;
            }
            if i.partial_cmp(&threshold).is_none_or(|order| order.is_gt()) {
                return String::new();
            }
        }
        let value = value * self.scale;
        if self.spec.kind == Some('c') {
            return self.assemble(String::new(), ecmascript(value), false, None);
        }
        let mut negative = value < 0. || 1. / value < 0.;
        let p = self.spec.precision.expect("prepared") as usize;
        let (mut text, prefix) = if value.is_nan() {
            (self.locale.nan.clone(), None)
        } else {
            format_type(value.abs(), p, self.spec.kind.expect("prepared"))
        };
        if self.spec.trim {
            trim(&mut text);
        }
        if negative && text.parse::<f64>().is_ok_and(|v| v == 0.) && self.spec.sign != '+' {
            negative = false;
        }
        self.assemble(text, String::new(), negative, prefix)
    }
    /// Literal string boundary of the `c` type, with the same padding and affixes.
    pub fn format_text(&self, value: &str) -> ChartResult<String> {
        if self.spec.kind != Some('c')
            || value.len() > MAX_TEXT
            || value.chars().any(char::is_control)
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Text formatting requires type c and bounded printable input.",
            ));
        }
        Ok(self.assemble(String::new(), value.into(), false, None))
    }
    fn assemble(
        &self,
        mut digits: String,
        text: String,
        negative: bool,
        si: Option<i32>,
    ) -> String {
        let s = &self.spec;
        let l = &self.locale;
        let kind = s.kind.expect("prepared");
        let mut prefix = if kind == 'c' {
            String::new()
        } else if negative {
            if s.sign == '(' {
                "(".into()
            } else {
                l.minus.clone()
            }
        } else if s.sign == '+' || s.sign == ' ' {
            s.sign.to_string()
        } else {
            String::new()
        };
        if s.symbol == Some('$') {
            prefix.push_str(&l.currency[0]);
        } else if s.symbol == Some('#') && "boxX".contains(kind) {
            prefix.push('0');
            prefix.push(kind.to_ascii_lowercase());
        }
        let mut suffix = text;
        if let Some(e) = si {
            suffix.push_str(PREFIXES[(8 + e / 3) as usize]);
        }
        if s.symbol == Some('$') {
            suffix.push_str(&l.currency[1]);
        } else if "%p".contains(kind) {
            suffix.push_str(&l.percent);
        }
        suffix.push_str(&self.suffix);
        if negative && s.sign == '(' {
            suffix.push(')');
        }
        if "defgprs%".contains(kind)
            && let Some(i) = digits.find(|c: char| !c.is_ascii_digit())
        {
            let tail = digits.split_off(i);
            suffix = format!(
                "{}{}",
                tail.strip_prefix('.')
                    .map_or_else(|| tail.clone(), |v| format!("{}{v}", l.decimal)),
                suffix
            );
        }
        if s.comma && !s.zero {
            digits = group(&digits, usize::MAX, l);
        }
        let len = utf16_len(&prefix) + utf16_len(&digits) + utf16_len(&suffix);
        let n = s.width.unwrap_or(0).saturating_sub(len);
        let mut padding = s.fill.to_string().repeat(n);
        if s.comma && s.zero {
            digits = group(
                &format!("{padding}{digits}"),
                if padding.is_empty() {
                    usize::MAX
                } else {
                    s.width.unwrap_or(0).saturating_sub(utf16_len(&suffix))
                },
                l,
            );
            padding.clear();
        }
        let result = match s.align {
            '<' => format!("{prefix}{digits}{suffix}{padding}"),
            '=' => format!("{prefix}{padding}{digits}{suffix}"),
            '^' => {
                let half = padding.chars().count() / 2;
                let left: String = padding.chars().take(half).collect();
                let right: String = padding.chars().skip(half).collect();
                format!("{left}{prefix}{digits}{suffix}{right}")
            }
            _ => format!("{padding}{prefix}{digits}{suffix}"),
        };
        if let Some(numerals) = &l.numerals {
            result
                .chars()
                .map(|c| {
                    c.to_digit(10)
                        .map_or_else(|| c.to_string(), |d| numerals[d as usize].clone())
                })
                .collect()
        } else {
            result
        }
    }
}
fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}
fn group(value: &str, width: usize, locale: &NumericLocale) -> String {
    let (Some(grouping), Some(thousands)) = (&locale.grouping, &locale.thousands) else {
        return value.into();
    };
    if grouping.is_empty() {
        return String::new();
    }
    let characters: Vec<_> = value.chars().collect();
    let mut end = characters.len();
    let mut parts = Vec::new();
    let mut j = 0;
    let mut length = 0_usize;
    while end > 0 {
        let mut g = grouping[j % grouping.len()];
        if g == 0 {
            break;
        }
        if length.saturating_add(g).saturating_add(1) > width {
            g = width.saturating_sub(length).max(1);
        }
        let start = end.saturating_sub(g);
        parts.push(characters[start..end].iter().collect::<String>());
        end = start;
        length = length.saturating_add(g).saturating_add(1);
        if length > width {
            break;
        }
        j += 1;
    }
    parts.reverse();
    parts.join(thousands)
}
fn trim(text: &mut String) {
    if let Some(dot) = text.find('.') {
        let end = text[dot..].find('e').map_or(text.len(), |i| dot + i);
        let mut i = end;
        while i > dot + 1 && text.as_bytes()[i - 1] == b'0' {
            i -= 1;
        }
        if i == dot + 1 {
            i = dot;
        }
        text.replace_range(i..end, "");
    }
}
fn scientific(digits: &str, e: i32) -> String {
    format!(
        "{}{}e{e:+}",
        &digits[..1],
        if digits.len() > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        }
    )
}
fn significant(x: f64, p: usize, scientific_only: bool) -> String {
    if !x.is_finite() {
        return ecmascript(x);
    }
    let (digits, e) = if x == 0. {
        ("0".repeat(p), 0)
    } else {
        decimal_parts(x, Some(p))
    };
    if scientific_only || e < -6 || e >= p as i32 {
        scientific(&digits, e)
    } else {
        decimal_expansion(&digits, e)
    }
}
fn format_type(x: f64, p: usize, kind: char) -> (String, Option<i32>) {
    let result = match kind {
        'f' => decimal_fixed(x, p),
        '%' => decimal_fixed(x * 100., p),
        'e' => significant(x, p + 1, true),
        'g' => significant(x, p, false),
        'r' | 'p' => {
            let x = if kind == 'p' { x * 100. } else { x };
            if !x.is_finite() || x == 0. {
                ecmascript(x)
            } else {
                let (d, e) = decimal_parts(x, Some(p));
                decimal_expansion(&d, e)
            }
        }
        's' => {
            if !x.is_finite() || x == 0. {
                return (significant(x, p, false), None);
            }
            let (mut d, e) = decimal_parts(x, Some(p));
            let prefix = (e.div_euclid(3)).clamp(-8, 8) * 3;
            let point = e - prefix + 1;
            if point <= 0 {
                d = decimal_parts(x, Some((p as i32 + point - 1).max(0) as usize)).0;
            }
            return (decimal_expansion(&d, point - 1), Some(prefix));
        }
        'd' => {
            let x = crate::interpolate::js_round(x);
            if x.is_infinite() {
                "∞".into()
            } else if x >= 1e21 {
                let (d, e) = decimal_parts(x, None);
                decimal_expansion(&d, e)
            } else {
                ecmascript(x)
            }
        }
        'b' | 'o' | 'x' | 'X' => radix(crate::interpolate::js_round(x), kind),
        _ => ecmascript(x),
    };
    (result, None)
}
fn radix(x: f64, kind: char) -> String {
    if !x.is_finite() || x == 0. {
        let s = ecmascript(x);
        return if kind == 'X' { s.to_uppercase() } else { s };
    }
    let bits = x.to_bits();
    let exponent = ((bits >> 52) & 2047) as i32 - 1075;
    let mantissa = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
    let binary = if exponent >= 0 {
        format!("{mantissa:b}{}", "0".repeat(exponent as usize))
    } else {
        format!("{:b}", mantissa >> (-exponent))
    };
    let n = match kind {
        'b' => 1,
        'o' => 3,
        _ => 4,
    };
    let padded = format!("{}{binary}", "0".repeat((n - binary.len() % n) % n));
    padded
        .as_bytes()
        .chunks(n)
        .map(|c| {
            let d = c.iter().fold(0, |v, c| v * 2 + c - b'0');
            let c = char::from_digit(u32::from(d), 16).expect("digit");
            if kind == 'X' {
                c.to_ascii_uppercase()
            } else {
                c
            }
        })
        .collect()
}

pub(crate) fn exponent(value: f64) -> f64 {
    if value == 0. || !value.is_finite() {
        f64::NAN
    } else {
        f64::from(decimal_parts(value.abs(), None).1)
    }
}

/// D3 standalone tick formatter with precision inferred in data space.
pub fn tick_format(
    start: f64,
    stop: f64,
    count: f64,
    specifier: Option<&str>,
    locale: NumericLocale,
) -> ChartResult<NumericFormatter> {
    let step = crate::scales::ticks::tick_step(start, stop, count);
    let mut spec = NumericSpecifier::parse(specifier.unwrap_or(",f"))?;
    let maximum = start.abs().max(stop.abs());
    let kind = spec.kind.unwrap_or('\0');
    let e = exponent(step.abs());
    let inferred = match kind {
        's' => (exponent(maximum) / 3.).floor().clamp(-8., 8.) * 3. - e,
        '\0' | 'e' | 'g' | 'p' | 'r' => {
            let delta = exponent(maximum.abs() - step.abs()) - e;
            if delta.is_nan() {
                f64::NAN
            } else {
                delta.max(0.) + 1. - f64::from(kind == 'e')
            }
        }
        'f' | '%' => {
            if e.is_nan() {
                f64::NAN
            } else {
                (-e).max(0.) - if kind == '%' { 2. } else { 0. }
            }
        }
        _ => f64::NAN,
    };
    if spec.precision.is_none() && !inferred.is_nan() {
        spec.precision = Some(if kind == 's' {
            inferred.max(0.)
        } else {
            inferred
        } as i32);
    }
    if kind == 's' {
        let e = (exponent(maximum) / 3.).floor().clamp(-8., 8.) * 3.;
        spec.kind = Some('f');
        let mut f = NumericFormatter::new(spec, locale)?;
        if e.is_nan() {
            f.scale = f64::NAN;
            f.suffix.clear();
            return Ok(f);
        }
        return Ok(f.prefix(e as i32));
    }
    NumericFormatter::new(spec, locale)
}
