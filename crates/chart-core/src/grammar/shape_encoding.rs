//! Shape parameters share stage-aware numeric readers/scales with all other aesthetics.
use super::{compiler::EncodedRow, *};
use crate::{
    ChartResult, DiagnosticCode, Point,
    path::{PathGeometry, PathLimits},
    shape::{
        Arc as ArcGenerator, ArcParameters, Pie, ShapeLimits, Symbol, SymbolKind, SymbolPaint,
    },
};
use std::collections::BTreeMap;
#[derive(Clone, Copy)]
pub(super) struct ShapeRow {
    parameters: Option<ArcParameters>,
    symbol: Option<(SymbolKind, f64)>,
    value: Option<f64>,
    radial: Option<RadialParameters>,
}
fn defaults(geom: Geom) -> Option<ArcParameters> {
    match geom {
        Geom::ShapeArc { parameters } | Geom::ShapePie { parameters, .. } => Some(parameters),
        _ => None,
    }
}
pub(super) fn validate_channel(geom: Geom, channel: NumericAesthetic) -> ChartResult<()> {
    use NumericAesthetic as A;
    let radial = super::radial_shapes::defaults(geom).is_some();
    let valid = if radial && !matches!(channel, A::Size | A::Opacity | A::StrokeWidth) {
        matches!(channel, A::Angle | A::Radius)
            || (!matches!(geom, Geom::ShapeLineRadial { .. })
                && matches!(
                    channel,
                    A::StartAngle | A::EndAngle | A::InnerRadius | A::OuterRadius
                ))
    } else {
        match channel {
            A::Size | A::Opacity | A::StrokeWidth => true,
            A::AreaSize => matches!(geom, Geom::ShapeSymbol { .. }),
            A::PieValue => matches!(geom, Geom::ShapePie { .. }),
            A::StartAngle | A::EndAngle | A::PadAngle => matches!(geom, Geom::ShapeArc { .. }),
            _ => defaults(geom).is_some() && !matches!(channel, A::Angle | A::Radius),
        }
    };
    if valid {
        Ok(())
    } else {
        Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Shape parameter is incompatible with this geometry; pie sweep/padding belong to its layout configuration.",
        ))
    }
}
pub(super) fn validate(layer: &Layer) -> ChartResult<()> {
    if let Some(parameters) = super::radial_shapes::defaults(layer.geom) {
        parameters.validate()?;
        if layer.geometry_extension.is_some() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Radial shapes cannot also invoke a geometry replacement.",
            ));
        }
        for (shared, first, second) in [
            (
                NumericAesthetic::Angle,
                NumericAesthetic::StartAngle,
                NumericAesthetic::EndAngle,
            ),
            (
                NumericAesthetic::Radius,
                NumericAesthetic::InnerRadius,
                NumericAesthetic::OuterRadius,
            ),
        ] {
            if layer.numeric_scales.contains_key(&shared)
                && (layer.numeric_scales.contains_key(&first)
                    || layer.numeric_scales.contains_key(&second))
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Shared radial channels cannot coexist with independent channels for the same dimension.",
                ));
            }
        }
    }
    if let Some(parameters) = defaults(layer.geom) {
        // Validate configuration even when the source population is empty.
        ArcGenerator::new().generate_by(&parameters, |p| Ok(*p))?;
        if layer.geometry_extension.is_some() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Shape geometry cannot also invoke a registered geometry replacement.",
            ));
        }
    }
    if let Geom::ShapePie { angles, order, .. } = layer.geom {
        Pie::new()
            .order(order)
            .start_angle(angles.start_angle)
            .end_angle(angles.end_angle)
            .pad_angle(angles.pad_angle)
            .validate()?;
        if !layer
            .numeric_scales
            .contains_key(&NumericAesthetic::PieValue)
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Pie geometry requires a named PieValue mapping.",
            ));
        }
    }
    for channel in layer.numeric_scales.keys() {
        validate_channel(layer.geom, *channel)?;
    }
    Ok(())
}
pub(super) fn set(
    geom: Geom,
    row: &mut EncodedRow,
    channel: NumericAesthetic,
    value: f64,
) -> ChartResult<bool> {
    use NumericAesthetic as A;
    if matches!(channel, A::Size | A::Opacity | A::StrokeWidth) {
        return Ok(false);
    }
    validate_channel(geom, channel)?;
    let shape = row.shape.get_or_insert_with(|| {
        Box::new(ShapeRow {
            parameters: defaults(geom),
            symbol: match geom {
                Geom::ShapeSymbol { kind, size, .. } => Some((kind, size)),
                _ => None,
            },
            value: None,
            radial: super::radial_shapes::defaults(geom),
        })
    });
    if let Some(parameters) = &mut shape.radial {
        super::radial_shapes::set(parameters, channel, value);
        return Ok(true);
    }
    if channel == A::AreaSize {
        Symbol::new().size(value).validate()?;
        shape.symbol.as_mut().expect("symbol channel").1 = value;
        return Ok(true);
    }
    let parameters = shape.parameters.as_mut().expect("arc channel");
    match channel {
        A::InnerRadius => parameters.datum.inner_radius = value,
        A::OuterRadius => parameters.datum.outer_radius = value,
        A::StartAngle => parameters.datum.start_angle = value,
        A::EndAngle => parameters.datum.end_angle = value,
        A::PadAngle => parameters.datum.pad_angle = value,
        A::PadRadius => parameters.pad_radius = Some(value),
        A::CornerRadius => parameters.corner_radius = value,
        A::PieValue => shape.value = Some(value),
        _ => unreachable!("style channels returned above"),
    }
    Ok(true)
}
pub(super) fn allocate(
    layer: &Layer,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
    protocols: &super::shape_extensions::ResolvedShapes,
) -> ChartResult<()> {
    let Geom::ShapePie {
        angles,
        order,
        grouped,
        ..
    } = layer.geom
    else {
        return Ok(());
    };
    let mut groups: BTreeMap<GroupValue, Vec<usize>> = BTreeMap::new();
    for (i, row) in rows.iter_mut().enumerate() {
        if row.shape.as_ref().and_then(|s| s.value).is_none() {
            row.x = None;
            row.y = None;
        }
        if row.x.is_some()
            && row.y.is_some()
            && let Some(group) = &row.group
        {
            groups
                .entry(if grouped {
                    group.clone()
                } else {
                    GroupValue::All
                })
                .or_default()
                .push(i);
        }
    }
    let pie = Pie::new()
        .order(order)
        .start_angle(angles.start_angle)
        .end_angle(angles.end_angle)
        .pad_angle(angles.pad_angle)
        .limits(ShapeLimits {
            max_points: limits.max_prepared_rows,
            ..Default::default()
        });
    for indices in groups.values() {
        let value = |i: &usize, _: usize, _: &[usize]| {
            Ok(rows[*i]
                .shape
                .as_ref()
                .and_then(|s| s.value)
                .expect("eligible weight"))
        };
        let slices = if let Some(compare) = protocols.pie_comparator() {
            // Comparison sees post-statistic identity and encoded position metadata;
            // curve controls never replace these source/derived observations.
            let metadata: BTreeMap<_, _> = indices
                .iter()
                .map(|i| {
                    (
                        *i,
                        serde_json::json!({
                            "key": rows[*i].key.map(|v| v.get().to_string()),
                            "ordinal": rows[*i].ordinal.to_string(),
                            "target": rows[*i].target,
                            "group": rows[*i].group,
                            "x": rows[*i].x, "y": rows[*i].y,
                        }),
                    )
                })
                .collect();
            pie.layout_by_fallible_comparator(indices, value, |i, vi, j, vj| {
                compare.compare(&metadata[i], vi, &metadata[j], vj)
            })?
        } else {
            pie.layout_by(indices, value)?
        };
        for slice in slices {
            let shape = rows[slice.data].shape.as_mut().expect("eligible shape");
            let parameters = shape.parameters.as_mut().expect("pie parameters");
            parameters.datum.start_angle = slice.start_angle;
            parameters.datum.end_angle = slice.end_angle;
            parameters.datum.pad_angle = slice.pad_angle;
        }
    }
    Ok(())
}
pub(super) fn geometry(
    geom: Geom,
    row: &EncodedRow,
    center: Point,
    limits: CompileLimits,
    custom_symbol: Option<&(dyn crate::shape::SymbolDraw + Send + Sync)>,
) -> ChartResult<PreparedGeometry> {
    if let Geom::ShapeSymbol { kind, size, paint } = geom {
        let (kind, size) = row
            .shape
            .as_deref()
            .and_then(|s| s.symbol)
            .unwrap_or((kind, size));
        let geometry = if size == 0. {
            PathGeometry::from_commands(vec![], 0)?
        } else {
            let generator = Symbol::new().kind(kind).size(size).limits(ShapeLimits {
                max_points: 1,
                path: PathLimits {
                    max_commands: limits.max_vertices,
                    ..Default::default()
                },
            });
            match custom_symbol {
                Some(symbol) => generator.generate_with(symbol, size)?,
                None => generator.generate()?,
            }
            .geometry()
        };
        return Ok(PreparedGeometry::ShapePath {
            center,
            geometry,
            anchor: Point::new(0., 0.)?,
            paint: paint.resolve(kind)?,
        });
    }
    let parameters = row.shape.as_deref().map_or_else(
        || defaults(geom).expect("shape"),
        |s| s.parameters.expect("arc parameters"),
    );
    let generator = ArcGenerator::new().limits(ShapeLimits {
        max_points: 1,
        path: PathLimits {
            max_commands: limits.max_vertices,
            ..Default::default()
        },
    });
    let path: PathGeometry = generator.generate_by(&parameters, |p| Ok(*p))?.geometry();
    let anchor = generator.centroid_by(&parameters, |p| Ok(*p))?;
    Ok(PreparedGeometry::ShapePath {
        paint: SymbolPaint::Fill,
        center,
        geometry: path,
        anchor: Point::new(anchor[0], anchor[1])?,
    })
}

pub(super) fn set_symbol(geom: Geom, row: &mut EncodedRow, kind: SymbolKind) {
    let Geom::ShapeSymbol { size, .. } = geom else {
        unreachable!("validated symbol mapping")
    };
    let shape = row.shape.get_or_insert_with(|| {
        Box::new(ShapeRow {
            parameters: None,
            symbol: Some((kind, size)),
            value: None,
            radial: super::radial_shapes::defaults(geom),
        })
    });
    shape.symbol.as_mut().expect("symbol mapping").0 = kind;
}

pub(super) fn radial_parameters(geom: Geom, row: &EncodedRow) -> RadialParameters {
    row.shape
        .as_deref()
        .and_then(|s| s.radial)
        .unwrap_or_else(|| super::radial_shapes::defaults(geom).expect("radial geometry"))
}
