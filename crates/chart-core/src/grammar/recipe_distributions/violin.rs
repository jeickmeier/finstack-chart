use super::common::error;
use super::{ViolinRecipe, common::*};
use crate::{
    ChartResult,
    grammar::{compiler::EncodedRow, *},
};
use std::collections::BTreeMap;
pub(super) fn setup(s: &ViolinRecipe, rows: &mut [EncodedRow]) -> ChartResult<()> {
    let default = s
        .width
        .unwrap_or(0.9 * super::super::recipe_emit::resolution(rows.iter().filter_map(|r| r.x)));
    for row in rows {
        if let (Some(x), Some(width)) = (row.x, number_or(row, RecipeAesthetic::Width, default)) {
            if !width.is_finite() || width < 0. {
                return Err(error("Violin width must be finite and nonnegative."));
            }
            row.x = Some(x - width / 2.);
            row.x2 = Some(x + width / 2.);
        }
    }
    Ok(())
}
pub(super) fn emit(
    layer: &Layer,
    s: &ViolinRecipe,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<()> {
    let mut groups: BTreeMap<GroupValue, Vec<&EncodedRow>> = BTreeMap::new();
    for row in rows {
        super::super::recipe_emit::retain_positions(prepared, row);
        groups
            .entry(row.group.clone().unwrap_or(GroupValue::All))
            .or_default()
            .push(row);
    }
    for rows in groups.values_mut() {
        rows.sort_by(|a, b| a.y.unwrap_or(0.).total_cmp(&b.y.unwrap_or(0.)));
        let mut left = vec![];
        let mut right = vec![];
        let mut anchors = vec![];
        let mut targets = vec![];
        let mut quantiles = vec![];
        for row in rows.iter() {
            if number_or(row, RecipeAesthetic::Width, 0.).is_none() {
                super::super::recipe_emit::retain_positions(prepared, row);
                continue;
            }
            let (Some((a, b)), Some(y), Some(w)) =
                (width(row), row.y, number(row, RecipeAesthetic::ViolinWidth))
            else {
                continue;
            };
            if !w.is_finite() || w < 0. {
                continue;
            }
            let x = a.midpoint(b);
            let lo = x + (a - x) * w;
            let hi = x + (b - x) * w;
            left.push(point(lo, y)?);
            right.push(point(hi, y)?);
            anchors.push(point(x, y)?);
            targets.push(row.target.clone());
            if number(row, RecipeAesthetic::QuantileFlag).is_some() {
                quantiles.push((*row, lo, hi, y));
            }
        }
        if left.len() < 2 {
            continue;
        }
        right.reverse();
        left.extend(right);
        let row = rows[0];
        let mut paint = style(layer, row, &IntervalStroke::default(), true)?;
        paint.fill = crate::grammar::compiler::row_style(layer, row)?.fill;
        paint.line_join = paint.line_join.or(Some(LineJoin::Round));
        paint.fill = paint.fill.or(Some(crate::scene::Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        }));
        push(
            prepared,
            row,
            PreparedGeometry::Recipe(Box::new(PreparedRecipe::Surface(
                PreparedSurface::Polygon {
                    contours: vec![left],
                    anchors,
                    rule: crate::scene::FillRule::NonZero,
                },
            ))),
            paint,
            targets,
            vertices,
        )?;
        if s.quantile.line_type != Some(LineType::Blank) {
            for (row, a, b, y) in quantiles {
                push(
                    prepared,
                    row,
                    line(a, y, b, y)?,
                    style(layer, row, &s.quantile, false)?,
                    vec![row.target.clone()],
                    vertices,
                )?;
            }
        }
    }
    Ok(())
}
