//! R 4.6.1 date_labels patterns with explicitly supplied calendar and locale resources.
use super::*;

/// Explicit ggplot date/time labels under the R C-locale pattern profile.
///
/// Missing locale uses C names and patterns. `%OS` defaults to zero fractional
/// digits (R's `digits.secs = 0`); explicit precision is capped at six.
/// `%s` depends on R's process timezone and is rejected. `%Z` requires UTC:
/// local abbreviation resources are not inferred from offsets or zone names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotTimeFormat {
    /// Required R date/time format pattern.
    pub pattern: String,
    /// Optional explicit locale; no process locale is read.
    #[serde(default)]
    pub locale: Option<TimeLocale>,
}
impl GgplotTimeFormat {
    /// Compile against the same immutable calendar used by the time axis.
    pub fn prepare(&self, calendar: Calendar) -> ChartResult<TimeFormatter> {
        let locale = self.locale.clone().unwrap_or_else(|| TimeLocale {
            date_time: "%a %b %e %H:%M:%S %Y".into(),
            date: "%m/%d/%y".into(),
            time: "%H:%M:%S".into(),
            ..TimeLocale::default()
        });
        locale.validate()?;
        let mut tokens = Vec::new();
        // R locates the first raw %OS before strftime parses escaped percent signs.
        if let Some(index) = self.pattern.find("%OS") {
            let mut prefix = &self.pattern[..index];
            if prefix.chars().rev().take_while(|c| *c == '%').count() % 2 == 1 {
                prefix = &prefix[..prefix.len() - 1];
            }
            append(prefix, &locale, &calendar, 0, &mut tokens)?;
            let suffix = &self.pattern[index + 3..];
            let precision = suffix.as_bytes().first().filter(|b| b.is_ascii_digit());
            tokens.push(Token::Fraction(
                precision.map_or(0, |b| u32::from(b - b'0').min(6)),
            ));
            append(
                &suffix[usize::from(precision.is_some())..],
                &locale,
                &calendar,
                0,
                &mut tokens,
            )?;
        } else {
            append(&self.pattern, &locale, &calendar, 0, &mut tokens)?;
        }
        if self.pattern.len() > 4096 || tokens.len() > 1024 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "R time pattern exceeds its byte or token budget.",
            ));
        }
        Ok(TimeFormatter {
            calendar,
            spec: Arc::new(TimeFormat {
                pattern: Some(self.pattern.clone()),
                locale,
            }),
            pattern: Some(tokens),
            defaults: std::array::from_fn(|_| Vec::new()),
            reference: Some(Arc::new(self.clone())),
        })
    }
}
fn append(
    pattern: &str,
    locale: &TimeLocale,
    calendar: &Calendar,
    depth: usize,
    out: &mut Vec<Token>,
) -> ChartResult<()> {
    if depth > 8 || pattern.len() > 4096 || pattern.contains('\0') {
        return Err(error(
            DiagnosticCode::Validation,
            "R time pattern exceeds its expansion budget or contains NUL.",
        ));
    }
    let mut chars = pattern.chars();
    let mut text = String::new();
    while let Some(c) = chars.next() {
        if c != '%' {
            text.push(c);
            continue;
        }
        if !text.is_empty() {
            out.push(Token::Text(std::mem::take(&mut text)));
        }
        let Some(mut field) = chars.next() else {
            out.push(Token::Text("%".into()));
            break;
        };
        if matches!(field, 'E' | 'O') {
            field = chars.next().unwrap_or(field);
        }
        let expansion = match field {
            'c' => Some(locale.date_time.as_str()),
            'x' => Some(locale.date.as_str()),
            'X' => Some(locale.time.as_str()),
            'F' => Some("%Y-%m-%d"),
            'D' => Some("%m/%d/%y"),
            'r' => Some("%I:%M:%S %p"),
            'R' => Some("%H:%M"),
            'T' => Some("%H:%M:%S"),
            'v' => Some("%e-%b-%Y"),
            '+' => Some("%a %b %e %X %Z %Y"),
            _ => None,
        };
        if let Some(expansion) = expansion {
            append(expansion, locale, calendar, depth + 1, out)?;
        } else {
            match field {
                's' => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "R %s requires an explicit process-timezone interpretation.",
                    ));
                }
                'Z' => {
                    if !matches!(calendar.zone(), CalendarZone::Utc) {
                        return Err(error(
                            DiagnosticCode::UnsupportedCapability,
                            "R %Z requires explicit local timezone abbreviations; only UTC is available.",
                        ));
                    }
                    out.push(Token::Text("UTC".into()));
                }
                'n' => out.push(Token::Text("\n".into())),
                't' => out.push(Token::Text("\t".into())),
                'z' => out.push(Token::Field('Z', Some('0'))),
                'k' => out.push(Token::Field('H', Some(' '))),
                'l' => out.push(Token::Field('I', Some(' '))),
                'h' => out.push(Token::Field('b', None)),
                'a' | 'A' | 'b' | 'B' | 'C' | 'd' | 'e' | 'H' | 'I' | 'j' | 'm' | 'M' | 'S'
                | 'p' | 'P' | 'u' | 'w' | 'U' | 'W' | 'V' | 'g' | 'G' | 'y' | 'Y' | '%' => out
                    .push(Token::Field(
                        field,
                        Some(if field == 'e' { ' ' } else { '0' }),
                    )),
                _ => out.push(Token::Text(field.to_string())),
            }
        }
        if out.len() > 1024 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "R time pattern token expansion exceeds its budget.",
            ));
        }
    }
    if !text.is_empty() {
        out.push(Token::Text(text));
    }
    Ok(())
}
