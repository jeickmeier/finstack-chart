//! Confidence ribbons reuse common aligned bands; the common line emitter adds the mean.
use super::{compiler::EncodedRow, *};
use crate::{ChartResult, DiagnosticCode, Point};
use std::{collections::BTreeMap, sync::Arc};
pub(super) fn emit(
    layer: &Layer,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<()> {
    let mut groups: BTreeMap<GroupValue, Vec<&EncodedRow>> = BTreeMap::new();
    for row in rows {
        if let Some(group) = &row.group {
            groups.entry(group.clone()).or_default().push(row);
        }
    }
    for (group, mut rows) in groups {
        rows.sort_by(|a, b| {
            a.x.partial_cmp(&b.x)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.ordinal.cmp(&b.ordinal))
        });
        let mut run = vec![];
        for row in rows.into_iter().map(Some).chain(std::iter::once(None)) {
            if let Some(row) = row.filter(|r| r.x.is_some() && r.low.is_some() && r.high.is_some())
            {
                run.push(row);
                continue;
            }
            if run.is_empty() {
                continue;
            }
            let mut lower = Vec::with_capacity(run.len());
            let mut upper = Vec::with_capacity(run.len());
            for row in &run {
                lower.push(Point::new(row.x.unwrap(), row.low.unwrap())?);
                upper.push(Point::new(row.x.unwrap(), row.high.unwrap())?);
            }
            *vertices = vertices.checked_sub(run.len() * 2).ok_or_else(|| {
                error(
                    DiagnosticCode::ResourceLimit,
                    "Model ribbon vertex budget exceeded.",
                )
            })?;
            let geometry = PreparedGeometry::BandRun { lower, upper };
            compiler::include_geometry(&mut prepared.domains, &geometry);
            let mut style = compiler::row_style(layer, run[0])?;
            let fill = layer.style.fill.or(run[0].fill).unwrap_or_else(|| {
                crate::color::Paint::from(crate::scene::Color {
                    red: 153,
                    green: 153,
                    blue: 153,
                    alpha: 255,
                })
            });
            style.fill = Some(numeric_aesthetics::apply_alpha(
                numeric_aesthetics::apply_opacity(fill, run[0].opacity),
                layer.style.alpha.or(run[0].alpha).or(Some(0.4)),
            ));
            style.stroke = Some(crate::scene::Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            });
            style.stroke_width = 0.;
            style.alpha = None;
            Arc::make_mut(&mut prepared.marks).push(PreparedMark {
                geometry,
                style,
                group: group.clone(),
                targets: run.iter().map(|r| r.target.clone()).collect(),
                aesthetics: run[0].values.clone(),
            });
            run.clear();
        }
    }
    Ok(())
}

pub(super) fn apply_defaults(
    layer: &mut Layer,
    theme: &crate::theme::GeometryTheme<crate::color::Paint>,
) {
    if !matches!(layer.recipe, Some(BuiltinRecipe::Smooth)) {
        return;
    }
    if let Some(grammar) = &layer.grammar {
        if grammar.default_color {
            layer.style.color = theme.accent;
        }
        if grammar.default_line_width.unwrap_or(grammar.default_size) {
            layer.style.stroke_width = theme.line_width * 2.;
        }
        if layer.style.fill.is_none() && !layer.paint_scales.contains_key(&PaintAesthetic::Fill) {
            let ink = theme.ink.resolve();
            let paper = theme.paper.resolve();
            let mix = |a: u8, b: u8| (f64::from(a) * 0.4 + f64::from(b) * 0.6).round() as u8;
            layer.style.fill = Some(
                crate::scene::Color {
                    red: mix(ink.red, paper.red),
                    green: mix(ink.green, paper.green),
                    blue: mix(ink.blue, paper.blue),
                    alpha: mix(ink.alpha, paper.alpha),
                }
                .into(),
            );
        }
    }
}
