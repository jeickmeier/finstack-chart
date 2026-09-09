//! Strict standalone descriptors shared by native callers, hosts and future scale consumers.
use super::*;
use crate::{color::ColorValue, path::Affine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Supported binary factories. Each maps to one independently qualified reference export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactoryKind {
    /// Target-kind dispatch.
    Value,
    /// Weighted numerical interpolation.
    Number,
    /// Numerical interpolation with JS rounding.
    Round,
    /// Numeric-token string interpolation.
    String,
    /// Date-compatible milliseconds.
    Date,
    /// General target-shaped arrays.
    Array,
    /// Numeric prefix interpolation preserving destination kind.
    NumberArray,
    /// Target-owned record fields.
    Object,
    /// Shortest normalized hue.
    Hue,
    /// Floating RGB, optional gamma.
    Rgb,
    /// Shortest HSL hue.
    Hsl,
    /// Direct HSL hue.
    HslLong,
    /// D50 Lab.
    Lab,
    /// Shortest cylindrical Lab hue.
    Hcl,
    /// Direct cylindrical Lab hue.
    HclLong,
    /// Shortest Cubehelix hue, optional lightness gamma.
    Cubehelix,
    /// Direct Cubehelix hue, optional lightness gamma.
    CubehelixLong,
}
/// A serializable built-in factory; native custom `Sample` implementations remain separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterpolationFactory {
    /// Binary operation.
    pub kind: FactoryKind,
    /// Optional RGB/Cubehelix gamma; other routes reject this configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gamma: Option<Number>,
}
impl InterpolationFactory {
    /// Select a factory with its reference default configuration.
    pub fn new(kind: FactoryKind) -> Self {
        Self { kind, gamma: None }
    }
    /// Compile a bounded pair with the shared engine.
    pub fn between(self, a: Value, b: Value) -> ChartResult<Interpolator> {
        Interpolator::new(InterpolationSpec::Between {
            factory: self,
            a,
            b,
        })
    }
}
/// Version-one operation payload; authored values are retained without backend objects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum InterpolationSpec {
    /// Version-one named chromatic evaluator, independent of domain normalization.
    Chromatic {
        /// Stable catalog identity and reversal.
        spec: crate::scales::chromatic::ChromaticSpec,
    },
    /// Binary factory and endpoints.
    Between {
        /// Built-in configuration.
        factory: InterpolationFactory,
        /// Source value.
        a: Value,
        /// Target value.
        b: Value,
    },
    /// Uniform scalar basis.
    Basis {
        /// Control sequence.
        values: Vec<Number>,
        /// Wrap when true, clamp open endpoints otherwise.
        closed: bool,
    },
    /// Opaque RGB spline using the same scalar basis.
    RgbBasis {
        /// Floating colors or color text.
        values: Vec<Value>,
        /// Closed cyclic basis when true.
        closed: bool,
    },
    /// Saturated discrete selection.
    Discrete {
        /// Nonempty owned targets.
        values: Vec<Value>,
    },
    /// Adjacent binary factories, compiled once per pair.
    Piecewise {
        /// Factory shared by every segment.
        factory: InterpolationFactory,
        /// Two or more control values.
        values: Vec<Value>,
    },
    /// Explicitly resolved finite affine matrices.
    Transform {
        /// Source matrix coefficients.
        a: [f64; 6],
        /// Target matrix coefficients.
        b: [f64; 6],
        /// Canonical output syntax.
        syntax: TransformSyntax,
    },
    /// Bounded headless absolute transform text.
    TransformText {
        /// Source transform list.
        a: String,
        /// Target transform list.
        b: String,
        /// Input and output syntax.
        syntax: TransformSyntax,
    },
    /// Smooth camera trajectory with signed duration metadata.
    Zoom {
        /// Source view.
        a: ZoomView,
        /// Target view.
        b: ZoomView,
        /// Finite custom rho; omitted selects exact default constants.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rho: Option<Number>,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    version: u32,
    spec: InterpolationSpec,
}
#[derive(Serialize)]
struct BorrowedWire<'a> {
    version: u32,
    spec: &'a InterpolationSpec,
}
/// An owned prepared standalone operation, preserving its versioned input descriptor.
#[derive(Clone, Debug)]
pub struct Interpolator {
    spec: Arc<InterpolationSpec>,
    compiled: Arc<Compiled>,
}
#[derive(Debug)]
enum Compiled {
    Chromatic(crate::scales::chromatic::ChromaticRamp),
    Scalar(ScalarInterpolator),
    Hue(HueInterpolator),
    Color(ColorInterpolator),
    Value(ValueInterpolator),
    Discrete(Discrete<Value>),
    Piecewise(Piecewise<Interpolator>),
    Transform(TransformInterpolator),
    Zoom(ZoomInterpolator),
}
impl Interpolator {
    /// Validate and compile a descriptor once; sampling never re-parses it.
    pub fn new(spec: InterpolationSpec) -> ChartResult<Self> {
        let compiled = compile(&spec)?;
        Ok(Self {
            spec: Arc::new(spec),
            compiled: Arc::new(compiled),
        })
    }
    /// Original immutable operation descriptor, independent of later caller mutations.
    pub fn spec(&self) -> &InterpolationSpec {
        &self.spec
    }
    /// Strict version-one standalone decoding with existing byte/depth/token budgets.
    pub fn from_json(json: &str) -> ChartResult<Self> {
        let wire: Wire = crate::portable::decode(json)?;
        if wire.version != 1 {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported interpolation descriptor version; expected 1.",
            ));
        }
        Self::new(wire.spec)
    }
    /// Serialize the authored operation, never generated samples or native callbacks.
    pub fn to_json(&self) -> ChartResult<String> {
        let json = crate::portable::encode(&BorrowedWire {
            version: 1,
            spec: &self.spec,
        })?;
        if json.len() > MAX_VALUE_BYTES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Interpolation descriptor exceeds its byte budget.",
            ));
        }
        Ok(json)
    }
    /// Sample an independently owned typed result.
    pub fn sample(&self, t: f64) -> ChartResult<Value> {
        parameter(t)?;
        match self.compiled.as_ref() {
            Compiled::Chromatic(f) => f
                .evaluate(t)
                .map(|c| Value::Color(crate::color::Paint::from(c).value())),
            Compiled::Scalar(f) => f.sample(t).map(Value::number),
            Compiled::Hue(f) => f.sample(t).map(Value::number),
            Compiled::Color(f) => f.sample(t).map(Value::Text),
            Compiled::Value(f) => f.sample(t),
            Compiled::Discrete(f) => f.sample(t),
            Compiled::Piecewise(f) => f.sample(t),
            Compiled::Transform(f) => f.sample(t).map(Value::Text),
            Compiled::Zoom(f) => f
                .sample(t)
                .map(|v| Value::Array(v.values().into_iter().map(Value::number).collect())),
        }
    }
    // Scale transforms can yield IEEE exceptional parameters. Keep this separate from
    // the public finite-parameter animation contract; reuse the already compiled kernels.
    pub(crate) fn scale_sample(&self, t: f64) -> ChartResult<Value> {
        match self.compiled.as_ref() {
            Compiled::Chromatic(f) => f
                .evaluate_scale(t)
                .map(|c| Value::Color(crate::color::Paint::from(c).value())),
            Compiled::Scalar(f) => Ok(Value::number(f.evaluate(t))),
            Compiled::Color(f) => f.scale_color(t).map(|v| Value::Text(v.format_rgb())),
            Compiled::Value(f) => Ok(f.scale_sample(t)),
            Compiled::Piecewise(f) => {
                let (f, t) = f.scale_segment(t)?;
                f.scale_sample(t)
            }
            _ => self.sample(t),
        }
    }
    pub(crate) fn scale_color(&self, t: f64) -> ChartResult<ColorValue> {
        match self.compiled.as_ref() {
            Compiled::Chromatic(f) => f
                .evaluate_scale(t)
                .map(|c| crate::color::Paint::from(c).value()),
            Compiled::Color(f) => f.scale_color(t),
            Compiled::Value(f) => f.color_sample(t)?.ok_or_else(not_color),
            Compiled::Piecewise(f) => {
                let (f, t) = f.scale_segment(t)?;
                f.scale_color(t)
            }
            _ => self.sample_color(t),
        }
    }
    /// Explicit destination overwrite; recursive value factories reuse capacity.
    pub fn sample_into(&self, t: f64, destination: &mut Value) -> ChartResult<()> {
        parameter(t)?;
        match self.compiled.as_ref() {
            Compiled::Value(f) => f.sample_into(t, destination),
            Compiled::Piecewise(f) => {
                let (f, t) = f.segment(t)?;
                f.sample_into(t, destination)
            }
            _ => {
                *destination = self.sample(t)?;
                Ok(())
            }
        }
    }
    /// Floating color sampling for scales/paint preparation, before byte formatting.
    pub fn sample_color(&self, t: f64) -> ChartResult<ColorValue> {
        parameter(t)?;
        match self.compiled.as_ref() {
            Compiled::Chromatic(f) => f.evaluate(t).map(|c| crate::color::Paint::from(c).value()),
            Compiled::Color(f) => f.sample_color(t),
            Compiled::Value(f) => f.color_sample(t)?.ok_or_else(not_color),
            Compiled::Piecewise(f) => {
                let (f, t) = f.segment(t)?;
                f.sample_color(t)
            }
            _ => Err(not_color()),
        }
    }
    /// Resolved sampled affine matrix, available independently of an animation.
    pub fn sample_transform(&self, t: f64) -> ChartResult<Affine> {
        match self.compiled.as_ref() {
            Compiled::Transform(f) => f.matrix(t),
            _ => Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Interpolator does not produce a transform.",
            )),
        }
    }
    /// Signed reference zoom duration; other operations have no duration metadata.
    pub fn duration_ms(&self) -> Option<f64> {
        match self.compiled.as_ref() {
            Compiled::Zoom(f) => Some(f.duration_ms()),
            _ => None,
        }
    }
    /// Explicit nonnegative host scheduling adaptation of a zoom duration.
    pub fn scheduling_duration_ms(&self) -> Option<f64> {
        self.duration_ms().map(f64::abs)
    }
    /// Bounded owned samples including both endpoints, without the reference's array/date aliasing.
    pub fn quantize(&self, samples: usize) -> ChartResult<Vec<Value>> {
        count(samples, 2)?;
        let mut out = Vec::with_capacity(samples);
        let mut nodes = 1;
        let mut bytes = 0;
        for i in 0..samples {
            let v = self.sample(i as f64 / (samples - 1) as f64)?;
            v.budget(1, &mut nodes, &mut bytes)?;
            out.push(v);
        }
        Ok(out)
    }
}
impl Sample<Value> for Interpolator {
    fn sample(&self, t: f64) -> ChartResult<Value> {
        self.sample(t)
    }
}
fn not_color() -> crate::Diagnostic {
    error(
        DiagnosticCode::UnsupportedCapability,
        "Interpolator does not produce floating colors.",
    )
}
fn values_budget(values: &[Value], minimum: usize) -> ChartResult<()> {
    count(values.len(), minimum)?;
    let mut nodes = 1;
    let mut bytes = 0;
    for v in values {
        v.budget(1, &mut nodes, &mut bytes)?;
    }
    Ok(())
}
fn compile(spec: &InterpolationSpec) -> ChartResult<Compiled> {
    Ok(match spec {
        InterpolationSpec::Chromatic { spec } => {
            Compiled::Chromatic(crate::scales::chromatic::ChromaticRamp::new(*spec)?)
        }
        InterpolationSpec::Between { factory, a, b } => {
            a.validate()?;
            b.validate()?;
            let route = match factory.kind {
                FactoryKind::Rgb => Some(ColorRoute::Rgb),
                FactoryKind::Hsl => Some(ColorRoute::Hsl),
                FactoryKind::HslLong => Some(ColorRoute::HslLong),
                FactoryKind::Lab => Some(ColorRoute::Lab),
                FactoryKind::Hcl => Some(ColorRoute::Hcl),
                FactoryKind::HclLong => Some(ColorRoute::HclLong),
                FactoryKind::Cubehelix => Some(ColorRoute::Cubehelix),
                FactoryKind::CubehelixLong => Some(ColorRoute::CubehelixLong),
                _ => None,
            };
            if let Some(route) = route {
                Compiled::Color(ColorInterpolator::new(
                    route,
                    structured::color(a)?,
                    structured::color(b)?,
                    factory.gamma.map(|v| v.0),
                )?)
            } else {
                if factory.gamma.is_some() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Gamma is only supported by the RGB/Cubehelix factories.",
                    ));
                }
                match factory.kind {
                    FactoryKind::Number => Compiled::Scalar(ScalarInterpolator::number(
                        structured::numeric(a)?,
                        structured::numeric(b)?,
                    )),
                    FactoryKind::Round => Compiled::Scalar(ScalarInterpolator::round(
                        structured::numeric(a)?,
                        structured::numeric(b)?,
                    )),
                    FactoryKind::Hue => Compiled::Hue(HueInterpolator::new(
                        structured::numeric(a)?,
                        structured::numeric(b)?,
                    )),
                    kind => {
                        let op = match kind {
                            FactoryKind::Value => ValueOperation::Value,
                            FactoryKind::String => ValueOperation::String,
                            FactoryKind::Date => ValueOperation::Date,
                            FactoryKind::Array => ValueOperation::Array,
                            FactoryKind::NumberArray => ValueOperation::NumberArray,
                            FactoryKind::Object => ValueOperation::Object,
                            _ => unreachable!(),
                        };
                        Compiled::Value(ValueInterpolator::with_operation(op, a, b)?)
                    }
                }
            }
        }
        InterpolationSpec::Basis { values, closed } => {
            count(values.len(), if *closed { 1 } else { 2 })?;
            let v = values.iter().map(|v| v.0).collect();
            Compiled::Scalar(if *closed {
                ScalarInterpolator::basis_closed(v)?
            } else {
                ScalarInterpolator::basis(v)?
            })
        }
        InterpolationSpec::RgbBasis { values, closed } => {
            values_budget(values, if *closed { 1 } else { 2 })?;
            let values = values
                .iter()
                .map(structured::color)
                .collect::<ChartResult<Vec<_>>>()?;
            Compiled::Color(ColorInterpolator::rgb_basis(&values, *closed)?)
        }
        InterpolationSpec::Discrete { values } => {
            values_budget(values, 1)?;
            Compiled::Discrete(Discrete::new(values.clone())?)
        }
        InterpolationSpec::Piecewise { factory, values } => {
            values_budget(values, 2)?;
            Compiled::Piecewise(Piecewise::new(values, |a, b| {
                factory.between(a.clone(), b.clone())
            })?)
        }
        InterpolationSpec::Transform { a, b, syntax } => Compiled::Transform(
            TransformInterpolator::new(Affine::new(*a)?, Affine::new(*b)?, *syntax)?,
        ),
        InterpolationSpec::TransformText { a, b, syntax } => {
            Compiled::Transform(TransformInterpolator::from_text(a, b, *syntax)?)
        }
        InterpolationSpec::Zoom { a, b, rho } => Compiled::Zoom(if let Some(rho) = rho {
            ZoomInterpolator::with_rho(*a, *b, rho.0)?
        } else {
            ZoomInterpolator::new(*a, *b)?
        }),
    })
}
