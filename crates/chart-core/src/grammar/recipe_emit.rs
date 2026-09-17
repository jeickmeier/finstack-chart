//! Shared row input, emission, validation and budget boundary for built-in recipes.
use super::{
    compiler::{
        EncodedRow, GeometryBudget, PositionedLayer, apply_custom_geometry, charge,
        include_geometry, row_style, run_style,
    },
    *,
};
use crate::provenance::Target;
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
fn value_number(value: &crate::interpolate::Value) -> Option<f64> {
    match value {
        crate::interpolate::Value::Number(v) => Some(v.0),
        _ => None,
    }
}
pub(super) fn number(row: &EncodedRow, channel: RecipeAesthetic) -> Option<f64> {
    row.recipe_values.get(&channel).and_then(value_number)
}
/// Defaults apply only to absent mappings; mapped missing values remain missing.
pub(super) fn number_or(row: &EncodedRow, channel: RecipeAesthetic, default: f64) -> Option<f64> {
    if row.recipe_values.contains_key(&channel) {
        number(row, channel)
    } else {
        Some(default)
    }
}
/// Retain positional training when a nonpositional recipe control suppresses ink.
pub(super) fn retain_positions(prepared: &mut PreparedLayer, row: &EncodedRow) {
    for value in [row.x, row.x2]
        .into_iter()
        .flatten()
        .filter(|v| v.is_finite())
    {
        Extent::include(&mut prepared.domains.x, value);
    }
    for value in [row.y, row.y2, row.low, row.high]
        .into_iter()
        .flatten()
        .filter(|v| v.is_finite())
    {
        Extent::include(&mut prepared.domains.y, value);
    }
}
pub(super) fn resolve(
    layer: &Layer,
    data: &crate::data::DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    domains: &DomainContributions,
    limits: CompileLimits,
) -> ChartResult<()> {
    for (channel, input) in &layer.recipe_aes {
        if super::colors::dropped(input, table) {
            continue;
        }
        let input_space = match input {
            ColorInput::Numeric(n) => Some(super::stats::numeric_space(data, n)?),
            ColorInput::Statistical(field) => match &table.schema {
                OutputSchema::Statistical { fields, .. } | OutputSchema::Custom { fields, .. } => {
                    fields
                        .iter()
                        .find(|f| &f.field == field)
                        .map(|f| f.space.clone())
                }
                _ => None,
            },
            _ => None,
        };
        let input = super::colors::read_inputs(input, data, table, rows, limits, None, true)?;
        for (i, row) in rows.iter_mut().enumerate() {
            let mut value = input.values[i]
                .map(|v| crate::interpolate::Value::Number(crate::interpolate::Number(v)))
                .or_else(|| {
                    input.categories[i]
                        .clone()
                        .map(crate::interpolate::Value::Text)
                })
                .unwrap_or(crate::interpolate::Value::Missing);
            if dependent_channel(*channel) {
                let mut v = value_number(&value);
                if input_space.as_ref() != domains.y_space.as_ref()
                    && let Some(ValueSpace::Scaled { scale, .. }) = &domains.y_space
                {
                    v = scale.project_optional(v);
                }
                if *channel == RecipeAesthetic::Lower {
                    row.low = v;
                } else if *channel == RecipeAesthetic::Upper {
                    row.high = v;
                }
                value = v
                    .map(|v| crate::interpolate::Value::Number(crate::interpolate::Number(v)))
                    .unwrap_or(crate::interpolate::Value::Missing);
            }
            row.recipe_values.insert(*channel, value);
        }
    }
    Ok(())
}
pub(super) fn dependent_channel(channel: RecipeAesthetic) -> bool {
    matches!(
        channel,
        RecipeAesthetic::Lower
            | RecipeAesthetic::Upper
            | RecipeAesthetic::Middle
            | RecipeAesthetic::WhiskerLower
            | RecipeAesthetic::WhiskerUpper
            | RecipeAesthetic::NotchLower
            | RecipeAesthetic::NotchUpper
    )
}
pub(super) fn push(
    prepared: &mut PreparedLayer,
    row: &EncodedRow,
    layer: &Layer,
    geometry: PreparedGeometry,
    vertices: &mut usize,
) -> ChartResult<()> {
    let count = match &geometry {
        PreparedGeometry::Recipe(v) => v.points().len().max(1),
        PreparedGeometry::LineRun(p) | PreparedGeometry::Polygon(p) => p.len(),
        PreparedGeometry::Point(_) => 1,
        _ => 4,
    };
    *vertices = vertices.checked_sub(count).ok_or_else(|| {
        error(
            DiagnosticCode::ResourceLimit,
            "Recipe geometry exceeds vertex budget.",
        )
    })?;
    if matches!(layer.recipe, Some(BuiltinRecipe::Rug(_))) {
        if let Some(v) = row.x {
            Extent::include(&mut prepared.domains.x, v);
        }
        if let Some(v) = row.y {
            Extent::include(&mut prepared.domains.y, v);
        }
    }
    super::compiler::include_geometry(&mut prepared.domains, &geometry);
    let component_style = if matches!(layer.recipe, Some(BuiltinRecipe::Interval(_))) {
        let box_fill = if matches!(
            layer.recipe,
            Some(BuiltinRecipe::Interval(IntervalRecipe {
                kind: IntervalKind::Crossbar,
                ..
            }))
        ) && matches!(geometry, PreparedGeometry::Rectangle { .. })
        {
            Some(super::compiler::row_style(layer, row)?.fill)
        } else {
            None
        };
        let mut component_layer = layer.clone();
        let mut component_row = row.clone();
        if let Some(BuiltinRecipe::Interval(spec)) = &layer.recipe {
            let controls = if spec.kind == IntervalKind::Crossbar
                && matches!(geometry, PreparedGeometry::Rule { .. })
            {
                component_layer.style.alpha = None;
                component_row.alpha = None;
                Some(&spec.middle)
            } else if spec.kind == IntervalKind::Crossbar
                && matches!(geometry, PreparedGeometry::Rectangle { .. })
            {
                component_layer.style.alpha = None;
                component_row.alpha = None;
                Some(&spec.box_style)
            } else {
                None
            };
            if let Some(controls) = controls {
                if let Some(color) = controls.color {
                    component_layer.style.color = color;
                    component_layer.style.stroke = Some(color);
                    component_row.color = None;
                    component_row.stroke = None;
                }
                if let Some(width) = controls.linewidth {
                    component_layer.style.stroke_width = width;
                    component_row.stroke_width = None;
                }
                if let Some(line) = controls.line_type {
                    component_layer.style.line_type = Some(line);
                }
            }
            if spec.kind == IntervalKind::PointRange
                && matches!(geometry, PreparedGeometry::Point(_))
                && let Some(fill) = spec.point.fill
            {
                component_layer.style.fill = Some(fill);
                component_row.fill = None;
            }
        }
        let mut component_style = super::compiler::row_style(&component_layer, &component_row)?;
        if let Some(fill) = box_fill {
            component_style.fill = fill;
        }
        component_style
    } else {
        super::compiler::row_style(layer, row)?
    };
    let mut component_style = component_style;
    if matches!(
        layer.recipe,
        Some(BuiltinRecipe::Interval(IntervalRecipe {
            kind: IntervalKind::PointRange,
            ..
        }))
    ) && matches!(geometry, PreparedGeometry::Point(_))
        && let Some(size) = row.size
    {
        component_style.radius = size;
    }
    Arc::make_mut(&mut prepared.marks).push(PreparedMark {
        geometry,
        style: component_style,
        group: row.group.clone().unwrap_or(GroupValue::All),
        targets: vec![row.target.clone()],
        aesthetics: row.values.clone(),
    });
    Ok(())
}
pub(super) fn resolution(values: impl Iterator<Item = f64>) -> f64 {
    super::ggplot_position::resolution(values.collect())
}
pub(super) fn validate(layer: &Layer) -> ChartResult<()> {
    if let Some(recipe) = &layer.recipe {
        super::recipe_distributions::validate(recipe)?;
        super::recipe_intervals::validate(recipe)?;
        super::recipe_surfaces::validate(recipe)?;
        super::recipe_marks::validate(recipe)?;
    }
    Ok(())
}
pub(super) fn setup(
    layer: &Layer,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
) -> ChartResult<()> {
    super::recipe_distributions::setup(layer, rows, limits)?;
    super::recipe_intervals::setup(layer, rows, limits)?;
    super::recipe_surfaces::setup(layer, rows, limits)?;
    super::recipe_marks::setup(layer, rows, limits)
}
pub(super) fn emit(
    layer: &Layer,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<bool> {
    if matches!(layer.recipe, Some(BuiltinRecipe::Smooth)) {
        super::recipe_models::emit(layer, rows, prepared, vertices)?;
        return Ok(false);
    }
    if layer.recipe.is_none() {
        return Ok(false);
    }
    if super::recipe_distributions::emit(layer, rows, prepared, vertices)? {
        return Ok(true);
    }
    if super::recipe_intervals::emit(layer, rows, prepared, vertices)? {
        return Ok(true);
    }
    if super::recipe_surfaces::emit(layer, rows, prepared, vertices)? {
        return Ok(true);
    }
    super::recipe_marks::emit(layer, rows, prepared, vertices)
}

pub(super) fn finish_layer(
    layer: &Layer,
    positioned: PositionedLayer,
    budget: &mut GeometryBudget<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ChartResult<PreparedLayer> {
    let PositionedLayer {
        raw_training: _,
        mut prepared,
        mut encoded,
        mapped_size,
        area_size,
        stack,
    } = positioned;
    let limits = budget.limits;
    let vertices = &mut budget.vertices;
    let extensions = budget.extensions;
    super::after_scale::apply(
        layer,
        budget.geometry_theme,
        mapped_size,
        &mut encoded,
        budget.profile,
    )?;
    // Retain IEEE coordinates through population training. Ordinary reference
    // points defer infinities to coordinate projection; finite geometry families
    // still exclude them before constructing checked Points.
    for row in &mut encoded {
        let retain_infinite_point = budget.profile == Profile::Ggplot2_4_0_3
            && layer.reference_point()
            && !row.values.contains_key(&ValueAesthetic::Shape);
        for value in [
            &mut row.x,
            &mut row.y,
            &mut row.x2,
            &mut row.y2,
            &mut row.low,
            &mut row.high,
        ] {
            *value = value.filter(|v| v.is_finite() || (retain_infinite_point && v.is_infinite()));
        }
    }
    let mut unpainted_categories: Option<Box<[BTreeSet<usize>; 2]>> = None;
    if layer.geom == Geom::Blank
        || (budget.profile == Profile::Ggplot2_4_0_3
            && matches!(
                layer.geom,
                Geom::Point
                    | Geom::ShapeSymbol { .. }
                    | Geom::Line { .. }
                    | Geom::ShapeLine { .. }
                    | Geom::Rule
            ))
    {
        for row in &mut encoded {
            if layer.geom == Geom::Blank
                || [
                    AfterScaleAesthetic::Color,
                    AfterScaleAesthetic::Stroke,
                    AfterScaleAesthetic::Size,
                    AfterScaleAesthetic::LineWidth,
                ]
                .iter()
                .any(|a| row.is_missing(*a))
            {
                // ggplot2 trains position scales before removing missing paint.
                // Retain each finite contribution without emitting an invisible
                // primitive or changing source category ordinals.
                for (axis, values, extent, space) in [
                    (
                        0,
                        [
                            row.x,
                            matches!(layer.geom, Geom::Rule | Geom::Blank)
                                .then_some(row.x2)
                                .flatten(),
                            None,
                            None,
                        ],
                        &mut prepared.domains.x,
                        &prepared.domains.x_space,
                    ),
                    (
                        1,
                        [
                            row.y,
                            matches!(layer.geom, Geom::Rule | Geom::Blank)
                                .then_some(row.y2)
                                .flatten(),
                            (layer.geom == Geom::Blank).then_some(row.low).flatten(),
                            (layer.geom == Geom::Blank).then_some(row.high).flatten(),
                        ],
                        &mut prepared.domains.y,
                        &prepared.domains.y_space,
                    ),
                ] {
                    for value in values.into_iter().flatten().filter(|v| v.is_finite()) {
                        Extent::include(extent, value);
                        if let Some(space) = space
                            && space.is_categorical()
                            && value >= 0.
                            && value.fract() == 0.
                            && value < space.category_count().unwrap() as f64
                        {
                            unpainted_categories.get_or_insert_with(Default::default)[axis]
                                .insert(value as usize);
                        }
                    }
                }
                row.x = None;
                row.y = None;
            }
        }
    }
    let mapped_size = mapped_size || layer.after_scale.contains_key(&AfterScaleAesthetic::Size);
    super::shape_encoding::allocate(layer, &mut encoded, limits, &prepared.shape_protocols)?;
    if matches!(layer.geom, Geom::ShapeArea { .. }) {
        for row in &mut encoded {
            if row.x2.is_none() || row.y2.is_none() {
                row.x = None;
                row.y = None;
            }
        }
    }
    prepared.unpainted_categories = unpainted_categories;
    if super::geography_geometry::emit(
        layer,
        &encoded,
        &mut prepared,
        vertices,
        budget.geography_coordinate,
    )? {
        return Ok(prepared);
    }
    if super::recipe_emit::emit(layer, &encoded, &mut prepared, vertices)? {
        super::orientation::output(&mut prepared)?;
        return Ok(prepared);
    }
    if layer.geom == Geom::Blank {
        return Ok(prepared);
    }
    let mut samples = vec![];
    if layer.geom == Geom::Hierarchy {
        super::hierarchy::emit(&mut prepared, layer, encoded, vertices)?;
    } else if let Some(stack) = stack.filter(|_| matches!(layer.geom, Geom::ShapeArea { .. })) {
        super::stack_position::emit(&mut prepared, layer, &encoded, stack, vertices)?;
    } else if let Some((_, connect_gaps)) = layer.geom.run() {
        let mut groups: Vec<(GroupValue, Vec<EncodedRow>)> = vec![];
        let mut indexes = BTreeMap::new();
        let mut boundary = 0;
        for row in encoded {
            let valid = row.x.is_some() && row.y.is_some() && row.group.is_some();
            if !valid {
                prepared.invalid_geometry += 1;
                if samples.len() < 32
                    && let Some(key) = row.key
                {
                    samples.push(key);
                }
            }
            let Some(group) = &row.group else {
                if !connect_gaps {
                    boundary += 1;
                }
                continue;
            };
            let index_key = (group.clone(), boundary);
            let i = if let Some(&i) = indexes.get(&index_key) {
                i
            } else {
                if groups.len() >= limits.max_groups {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Geometry group/run boundary budget exceeded.",
                    ));
                }
                let i = groups.len();
                indexes.insert(index_key, i);
                groups.push((group.clone(), vec![]));
                i
            };
            groups[i].1.push(row);
        }
        let segment_styles = budget.profile == Profile::Ggplot2_4_0_3
            && matches!(
                layer.geom,
                Geom::Line { .. }
                    | Geom::ShapeLine {
                        curve: crate::shape::CurveSpec::Linear,
                        ..
                    }
            );
        if segment_styles {
            let mut varied = false;
            let mut non_solid = false;
            for (_, rows) in &groups {
                varied |= run_style(layer, rows.iter()).is_err();
                for row in rows.iter().filter(|r| r.x.is_some() && r.y.is_some()) {
                    non_solid |= row_style(layer, row)?
                        .line_type
                        .is_some_and(|t| t != LineType::Solid);
                }
            }
            if varied && non_solid {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Ggplot lines cannot vary color, alpha, linewidth or line type when any line is non-solid.",
                ));
            }
        }
        for (group, rows) in groups {
            let resolved = run_style(layer, rows.iter());
            let varying = segment_styles && resolved.is_err();
            let style = if varying {
                LineBlockStyle::Varying(layer)
            } else {
                LineBlockStyle::Constant(resolved?)
            };

            // Missing x has no sortable position. It separates authored blocks before x ordering.
            let mut block = vec![];
            for row in rows {
                if row.x.is_none() {
                    if !connect_gaps {
                        emit_line_block(
                            &mut prepared,
                            &mut block,
                            &group,
                            style,
                            layer.geom,
                            vertices,
                            limits,
                        )?;
                    }
                } else {
                    block.push(row);
                }
            }
            emit_line_block(
                &mut prepared,
                &mut block,
                &group,
                style,
                layer.geom,
                vertices,
                limits,
            )?;
        }
    } else {
        let candle_colors = layer.candle_colors;
        for row in encoded {
            let base_style = row_style(layer, &row)?;
            let style = if mapped_size {
                row.size
                    .filter(|v| {
                        *v > 0.
                            || (budget.profile == Profile::Ggplot2_4_0_3
                                && ((!v.is_nan() && layer.reference_point())
                                    || (layer.reference_linewidth() && *v == 0.)))
                    })
                    .map(|size| Style {
                        radius: size,
                        stroke_width: row.stroke_width.unwrap_or(
                            if area_size
                                || (budget.profile == Profile::Ggplot2_4_0_3
                                    && matches!(layer.geom, Geom::Point))
                            {
                                base_style.stroke_width
                            } else {
                                size
                            },
                        ),
                        ..base_style
                    })
            } else {
                Some(base_style)
            };
            if let Geom::Ohlc { width } = layer.geom
                && let (
                    Some(x),
                    Some(open),
                    Some(close),
                    Some(low),
                    Some(high),
                    Some(group),
                    Some(style),
                ) = (
                    row.x,
                    row.y,
                    row.y2,
                    row.low,
                    row.high,
                    row.group.as_ref(),
                    style,
                )
                && low <= open
                && open <= high
                && low <= close
                && close <= high
            {
                let style = if row.color.is_none() {
                    candle_colors.map_or(style, |c| Style {
                        color: super::numeric_aesthetics::apply_opacity(
                            if close >= open { c.up } else { c.down },
                            row.opacity,
                        ),
                        ..style
                    })
                } else {
                    style
                };
                charge(vertices, 6, "OHLC vertex")?;
                for geometry in [
                    PreparedGeometry::Rule {
                        from: Point::new(x, low)?,
                        to: Point::new(x, high)?,
                    },
                    PreparedGeometry::Bar {
                        from: Point::new(x, open)?,
                        to: Point::new(x, close)?,
                        width,
                    },
                ] {
                    include_geometry(&mut prepared.domains, &geometry);
                    Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                        aesthetics: row.values.clone(),
                        geometry,
                        targets: vec![row.target.clone()],
                        group: group.clone(),
                        style,
                    });
                }
                continue;
            }
            let geometry = match (row.x, row.y) {
                (Some(x), Some(y)) => match layer.geom {
                    Geom::ShapeArc { .. } | Geom::ShapePie { .. } | Geom::ShapeSymbol { .. } => {
                        Some(super::shape_encoding::geometry(
                            layer.geom,
                            &row,
                            Point::new(x, y)?,
                            limits,
                            prepared.shape_protocols.symbol(),
                        )?)
                    }
                    Geom::ShapeLinkRadial { .. } => Some(super::radial_shapes::link_geometry(
                        layer.geom,
                        &row,
                        Point::new(x, y)?,
                        limits,
                    )?),
                    Geom::Point => match row.values.get(&ValueAesthetic::Shape) {
                        Some(
                            crate::interpolate::Value::Missing | crate::interpolate::Value::Null,
                        ) => None,
                        Some(crate::interpolate::Value::Number(crate::interpolate::Number(
                            code,
                        ))) => style
                            .map(|style| {
                                super::shape_encoding::geometry(
                                    Geom::ShapeSymbol {
                                        kind: crate::shape::SymbolKind::Ggplot(*code as u8),
                                        size: {
                                            let radius = if budget.profile == Profile::Ggplot2_4_0_3
                                                && !area_size
                                                && !layer.grammar.as_ref().is_some_and(|g| {
                                                    g.default_radius == Some(false)
                                                }) {
                                                // gg_par fontsize = size * .pt + stroke * .stroke / 2;
                                                // R's circle glyph radius is 3/8 of that device fontsize.
                                                super::reference_point_radius(
                                                    style.radius,
                                                    style.stroke_width,
                                                )
                                            } else {
                                                style.radius
                                            };
                                            // Grid retains nonfinite point sizes in the built
                                            // data/grob but paints no glyph for either infinity.
                                            if radius.is_finite() {
                                                std::f64::consts::PI * radius * radius
                                            } else {
                                                0.
                                            }
                                        },
                                        paint: crate::shape::SymbolPaint::Auto,
                                    },
                                    &row,
                                    Point::new(x, y)?,
                                    limits,
                                    None,
                                )
                            })
                            .transpose()?,
                        None if budget.profile == Profile::Ggplot2_4_0_3
                            && style.is_some_and(|s| s.radius.is_infinite()) =>
                        {
                            Some(super::shape_encoding::geometry(
                                Geom::ShapeSymbol {
                                    kind: crate::shape::SymbolKind::Ggplot(19),
                                    size: 0.,
                                    paint: crate::shape::SymbolPaint::Auto,
                                },
                                &row,
                                Point::new(x, y)?,
                                limits,
                                None,
                            )?)
                        }
                        None if x.is_infinite() || y.is_infinite() => {
                            Some(PreparedGeometry::UnboundedPoint([
                                crate::interpolate::Number(x),
                                crate::interpolate::Number(y),
                            ]))
                        }
                        None => Some(PreparedGeometry::Point(Point::new(x, y)?)),
                        _ => unreachable!("validated point shape"),
                    },
                    Geom::Bar { width, .. } => row
                        .y2
                        .map(|y2| {
                            Ok(PreparedGeometry::Bar {
                                from: Point::new(x, y)?,
                                to: Point::new(x, y2)?,
                                width,
                            })
                        })
                        .transpose()?,
                    Geom::Ohlc { .. } => None,
                    Geom::Rule | Geom::ShapeLink { .. } => match (row.x2, row.y2) {
                        (Some(x2), Some(y2)) => Some(PreparedGeometry::Rule {
                            from: Point::new(x, y)?,
                            to: Point::new(x2, y2)?,
                        }),
                        _ => None,
                    },
                    Geom::Rectangle => match (row.x2, row.y2) {
                        (Some(x2), Some(y2)) => Some(PreparedGeometry::Rectangle {
                            from: Point::new(x, y)?,
                            to: Point::new(x2, y2)?,
                        }),
                        _ => None,
                    },
                    Geom::Blank
                    | Geom::Hierarchy
                    | Geom::Line { .. }
                    | Geom::ShapeLineRadial { .. }
                    | Geom::ShapeAreaRadial { .. }
                    | Geom::ShapeLine { .. }
                    | Geom::ShapeArea { .. }
                    | Geom::Area { .. }
                    | Geom::Ribbon { .. } => None,
                },
                _ => None,
            };
            if let (Some(geometry), Some(style), Some(group)) = (geometry, style, row.group) {
                let n = match geometry {
                    PreparedGeometry::Recipe(ref v) => v.points().len(),
                    PreparedGeometry::ShapePathRun {
                        ref geometry,
                        ref anchors,
                        ..
                    } => geometry.commands().len().saturating_add(anchors.len()),
                    PreparedGeometry::ShapePath { ref geometry, .. } => {
                        geometry.commands().len() + 1
                    }
                    PreparedGeometry::HierarchyNode(_) | PreparedGeometry::HierarchyLink { .. } => {
                        0
                    }
                    PreparedGeometry::Point(_) | PreparedGeometry::UnboundedPoint(_) => 1,
                    PreparedGeometry::Rule { .. } => 2,
                    PreparedGeometry::Rectangle { .. } | PreparedGeometry::Bar { .. } => 4,
                    PreparedGeometry::LineRun(_)
                    | PreparedGeometry::StackBandRun { .. }
                    | PreparedGeometry::BandRun { .. }
                    | PreparedGeometry::Polygon(_)
                    | PreparedGeometry::NativePaint { .. } => 0,
                };
                charge(vertices, n, "vertex")?;
                include_geometry(&mut prepared.domains, &geometry);
                Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                    aesthetics: row.values.clone(),
                    geometry,
                    targets: if matches!(layer.geom, Geom::ShapeLinkRadial { .. }) {
                        vec![row.target.clone(), row.target]
                    } else {
                        vec![row.target]
                    },
                    group,
                    style,
                });
            } else {
                prepared.invalid_geometry += 1;
                if samples.len() < 32
                    && let Some(key) = row.key
                {
                    samples.push(key);
                }
            }
        }
    }
    stats::warning(
        layer.invalid,
        prepared.invalid_geometry,
        samples,
        format!(
            "Geometry excluded {} rows with invalid required coordinates, groups or sizes; gaps are not zero values.",
            prepared.invalid_geometry
        ),
        diagnostics,
    )?;
    if let Some(extension) = &layer.geometry_extension {
        apply_custom_geometry(extensions, extension, layer, &mut prepared, vertices)?;
    }
    super::orientation::output(&mut prepared)?;
    Ok(prepared)
}

