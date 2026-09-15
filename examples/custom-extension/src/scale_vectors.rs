//! Pure OOB/rescaler examples shared by the actual proof hosts.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    grammar::{
        CustomScaleVector, ExtensionDescriptor, ExtensionRegistry, ScaleVectorInput,
        ScaleVectorStage,
    },
    interpolate::Number,
};
use std::sync::Arc;
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Default,
    OobReverse,
    OobIndex,
    OobShort,
    OobEmpty,
    OobNull,
    BothIndex,
    RescaleReverse,
    RescaleIndex,
    RescaleShort,
    RescaleEmpty,
    RescaleNull,
}
#[derive(Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Family {
    #[default]
    Continuous,
    Binned,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    mode: Mode,
    #[serde(default)]
    family: Family,
}
fn parameters(value: &serde_json::Value) -> ChartResult<Parameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
/// Explicitly installed numeric callback; it never captures a host runtime.
pub struct Vector {
    /// Whether this identity is portable.
    pub portable: bool,
}
impl CustomScaleVector for Vector {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                "example.scale_vector"
            } else {
                "example.native_scale_vector"
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, value: &serde_json::Value) -> ChartResult<()> {
        parameters(value).map(|_| ())
    }
    fn evaluate(&self, input: ScaleVectorInput<'_>) -> ChartResult<Option<Vec<Number>>> {
        let p = parameters(input.parameters)?;
        let a = input.limits.first().map_or(f64::NAN, |v| v.0);
        let b = input.limits.get(1).map_or(f64::NAN, |v| v.0);
        if input.stage == ScaleVectorStage::OutOfBounds {
            if matches!(p.mode, Mode::OobNull) {
                return Ok(None);
            }
            return Ok(Some(match p.mode {
                Mode::OobShort => input.values.iter().take(1).copied().collect(),
                Mode::OobEmpty => vec![],
                Mode::OobReverse => input.values.iter().rev().copied().collect(),
                Mode::OobIndex | Mode::BothIndex => {
                    (1..=input.values.len()).map(|i| Number(i as f64)).collect()
                }
                _ => input
                    .values
                    .iter()
                    .map(|v| {
                        Number(if v.0.is_finite() && matches!(p.family, Family::Binned) {
                            if v.0 < a {
                                a
                            } else if v.0 > b {
                                b
                            } else {
                                v.0
                            }
                        } else if v.0.is_finite() && (v.0 < a || v.0 > b) {
                            f64::NAN
                        } else {
                            v.0
                        })
                    })
                    .collect(),
            }));
        }
        if matches!(p.mode, Mode::RescaleNull) {
            return Ok(None);
        }
        if matches!(p.mode, Mode::RescaleEmpty) {
            return Ok(Some(vec![]));
        }
        let mut values = if matches!(p.mode, Mode::RescaleIndex | Mode::BothIndex) {
            (1..=input.values.len())
                .map(|i| Number(i as f64 / input.values.len() as f64))
                .collect::<Vec<_>>()
        } else {
            if !(1..=2).contains(&input.limits.len())
                || input.limits.len() == 2 && (a.is_nan() || b.is_nan())
            {
                return Err(super::error(
                    DiagnosticCode::NumericalDomain,
                    "Example rescaler requires one or two comparable limits.",
                ));
            }
            let constant = input.limits.len() == 1
                || a == b
                || (a != 0.
                    && b != 0.
                    && (a - b).abs() / a.abs().min(b.abs()) < 1000. * f64::EPSILON);
            input
                .values
                .iter()
                .map(|v| {
                    Number(if v.0.is_nan() {
                        f64::NAN
                    } else if constant {
                        0.5
                    } else {
                        (v.0 - a) / (b - a)
                    })
                })
                .collect::<Vec<_>>()
        };
        match p.mode {
            Mode::RescaleReverse => values.reverse(),
            Mode::RescaleShort => values.truncate(1),
            _ => {}
        }
        Ok(Some(values))
    }
}
pub(crate) fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_scale_vector(Arc::new(Vector { portable: true }))?;
    registry.register_scale_vector(Arc::new(Vector { portable: false }))
}
