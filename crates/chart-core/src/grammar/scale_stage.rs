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
    /// Ordered transformed function result, retaining singleton and missing endpoints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_limits: Option<Box<Vec<crate::interpolate::Number>>>,
    /// Missing positional replacement, already in transformed units.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub missing: Option<crate::interpolate::Number>,
    /// Explicit out-of-bounds behavior.
    pub outside: ScaleOob,
}
impl ScaleProjection {
    /// Validate parameters before any source row is evaluated.
    pub fn validate(&self) -> ChartResult<()> {
        if let Some(limits) = &self.function_limits {
            crate::limits::require_within(
                limits.len() <= crate::interpolate::MAX_VALUES,
                "positional function limits",
            )?;
        }
        if let Some(binned) = &self.binned {
            binned.validate()?;
            if self.timestamp.is_some()
                || self.missing.is_some()
                || self.limits.is_some()
                || self.function_limits.is_some()
                || self.transform != binned.authored().transform
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Binned positional state cannot also carry another limit or transform policy.",
                ));
            }
        }
        if let Some(time) = &self.timestamp {
            if self.transform.is_some() || self.limits.is_some() || self.missing.is_some() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Timestamp population policies cannot also carry numeric limits or transforms.",
                ));
            }
            time.relative_limits()?;
        }
        if let Some(transform) = &self.transform {
            transform.validate()?;
        }
        if let Some(limits) = self.limits {
            for v in [limits.start(), limits.end()] {
                if self
                    .transform
                    .as_ref()
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
        if self.binned.is_some() || self.missing.is_some() {
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
                    "Positional bins and missing replacement require numeric source values.",
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
    /// Project a nullable observation, replacing missing values after transform and OOB.
    pub fn project_optional(&self, value: Option<f64>) -> Option<f64> {
        if let Some(binned) = &self.binned {
            return binned.project_source(value);
        }
        if self.missing.is_none() {
            return value.and_then(|v| self.project_without_missing(v));
        }
        self.project_transformed_optional(
            value.map(|v| self.transform.as_ref().map_or(v, |t| t.forward_raw(v))),
        )
    }
    /// Apply post-statistic OOB and missing replacement without transforming twice.
    pub fn project_transformed_optional(&self, value: Option<f64>) -> Option<f64> {
        let mapped = value.filter(|v| !v.is_nan()).and_then(|v| {
            if self.missing.is_some() && v.is_infinite() {
                Some(v)
            } else {
                self.project_transformed_without_missing(v)
            }
        });
        mapped.or_else(|| self.missing.map(|v| v.0).filter(|v| !v.is_nan()))
    }
    /// Project one observation through transformation, OOB and missing replacement.
    pub fn project(&self, value: f64) -> Option<f64> {
        self.project_optional(Some(value))
    }
    fn project_without_missing(&self, value: f64) -> Option<f64> {
        if let Some(binned) = &self.binned {
            return binned.project_source(Some(value));
        }
        if value.is_nan() {
            return None;
        }
        let value = self
            .transform
            .as_ref()
            .map_or(value, |t| t.forward_raw(value));
        self.project_transformed_without_missing(value)
    }
    /// Reapply limits to generated transformed coordinates without applying the transform twice.
    pub fn project_transformed(&self, value: f64) -> Option<f64> {
        self.project_transformed_optional(Some(value))
    }
    fn project_transformed_without_missing(&self, value: f64) -> Option<f64> {
        if let Some(binned) = &self.binned {
            return binned.project_statistic(value).filter(|v| !v.is_nan());
        }
        if value.is_nan() {
            return None;
        }
        // Reference positional OOB policies operate on finite observations only.
        // Infinities remain data until the Cartesian coordinate stage.
        if value.is_infinite() {
            return Some(value);
        }
        let forward = |v| {
            self.transform
                .as_ref()
                .map_or(Some(v), |t| t.forward(v).ok().flatten())
        };
        if let Some(limits) = &self.function_limits {
            let bounds = [limits.first(), limits.get(1)].map(|v| v.map_or(f64::NAN, |v| v.0));
            let oob = match self.outside {
                ScaleOob::Censor => crate::scales::GgplotOob::Censor,
                ScaleOob::Squish => crate::scales::GgplotOob::Squish,
                ScaleOob::Keep => crate::scales::GgplotOob::Keep,
            };
            return oob.apply(Some(value), bounds);
        }
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
                samples: None,
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
            transform: bins.authored().transform.clone(),
            limits: None,
            function_limits: None,
            missing: None,
            outside: ScaleOob::Keep,
        });
    }
    if let Some(time) = axis.resolved_temporal.as_deref() {
        return timestamp_projection(axis, time.origin);
    }
    let (transform, limits) = match &axis.scale {
        AxisScale::Nonlinear { transform, domain } => (Some(transform), domain.explicit),
        AxisScale::Linear(domain) | AxisScale::Duration(domain)
            if domain.explicit.is_some()
                || axis.resolved_limits.is_some()
                || axis.population_missing.is_some() =>
        {
            (None, domain.explicit)
        }
        AxisScale::Auto if axis.resolved_limits.is_some() || axis.population_missing.is_some() => {
            (None, None)
        }
        _ => return None,
    };
    Some(ScaleProjection {
        id: axis.id,
        timestamp: None,
        binned: None,
        transform: transform.cloned(),
        limits,
        function_limits: axis.resolved_limits.clone(),
        // ggplot2 4.0.3 exposes na.value on scale_*_time but does not forward
        // it to datetime_scale. Retain the authored no-op in the definition.
        missing: if matches!(axis.scale, AxisScale::Duration(_)) {
            None
        } else {
            axis.population_missing
        },
        outside: if axis.oob_function.is_some() {
            ScaleOob::Keep
        } else {
            axis.population_oob.unwrap_or_default()
        },
    })
}