#[derive(Clone, Copy)]
enum LineBlockStyle<'a> {
    Constant(Style),
    Varying(&'a Layer),
}

fn emit_line_block(
    prepared: &mut PreparedLayer,
    rows: &mut Vec<EncodedRow>,
    group: &GroupValue,
    block_style: LineBlockStyle<'_>,
    geom: Geom,
    vertices: &mut usize,
    limits: CompileLimits,
) -> ChartResult<()> {
    let style = match block_style {
        LineBlockStyle::Constant(style) => style,
        LineBlockStyle::Varying(layer) => layer.style.resolve(),
    };
    if matches!(
        geom,
        Geom::ShapeLineRadial { .. } | Geom::ShapeAreaRadial { .. }
    ) {
        return super::radial_shapes::emit_block(
            prepared, rows, group, style, geom, vertices, limits,
        );
    }
    let (order, connect) = geom.run().expect("run geometry");
    if order == LineOrder::X {
        rows.sort_by(|a, b| {
            // Signed zeros are equal x values and therefore use insertion ordinal.
            a.x.partial_cmp(&b.x)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.ordinal.cmp(&b.ordinal))
        });
    }
    if let LineBlockStyle::Varying(layer) = block_style {
        let mut previous: Option<EncodedRow> = None;
        for row in rows.drain(..) {
            if row.x.is_none() || row.y.is_none() {
                if !connect {
                    previous = None;
                }
                continue;
            }
            if let Some(before) = previous.take() {
                let style = row_style(layer, &before)?;
                charge(vertices, 2, "styled line segment vertex")?;
                let geometry = PreparedGeometry::LineRun(vec![
                    Point::new(before.x.expect("valid"), before.y.expect("valid"))?,
                    Point::new(row.x.expect("valid"), row.y.expect("valid"))?,
                ]);
                include_geometry(&mut prepared.domains, &geometry);
                Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                    geometry,
                    style,
                    group: group.clone(),
                    targets: vec![before.target, row.target.clone()],
                    aesthetics: before.values,
                });
            }
            previous = Some(row);
        }
        return Ok(());
    }
    let mut points = vec![];
    let mut upper = vec![];
    let mut targets = vec![];
    for row in rows.drain(..) {
        if let (Some(x), Some(y)) = (row.x, row.y) {
            charge(
                vertices,
                if matches!(geom, Geom::Line { .. } | Geom::ShapeLine { .. }) {
                    1
                } else {
                    2
                },
                "vertex",
            )?;
            points.push(Point::new(x, y)?);
            if !matches!(geom, Geom::Line { .. } | Geom::ShapeLine { .. }) {
                upper.push(Point::new(
                    if matches!(geom, Geom::ShapeArea { .. }) {
                        row.x2.expect("validated paired coordinate")
                    } else {
                        x
                    },
                    row.y2.expect("validated boundary"),
                )?);
            }
            targets.push(row.target);
        } else if !connect {
            push_run(
                prepared,
                &mut points,
                &mut upper,
                &mut targets,
                group,
                style,
            );
        }
    }
    push_run(
        prepared,
        &mut points,
        &mut upper,
        &mut targets,
        group,
        style,
    );
    Ok(())
}
fn push_run(
    prepared: &mut PreparedLayer,
    points: &mut Vec<Point>,
    upper: &mut Vec<Point>,
    targets: &mut Vec<Target>,
    group: &GroupValue,
    style: Style,
) {
    if !points.is_empty() {
        let geometry = if upper.is_empty() {
            PreparedGeometry::LineRun(std::mem::take(points))
        } else {
            PreparedGeometry::BandRun {
                lower: std::mem::take(points),
                upper: std::mem::take(upper),
            }
        };
        include_geometry(&mut prepared.domains, &geometry);
        Arc::make_mut(&mut prepared.marks).push(PreparedMark {
            aesthetics: BTreeMap::new(),
            geometry,
            targets: std::mem::take(targets),
            group: group.clone(),
            style,
        });
    }
}
