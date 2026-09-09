use super::{curves, domain, invalid, limit};
use crate::{ChartResult, path::Path};

/// Complete pinned curve selection; parameters retain their finite authored values.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum CurveSpec {
    /// Straight segments, including the reference singleton close.
    #[default]
    Linear,
    /// Closed straight segments.
    LinearClosed,
    /// Clamped cubic basis spline.
    Basis,
    /// Open cubic basis spline.
    BasisOpen,
    /// Periodic cubic basis spline.
    BasisClosed,
    /// Smooth x-oriented links between adjacent points.
    BumpX,
    /// Smooth y-oriented links between adjacent points.
    BumpY,
    /// Line-only basis bundle with beta defaulting to 0.85.
    Bundle {
        /// Finite straight-line versus basis blend; not implicitly clamped.
        #[serde(default = "default_beta")]
        beta: f64,
    },
    /// Clamped cardinal spline with zero default tension.
    Cardinal {
        /// Finite tension; not implicitly clamped.
        #[serde(default)]
        tension: f64,
    },
    /// Open cardinal spline.
    CardinalOpen {
        /// Finite tension; not implicitly clamped.
        #[serde(default)]
        tension: f64,
    },
    /// Periodic cardinal spline.
    CardinalClosed {
        /// Finite tension; not implicitly clamped.
        #[serde(default)]
        tension: f64,
    },
    /// Clamped Catmull-Rom spline with alpha defaulting to 0.5.
    CatmullRom {
        /// Finite parameterization exponent; zero selects uniform cardinal.
        #[serde(default = "default_alpha")]
        alpha: f64,
    },
    /// Open Catmull-Rom spline.
    CatmullRomOpen {
        /// Finite parameterization exponent.
        #[serde(default = "default_alpha")]
        alpha: f64,
    },
    /// Periodic Catmull-Rom spline.
    CatmullRomClosed {
        /// Finite parameterization exponent.
        #[serde(default = "default_alpha")]
        alpha: f64,
    },
    /// Steffen monotone cubic interpolation along x.
    MonotoneX,
    /// Reflected Steffen monotone interpolation along y.
    MonotoneY,
    /// Natural cubic spline with zero endpoint second derivatives.
    Natural,
    /// Midpoint step interpolation.
    Step,
    /// Step before each new x coordinate.
    StepBefore,
    /// Step after each new x coordinate.
    StepAfter,
}
// Internally tagged Serde unit variants otherwise ignore unexpected parameter fields.
// Decode the exact parameter surface explicitly before accepting portable configuration.
impl<'de> serde::Deserialize<'de> for CurveSpec {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let mut fields =
            std::collections::BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let kind = fields
            .remove("kind")
            .and_then(|v| v.as_str().map(str::to_owned))
            .ok_or_else(|| D::Error::custom("A curve requires a string kind."))?;
        fn parameter<E: serde::de::Error>(
            fields: &mut std::collections::BTreeMap<String, serde_json::Value>,
            name: &str,
            default: f64,
        ) -> Result<f64, E> {
            fields
                .remove(name)
                .map(serde_json::from_value::<f64>)
                .transpose()
                .map(|v| v.unwrap_or(default))
                .map_err(E::custom)
        }
        let result = match kind.as_str() {
            "Linear" => Self::Linear,
            "LinearClosed" => Self::LinearClosed,
            "Basis" => Self::Basis,
            "BasisOpen" => Self::BasisOpen,
            "BasisClosed" => Self::BasisClosed,
            "BumpX" => Self::BumpX,
            "BumpY" => Self::BumpY,
            "Bundle" => Self::Bundle {
                beta: parameter::<D::Error>(&mut fields, "beta", default_beta())?,
            },
            "Cardinal" => Self::Cardinal {
                tension: parameter::<D::Error>(&mut fields, "tension", 0.)?,
            },
            "CardinalOpen" => Self::CardinalOpen {
                tension: parameter::<D::Error>(&mut fields, "tension", 0.)?,
            },
            "CardinalClosed" => Self::CardinalClosed {
                tension: parameter::<D::Error>(&mut fields, "tension", 0.)?,
            },
            "CatmullRom" => Self::CatmullRom {
                alpha: parameter::<D::Error>(&mut fields, "alpha", default_alpha())?,
            },
            "CatmullRomOpen" => Self::CatmullRomOpen {
                alpha: parameter::<D::Error>(&mut fields, "alpha", default_alpha())?,
            },
            "CatmullRomClosed" => Self::CatmullRomClosed {
                alpha: parameter::<D::Error>(&mut fields, "alpha", default_alpha())?,
            },
            "MonotoneX" => Self::MonotoneX,
            "MonotoneY" => Self::MonotoneY,
            "Natural" => Self::Natural,
            "Step" => Self::Step,
            "StepBefore" => Self::StepBefore,
            "StepAfter" => Self::StepAfter,
            _ => return Err(D::Error::custom("Unknown curve kind.")),
        };
        if !fields.is_empty() {
            return Err(D::Error::custom(
                "Unknown parameter for the selected curve kind.",
            ));
        }
        result.validate().map_err(D::Error::custom)?;
        Ok(result)
    }
}