fn timestamp_projection(axis: &crate::layout::AxisSpec, origin: i64) -> Option<ScaleProjection> {
    use crate::layout::AxisScale;
    if axis.scale_stage == Some(ScaleStage::AfterStatistics) {
        return None;
    }
    let (limits, unit) = if let Some(time) = axis.resolved_temporal.as_deref() {
        (
            crate::scales::TimeBounds {
                start: time.origin,
                end: time.origin,
            },
            Some(time.unit),
        )
    } else {
        match &axis.scale {
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
        }
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
        function_limits: axis.resolved_limits.clone(),
        missing: None,
        outside: if axis.oob_function.is_some() {
            ScaleOob::Keep
        } else {
            axis.population_oob.unwrap_or_default()
        },
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
        StatParameters::Distribution(s) => {
            let axis = s.sample_axis();
            let scales = [x.as_ref(), y.as_ref()];
            wrap(&mut s.input, scales[axis]);
            if let Some(p) = &mut s.position {
                wrap(p, scales[1 - axis]);
            }
        }
        StatParameters::Univariate(s) => {
            let axis = s.sample_axis();
            let scales = [x.as_ref(), y.as_ref()];
            wrap(&mut s.input, scales[axis]);
            if let Some(p) = &mut s.second {
                wrap(p, scales[1 - axis]);
            }
            if matches!(s.kind, UnivariateKind::Function { .. }) {
                s.output_scale = y.clone();
            }
        }
        StatParameters::Bin(s) => {
            if let Some(scale) = &x
                && let Some(transform) = &scale.transform
            {
                for edge in &mut s.edges {
                    *edge = transform.forward(*edge)?.ok_or_else(|| {
                        error(
                            DiagnosticCode::NumericalDomain,
                            "Bin breaks must lie in the scale transform domain.",
                        )
                    })?;
                }
                if *transform == ScaleTransform::Reverse {
                    s.edges.reverse();
                }
            }
            wrap(&mut s.input, x.as_ref());
        }
        StatParameters::AutoBin(s) => wrap(&mut s.input, x.as_ref()),
        StatParameters::Summary(s) => {
            let input_scale = if s.ggplot.is_some()
                || layer
                    .grammar
                    .as_ref()
                    .is_some_and(|g| g.source.y.as_ref() == Some(&s.input))
            {
                y.as_ref()
            } else {
                x.as_ref()
            };
            wrap(&mut s.input, input_scale);
            if let Some(n) = s.ggplot.as_mut().and_then(|g| g.position.as_mut()) {
                wrap(n, x.as_ref());
            }
        }
        StatParameters::Ols(s) => {
            wrap(&mut s.x, x.as_ref());
            wrap(&mut s.y, y.as_ref());
        }
        StatParameters::Model(s) => {
            wrap(&mut s.x, x.as_ref());
            wrap(&mut s.y, y.as_ref());
        }
        StatParameters::Spatial(s) => {
            wrap(&mut s.x, x.as_ref());
            wrap(&mut s.y, y.as_ref());
        }
        StatParameters::Count(s) => {
            if let Some(n) = s.ggplot.as_mut().and_then(|g| g.joint_position.as_mut()) {
                wrap(n, y.as_ref());
            }
            if let Some(n) = s.ggplot.as_mut().and_then(|g| g.position.as_mut()) {
                wrap(n, x.as_ref());
            }
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
pub(super) fn mapped_endpoints(layer: &Layer) -> [bool; 4] {
    match &layer.mappings {
        Mappings::Source(a) => [
            a.x2.is_some(),
            a.y2.is_some(),
            a.low.is_some(),
            a.high.is_some(),
        ],
        Mappings::Statistical(a) => [a.x2.is_some(), a.y2.is_some(), false, false],
        Mappings::Binned(a) => [a.x2.is_some(), a.y2.is_some(), false, false],
    }
}
/// Transform newly generated aesthetics as one layer vector across its panels,
/// before any panel applies OOB, palettes, or positions.
pub(super) fn transform_generated_populations(
    groups: &mut [(
        &Layer,
        &[crate::layout::AxisSpec],
        &mut DomainContributions,
        &mut Vec<super::compiler::EncodedRow>,
    )],
) -> ChartResult<()> {
    for dimension in 0..2 {
        let projections = groups
            .iter()
            .map(|(layer, axes, domains, _)| {
                let scale = layer_projections(layer, axes)[dimension].clone()?;
                let space = if dimension == 0 {
                    &domains.x_space
                } else {
                    &domains.y_space
                };
                (needs_projection(space, &scale)
                    && scale
                        .transform
                        .as_ref()
                        .and_then(|t| t.ggplot_transform())
                        .is_some_and(|t| !t.is_pointwise()))
                .then_some(scale)
            })
            .collect::<Vec<_>>();
        let Some(scale) = projections.iter().flatten().next() else {
            continue;
        };
        let (family, reverse) =
            crate::scales::ggplot_numeric_limits::positional_coordinates(scale.transform.clone());
        let candidates: &[usize] = if dimension == 0 {
            &[0, 2]
        } else {
            &[1, 3, 4, 5]
        };
        for &field in candidates {
            let present = field == dimension
                || groups.iter().any(|(_, _, _, rows)| {
                    rows.iter()
                        .any(|r| super::positional_vectors::value(r, field).is_some())
                });
            if !present {
                continue;
            }
            let raw = groups
                .iter()
                .zip(&projections)
                .filter(|(_, p)| p.is_some())
                .flat_map(|((_, _, _, rows), _)| {
                    rows.iter()
                        .map(|r| super::positional_vectors::value(r, field).unwrap_or(f64::NAN))
                })
                .collect::<Vec<_>>();
            let transformed =
                crate::scales::ggplot_numeric_limits::forward_values(&family, reverse, &raw)?;
            let mut values = transformed.into_iter();
            for ((_, _, _, rows), projection) in groups.iter_mut().zip(&projections) {
                if projection.is_none() {
                    continue;
                }
                for row in rows.iter_mut() {
                    let value = values.next().expect("length preserving transform");
                    let value = (!value.is_nan()).then_some(value);
                    match field {
                        0 => row.x = value,
                        1 => row.y = value,
                        2 => row.x2 = value,
                        3 => row.y2 = value,
                        4 => row.low = value,
                        _ => row.high = value,
                    }
                }
            }
        }
        if dimension == 1 {
            let channels = groups
                .iter()
                .flat_map(|(_, _, _, rows)| {
                    rows.iter().flat_map(|r| r.recipe_values.keys().copied())
                })
                .filter(|channel| super::recipe_emit::dependent_channel(*channel))
                .collect::<std::collections::BTreeSet<_>>();
            for channel in channels {
                let raw = groups
                    .iter()
                    .zip(&projections)
                    .filter(|(_, p)| p.is_some())
                    .flat_map(|((_, _, _, rows), _)| {
                        rows.iter().map(|r| match r.recipe_values.get(&channel) {
                            Some(crate::interpolate::Value::Number(v)) => v.0,
                            _ => f64::NAN,
                        })
                    })
                    .collect::<Vec<_>>();
                let mut values =
                    crate::scales::ggplot_numeric_limits::forward_values(&family, reverse, &raw)?
                        .into_iter();
                for ((_, _, _, rows), projection) in groups.iter_mut().zip(&projections) {
                    if projection.is_none() {
                        continue;
                    }
                    for row in rows.iter_mut() {
                        let value = values.next().expect("length preserving transform");
                        if let Some(slot) = row.recipe_values.get_mut(&channel) {
                            *slot = if value.is_nan() {
                                crate::interpolate::Value::Missing
                            } else {
                                crate::interpolate::Value::Number(crate::interpolate::Number(value))
                            };
                        }
                    }
                }
            }
            let raw = groups
                .iter()
                .zip(&projections)
                .filter(|(_, p)| p.is_some())
                .flat_map(|((_, _, _, rows), _)| {
                    rows.iter()
                        .flat_map(|r| r.stat_outliers.iter().map(|v| v.value))
                })
                .collect::<Vec<_>>();
            if !raw.is_empty() {
                let mut values =
                    crate::scales::ggplot_numeric_limits::forward_values(&family, reverse, &raw)?
                        .into_iter();
                for ((_, _, _, rows), projection) in groups.iter_mut().zip(&projections) {
                    if projection.is_none() {
                        continue;
                    }
                    for row in rows.iter_mut() {
                        row.stat_outliers.retain_mut(|outlier| {
                            outlier.value = values.next().expect("length preserving transform");
                            !outlier.value.is_nan()
                        });
                    }
                }
            }
        }
        for ((_, _, domains, _), scale) in groups.iter_mut().zip(projections) {
            if let Some(scale) = scale {
                let space = if dimension == 0 {
                    &mut domains.x_space
                } else {
                    &mut domains.y_space
                };
                *space = Some(scale.space(space.clone().unwrap_or(ValueSpace::Data)));
            }
        }
    }
    Ok(())
}

pub(super) fn generated_rows(
    layer: &Layer,
    axes: &[crate::layout::AxisSpec],
    domains: &mut DomainContributions,
    rows: &mut [super::compiler::EncodedRow],
) -> ChartResult<()> {
    let [mut x, mut y] = layer_projections(layer, axes);
    let [x2, y2, low, high] = mapped_endpoints(layer);
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
                scale.project_optional(v)
            } else if scale.function_limits.is_some()
                && (scale.missing.is_some() || scale.timestamp.is_some())
            {
                v
            } else {
                scale.project_transformed_optional(v)
            }
        };
        for row in rows.iter_mut() {
            row.x = project(row.x);
            if x2 || row.x2.is_some() {
                row.x2 = project(row.x2);
            }
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
                scale.project_optional(v)
            } else if scale.function_limits.is_some()
                && (scale.missing.is_some() || scale.timestamp.is_some())
            {
                v
            } else {
                scale.project_transformed_optional(v)
            }
        };
        for row in rows.iter_mut() {
            row.y = project(row.y);
            for (channel, value) in &mut row.recipe_values {
                if super::recipe_emit::dependent_channel(*channel) {
                    let raw = if let crate::interpolate::Value::Number(value) = value {
                        Some(value.0)
                    } else {
                        None
                    };
                    *value = project(raw)
                        .map(|v| crate::interpolate::Value::Number(crate::interpolate::Number(v)))
                        .unwrap_or(crate::interpolate::Value::Missing);
                }
            }
            row.stat_outliers.retain_mut(|outlier| {
                if let Some(value) = project(Some(outlier.value)) {
                    outlier.value = value;
                    true
                } else {
                    false
                }
            });
            if y2 || row.y2.is_some() {
                row.y2 = project(row.y2);
            }
            if low || row.low.is_some() {
                row.low = project(row.low);
            }
            if high || row.high.is_some() {
                row.high = project(row.high);
            }
        }
    }
    generated_spaces(layer, axes, domains);
    Ok(())
}

