//! Portable pointwise factories corresponding to registered-transforms.json.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    grammar::{CustomTransformFactory, ExtensionDescriptor, ExtensionRegistry, PointwiseTransform},
    interpolate::Number,
};
use std::sync::Arc;
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Family {
    Affine,
    Cubic,
    Square,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    family: Family,
    custom: bool,
}
fn parameters(value: &serde_json::Value) -> ChartResult<Parameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
struct Kernel(Parameters);
impl PointwiseTransform for Kernel {
    fn forward(&self, x: f64) -> f64 {
        match self.0.family {
            Family::Affine => 2. * x + 3.,
            Family::Cubic => x * x * x,
            Family::Square => x * x,
        }
    }
    fn inverse(&self, x: f64) -> f64 {
        match self.0.family {
            Family::Affine => (x - 3.) / 2.,
            Family::Square => libm::sqrt(x),
            Family::Cubic => {
                if x == 0. {
                    x
                } else {
                    x.signum() * libm::pow(x.abs(), 1. / 3.)
                }
            }
        }
    }
    fn domain(&self) -> [Number; 2] {
        [
            if matches!(self.0.family, Family::Square) {
                0.
            } else {
                f64::NEG_INFINITY
            },
            f64::INFINITY,
        ]
        .map(Number)
    }
    fn monotone_on(&self, bounds: [f64; 2]) -> bool {
        !matches!(self.0.family, Family::Square) || bounds.iter().all(|v| *v >= 0.)
    }
    fn breaks(&self, _: [f64; 2], _: f64) -> ChartResult<Option<Vec<Number>>> {
        Ok(self
            .0
            .custom
            .then(|| [-2., -1., 0., 1., 2.].map(Number).to_vec()))
    }
    fn labels(&self, values: &[Number]) -> ChartResult<Option<Vec<Option<String>>>> {
        if !self.0.custom {
            return Ok(None);
        }
        if values.is_empty() {
            return Ok(Some(vec![Some("value=".into())]));
        }
        let finite = values
            .iter()
            .filter(|v| v.0.is_finite())
            .map(|v| v.0)
            .collect::<Vec<_>>();
        let labels = chart_core::typography::ggplot_numeric_labels(&finite, 65536)?;
        let mut labels = labels.into_iter();
        Ok(Some(
            values
                .iter()
                .map(|v| {
                    Some(format!(
                        "value={}",
                        if v.0.is_finite() {
                            labels.next().unwrap()
                        } else if v.0.is_nan() {
                            "NA".into()
                        } else if v.0 > 0. {
                            "Inf".into()
                        } else {
                            "-Inf".into()
                        }
                    ))
                })
                .collect(),
        ))
    }
}
/// Example-only affine/cubic/square factory, installed separately from the core engine.
pub struct Transform {
    /// Whether this installed identity permits portable publication.
    pub portable: bool,
}
impl CustomTransformFactory for Transform {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                "example.scale_transform"
            } else {
                "example.native_scale_transform"
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        parameters(p).map(|_| ())
    }
    fn compile(&self, p: &serde_json::Value) -> ChartResult<Arc<dyn PointwiseTransform>> {
        Ok(Arc::new(Kernel(parameters(p)?)))
    }
}
pub(crate) fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_transform(Arc::new(Transform { portable: true }))?;
    registry.register_transform(Arc::new(Transform { portable: false }))?;
    registry.register_transform(Arc::new(MinorTransform))?;
    registry.register_transform(Arc::new(VectorTransform))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum MinorMode {
    Default,
    Fixed,
    Limits,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MinorParameters {
    family: Family,
    mode: MinorMode,
}
fn minor_parameters(value: &serde_json::Value) -> ChartResult<MinorParameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
struct MinorKernel {
    base: Kernel,
    mode: MinorMode,
}
impl PointwiseTransform for MinorKernel {
    fn forward(&self, x: f64) -> f64 {
        self.base.forward(x)
    }
    fn inverse(&self, x: f64) -> f64 {
        self.base.inverse(x)
    }
    fn domain(&self) -> [Number; 2] {
        self.base.domain()
    }
    fn monotone_on(&self, bounds: [f64; 2]) -> bool {
        self.base.monotone_on(bounds)
    }
    fn minor_breaks(
        &self,
        _: &[Number],
        limits: [Number; 2],
        _: usize,
    ) -> ChartResult<Option<Vec<Number>>> {
        Ok(match self.mode {
            MinorMode::Default => None,
            MinorMode::Fixed => Some(vec![Number(-1.), Number(0.), Number(1.)]),
            MinorMode::Limits => Some(vec![
                limits[0],
                Number((limits[0].0 + limits[1].0) / 2.),
                limits[1],
            ]),
        })
    }
}
/// Separate versioned demonstration of transform-provided minor-break defaults.
struct MinorTransform;
impl CustomTransformFactory for MinorTransform {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.scale_transform_minor", Revision::new(1), true)
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        minor_parameters(p).map(|_| ())
    }
    fn compile(&self, p: &serde_json::Value) -> ChartResult<Arc<dyn PointwiseTransform>> {
        let p = minor_parameters(p)?;
        Ok(Arc::new(MinorKernel {
            base: Kernel(Parameters {
                family: p.family,
                custom: false,
            }),
            mode: p.mode,
        }))
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum VectorFamily {
    Cardinality,
    Center,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorParameters {
    family: VectorFamily,
}
fn vector_parameters(value: &serde_json::Value) -> ChartResult<VectorParameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
struct VectorKernel(VectorParameters);
impl chart_core::grammar::PreparedTransform for VectorKernel {
    fn forward(&self, x: f64) -> f64 {
        match self.0.family {
            VectorFamily::Cardinality => x + 1.,
            VectorFamily::Center => {
                if x.is_finite() {
                    0.
                } else {
                    f64::NAN
                }
            }
        }
    }
    fn inverse(&self, x: f64) -> f64 {
        match self.0.family {
            VectorFamily::Cardinality => x - 1.,
            VectorFamily::Center => x,
        }
    }
    fn is_pointwise(&self) -> bool {
        false
    }
    fn forward_batch(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
        let offset = match self.0.family {
            VectorFamily::Cardinality => -(values.len() as f64),
            VectorFamily::Center => {
                let finite = values
                    .iter()
                    .copied()
                    .filter(|v| !v.is_nan())
                    .collect::<Vec<_>>();
                finite.iter().sum::<f64>() / finite.len() as f64
            }
        };
        Ok(values.iter().map(|x| x - offset).collect())
    }
    fn inverse_batch(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
        let offset = match self.0.family {
            VectorFamily::Cardinality => values.len() as f64,
            VectorFamily::Center => 0.,
        };
        Ok(values.iter().map(|x| x - offset).collect())
    }
    fn inverse_null(&self) -> ChartResult<Option<Vec<f64>>> {
        Ok(match self.0.family {
            VectorFamily::Cardinality => Some(vec![]),
            VectorFamily::Center => None,
        })
    }
    fn domain(&self) -> [Number; 2] {
        [f64::NEG_INFINITY, f64::INFINITY].map(Number)
    }
    fn monotone_on(&self, _: [f64; 2]) -> bool {
        false
    }
}
/// Example population-coupled arithmetic; no host objects or second registry.
pub struct VectorTransform;
impl CustomTransformFactory for VectorTransform {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch("example.scale_transform_vector", Revision::new(1), true)
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        vector_parameters(p).map(|_| ())
    }
    fn compile(&self, p: &serde_json::Value) -> ChartResult<Arc<dyn PointwiseTransform>> {
        Ok(Arc::new(VectorKernel(vector_parameters(p)?)))
    }
}
