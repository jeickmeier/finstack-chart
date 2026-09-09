//! Version 1 portable contracts. JSON contains no pointers or executable operations.
//! Public decoders cap UTF-8 bytes/tokens/depth before deserialization and use core validation.
mod encoding;
mod input;
pub use input::{InputOperation, InputQuery, NavigationWire, SelectionWire};
mod scales;
mod session;
mod stream;
pub use scales::{ScaleChange, ScaleQuery};
pub use stream::{StreamEnvelope, StreamOperation};
mod wire;
use crate::{ChartResult, Diagnostic, DiagnosticCode};
pub(crate) use encoding::{floats, signed, signed_vec, unsigned, unsigned_vec, wire_identity};
use serde::{Serialize, de::DeserializeOwned};
pub use session::Session;
pub use wire::*;
/// Maximum UTF-8 bytes per input envelope, before parsing/owned allocation.
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;
/// Supported envelope version. Version zero and future versions reject; no migrations exist yet.
pub const VERSION: u32 = 1;
pub(crate) fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use the documented version 1 portable subset, explicit resource bytes, canonical decimal 64-bit strings and current revisions.",
    )
}
/// Decode a bounded strict DTO. Validation of a complete chart/data operation occurs at Session.
pub fn decode<T: DeserializeOwned>(input: &str) -> ChartResult<T> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Portable envelope exceeds 4 MiB",
        ));
    }
    let (mut quoted, mut escape, mut depth, mut tokens) = (false, false, 0i32, 0usize);
    for b in input.bytes() {
        if quoted {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                quoted = false;
            }
            continue;
        }
        match b {
            b'"' => {
                quoted = true;
                tokens += 1;
            }
            b'{' | b'[' => {
                depth += 1;
                tokens += 1;
            }
            b'}' | b']' => depth -= 1,
            b',' | b':' => tokens += 1,
            _ => {}
        }
        if depth > 32 || tokens > 200_000 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Portable nesting/token budget exceeded",
            ));
        }
    }
    serde_json::from_str(input).map_err(|e| {
        error(
            DiagnosticCode::Validation,
            format!("Invalid portable envelope: {e}"),
        )
    })
}
/// Serialize a portable DTO. This does not serialize native callbacks or host objects.
pub fn encode<T: Serialize>(value: &T) -> ChartResult<String> {
    serde_json::to_string(value).map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
}
fn version(v: u32) -> ChartResult<()> {
    if v != VERSION {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            format!("Envelope version {v} is unsupported; supported range is 1..=1"),
        ));
    }
    Ok(())
}

mod shapes;
pub use shapes::{
    pie_layout_json, pie_layout_registered_json, stack_layout_json, stack_layout_registered_json,
};