pub(super) fn train_binned_axes(
    definition: &mut ChartDefinition,
    source: &crate::data::StoreSnapshot,
    limits: CompileLimits,
    registry: &ExtensionRegistry,
    scope: Option<&super::facets::PanelScope>,
) -> ChartResult<()> {
    use crate::{layout::AxisScale, scales::GgplotOob};
    let mut source_limits = std::collections::BTreeMap::new();
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
        let free = definition.facets.as_ref().is_some_and(|f| {
            if axis.side.horizontal() {
                f.scales.free_x
            } else {
                f.scales.free_y
            }
        });
        if free && scope.is_none() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Free positional bins require a panel population.",
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
        let values = numeric_population(
            definition,
            source,
            axis.id,
            &mut remaining,
            None,
            super::facet_policy::axis_scope(
                definition,
                scope.filter(|_| free),
                axis.side.horizontal(),
            )
            .as_ref(),
            None,
        )?;
        // A retained free panel has an untrained NULL bin population in the
        // reference build, unlike the typed zero-row global scale fallback.
        if free && values.is_empty() && axis.oob_function.is_none() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "A free binned facet has no numeric source population.",
            ));
        }
        // Labels run over the final panel candidates, after classification/reset.
        // Retain their authored policy without invoking callbacks during training.
        let deferred_labels = matches!(
            spec.labels,
            crate::scales::GgplotGuideLabels::Registered { .. }
        )
        .then(|| std::mem::replace(&mut spec.labels, crate::scales::GgplotGuideLabels::Hidden));
        let bins = if axis.limits_function.is_some() {
            let initial =
                function_limits_optional(axis, &values, !values.is_empty(), registry, None)?;
            let null_limits = initial.is_none();
            if axis.oob_function.is_some() {
                source_limits.insert(axis.id, initial.clone().unwrap_or_default());
            }
            let (bins, null_cuts) = spec.train_function(initial, registry)?;
            let bins = if spec.breaks_function.is_some() && axis.oob_function.is_none() {
                bins.bind_initial_source(&values)?
            } else {
                bins
            };
            let reset = bins.reset_population();
            let final_limits =
                function_limits(axis, &reset, !reset.is_empty() || !null_cuts, registry)?;
            if null_limits && final_limits.is_empty() && spec.breaks_function.is_some() {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Positional bin reset cannot assign NULL function limits.",
                ));
            }
            bins.reset_function(final_limits)?
        } else {
            spec.train_with_registry(&values, registry)?
        };
        let bins = if axis.oob_function.is_some() {
            bins.bind_vector_population(values.is_empty())?
        } else {
            bins.bind_source(&values)?
        };
        let bins = if let Some(labels) = deferred_labels {
            bins.with_authored_labels(labels)
        } else {
            bins
        };
        prepared.insert(axis.id, std::sync::Arc::new(bins));
    }
    for axis in &mut definition.axes {
        if let AxisScale::Binned {
            prepared: target, ..
        } = &mut axis.scale
        {
            *target = prepared.remove(&axis.id);
            axis.resolved_limits = source_limits.remove(&axis.id).map(Box::new);
        }
    }
    Ok(())
}

