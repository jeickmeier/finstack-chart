use super::*;
use crate::grammar::{ClipPolicy, JitterUnits, Position, PreparedGeometry, ValueSpace};
use crate::provenance::Target;
use crate::scales::error;
use crate::scene::{PathCommand, Primitive, SceneItem, Stroke};
use crate::{ChartResult, DiagnosticCode, Point, Rect};
use std::collections::BTreeMap;

pub(super) fn timestamp(value: f64, origin: i64) -> ChartResult<i64> {
    if !value.is_finite() || value.fract() != 0. || value.abs() > (1_u64 << 53) as f64 {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "UTC coordinates must preserve exact integer source ticks.",
        ));
    }
    i64::try_from(i128::from(origin) + value as i128).map_err(|_| {
        error(
            DiagnosticCode::PrecisionLoss,
            "Timestamp origin addition exceeds i64.",
        )
    })
}
impl ResolvedAxis {
    fn map_reference_coordinate(&self, value: f64, space: &ValueSpace) -> ChartResult<Option<f64>> {
        if value.is_infinite() && space == &self.space {
            let pair = match &self.scale {
                ResolvedScale::Linear(s) => Some((s.viewport(), s.range())),
                ResolvedScale::Nonlinear(s) => Some((s.transformed_viewport(), s.range())),
                _ => None,
            };
            if let Some((viewport, range)) = pair {
                return crate::scales::ggplot_coordinate_position(
                    [
                        crate::interpolate::Number(viewport.start()),
                        crate::interpolate::Number(viewport.end()),
                    ],
                    range,
                    value,
                );
            }
        }
        self.map(value, space)
    }
    // Reference positions move within category steps without changing catalog identities.
    fn map_position_coordinate(
        &self,
        value: f64,
        space: &ValueSpace,
        position: &Position,
        recipe: bool,
    ) -> ChartResult<Option<f64>> {
        if !recipe
            && !matches!(
                position,
                Position::Nudge(_)
                    | Position::GgplotDodge(_)
                    | Position::GgplotDodge2(_)
                    | Position::JitterDodge(_)
            )
            || !space.is_categorical()
        {
            return self.map_reference_coordinate(value, space);
        }
        let count = space.category_count().unwrap_or(0);
        if count == 0 {
            return Ok(None);
        }
        let anchor = value.round().clamp(0., count.saturating_sub(1) as f64);
        let Some(mapped) = self.map(anchor, space)? else {
            return Ok(None);
        };
        let step = if count > 1 {
            let other = if anchor < (count - 1) as f64 {
                anchor + 1.
            } else {
                anchor - 1.
            };
            let Some(next) = self.map(other, space)? else {
                return Ok(None);
            };
            (next - mapped) / (other - anchor)
        } else if let ResolvedScale::Band(scale) = &self.scale {
            scale.step()
                * if scale.range().end() >= scale.range().start() {
                    1.
                } else {
                    -1.
                }
        } else {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Single-category positioned axes require a band step.",
            ));
        };
        Ok(Some(mapped + (value - anchor) * step))
    }
    /// Map a prepared layer coordinate using that layer's exact value-space metadata.
    /// Category ordinals never become identities; UTC adds the checked integer origin first.
    pub fn map(&self, value: f64, layer_space: &ValueSpace) -> ChartResult<Option<f64>> {
        if let ResolvedScale::Provider(scale) = &self.scale {
            let semantic = match layer_space {
                space if space.is_categorical() => {
                    space.category_value(value).ok_or_else(|| {
                        error(
                            DiagnosticCode::PrecisionLoss,
                            "Category ordinal does not address its provider catalog.",
                        )
                    })?
                }
                ValueSpace::Timestamp {
                    representation,
                    origin,
                } => crate::composition::ScaleValue::Timestamp {
                    value: timestamp(value, *origin)?,
                    unit: representation.unit,
                },
                space if space == &self.space => crate::composition::ScaleValue::Number(value),
                _ => {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Prepared coordinate space differs from its positional provider.",
                    ));
                }
            };
            return scale.map(&semantic);
        }
        match (&self.scale, layer_space) {
            (ResolvedScale::Unbounded(scale), space) if space == &self.space => {
                scale.map_transformed(value)
            }
            (ResolvedScale::Nonlinear(scale), ValueSpace::Scaled { scale: stage, .. })
                if layer_space == &self.space && stage.transform == Some(scale.transform()) =>
            {
                scale.map_transformed(value)
            }
            (ResolvedScale::Linear(scale), space) if space == &self.space => scale.map(value),
            (ResolvedScale::Numeric(scale), space) if space == &self.space => scale.map(value),
            (ResolvedScale::Nonlinear(scale), space) if space == &self.space => scale.map(value),
            (
                ResolvedScale::Session(scale),
                ValueSpace::Timestamp {
                    representation,
                    origin,
                },
            ) if representation.unit == scale.calendar().unit => {
                scale.map(timestamp(value, *origin)?)
            }
            (
                ResolvedScale::Utc(scale),
                ValueSpace::Timestamp {
                    representation,
                    origin,
                },
            ) if representation.unit == scale.unit() => scale.map(timestamp(value, *origin)?),
            (
                ResolvedScale::Calendar(scale),
                ValueSpace::Timestamp {
                    representation,
                    origin,
                },
            ) if representation.unit == scale.unit() => {
                if let Some(time) = self.spec.resolved_temporal.as_deref()
                    && *origin == scale.origin()
                {
                    scale.map_reference_relative(value, time.date)
                } else {
                    scale.map(timestamp(value, *origin)?)
                }
            }
            (ResolvedScale::Band(_) | ResolvedScale::Point(_), space) if space.is_categorical() => {
                let category = space.category_value(value).ok_or_else(|| {
                    error(
                        DiagnosticCode::PrecisionLoss,
                        "Category ordinal does not address its layer catalog.",
                    )
                })?;
                let crate::composition::ScaleValue::Category(label) = category else {
                    return Ok(None);
                };
                match &self.scale {
                    ResolvedScale::Band(scale) => scale.center(&label),
                    ResolvedScale::Point(scale) => scale.center(&label),
                    _ => unreachable!(),
                }
            }
            _ => Err(error(
                DiagnosticCode::SchemaConflict,
                "Prepared coordinate space is incompatible with its named scale.",
            )),
        }
    }
}

