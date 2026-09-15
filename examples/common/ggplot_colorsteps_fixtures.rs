//! Shared Rust authoring for stepped guide reference fixtures.
use super::fixtures::scale;
use chart_core::{interpolate::Number, scales::*};
/// Nonuniform source cuts for continuous and binned stepped guides.
pub fn steps_scale(family: &str, endpoints: bool, hidden: bool) -> MappedScaleSpec {
    let breaks = if endpoints {
        vec![-2., 0., 3., 8.]
    } else {
        vec![0., 3.]
    };
    let descriptor = scale("asymmetric", true);
    if family == "binned" {
        descriptor
            .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                breaks: GgplotBreaks::Explicit(breaks.into_iter().map(Number).collect()),
                limits: Some([Some(Number(-2.)), Some(Number(8.))]),
                ..Default::default()
            })))
            .unwrap()
            .with_guide(if hidden {
                GgplotScaleGuide::Hidden
            } else {
                GgplotScaleGuide::BinnedSteps(GgplotGuideLabels::Automatic)
            })
            .unwrap()
    } else {
        descriptor
            .with_guide(if hidden {
                GgplotScaleGuide::Hidden
            } else {
                GgplotScaleGuide::ContinuousSteps(GgplotContinuousGuide {
                    breaks: Some(breaks.into_iter().map(Number).collect()),
                    ..Default::default()
                })
            })
            .unwrap()
    }
}
