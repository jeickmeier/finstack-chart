use super::common::error;
use super::{DotAxis, DotStack, DotplotRecipe, PreparedDistribution, common::*};
use crate::{
    ChartResult,
    grammar::{compiler::EncodedRow, *},
};
use std::collections::BTreeMap;
fn span(stack: DotStack) -> (f64, f64) {
    match stack {
        DotStack::Up => (0., 1.),
        DotStack::Down => (-1., 0.),
        DotStack::Center | DotStack::CenterWhole => (-0.5, 0.5),
    }
}
pub(super) fn setup(s: &DotplotRecipe, rows: &mut [EncodedRow]) -> ChartResult<()> {
    let default = s
        .width
        .unwrap_or(0.9 * super::super::recipe_emit::resolution(rows.iter().filter_map(|r| r.x)));
    let (min, max) = span(s.stack);
    for row in rows {
        let Some(bw) = number(row, RecipeAesthetic::BinWidth) else {
            continue;
        };
        if !bw.is_finite() {
            return Err(error("Dotplot bin width must be finite."));
        }
        match s.bin_axis {
            DotAxis::X => {
                if let Some(x) = row.x {
                    row.x = Some(x - bw / 2.);
                    row.x2 = Some(x + bw / 2.);
                }
                let baseline = row.y.unwrap_or(0.);
                row.y = Some(baseline + min);
                row.y2 = Some(baseline + max);
            }
            DotAxis::Y => {
                if let (Some(x), Some(y), Some(width)) = (
                    row.x,
                    row.y,
                    number_or(row, RecipeAesthetic::Width, default),
                ) {
                    row.x = Some(x + width * min);
                    row.x2 = Some(x + width * max);
                    row.y = Some(y - bw / 2.);
                    row.y2 = Some(y + bw / 2.);
                }
            }
        }
    }
    Ok(())
}
pub(super) fn emit(
    layer: &Layer,
    s: &DotplotRecipe,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<()> {
    let mut groups: BTreeMap<(u64, u64, GroupValue), Vec<&EncodedRow>> = BTreeMap::new();
    let (min, _) = span(s.stack);
    for row in rows {
        super::super::recipe_emit::retain_positions(prepared, row);
        let (Some(a), Some(b), Some(y), Some(y2)) = (row.x, row.x2, row.y, row.y2) else {
            continue;
        };
        let (bin, baseline) = match s.bin_axis {
            DotAxis::X => (a.midpoint(b), y - min),
            DotAxis::Y => (y.midpoint(y2), a - (b - a) * min),
        };
        let group = if s.stack_groups {
            GroupValue::All
        } else {
            row.group.clone().unwrap_or(GroupValue::All)
        };
        groups
            .entry((bin.to_bits(), baseline.to_bits(), group))
            .or_default()
            .push(row);
    }
    for group in groups.values() {
        let mut total = 0usize;
        for row in group {
            let Some(n) = number(row, RecipeAesthetic::Count) else {
                continue;
            };
            if !n.is_finite() || n < 0. || n.fract() != 0. {
                return Err(error("Dotplot count must be a nonnegative integer."));
            }
            crate::limits::require_within(n <= *vertices as f64, "dotplot circle count")?;
            total = total
                .checked_add(n as usize)
                .ok_or_else(|| error("Dotplot count overflow."))?;
        }
        let mut index = 0usize;
        for row in group {
            let n = number(row, RecipeAesthetic::Count).unwrap_or(0.) as usize;
            let (a, b, y, y2) = (
                row.x.unwrap(),
                row.x2.unwrap(),
                row.y.unwrap(),
                row.y2.unwrap(),
            );
            let bw = number(row, RecipeAesthetic::BinWidth).unwrap();
            let (center, bin, horizontal) = match s.bin_axis {
                DotAxis::X => {
                    let base = y - min;
                    (
                        point(a.midpoint(b), base)?,
                        [
                            point(a.midpoint(b) - bw / 2., base)?,
                            point(a.midpoint(b) + bw / 2., base)?,
                        ],
                        false,
                    )
                }
                DotAxis::Y => {
                    let base = a - (b - a) * min;
                    (
                        point(base, y.midpoint(y2))?,
                        [
                            point(base, y.midpoint(y2) - bw / 2.)?,
                            point(base, y.midpoint(y2) + bw / 2.)?,
                        ],
                        true,
                    )
                }
            };
            let mut paint = style(layer, row, &IntervalStroke::default(), false)?;
            // dotstack uses the raw stroke aesthetic in device line units, unlike path linewidth.
            let authored = crate::grammar::compiler::row_style(layer, row)?;
            paint.stroke_width = authored.stroke_width
                * authored
                    .units
                    .map_or(72. / 96., |u| u.factor(crate::services::Units::Points));
            paint.units = Some(AestheticUnits::Points);
            for _ in 0..n {
                let stack_position = number(row, RecipeAesthetic::StackPosition).unwrap_or(match s
                    .stack
                {
                    DotStack::Up => index as f64 + 0.5,
                    DotStack::Down => -(index as f64) - 0.5,
                    DotStack::Center => index as f64 - (total.saturating_sub(1)) as f64 / 2.,
                    DotStack::CenterWhole => index as f64 - (total.saturating_sub(1) / 2) as f64,
                });
                let stack_offset = match s.stack {
                    DotStack::Up => (1. - s.stack_ratio) / 2.,
                    DotStack::Down => -(1. - s.stack_ratio) / 2.,
                    _ => 0.,
                };
                push(
                    prepared,
                    row,
                    PreparedGeometry::Recipe(Box::new(PreparedRecipe::Distribution(
                        PreparedDistribution::Dot {
                            center,
                            bin,
                            stack_position,
                            stack_ratio: s.stack_ratio,
                            stack_offset,
                            dot_size: s.dot_size,
                            horizontal,
                        },
                    ))),
                    paint,
                    vec![row.target.clone()],
                    vertices,
                )?;
                index += 1;
            }
        }
    }
    Ok(())
}
