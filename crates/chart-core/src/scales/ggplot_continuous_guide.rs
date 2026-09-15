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
    /// Timestamp colorbar selection preserves calendar candidates and shared ramp sampling.
    TemporalColorbar(GgplotTemporalGuide),
    /// Interval keys retain calendar labels without binning timestamp marks.
    TemporalBins(GgplotTemporalGuide),
    /// Stepped color keys retain calendar labels and timestamp normalization.
    TemporalSteps(GgplotTemporalGuide),
    /// Discrete break intersection and label replacement.
    Discrete(GgplotDiscreteGuide),
    /// Continuous candidates and vector-wide labels, before guide censoring.
    Continuous(GgplotContinuousGuide),
    /// Explicit continuous colorbar selection; non-color aesthetics suppress it.
    /// The scale retains the default 300-sample ramp for guide composition.
    Colorbar(GgplotContinuousGuide),
    /// Interval keys selected from continuous breaks, without binning marks.
    ContinuousBins(GgplotContinuousGuide),
    /// Stepped color keys selected from continuous breaks.
    ContinuousSteps(GgplotContinuousGuide),
    /// Labels for the cuts selected by the existing binned population policy.
    Binned(GgplotGuideLabels),
    /// Explicit reference legend keys on a binned scale, before guide composition.
    BinnedLegend(GgplotGuideLabels),
    /// Explicit interval keys for any binned aesthetic.
    BinnedBins(GgplotGuideLabels),
    /// Explicit stepped color keys; ignored by non-color aesthetics.
    BinnedSteps(GgplotGuideLabels),
}
/// Rendering policy for a continuous colorbar, independent of mark mapping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GgplotColorbarDisplay {
    /// Interpolated cell-centered samples; the reference default.
    #[default]
    Raster,
    /// Adjacent constant-color cells.
    Rectangles,
    /// Interpolated endpoint-aligned samples; defaults to 15 samples.
    Gradient,
}
impl GgplotColorbarDisplay {
    fn is_raster(&self) -> bool {
        *self == Self::Raster
    }
}
/// Authored controls for a continuous colorbar.
/// Sampling is demanded only when the selected guide has keys.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GgplotColorbarOptions {
    /// Reference `nbin`; omitted selects 300 for raster/rectangles or 15 for gradient. Fractional counts round up for
    /// sampling but retain their authored value for raster key placement.
    pub nbin: Option<Number>,
    /// Display geometry; omitted retains the raster default.
    #[serde(skip_serializing_if = "GgplotColorbarDisplay::is_raster")]
    pub display: GgplotColorbarDisplay,
    /// Replace colorbar paint alpha; omitted retains palette alpha and mark colors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<Number>,
    /// Guide orientation; omitted selects a vertical bar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<crate::scene::GradientDirection>,
    /// Reverse the bar and keys without changing the scale's mark mapping.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub reverse: bool,
    /// Draw the first selected key's tick, independently of its label.
    #[serde(skip_serializing_if = "is_true")]
    pub draw_lower_limit: bool,
    /// Draw the last selected key's tick, independently of its label.
    #[serde(skip_serializing_if = "is_true")]
    pub draw_upper_limit: bool,
}
fn is_true(value: &bool) -> bool {
    *value
}
impl Default for GgplotColorbarOptions {
    fn default() -> Self {
        Self {
            nbin: None,
            display: GgplotColorbarDisplay::Raster,
            alpha: None,
            direction: None,
            reverse: false,
            draw_lower_limit: true,
            draw_upper_limit: true,
        }
    }
}
impl GgplotColorbarOptions {
    pub(crate) fn validate(&self) -> ChartResult<()> {
        if self
            .alpha
            .is_some_and(|a| a.0.is_nan() || (a.0.is_finite() && !(0. ..=1.).contains(&a.0)))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Colorbar alpha must be between zero and one or infinite.",
            ));
        }
        Ok(())
    }
    pub(crate) fn has_presentation(&self) -> bool {
        self.direction.is_some() || self.reverse || !self.draw_lower_limit || !self.draw_upper_limit
    }
    pub(crate) fn nbin(&self) -> f64 {
        self.nbin.map_or(
            if self.display == GgplotColorbarDisplay::Gradient {
                15.
            } else {
                300.
            },
            |n| n.0,
        )
    }
    pub(crate) fn sample_count(&self) -> ChartResult<usize> {
        let n = self.nbin();
        if !n.is_finite() || n < 0. {
            return Err(error(
                DiagnosticCode::Validation,
                "Colorbar sample count must be finite and nonnegative.",
            ));
        }
        crate::limits::require_within(
            n.ceil() <= crate::interpolate::MAX_VALUES as f64,
            "colorbar sample",
        )?;
        Ok(if n == 0. { 2 } else { n.ceil() as usize })
    }
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
    /// Retained output from a registered continuous palette's complete guide batch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mapped: Option<crate::interpolate::Value>,
    /// Name retained from a registered break vector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
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
    pub(crate) fn temporal(&self) -> Option<&GgplotTemporalGuide> {
        match self {
            Self::Temporal(g)
            | Self::TemporalColorbar(g)
            | Self::TemporalBins(g)
            | Self::TemporalSteps(g) => Some(g),
            _ => None,
        }
    }
    pub(crate) fn labels(&self) -> Option<&GgplotGuideLabels> {
        match self {
            Self::Hidden => None,
            Self::Temporal(g)
            | Self::TemporalColorbar(g)
            | Self::TemporalBins(g)
            | Self::TemporalSteps(g) => Some(&g.arguments.labels),
            Self::Discrete(g) => Some(&g.labels),
            Self::Continuous(g)
            | Self::Colorbar(g)
            | Self::ContinuousBins(g)
            | Self::ContinuousSteps(g) => Some(&g.labels),
            Self::Binned(labels)
            | Self::BinnedLegend(labels)
            | Self::BinnedBins(labels)
            | Self::BinnedSteps(labels) => Some(labels),
        }
    }
    pub(super) fn validate(&self) -> ChartResult<()> {
        match self {
            Self::Hidden => Ok(()),
            Self::Temporal(g)
            | Self::TemporalColorbar(g)
            | Self::TemporalBins(g)
            | Self::TemporalSteps(g) => g.validate(),
            Self::Discrete(g) => g.validate(),
            Self::Continuous(g)
            | Self::Colorbar(g)
            | Self::ContinuousBins(g)
            | Self::ContinuousSteps(g) => g.validate(),
            Self::Binned(labels)
            | Self::BinnedLegend(labels)
            | Self::BinnedBins(labels)
            | Self::BinnedSteps(labels) => labels.validate(None),
        }
    }
}
fn transform(family: &NumericFamily) -> Option<ScaleTransform> {
    match *family {
        NumericFamily::Ggplot { ref transform } => Some(ScaleTransform::Ggplot {
            transform: transform.clone(),
        }),
        NumericFamily::Log { base } => Some(ScaleTransform::Log { base }),
        NumericFamily::Pow { exponent: 0.5 } => Some(ScaleTransform::Sqrt),
        NumericFamily::Symlog { constant } => Some(ScaleTransform::Symlog {
            threshold: constant,
        }),
        _ => None,
    }
}
pub(super) fn forward(family: &NumericFamily, reverse: bool, x: f64) -> f64 {
    if let NumericFamily::Ggplot { transform } = family {
        let y = transform.forward(x);
        return if reverse { -y } else { y };
    }
    let y = transform(family).map_or_else(
        || super::numeric::transform(family, false, x, false),
        |t| t.forward_raw(x),
    );
    if reverse { -y } else { y }
}
pub(super) fn inverse(family: &NumericFamily, reverse: bool, x: f64) -> f64 {
    let x = if reverse { -x } else { x };
    if let NumericFamily::Ggplot { transform } = family {
        return transform.inverse(x);
    }
    transform(family).map_or_else(
        || super::numeric::transform(family, false, x, true),
        |t| t.inverse_raw(x),
    )
}
/// Preserve the authored batch boundary for registered arithmetic.
pub(crate) fn forward_values(
    family: &NumericFamily,
    reverse: bool,
    values: &[f64],
) -> ChartResult<Vec<f64>> {
    if let NumericFamily::Ggplot { transform } = family {
        let mut result = transform.forward_population(values)?;
        if reverse {
            for v in &mut result {
                *v = -*v;
            }
        }
        return Ok(result);
    }
    Ok(values
        .iter()
        .map(|v| forward(family, reverse, *v))
        .collect())
}
/// Preserve NULL versus typed empty output when dispatching callback results.
pub(crate) fn forward_null(family: &NumericFamily, reverse: bool) -> ChartResult<Option<Vec<f64>>> {
    if reverse || family.ggplot_transform().is_none() && *family != NumericFamily::Linear {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "This numeric transformation cannot transform a NULL callback result.",
        ));
    }
    family
        .ggplot_transform()
        .map_or(Ok(None), GgplotTransform::forward_null)
}
/// Preserve each source layer's vector boundary while sharing transform arithmetic.
pub(super) fn forward_batches(
    family: &NumericFamily,
    reverse: bool,
    values: &[f64],
    batches: &[usize],
) -> ChartResult<Vec<f64>> {
    let mut output = Vec::with_capacity(values.len());
    let mut offset = 0;
    for count in batches {
        let end = offset + count;
        output.extend(forward_values(family, reverse, &values[offset..end])?);
        offset = end;
    }
    debug_assert_eq!(offset, values.len());
    Ok(output)
}
pub(crate) fn inverse_values(
    family: &NumericFamily,
    reverse: bool,
    values: &[f64],
) -> ChartResult<Vec<f64>> {
    if let NumericFamily::Ggplot { transform } = family {
        let values = values
            .iter()
            .map(|v| if reverse { -*v } else { *v })
            .collect::<Vec<_>>();
        return transform.inverse_population(&values);
    }
    Ok(values
        .iter()
        .map(|v| inverse(family, reverse, *v))
        .collect())
}
/// Authored endpoints whose transform is NaN retain missing-limit semantics.
/// Preserve Some([None, None]): authored limits still suppress automatic extension.
pub(super) fn comparable_limits(
    limits: Option<[Option<Number>; 2]>,
    family: NumericFamily,
) -> Option<[Option<Number>; 2]> {
    limits.map(|limits| {
        limits.map(|value| value.filter(|value| !forward(&family, false, value.0).is_nan()))
    })
}
/// Apply transformed authored endpoints without reordering partial limits.
pub(super) fn authored_bounds(
    limits: Option<[Option<Number>; 2]>,
    family: NumericFamily,
    reverse: bool,
    mut fallback: [f64; 2],
) -> [f64; 2] {
    if let Some(limits) = comparable_limits(limits, family.clone()) {
        let limits = limits.map(|v| v.map(|v| forward(&family, reverse, v.0)));
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
/// Apply an authored endpoint vector as one transform invocation, retaining missing slots.
pub(super) fn authored_bounds_batch(
    limits: Option<[Option<Number>; 2]>,
    family: &NumericFamily,
    reverse: bool,
    mut fallback: [f64; 2],
) -> ChartResult<[f64; 2]> {
    if let Some(limits) = limits {
        let raw = limits.map(|v| v.map_or(f64::NAN, |v| v.0));
        let values = forward_values(family, reverse, &raw)?;
        if values.iter().all(|v| !v.is_nan()) {
            return Ok([values[0].min(values[1]), values[0].max(values[1])]);
        }
        for (i, value) in values.into_iter().enumerate() {
            if !value.is_nan() {
                fallback[i] = value;
            }
        }
    }
    Ok(fallback)
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
    /// Resolve source-space labels through an explicitly supplied registry.
    pub fn resolve_with_registry(
        &self,
        domain: [Number; 2],
        family: NumericFamily,
        reverse: bool,
        limits: crate::Limits,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        let budget = limits.max_items;
        let label_budget = limits.max_text_bytes;
        if !matches!(self.labels, GgplotGuideLabels::Registered { .. }) {
            return self.resolve(domain, family, reverse, budget, label_budget);
        }
        let mut selection = self.clone();
        selection.labels = GgplotGuideLabels::Hidden;
        let mut entries = selection.resolve(domain, family, reverse, budget, label_budget)?;
        let values = entries
            .iter()
            .map(|entry| crate::composition::ScaleValue::Number(entry.value.0))
            .collect::<Vec<_>>();
        let labels =
            self.labels
                .registered_values(&values, None, None, &registry.guides, label_budget)?;
        if labels.len() != entries.len() {
            return Err(error(
                DiagnosticCode::Validation,
                "Breaks and labels have different lengths.",
            ));
        }
        for (entry, label) in entries.iter_mut().zip(labels) {
            entry.label = label;
        }
        Ok(entries)
    }
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
        let mut bounds: [f64; 2] = forward_values(&family, reverse, &domain.map(|v| v.0))?
            .try_into()
            .expect("length-preserving transform");
        bounds.sort_by(f64::total_cmp);
        self.resolve_bounds(bounds, family, reverse, budget, label_budget)
    }
    pub(crate) fn resolve_bounds(
        &self,
        bounds: [f64; 2],
        family: NumericFamily,
        reverse: bool,
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        self.resolve_bounds_with_label_censor(bounds, family, reverse, budget, label_budget, true)
    }
    pub(crate) fn resolve_bounds_with_label_censor(
        &self,
        bounds: [f64; 2],
        family: NumericFamily,
        reverse: bool,
        budget: usize,
        label_budget: usize,
        censor_labels: bool,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        self.validate()?;
        // Validate the shared family without generating an independent scale mapping.
        if let Some(t) = transform(&family) {
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
        let mut selection_bounds = bounds;
        if let NumericFamily::Ggplot { ref transform } = family {
            let mut transformed_domain: [f64; 2] =
                forward_values(&family, reverse, &transform.domain())?
                    .try_into()
                    .expect("length-preserving transform");
            transformed_domain.sort_by(f64::total_cmp);
            if transformed_domain.iter().all(|v| !v.is_nan())
                && !super::ggplot::zero_range(transformed_domain[0], transformed_domain[1])
            {
                for value in &mut selection_bounds {
                    if value.is_finite() {
                        *value = value.max(transformed_domain[0]).min(transformed_domain[1]);
                    }
                }
            }
        }
        let values = if super::ggplot::zero_range(selection_bounds[0], selection_bounds[1]) {
            if budget == 0 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Continuous guide exceeds the tick budget.",
                ));
            }
            vec![selection_bounds[0]]
        } else if let Some(breaks) = &self.breaks {
            if breaks.len() > budget {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Continuous guide exceeds the tick budget.",
                ));
            }
            forward_values(
                &family,
                reverse,
                &breaks.iter().map(|v| v.0).collect::<Vec<_>>(),
            )?
        } else {
            let mut limits: [f64; 2] = inverse_values(&family, reverse, &selection_bounds)?
                .try_into()
                .expect("length-preserving transform");
            if matches!(family, NumericFamily::Ggplot { .. }) {
                let finite: Vec<_> = limits.iter().copied().filter(|v| v.is_finite()).collect();
                if finite.is_empty() {
                    return Ok(vec![]);
                }
                limits = [
                    finite.iter().copied().fold(f64::INFINITY, f64::min),
                    finite.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                ];
            }
            let count = self.count.unwrap_or(5.);
            let breaks = numeric_breaks(&family, limits, count, budget)?;
            forward_values(&family, reverse, &breaks)?
        };
        if values.len() > budget {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Continuous guide exceeds the tick budget.",
            ));
        }
        numeric_guide_entries_with_label_censor(
            values,
            bounds,
            family,
            reverse,
            &self.labels,
            label_budget,
            censor_labels,
        )
    }
}
/// Default transform-owned break policy, including composition's one-argument callback.
pub(crate) fn numeric_breaks(
    family: &NumericFamily,
    limits: [f64; 2],
    count: f64,
    budget: usize,
) -> ChartResult<Vec<f64>> {
    match family {
        NumericFamily::Log { base } => ggplot_breaks_log(limits, count, *base, budget),
        NumericFamily::Ggplot { transform } => {
            if let Some(breaks) = transform.default_breaks(limits, count, budget)? {
                return Ok(breaks);
            }
            let count = if matches!(transform, GgplotTransform::Compose { .. }) {
                5.
            } else {
                count
            };
            if let Some(base) = transform.break_log_base() {
                super::log_breaks::builtin_breaks_log(limits, count, base, budget, true)
            } else {
                ggplot_breaks_extended(limits, count, budget)
            }
        }
        _ => ggplot_breaks_extended(limits, count, budget),
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
    numeric_guide_entries_with_label_censor(
        values,
        bounds,
        family,
        reverse,
        labels_policy,
        label_budget,
        true,
    )
}
/// Identity guides format the complete candidate vector before guide filtering.
pub(super) fn numeric_guide_entries_with_label_censor(
    values: Vec<f64>,
    bounds: [f64; 2],
    family: NumericFamily,
    reverse: bool,
    labels_policy: &GgplotGuideLabels,
    label_budget: usize,
    censor_labels: bool,
) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
    let label_values: Vec<_> = values
        .iter()
        .map(|v| {
            if censor_labels
                && matches!(family, NumericFamily::Ggplot { .. })
                && !visible(*v, bounds)
            {
                f64::NAN
            } else {
                *v
            }
        })
        .collect();
    let source = inverse_values(&family, reverse, &label_values)?;
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
            transform_numeric_labels(family.ggplot_transform(), &source, label_budget)?
        }
        GgplotGuideLabels::Hidden => vec![None; source.len()],
        GgplotGuideLabels::Explicit(labels) => labels.clone(),
        GgplotGuideLabels::Named(labels) => labels.iter().map(|(_, v)| v.clone()).collect(),
        GgplotGuideLabels::Registered { .. } => {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Registered scale labels require an explicit guide registry.",
            ));
        }
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
            mapped: None,
            name: None,
            value: Number(source),
            transformed: Number(value),
            label,
            visible: visible(value, bounds),
        })
        .collect())
}

