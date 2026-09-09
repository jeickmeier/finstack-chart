use super::text::{TextInterpolator, numeric_text};
use super::value::time_clip;
use super::{
    ChartResult, ColorInterpolator, ColorRoute, DiagnosticCode, MAX_VALUE_BYTES, Number,
    NumericArray, NumericKind, Sample, ScalarInterpolator, Value, error, parameter,
};
use crate::color::{ColorSpace, ColorValue};
use std::collections::BTreeMap;

/// Explicit entry points alongside target-driven generic dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValueOperation {
    /// Select the operation from the target kind.
    Value,
    /// Pair numeric tokens after explicit primitive text conversion.
    String,
    /// Epoch milliseconds with Date TimeClip semantics.
    Date,
    /// Target-shaped general arrays, preserving a typed numeric destination.
    Array,
    /// Numeric prefix blending with a typed or ordinary numeric-array destination.
    NumberArray,
    /// Target-owned fields; explicit arrays expose their numeric-index fields.
    Object,
}
/// Compiled recursive interpolation. Sampling does not parse endpoints or rebuild factories.
#[derive(Clone, Debug)]
pub struct ValueInterpolator {
    kernel: Kernel,
}
#[derive(Clone, Debug)]
enum Kernel {
    Constant(Value),
    Number(ScalarInterpolator),
    Date(ScalarInterpolator),
    Text(TextInterpolator),
    Color(ColorInterpolator),
    NumericArray {
        element: Option<NumericKind>,
        target: Vec<Number>,
        prefix: Vec<ScalarInterpolator>,
    },
    Array(Vec<Kernel>),
    Record(BTreeMap<String, Kernel>),
}
impl ValueInterpolator {
    pub(super) fn color_sample(&self, t: f64) -> ChartResult<Option<ColorValue>> {
        match &self.kernel {
            Kernel::Color(f) => f.scale_color(t).map(Some),
            _ => Ok(None),
        }
    }
    pub(super) fn scale_sample(&self, t: f64) -> Value {
        let mut value = Value::Missing;
        self.kernel.sample_into(t, &mut value);
        value
    }
    /// Compile target-kind dispatch with bounded owned endpoints.
    pub fn new(a: &Value, b: &Value) -> ChartResult<Self> {
        Self::with_operation(ValueOperation::Value, a, b)
    }
    /// Compile an explicit operation. No host language callbacks or prototype coercions run.
    pub fn with_operation(op: ValueOperation, a: &Value, b: &Value) -> ChartResult<Self> {
        a.validate()?;
        b.validate()?;
        let kernel = compile(op, a, b)?;
        if kernel.max_bytes() > MAX_VALUE_BYTES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Sampled interpolation text can exceed its aggregate byte budget.",
            ));
        }
        Ok(Self { kernel })
    }
    /// Return an independently owned result; it survives subsequent samples and disposal.
    pub fn sample(&self, t: f64) -> ChartResult<Value> {
        let mut v = Value::Missing;
        self.sample_into(t, &mut v)?;
        Ok(v)
    }
    /// Explicitly overwrite a caller-owned destination, reusing string/array/record
    /// storage where its kind permits. Invalid parameters leave it unchanged.
    pub fn sample_into(&self, t: f64, destination: &mut Value) -> ChartResult<()> {
        parameter(t)?;
        self.kernel.sample_into(t, destination);
        Ok(())
    }
}
impl Sample<Value> for ValueInterpolator {
    fn sample(&self, t: f64) -> ChartResult<Value> {
        self.sample(t)
    }
}
pub(super) fn numeric(v: &Value) -> ChartResult<f64> {
    Ok(match v {
        Value::Missing => f64::NAN,
        Value::Null => 0.,
        Value::Boolean(v) => {
            if *v {
                1.
            } else {
                0.
            }
        }
        Value::Number(v) | Value::Date(v) => v.0,
        Value::Text(s) => numeric_text(s),
        _ => {
            return Err(error(
                DiagnosticCode::Validation,
                "This typed value has no primitive numeric conversion.",
            ));
        }
    })
}
fn text(v: &Value) -> ChartResult<String> {
    Ok(match v {
        Value::Missing => "undefined".into(),
        Value::Null => "null".into(),
        Value::Boolean(v) => v.to_string(),
        Value::Number(v) => crate::number::ecmascript(v.0),
        Value::Text(s) => s.clone(),
        Value::Color(v) => v.format_rgb(),
        _ => {
            return Err(error(
                DiagnosticCode::Validation,
                "Structured/date values require explicit text before string interpolation.",
            ));
        }
    })
}
pub(super) fn color(v: &Value) -> ChartResult<ColorValue> {
    match v {
        Value::Color(v) => Ok(*v),
        Value::Text(s) => Ok(crate::color::parse_with_limit(s, MAX_VALUE_BYTES)?
            .unwrap_or_else(|| ColorValue::undefined(ColorSpace::Rgb))),
        Value::Missing | Value::Null => Ok(ColorValue::undefined(ColorSpace::Rgb)),
        _ => Err(error(
            DiagnosticCode::Validation,
            "Color interpolation requires a color or CSS text.",
        )),
    }
}
fn array(v: &Value) -> ChartResult<Vec<Value>> {
    match v {
        Value::Array(v) => Ok(v.clone()),
        Value::NumericArray(v) => Ok(v.values.iter().copied().map(Value::Number).collect()),
        Value::Missing | Value::Null => Ok(vec![]),
        _ => Err(error(
            DiagnosticCode::Validation,
            "Array interpolation requires an array or an explicit empty source.",
        )),
    }
}
fn record(v: &Value) -> BTreeMap<String, Value> {
    match v {
        Value::Record(v) => v.clone(),
        Value::Array(v) => v
            .iter()
            .enumerate()
            .map(|(i, v)| (i.to_string(), v.clone()))
            .collect(),
        Value::NumericArray(v) => v
            .values
            .iter()
            .enumerate()
            .map(|(i, v)| (i.to_string(), Value::Number(*v)))
            .collect(),
        _ => BTreeMap::new(),
    }
}
fn compile(op: ValueOperation, a: &Value, b: &Value) -> ChartResult<Kernel> {
    match op {
        ValueOperation::String => {
            return Ok(Kernel::Text(TextInterpolator::new(&text(a)?, &text(b)?)?));
        }
        ValueOperation::Date => {
            return Ok(Kernel::Date(ScalarInterpolator::number(
                numeric(a)?,
                numeric(b)?,
            )));
        }
        ValueOperation::NumberArray => return numeric_array(a, b),
        ValueOperation::Array => {
            return if matches!(b, Value::NumericArray(_)) {
                numeric_array(a, b)
            } else {
                generic_array(a, b)
            };
        }
        ValueOperation::Object => {
            let a = record(a);
            let b = record(b);
            return b
                .into_iter()
                .map(|(k, v)| {
                    let f = if let Some(a) = a.get(&k) {
                        compile(ValueOperation::Value, a, &v)?
                    } else {
                        Kernel::Constant(v)
                    };
                    Ok((k, f))
                })
                .collect::<ChartResult<_>>()
                .map(Kernel::Record);
        }
        ValueOperation::Value => {}
    }
    Ok(match b {
        Value::Missing | Value::Null | Value::Boolean(_) => Kernel::Constant(b.clone()),
        Value::Number(b) => Kernel::Number(ScalarInterpolator::number(numeric(a)?, b.0)),
        Value::Date(b) => Kernel::Date(ScalarInterpolator::number(numeric(a)?, b.0)),
        Value::Text(b) => {
            if let Some(b) = crate::color::parse_with_limit(b, MAX_VALUE_BYTES)? {
                Kernel::Color(ColorInterpolator::new(ColorRoute::Rgb, color(a)?, b, None)?)
            } else {
                Kernel::Text(TextInterpolator::new(&text(a)?, b)?)
            }
        }
        Value::Color(b) => Kernel::Color(ColorInterpolator::new(
            ColorRoute::Rgb,
            color(a)?,
            *b,
            None,
        )?),
        Value::NumericArray(_) => return numeric_array(a, b),
        Value::Array(_) => return generic_array(a, b),
        Value::Record(_) => return compile(ValueOperation::Object, a, b),
    })
}
fn generic_array(a: &Value, b: &Value) -> ChartResult<Kernel> {
    let a = array(a)?;
    let b = array(b)?;
    b.into_iter()
        .enumerate()
        .map(|(i, b)| {
            if let Some(a) = a.get(i) {
                compile(ValueOperation::Value, a, &b)
            } else {
                Ok(Kernel::Constant(b))
            }
        })
        .collect::<ChartResult<_>>()
        .map(Kernel::Array)
}
fn numeric_array(a: &Value, b: &Value) -> ChartResult<Kernel> {
    let element = if let Value::NumericArray(b) = b {
        Some(b.element)
    } else {
        None
    };
    let a = array(a)?;
    let b = array(b)?;
    let target = b
        .iter()
        .map(numeric)
        .map(|v| v.map(Number))
        .collect::<ChartResult<Vec<_>>>()?;
    let prefix = a
        .iter()
        .zip(&target)
        .map(|(a, b)| Ok(ScalarInterpolator::number(numeric(a)?, b.0)))
        .collect::<ChartResult<_>>()?;
    Ok(Kernel::NumericArray {
        element,
        target,
        prefix,
    })
}
fn owned_clone(from: &Value, to: &mut Value) {
    match (from, to) {
        (Value::Text(a), Value::Text(b)) => b.clone_from(a),
        (Value::Array(a), Value::Array(b)) => {
            b.resize_with(a.len(), || Value::Missing);
            for (a, b) in a.iter().zip(b) {
                owned_clone(a, b);
            }
        }
        (Value::Record(a), Value::Record(b)) => {
            b.retain(|k, _| a.contains_key(k));
            for (k, v) in a {
                owned_clone(v, b.entry(k.clone()).or_insert(Value::Missing));
            }
        }
        (Value::NumericArray(a), Value::NumericArray(b)) => {
            b.element = a.element;
            b.values.clone_from(&a.values);
        }
        (from, to) => *to = from.clone(),
    }
}
impl Kernel {
    fn max_bytes(&self) -> usize {
        match self {
            Self::Text(v) => v.max_bytes,
            Self::Color(_) => 128,
            Self::Constant(v) => value_bytes(v),
            Self::Record(v) => v
                .iter()
                .map(|(k, v)| k.len().saturating_add(v.max_bytes()))
                .fold(0, usize::saturating_add),
            Self::Array(v) => v.iter().map(Self::max_bytes).fold(0, usize::saturating_add),
            _ => 0,
        }
    }
    fn sample_into(&self, t: f64, dst: &mut Value) {
        match self {
            Self::Constant(v) => owned_clone(v, dst),
            Self::Number(f) => *dst = Value::number(f.evaluate(t)),
            Self::Date(f) => *dst = Value::Date(Number(time_clip(f.evaluate(t)))),
            Self::Text(f) => {
                if !matches!(dst, Value::Text(_)) {
                    *dst = Value::Text(String::new());
                }
                let Value::Text(out) = dst else {
                    unreachable!()
                };
                f.sample_into(t, out);
            }
            Self::Color(f) => {
                let s = f
                    .scale_color(t)
                    .expect("compiled channel interpolation")
                    .format_rgb();
                if let Value::Text(out) = dst {
                    out.clear();
                    out.push_str(&s);
                } else {
                    *dst = Value::Text(s);
                }
            }
            Self::Array(v) => {
                if !matches!(dst, Value::Array(_)) {
                    *dst = Value::Array(vec![]);
                }
                let Value::Array(out) = dst else {
                    unreachable!()
                };
                out.resize_with(v.len(), || Value::Missing);
                for (f, v) in v.iter().zip(out) {
                    f.sample_into(t, v);
                }
            }
            Self::Record(v) => {
                if !matches!(dst, Value::Record(_)) {
                    *dst = Value::Record(BTreeMap::new());
                }
                let Value::Record(out) = dst else {
                    unreachable!()
                };
                out.retain(|k, _| v.contains_key(k));
                for (k, f) in v {
                    f.sample_into(t, out.entry(k.clone()).or_insert(Value::Missing));
                }
            }
            Self::NumericArray {
                element,
                target,
                prefix,
            } => {
                if let Some(kind) = element {
                    if !matches!(dst, Value::NumericArray(_)) {
                        *dst = Value::NumericArray(NumericArray {
                            element: *kind,
                            values: vec![],
                        });
                    }
                    let Value::NumericArray(out) = dst else {
                        unreachable!()
                    };
                    out.element = *kind;
                    out.values.clone_from(target);
                    for (f, v) in prefix.iter().zip(&mut out.values) {
                        v.0 = kind.cast(f.evaluate(t));
                    }
                } else {
                    if !matches!(dst, Value::Array(_)) {
                        *dst = Value::Array(vec![]);
                    }
                    let Value::Array(out) = dst else {
                        unreachable!()
                    };
                    out.resize_with(target.len(), || Value::Missing);
                    for (i, v) in target.iter().enumerate() {
                        out[i] = Value::Number(*v);
                    }
                    for (f, v) in prefix.iter().zip(out) {
                        *v = Value::number(f.evaluate(t));
                    }
                }
            }
        }
    }
}
fn value_bytes(v: &Value) -> usize {
    match v {
        Value::Text(s) => s.len(),
        Value::Array(v) => v.iter().map(value_bytes).fold(0, usize::saturating_add),
        Value::Record(v) => v
            .iter()
            .map(|(k, v)| k.len().saturating_add(value_bytes(v)))
            .fold(0, usize::saturating_add),
        _ => 0,
    }
}
