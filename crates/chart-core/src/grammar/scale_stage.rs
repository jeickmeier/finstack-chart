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
/// Exact timestamp population limits evaluated before floating projection.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampProjection {
    /// Shared source origin; it must agree with the timestamp mapping.
    #[serde(with = "crate::portable::signed")]
    pub origin: i64,
    /// Exact source-unit limits. Descending and constant limits are permitted.
    pub limits: crate::scales::TimeBounds,
    /// Authored calendar unit when supplied; otherwise inherit the source unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<crate::data::TimeUnit>,
}
impl TimestampProjection {
    fn relative_limits(&self) -> ChartResult<Bounds> {
        let relative = |v| {
            let delta = i128::from(v) - i128::from(self.origin);
            if delta.unsigned_abs() > 1_u128 << 53 {
                Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Timestamp population limits exceed exact origin-relative precision.",
                ))
            } else {
                Ok(delta as f64)
            }
        };
        Bounds::new(relative(self.limits.start)?, relative(self.limits.end)?)
    }
    pub(super) fn project(&self, value: i64, outside: ScaleOob) -> Option<i64> {
        let low = self.limits.start.min(self.limits.end);
        let high = self.limits.start.max(self.limits.end);
        match outside {
            ScaleOob::Censor => (low..=high).contains(&value).then_some(value),
            ScaleOob::Squish => Some(value.clamp(low, high)),
            ScaleOob::Keep => Some(value),
        }
    }
}

