//! Interval and reference recipes reuse encoded coordinates and common emission.
use super::{compiler::EncodedRow, *};
use crate::{ChartResult, DiagnosticCode, Point};
pub(super) fn validate(recipe: &BuiltinRecipe) -> ChartResult<()> {
    if let BuiltinRecipe::Interval(s) = recipe
        && ([
            s.fatten,
            s.middle.linewidth,
            s.box_style.linewidth,
            s.point.size,
            s.point.stroke,
        ]
        .into_iter()
        .flatten()
        .any(|v| !v.is_finite() || v < 0.)
            || s.point.shape.is_some_and(|v| v > 25))
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Interval dimensions must be finite and nonnegative; point shape must be zero through 25.",
        ));
    }
    let arrow = match recipe {
        BuiltinRecipe::Reference(s) => s.arrow.as_ref(),
        BuiltinRecipe::Segment { arrow } => arrow.as_ref(),
        BuiltinRecipe::Curve(s) => s.arrow.as_ref(),
        BuiltinRecipe::Spoke(s) => s.arrow.as_ref(),
        _ => None,
    };
    if arrow.is_some_and(|a| {
        !a.angle.is_finite()
            || a.angle <= 0.
            || a.angle >= 180.
            || !a.length_mm.is_finite()
            || a.length_mm < 0.
    }) {
        return Err(error(
            DiagnosticCode::Validation,
            "Arrow angle and length are invalid.",
        ));
    }
    if matches!(recipe,BuiltinRecipe::Interval(s) if s.width.is_some_and(|w|!w.is_finite()||w<0.))
        || matches!(recipe,BuiltinRecipe::Reference(s)if !s.slope.is_finite()||!s.intercept.is_finite())
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Recipe parameters must be finite with nonnegative widths.",
        ));
    }
    Ok(())
}
pub(super) fn setup(layer: &Layer, rows: &mut [EncodedRow], _: CompileLimits) -> ChartResult<()> {
    if let Some(BuiltinRecipe::Interval(s)) = &layer.recipe {
        let width = s
            .width
            .unwrap_or(0.9 * super::recipe_emit::resolution(rows.iter().filter_map(|r| r.x)));
        for row in rows {
            if let Some(x) = row.x {
                let Some(width) = super::recipe_emit::number_or(row, RecipeAesthetic::Width, width)
                else {
                    continue;
                };
                if !width.is_finite() || width < 0. {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Interval width must be finite and nonnegative.",
                    ));
                }
                row.x = Some(x - width / 2.);
                row.x2 = Some(x + width / 2.);
            }
        }
    }
    Ok(())
}
pub(super) fn emit(
    layer: &Layer,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<bool> {
    match &layer.recipe {
        Some(BuiltinRecipe::Reference(s)) => {
            prepared.domains.x = None;
            prepared.domains.y = None;
            for row in rows {
                let mut s = s.clone();
                let (Some(slope), Some(intercept)) = (
                    super::recipe_emit::number_or(row, RecipeAesthetic::Slope, s.slope),
                    super::recipe_emit::number_or(row, RecipeAesthetic::Intercept, s.intercept),
                ) else {
                    continue;
                };
                s.slope = slope;
                s.intercept = intercept;
                super::recipe_emit::push(
                    prepared,
                    row,
                    layer,
                    PreparedGeometry::Recipe(Box::new(PreparedRecipe::Reference(s))),
                    vertices,
                )?;
            }
            let marks = std::sync::Arc::make_mut(&mut prepared.marks);
            let mut unique: Vec<PreparedMark> = Vec::new();
            let mut keys = std::collections::BTreeMap::new();
            for mark in marks.drain(..) {
                let PreparedGeometry::Recipe(recipe) = &mark.geometry else {
                    unreachable!()
                };
                let PreparedRecipe::Reference(reference) = recipe.as_ref() else {
                    unreachable!()
                };
                let key =
                    serde_json::to_string(&(reference, &mark.style, &mark.group, &mark.aesthetics))
                        .map_err(|_| {
                            error(
                                DiagnosticCode::Validation,
                                "Reference paint identity cannot be encoded.",
                            )
                        })?;
                if let Some(&index) = keys.get(&key) {
                    let prior: &mut PreparedMark = &mut unique[index];
                    prior.targets.extend(mark.targets);
                } else {
                    keys.insert(key, unique.len());
                    unique.push(mark);
                }
            }
            *marks = unique;
            Ok(true)
        }
        Some(BuiltinRecipe::Interval(s)) => {
            for row in rows {
                if super::recipe_emit::number_or(row, RecipeAesthetic::Width, 0.).is_none() {
                    super::recipe_emit::retain_positions(prepared, row);
                    continue;
                }

                let (Some(a), Some(b), Some(lo), Some(hi)) = (row.x, row.x2, row.low, row.high)
                else {
                    super::recipe_emit::retain_positions(prepared, row);
                    continue;
                };
                let x = a.midpoint(b);
                let rule = |x1, y1, x2, y2| -> ChartResult<PreparedGeometry> {
                    Ok(PreparedGeometry::Rule {
                        from: Point::new(x1, y1)?,
                        to: Point::new(x2, y2)?,
                    })
                };
                let mut components = match s.kind {
                    IntervalKind::Crossbar => vec![PreparedGeometry::Rectangle {
                        from: Point::new(a, lo)?,
                        to: Point::new(b, hi)?,
                    }],
                    _ => vec![rule(x, lo, x, hi)?],
                };
                match s.kind {
                    IntervalKind::ErrorBar => {
                        components.push(rule(a, lo, b, lo)?);
                        components.push(rule(a, hi, b, hi)?);
                    }
                    IntervalKind::PointRange => {
                        if let Some(y) = row.y {
                            components.push(PreparedGeometry::Point(Point::new(x, y)?));
                        }
                    }
                    IntervalKind::Crossbar => {
                        if let Some(y) = row.y {
                            components.push(rule(a, y, b, y)?);
                        }
                    }
                    IntervalKind::LineRange => {}
                }
                for geometry in components {
                    super::recipe_emit::push(prepared, row, layer, geometry, vertices)?;
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
