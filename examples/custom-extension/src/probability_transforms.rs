//! Example quantile/CDF pair adaptation of scales::transform_probability.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    grammar::{CustomTransformFactory, ExtensionDescriptor, ExtensionRegistry, PointwiseTransform},
    interpolate::Number,
};
use std::sync::Arc;

#[derive(serde::Deserialize)]
#[serde(tag = "distribution", deny_unknown_fields)]
enum Probability {
    Uniform { min: f64, max: f64 },
    Exponential { rate: f64 },
}
fn parameters(value: &serde_json::Value) -> ChartResult<Probability> {
    let p: Probability = serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))?;
    let valid = match p {
        Probability::Uniform { min, max } => min.is_finite() && max.is_finite() && min < max,
        Probability::Exponential { rate } => rate.is_finite() && rate > 0.,
    };
    if !valid {
        return Err(super::error(
            DiagnosticCode::NumericalDomain,
            "Invalid probability transform parameters.",
        ));
    }
    Ok(p)
}
impl PointwiseTransform for Probability {
    fn forward(&self, p: f64) -> f64 {
        if !(0. ..=1.).contains(&p) {
            return f64::NAN;
        }
        match *self {
            Self::Uniform { min, max } => min + p * (max - min),
            Self::Exponential { rate } => -libm::log1p(-p) / rate,
        }
    }
    fn inverse(&self, x: f64) -> f64 {
        if x.is_nan() {
            return x;
        }
        match *self {
            Self::Uniform { min, max } => ((x - min) / (max - min)).clamp(0., 1.),
            Self::Exponential { rate } => {
                if x <= 0. {
                    0.
                } else {
                    -libm::expm1(-rate * x)
                }
            }
        }
    }
    fn domain(&self) -> [Number; 2] {
        [Number(0.), Number(1.)]
    }
    fn monotone_on(&self, _: [f64; 2]) -> bool {
        true
    }
}
struct Factory;
impl CustomTransformFactory for Factory {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.probability_transform", Revision::new(1), true)
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        parameters(p).map(|_| ())
    }
    fn compile(&self, p: &serde_json::Value) -> ChartResult<Arc<dyn PointwiseTransform>> {
        Ok(Arc::new(parameters(p)?))
    }
}
pub(crate) fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_transform(Arc::new(Factory))
}
