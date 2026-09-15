//! Strict standalone descriptors shared by native callers, hosts and future scale consumers.
use super::*;
use crate::{color::ColorValue, path::Affine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Supported binary factories. Each maps to one independently qualified reference export.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactoryKind {
    /// Explicitly installed versioned native factory.
    Registered,
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
/// Versioned registration and bounded declarative parameters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterpolationRegistration {
    /// Exact installed implementation identity.
    pub operation: crate::grammar::OperationRef,
    /// Checked native factory configuration.
    pub parameters: serde_json::Value,
}
/// A built-in or explicitly registered factory. Built-in wire fields are unchanged.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterpolationFactory {
    /// Binary operation.
    pub kind: FactoryKind,
    /// Optional RGB/Cubehelix gamma; other routes reject this configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gamma: Option<Number>,
    /// Present only for `Registered`; endpoints remain in the surrounding descriptor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration: Option<Box<InterpolationRegistration>>,
}
impl InterpolationFactory {
    /// Select a factory with its reference default configuration.
    pub fn new(kind: FactoryKind) -> Self {
        Self {
            kind,
            gamma: None,
            registration: None,
        }
    }
    /// Select installed native code without serializing executable callbacks.
    pub fn registered(
        operation: crate::grammar::OperationRef,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            kind: FactoryKind::Registered,
            gamma: None,
            registration: Some(Box::new(InterpolationRegistration {
                operation,
                parameters,
            })),
        }
    }
    /// Compile a bounded pair with the shared engine and builtin-only registry.
    pub fn between(&self, a: Value, b: Value) -> ChartResult<Interpolator> {
        Interpolator::new(InterpolationSpec::Between {
            factory: self.clone(),
            a,
            b,
        })
    }
    /// Compile a pair against an explicitly supplied registry snapshot.
    pub fn between_with_registry(
        &self,
        a: Value,
        b: Value,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Interpolator> {
        Interpolator::new_with_registry(
            InterpolationSpec::Between {
                factory: self.clone(),
                a,
                b,
            },
            registry,
        )
    }
    pub(crate) fn has_registration(&self) -> bool {
        self.kind == FactoryKind::Registered || self.registration.is_some()
    }
    pub(crate) fn validate_registration(
        &self,
        registry: &crate::grammar::interpolation_extensions::InterpolationRegistrations,
        portable: bool,
    ) -> ChartResult<()> {
        match (&self.registration, self.kind) {
            (Some(r), FactoryKind::Registered) if self.gamma.is_none() => {
                registry.validate(&r.operation, &r.parameters, portable)
            }
            (None, kind) if kind != FactoryKind::Registered => Ok(()),
            _ => Err(error(
                DiagnosticCode::Validation,
                "Registered interpolation requires its registration and cannot use builtin gamma options.",
            )),
        }
    }
}
/// Version-one operation payload; authored values are retained without backend objects.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum InterpolationSpec {
    /// Version-three numeric power range used by radius/area style policies.
    PowerRange {
        /// Affine output range after exponentiation.
        range: [Number; 2],
        /// Finite exponent; one gives radius/width interpolation, half gives area sizing.
        exponent: Number,
        /// Take the absolute normalized parameter before exponentiation.
        absolute: bool,
    },
    /// ggplot2 palette using shared color/catalog/spline owners (v3; count gradients v4).
    GgplotPalette {
        /// Portable palette recipe; scales supply normalized positions.
        spec: crate::scales::chromatic::ggplot::PaletteSpec,
    },
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
    PowerRange(ScalarInterpolator),
    GgplotPalette(crate::scales::chromatic::ggplot::PaletteRamp),
    Chromatic(crate::scales::chromatic::ChromaticRamp),
    Scalar(ScalarInterpolator),
    Hue(HueInterpolator),
    Color(ColorInterpolator),
    Value(ValueInterpolator),
    Discrete(Discrete<Value>),
    Piecewise(Piecewise<Interpolator>),
    Transform(TransformInterpolator),
    Zoom(ZoomInterpolator),
    Registered(crate::grammar::interpolation_extensions::RegisteredInterpolator),
}
impl Interpolator {
    /// Validate and compile a descriptor once; sampling never re-parses it.
    pub fn new(spec: InterpolationSpec) -> ChartResult<Self> {
        Self::new_with_registrations(spec, &Default::default())
    }
    /// Compile against explicitly installed native implementations; retain prepared samplers.
    pub fn new_with_registry(
        spec: InterpolationSpec,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        Self::new_with_registrations(spec, &registry.interpolations)
    }
    pub(crate) fn new_with_registrations(
        spec: InterpolationSpec,
        registrations: &crate::grammar::interpolation_extensions::InterpolationRegistrations,
    ) -> ChartResult<Self> {
        // Apply the same aggregate descriptor budget before native factory callbacks.
        let encoded = crate::portable::encode(&spec)?;
        let _: InterpolationSpec = crate::portable::decode(&encoded)?;
        let compiled = compile(&spec, registrations)?;
        Ok(Self {
            spec: Arc::new(spec),
            compiled: Arc::new(compiled),
        })
    }
    /// Original immutable operation descriptor, independent of later caller mutations.
    pub fn spec(&self) -> &InterpolationSpec {
        &self.spec
    }
    /// Strict builtin-only decoding. Registered descriptors need `from_json_with_registry`.
    pub fn from_json(json: &str) -> ChartResult<Self> {
        Self::from_json_with_registry(json, &crate::grammar::ExtensionRegistry::new())
    }
    /// Decode a versioned descriptor with an explicitly installed registry.
    pub fn from_json_with_registry(
        json: &str,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        let wire: Wire = crate::portable::decode(json)?;
        if wire.version != wire.spec.wire_version() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Interpolation descriptor version does not match its capabilities.",
            ));
        }
        wire.spec
            .validate_registrations(&registry.interpolations, true)?;
        let result = Self::new_with_registry(wire.spec, registry)?;
        result.require_portable()?;
        Ok(result)
    }
    /// Bounded authored descriptor for another in-process consumer, without installing code.
    /// This is not a portable serialization claim; receiving consumers must validate registries.
    pub fn descriptor_json(&self) -> ChartResult<String> {
        let json = crate::portable::encode(&BorrowedWire {
            version: self.spec.wire_version(),
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
    /// Serialize only portable operations, never native-only callbacks or generated samples.
    pub fn to_json(&self) -> ChartResult<String> {
        self.require_portable()?;
        self.descriptor_json()
    }
    pub(crate) fn require_portable(&self) -> ChartResult<()> {
        let portable = match self.compiled.as_ref() {
            Compiled::Registered(f) => f.portable(),
            Compiled::Piecewise(f) => {
                for segment in f.segments() {
                    segment.require_portable()?;
                }
                true
            }
            _ => true,
        };
        if portable {
            Ok(())
        } else {
            Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Native-only interpolation cannot be serialized as a portable operation.",
            ))
        }
    }
    /// Sample an independently owned typed result.
    pub fn sample(&self, t: f64) -> ChartResult<Value> {
        parameter(t)?;
        match self.compiled.as_ref() {
            Compiled::Registered(f) => f.sample(t),
            Compiled::GgplotPalette(f) => palette_value(f, t),
            Compiled::Chromatic(f) => f
                .evaluate(t)
                .map(|c| Value::Color(crate::color::Paint::from(c).value())),
            Compiled::Scalar(f) => f.sample(t).map(Value::number),
            Compiled::PowerRange(f) => Ok(power_value(f.evaluate(t))),
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
            Compiled::GgplotPalette(f) => palette_value(f, t),
            Compiled::Chromatic(f) => f
                .evaluate_scale(t)
                .map(|c| Value::Color(crate::color::Paint::from(c).value())),
            Compiled::Scalar(f) => Ok(Value::number(f.evaluate(t))),
            Compiled::PowerRange(f) => Ok(power_value(f.evaluate(t))),
            Compiled::Color(f) => f.scale_color(t).map(|v| Value::Text(v.format_rgb())),
            Compiled::Value(f) => Ok(f.scale_sample(t)),
            Compiled::Piecewise(f) => {
                let (f, t) = f.scale_segment(t)?;
                f.scale_sample(t)
            }
            _ => self.sample(t),
        }
    }
    pub(crate) fn scale_optional_color(&self, t: f64) -> ChartResult<Option<ColorValue>> {
        match self.compiled.as_ref() {
            Compiled::GgplotPalette(f) => {
                Ok(f.sample(t)?.map(|c| crate::color::Paint::from(c).value()))
            }
            _ => self.scale_color(t).map(Some),
        }
    }
    pub(crate) fn scale_color(&self, t: f64) -> ChartResult<ColorValue> {
        match self.compiled.as_ref() {
            Compiled::GgplotPalette(f) => f
                .sample(t)?
                .map(|c| crate::color::Paint::from(c).value())
                .ok_or_else(not_color),
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
            Compiled::GgplotPalette(f) => f
                .sample(t)?
                .map(|c| crate::color::Paint::from(c).value())
                .ok_or_else(not_color),
            Compiled::Registered(f) => match f.sample(t)? {
                Value::Color(value) => Ok(value),
                Value::Text(text) => crate::color::parse(&text)?.ok_or_else(not_color),
                _ => Err(not_color()),
            },
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
fn power_value(v: f64) -> Value {
    if v.is_nan() {
        Value::Missing
    } else {
        Value::number(v)
    }
}
fn palette_value(f: &crate::scales::chromatic::ggplot::PaletteRamp, t: f64) -> ChartResult<Value> {
    Ok(f.sample(t)?.map_or(Value::Null, |c| {
        Value::Color(crate::color::Paint::from(c).value())
    }))
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
fn compile(
    spec: &InterpolationSpec,
    registrations: &crate::grammar::interpolation_extensions::InterpolationRegistrations,
) -> ChartResult<Compiled> {
    Ok(match spec {
        InterpolationSpec::PowerRange {
            range,
            exponent,
            absolute,
        } => Compiled::PowerRange(ScalarInterpolator::power(
            range[0].0, range[1].0, exponent.0, *absolute,
        )?),
        InterpolationSpec::GgplotPalette { spec } => {
            Compiled::GgplotPalette(crate::scales::chromatic::ggplot::PaletteRamp::new(spec)?)
        }
        InterpolationSpec::Chromatic { spec } => {
            Compiled::Chromatic(crate::scales::chromatic::ChromaticRamp::new(*spec)?)
        }
        InterpolationSpec::Between { factory, a, b } => {
            a.validate()?;
            b.validate()?;
            factory.validate_registration(registrations, false)?;
            if let Some(registration) = &factory.registration {
                return Ok(Compiled::Registered(registrations.compile(
                    &registration.operation,
                    crate::grammar::InterpolationInput {
                        source: a,
                        target: b,
                        parameters: &registration.parameters,
                    },
                )?));
            }
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
                Interpolator::new_with_registrations(
                    InterpolationSpec::Between {
                        factory: factory.clone(),
                        a: a.clone(),
                        b: b.clone(),
                    },
                    registrations,
                )
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

impl InterpolationSpec {
    /// Minimum standalone envelope version; builtins retain version one.
    pub fn wire_version(&self) -> u32 {
        if matches!(self, Self::GgplotPalette {
            spec: crate::scales::chromatic::ggplot::PaletteSpec::Gradient { values: Some(values), .. }
                | crate::scales::chromatic::ggplot::PaletteSpec::CountGradient { values: Some(values), .. }
        } if values.iter().any(|n| !n.0.is_finite() || n.0 == 0. && n.0.is_sign_negative()))
        {
            return 5;
        }
        if matches!(
            self,
            Self::GgplotPalette {
                spec: crate::scales::chromatic::ggplot::PaletteSpec::CountGradient { .. }
            }
        ) {
            4
        } else if matches!(self, Self::GgplotPalette { .. } | Self::PowerRange { .. }) {
            3
        } else if self.has_registration() {
            2
        } else {
            1
        }
    }
    /// Whether the descriptor requires an explicitly installed factory.
    pub fn has_registration(&self) -> bool {
        match self {
            Self::Between { factory, .. } | Self::Piecewise { factory, .. } => {
                factory.has_registration()
            }
            _ => false,
        }
    }
    pub(crate) fn validate_registrations(
        &self,
        registry: &crate::grammar::interpolation_extensions::InterpolationRegistrations,
        portable: bool,
    ) -> ChartResult<()> {
        match self {
            Self::Between { factory, .. } | Self::Piecewise { factory, .. } => {
                factory.validate_registration(registry, portable)
            }
            _ => Ok(()),
        }
    }
}
