//! Checked reference-name overloads for actual host constructors; all math stays in core.
use super::*;
use serde::{Deserialize, Serialize};

/// Optional built-in factory configuration; irrelevant options reject explicitly.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterpolationOptions {
    /// RGB or Cubehelix gamma.
    #[serde(default)]
    pub gamma: Option<Number>,
    /// Smooth zoom rho.
    #[serde(default)]
    pub rho: Option<Number>,
    /// Piecewise binary factory; omitted means target-driven dispatch.
    #[serde(default)]
    pub factory: Option<InterpolationFactory>,
}
impl FactoryKind {
    /// Resolve one of the pinned binary exports; no arbitrary operation code is loaded.
    pub fn from_name(name: &str) -> ChartResult<Self> {
        Ok(match name {
            "interpolate" => Self::Value,
            "interpolateNumber" => Self::Number,
            "interpolateRound" => Self::Round,
            "interpolateString" => Self::String,
            "interpolateDate" => Self::Date,
            "interpolateArray" => Self::Array,
            "interpolateNumberArray" => Self::NumberArray,
            "interpolateObject" => Self::Object,
            "interpolateHue" => Self::Hue,
            "interpolateRgb" => Self::Rgb,
            "interpolateHsl" => Self::Hsl,
            "interpolateHslLong" => Self::HslLong,
            "interpolateLab" => Self::Lab,
            "interpolateHcl" => Self::Hcl,
            "interpolateHclLong" => Self::HclLong,
            "interpolateCubehelix" => Self::Cubehelix,
            "interpolateCubehelixLong" => Self::CubehelixLong,
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Unknown binary interpolation factory.",
                ));
            }
        })
    }
}
impl InterpolationFactory {
    /// Validate an optional gamma configuration before compiling any endpoints.
    pub fn with_gamma(mut self, gamma: f64) -> ChartResult<Self> {
        if !matches!(
            self.kind,
            FactoryKind::Rgb | FactoryKind::Cubehelix | FactoryKind::CubehelixLong
        ) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "This factory has no gamma configuration.",
            ));
        }
        if !gamma.is_finite() || gamma <= 0. {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Gamma must be positive and finite.",
            ));
        }
        self.gamma = Some(Number(gamma));
        Ok(self)
    }
}
fn arity(args: &[Value], n: usize) -> ChartResult<()> {
    if args.len() == n {
        Ok(())
    } else {
        Err(error(
            DiagnosticCode::Validation,
            format!("Interpolation constructor expects {n} arguments."),
        ))
    }
}
fn values(v: &Value) -> ChartResult<Vec<Value>> {
    match v {
        Value::Array(v) => Ok(v.clone()),
        Value::NumericArray(v) => Ok(v.values().iter().copied().map(Value::Number).collect()),
        _ => Err(error(
            DiagnosticCode::Validation,
            "Interpolation controls must be an array.",
        )),
    }
}
fn numbers(v: &Value) -> ChartResult<Vec<Number>> {
    values(v)?
        .iter()
        .map(|v| match v {
            Value::Number(v) => Ok(*v),
            _ => Err(error(
                DiagnosticCode::Validation,
                "Numeric controls must contain numbers.",
            )),
        })
        .collect()
}
fn plain_options(options: InterpolationOptions) -> ChartResult<()> {
    if options.gamma.is_some() || options.rho.is_some() || options.factory.is_some() {
        Err(error(
            DiagnosticCode::Validation,
            "This interpolation constructor has no such options.",
        ))
    } else {
        Ok(())
    }
}
impl Interpolator {
    /// Bounded constructor transport used by both thin host adapters.
    pub fn construct_json(name: &str, args: &str, options: &str) -> ChartResult<Self> {
        let args: Vec<Value> = crate::portable::decode(args)?;
        let options: InterpolationOptions = crate::portable::decode(options)?;
        Self::construct(name, &args, options)
    }
    /// Resolve pinned reference overloads for native and portable callers. The input
    /// values are typed, bounded and owned; no host parser or numerical implementation is used.
    pub fn construct(
        name: &str,
        args: &[Value],
        options: InterpolationOptions,
    ) -> ChartResult<Self> {
        let mut nodes = 1;
        let mut bytes = 0;
        for v in args {
            v.budget(1, &mut nodes, &mut bytes)?;
        }
        let spec = match name {
            "interpolateBasis" | "interpolateBasisClosed" => {
                arity(args, 1)?;
                plain_options(options)?;
                InterpolationSpec::Basis {
                    values: numbers(&args[0])?,
                    closed: name.ends_with("Closed"),
                }
            }
            "interpolateRgbBasis" | "interpolateRgbBasisClosed" => {
                arity(args, 1)?;
                plain_options(options)?;
                InterpolationSpec::RgbBasis {
                    values: values(&args[0])?,
                    closed: name.ends_with("Closed"),
                }
            }
            "interpolateDiscrete" => {
                arity(args, 1)?;
                plain_options(options)?;
                InterpolationSpec::Discrete {
                    values: values(&args[0])?,
                }
            }
            "piecewise" => {
                arity(args, 1)?;
                if options.gamma.is_some() || options.rho.is_some() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Piecewise configuration belongs to its binary factory.",
                    ));
                }
                InterpolationSpec::Piecewise {
                    factory: options
                        .factory
                        .unwrap_or(InterpolationFactory::new(FactoryKind::Value)),
                    values: values(&args[0])?,
                }
            }
            "interpolateZoom" => {
                arity(args, 2)?;
                if options.gamma.is_some() || options.factory.is_some() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Zoom accepts only rho configuration.",
                    ));
                }
                let view = |v: &Value| {
                    let v = numbers(v)?;
                    if v.len() != 3 {
                        return Err(error(
                            DiagnosticCode::Validation,
                            "Zoom view needs exactly three numbers.",
                        ));
                    }
                    ZoomView::new([v[0].0, v[1].0, v[2].0])
                };
                InterpolationSpec::Zoom {
                    a: view(&args[0])?,
                    b: view(&args[1])?,
                    rho: options.rho,
                }
            }
            "interpolateTransformCss" | "interpolateTransformSvg" => {
                arity(args, 2)?;
                plain_options(options)?;
                let syntax = if name.ends_with("Css") {
                    TransformSyntax::Css
                } else {
                    TransformSyntax::Svg
                };
                if matches!(args[0], Value::Array(_) | Value::NumericArray(_))
                    && matches!(args[1], Value::Array(_) | Value::NumericArray(_))
                {
                    let matrix = |v: &Value| {
                        let v = numbers(v)?;
                        if v.len() != 6 {
                            return Err(error(
                                DiagnosticCode::Validation,
                                "Affine input needs six coefficients.",
                            ));
                        }
                        Ok(std::array::from_fn(|i| v[i].0))
                    };
                    InterpolationSpec::Transform {
                        a: matrix(&args[0])?,
                        b: matrix(&args[1])?,
                        syntax,
                    }
                } else {
                    let text = |v: &Value| match v {
                        Value::Text(s) => Ok(s.clone()),
                        Value::Null | Value::Missing if syntax == TransformSyntax::Svg => {
                            Ok(String::new())
                        }
                        _ => Err(error(
                            DiagnosticCode::Validation,
                            "Use two transform texts or two resolved matrices.",
                        )),
                    };
                    InterpolationSpec::TransformText {
                        a: text(&args[0])?,
                        b: text(&args[1])?,
                        syntax,
                    }
                }
            }
            _ => {
                arity(args, 2)?;
                if options.rho.is_some() || options.factory.is_some() {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Binary interpolation does not accept rho or nested factories.",
                    ));
                }
                InterpolationSpec::Between {
                    factory: InterpolationFactory {
                        kind: FactoryKind::from_name(name)?,
                        gamma: options.gamma,
                    },
                    a: args[0].clone(),
                    b: args[1].clone(),
                }
            }
        };
        Self::new(spec)
    }
}