pub(super) struct Output {
    pub(super) coordinate: Option<super::coordinate_map::CoordinateMap>,
    pub(super) coordinate_fixed: Option<Point>,
    pub(super) coordinate_preprojected: bool,
    pub(super) coordinate_remaining: usize,
    pub(super) stroke_end: Option<crate::grammar::LineEnd>,
    pub(super) stroke_join: Option<crate::grammar::LineJoin>,
    pub(super) stroke_remaining: usize,
    pub diagnostics: Vec<crate::Diagnostic>,
    pub hierarchies: BTreeMap<crate::LayerId, super::ResolvedHierarchy>,
    pub interactions: BTreeMap<usize, crate::grammar::GeometryInteraction>,
    pub items: Vec<SceneItem>,
    pub targets: Vec<Vec<Target>>,
    pub omitted: usize,
}
impl Output {
    pub(super) fn interaction(
        &mut self,
        index: usize,
        mut info: crate::grammar::GeometryInteraction,
        request: &LayoutRequest,
    ) -> ChartResult<()> {
        if let Some(map) = &self.coordinate {
            let hit = if self.coordinate_preprojected {
                super::coordinate_hit::clip_presented(
                    info.hit,
                    map,
                    request,
                    &mut self.coordinate_remaining,
                )?
            } else {
                super::coordinate_hit::project(
                    info.hit,
                    map,
                    request,
                    &mut self.coordinate_remaining,
                )?
            };
            let Some(hit) = hit else { return Ok(()) };
            info.hit = hit;
        }
        self.interactions.insert(index, info);
        Ok(())
    }
    pub fn push(
        &mut self,
        item: SceneItem,
        targets: Vec<Target>,
        request: &LayoutRequest,
    ) -> ChartResult<()> {
        self.push_projected(item, targets, request, self.coordinate_preprojected)
    }
    fn push_projected(
        &mut self,
        item: SceneItem,
        targets: Vec<Target>,
        request: &LayoutRequest,
        projected: bool,
    ) -> ChartResult<()> {
        if let Some(map) = self.coordinate.clone() {
            let primitives = if projected {
                vec![item.primitive]
            } else {
                super::coordinate_primitive::project(
                    item.primitive,
                    &map,
                    self.coordinate_fixed,
                    targets.len(),
                    super::coordinate_path::tolerance(request),
                    &mut self.coordinate_remaining,
                    request,
                )?
            };
            for primitive in primitives {
                let clip = if map.clip() == crate::grammar::CoordinateClip::Off {
                    Some(request.figure_bounds.unwrap_or(request.bounds))
                } else {
                    item.clip
                };
                let styled = if matches!(&map.spec,crate::grammar::CoordinateSpec::Radial(v) if v.mode==crate::grammar::RadialMode::Radial)
                    && map.clip() == crate::grammar::CoordinateClip::On
                {
                    super::stroke_outline::expand(
                        primitive,
                        targets.len(),
                        Some(self.stroke_end.unwrap_or_default()),
                        Some(self.stroke_join.unwrap_or_default()),
                        &mut self.stroke_remaining,
                    )?
                } else {
                    vec![primitive]
                };
                for primitive in styled {
                    let clipped = super::coordinate_clip::clip(
                        primitive,
                        &map,
                        targets.len(),
                        super::coordinate_path::tolerance(request),
                        &mut self.coordinate_remaining,
                    )?;
                    for primitive in clipped {
                        self.push_styled(
                            SceneItem {
                                primitive,
                                clip,
                                guide: item.guide.clone(),
                                layer: item.layer,
                            },
                            targets.clone(),
                            request,
                        )?;
                    }
                }
            }
            return Ok(());
        }
        self.push_styled(item, targets, request)
    }
    fn push_styled(
        &mut self,
        item: SceneItem,
        targets: Vec<Target>,
        request: &LayoutRequest,
    ) -> ChartResult<()> {
        if self.stroke_end.is_none() && self.stroke_join.is_none() {
            crate::limits::require_within(
                self.items.len() < request.limits.max_items,
                "layout scene item",
            )?;
            self.items.push(item);
            self.targets.push(targets);
            return Ok(());
        }
        let SceneItem {
            guide,
            layer,
            clip,
            primitive,
        } = item;
        let primitives = super::stroke_outline::expand(
            primitive,
            targets.len(),
            self.stroke_end,
            self.stroke_join,
            &mut self.stroke_remaining,
        )?;
        crate::limits::require_within(
            self.items.len().saturating_add(primitives.len()) <= request.limits.max_items,
            "layout scene item",
        )?;
        for primitive in primitives {
            self.items.push(SceneItem {
                primitive,
                guide: guide.clone(),
                layer,
                clip,
            });
            self.targets.push(targets.clone());
        }
        Ok(())
    }
}
pub(super) fn project(
    chart: &crate::grammar::PreparedChart,
    axes: &BTreeMap<crate::ScaleId, ResolvedAxis>,
    plot: Rect,
    request: &LayoutRequest,
    measurer: &dyn crate::services::TextMeasurer,
) -> ChartResult<Output> {
    let mut out = Output {
        coordinate: None,
        coordinate_fixed: None,
        coordinate_preprojected: false,
        coordinate_remaining: request.limits.max_path_commands,
        stroke_end: None,
        stroke_join: None,
        stroke_remaining: request.limits.max_path_commands,
        diagnostics: Vec::new(),
        hierarchies: BTreeMap::new(),
        interactions: BTreeMap::new(),
        items: vec![],
        targets: vec![],
        omitted: 0,
    };
    for layer in chart.layers().iter().filter(|l| l.visible()) {
        out.stroke_end = None;
        out.stroke_join = None;
        let definition = chart
            .definition()
            .layers
            .iter()
            .find(|l| l.id == layer.id());
        if definition.is_some_and(|l| l.geom == crate::grammar::Geom::Hierarchy) {
            super::hierarchy::project(layer, plot, request, &mut out)?;
            continue;
        }
        let shape = definition.and_then(|l| match l.geom {
            crate::grammar::Geom::ShapeLine { curve, .. } => Some((curve, false)),
            crate::grammar::Geom::ShapeArea { curve, .. } => Some((curve, true)),
            _ => None,
        });
        let link = definition.and_then(|l| match l.geom {
            crate::grammar::Geom::ShapeLink { curve } => Some(curve),
            _ => None,
        });
        let x = &axes[&layer.scales().x];
        let y = &axes[&layer.scales().y];
        out.coordinate = chart
            .definition()
            .coordinate
            .as_ref()
            .map(|spec| {
                super::coordinate_resolve::resolve_with_windows(
                    spec,
                    [x, y],
                    plot,
                    Some(&chart.state().axis_windows()),
                )
                .and_then(|map| map.with_chart_resources(chart))
            })
            .transpose()?;
        if definition.is_some_and(|layer| layer.geography.is_some())
            && let Some(geographic) = out
                .coordinate
                .as_mut()
                .and_then(|map| map.geographic.as_mut())
        {
            // sf transforms supplied feature vertices; overlay paths use coordinate subdivision.
            geographic.vertices_only = matches!(
                chart.definition().coordinate,
                Some(crate::grammar::CoordinateSpec::Geographic(
                    crate::grammar::GeographicCoordinate {
                        projection: crate::grammar::GeoProjectionSelection::Crs(_),
                        ..
                    }
                ))
            );
        }
        let xspace = layer.domains().x_space.as_ref().unwrap_or(&x.space);
        let yspace = layer.domains().y_space.as_ref().unwrap_or(&y.space);
        let clip = Some(if layer.clip() == ClipPolicy::Plot {
            plot
        } else {
            request.figure_bounds.unwrap_or(request.bounds)
        });
        let mut text_bounds = Vec::new();
        let mut text_bytes = 0_usize;
        for (mark_index, mark) in layer.marks().iter().enumerate() {
            out.stroke_end = mark.style.line_end;
            out.stroke_join = mark.style.line_join;
            let output_index = out.items.len();
            let coordinates =
                |px: f64, py: f64, target: &Target, edge: f64| -> ChartResult<Option<Point>> {
                    let (Some(mut a), Some(mut b)) = (
                        x.map_position_coordinate(
                            px,
                            xspace,
                            layer.position(),
                            definition.is_some_and(|l| l.recipe.is_some()),
                        )?,
                        y.map_position_coordinate(
                            py,
                            yspace,
                            layer.position(),
                            definition.is_some_and(|l| l.recipe.is_some()),
                        )?,
                    ) else {
                        return Ok(None);
                    };
                    match layer.position() {
                        Position::Jitter(spec) if spec.units == JitterUnits::Display => {
                            let (dx, dy) = crate::grammar::positions::jitter(
                                spec,
                                target,
                                &Some(mark.group.clone()),
                            );
                            a += dx;
                            b += dy;
                        }
                        Position::Dodge(spec) => {
                            let horizontal =
                                layer.orientation() == crate::grammar::Orientation::Horizontal;
                            let (axis, space, value) = if horizontal {
                                (&y.scale, yspace, py)
                            } else {
                                (&x.scale, xspace, px)
                            };
                            let category = space.category_value(value).ok_or_else(|| {
                                error(
                                    DiagnosticCode::SchemaConflict,
                                    "Dodge requires checked categorical bands.",
                                )
                            })?;
                            let bounds = match axis {
                                ResolvedScale::Band(scale) => {
                                    if let crate::composition::ScaleValue::Category(label) =
                                        &category
                                    {
                                        scale.extent(label)?
                                    } else {
                                        None
                                    }
                                }
                                ResolvedScale::Provider(scale) => scale.band_extent(&category)?,
                                _ => {
                                    return Err(error(
                                        DiagnosticCode::SchemaConflict,
                                        "Dodge requires resolved categorical bands.",
                                    ));
                                }
                            };
                            let Some(bounds) = bounds else {
                                return Ok(None);
                            };
                            let slot = spec
                                .order
                                .iter()
                                .position(|g| g == &mark.group)
                                .ok_or_else(|| {
                                    error(
                                        DiagnosticCode::Validation,
                                        "Dodge group is absent from its fixed order.",
                                    )
                                })?;
                            let width = (bounds.end() - bounds.start()) * spec.width;
                            let offset = width
                                * ((slot as f64 + 0.5 + edge) / spec.order.len() as f64 - 0.5);
                            if horizontal {
                                b += offset;
                            } else {
                                a += offset;
                            }
                        }
                        _ => {}
                    }
                    Ok(Some(Point::new(a, b)?))
                };

            let point =
                |p: Point, target: &Target, edge: f64| coordinates(p.x(), p.y(), target, edge);

            out.coordinate_fixed = match &mark.geometry {
                PreparedGeometry::Point(p)
                | PreparedGeometry::ShapePath { center: p, .. }
                | PreparedGeometry::ShapePathRun { center: p, .. } => {
                    point(*p, &mark.targets[0], 0.)?
                }
                PreparedGeometry::UnboundedPoint(p) => {
                    coordinates(p[0].0, p[1].0, &mark.targets[0], 0.)?
                }
                _ => None,
            };

            if let Some(annotation) = definition.and_then(|l| l.annotation.as_ref()) {
                if let PreparedGeometry::Point(center) = &mark.geometry {
                    if let Some(anchor) = point(*center, &mark.targets[0], 0.)? {
                        super::row_annotation::project(
                            annotation,
                            mark,
                            layer.id(),
                            anchor,
                            clip,
                            request,
                            &mut out,
                        )?;
                    } else {
                        out.omitted += 1;
                    }
                }
                continue;
            }
            if let Some(options) = definition.and_then(|l| l.text.as_ref()) {
                let center = match &mark.geometry {
                    PreparedGeometry::Point(p) => Some([p.x(), p.y()]),
                    PreparedGeometry::UnboundedPoint(p) => Some([p[0].0, p[1].0]),
                    _ => None,
                };
                if let Some(center) = center {
                    if let Some(anchor) = coordinates(center[0], center[1], &mark.targets[0], 0.)? {
                        let mut transformed_mark = None;
                        let anchor = if let Some(map) = &out.coordinate {
                            if let crate::grammar::CoordinateSpec::Radial(v) = &map.spec
                                && v.rotate_angle
                            {
                                let mut copy = mark.clone();
                                let angle = match copy
                                    .aesthetics
                                    .get(&crate::grammar::ValueAesthetic::TextAngle)
                                {
                                    Some(crate::interpolate::Value::Number(n)) => n.0,
                                    _ => options.angle,
                                };
                                let mut angle = (angle
                                    - map.theta_angle(anchor).unwrap_or(0.) * 180.
                                        / std::f64::consts::PI)
                                    .rem_euclid(360.);
                                if angle > 90. && angle < 270. {
                                    angle = (angle + 180.).rem_euclid(360.);
                                }
                                copy.aesthetics.insert(
                                    crate::grammar::ValueAesthetic::TextAngle,
                                    crate::interpolate::Value::Number(crate::interpolate::Number(
                                        angle,
                                    )),
                                );
                                transformed_mark = Some(copy);
                            }
                            map.project(anchor)?
                        } else {
                            Some(anchor)
                        };
                        let Some(anchor) = anchor else {
                            out.omitted += 1;
                            continue;
                        };
                        out.coordinate_preprojected = out.coordinate.is_some();
                        super::text_marks::project(
                            options,
                            transformed_mark.as_ref().unwrap_or(mark),
                            layer.id(),
                            anchor,
                            clip,
                            request,
                            measurer,
                            &mut text_bounds,
                            &mut text_bytes,
                            &mut out,
                        )?;
                        out.coordinate_preprojected = false;
                    } else {
                        out.omitted += 1;
                    }
                }
                continue;
            }

            let mut style = mark.style;
            if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
                && matches!(mark.geometry, PreparedGeometry::Rule { .. })
                && definition.is_some_and(|l| {
                    matches!(
                        l.recipe,
                        Some(crate::grammar::BuiltinRecipe::Interval(
                            crate::grammar::IntervalRecipe {
                                kind: crate::grammar::IntervalKind::Crossbar,
                                ..
                            }
                        ))
                    )
                })
            {
                let Some(crate::grammar::BuiltinRecipe::Interval(spec)) =
                    definition.unwrap().recipe.as_ref()
                else {
                    unreachable!()
                };
                if spec.middle.linewidth.is_none() {
                    style.stroke_width *= spec.fatten.unwrap_or(2.5);
                }
            }
            if let PreparedGeometry::ShapePath { paint, .. }
            | PreparedGeometry::ShapePathRun { paint, .. } = &mark.geometry
            {
                if paint.color_fill() {
                    style.fill = Some(style.stroke.unwrap_or(style.color));
                }
                if *paint == crate::shape::SymbolPaint::ColorFill {
                    style.stroke = None;
                }
            }
            if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
                && matches!(mark.geometry, PreparedGeometry::Point(_))
                && definition.is_some_and(|l| {
                    matches!(
                        l.recipe,
                        Some(crate::grammar::BuiltinRecipe::Interval(
                            crate::grammar::IntervalRecipe {
                                kind: crate::grammar::IntervalKind::PointRange,
                                ..
                            }
                        ))
                    )
                })
            {
                let definition = definition.unwrap();
                let default_size = definition.grammar.as_ref().is_none_or(|g| g.default_size);
                let mapped_size = definition
                    .numeric_scales
                    .contains_key(&crate::grammar::NumericAesthetic::Size)
                    || definition
                        .grammar
                        .as_ref()
                        .is_some_and(|g| g.source.size.is_some());
                let Some(crate::grammar::BuiltinRecipe::Interval(spec)) =
                    definition.recipe.as_ref()
                else {
                    unreachable!()
                };
                style.radius = spec.point.size.unwrap_or(if default_size && !mapped_size {
                    0.5
                } else {
                    style.radius
                }) * spec.fatten.unwrap_or(4.);
                style.stroke_width = spec.point.stroke.unwrap_or(1.);
                style
                    .units
                    .get_or_insert(crate::grammar::AestheticUnits::Millimeters);
            }
            if matches!(mark.geometry, PreparedGeometry::Point(_)) {
                let code = definition
                    .and_then(|l| match &l.recipe {
                        Some(crate::grammar::BuiltinRecipe::Interval(s))
                            if s.kind == crate::grammar::IntervalKind::PointRange =>
                        {
                            s.point.shape
                        }
                        _ => None,
                    })
                    .or_else(|| {
                        match mark.aesthetics.get(&crate::grammar::ValueAesthetic::Shape) {
                            Some(crate::interpolate::Value::Number(v)) => Some(v.0 as u8),
                            _ => None,
                        }
                    });
                if let Some(code) = code {
                    let paint = crate::shape::SymbolPaint::Auto
                        .resolve(crate::shape::SymbolKind::Ggplot(code))?;
                    if paint.color_fill() {
                        style.fill = Some(style.stroke.unwrap_or(style.color));
                    }
                    if paint == crate::shape::SymbolPaint::ColorFill {
                        style.stroke = None;
                    }
                }
            }
            let factor = style
                .units
                .unwrap_or(crate::grammar::AestheticUnits::Destination)
                .factor(request.units);
            style.radius *= factor;
            style.stroke_width *= factor;
            if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
                && style.units.is_none()
                && definition.is_some_and(|l| l.reference_linewidth())
            {
                style.stroke_width =
                    crate::grammar::reference_linewidth(style.stroke_width, request.units);
            }
            if chart.definition().profile() == crate::grammar::Profile::Ggplot2_4_0_3
                && definition.is_some_and(|l| {
                    (l.reference_point()
                        || (matches!(mark.geometry, PreparedGeometry::Point(_))
                            && matches!(
                                l.recipe,
                                Some(crate::grammar::BuiltinRecipe::Interval(
                                    crate::grammar::IntervalRecipe {
                                        kind: crate::grammar::IntervalKind::PointRange,
                                        ..
                                    }
                                ))
                            )))
                        && !l
                            .grammar
                            .as_ref()
                            .is_some_and(|g| g.default_radius == Some(false))
                })
            {
                if matches!(
                    mark.geometry,
                    PreparedGeometry::Point(_) | PreparedGeometry::UnboundedPoint(_)
                ) {
                    style.radius =
                        crate::grammar::reference_point_radius(style.radius, style.stroke_width);
                }
                // R's point stroke parameter is twice its physical outline width.
                style.stroke_width *= 0.5;
                if style.stroke_width == 0. {
                    // The reference PDF device preserves a zero-width point outline
                    // as a 0.01 big-point hairline. Keep that physical width across hosts.
                    style.stroke_width = 0.01
                        * if request.units == crate::services::Units::LogicalPixels {
                            96. / 72.
                        } else {
                            1.
                        };
                }
            }
            let stroke = Stroke {
                color: style.color,
                width: style.stroke_width,
            };
            let item_with_targets = |primitive, target_count| -> ChartResult<SceneItem> {
                Ok(SceneItem {
                    guide: None,
                    layer: Some(layer.id()),
                    clip,
                    primitive: independent_paints(primitive, style, target_count)?,
                })
            };
            let item = |primitive| item_with_targets(primitive, mark.targets.len());
            if let (Some(definition), Some(coordinate)) = (definition, out.coordinate.as_ref())
                && let PreparedGeometry::Recipe(recipe) = &mark.geometry
                && let crate::grammar::PreparedRecipe::Distribution(
                    payload @ crate::grammar::PreparedDistribution::Dot { .. },
                ) = recipe.as_ref()
            {
                let primitives = super::recipe_distributions::project_dot_coordinate(
                    payload,
                    mark,
                    definition,
                    request,
                    &|p| point(p, &mark.targets[0], 0.),
                    coordinate,
                )?;
                for primitive in primitives {
                    out.push_projected(
                        SceneItem {
                            guide: None,
                            layer: Some(layer.id()),
                            clip,
                            primitive,
                        },
                        mark.targets.clone(),
                        request,
                        true,
                    )?;
                }
                continue;
            }
            if let Some(definition) = definition
                && let Some(primitives) = super::recipe_intervals::project(
                    mark,
                    definition,
                    request,
                    plot,
                    &|p| point(p, &mark.targets[0], 0.),
                    [x, y],
                    &|v, is_x| {
                        if is_x {
                            x.map_position_coordinate(v, xspace, layer.position(), true)
                        } else {
                            y.map_position_coordinate(v, yspace, layer.position(), true)
                        }
                    },
                )?
            {
                let arrow = definition.recipe.as_ref().and_then(|r| match r {
                    crate::grammar::BuiltinRecipe::Segment { arrow } => arrow.as_ref(),
                    crate::grammar::BuiltinRecipe::Reference(v) => v.arrow.as_ref(),
                    crate::grammar::BuiltinRecipe::Curve(v) => v.arrow.as_ref(),
                    crate::grammar::BuiltinRecipe::Spoke(v) => v.arrow.as_ref(),
                    _ => None,
                });
                let projected =
                    out.coordinate.is_some() && arrow.is_some() && !primitives.is_empty();
                let primitives = if projected {
                    let map = out.coordinate.as_ref().expect("coordinate selected");
                    let stem = primitives.into_iter().next().expect("nonempty recipe");
                    let stems = super::coordinate_primitive::project(
                        stem,
                        map,
                        None,
                        mark.targets.len(),
                        super::coordinate_path::tolerance(request),
                        &mut out.coordinate_remaining,
                        request,
                    )?;
                    super::recipe_intervals::with_arrows(stems, arrow, &mark.style, request)?
                } else {
                    primitives
                };
                for mut primitive in primitives {
                    if matches!(&mark.geometry, PreparedGeometry::Recipe(recipe) if matches!(recipe.as_ref(),crate::grammar::PreparedRecipe::Distribution(crate::grammar::PreparedDistribution::Outlier{..})))
                        && let Primitive::ShapePath { anchors, .. } = &primitive
                    {
                        out.coordinate_fixed = anchors.first().copied();
                    }
                    if let Primitive::ShapePath { anchors, .. } = &mut primitive
                        && anchors.len() == 1
                        && mark.targets.len() > 1
                    {
                        *anchors = vec![anchors[0]; mark.targets.len()];
                    }
                    let scene_item = if matches!(&mark.geometry, PreparedGeometry::Recipe(recipe) if matches!(recipe.as_ref(), crate::grammar::PreparedRecipe::Distribution(_)))
                    {
                        // Distribution glyphs already resolve symbol fill policy and physical stroke units.
                        SceneItem {
                            guide: None,
                            layer: Some(layer.id()),
                            clip,
                            primitive,
                        }
                    } else {
                        item(primitive)?
                    };
                    out.push_projected(scene_item, mark.targets.clone(), request, projected)?;
                }
                continue;
            }
            match &mark.geometry {
                PreparedGeometry::Recipe(_) => {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Recipe requires its normalized layer definition.",
                    ));
                }
                PreparedGeometry::HierarchyNode(_) | PreparedGeometry::HierarchyLink { .. } => {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Deferred hierarchy geometry has no hierarchy recipe.",
                    ));
                }
                PreparedGeometry::ShapePath {
                    paint,
                    center,
                    geometry,
                    ..
                }
                | PreparedGeometry::ShapePathRun {
                    paint,
                    center,
                    geometry,
                    ..
                } => {
                    let (local_anchors, run): (&[Point], bool) = match &mark.geometry {
                        PreparedGeometry::ShapePath { anchor, .. } => {
                            (std::slice::from_ref(anchor), false)
                        }
                        PreparedGeometry::ShapePathRun { anchors, .. } => (anchors, true),
                        _ => unreachable!("shape path"),
                    };
                    if let Some(center) = point(*center, &mark.targets[0], 0.)? {
                        let map = crate::path::Affine::new([
                            factor,
                            0.,
                            0.,
                            factor,
                            center.x(),
                            center.y(),
                        ])?;
                        let geometry =
                            geometry.transformed(map, 0.01, request.limits.max_path_commands)?;
                        let anchors = local_anchors
                            .iter()
                            .map(|a| {
                                let p = map.point([a.x(), a.y()])?;
                                Point::new(p[0], p[1])
                            })
                            .collect::<ChartResult<Vec<_>>>()?;
                        if geometry.has_segments() || (run && !geometry.commands().is_empty()) {
                            out.push(
                                item(Primitive::ShapePath {
                                    fill_rule: crate::scene::FillRule::NonZero,
                                    dashes: vec![],
                                    geometry,
                                    fill: paint.fills().then_some(style.color),
                                    stroke: paint.strokes().then_some(Stroke {
                                        width: style.stroke_width,
                                        color: style.color,
                                    }),
                                    anchors,
                                })?,
                                mark.targets.clone(),
                                request,
                            )?;
                        }
                    } else {
                        out.omitted += 1;
                    }
                }
                PreparedGeometry::Polygon(points) => {
                    let projected = points
                        .iter()
                        .map(|p| point(*p, &mark.targets[0], 0.))
                        .collect::<ChartResult<Option<Vec<_>>>>()?;
                    if let Some(points) = projected {
                        let mut commands: Vec<_> = points
                            .into_iter()
                            .enumerate()
                            .map(|(i, p)| {
                                if i == 0 {
                                    PathCommand::MoveTo(p)
                                } else {
                                    PathCommand::LineTo(p)
                                }
                            })
                            .collect();
                        commands.push(PathCommand::Close);
                        out.push(
                            item(Primitive::FilledPath {
                                commands,
                                fill: style.color,
                            })?,
                            mark.targets.clone(),
                            request,
                        )?;
                    } else {
                        out.omitted += 1;
                    }
                }
                PreparedGeometry::NativePaint {
                    from,
                    to,
                    painter,
                    parameters,
                } => {
                    if let (Some(a), Some(b)) = (
                        point(*from, &mark.targets[0], 0.)?,
                        point(*to, &mark.targets[0], 0.)?,
                    ) {
                        out.push(
                            item(Primitive::NativePaint {
                                bounds: Rect::new(
                                    a.x().min(b.x()),
                                    a.y().min(b.y()),
                                    (b.x() - a.x()).abs(),
                                    (b.y() - a.y()).abs(),
                                )?,
                                painter: painter.clone(),
                                parameters: parameters.clone(),
                                fill: style.color,
                            })?,
                            mark.targets.clone(),
                            request,
                        )?;
                    } else {
                        out.omitted += 1;
                    }
                }

                PreparedGeometry::BandRun { lower, upper }
                | PreparedGeometry::StackBandRun { lower, upper, .. } => {
                    let sources =
                        if let PreparedGeometry::StackBandRun { sources, .. } = &mark.geometry {
                            Some(sources)
                        } else {
                            None
                        };
                    let mut anchors = vec![];
                    let mut lo = vec![];
                    let mut hi = vec![];
                    let mut targets = vec![];
                    let flush = |out: &mut Output,
                                 lo: &mut Vec<Point>,
                                 hi: &mut Vec<Point>,
                                 targets: &mut Vec<Target>,
                                 anchors: &mut Vec<Point>|
                     -> ChartResult<()> {
                        if lo.is_empty() {
                            return Ok(());
                        }
                        if let Some(crate::grammar::BuiltinRecipe::Density(spec)) =
                            definition.and_then(|l| l.recipe.as_ref())
                        {
                            // Area bands retain data y first and baseline y2 second;
                            // these slots are not sorted geometric lower/upper boundaries.
                            let (baseline, curve) = if definition.is_some_and(|l| {
                                matches!(l.geom, crate::grammar::Geom::Area { .. })
                            }) {
                                (&*hi, &*lo)
                            } else {
                                (&*lo, &*hi)
                            };
                            for primitive in super::recipe_distributions::density_band(
                                baseline,
                                curve,
                                anchors,
                                style,
                                spec.outline,
                            )? {
                                out.push(
                                    SceneItem {
                                        guide: None,
                                        layer: Some(layer.id()),
                                        clip,
                                        primitive,
                                    },
                                    targets.clone(),
                                    request,
                                )?;
                            }
                            targets.clear();
                            anchors.clear();
                            lo.clear();
                            hi.clear();
                            return Ok(());
                        }
                        if let Some((curve, true)) = shape {
                            let data: Vec<_> = lo
                                .iter()
                                .zip(hi.iter())
                                .map(|(a, b)| crate::shape::AreaPoint {
                                    lower: [a.x(), a.y()],
                                    upper: [b.x(), b.y()],
                                })
                                .collect();
                            let geometry = crate::shape::Area::new()
                                .curve(curve)?
                                .generate_with(
                                    &data,
                                    layer
                                        .shape_protocols
                                        .curve()
                                        .map_or(&curve as &dyn crate::shape::CurveFactory, |p| p),
                                    |p, _, _| Ok(Some(*p)),
                                )?
                                .geometry();
                            if !geometry.commands().is_empty() {
                                out.push(
                                    item(Primitive::ShapePath {
                                        fill_rule: crate::scene::FillRule::NonZero,
                                        dashes: vec![],
                                        geometry,
                                        fill: Some(style.color),
                                        stroke: None,
                                        anchors: std::mem::take(anchors),
                                    })?,
                                    std::mem::take(targets),
                                    request,
                                )?;
                            } else {
                                targets.clear();
                            }
                            anchors.clear();
                            lo.clear();
                            hi.clear();
                            return Ok(());
                        }
                        if lo.len() == 1 {
                            out.push(
                                item(Primitive::Rule {
                                    from: lo[0],
                                    to: hi[0],
                                    stroke,
                                })?,
                                std::mem::take(targets),
                                request,
                            )?;
                            anchors.clear();
                            lo.clear();
                            hi.clear();
                            return Ok(());
                        }
                        let mut commands: Vec<_> = lo
                            .iter()
                            .chain(hi.iter().rev())
                            .enumerate()
                            .map(|(i, p)| {
                                if i == 0 {
                                    PathCommand::MoveTo(*p)
                                } else {
                                    PathCommand::LineTo(*p)
                                }
                            })
                            .collect();
                        commands.push(PathCommand::Close);
                        let mirrored: Vec<_> = targets.iter().rev().cloned().collect();
                        targets.extend(mirrored);
                        out.push(
                            item_with_targets(
                                Primitive::FilledPath {
                                    commands,
                                    fill: style.color,
                                },
                                targets.len(),
                            )?,
                            std::mem::take(targets),
                            request,
                        )?;
                        lo.clear();
                        hi.clear();
                        Ok(())
                    };
                    for (index, (a, b)) in lower.iter().zip(upper).enumerate() {
                        let source = sources.map_or(Some(index), |s| s[index]);
                        // Virtual cells only occur with ShapeStack, which has no target-based display displacement.
                        let target = source.map(|i| &mark.targets[i]);
                        let projection_target = target.unwrap_or(&mark.targets[0]);
                        if let (Some(a), Some(b)) = (
                            point(*a, projection_target, 0.)?,
                            point(*b, projection_target, 0.)?,
                        ) {
                            lo.push(a);
                            hi.push(b);
                            if let Some(target) = target {
                                anchors.push(b);
                                targets.push(target.clone());
                            }
                        } else {
                            out.omitted += 1;
                            flush(&mut out, &mut lo, &mut hi, &mut targets, &mut anchors)?;
                        }
                    }
                    flush(&mut out, &mut lo, &mut hi, &mut targets, &mut anchors)?;
                }
                PreparedGeometry::LineRun(points) => {
                    let mut run = Vec::new();
                    let mut targets = vec![];
                    let flush = |out: &mut Output,
                                 run: &mut Vec<Point>,
                                 targets: &mut Vec<Target>|
                     -> ChartResult<()> {
                        if run.is_empty() {
                            return Ok(());
                        }
                        if let Some((curve, false)) = shape {
                            let data: Vec<_> = run.iter().map(|p| [p.x(), p.y()]).collect();
                            let geometry = crate::shape::Line::new()
                                .curve(curve)?
                                .generate_with(
                                    &data,
                                    layer
                                        .shape_protocols
                                        .curve()
                                        .map_or(&curve as &dyn crate::shape::CurveFactory, |p| p),
                                    |p, _, _| Ok(Some(*p)),
                                )?
                                .geometry();
                            if !geometry.commands().is_empty() {
                                out.push(
                                    item(Primitive::ShapePath {
                                        fill_rule: crate::scene::FillRule::NonZero,
                                        dashes: vec![],
                                        geometry,
                                        fill: None,
                                        stroke: Some(stroke),
                                        anchors: run.clone(),
                                    })?,
                                    std::mem::take(targets),
                                    request,
                                )?;
                            } else {
                                targets.clear();
                            }
                            run.clear();
                            return Ok(());
                        }
                        let primitive = if run.len() == 1 {
                            Primitive::Point {
                                center: run[0],
                                radius: stroke.width / 2.,
                                fill: stroke.color,
                            }
                        } else {
                            Primitive::Path {
                                commands: run
                                    .iter()
                                    .enumerate()
                                    .map(|(i, p)| {
                                        if i == 0 {
                                            PathCommand::MoveTo(*p)
                                        } else {
                                            PathCommand::LineTo(*p)
                                        }
                                    })
                                    .collect(),
                                stroke,
                            }
                        };
                        out.push(item(primitive)?, std::mem::take(targets), request)?;
                        run.clear();
                        Ok(())
                    };
                    for (p, target) in points.iter().zip(&mark.targets) {
                        if let Some(p) = point(*p, target, 0.)? {
                            run.push(p);
                            targets.push(target.clone());
                        } else {
                            out.omitted += 1;
                            flush(&mut out, &mut run, &mut targets)?;
                        }
                    }
                    flush(&mut out, &mut run, &mut targets)?;
                }
                PreparedGeometry::Bar { from, to, width } => {
                    if let (Some(a), Some(b)) = (
                        point(*from, &mark.targets[0], 0.)?,
                        point(*to, &mark.targets[0], 0.)?,
                    ) {
                        let primitive =
                            if layer.orientation() == crate::grammar::Orientation::Horizontal {
                                if a.x() == b.x() {
                                    Primitive::Rule {
                                        from: Point::new(a.x(), a.y() - width / 2.)?,
                                        to: Point::new(a.x(), a.y() + width / 2.)?,
                                        stroke,
                                    }
                                } else {
                                    Primitive::Rectangle {
                                        bounds: Rect::new(
                                            a.x().min(b.x()),
                                            a.y() - width / 2.,
                                            (b.x() - a.x()).abs(),
                                            *width,
                                        )?,
                                        fill: style.color,
                                    }
                                }
                            } else if a.y() == b.y() {
                                Primitive::Rule {
                                    from: Point::new(a.x() - width / 2., a.y())?,
                                    to: Point::new(a.x() + width / 2., a.y())?,
                                    stroke,
                                }
                            } else {
                                Primitive::Rectangle {
                                    bounds: Rect::new(
                                        a.x() - width / 2.,
                                        a.y().min(b.y()),
                                        *width,
                                        (b.y() - a.y()).abs(),
                                    )?,
                                    fill: style.color,
                                }
                            };
                        out.push(item(primitive)?, mark.targets.clone(), request)?;
                    } else {
                        out.omitted += 1;
                    }
                }
                PreparedGeometry::Point(_) | PreparedGeometry::UnboundedPoint(_) => {
                    let [px, py] = match &mark.geometry {
                        PreparedGeometry::Point(p) => [p.x(), p.y()],
                        PreparedGeometry::UnboundedPoint(p) => [p[0].0, p[1].0],
                        _ => unreachable!(),
                    };
                    if let Some(center) = coordinates(px, py, &mark.targets[0], 0.)? {
                        // Empty reference glyphs retain their prepared row, just as
                        // empty explicitly selected symbol paths do above.
                        if style.radius > 0. {
                            let mapped_shape =
                                mark.aesthetics.get(&crate::grammar::ValueAesthetic::Shape);
                            let configured_shape = definition.and_then(|l| match &l.recipe {
                                Some(crate::grammar::BuiltinRecipe::Interval(s))
                                    if s.kind == crate::grammar::IntervalKind::PointRange =>
                                {
                                    s.point.shape
                                }
                                _ => None,
                            });
                            if configured_shape.is_none()
                                && matches!(
                                    mapped_shape,
                                    Some(
                                        crate::interpolate::Value::Missing
                                            | crate::interpolate::Value::Null
                                    )
                                )
                            {
                                continue;
                            }
                            let code = configured_shape.or(match mapped_shape {
                                Some(crate::interpolate::Value::Number(v)) => Some(v.0 as u8),
                                _ => None,
                            });

                            if let Some(code) = code {
                                let kind = crate::shape::SymbolKind::Ggplot(code);
                                let policy = crate::shape::SymbolPaint::Auto.resolve(kind)?;
                                let geometry = crate::shape::Symbol::new()
                                    .kind(kind)
                                    .size(std::f64::consts::PI * style.radius * style.radius)
                                    .generate()?
                                    .geometry()
                                    .transformed(
                                        crate::path::Affine::new([
                                            1.,
                                            0.,
                                            0.,
                                            1.,
                                            center.x(),
                                            center.y(),
                                        ])?,
                                        0.01,
                                        request.limits.max_path_commands,
                                    )?;
                                out.push(
                                    item(Primitive::ShapePath {
                                        geometry,
                                        fill: if policy.color_fill() {
                                            Some(style.color)
                                        } else if policy.fills() {
                                            style.fill
                                        } else {
                                            None
                                        },
                                        stroke: policy.strokes().then_some(stroke),
                                        dashes: vec![],
                                        anchors: vec![center],
                                        fill_rule: crate::scene::FillRule::NonZero,
                                    })?,
                                    mark.targets.clone(),
                                    request,
                                )?;
                                continue;
                            }
                            out.push(
                                item(Primitive::Point {
                                    center,
                                    radius: style.radius,
                                    fill: style.color,
                                })?,
                                mark.targets.clone(),
                                request,
                            )?;
                        }
                    } else {
                        out.omitted += 1;
                    }
                }
                PreparedGeometry::Rule { from, to } | PreparedGeometry::Rectangle { from, to } => {
                    let edge = if matches!(mark.geometry, PreparedGeometry::Rectangle { .. })
                        && !definition.is_some_and(|l| l.recipe.is_some())
                    {
                        0.5
                    } else {
                        0.
                    };
                    if let (Some(from), Some(to)) = (
                        point(*from, &mark.targets[0], -edge)?,
                        point(*to, &mark.targets[0], edge)?,
                    ) {
                        if let Some(curve) = link {
                            let geometry = crate::shape::Link::new(curve)?
                                .generate_with(
                                    &(),
                                    layer
                                        .shape_protocols
                                        .curve()
                                        .map_or(&curve as &dyn crate::shape::CurveFactory, |p| p),
                                    |_| Ok([[from.x(), from.y()], [to.x(), to.y()]]),
                                )?
                                .geometry();
                            if !geometry.commands().is_empty() {
                                out.push(
                                    item(Primitive::ShapePath {
                                        fill_rule: crate::scene::FillRule::NonZero,
                                        dashes: vec![],
                                        geometry,
                                        fill: None,
                                        stroke: Some(stroke),
                                        anchors: vec![from, to],
                                    })?,
                                    vec![mark.targets[0].clone(), mark.targets[0].clone()],
                                    request,
                                )?;
                            }
                            continue;
                        }
                        let primitive = if matches!(mark.geometry, PreparedGeometry::Rule { .. }) {
                            Primitive::Rule { from, to, stroke }
                        } else if definition.is_some_and(|l| {
                            matches!(
                                l.recipe,
                                Some(crate::grammar::BuiltinRecipe::Interval(
                                    crate::grammar::IntervalRecipe {
                                        kind: crate::grammar::IntervalKind::Crossbar,
                                        ..
                                    }
                                ))
                            )
                        }) {
                            let mut path = crate::path::Path::new();
                            path.rect(
                                from.x().min(to.x()),
                                from.y().min(to.y()),
                                (to.x() - from.x()).abs(),
                                (to.y() - from.y()).abs(),
                            )?;
                            Primitive::ShapePath {
                                geometry: path.geometry(),
                                fill: style.fill,
                                stroke: Some(stroke),
                                dashes: vec![],
                                anchors: vec![from],
                                fill_rule: crate::scene::FillRule::NonZero,
                            }
                        } else {
                            Primitive::Rectangle {
                                bounds: Rect::new(
                                    from.x().min(to.x()),
                                    from.y().min(to.y()),
                                    (to.x() - from.x()).abs(),
                                    (to.y() - from.y()).abs(),
                                )?,
                                fill: style.color,
                            }
                        };
                        out.push(item(primitive)?, mark.targets.clone(), request)?;
                    } else {
                        out.omitted += 1;
                    }
                }
            }
            if let Some(interaction) = layer.interactions().get(&mark_index)
                && out.items.len() > output_index
            {
                use crate::grammar::HitGeometry;
                let map = |p| point(p, &mark.targets[0], 0.);
                let hit = match &interaction.hit {
                    HitGeometry::Path {
                        geometry,
                        fill_rule,
                        anchor,
                    } => {
                        let commands = geometry.lower(0.01, request.limits.max_path_commands)?;
                        let mut mapped = vec![];
                        let mut valid = true;
                        for command in commands {
                            let command = match command {
                                PathCommand::MoveTo(p) => map(p)?.map(PathCommand::MoveTo),
                                PathCommand::LineTo(p) => map(p)?.map(PathCommand::LineTo),
                                PathCommand::QuadraticTo(a, b) => match (map(a)?, map(b)?) {
                                    (Some(a), Some(b)) => Some(PathCommand::QuadraticTo(a, b)),
                                    _ => None,
                                },
                                PathCommand::CubicTo(a, b, c) => {
                                    match (map(a)?, map(b)?, map(c)?) {
                                        (Some(a), Some(b), Some(c)) => {
                                            Some(PathCommand::CubicTo(a, b, c))
                                        }
                                        _ => None,
                                    }
                                }
                                PathCommand::Close => Some(PathCommand::Close),
                            };
                            if let Some(command) = command {
                                mapped.push(command);
                            } else {
                                valid = false;
                                break;
                            }
                        }
                        if valid {
                            map(*anchor)?
                                .map(|anchor| {
                                    Ok(HitGeometry::Path {
                                        geometry: crate::path::PathGeometry::from_beziers(&mapped)?,
                                        fill_rule: *fill_rule,
                                        anchor,
                                    })
                                })
                                .transpose()?
                        } else {
                            None
                        }
                    }
                    HitGeometry::Point { center, radius } => {
                        map(*center)?.map(|center| HitGeometry::Point {
                            center,
                            radius: *radius,
                        })
                    }
                    HitGeometry::Rectangle { from, to } => match (map(*from)?, map(*to)?) {
                        (Some(from), Some(to)) => Some(HitGeometry::Rectangle { from, to }),
                        _ => None,
                    },
                    HitGeometry::Polygon(points) => points
                        .iter()
                        .map(|p| map(*p))
                        .collect::<ChartResult<Option<Vec<_>>>>()?
                        .map(HitGeometry::Polygon),
                };
                let hit = if let (Some(hit), Some(map)) = (hit.clone(), out.coordinate.as_ref()) {
                    super::coordinate_hit::project(
                        hit,
                        map,
                        request,
                        &mut out.coordinate_remaining,
                    )?
                } else {
                    hit
                };
                if let Some(hit) = hit {
                    for index in output_index..out.items.len() {
                        out.interactions.insert(
                            index,
                            crate::grammar::GeometryInteraction {
                                hit: hit.clone(),
                                ..interaction.clone()
                            },
                        );
                    }
                }
            }
        }
    }
    out.stroke_end = None;
    out.stroke_join = None;
    out.coordinate = None;
    out.coordinate_fixed = None;
    Ok(out)
}

