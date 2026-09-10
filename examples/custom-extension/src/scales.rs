//! AXIS-01 provider implemented outside core and installed explicitly in proof hosts.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    composition::ScaleValue,
    grammar::{
        CustomScale, ExtensionDescriptor, ExtensionRegistry, ScaleProviderInput, ValueSpace,
    },
    scales::{Bounds, ContinuousDomain, LinearScale, PositionalScale},
};
use serde_json::Value;
use std::sync::Arc;

/// Install a noninjective numeric mapping and its native-only counterpart.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_scale(Arc::new(Fold { portable: true }))?;
    registry.register_scale(Arc::new(Fold { portable: false }))
}
struct Fold {
    portable: bool,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    limit: f64,
}
fn limit(parameters: &Value) -> ChartResult<f64> {
    let p: Parameters = serde_json::from_value(parameters.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))?;
    if !p.limit.is_finite() || p.limit <= 0. {
        return Err(super::error(
            DiagnosticCode::NumericalDomain,
            "Fold limit must be finite and positive.",
        ));
    }
    Ok(p.limit)
}
impl CustomScale for Fold {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                "example.fold"
            } else {
                "example.native_fold"
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, parameters: &Value) -> ChartResult<()> {
        limit(parameters).map(|_| ())
    }
    fn resolve(&self, input: ScaleProviderInput<'_>) -> ChartResult<Arc<dyn PositionalScale>> {
        let limit = limit(input.parameters)?;
        if !matches!(
            input.space,
            ValueSpace::Data | ValueSpace::Transformed { .. } | ValueSpace::Scaled { .. }
        ) || input.window.is_some()
        {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "The fold example requires numeric values and has no navigation window.",
            ));
        }
        if input.max_values < 3 {
            return Err(super::error(
                DiagnosticCode::ResourceLimit,
                "The fold example needs three domain values.",
            ));
        }
        Ok(Arc::new(Resolved {
            domain: vec![
                ScaleValue::Number(-limit),
                ScaleValue::Number(0.),
                ScaleValue::Number(limit),
            ],
            range: input.range,
            magnitude: LinearScale::resolve(
                None,
                ContinuousDomain::explicit(Bounds::new(0., limit)?),
                input.range,
                None,
                input.outside,
            )?,
        }))
    }
}
struct Resolved {
    domain: Vec<ScaleValue>,
    range: Bounds,
    magnitude: LinearScale,
}
impl PositionalScale for Resolved {
    fn domain(&self) -> &[ScaleValue] {
        &self.domain
    }
    fn range(&self) -> Bounds {
        self.range
    }
    fn map(&self, value: &ScaleValue) -> ChartResult<Option<f64>> {
        let ScaleValue::Number(value) = value else {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "Fold requires a semantic number.",
            ));
        };
        self.magnitude.map(value.abs())
    }
}