/// Resolved positional scale stage. Limits are authored in source units.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleProjection {
    /// Canonical positional scale identity.
    pub id: ScaleId,
    /// Exact timestamp bounds, mutually exclusive with numeric limits/transforms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<TimestampProjection>,
    /// Captured pre-statistic bin classification and post-statistic interval mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binned: Option<std::sync::Arc<crate::scales::PreparedGgplotBinnedPosition>>,
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
        if let Some(binned) = &self.binned {
            binned.validate()?;
            if self.timestamp.is_some()
                || self.limits.is_some()
                || self.transform != binned.authored().transform
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Binned positional state cannot also carry another limit or transform policy.",
                ));
            }
        }
        if let Some(time) = &self.timestamp {
            if self.transform.is_some() || self.limits.is_some() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Timestamp population policies cannot also carry numeric limits or transforms.",
                ));
            }
            time.relative_limits()?;
        }
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
    pub(super) fn validate_input(&self, input: &ValueSpace) -> ChartResult<()> {
        if self.binned.is_some() {
            let mut source = input;
            while let ValueSpace::Scaled { input, .. } = source {
                source = input;
            }
            if matches!(
                source,
                ValueSpace::Timestamp { .. }
                    | ValueSpace::Categorical { .. }
                    | ValueSpace::NullableCategorical { .. }
            ) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Positional bins require numeric source values.",
                ));
            }
        }
        if let Some(time) = &self.timestamp {
            match input {
                ValueSpace::Timestamp {
                    origin,
                    representation,
                } if *origin == time.origin
                    && time
                        .unit
                        .as_ref()
                        .is_none_or(|unit| *unit == representation.unit) => {}
                _ => {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Timestamp population policy requires matching source origin and units.",
                    ));
                }
            }
        }
        Ok(())
    }
    /// Project one observation; missing/invalid values remain missing.
    pub fn project(&self, value: f64) -> Option<f64> {
        if let Some(binned) = &self.binned {
            return binned.project_source(Some(value));
        }
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
        if let Some(binned) = &self.binned {
            return binned.project_statistic(value).filter(|v| v.is_finite());
        }
        if !value.is_finite() {
            return None;
        }
        let forward = |v| {
            self.transform
                .map_or(Some(v), |t| t.forward(v).ok().flatten())
        };
        let limits = match &self.timestamp {
            Some(time) => Some(time.relative_limits().ok()?),
            None => self.limits,
        };
        let Some(limits) = limits else {
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
        if self.timestamp.is_some() {
            return input;
        }
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
    if let AxisScale::Binned {
        prepared: Some(ref bins),
        ..
    } = axis.scale
    {
        return Some(ScaleProjection {
            id: axis.id,
            timestamp: None,
            binned: Some(bins.clone()),
            transform: bins.authored().transform,
            limits: None,
            outside: ScaleOob::Keep,
        });
    }
    let (transform, limits) = match axis.scale {
        AxisScale::Nonlinear { transform, domain } => (Some(transform), domain.explicit),
        AxisScale::Linear(domain) | AxisScale::Duration(domain) if domain.explicit.is_some() => {
            (None, domain.explicit)
        }
        _ => return None,
    };
    Some(ScaleProjection {
        id: axis.id,
        timestamp: None,
        binned: None,
        transform,
        limits,
        outside: axis.population_oob.unwrap_or_default(),
    })
}

fn timestamp_projection(axis: &crate::layout::AxisSpec, origin: i64) -> Option<ScaleProjection> {
    use crate::layout::AxisScale;
    if axis.scale_stage == Some(ScaleStage::AfterStatistics) {
        return None;
    }
    let (limits, unit) = match &axis.scale {
        AxisScale::Utc {
            domain: Some(domain),
            ..
        }
        | AxisScale::Date {
            domain: Some(domain),
        } => (*domain, None),
        AxisScale::Calendar { spec, .. } => (
            crate::scales::TimeBounds {
                start: *spec.domain.first()?,
                end: *spec.domain.last()?,
            },
            Some(spec.unit),
        ),
        _ => return None,
    };
    Some(ScaleProjection {
        id: axis.id,
        timestamp: Some(TimestampProjection {
            origin,
            limits,
            unit,
        }),
        binned: None,
        transform: None,
        limits: None,
        outside: axis.population_oob.unwrap_or_default(),
    })
}
fn source_origin(mut value: &Numeric) -> Option<i64> {
    while let Numeric::Scaled { input, .. } = value {
        value = input;
    }
    if let Numeric::Timestamp { origin, .. } = value {
        Some(*origin)
    } else {
        None
    }
}
fn axes_for_layer<'a>(
    layer: &Layer,
    axes: &'a [crate::layout::AxisSpec],
) -> [Option<&'a crate::layout::AxisSpec>; 2] {
    let ids = if layer.orientation == Orientation::Horizontal {
        [layer.scales.y, layer.scales.x]
    } else {
        [layer.scales.x, layer.scales.y]
    };
    ids.map(|id| axes.iter().find(|a| a.id == id))
}
pub(super) fn layer_projections(
    layer: &Layer,
    axes: &[crate::layout::AxisSpec],
) -> [Option<ScaleProjection>; 2] {
    let source = layer.grammar.as_ref().map(|g| &g.source).or({
        if let Mappings::Source(aes) = &layer.mappings {
            Some(aes)
        } else {
            None
        }
    });
    let origins = source
        .map(|a| {
            [
                [&a.x, &a.x2].into_iter().flatten().find_map(source_origin),
                [&a.y, &a.y2, &a.low, &a.high]
                    .into_iter()
                    .flatten()
                    .find_map(source_origin),
            ]
        })
        .unwrap_or([None; 2]);
    let axes = axes_for_layer(layer, axes);
    std::array::from_fn(|i| {
        axes[i].and_then(|axis| {
            origins[i]
                .and_then(|origin| timestamp_projection(axis, origin))
                .or_else(|| projection(axis))
        })
    })
}
pub(super) fn source_layer(layer: &mut Layer, axes: &[crate::layout::AxisSpec]) -> ChartResult<()> {
    if axes.iter().any(|axis| {
        [layer.scales.x, layer.scales.y].contains(&axis.id)
            && matches!(
                axis.scale,
                crate::layout::AxisScale::Numeric(_) | crate::layout::AxisScale::Registered { .. }
            )
            && axis.scale_stage != Some(ScaleStage::AfterStatistics)
    }) {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Numeric knot axes and registered providers require explicit after-statistics projection in this grammar profile.",
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
                if transform == ScaleTransform::Reverse {
                    s.edges.reverse();
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
            Some(
                ValueSpace::Categorical { .. }
                    | ValueSpace::NullableCategorical { .. }
                    | ValueSpace::Timestamp { .. }
            )
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
) -> ChartResult<()> {
    let [mut x, mut y] = layer_projections(layer, axes);
    let [x_axis, y_axis] = axes_for_layer(layer, axes);
    for (policy, axis, space) in [
        (&mut x, x_axis, &domains.x_space),
        (&mut y, y_axis, &domains.y_space),
    ] {
        if let (Some(axis), Some(ValueSpace::Timestamp { origin, .. })) = (axis, space) {
            *policy = timestamp_projection(axis, *origin).or_else(|| policy.clone());
        }
        if let Some(policy) = policy {
            policy.validate()?;
            if let Some(space) = space {
                policy.validate_input(space)?;
            }
        }
    }
    if let Some(scale) = x
        && !matches!(
            domains.x_space,
            Some(ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. })
        )
        && (!matches!(domains.x_space, Some(ValueSpace::Timestamp { .. }))
            || scale.timestamp.is_some())
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
            Some(ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. })
        )
        && (!matches!(domains.y_space, Some(ValueSpace::Timestamp { .. }))
            || scale.timestamp.is_some())
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
    Ok(())
}

