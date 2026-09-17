//! Facet furniture uses the existing axis resolver and text services.
use super::*;
use crate::grammar::{
    FacetLayout, FacetPolicy, FacetStripPosition, FacetSwitch, PreparedChart, PreparedPanel,
};
use crate::services::TextMeasurer;
use crate::{ChartResult, Rect};
#[derive(Clone)]
pub(super) struct Strip {
    pub side: AxisSide,
    pub block: super::text::Block,
}
pub(super) fn strips(
    policy: &FacetPolicy,
    layout: &FacetLayout,
    panel: &PreparedPanel,
    dimensions: (usize, usize),
    r: &LayoutRequest,
    measurer: &dyn TextMeasurer,
    remaining: &mut usize,
) -> ChartResult<Vec<Strip>> {
    let (rows, columns) = dimensions;
    let mut values = vec![];
    match layout {
        FacetLayout::Wrap { .. } => values.push((
            match policy.strip_position {
                FacetStripPosition::Top => AxisSide::Top,
                FacetStripPosition::Bottom => AxisSide::Bottom,
                FacetStripPosition::Left => AxisSide::Left,
                FacetStripPosition::Right => AxisSide::Right,
            },
            0,
            policy.field_names.len(),
        )),
        FacetLayout::Grid => {
            let x = if matches!(policy.switch, FacetSwitch::X | FacetSwitch::Both) {
                AxisSide::Bottom
            } else {
                AxisSide::Top
            };
            let y = if matches!(policy.switch, FacetSwitch::Y | FacetSwitch::Both) {
                AxisSide::Left
            } else {
                AxisSide::Right
            };
            if (x == AxisSide::Top && panel.row == 0
                || x == AxisSide::Bottom && panel.row + 1 == rows)
                && policy.row_fields < policy.field_names.len()
            {
                values.push((x, policy.row_fields, policy.field_names.len()));
            }
            if (y == AxisSide::Left && panel.column == 0
                || y == AxisSide::Right && panel.column + 1 == columns)
                && policy.row_fields > 0
            {
                values.push((y, 0, policy.row_fields));
            }
        }
    }
    values
        .into_iter()
        .map(|(side, start, end)| {
            let value = if let Some(call) = &policy.labeller.registered {
                let values = panel.key.values[start..end]
                    .iter()
                    .map(|v| {
                        if *v == crate::grammar::GroupValue::Missing {
                            crate::composition::ScaleValue::MissingCategory
                        } else {
                            crate::composition::ScaleValue::Category(
                                if *v == crate::grammar::GroupValue::All {
                                    "(all)".into()
                                } else {
                                    v.label()
                                },
                            )
                        }
                    })
                    .collect::<Vec<_>>();
                panel
                    .chart
                    .guide_registrations
                    .labels(
                        &call.operation,
                        crate::grammar::GuideLabelsInput {
                            facet: Some(crate::grammar::FacetLabelContext {
                                values: &panel.key.values[start..end],
                                panel: &panel.key,
                                location: (panel.row, panel.column),
                                side,
                                grid: matches!(layout, FacetLayout::Grid),
                            }),
                            values: &values,
                            names: Some(&policy.field_names[start..end]),
                            temporal: None,
                            parameters: &call.parameters,
                            limits: crate::Limits {
                                max_text_bytes: *remaining,
                                ..r.limits
                            },
                        },
                        r.units == crate::services::Units::Points,
                    )?
                    .join(if policy.labeller.multiline {
                        "\n"
                    } else {
                        ", "
                    })
            } else {
                crate::grammar::facet_policy::label(policy, &panel.key, start, end)
            };
            crate::limits::require_within(value.len() <= *remaining, "facet label bytes")?;
            *remaining -= value.len();
            let color = r
                .host_theme
                .foreground
                .map(crate::color::Paint::resolve)
                .unwrap_or(crate::scene::Color {
                    red: 55,
                    green: 60,
                    blue: 65,
                    alpha: 255,
                });
            if side.horizontal() && policy.labeller.math.is_none() && r.resolved_theme.is_none() {
                return Ok(Strip {
                    side,
                    block: super::text::plain_lines(&value, r, measurer, color)?,
                });
            }
            let mut rich = crate::typography::RichText::plain("");
            rich.lines = value
                .lines()
                .map(|line| {
                    if let Some(fonts) = &policy.labeller.math {
                        crate::typography::RichText::math(line, fonts.clone())
                            .map(|r| r.lines.into_iter().flatten().collect())
                    } else {
                        Ok(vec![crate::typography::RichRun::new(line)])
                    }
                })
                .collect::<ChartResult<_>>()?;
            if !side.horizontal() {
                rich.rotation = if side == AxisSide::Left { -90. } else { 90. };
            }
            let elements = r.resolved_theme.as_deref();
            if let Some(elements) = &elements {
                let name = match side {
                    AxisSide::Top => "strip.text.x.top",
                    AxisSide::Bottom => "strip.text.x.bottom",
                    AxisSide::Left => "strip.text.y.left",
                    AxisSide::Right => "strip.text.y.right",
                };
                if let Some(styled) = super::theme_elements::text_style(elements, name, &rich, r)? {
                    rich = styled;
                } else {
                    rich.lines.clear();
                }
            }
            let mut block = super::text::measure(&rich, r, measurer, color)?;
            if let Some(elements) = &elements {
                let name = match side {
                    AxisSide::Top => "strip.text.x.top",
                    AxisSide::Bottom => "strip.text.x.bottom",
                    AxisSide::Left => "strip.text.y.left",
                    AxisSide::Right => "strip.text.y.right",
                };
                super::theme_elements::margins(elements, name, &mut block, r)?;
            }
            Ok(Strip { side, block })
        })
        .collect()
}
pub(super) fn outside(r: &LayoutRequest, side: AxisSide) -> bool {
    r.resolved_theme.as_ref().is_some_and(|e|matches!(e.value(if side.horizontal(){"strip.placement.x"}else{"strip.placement.y"},""),Some(crate::theme::ThemeValue::Text(v)) if v=="outside"))
}
pub(super) fn switch_padding(r: &LayoutRequest, layout: &FacetLayout) -> f64 {
    r.resolved_theme
        .as_ref()
        .and_then(|e| {
            e.destination_length(
                if matches!(layout, FacetLayout::Grid) {
                    "strip.switch.pad.grid"
                } else {
                    "strip.switch.pad.wrap"
                },
                "",
                0,
            )
        })
        .unwrap_or(0.)
}
pub(super) fn side_index(side: AxisSide) -> usize {
    match side {
        AxisSide::Left => 0,
        AxisSide::Right => 1,
        AxisSide::Top => 2,
        AxisSide::Bottom => 3,
    }
}
pub(super) fn insets(strips: &[Vec<Strip>], gap: f64) -> [f64; 4] {
    let mut sizes = [0_f64; 4];
    for strip in strips.iter().flatten() {
        let size = if strip.side.horizontal() {
            strip.block.bounds.height()
        } else {
            strip.block.bounds.width()
        };
        let i = side_index(strip.side);
        sizes[i] = sizes[i].max(size + 2. * gap);
    }
    sizes
}
pub(super) fn inset(rect: Rect, s: [f64; 4]) -> ChartResult<Rect> {
    Rect::new(
        rect.origin().x() + s[0],
        rect.origin().y() + s[2],
        (rect.width() - s[0] - s[1]).max(0.),
        (rect.height() - s[2] - s[3]).max(0.),
    )
}
pub(super) fn apply_axes(
    policy: &FacetPolicy,
    layout: &FacetLayout,
    panel: &PreparedPanel,
    all: &[PreparedPanel],
    request: &mut LayoutRequest,
) {
    for axis in &mut request.axes {
        let exterior = match axis.side {
            AxisSide::Top => !all
                .iter()
                .any(|p| p.column == panel.column && p.row < panel.row),
            AxisSide::Bottom => !all
                .iter()
                .any(|p| p.column == panel.column && p.row > panel.row),
            AxisSide::Left => !all
                .iter()
                .any(|p| p.row == panel.row && p.column < panel.column),
            AxisSide::Right => !all
                .iter()
                .any(|p| p.row == panel.row && p.column > panel.column),
        };
        let free = if axis.side.horizontal() {
            panel.x_group != crate::grammar::GroupValue::All
        } else {
            panel.y_group != crate::grammar::GroupValue::All
        };
        // Free wrap axes are independent; a grid retains one axis per sharing row/column.
        let independent = matches!(layout, FacetLayout::Wrap { .. }) && free;
        if !exterior && !independent && !policy.axes.includes(axis.side.horizontal()) {
            axis.guide.visible = false;
        }
        if !exterior && !independent && !policy.axis_labels.includes(axis.side.horizontal()) {
            axis.guide
                .components
                .get_or_insert_with(Default::default)
                .labels
                .visible = Some(false);
        }
    }
}
pub(super) fn span(
    chart: &PreparedChart,
    request: &LayoutRequest,
    horizontal: bool,
) -> ChartResult<f64> {
    let Some(spec) = request.axes.iter().find(|a| {
        a.side.horizontal() == horizontal && !matches!(a.scale, AxisScale::Secondary { .. })
    }) else {
        return Ok(1.);
    };
    let axis = super::axes::resolve_axis(
        chart,
        request,
        spec,
        Rect::new(0., 0., 100., 100.)?,
        &mut Default::default(),
    )?;
    let span = match &axis.scale {
        ResolvedScale::Linear(s) => {
            let d = s.viewport();
            (d.end() - d.start()).abs()
        }
        ResolvedScale::Numeric(s) => {
            let d = s.viewport();
            (d.end() - d.start()).abs()
        }
        ResolvedScale::Nonlinear(s) => {
            let d = s.transformed_viewport();
            (d.end() - d.start()).abs()
        }
        ResolvedScale::Band(s) => s
            .reference_viewport()
            .map(|d| (d[1].0 - d[0].0).abs())
            .unwrap_or(s.domain().len() as f64),
        ResolvedScale::Point(s) => s
            .reference_viewport()
            .map(|d| (d[1].0 - d[0].0).abs())
            .unwrap_or(s.domain().len() as f64),
        ResolvedScale::Utc(s) => {
            let d = s.viewport();
            (i128::from(d.end) - i128::from(d.start)).unsigned_abs() as f64
        }
        ResolvedScale::Calendar(s) => {
            let d = s.viewport();
            (i128::from(d.end) - i128::from(d.start)).unsigned_abs() as f64
        }
        ResolvedScale::Provider(s) => s
            .category_viewport()
            .map(|d| (d[1].0 - d[0].0).abs())
            .unwrap_or(1.),
        _ => 1.,
    };
    Ok(if span.is_finite() && span > 0. {
        span
    } else {
        1.
    })
}