struct TemporalPopulation {
    normalization: Option<crate::scales::GgplotTimestampNormalization>,
    infer: bool,
    numeric_seen: bool,
}

/// Collect filtered source observations for a positional scale before statistics.
/// Shared by population-dependent scale policies; charge each mapped observation
/// and retain missing entries so an empty dataset differs from all-missing data.
#[derive(Default)]
struct PopulationBatches {
    counts: Vec<usize>,
    selected: Vec<bool>,
}
fn vector_transform_axis(definition: &ChartDefinition, id: ScaleId) -> bool {
    definition
        .axes
        .iter()
        .find(|axis| axis.id == id)
        .and_then(projection)
        .and_then(|p| p.transform)
        .and_then(|t| t.ggplot_transform().cloned())
        .is_some_and(|t| !t.is_pointwise())
}
fn numeric_population(
    definition: &ChartDefinition,
    source: &crate::data::StoreSnapshot,
    axis: ScaleId,
    remaining: &mut usize,
    mut temporal: Option<&mut TemporalPopulation>,
    scope: Option<&super::facets::PanelScope>,
    mut batches: Option<&mut PopulationBatches>,
) -> ChartResult<Vec<Option<crate::interpolate::Number>>> {
    if scope.is_none()
        && !(batches.is_some() && vector_transform_axis(definition, axis))
        && let Some(facets) = &definition.facets
    {
        let mut values = Vec::new();
        for key in &facets.order {
            let panel = super::facets::PanelScope {
                axis_group: false,
                fields: facets.fields.clone(),
                names: facets
                    .reference
                    .as_ref()
                    .map(|p| p.field_names.clone())
                    .unwrap_or_default(),
                key: key.clone(),
            };
            values.extend(numeric_population_scoped(
                definition,
                source,
                axis,
                remaining,
                temporal.as_deref_mut(),
                Some(&panel),
                batches.as_deref_mut(),
            )?);
        }
        return Ok(values);
    }
    numeric_population_scoped(
        definition, source, axis, remaining, temporal, scope, batches,
    )
}