fn default_beta() -> f64 {
    0.85
}
fn default_alpha() -> f64 {
    0.5
}
impl CurveSpec {
    /// Reject non-finite factory parameters without changing their finite semantics.
    pub fn validate(self) -> ChartResult<()> {
        let parameter = match self {
            Self::Bundle { beta } => Some(beta),
            Self::Cardinal { tension }
            | Self::CardinalOpen { tension }
            | Self::CardinalClosed { tension } => Some(tension),
            Self::CatmullRom { alpha }
            | Self::CatmullRomOpen { alpha }
            | Self::CatmullRomClosed { alpha } => Some(alpha),
            _ => None,
        };
        if parameter.is_some_and(|value| !value.is_finite()) {
            return Err(domain("Curve parameters must be finite."));
        }
        Ok(())
    }
    /// Whether this curve can receive paired area boundaries under the shape contract.
    pub fn supports_area(self) -> bool {
        !matches!(self, Self::Bundle { .. })
    }
    /// Create a reusable checked native lifecycle over a borrowed numeric path.
    /// Input points are bounded across all runs in this context; path limits also apply.
    pub fn context(self, path: &mut Path, max_points: usize) -> ChartResult<CurveContext<'_>> {
        self.validate()?;
        Ok(CurveContext {
            inner: curves::create(self, path),
            max_points,
            points: 0,
            area: false,
            line: false,
            failed: false,
            supports_area: self.supports_area(),
        })
    }
}

/// Native curve lifecycle. Custom curves write through the existing checked path API.
/// A failed native sink may retain its accepted prefix; owned generators return no result.
pub trait CurveProtocol {
    /// Begin paired area boundaries; line-only curves explicitly reject this operation.
    fn area_start(&mut self) -> ChartResult<()> {
        Err(invalid("This curve supports lines only."))
    }
    /// Finish paired area boundaries.
    fn area_end(&mut self) -> ChartResult<()> {
        Err(invalid("This curve supports lines only."))
    }
    /// Begin an authored-order run.
    fn line_start(&mut self) -> ChartResult<()>;
    /// Finish the current run and emit any delayed endpoint controls.
    fn line_end(&mut self) -> ChartResult<()>;
    /// Consume a finite projected point in the active run.
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()>;
}

/// Reusable native curve factory, independent of charts and host interpreter objects.
pub trait CurveFactory {
    /// Conservative command bound for this many total input points, including any
    /// partition into gapped runs. None leaves native standalone output bounded only
    /// by PathLimits; registered chart curves must declare a bound before layout.
    fn command_bound(&self, _max_points: usize) -> Option<usize> {
        None
    }
    /// Construct a curve over the supplied checked destination, bounded by input points.
    fn create<'a>(
        &'a self,
        path: &'a mut Path,
        max_points: usize,
    ) -> ChartResult<Box<dyn CurveProtocol + 'a>>;
    /// Report unsupported area use before any drawing begins.
    fn supports_area(&self) -> bool;
}
impl CurveFactory for CurveSpec {
    fn command_bound(&self, max_points: usize) -> Option<usize> {
        max_points.checked_mul(3)
    }
    fn create<'a>(
        &'a self,
        path: &'a mut Path,
        max_points: usize,
    ) -> ChartResult<Box<dyn CurveProtocol + 'a>> {
        Ok(Box::new(self.context(path, max_points)?))
    }
    fn supports_area(&self) -> bool {
        (*self).supports_area()
    }
}

/// Checked built-in curve lifecycle with finite input, ordering and work limits.
pub struct CurveContext<'a> {
    inner: Box<dyn CurveProtocol + 'a>,
    max_points: usize,
    points: usize,
    area: bool,
    line: bool,
    failed: bool,
    supports_area: bool,
}
impl CurveContext<'_> {
    fn ready(&self) -> ChartResult<()> {
        if self.failed {
            Err(invalid("A failed curve context cannot be reused."))
        } else {
            Ok(())
        }
    }
    fn record(&mut self, result: ChartResult<()>) -> ChartResult<()> {
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
impl CurveProtocol for CurveContext<'_> {
    fn area_start(&mut self) -> ChartResult<()> {
        self.ready()?;
        if self.area || self.line || !self.supports_area {
            return Err(invalid(
                "Curve areaStart requires an idle area-capable context.",
            ));
        }
        let result = self.inner.area_start();
        self.record(result)?;
        self.area = true;
        Ok(())
    }
    fn area_end(&mut self) -> ChartResult<()> {
        self.ready()?;
        if !self.area || self.line {
            return Err(invalid("Curve areaEnd requires a completed area run."));
        }
        let result = self.inner.area_end();
        self.record(result)?;
        self.area = false;
        Ok(())
    }
    fn line_start(&mut self) -> ChartResult<()> {
        self.ready()?;
        if self.line {
            return Err(invalid("Curve lineStart cannot nest active runs."));
        }
        let result = self.inner.line_start();
        self.record(result)?;
        self.line = true;
        Ok(())
    }
    fn line_end(&mut self) -> ChartResult<()> {
        self.ready()?;
        if !self.line {
            return Err(invalid("Curve lineEnd requires an active run."));
        }
        let result = self.inner.line_end();
        self.record(result)?;
        self.line = false;
        Ok(())
    }
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()> {
        self.ready()?;
        if !self.line {
            return Err(invalid("Curve points require an active run."));
        }
        if !x.is_finite() || !y.is_finite() {
            return Err(domain("Curve point coordinates must be finite."));
        }
        if self.points >= self.max_points {
            return Err(limit("Curve input point limit exceeded."));
        }
        let result = self.inner.point(x, y);
        self.record(result)?;
        self.points += 1;
        Ok(())
    }
}