/// Preserve named break vectors through both raw candidates and default formatting.
pub(super) fn apply_break_names(
    entries: &mut [GgplotContinuousGuideEntry],
    names: &[String],
    automatic: bool,
    label_budget: usize,
) -> ChartResult<()> {
    crate::limits::require_within(
        entries.len() == names.len(),
        "named guide candidate alignment",
    )?;
    if automatic {
        crate::limits::require_within(
            names
                .iter()
                .fold(0usize, |n, name| n.saturating_add(name.len()))
                <= label_budget,
            "named guide label byte",
        )?;
    }
    for (entry, name) in entries.iter_mut().zip(names) {
        entry.name = Some(name.clone());
        if automatic {
            entry.label = Some(name.clone());
        }
    }
    Ok(())
}

/// Default reference numeric labels retain missing and infinite candidates.
pub(crate) fn default_numeric_labels(
    source: &[f64],
    label_budget: usize,
) -> ChartResult<Vec<Option<String>>> {
    let finite: Vec<_> = source.iter().copied().filter(|v| v.is_finite()).collect();
    let mut labels = crate::typography::ggplot_numeric_labels(&finite, label_budget)?.into_iter();
    Ok(source
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
        .collect::<Vec<_>>())
}

/// Transform-owned default formatter; explicit guide labels bypass this route.
pub(crate) fn transform_numeric_labels(
    transform: Option<&GgplotTransform>,
    values: &[f64],
    budget: usize,
) -> ChartResult<Vec<Option<String>>> {
    if let Some(transform) = transform
        && let Some(labels) = transform.default_labels(values, budget)?
    {
        return Ok(labels);
    }
    default_numeric_labels(values, budget)
}
