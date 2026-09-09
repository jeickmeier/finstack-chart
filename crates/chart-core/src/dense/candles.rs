use super::{DensityMetrics, DensityOptions, column, error};
use crate::grammar::{ClipPolicy, Geom, GroupValue, PanelKey, PreparedGeometry, Style};
use crate::layout::LaidOutChart;
use crate::provenance::Target;
use crate::scene::{Primitive, SceneItem, Stroke};
use crate::{ChartResult, LayerId, Point, Rect};
use std::collections::BTreeMap;

/// Supplied exact candle, in data space, with its resolved horizontal display position.
#[derive(Clone, Debug)]
pub struct CandleSample {
    /// Data/time-origin coordinate, used for chronological order (stable input ties).
    pub x: f64,
    /// Finite projected x in the destination bucket coordinate system.
    pub display_x: f64,
    /// First supplied price.
    pub open: f64,
    /// Supplied high.
    pub high: f64,
    /// Supplied low.
    pub low: f64,
    /// Last supplied price.
    pub close: f64,
    /// Valid finite supplied volume; None is missing, never manufactured zero.
    pub volume: Option<f64>,
    /// Original exact target retained within the represented population.
    pub target: Target,
}
/// Explicit presentation aggregate; never masquerades as one original source observation.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct CandleBucket {
    /// Native/shared layer context, absent for a standalone aggregation call.
    pub layer: Option<LayerId>,
    /// Exact panel identity.
    pub panel: Option<PanelKey>,
    /// Exact series/group identity.
    pub group: GroupValue,
    /// Horizontal display bucket index, with endpoints relative to the supplied plot.
    pub column: i64,
    /// Midpoint between the first and last represented projected positions.
    pub display_x: f64,
    /// First chronological input coordinate.
    pub first_x: f64,
    /// Last chronological input coordinate.
    pub last_x: f64,
    /// First open.
    pub open: f64,
    /// Maximum high.
    pub high: f64,
    /// Minimum low.
    pub low: f64,
    /// Last close.
    pub close: f64,
    /// Compensated sum of supplied valid volumes; None if none were supplied.
    pub volume: Option<f64>,
    /// Count of valid volume inputs.
    pub valid_volumes: usize,
    /// Every exact represented target in chronological order.
    pub targets: Vec<Target>,
}
/// Aggregate one series using explicit horizontal display buckets. Inputs are stably
/// ordered by data x; ties keep supplied order. Only projected centers inside the view
/// are included. Prices satisfy low <= open/close <= high; all supplied values are finite.
/// Sum uses compensated binary64 arithmetic; overflow is a recoverable error.
pub fn candle_buckets(
    samples: &[CandleSample],
    view: Rect,
    width: f64,
    max_columns: usize,
) -> ChartResult<Vec<CandleBucket>> {
    if !width.is_finite() || width <= 0. || !(1..=65_536).contains(&max_columns) {
        return Err(error("Invalid candle bucket policy."));
    }
    column(view.max_x(), view, width, max_columns)?;
    let mut order: Vec<_> = (0..samples.len()).collect();
    for s in samples {
        if [s.x, s.display_x, s.open, s.high, s.low, s.close]
            .iter()
            .any(|v| !v.is_finite())
            || s.volume.is_some_and(|v| !v.is_finite())
            || s.low > s.high
            || s.low > s.open
            || s.open > s.high
            || s.low > s.close
            || s.close > s.high
        {
            return Err(error(
                "Candle aggregation requires finite supplied OHLC with valid price ordering.",
            ));
        }
    }
    order.sort_by(|a, b| samples[*a].x.total_cmp(&samples[*b].x));
    let mut groups: BTreeMap<i64, Vec<&CandleSample>> = BTreeMap::new();
    for i in order {
        let s = &samples[i];
        if s.display_x < view.origin().x() || s.display_x > view.max_x() {
            continue;
        }
        groups
            .entry(column(s.display_x, view, width, max_columns)?)
            .or_default()
            .push(s);
    }
    groups
        .into_iter()
        .map(|(column, group)| {
            let first = group[0];
            let last = group[group.len() - 1];
            let (mut sum, mut correction, mut valid) = (0., 0., 0);
            for s in &group {
                if let Some(v) = s.volume {
                    let next = sum + v;
                    correction += if sum.abs() >= v.abs() {
                        (sum - next) + v
                    } else {
                        (v - next) + sum
                    };
                    sum = next;
                    valid += 1;
                }
            }
            let volume = sum + correction;
            if valid > 0 && !volume.is_finite() {
                return Err(error("Candle volume sum overflowed finite binary64."));
            }
            Ok(CandleBucket {
                layer: None,
                panel: None,
                group: GroupValue::All,
                column,
                display_x: first.display_x * 0.5 + last.display_x * 0.5,
                first_x: first.x,
                last_x: last.x,
                open: first.open,
                close: last.close,
                high: group
                    .iter()
                    .map(|s| s.high)
                    .fold(f64::NEG_INFINITY, f64::max),
                low: group.iter().map(|s| s.low).fold(f64::INFINITY, f64::min),
                volume: (valid > 0).then_some(volume),
                valid_volumes: valid,
                targets: group.into_iter().map(|s| s.target.clone()).collect(),
            })
        })
        .collect()
}
#[allow(clippy::too_many_arguments)] // One bounded layout pass shares output/accounting owners.
pub(super) fn prepare(
    chart: &LaidOutChart,
    panel: Option<PanelKey>,
    width: f64,
    options: &DensityOptions,
    replacements: &mut BTreeMap<(Option<PanelKey>, LayerId), Vec<SceneItem>>,
    buckets: &mut Vec<CandleBucket>,
    metrics: &mut DensityMetrics,
) -> ChartResult<()> {
    let Some(plot) = chart.plot() else {
        return Ok(());
    };
    for layer in chart.prepared().layers().iter().filter(|l| l.visible()) {
        let Some(authored) = chart
            .prepared()
            .definition()
            .layers
            .iter()
            .find(|l| l.id == layer.id())
        else {
            continue;
        };
        if !matches!(authored.geom, Geom::Ohlc { .. }) {
            continue;
        }
        let mapped_size = matches!(&authored.mappings,crate::grammar::Mappings::Source(a) if a.size.is_some() || (authored.inherit && chart.prepared().definition().mappings.size.is_some()));
        if !chart.insets().is_empty()
            || mapped_size
            || authored.geometry_extension.is_some()
            || authored.color.is_some()
        {
            metrics.candle_fallbacks += 1;
            continue;
        }
        let x = &chart.axes()[&layer.scales().x];
        let y = &chart.axes()[&layer.scales().y];
        let xspace = layer.domains().x_space.as_ref().unwrap_or(&x.space);
        let yspace = layer.domains().y_space.as_ref().unwrap_or(&y.space);
        let mut groups: BTreeMap<GroupValue, (Style, Vec<CandleSample>)> = BTreeMap::new();
        for marks in layer.marks().chunks_exact(2) {
            let (
                PreparedGeometry::Rule {
                    from: low,
                    to: high,
                },
                PreparedGeometry::Bar {
                    from: open,
                    to: close,
                    ..
                },
            ) = (&marks[0].geometry, &marks[1].geometry)
            else {
                return Err(error(
                    "Prepared OHLC marks do not contain paired wick/body values.",
                ));
            };
            let Some(display_x) = x.map(open.x(), xspace)? else {
                continue;
            };
            let target = marks[0].targets[0].clone();
            let volume = if let Some(field) = options.candle_volume.get(&layer.id()) {
                let Target::Source(r) = &target else {
                    return Err(error("Volume binding requires a supplied source candle."));
                };
                let data = chart.prepared().source().get()?.dataset(r.dataset)?;
                if data.schema().field(*field).is_none() {
                    return Err(error(
                        "Candle volume field is absent from the source schema.",
                    ));
                }
                match data.row(r.key).and_then(|r| r.value(*field)) {
                    Some(crate::data::ValueRef::Float64(v)) => v.is_finite().then_some(v),
                    Some(crate::data::ValueRef::Int64(v))
                        if v.unsigned_abs() <= 9_007_199_254_740_992 =>
                    {
                        Some(v as f64)
                    }
                    Some(crate::data::ValueRef::UInt64(v)) if v <= 9_007_199_254_740_992 => {
                        Some(v as f64)
                    }
                    None => None,
                    _ => {
                        return Err(error(
                            "Candle volume must be numeric and preserve binary64 integer precision.",
                        ));
                    }
                }
            } else {
                None
            };
            groups
                .entry(marks[0].group.clone())
                .or_insert_with(|| (marks[0].style, vec![]))
                .1
                .push(CandleSample {
                    x: open.x(),
                    display_x,
                    open: open.y(),
                    high: high.y(),
                    low: low.y(),
                    close: close.y(),
                    volume,
                    target,
                });
        }
        let mut items = vec![];
        let mut layer_buckets = vec![];
        let paint = chart.paint_theme(layer.id()).cloned().unwrap_or_default();
        let candle_colors = authored
            .candle_colors
            .map(|c| c.map_colors(crate::color::Paint::resolve));
        for (group, (style, samples)) in groups {
            for mut b in candle_buckets(&samples, plot, width, options.max_columns)? {
                let (Some(low), Some(high), Some(open), Some(close)) = (
                    y.map(b.low, yspace)?,
                    y.map(b.high, yspace)?,
                    y.map(b.open, yspace)?,
                    y.map(b.close, yspace)?,
                ) else {
                    continue;
                };
                let color =
                    paint.mark.unwrap_or_else(|| {
                        candle_colors.map_or(style.color, |c| {
                            if b.close >= b.open { c.up } else { c.down }
                        })
                    });
                let color = crate::theme::paint_color(color, paint.color_mode);
                let stroke_width = paint.stroke_width.unwrap_or(style.stroke_width);
                let clip = Some(if layer.clip() == ClipPolicy::Plot {
                    plot
                } else {
                    chart.scene().bounds()
                });
                items.push(SceneItem {
                    layer: Some(layer.id()),
                    clip,
                    primitive: Primitive::Rule {
                        from: Point::new(b.display_x, low)?,
                        to: Point::new(b.display_x, high)?,
                        stroke: Stroke {
                            color,
                            width: stroke_width,
                        },
                    },
                });
                let body = if open == close {
                    Primitive::Rule {
                        from: Point::new(b.display_x - width * 0.4, open)?,
                        to: Point::new(b.display_x + width * 0.4, open)?,
                        stroke: Stroke {
                            color,
                            width: stroke_width,
                        },
                    }
                } else {
                    Primitive::Rectangle {
                        bounds: Rect::new(
                            b.display_x - width * 0.4,
                            open.min(close),
                            width * 0.8,
                            (close - open).abs(),
                        )?,
                        fill: color,
                    }
                };
                items.push(SceneItem {
                    layer: Some(layer.id()),
                    clip,
                    primitive: body,
                });
                b.layer = Some(layer.id());
                b.panel = panel.clone();
                b.group = group.clone();
                layer_buckets.push(b);
            }
        }
        if let Some(dashes) = paint.dashes.filter(|d| !d.is_empty()) {
            for item in &mut items {
                if let Primitive::Rule { from, to, stroke } = &item.primitive {
                    item.primitive = Primitive::DashedPath {
                        commands: vec![
                            crate::scene::PathCommand::MoveTo(*from),
                            crate::scene::PathCommand::LineTo(*to),
                        ],
                        stroke: *stroke,
                        dashes: dashes.clone(),
                    };
                }
            }
        }
        if items.len() < layer.marks().len() {
            metrics.represented_samples +=
                layer_buckets.iter().map(|b| b.targets.len()).sum::<usize>();
            metrics.candle_buckets += layer_buckets.len();
            buckets.extend(layer_buckets);
            replacements.insert((panel.clone(), layer.id()), items);
        }
    }
    Ok(())
}