fn numeric_population_scoped(
    definition: &ChartDefinition,
    source: &crate::data::StoreSnapshot,
    axis: ScaleId,
    remaining: &mut usize,
    mut temporal: Option<&mut TemporalPopulation>,
    scope: Option<&super::facets::PanelScope>,
    mut batches: Option<&mut PopulationBatches>,
) -> ChartResult<Vec<Option<crate::interpolate::Number>>> {
    use crate::interpolate::Number;
    let mut values = vec![];
    let whole_vector = batches.is_some() && vector_transform_axis(definition, axis);
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
        let mut targets = vec![(&layer.facet, layer.scope)];
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
            targets.push((&node.facet, node.scope));
            filters.extend(&node.filters);
            input = node.input;
        }
        let DataRef::Dataset(id) = input else {
            return Err(error(
                DiagnosticCode::Validation,
                "Positional scale source graph contains a cycle.",
            ));
        };
        if targets
            .iter()
            .any(|(target, _)| !super::facets::targeted(target, scope))
        {
            continue;
        }
        let data = source.dataset(id)?;
        let panel = scope.filter(|_| {
            targets
                .iter()
                .any(|(target, stat)| **target == FacetTarget::Match && *stat != StatScope::Chart)
        });
        if let Some(panel) = panel {
            for field in &panel.fields {
                super::stats::validate_group(data, &Grouping::Field(*field))?;
            }
        }
        for input in inputs.into_iter().flatten() {
            let space = super::stats::numeric_space(data, input)?;
            if let ValueSpace::Timestamp {
                origin,
                representation,
            } = &space
            {
                let target = temporal.as_deref_mut().ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "This positional scale requires numeric source values.",
                    )
                })?;
                let candidate = crate::scales::GgplotTimestampNormalization {
                    origin: *origin,
                    unit: representation.unit,
                    date: false,
                };
                if target.numeric_seen || target.normalization.is_some_and(|old| old != candidate) {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Temporal positional callbacks require a shared source origin and unit.",
                    ));
                }
                target.normalization = Some(candidate);
            } else if let Some(target) = temporal.as_deref_mut() {
                if !target.infer || target.normalization.is_some() {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Temporal positional callbacks require timestamp source values.",
                    ));
                }
                target.numeric_seen = true;
            }
            if matches!(
                space,
                ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. }
            ) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "This positional scale requires numeric source values.",
                ));
            }
            let start = values.len();
            for row in data
                .rows()
                .filter(|row| {
                    whole_vector || panel.is_none_or(|p| super::facets::row_matches(*row, p))
                })
                .filter(|row| {
                    filters
                        .iter()
                        .all(|f| super::stats::filter_matches(*row, f) == Some(true))
                })
            {
                super::compiler::charge(remaining, 1, "positional scale population")?;
                if temporal.is_some()
                    && let Some((value, origin)) = super::stats::timestamp_value(row, input)
                    && (i128::from(value) - i128::from(origin)).unsigned_abs() > 1_u128 << 53
                {
                    return Err(error(
                        DiagnosticCode::PrecisionLoss,
                        "Temporal callback population exceeds exact origin-relative precision.",
                    ));
                }
                values.push(super::stats::raw_number(row, input).map(Number));
                if let Some(batches) = batches.as_deref_mut() {
                    batches
                        .selected
                        .push(panel.is_none_or(|p| super::facets::row_matches(row, p)));
                }
            }
            if let Some(batches) = batches.as_deref_mut() {
                batches.counts.push(values.len() - start);
            }
        }
    }
    Ok(values)
}

