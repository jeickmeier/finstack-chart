//! Bounded typed values and explicit numerical transport, independent of a host VM.
use super::{ChartResult, DiagnosticCode, MAX_VALUES, error};
use crate::color::ColorValue;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use crate::number::Number;

/// Supported Number-based array destinations; the name is a portable type, not a host object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NumericKind {
    /// IEEE binary32, nearest-even cast.
    #[serde(rename = "Float32Array")]
    Float32,
    /// IEEE binary64.
    #[serde(rename = "Float64Array")]
    Float64,
    /// Signed eight-bit wrap.
    #[serde(rename = "Int8Array")]
    Int8,
    /// Unsigned eight-bit wrap.
    #[serde(rename = "Uint8Array")]
    Uint8,
    /// Saturated eight-bit, nearest with even ties.
    #[serde(rename = "Uint8ClampedArray")]
    Uint8Clamped,
    /// Signed sixteen-bit wrap.
    #[serde(rename = "Int16Array")]
    Int16,
    /// Unsigned sixteen-bit wrap.
    #[serde(rename = "Uint16Array")]
    Uint16,
    /// Signed thirty-two-bit wrap.
    #[serde(rename = "Int32Array")]
    Int32,
    /// Unsigned thirty-two-bit wrap.
    #[serde(rename = "Uint32Array")]
    Uint32,
}
impl NumericKind {
    /// Apply the destination cast, preserving the reference's wrap/clamp distinction.
    pub fn cast(self, v: f64) -> f64 {
        match self {
            Self::Float64 => v,
            Self::Float32 => f64::from(v as f32),
            Self::Uint8Clamped => {
                if v.is_nan() || v <= 0. {
                    return 0.;
                }
                if v >= 255. {
                    return 255.;
                }
                let floor = v.floor();
                let fract = v - floor;
                if fract > 0.5 || fract == 0.5 && floor % 2. != 0. {
                    floor + 1.
                } else {
                    floor
                }
            }
            kind => {
                if !v.is_finite() || v == 0. {
                    return 0.;
                }
                let (modulus, signed) = match kind {
                    Self::Int8 => (256., true),
                    Self::Uint8 => (256., false),
                    Self::Int16 => (65536., true),
                    Self::Uint16 => (65536., false),
                    Self::Int32 => (4294967296., true),
                    Self::Uint32 => (4294967296., false),
                    _ => unreachable!(),
                };
                let mut v = v.trunc() % modulus;
                if v < 0. {
                    v += modulus;
                }
                if signed && v >= modulus / 2. {
                    v -= modulus;
                }
                if v == 0. { 0. } else { v }
            }
        }
    }
}
/// An owned numeric array with already applied destination casting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "NumericArrayWire", into = "NumericArrayWire")]
pub struct NumericArray {
    pub(super) element: NumericKind,
    pub(super) values: Vec<Number>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NumericArrayWire {
    element: NumericKind,
    values: Vec<Number>,
}
impl TryFrom<NumericArrayWire> for NumericArray {
    type Error = String;
    fn try_from(v: NumericArrayWire) -> Result<Self, String> {
        Self::new(v.element, v.values.into_iter().map(|v| v.0).collect()).map_err(|e| e.to_string())
    }
}
impl From<NumericArray> for NumericArrayWire {
    fn from(v: NumericArray) -> Self {
        Self {
            element: v.element,
            values: v.values,
        }
    }
}
impl NumericArray {
    /// Cast and retain a bounded input sequence.
    pub fn new(element: NumericKind, values: Vec<f64>) -> ChartResult<Self> {
        super::count(values.len(), 0)?;
        Ok(Self {
            element,
            values: values
                .into_iter()
                .map(|v| Number(element.cast(v)))
                .collect(),
        })
    }
    /// Destination element kind.
    pub fn element(&self) -> NumericKind {
        self.element
    }
    /// Owned numeric payloads; exceptional Float32/64 values remain classified.
    pub fn values(&self) -> &[Number] {
        &self.values
    }
}

/// A typed interpolation value. No arbitrary object coercion or prototypes are executable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum Value {
    /// An explicitly missing value, distinct from null.
    Missing,
    /// Explicit null.
    Null,
    /// Constant truth value.
    Boolean(bool),
    /// Binary64 numerical boundary.
    Number(Number),
    /// Owned UTF-8 text.
    Text(String),
    /// Clipped integer epoch milliseconds or invalid-date NaN.
    Date(Number),
    /// Floating authored color, preserving its space.
    Color(ColorValue),
    /// Destination-typed numeric array.
    NumericArray(NumericArray),
    /// Recursive general array.
    Array(Vec<Value>),
    /// Deterministically ordered string-keyed own fields; duplicate keys reject on decode.
    Record(#[serde(deserialize_with = "record")] BTreeMap<String, Value>),
}
fn record<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Value>, D::Error> {
    struct Visitor;
    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = BTreeMap<String, Value>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("unique string-keyed fields")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut out = BTreeMap::new();
            while let Some((k, v)) = map.next_entry::<String, Value>()? {
                if out.insert(k, v).is_some() {
                    return Err(serde::de::Error::custom(
                        "Duplicate interpolation record key.",
                    ));
                }
            }
            Ok(out)
        }
    }
    d.deserialize_map(Visitor)
}
/// Maximum aggregate text bytes in an input or sampled value.
pub const MAX_VALUE_BYTES: usize = 4 * 1024 * 1024;
/// Maximum recursive typed value depth.
pub const MAX_VALUE_DEPTH: usize = 32;
impl Value {
    /// Construct a binary64 scalar without changing its exceptional classification.
    pub fn number(v: f64) -> Self {
        Self::Number(Number(v))
    }
    /// Construct a Date-compatible value with TimeClip/truncation semantics.
    pub fn date(ms: f64) -> Self {
        Self::Date(Number(time_clip(ms)))
    }
    /// Validate aggregate nodes/bytes/depth and canonical date states before compilation.
    pub fn validate(&self) -> ChartResult<()> {
        let mut nodes = 0;
        let mut bytes = 0;
        self.budget(0, &mut nodes, &mut bytes)
    }
    pub(crate) fn budget(
        &self,
        depth: usize,
        nodes: &mut usize,
        bytes: &mut usize,
    ) -> ChartResult<()> {
        *nodes += 1;
        if depth > MAX_VALUE_DEPTH {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Interpolation nesting exceeds its budget.",
            ));
        }
        match self {
            Self::Text(s) => *bytes = bytes.saturating_add(s.len()),
            Self::Date(v) => {
                if !v.0.is_nan() && (v.0 != time_clip(v.0) || v.0 == 0. && v.0.is_sign_negative()) {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Date payload must be clipped integer milliseconds or NaN.",
                    ));
                }
            }
            Self::Array(v) => {
                for v in v {
                    v.budget(depth + 1, nodes, bytes)?;
                }
            }
            Self::NumericArray(v) => *nodes = nodes.saturating_add(v.values.len()),
            Self::Record(v) => {
                for (k, v) in v {
                    *bytes = bytes.saturating_add(k.len());
                    v.budget(depth + 1, nodes, bytes)?;
                }
            }
            _ => {}
        }
        if *nodes > MAX_VALUES || *bytes > MAX_VALUE_BYTES {
            Err(error(
                DiagnosticCode::ResourceLimit,
                "Interpolation value exceeds its aggregate node/text budget.",
            ))
        } else {
            Ok(())
        }
    }
    /// Strict bounded value decoding; unknown fields, kinds and duplicate record keys reject.
    pub fn from_json(json: &str) -> ChartResult<Self> {
        let v: Self = crate::portable::decode(json)?;
        v.validate()?;
        Ok(v)
    }
    /// Encode exceptional states without ambiguous JSON null.
    pub fn to_json(&self) -> ChartResult<String> {
        self.validate()?;
        let json = crate::portable::encode(self)?;
        if json.len() > MAX_VALUE_BYTES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Encoded interpolation output exceeds its byte budget.",
            ));
        }
        Ok(json)
    }
}
pub(super) fn time_clip(ms: f64) -> f64 {
    if !ms.is_finite() || ms.abs() > 8.64e15 {
        f64::NAN
    } else {
        let t = ms.trunc();
        if t == 0. { 0. } else { t }
    }
}
