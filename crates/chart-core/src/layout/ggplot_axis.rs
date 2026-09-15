//! Reference positional guide presentation over existing scale and tick components.
use crate::{ChartResult, DiagnosticCode};
/// Trim the domain rule at selected tick positions.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AxisCap {
    /// Retain the complete panel extent.
    #[default]
    None,
    /// Trim the lower coordinate end.
    Lower,
    /// Trim the upper coordinate end.
    Upper,
    /// Trim both coordinate ends.
    Both,
}
/// ggplot axis label and tick policies. Selection and formatting remain independent.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GgplotAxisOptions {
    /// Omit overlapping labels in endpoint-first recursive priority order.
    pub check_overlap: bool,
    /// Number of alternating label rows, starting nearest the axis.
    pub n_dodge: usize,
    /// Domain rule truncation.
    pub cap: AxisCap,
    /// Paint the already selected minor breaks, without labels.
    pub minor_ticks: bool,
    /// Optional logarithmic tick ladder, replacing ordinary tick rules.
    pub logticks: Option<LogTickOptions>,
    /// Stack guides sharing a scale and side in ascending order.
    pub stack_order: Option<u32>,
    /// Destination-unit gap after this member in its stack.
    pub stack_spacing: f64,
}
impl Default for GgplotAxisOptions {
    fn default() -> Self {
        Self {
            check_overlap: false,
            n_dodge: 1,
            cap: AxisCap::None,
            minor_ticks: false,
            logticks: None,
            stack_order: None,
            stack_spacing: 0.,
        }
    }
}
impl GgplotAxisOptions {
    pub(super) fn validate(&self) -> ChartResult<()> {
        if !(1..=4096).contains(&self.n_dodge)
            || !self.stack_spacing.is_finite()
            || self.stack_spacing < 0.
        {
            return Err(crate::scales::error(
                DiagnosticCode::Validation,
                "Axis label dodge count must be 1..4096.",
            ));
        }
        if let Some(o) = &self.logticks {
            o.validate()?;
        }
        Ok(())
    }
}
/// Grid's endpoint-first recursive order, independently within each dodge row.
pub(super) fn label_priority(count: usize, dodge: usize) -> Vec<usize> {
    fn middle(values: &[usize], result: &mut Vec<usize>) {
        if values.is_empty() {
            return;
        }
        let mid = (values.len() - 1) / 2;
        result.push(values[mid]);
        middle(&values[..mid], result);
        middle(&values[mid + 1..], result);
    }
    let mut result = Vec::with_capacity(count);
    for row in 0..dodge.min(count) {
        let values = (row..count).step_by(dodge).collect::<Vec<_>>();
        result.push(values[0]);
        if values.len() > 1 {
            result.push(values[values.len() - 1]);
        }
        if values.len() > 2 {
            middle(&values[1..values.len() - 1], &mut result);
        }
    }
    result
}
#[cfg(test)]
mod tests {
    #[test]
    fn source_log_ladders() {
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/axis-guide-policies.json"
        ))
        .unwrap();
        for case in source["log_ticks"].as_array().unwrap() {
            let actual = super::log_ladder(
                [
                    case["bounds"][0].as_f64().unwrap(),
                    case["bounds"][1].as_f64().unwrap(),
                ],
                case["smallest"].as_f64(),
                4096,
            )
            .unwrap();
            let values = case["values"].as_array().unwrap();
            let kinds = case["kind"].as_array().unwrap();
            assert_eq!(actual.len(), values.len(), "{case}");
            for ((value, kind), (expected, expected_kind)) in
                actual.iter().zip(values.iter().zip(kinds))
            {
                let expected = expected.as_f64().unwrap();
                assert!(
                    (value - expected).abs() <= expected.abs().max(1e-100) * 3e-14,
                    "{value} != {expected}"
                );
                assert_eq!(*kind, expected_kind.as_u64().unwrap() as usize);
            }
        }
    }
    #[test]
    fn source_label_priority() {
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/axis-guide-policies.json"
        ))
        .unwrap();
        for case in source["priority"].as_array().unwrap() {
            let count = case["count"].as_u64().unwrap() as usize;
            let mut expected = case["order"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as usize - 1)
                .collect::<Vec<_>>();
            expected.dedup();
            assert_eq!(super::label_priority(count, 1), expected, "count {count}");
        }
    }
}