pub(super) fn function_limits(
    axis: &crate::layout::AxisSpec,
    values: &[Option<crate::interpolate::Number>],
    trained: bool,
    registry: &ExtensionRegistry,
) -> ChartResult<Vec<crate::interpolate::Number>> {
    function_limits_optional(axis, values, trained, registry, None).map(Option::unwrap_or_default)
}
fn function_limits_optional(
    axis: &crate::layout::AxisSpec,
    values: &[Option<crate::interpolate::Number>],
    trained: bool,
    registry: &ExtensionRegistry,
    batches: Option<&PopulationBatches>,
) -> ChartResult<Option<Vec<crate::interpolate::Number>>> {
    let transform = match &axis.scale {
        crate::layout::AxisScale::Nonlinear { transform, .. } => Some(transform.clone()),
        crate::layout::AxisScale::Binned { spec, .. } => spec.transform.clone(),
        _ => None,
    };
    let (family, reverse) =
        crate::scales::ggplot_numeric_limits::positional_coordinates(transform.clone());
    let raw = values
        .iter()
        .map(|v| v.map_or(f64::NAN, |v| v.0))
        .collect::<Vec<_>>();
    let mut transformed = Vec::with_capacity(values.len());
    let mut offset = 0;
    let default_batch = [values.len()];
    for count in batches.map_or(default_batch.as_slice(), |b| b.counts.as_slice()) {
        transformed.extend(crate::scales::ggplot_numeric_limits::forward_values(
            &family,
            reverse,
            &raw[offset..offset + count],
        )?);
        offset += count;
    }
    debug_assert_eq!(offset, values.len());
    if let Some(batches) = batches {
        debug_assert_eq!(batches.selected.len(), transformed.len());
        transformed = transformed
            .into_iter()
            .zip(&batches.selected)
            .filter_map(|(value, selected)| selected.then_some(value))
            .collect();
    }
    // An empty source retains its existing callback training state. Only a
    // nonempty source wholly excluded by panel selection becomes untrained.
    let trained = trained && (raw.is_empty() || !transformed.is_empty());
    if let Some(limits) = axis.authored_population_limits()? {
        return Ok(Some(
            crate::scales::ggplot_numeric_limits::authored_transformed(
                limits,
                transform.clone(),
                transformed.iter().copied(),
                trained,
            )?
            .to_vec(),
        ));
    }
    if axis.limits_function.is_none() && axis.oob_function.is_some() {
        return super::positional_vectors::train_limits(
            axis,
            transform.clone(),
            transformed.iter().copied(),
            trained,
        )
        .map(Some);
    }
    let result = crate::scales::ggplot_numeric_limits::evaluate_transformed_optional(
        axis.limits_function.as_ref().expect("selected function"),
        transformed.iter().copied(),
        family.clone(),
        reverse,
        axis.resolved_temporal.as_deref().copied(),
        trained,
        registry,
    )?;
    result
        .map(|result| {
            crate::scales::ggplot_numeric_limits::forward_values(
                &family,
                reverse,
                &result.iter().map(|v| v.0).collect::<Vec<_>>(),
            )
            .map(|values| values.into_iter().map(crate::interpolate::Number).collect())
        })
        .transpose()
}

