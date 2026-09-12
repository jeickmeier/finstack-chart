//! Continuous aesthetic guide policy over the common transform, break and label owners.
use super::*;
use crate::{ChartResult, DiagnosticCode, interpolate::Number};

/// Scale-level guide arguments; guide layout and composition consume these results.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GgplotScaleGuide {
    /// Suppress the aesthetic guide while retaining scale training and mapping.
    Hidden,
    /// Timestamp candidates retain integer origins and calendar labels.
    Temporal(GgplotTemporalGuide),
    /// Discrete break intersection and label replacement.
    Discrete(GgplotDiscreteGuide),
    /// Continuous candidates and vector-wide labels, before guide censoring.
    Continuous(GgplotContinuousGuide),
    /// Labels for the cuts selected by the existing binned population policy.
    Binned(GgplotGuideLabels),
}
/// Continuous aesthetic break and label arguments.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GgplotContinuousGuide {
    /// None selects automatic breaks. Empty selects none unless the domain is constant.
    pub breaks: Option<Vec<Number>>,
    /// Optional desired count for automatic selection; explicit breaks bypass it.
    pub count: Option<f64>,
    /// Labels are computed before out-of-domain candidates are removed. Named labels
    /// are positional on continuous scales, matching the reference.
    pub labels: GgplotGuideLabels,
}
/// One continuous guide candidate, retained even when outside the scale limits.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GgplotContinuousGuideEntry {
    /// Source-space value after the transform/inverse round-trip used for labels.
    pub value: Number,
    /// Transformed scale-space value, prior to rescaling into the aesthetic range.
    pub transformed: Number,
    /// Optional label; absent for hidden labels or missing numeric candidates.
    pub label: Option<String>,
    /// Whether the finite transformed value is within the scale's guide limits.
    pub visible: bool,
}
impl GgplotScaleGuide {
    pub(super) fn validate(&self) -> ChartResult<()> {
        match self {
            Self::Hidden => Ok(()),
            Self::Temporal(g) => g.validate(),
            Self::Discrete(g) => g.validate(),
            Self::Continuous(g) => g.validate(),
            Self::Binned(labels) => labels.validate(None),
        }
    }
}
fn transform(family: NumericFamily) -> Option<ScaleTransform> {
    match family {
        NumericFamily::Log { base } => Some(ScaleTransform::Log { base }),
        NumericFamily::Pow { exponent: 0.5 } => Some(ScaleTransform::Sqrt),
        NumericFamily::Symlog { constant } => Some(ScaleTransform::Symlog {
            threshold: constant,
        }),
        _ => None,
    }
}
pub(super) fn forward(family: NumericFamily, reverse: bool, x: f64) -> f64 {
    let y = transform(family).map_or_else(
        || super::numeric::transform(family, false, x, false),
        |t| t.forward_raw(x),
    );
    if reverse { -y } else { y }
}
pub(super) fn inverse(family: NumericFamily, reverse: bool, x: f64) -> f64 {
    let x = if reverse { -x } else { x };
    transform(family).map_or_else(
        || super::numeric::transform(family, false, x, true),
        |t| t.inverse_raw(x),
    )
}
/// Authored endpoints whose transform is NaN retain missing-limit semantics.
/// Preserve Some([None, None]): authored limits still suppress automatic extension.
pub(super) fn comparable_limits(
    limits: Option<[Option<Number>; 2]>,
    family: NumericFamily,
) -> Option<[Option<Number>; 2]> {
    limits.map(|limits| {
        limits.map(|value| value.filter(|value| !forward(family, false, value.0).is_nan()))
    })
}
/// Apply transformed authored endpoints without reordering partial limits.
pub(super) fn authored_bounds(
    limits: Option<[Option<Number>; 2]>,
    family: NumericFamily,
    reverse: bool,
    mut fallback: [f64; 2],
) -> [f64; 2] {
    if let Some(limits) = comparable_limits(limits, family) {
        let limits = limits.map(|v| v.map(|v| forward(family, reverse, v.0)));
        if let [Some(a), Some(b)] = limits {
            return [a.min(b), a.max(b)];
        }
        for (i, value) in limits.into_iter().enumerate() {
            if let Some(value) = value {
                fallback[i] = value;
            }
        }
    }
    fallback
}
pub(super) fn finite_limits(
    limits: Option<[Option<Number>; 2]>,
    family: NumericFamily,
    reverse: bool,
) -> bool {
    authored_bounds(limits, family, reverse, [f64::NAN; 2])
        .iter()
        .all(|v| v.is_finite())
}
impl GgplotContinuousGuide {
    pub(super) fn validate(&self) -> ChartResult<()> {
        self.labels.validate(self.breaks.as_ref().map(Vec::len))?;
        if self.count.is_some_and(|n| !n.is_finite()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Guide count must be finite.",
            ));
        }
        Ok(())
    }
    /// Select candidates over raw source limits, ordering the complete range as
    /// a reference scale constructor does. Prepared partial limits retain order.
    pub fn resolve(
        &self,
        domain: [Number; 2],
        family: NumericFamily,
        reverse: bool,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        let mut bounds = domain.map(|v| forward(family, reverse, v.0));
        bounds.sort_by(f64::total_cmp);
        self.resolve_bounds(bounds, family, reverse, budget, label_budget)
    }
    pub(super) fn resolve_prepared(
        &self,
        domain: [Number; 2],
        family: NumericFamily,
        reverse: bool,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        self.resolve_bounds(
            domain.map(|v| forward(family, reverse, v.0)),
            family,
            reverse,
            budget,
            label_budget,
        )
    }
    pub(super) fn resolve_nonfinite(
        &self,
        limits: Option<[Option<Number>; 2]>,
        family: NumericFamily,
        reverse: bool,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        self.resolve_bounds(
            authored_bounds(limits, family, reverse, [f64::INFINITY, f64::NEG_INFINITY]),
            family,
            reverse,
            budget,
            label_budget,
        )
    }
    fn resolve_bounds(
        &self,
        bounds: [f64; 2],
        family: NumericFamily,
        reverse: bool,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        self.validate()?;
        // Validate the shared family without generating an independent scale mapping.
        if let Some(t) = transform(family) {
            t.validate()?;
        }
        if matches!(family,NumericFamily::Pow {exponent} if !exponent.is_finite() || exponent==0.) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Continuous guide requires an invertible finite power transform.",
            ));
        }
        if bounds.iter().any(|v| v.is_nan()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Guide limits are outside the transformation domain.",
            ));
        }
        let values = if super::ggplot::zero_range(bounds[0], bounds[1]) {
            if budget == 0 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Continuous guide exceeds the tick budget.",
                ));
            }
            vec![bounds[0]]
        } else if let Some(breaks) = &self.breaks {
            if breaks.len() > budget {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Continuous guide exceeds the tick budget.",
                ));
            }
            breaks
                .iter()
                .map(|v| forward(family, reverse, v.0))
                .collect()
        } else {
            let limits = bounds.map(|v| inverse(family, reverse, v));
            let count = self.count.unwrap_or(5.);
            let breaks = if let NumericFamily::Log { base } = family {
                ggplot_breaks_log(limits, count, base, budget)?
            } else {
                ggplot_breaks_extended(limits, count, budget)?
            };
            breaks
                .into_iter()
                .map(|v| forward(family, reverse, v))
                .collect()
        };
        if values.len() > budget {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Continuous guide exceeds the tick budget.",
            ));
        }
        numeric_guide_entries(values, bounds, family, reverse, &self.labels, label_budget)
    }
}
fn visible(value: f64, bounds: [f64; 2]) -> bool {
    value.is_finite() && value >= bounds[0] && value <= bounds[1]
}
pub(super) fn numeric_guide_entries(
    values: Vec<f64>,
    bounds: [f64; 2],
    family: NumericFamily,
    reverse: bool,
    labels_policy: &GgplotGuideLabels,
    label_budget: usize,
) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
    let source: Vec<_> = values
        .iter()
        .map(|v| inverse(family, reverse, *v))
        .collect();
    let authored_len = match labels_policy {
        GgplotGuideLabels::Explicit(labels) => Some(labels.len()),
        GgplotGuideLabels::Named(labels) => Some(labels.len()),
        _ => None,
    };
    if authored_len.is_some_and(|n| n != values.len()) {
        return Err(error(
            DiagnosticCode::Validation,
            "Breaks and labels have different lengths.",
        ));
    }
    let labels = match labels_policy {
        GgplotGuideLabels::Automatic => {
            let finite: Vec<_> = source.iter().copied().filter(|v| v.is_finite()).collect();
            let mut labels =
                crate::typography::ggplot_numeric_labels(&finite, label_budget)?.into_iter();
            source
                .iter()
                .map(|v| {
                    if v.is_nan() {
                        None
                    } else if v.is_infinite() {
                        Some(if *v > 0. { "Inf" } else { "-Inf" }.into())
                    } else {
                        labels.next()
                    }
                })
                .collect::<Vec<_>>()
        }
        GgplotGuideLabels::Hidden => vec![None; source.len()],
        GgplotGuideLabels::Explicit(labels) => labels.clone(),
        GgplotGuideLabels::Named(labels) => labels.iter().map(|(_, v)| v.clone()).collect(),
    };
    if labels.len() != values.len() {
        return Err(error(
            DiagnosticCode::Validation,
            "Breaks and labels have different lengths.",
        ));
    }
    let mut remaining = label_budget;
    for label in labels.iter().flatten() {
        if label.len() > remaining {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Continuous guide exceeds the label byte budget.",
            ));
        }
        remaining -= label.len();
    }
    Ok(values
        .into_iter()
        .zip(source)
        .zip(labels)
        .map(|((value, source), label)| GgplotContinuousGuideEntry {
            value: Number(source),
            transformed: Number(value),
            label,
            visible: visible(value, bounds),
        })
        .collect())
}