/// Logarithmic guide ticks, with lengths relative to the ordinary major tick.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LogTickOptions {
    /// Lengths for powers of ten, five-times powers and intervening values.
    pub lengths: [f64; 3],
    /// Input coordinates already represent logarithms in this base.
    pub prescale_base: Option<f64>,
    /// Smallest positive tick when the source range includes zero or negatives.
    pub negative_small: Option<f64>,
    /// Include ticks in the expanded panel range.
    pub expanded: bool,
}
impl Default for LogTickOptions {
    fn default() -> Self {
        Self {
            lengths: [2.25, 1.5, 0.75],
            prescale_base: None,
            negative_small: None,
            expanded: true,
        }
    }
}
impl LogTickOptions {
    fn validate(&self) -> ChartResult<()> {
        if self.lengths.iter().any(|v| !v.is_finite())
            || self
                .prescale_base
                .is_some_and(|v| !v.is_finite() || v <= 0. || v == 1.)
            || self
                .negative_small
                .is_some_and(|v| !v.is_finite() || v < 1e-100)
        {
            return Err(crate::scales::error(
                DiagnosticCode::Validation,
                "Invalid logarithmic guide tick controls.",
            ));
        }
        Ok(())
    }
}
/// The reference decade ladder in source order; selection is bounded before allocation.
pub(super) fn log_ladder(
    mut bounds: [f64; 2],
    smallest: Option<f64>,
    budget: usize,
) -> ChartResult<Vec<(f64, usize)>> {
    if bounds.iter().any(|v| !v.is_finite()) {
        return Err(crate::scales::error(
            DiagnosticCode::NumericalDomain,
            "Log tick limits must be finite.",
        ));
    }
    let negative = bounds.iter().any(|v| *v <= 0.);
    let large = bounds[0].abs().max(bounds[1].abs());
    let small = smallest.unwrap_or(large.min(1.) * 0.1);
    if negative {
        bounds = [small * 10., large];
    }
    let lower = bounds[0].min(bounds[1]);
    let upper = bounds[0].max(bounds[1]);
    if lower <= 0. || upper <= 0. {
        return Ok(vec![]);
    }
    let start = lower.log10().floor() as i32 - 1;
    let end = upper.log10().ceil() as i32 + 1;
    let span = end - start;
    let detail = if span <= 8 {
        1
    } else if span <= 15 {
        5
    } else {
        10
    };
    let mut groups = [vec![], vec![], vec![]];
    for power in start..=end {
        let ten = 10_f64.powi(power);
        if ten.is_finite() && ten > 0. {
            groups[0].push(ten);
        }
        if detail <= 5 && (5. * ten).is_finite() && ten > 0. {
            groups[1].push(5. * ten);
        }
        if detail == 1 {
            for n in [2., 3., 4., 6., 7., 8., 9.] {
                let value = n * ten;
                if value.is_finite() && value > 0. {
                    groups[2].push(value);
                }
            }
        }
    }
    if negative {
        for values in &mut groups {
            values.retain(|v| *v >= small);
            values.extend(values.clone().into_iter().map(|v| -v));
        }
        groups[0].push(0.);
    }
    crate::limits::require_within(
        groups.iter().map(Vec::len).sum::<usize>() <= budget,
        "logarithmic guide tick",
    )?;
    Ok(groups
        .into_iter()
        .enumerate()
        .flat_map(|(kind, values)| values.into_iter().map(move |v| (v, kind)))
        .collect())
}
pub(super) fn log_ticks(
    axis: &super::ResolvedAxis,
    spec: &super::GuideSpec,
    request: &super::LayoutRequest,
) -> ChartResult<Vec<(crate::composition::ScaleValue, f64, f64)>> {
    use super::ResolvedScale;
    use crate::composition::ScaleValue;
    let Some(options) = spec.ggplot_axis.as_ref().and_then(|o| o.logticks.as_ref()) else {
        return Ok(vec![]);
    };
    let (domain, view) = match &axis.scale {
        ResolvedScale::Linear(s) => (s.domain(), s.viewport()),
        ResolvedScale::Nonlinear(s) => (s.domain(), s.viewport()),
        ResolvedScale::Numeric(s) => (s.domain(), s.viewport()),
        ResolvedScale::Secondary { domain, view, .. } => (*domain, *view),
        ResolvedScale::Unbounded(s) => match (s.invertible_domain(), s.invertible_viewport()) {
            (Some(domain), Some(view)) => (domain, view),
            _ => {
                return Err(crate::scales::error(
                    DiagnosticCode::UnsupportedCapability,
                    "Log tick guides require finite invertible numeric bounds.",
                ));
            }
        },
        _ => {
            return Err(crate::scales::error(
                DiagnosticCode::UnsupportedCapability,
                "Log tick guides require a continuous numeric scale.",
            ));
        }
    };
    let selected = if options.expanded { view } else { domain };
    let raw = [selected.start(), selected.end()];
    let bounds = options
        .prescale_base
        .map_or(raw, |base| raw.map(|x| base.powf(x)));
    let range = super::guide_geometry::range(axis, &std::collections::BTreeMap::new());
    let mut result = vec![];
    for (value, kind) in log_ladder(bounds, options.negative_small, request.max_ticks)? {
        let coordinate = options.prescale_base.map_or(value, |base| value.log(base));
        if !coordinate.is_finite() || (!options.expanded && !domain.contains(coordinate)) {
            continue;
        }
        let value = ScaleValue::Number(coordinate);
        if let Some(position) = axis
            .guide_value_position(&value)?
            .filter(|p| range.contains(*p))
        {
            let geometry = super::guide_geometry::Geometry::resolve(&spec.style, request);
            let shift = spec.translation[usize::from(!spec.side.horizontal())];
            result.push((
                value,
                position + shift + geometry.offset,
                options.lengths[kind],
            ));
        }
    }
    Ok(result)
}
