//! Shared Rust author for the pinned stepped-guide boundary cases.
use super::fixtures::scale;
pub fn boundary_scale(
    case: &serde_json::Value,
) -> chart_core::ChartResult<chart_core::scales::MappedScaleSpec> {
    use chart_core::{interpolate::Number, scales::*};
    let limits = if case["population"] == "constant" {
        [2., 2.]
    } else {
        [-2., 8.]
    };
    let empty_population = case["population"] == "empty";
    let mut descriptor = scale("asymmetric", true);
    if let ScaleFunctionSpec::Interpolated(scale) = &mut descriptor.function {
        if let NormalizationSpec::Ggplot { domain, .. }
        | NormalizationSpec::Sequential { domain, .. } = &mut scale.normalization
        {
            *domain = limits.map(Number);
        } else {
            panic!("reference normalization");
        }
    }

    let breaks = case["breaks"].as_array().map(|values| {
        values
            .iter()
            .map(|v| {
                Number(match v.as_str() {
                    Some("Infinity") => f64::INFINITY,
                    Some("-Infinity") => f64::NEG_INFINITY,
                    _ => v.as_f64().unwrap_or(f64::NAN),
                })
            })
            .collect::<Vec<_>>()
    });
    if case["family"] == "binned" {
        descriptor =
            descriptor.with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                breaks: breaks.map_or(GgplotBreaks::Nice(5.), GgplotBreaks::Explicit),
                limits: Some(limits.map(|v| Some(Number(v)))),
                empty_population,
                ..Default::default()
            })))?;
        descriptor.with_guide(GgplotScaleGuide::BinnedSteps(GgplotGuideLabels::Automatic))
    } else {
        descriptor = descriptor.with_ggplot(GgplotScalePolicy::Continuous {
            limits: Some(limits.map(|v| Some(Number(v)))),
            empty_population,
            nonfinite_population: false,
            oob: GgplotOob::Censor,
        })?;
        descriptor.with_guide(if case["mode"] == "null" {
            GgplotScaleGuide::Hidden
        } else {
            GgplotScaleGuide::ContinuousSteps(GgplotContinuousGuide {
                breaks,
                ..Default::default()
            })
        })
    }
}