/// Apply independent paints after geometry projection, retaining the existing path engine.
fn independent_paints(
    mut primitive: Primitive,
    style: crate::grammar::Style,
    target_count: usize,
) -> ChartResult<Primitive> {
    if style.fill.is_none() && style.stroke.is_none() && style.line_type.is_none() {
        return Ok(primitive);
    }
    let outline = style.stroke.map(|color| Stroke {
        color,
        width: style.stroke_width,
    });
    if let Some(line_type) = style.line_type {
        let dashes = line_type.pattern(style.stroke_width)?;
        let stroke_visible = line_type != crate::grammar::LineType::Blank;
        match primitive {
            Primitive::Rule { from, to, stroke } => {
                let mut path = crate::path::Path::new();
                path.move_to(from.x(), from.y())?;
                path.line_to(to.x(), to.y())?;
                let anchors = if target_count == 1 {
                    vec![Point::new(
                        from.x().midpoint(to.x()),
                        from.y().midpoint(to.y()),
                    )?]
                } else {
                    vec![from, to]
                };
                return Ok(Primitive::ShapePath {
                    fill_rule: crate::scene::FillRule::NonZero,
                    geometry: path.geometry(),
                    fill: None,
                    stroke: stroke_visible.then_some(outline.unwrap_or(stroke)),
                    dashes,
                    anchors,
                });
            }
            Primitive::Path {
                ref commands,
                stroke,
            } => {
                return Ok(Primitive::ShapePath {
                    fill_rule: crate::scene::FillRule::NonZero,
                    geometry: crate::path::PathGeometry::from_beziers(commands)?,
                    fill: None,
                    stroke: stroke_visible.then_some(outline.unwrap_or(stroke)),
                    dashes,
                    anchors: command_anchors(commands, target_count),
                });
            }
            Primitive::ShapePath { ref mut dashes, .. }
            | Primitive::VectorPath { ref mut dashes, .. } => {
                *dashes = line_type.pattern(style.stroke_width)?;
            }
            _ => {}
        }
    }
    match &mut primitive {
        Primitive::ShapePath { fill, stroke, .. } | Primitive::VectorPath { fill, stroke, .. } => {
            if fill.is_some()
                && let Some(value) = style.fill
            {
                *fill = Some(value);
            }
            if let Some(value) = outline {
                *stroke = Some(value);
            }
            if style.line_type == Some(crate::grammar::LineType::Blank) {
                *stroke = None;
            }
        }
        Primitive::Rule { stroke, .. } | Primitive::Path { stroke, .. } => {
            if let Some(value) = outline {
                *stroke = value;
            }
        }
        Primitive::Point {
            center,
            radius,
            fill,
        } if outline.is_some() => {
            let mut path = crate::path::Path::new();
            path.arc(
                [center.x(), center.y()],
                *radius,
                0.,
                std::f64::consts::TAU,
                false,
            )?;
            path.close_path()?;
            return Ok(Primitive::ShapePath {
                fill_rule: crate::scene::FillRule::NonZero,
                geometry: path.geometry(),
                fill: Some(style.fill.unwrap_or(*fill)),
                stroke: outline.filter(|stroke| {
                    stroke.width > 0. && style.line_type != Some(crate::grammar::LineType::Blank)
                }),
                dashes: style
                    .line_type
                    .map(|l| l.pattern(style.stroke_width))
                    .transpose()?
                    .unwrap_or_default(),
                anchors: vec![*center],
            });
        }
        Primitive::Rectangle { bounds, fill } if outline.is_some() => {
            let mut path = crate::path::Path::new();
            path.rect(
                bounds.origin().x(),
                bounds.origin().y(),
                bounds.width(),
                bounds.height(),
            )?;
            return Ok(Primitive::ShapePath {
                fill_rule: crate::scene::FillRule::NonZero,
                geometry: path.geometry(),
                fill: Some(style.fill.unwrap_or(*fill)),
                stroke: outline.filter(|stroke| {
                    stroke.width > 0. && style.line_type != Some(crate::grammar::LineType::Blank)
                }),
                dashes: style
                    .line_type
                    .map(|l| l.pattern(style.stroke_width))
                    .transpose()?
                    .unwrap_or_default(),
                anchors: vec![Point::new(
                    bounds.origin().x() + bounds.width() / 2.,
                    bounds.origin().y() + bounds.height() / 2.,
                )?],
            });
        }
        Primitive::FilledPath { commands, fill } if outline.is_some() => {
            return Ok(Primitive::ShapePath {
                fill_rule: crate::scene::FillRule::NonZero,
                anchors: command_anchors(commands, target_count),
                geometry: crate::path::PathGeometry::from_beziers(commands)?,
                fill: Some(style.fill.unwrap_or(*fill)),
                stroke: outline.filter(|stroke| {
                    stroke.width > 0. && style.line_type != Some(crate::grammar::LineType::Blank)
                }),
                dashes: style
                    .line_type
                    .map(|l| l.pattern(style.stroke_width))
                    .transpose()?
                    .unwrap_or_default(),
            });
        }
        Primitive::Point { fill, .. }
        | Primitive::Rectangle { fill, .. }
        | Primitive::FilledPath { fill, .. }
        | Primitive::NativePaint { fill, .. } => {
            if let Some(value) = style.fill {
                *fill = value;
            }
        }
        _ => {}
    }
    Ok(primitive)
}

pub(super) fn command_anchors(commands: &[PathCommand], count: usize) -> Vec<Point> {
    commands
        .iter()
        .filter_map(|c| match c {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => Some(*p),
            _ => None,
        })
        .take(count)
        .collect()
}