pub(super) fn train_binned_axes(
    definition: &mut ChartDefinition,
    source: &crate::data::StoreSnapshot,
    limits: CompileLimits,
) -> ChartResult<()> {
    use crate::{layout::AxisScale, scales::GgplotOob};
    let mut prepared = std::collections::BTreeMap::new();
    let mut remaining = limits.max_prepared_rows;
    for axis in &definition.axes {
        let AxisScale::Binned { spec, .. } = &axis.scale else {
            continue;
        };
        if definition.profile() != Profile::Ggplot2_4_0_3
            || axis.scale_stage == Some(ScaleStage::AfterStatistics)
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Positional bins require the ggplot2 pre-statistic scale stage.",
            ));
        }
        if definition.facets.as_ref().is_some_and(|f| {
            if axis.side.horizontal() {
                f.scales.free_x
            } else {
                f.scales.free_y
            }
        }) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Free-facet positional bin populations require the facet scale training adapter.",
            ));
        }
        let mut spec = spec.as_ref().clone();
        if let Some(oob) = axis.population_oob {
            spec.bins.oob = match oob {
                ScaleOob::Censor => GgplotOob::Censor,
                ScaleOob::Squish => GgplotOob::Squish,
                ScaleOob::Keep => GgplotOob::Keep,
            };
        }
        let values = numeric_population(definition, source, axis.id, &mut remaining)?;
        let bins = spec.train(&values)?.bind_source(&values)?;
        prepared.insert(axis.id, std::sync::Arc::new(bins));
    }
    for axis in &mut definition.axes {
        if let AxisScale::Binned {
            prepared: target, ..
        } = &mut axis.scale
        {
            *target = prepared.remove(&axis.id);
        }
    }
    Ok(())
}

/// Collect filtered source observations for a positional scale before statistics.
/// Shared by population-dependent scale policies; charge each mapped observation
/// and retain missing entries so an empty dataset differs from all-missing data.
fn numeric_population(
    definition: &ChartDefinition,
    source: &crate::data::StoreSnapshot,
    axis: ScaleId,
    remaining: &mut usize,
) -> ChartResult<Vec<Option<crate::interpolate::Number>>> {
    use crate::interpolate::Number;
    let mut values = vec![];
    for layer in &definition.layers {
        let ids = if layer.orientation == Orientation::Horizontal {
            [layer.scales.y, layer.scales.x]
        } else {
            [layer.scales.x, layer.scales.y]
        };
        let Some(dimension) = ids.iter().position(|id| *id == axis) else {
            continue;
        };
        let aes = match &layer.mappings {
            Mappings::Source(aes) => aes.clone(),
            _ => match &layer.grammar {
                Some(grammar) => grammar.source.clone(),
                None => continue,
            },
        };
        let inputs = if dimension == 0 {
            vec![&aes.x, &aes.x2]
        } else {
            vec![&aes.y, &aes.y2, &aes.low, &aes.high]
        };
        let mut input = layer.data;
        let mut filters = layer.filters.iter().collect::<Vec<_>>();
        for _ in 0..=definition.transforms.len() {
            let DataRef::Transform(id) = input else { break };
            let node = definition
                .transforms
                .iter()
                .find(|t| t.id == id)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        "Positional scale source transform is absent.",
                    )
                })?;
            filters.extend(&node.filters);
            input = node.input;
        }
        let DataRef::Dataset(id) = input else {
            return Err(error(
                DiagnosticCode::Validation,
                "Positional scale source graph contains a cycle.",
            ));
        };
        let data = source.dataset(id)?;
        for input in inputs.into_iter().flatten() {
            let space = super::stats::numeric_space(data, input)?;
            if matches!(
                space,
                ValueSpace::Timestamp { .. }
                    | ValueSpace::Categorical { .. }
                    | ValueSpace::NullableCategorical { .. }
            ) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "This positional scale requires numeric source values.",
                ));
            }
            for row in data.rows().filter(|row| {
                filters
                    .iter()
                    .all(|f| super::stats::filter_matches(*row, f) == Some(true))
            }) {
                super::compiler::charge(remaining, 1, "positional scale population")?;
                values.push(super::stats::raw_number(row, input).map(Number));
            }
        }
    }
    Ok(values)
}