pub(super) fn train_function_axes(
    definition: &mut ChartDefinition,
    source: &crate::data::StoreSnapshot,
    limits: CompileLimits,
    registry: &ExtensionRegistry,
    scope: Option<&super::facets::PanelScope>,
) -> ChartResult<()> {
    use crate::layout::AxisScale;
    let mut trained = std::collections::BTreeMap::new();
    let mut times = std::collections::BTreeMap::new();
    let mut remaining = limits.max_prepared_rows;
    for axis in &definition.axes {
        let transform = match &axis.scale {
            AxisScale::Nonlinear { transform, .. } => Some(transform.clone()),
            AxisScale::Binned { spec, .. } => spec.transform.clone(),
            _ => None,
        };
        if definition.profile() == Profile::Ggplot2_4_0_3
            && axis.scale_stage != Some(ScaleStage::AfterStatistics)
            && let Some(ScaleTransform::Ggplot { transform }) = transform
            && transform.requires_population_validation()
        {
            let values = numeric_population_scoped(
                definition,
                source,
                axis.id,
                &mut remaining,
                None,
                scope,
                None,
            )?;
            transform.validate_population(values.into_iter().flatten().map(|v| v.0))?;
        }
        if axis.limits_function.is_none()
            && axis.numeric_limits.is_none()
            && axis.temporal_limits.is_none()
            && axis.oob_function.is_none()
        {
            continue;
        }
        if definition.profile() != Profile::Ggplot2_4_0_3
            || axis.scale_stage == Some(ScaleStage::AfterStatistics)
            || !matches!(
                axis.scale,
                AxisScale::Auto
                    | AxisScale::Linear(_)
                    | AxisScale::Nonlinear { .. }
                    | AxisScale::Duration(_)
                    | AxisScale::Date { .. }
                    | AxisScale::Utc { .. }
                    | AxisScale::Calendar { .. }
                    | AxisScale::Binned { .. }
            )
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Positional limit functions require a supported numeric or temporal ggplot2 population scale.",
            ));
        }
        let free = definition.facets.as_ref().is_some_and(|f| {
            if axis.side.horizontal() {
                f.scales.free_x
            } else {
                f.scales.free_y
            }
        });
        if free && scope.is_none() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Free positional callbacks require a panel population.",
            ));
        }
        if matches!(axis.scale, AxisScale::Binned { .. }) {
            continue;
        }
        let infer = matches!(axis.scale, AxisScale::Auto)
            && axis.numeric_limits.is_none()
            && axis.temporal_limits.is_none();
        let mut population = TemporalPopulation {
            normalization: None,
            infer,
            numeric_seen: false,
        };
        let is_temporal = infer
            || matches!(
                axis.scale,
                AxisScale::Date { .. } | AxisScale::Utc { .. } | AxisScale::Calendar { .. }
            );
        let mut batches = PopulationBatches::default();
        let values = numeric_population(
            definition,
            source,
            axis.id,
            &mut remaining,
            is_temporal.then_some(&mut population),
            super::facet_policy::axis_scope(
                definition,
                scope.filter(|_| free),
                axis.side.horizontal(),
            )
            .as_ref(),
            Some(&mut batches),
        )?;
        let mut temporal = population.normalization;
        if let (AxisScale::Calendar { spec, .. }, Some(time)) = (&axis.scale, &temporal)
            && spec.unit != time.unit
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Calendar callback source and scale units must agree.",
            ));
        }
        if let Some(time) = &mut temporal {
            time.date = matches!(axis.scale, AxisScale::Date { .. });
        }
        let mut prepared_axis = axis.clone();
        prepared_axis.resolved_temporal = temporal.map(Box::new);
        times.insert(axis.id, prepared_axis.resolved_temporal.clone());
        trained.insert(
            axis.id,
            function_limits_optional(
                &prepared_axis,
                &values,
                !values.is_empty(),
                registry,
                Some(&batches),
            )?
            .unwrap_or_default(),
        );
    }
    for axis in &mut definition.axes {
        axis.resolved_limits = trained.remove(&axis.id).map(Box::new);
        axis.resolved_temporal = times.remove(&axis.id).flatten();
    }
    Ok(())
}
