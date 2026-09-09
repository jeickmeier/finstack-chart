//! Positional scale preparation shared by source and generated-stage mappings.
use super::*;
use crate::{
    ChartResult, ScaleId,
    scales::{Bounds, ScaleTransform},
};

/// Population policy for finite values outside positional scale limits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScaleOob {
    /// Replace out-of-limit values with missing values before the consuming statistic.
    #[default]
    Censor,
    /// Clamp transformed values to transformed limits.
    Squish,
    /// Retain finite out-of-limit values; clipping remains a coordinate concern.
    Keep,
}
/// Resolved positional scale stage. Limits are authored in source units.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleProjection {
    /// Canonical positional scale identity.
    pub id: ScaleId,
    /// Optional nonlinear transformation before limits and statistics.
    pub transform: Option<ScaleTransform>,
    /// Explicit source-unit population limits, separate from a viewport.
    pub limits: Option<Bounds>,
    /// Explicit out-of-bounds behavior.
    pub outside: ScaleOob,
}
impl ScaleProjection {
    /// Validate parameters before any source row is evaluated.
    pub fn validate(&self) -> ChartResult<()> {
        if let Some(transform) = self.transform {
            transform.validate()?;
        }
        if let Some(limits) = self.limits {
            for v in [limits.start(), limits.end()] {
                if self
                    .transform
                    .map(|t| t.forward(v))
                    .transpose()?
                    .is_some_and(|v| v.is_none())
                {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Scale population limits are outside the transform domain.",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Project one observation; missing/invalid values remain missing.
    pub fn project(&self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        let forward = |v| {
            self.transform
                .map_or(Some(v), |t| t.forward(v).ok().flatten())
        };
        let value = forward(value)?;
        self.project_transformed(value)
    }
    /// Reapply limits to generated transformed coordinates without applying the transform twice.
    pub fn project_transformed(&self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        let forward = |v| {
            self.transform
                .map_or(Some(v), |t| t.forward(v).ok().flatten())
        };
        let Some(limits) = self.limits else {
            return Some(value);
        };
        let a = forward(limits.start())?;
        let b = forward(limits.end())?;
        let (lo, hi) = (a.min(b), a.max(b));
        match self.outside {
            ScaleOob::Keep => Some(value),
            ScaleOob::Squish => Some(value.clamp(lo, hi)),
            ScaleOob::Censor => (value >= lo && value <= hi).then_some(value),
        }
    }
    pub(super) fn mapping(&self, input: Numeric) -> Numeric {
        if matches!(&input, Numeric::Scaled { scale, .. } if scale.as_ref() == self) {
            input
        } else {
            Numeric::Scaled {
                input: Box::new(input),
                scale: Box::new(self.clone()),
            }
        }
    }
    pub(super) fn space(&self, input: ValueSpace) -> ValueSpace {
        if matches!(&input, ValueSpace::Scaled { scale, .. } if scale == self) {
            input
        } else {
            ValueSpace::Scaled {
                input: Box::new(input),
                scale: self.clone(),
            }
        }
    }
}
pub(super) fn projection(axis: &crate::layout::AxisSpec) -> Option<ScaleProjection> {
    use crate::layout::AxisScale;
    if axis.scale_stage == Some(ScaleStage::AfterStatistics) {
        return None;
    }
    let (transform, limits) = match axis.scale {
        AxisScale::Nonlinear { transform, domain } => (Some(transform), domain.explicit),
        AxisScale::Linear(domain) if domain.explicit.is_some() => (None, domain.explicit),
        _ => return None,
    };
    Some(ScaleProjection {
        id: axis.id,
        transform,
        limits,
        outside: axis.population_oob.unwrap_or_default(),
    })
}

pub(super) fn layer_projections(
    layer: &Layer,
    axes: &[crate::layout::AxisSpec],
) -> [Option<ScaleProjection>; 2] {
    let ids = if layer.orientation == Orientation::Horizontal {
        [layer.scales.y, layer.scales.x]
    } else {
        [layer.scales.x, layer.scales.y]
    };
    ids.map(|id| axes.iter().find(|a| a.id == id).and_then(projection))
}
pub(super) fn source_layer(layer: &mut Layer, axes: &[crate::layout::AxisSpec]) -> ChartResult<()> {
    if axes.iter().any(|axis| {
        [layer.scales.x, layer.scales.y].contains(&axis.id)
            && matches!(axis.scale, crate::layout::AxisScale::Numeric(_))
            && axis.scale_stage != Some(ScaleStage::AfterStatistics)
    }) {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Numeric knot axes require explicit after-statistics projection in this grammar profile.",
        ));
    }
    let [x, y] = layer_projections(layer, axes);
    for p in [x.as_ref(), y.as_ref()].into_iter().flatten() {
        p.validate()?;
    }
    let wrap = |value: &mut Numeric, scale: Option<&ScaleProjection>| {
        if !matches!(value, Numeric::Category(_))
            && let Some(scale) = scale
        {
            *value = scale.mapping(value.clone());
        }
    };
    if let Mappings::Source(aes) = &mut layer.mappings {
        for value in [&mut aes.x, &mut aes.x2].into_iter().flatten() {
            wrap(value, x.as_ref());
        }
        for value in [&mut aes.y, &mut aes.y2, &mut aes.low, &mut aes.high]
            .into_iter()
            .flatten()
        {
            wrap(value, y.as_ref());
        }
    }
    match &mut layer.statistic.parameters {
        StatParameters::Bin(s) => {
            if let Some(scale) = &x
                && let Some(transform) = scale.transform
            {
                for edge in &mut s.edges {
                    *edge = transform.forward(*edge)?.ok_or_else(|| {
                        error(
                            DiagnosticCode::NumericalDomain,
                            "Bin breaks must lie in the scale transform domain.",
                        )
                    })?;
                }
            }
            wrap(&mut s.input, x.as_ref());
        }
        StatParameters::AutoBin(s) => wrap(&mut s.input, x.as_ref()),
        StatParameters::Summary(s) => {
            let input_scale = if layer
                .grammar
                .as_ref()
                .is_some_and(|g| g.source.y.as_ref() == Some(&s.input))
            {
                y.as_ref()
            } else {
                x.as_ref()
            };
            wrap(&mut s.input, input_scale);
        }
        StatParameters::Ols(s) => {
            wrap(&mut s.x, x.as_ref());
            wrap(&mut s.y, y.as_ref());
        }
        StatParameters::Count(s) => {
            if let Some(grammar) = &layer.grammar {
                for value in &mut s.required {
                    if grammar.source.x.as_ref() == Some(value) {
                        wrap(value, x.as_ref());
                    } else if grammar.source.y.as_ref() == Some(value) {
                        wrap(value, y.as_ref());
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
fn needs_projection(space: &Option<ValueSpace>, scale: &ScaleProjection) -> bool {
    !matches!(space, Some(ValueSpace::Scaled { scale: existing, .. }) if existing == scale)
        && !matches!(
            space,
            Some(ValueSpace::Categorical { .. } | ValueSpace::Timestamp { .. })
        )
}
pub(super) fn generated_spaces(
    layer: &Layer,
    axes: &[crate::layout::AxisSpec],
    domains: &mut DomainContributions,
) {
    let [x, y] = layer_projections(layer, axes);
    for (space, scale) in [(&mut domains.x_space, x), (&mut domains.y_space, y)] {
        if let Some(scale) = scale
            && needs_projection(space, &scale)
        {
            *space = Some(scale.space(space.clone().unwrap_or(ValueSpace::Data)));
        }
    }
}
pub(super) fn generated_rows(
    layer: &Layer,
    axes: &[crate::layout::AxisSpec],
    domains: &mut DomainContributions,
    rows: &mut [super::compiler::EncodedRow],
) {
    let [x, y] = layer_projections(layer, axes);
    if let Some(scale) = x
        && !matches!(
            domains.x_space,
            Some(ValueSpace::Categorical { .. } | ValueSpace::Timestamp { .. })
        )
    {
        let project = |v| {
            if needs_projection(&domains.x_space, &scale) {
                scale.project(v)
            } else {
                scale.project_transformed(v)
            }
        };
        for row in rows.iter_mut() {
            row.x = row.x.and_then(project);
            row.x2 = row.x2.and_then(project);
        }
    }
    if let Some(scale) = y
        && !matches!(
            domains.y_space,
            Some(ValueSpace::Categorical { .. } | ValueSpace::Timestamp { .. })
        )
    {
        let project = |v| {
            if needs_projection(&domains.y_space, &scale) {
                scale.project(v)
            } else {
                scale.project_transformed(v)
            }
        };
        for row in rows.iter_mut() {
            row.y = row.y.and_then(project);
            row.y2 = row.y2.and_then(project);
            row.low = row.low.and_then(project);
            row.high = row.high.and_then(project);
        }
    }
    generated_spaces(layer, axes, domains);
}
