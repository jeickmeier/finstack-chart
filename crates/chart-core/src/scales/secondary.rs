//! Strict one-to-one secondary-unit adapters over the shared numeric mapping.
use super::{NumericFamily, NumericScale, NumericScaleSpec, error};
use crate::{ChartResult, DiagnosticCode, interpolate::Number};

pub(crate) fn secondary_mapping(spec: &NumericScaleSpec) -> ChartResult<NumericScale> {
    let strict = |knots: &[Number]| {
        knots.len() >= 2
            && knots.iter().all(|v| v.0.is_finite())
            && (knots.windows(2).all(|v| v[0].0 < v[1].0)
                || knots.windows(2).all(|v| v[0].0 > v[1].0))
    };
    let monotone = match &spec.family {
        NumericFamily::Linear | NumericFamily::Radial => true,
        NumericFamily::Ggplot { transform } => {
            transform.monotone_on(&spec.domain.iter().map(|v| v.0).collect::<Vec<_>>())
        }
        NumericFamily::Identity => spec.domain == spec.range,
        NumericFamily::Pow { exponent } => exponent.is_finite() && *exponent > 0.,
        NumericFamily::Log { base } => base.is_finite() && *base > 0. && *base != 1.,
        NumericFamily::Symlog { constant } => constant.is_finite() && *constant > 0.,
    };
    if spec.clamp
        || spec.round
        || !monotone
        || spec.domain.len() != spec.range.len()
        || !strict(&spec.domain)
        || !strict(&spec.range)
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Secondary transforms require unrounded, unclamped, strictly monotone matching knots and an invertible family.",
        ));
    }
    NumericScale::new(spec.clone())
}
