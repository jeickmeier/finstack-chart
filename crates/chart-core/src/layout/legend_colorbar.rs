//! Sampled ggplot colorbars, consuming the prepared mark mapping.
use super::{LayoutRequest, engine::text_request};
use crate::{
    ChartResult, Point, Rect,
    limits::require_within,
    scales::{ColorLegend, GgplotColorbarDisplay, GgplotColorbarOptions},
    scene::{
        Color, GradientDirection, PathCommand, Primitive, SampledGradientMode, SceneItem, Stroke,
    },
    services::{TextMeasurer, TextMetrics, measure_text},
};

struct Label {
    text: String,
    metrics: TextMetrics,
}
struct Key {
    component: crate::scene::GuideComponent,
    position: f64,
    label: Option<Label>,
    tick: bool,
}

pub(super) struct Colorbar {
    id: crate::ScaleId,
    colors: Vec<Color>,
    intervals: Option<Vec<(f64, f64)>>,
    keys: Vec<Key>,
    direction: GradientDirection,
    display: GgplotColorbarDisplay,
}

impl Colorbar {
    pub fn measure(
        legend: &ColorLegend,
        request: &LayoutRequest,
        measurer: &dyn TextMeasurer,
        remaining: &mut usize,
    ) -> ChartResult<Self> {
        let stepped = !legend.colorsteps.is_empty();
        let count = if stepped {
            legend.colorsteps.len()
        } else {
            legend.colorbar.len()
        };
        debug_assert!(count > 0);
        let lower = if stepped {
            legend.colorsteps[0].start.0
        } else {
            legend.colorbar[0].value.0
        };
        let upper = if stepped {
            legend.colorsteps[count - 1].end.0
        } else {
            legend.colorbar[count - 1].value.0
        };
        // Palette samples use absolute Date/POSIX arithmetic; guide keys retain
        // offsets in the declared timestamp unit. Project both in that same unit.
        let (lower, upper) = if !stepped
            && let Some(mapping) = &legend.mapping
            && let crate::scales::ScaleFunctionSpec::Interpolated(scale) = &mapping.function
            && let crate::scales::NormalizationSpec::Ggplot {
                timestamp: Some(timestamp),
                ..
            } = &scale.normalization
        {
            (timestamp.relative(lower), timestamp.relative(upper))
        } else {
            (lower, upper)
        };
        let defaults = GgplotColorbarOptions::default();
        let options = legend
            .mapping
            .as_ref()
            .and_then(|m| m.colorbar_options.as_deref())
            .unwrap_or(&defaults);
        let nbin = options.nbin();
        let pad = if options.display == GgplotColorbarDisplay::Gradient {
            0.
        } else {
            0.5 / nbin
        };
        let positioned = legend
            .numeric_breaks
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| {
                let position = if stepped {
                    legend.colorstep_positions.get(i).copied().flatten()?.0
                } else {
                    if !entry.visible {
                        return None;
                    }
                    let unit = if lower == upper {
                        0.5
                    } else {
                        (entry.transformed.0 - lower) / (upper - lower)
                    };
                    pad + (1. - 2. * pad) * unit
                };
                Some((i, entry, position))
            })
            .collect::<Vec<_>>();
        let visible = positioned.len();
        let mut keys = vec![];
        let mut occurrences = std::collections::BTreeMap::<u64, usize>::new();
        for (index, (original_index, entry, position)) in positioned.into_iter().enumerate() {
            if !position.is_finite() {
                continue;
            }
            let label = if let Some(text) = &entry.label {
                require_within(text.len() <= *remaining, "figure furniture text byte")?;
                *remaining -= text.len();
                Some(Label {
                    text: text.clone(),
                    metrics: measure_text(measurer, text_request(request, text), request.limits)?,
                })
            } else {
                None
            };
            let occurrence = occurrences
                .entry(if entry.value.0 == 0. {
                    0
                } else {
                    entry.value.0.to_bits()
                })
                .or_default();
            let mut component = super::facets::legend_component(
                legend.id,
                crate::scene::GuideRole::LegendTick,
                Some(original_index),
                Some(entry.label.clone().unwrap_or_default()),
            );
            component.tick = Some(crate::scene::GuideTickIdentity {
                value: crate::composition::ScaleValue::Number(entry.value.0),
                occurrence: *occurrence,
            });
            component.side = if options.direction == Some(GradientDirection::Horizontal) {
                super::AxisSide::Bottom
            } else {
                super::AxisSide::Right
            };
            *occurrence += 1;
            keys.push(Key {
                component,
                position,
                label,
                tick: (index != 0 || options.draw_lower_limit)
                    && (index + 1 != visible || options.draw_upper_limit),
            });
        }
        Ok(Self {
            id: legend.id,
            intervals: (stepped && !options.even_steps).then(|| {
                legend
                    .colorsteps
                    .iter()
                    .map(|step| {
                        (
                            (step.start.0 - lower) / (upper - lower),
                            (step.end.0 - lower) / (upper - lower),
                        )
                    })
                    .collect()
            }),
            colors: if stepped {
                legend.colorsteps.iter().map(|step| step.color).collect()
            } else if options.display == GgplotColorbarDisplay::Gradient && lower == upper {
                vec![legend.colorbar[0].color]
            } else {
                legend.colorbar.iter().map(|sample| sample.color).collect()
            },
            keys,
            direction: options.direction.unwrap_or(GradientDirection::Vertical),
            display: if stepped {
                GgplotColorbarDisplay::Rectangles
            } else {
                options.display
            },
        })
    }

    pub(super) fn horizontal(&self) -> bool {
        self.direction == GradientDirection::Horizontal
    }
    fn thickness(request: &LayoutRequest) -> f64 {
        1.5 * request.font_size
    }
    fn length(request: &LayoutRequest) -> f64 {
        10. * request.font_size
    }
    fn horizontal_insets(&self, request: &LayoutRequest) -> (f64, f64) {
        let length = Self::length(request);
        self.keys
            .iter()
            .filter_map(|key| key.label.as_ref().map(|label| (key.position, label)))
            .fold((0_f64, 0_f64), |(left, right), (position, label)| {
                let half = label.metrics.width() / 2.;
                (
                    left.max(half - position * length),
                    right.max(position * length + half - length),
                )
            })
    }
    pub fn width(&self, request: &LayoutRequest) -> f64 {
        if self.horizontal() {
            let (left, right) = self.horizontal_insets(request);
            left + Self::length(request) + right
        } else {
            Self::thickness(request)
                + request.label_gap
                + self
                    .keys
                    .iter()
                    .filter_map(|key| key.label.as_ref())
                    .map(|label| label.metrics.width())
                    .fold(0., f64::max)
        }
    }
    pub fn height(&self, request: &LayoutRequest) -> f64 {
        if self.horizontal() {
            let labels = self
                .keys
                .iter()
                .filter_map(|key| key.label.as_ref())
                .map(|label| label.metrics.height())
                .fold(0., f64::max);
            Self::thickness(request)
                + if labels > 0. {
                    request.label_gap + labels
                } else {
                    0.
                }
        } else {
            Self::length(request)
        }
    }
    fn bar_bounds(&self, bounds: Rect, y: f64, request: &LayoutRequest) -> ChartResult<Rect> {
        if self.horizontal() {
            let (left, right) = self.horizontal_insets(request);
            Rect::new(
                bounds.origin().x() + left.min(bounds.width()),
                y,
                Self::length(request).min((bounds.width() - left - right).max(0.)),
                Self::thickness(request),
            )
        } else {
            Rect::new(
                bounds.origin().x(),
                y,
                Self::thickness(request).min(bounds.width()),
                Self::length(request),
            )
        }
    }

    pub fn paint(
        &self,
        items: &mut Vec<SceneItem>,
        bounds: Rect,
        y: f64,
        request: &LayoutRequest,
    ) -> ChartResult<bool> {
        require_within(
            self.keys
                .len()
                .saturating_mul(2)
                .saturating_add(self.intervals.as_ref().map_or(1, Vec::len))
                <= request.limits.max_items.saturating_sub(items.len()),
            "colorbar scene item",
        )?;
        let bar = self.bar_bounds(bounds, y, request)?;
        if bar.width() <= 0. || bar.height() <= 0. {
            return Ok(true);
        }
        let horizontal = self.horizontal();
        let mut bar_component = super::facets::legend_component(
            self.id,
            crate::scene::GuideRole::LegendBar,
            None,
            None,
        );
        bar_component.side = if horizontal {
            super::AxisSide::Bottom
        } else {
            super::AxisSide::Right
        };
        let mut push = |primitive, component: &crate::scene::GuideComponent| {
            items.push(SceneItem {
                guide: Some(component.clone()),
                layer: None,
                clip: Some(bounds),
                primitive,
            });
        };
        let colors: Vec<_> = if horizontal {
            self.colors.clone()
        } else {
            self.colors.iter().rev().copied().collect()
        };
        if let Some(intervals) = &self.intervals {
            for ((start, end), color) in intervals.iter().zip(&self.colors) {
                if !start.is_finite() || !end.is_finite() {
                    continue;
                }
                let cell = if horizontal {
                    Rect::new(
                        bar.origin().x() + bar.width() * start,
                        bar.origin().y(),
                        bar.width() * (end - start),
                        bar.height(),
                    )?
                } else {
                    Rect::new(
                        bar.origin().x(),
                        bar.max_y() - bar.height() * end,
                        bar.width(),
                        bar.height() * (end - start),
                    )?
                };
                push(
                    Primitive::Rectangle {
                        bounds: cell,
                        fill: *color,
                    },
                    &bar_component,
                );
            }
        } else if colors.len() == 1 {
            push(
                Primitive::Rectangle {
                    bounds: bar,
                    fill: colors[0],
                },
                &bar_component,
            );
        } else {
            push(
                Primitive::SampledGradientRectangle {
                    bounds: bar,
                    direction: self.direction,
                    colors,
                    mode: match self.display {
                        GgplotColorbarDisplay::Raster => SampledGradientMode::CellCenters,
                        GgplotColorbarDisplay::Gradient => SampledGradientMode::Endpoints,
                        GgplotColorbarDisplay::Rectangles => SampledGradientMode::Steps,
                    },
                },
                &bar_component,
            );
        }
        let ink = request
            .host_theme
            .foreground
            .map(crate::color::Paint::resolve)
            .unwrap_or(crate::theme::rgb(55, 60, 65));
        let tick = (if horizontal {
            bar.height()
        } else {
            bar.width()
        })
        .min(request.font_size * 0.25);
        let mut constrained = self.width(request) > bounds.width();
        let mut label_spans = Vec::with_capacity(self.keys.len());
        for key in &self.keys {
            let center = if horizontal {
                bar.origin().x() + bar.width() * key.position
            } else {
                bar.max_y() - bar.height() * key.position
            };
            if key.tick {
                push(
                    Primitive::Path {
                        commands: tick_commands(bar, center, tick, horizontal)?,
                        stroke: Stroke {
                            color: ink,
                            width: 0.5,
                        },
                    },
                    &key.component,
                );
            }
            let Some(label) = &key.label else {
                continue;
            };
            let (left, top) = if horizontal {
                (
                    center - label.metrics.width() / 2.,
                    bar.max_y() + request.label_gap,
                )
            } else {
                (
                    bar.max_x() + request.label_gap,
                    center - label.metrics.height() / 2.,
                )
            };
            constrained |= left < bounds.origin().x()
                || left + label.metrics.width() > bounds.max_x()
                || top < bounds.origin().y()
                || top + label.metrics.height() > bounds.max_y();
            label_spans.push(if horizontal {
                (left, left + label.metrics.width())
            } else {
                (top, top + label.metrics.height())
            });
            let mut component = key.component.clone();
            component.role = crate::scene::GuideRole::LegendLabel;
            push(
                Primitive::Text {
                    origin: Point::new(left, top + label.metrics.ascent())?,
                    text: label.text.clone(),
                    font: request.font.id,
                    font_size: request.font_size,
                    color: ink,
                },
                &component,
            );
        }
        label_spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut end = f64::NEG_INFINITY;
        for (start, finish) in label_spans {
            constrained |= start < end;
            end = end.max(finish);
        }
        Ok(constrained)
    }
}

fn tick_commands(
    bar: Rect,
    center: f64,
    tick: f64,
    horizontal: bool,
) -> ChartResult<Vec<PathCommand>> {
    let (a, b, c, d) = if horizontal {
        (
            Point::new(center, bar.origin().y())?,
            Point::new(center, bar.origin().y() + tick)?,
            Point::new(center, bar.max_y() - tick)?,
            Point::new(center, bar.max_y())?,
        )
    } else {
        (
            Point::new(bar.origin().x(), center)?,
            Point::new(bar.origin().x() + tick, center)?,
            Point::new(bar.max_x() - tick, center)?,
            Point::new(bar.max_x(), center)?,
        )
    };
    Ok(vec![
        PathCommand::MoveTo(a),
        PathCommand::LineTo(b),
        PathCommand::MoveTo(c),
        PathCommand::LineTo(d),
    ])
}
