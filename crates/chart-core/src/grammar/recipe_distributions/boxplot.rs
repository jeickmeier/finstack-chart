use super::common::error;
use super::{BoxplotRecipe, PreparedDistribution, common::*};
use crate::{
    ChartResult,
    grammar::{compiler::EncodedRow, *},
    provenance::Target,
};
fn project_source(input: &Numeric, value: f64) -> Option<f64> {
    match input {
        Numeric::Scaled { input, scale, .. } => {
            scale.project_optional(project_source(input, value))
        }
        _ => Some(value),
    }
}
pub(super) fn setup(
    layer: &Layer,
    s: &BoxplotRecipe,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
) -> ChartResult<()> {
    let count = s
        .source_outliers
        .iter()
        .try_fold(0usize, |n, v| n.checked_add(v.values.len() + 1))
        .ok_or_else(|| error("Source outlier list exceeds budget."))?;
    crate::limits::require_within(
        count <= limits.max_prepared_rows,
        "source box outlier values",
    )?;
    if !s.source_outliers.is_empty() && !matches!(layer.mappings, Mappings::Source(_)) {
        return Err(error(
            "Source outlier descriptors cannot be mixed with generated boxplot rows.",
        ));
    }
    let width = s
        .width
        .unwrap_or(0.9 * super::super::recipe_emit::resolution(rows.iter().filter_map(|r| r.x)));
    let max_relative = rows
        .iter()
        .filter_map(|r| number(r, RecipeAesthetic::RelativeWidth))
        .reduce(f64::max)
        .unwrap_or(1.);
    for row in rows {
        if let Some(middle) = number(row, RecipeAesthetic::Middle) {
            row.y = Some(middle);
        }
        row.outlier_anchor_y = row.y;
        if let Target::Source(source) = &row.target
            && let Some(values) = s.source_outliers.iter().find(|v| v.row == source.key)
        {
            let input = match &layer.mappings {
                Mappings::Source(a) => {
                    if layer.orientation == Orientation::Horizontal {
                        a.x.as_ref()
                    } else {
                        a.y.as_ref()
                    }
                }
                _ => None,
            };
            for value in &values.values {
                let value = input.map_or(Some(*value), |input| project_source(input, *value));
                if let Some(value) = value {
                    row.stat_outliers.push(StatOutlier {
                        value,
                        target: row.target.clone(),
                    });
                }
            }
        }
        if let (Some(x), Some(mut width)) = (row.x, number_or(row, RecipeAesthetic::Width, width)) {
            if s.variable_width {
                width *=
                    number_or(row, RecipeAesthetic::RelativeWidth, 1.).unwrap_or(0.) / max_relative;
            }
            if !width.is_finite() || width < 0. {
                return Err(error("Boxplot width must be finite and nonnegative."));
            }
            row.x = Some(x - width / 2.);
            row.x2 = Some(x + width / 2.);
        }
    }
    Ok(())
}
pub(super) fn emit(
    layer: &Layer,
    s: &BoxplotRecipe,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<()> {
    let mut groups = std::collections::BTreeSet::new();
    for row in rows {
        if !groups.insert(row.group.clone().unwrap_or(GroupValue::All)) {
            return Err(error(
                "Boxplot requires one summary row per group; map a source grouping field.",
            ));
        }
    }
    for row in rows {
        if number_or(row, RecipeAesthetic::Width, 0.).is_none() {
            super::super::recipe_emit::retain_positions(prepared, row);
            continue;
        }
        let (Some((a, b)), Some(lo), Some(hi), Some(middle)) =
            (width(row), row.low, row.high, row.y)
        else {
            super::super::recipe_emit::retain_positions(prepared, row);
            continue;
        };
        let x = a.midpoint(b);
        let shift = middle - number(row, RecipeAesthetic::Middle).unwrap_or(middle);
        let (Some(whisker_lo), Some(whisker_hi)) = (
            number(row, RecipeAesthetic::WhiskerLower),
            number(row, RecipeAesthetic::WhiskerUpper),
        ) else {
            continue;
        };
        let whisker_lo = whisker_lo + shift;
        let whisker_hi = whisker_hi + shift;
        if s.outliers {
            let mut paint = style(layer, row, &IntervalStroke::default(), false)?;
            if let Some(color) = s.outlier_color {
                paint.color = color.resolve();
                paint.stroke = Some(paint.color);
            }
            if let Some(fill) = s.outlier.fill {
                paint.fill = Some(fill.resolve());
            }
            if let Some(alpha) = s.outlier_alpha {
                paint.alpha = Some(alpha);
            }
            let size = s.outlier.size.or(row.size).unwrap_or(1.5);
            let stroke = s.outlier.stroke.unwrap_or(0.5);
            let shape = s
                .outlier
                .shape
                .or_else(|| match row.values.get(&ValueAesthetic::Shape) {
                    Some(crate::interpolate::Value::Number(n)) => Some(n.0 as u8),
                    _ => None,
                })
                .unwrap_or(19);
            for outlier in &row.stat_outliers {
                let y = outlier.value + row.y.unwrap_or(0.) - row.outlier_anchor_y.unwrap_or(0.);
                push(
                    prepared,
                    row,
                    PreparedGeometry::Recipe(Box::new(PreparedRecipe::Distribution(
                        PreparedDistribution::Outlier {
                            center: point(x, y)?,
                            size,
                            stroke,
                            shape,
                        },
                    ))),
                    paint,
                    vec![outlier.target.clone()],
                    vertices,
                )?;
            }
        }
        if s.staple_width > 0. {
            for y in [whisker_lo, whisker_hi] {
                push(
                    prepared,
                    row,
                    line(
                        x + (a - x) * s.staple_width,
                        y,
                        x + (b - x) * s.staple_width,
                        y,
                    )?,
                    style(layer, row, &s.staple, true)?,
                    vec![row.target.clone()],
                    vertices,
                )?;
            }
        }
        for (from, to) in [(lo, whisker_lo), (hi, whisker_hi)] {
            push(
                prepared,
                row,
                line(x, from, x, to)?,
                style(layer, row, &s.whisker, true)?,
                vec![row.target.clone()],
                vertices,
            )?;
        }
        let points = if s.notch {
            let (Some(nl), Some(nu)) = (
                number(row, RecipeAesthetic::NotchLower),
                number(row, RecipeAesthetic::NotchUpper),
            ) else {
                continue;
            };
            let left = x + (a - x) * s.notch_width;
            let right = x + (b - x) * s.notch_width;
            [
                (a, hi),
                (a, nu + shift),
                (left, middle),
                (a, nl + shift),
                (a, lo),
                (b, lo),
                (b, nl + shift),
                (right, middle),
                (b, nu + shift),
                (b, hi),
            ]
            .to_vec()
        } else {
            vec![(a, lo), (a, hi), (b, hi), (b, lo)]
        };
        let contour = points
            .into_iter()
            .map(|(x, y)| point(x, y))
            .collect::<ChartResult<Vec<_>>>()?;
        let surface = PreparedSurface::Polygon {
            contours: vec![contour],
            anchors: vec![point(x, middle)?],
            rule: crate::scene::FillRule::NonZero,
        };
        let mut box_style = style(layer, row, &s.box_style, true)?;
        box_style.line_join = box_style.line_join.or(Some(LineJoin::Miter));
        box_style.fill = crate::grammar::compiler::row_style(layer, row)?.fill;
        box_style.fill = box_style.fill.or(Some(crate::scene::Color {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        }));
        push(
            prepared,
            row,
            PreparedGeometry::Recipe(Box::new(PreparedRecipe::Surface(surface))),
            box_style,
            vec![row.target.clone()],
            vertices,
        )?;
        let mut median_style = style(layer, row, &s.median, true)?;
        if s.median.linewidth.is_none() {
            median_style.stroke_width *= s.fatten;
        }
        let fraction = if s.notch { s.notch_width } else { 1. };
        push(
            prepared,
            row,
            line(
                x + (a - x) * fraction,
                middle,
                x + (b - x) * fraction,
                middle,
            )?,
            median_style,
            vec![row.target.clone()],
            vertices,
        )?;
    }
    Ok(())
}
