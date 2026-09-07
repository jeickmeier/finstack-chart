use super::*;
use crate::grammar::{ClipPolicy, JitterUnits, Position, PreparedGeometry, ValueSpace};
use crate::provenance::Target;
use crate::scales::error;
use crate::scene::{PathCommand, Primitive, SceneItem, Stroke};
use crate::{ChartResult, DiagnosticCode, Point, Rect};
use std::collections::BTreeMap;

pub(super) fn timestamp(value: f64, origin: i64) -> ChartResult<i64> {
    if !value.is_finite() || value.fract() != 0. || value.abs() > (1_u64 << 53) as f64 {
        return Err(error(
            DiagnosticCode::PrecisionLoss,
            "UTC coordinates must preserve exact integer source ticks.",
        ));
    }
    i64::try_from(i128::from(origin) + value as i128).map_err(|_| {
        error(
            DiagnosticCode::PrecisionLoss,
            "Timestamp origin addition exceeds i64.",
        )
    })
}
impl ResolvedAxis {
    /// Map a prepared layer coordinate using that layer's exact value-space metadata.
    /// Category ordinals never become identities; UTC adds the checked integer origin first.
    pub fn map(&self, value: f64, layer_space: &ValueSpace) -> ChartResult<Option<f64>> {
        match (&self.scale, layer_space) {
            (ResolvedScale::Linear(scale), space) if space == &self.space => scale.map(value),
            (ResolvedScale::Nonlinear(scale), space) if space == &self.space => scale.map(value),
            (
                ResolvedScale::Session(scale),
                ValueSpace::Timestamp {
                    representation,
                    origin,
                },
            ) if representation.unit == scale.calendar().unit => {
                scale.map(timestamp(value, *origin)?)
            }
            (
                ResolvedScale::Utc(scale),
                ValueSpace::Timestamp {
                    representation,
                    origin,
                },
            ) if representation.unit == scale.unit() => scale.map(timestamp(value, *origin)?),
            (
                ResolvedScale::Band(_) | ResolvedScale::Point(_),
                ValueSpace::Categorical { categories },
            ) => {
                if !value.is_finite()
                    || value < 0.
                    || value.fract() != 0.
                    || value >= categories.len() as f64
                {
                    return Err(error(
                        DiagnosticCode::PrecisionLoss,
                        "Category ordinal does not address its layer catalog.",
                    ));
                }
                match &self.scale {
                    ResolvedScale::Band(scale) => scale.center(&categories[value as usize]),
                    ResolvedScale::Point(scale) => scale.center(&categories[value as usize]),
                    _ => unreachable!(),
                }
            }
            _ => Err(error(
                DiagnosticCode::SchemaConflict,
                "Prepared coordinate space is incompatible with its named scale.",
            )),
        }
    }
}

pub(super) struct Output {
    pub interactions: BTreeMap<usize, crate::grammar::GeometryInteraction>,
    pub items: Vec<SceneItem>,
    pub targets: Vec<Vec<Target>>,
    pub omitted: usize,
}
impl Output {
    pub fn push(
        &mut self,
        item: SceneItem,
        targets: Vec<Target>,
        request: &LayoutRequest,
    ) -> ChartResult<()> {
        crate::limits::require_within(
            self.items.len() < request.limits.max_items,
            "layout scene item",
        )?;
        self.items.push(item);
        self.targets.push(targets);
        Ok(())
    }
}
pub(super) fn project(
    chart: &crate::grammar::PreparedChart,
    axes: &BTreeMap<crate::ScaleId, ResolvedAxis>,
    plot: Rect,
    request: &LayoutRequest,
) -> ChartResult<Output> {
    let mut out = Output {
        interactions: BTreeMap::new(),
        items: vec![],
        targets: vec![],
        omitted: 0,
    };
    for layer in chart.layers().iter().filter(|l| l.visible()) {
        let x = &axes[&layer.scales().x];
        let y = &axes[&layer.scales().y];
        let xspace = layer.domains().x_space.as_ref().unwrap_or(&x.space);
        let yspace = layer.domains().y_space.as_ref().unwrap_or(&y.space);
        let clip = Some(if layer.clip() == ClipPolicy::Plot {
            plot
        } else {
            request.figure_bounds.unwrap_or(request.bounds)
        });
        for (mark_index, mark) in layer.marks().iter().enumerate() {
            let output_index = out.items.len();
            let point = |p: Point, target: &Target, edge: f64| -> ChartResult<Option<Point>> {
                let (Some(mut a), Some(mut b)) = (x.map(p.x(), xspace)?, y.map(p.y(), yspace)?)
                else {
                    return Ok(None);
                };
                match layer.position() {
                    Position::Jitter(spec) if spec.units == JitterUnits::Display => {
                        let (dx, dy) = crate::grammar::positions::jitter(
                            spec,
                            target,
                            &Some(mark.group.clone()),
                        );
                        a += dx;
                        b += dy;
                    }
                    Position::Dodge(spec) => {
                        let (ResolvedScale::Band(scale), ValueSpace::Categorical { categories }) =
                            (&x.scale, xspace)
                        else {
                            return Err(error(
                                DiagnosticCode::SchemaConflict,
                                "Dodge requires resolved categorical bands.",
                            ));
                        };
                        let Some(bounds) = scale.extent(&categories[p.x() as usize])? else {
                            return Ok(None);
                        };
                        let slot = spec
                            .order
                            .iter()
                            .position(|g| g == &mark.group)
                            .ok_or_else(|| {
                                error(
                                    DiagnosticCode::Validation,
                                    "Dodge group is absent from its fixed order.",
                                )
                            })?;
                        let width = (bounds.end() - bounds.start()) * spec.width;
                        a += width * ((slot as f64 + 0.5 + edge) / spec.order.len() as f64 - 0.5);
                    }
                    _ => {}
                }
                Ok(Some(Point::new(a, b)?))
            };

            let stroke = Stroke {
                color: mark.style.color,
                width: mark.style.stroke_width,
            };
            let item = |primitive| SceneItem {
                layer: Some(layer.id()),
                clip,
                primitive,
            };
            match &mark.geometry {
                PreparedGeometry::Polygon(points) => {
                    let projected = points
                        .iter()
                        .map(|p| point(*p, &mark.targets[0], 0.))
                        .collect::<ChartResult<Option<Vec<_>>>>()?;
                    if let Some(points) = projected {
                        let mut commands: Vec<_> = points
                            .into_iter()
                            .enumerate()
                            .map(|(i, p)| {
                                if i == 0 {
                                    PathCommand::MoveTo(p)
                                } else {
                                    PathCommand::LineTo(p)
                                }
                            })
                            .collect();
                        commands.push(PathCommand::Close);
                        out.push(
                            item(Primitive::FilledPath {
                                commands,
                                fill: mark.style.color,
                            }),
                            mark.targets.clone(),
                            request,
                        )?;
                    } else {
                        out.omitted += 1;
                    }
                }
                PreparedGeometry::NativePaint {
                    from,
                    to,
                    painter,
                    parameters,
                } => {
                    if let (Some(a), Some(b)) = (
                        point(*from, &mark.targets[0], 0.)?,
                        point(*to, &mark.targets[0], 0.)?,
                    ) {
                        out.push(
                            item(Primitive::NativePaint {
                                bounds: Rect::new(
                                    a.x().min(b.x()),
                                    a.y().min(b.y()),
                                    (b.x() - a.x()).abs(),
                                    (b.y() - a.y()).abs(),
                                )?,
                                painter: painter.clone(),
                                parameters: parameters.clone(),
                                fill: mark.style.color,
                            }),
                            mark.targets.clone(),
                            request,
                        )?;
                    } else {
                        out.omitted += 1;
                    }
                }

                PreparedGeometry::BandRun { lower, upper } => {
                    let mut lo = vec![];
                    let mut hi = vec![];
                    let mut targets = vec![];
                    let flush = |out: &mut Output,
                                 lo: &mut Vec<Point>,
                                 hi: &mut Vec<Point>,
                                 targets: &mut Vec<Target>|
                     -> ChartResult<()> {
                        if lo.is_empty() {
                            return Ok(());
                        }
                        if lo.len() == 1 {
                            out.push(
                                item(Primitive::Rule {
                                    from: lo[0],
                                    to: hi[0],
                                    stroke,
                                }),
                                std::mem::take(targets),
                                request,
                            )?;
                            lo.clear();
                            hi.clear();
                            return Ok(());
                        }
                        let mut commands: Vec<_> = lo
                            .iter()
                            .chain(hi.iter().rev())
                            .enumerate()
                            .map(|(i, p)| {
                                if i == 0 {
                                    PathCommand::MoveTo(*p)
                                } else {
                                    PathCommand::LineTo(*p)
                                }
                            })
                            .collect();
                        commands.push(PathCommand::Close);
                        let mirrored: Vec<_> = targets.iter().rev().cloned().collect();
                        targets.extend(mirrored);
                        out.push(
                            item(Primitive::FilledPath {
                                commands,
                                fill: mark.style.color,
                            }),
                            std::mem::take(targets),
                            request,
                        )?;
                        lo.clear();
                        hi.clear();
                        Ok(())
                    };
                    for ((a, b), target) in lower.iter().zip(upper).zip(&mark.targets) {
                        if let (Some(a), Some(b)) = (point(*a, target, 0.)?, point(*b, target, 0.)?)
                        {
                            lo.push(a);
                            hi.push(b);
                            targets.push(target.clone());
                        } else {
                            out.omitted += 1;
                            flush(&mut out, &mut lo, &mut hi, &mut targets)?;
                        }
                    }
                    flush(&mut out, &mut lo, &mut hi, &mut targets)?;
                }
                PreparedGeometry::LineRun(points) => {
                    let mut run = Vec::new();
                    let mut targets = vec![];
                    let flush = |out: &mut Output,
                                 run: &mut Vec<Point>,
                                 targets: &mut Vec<Target>|
                     -> ChartResult<()> {
                        if run.is_empty() {
                            return Ok(());
                        }
                        let primitive = if run.len() == 1 {
                            Primitive::Point {
                                center: run[0],
                                radius: stroke.width / 2.,
                                fill: stroke.color,
                            }
                        } else {
                            Primitive::Path {
                                commands: run
                                    .iter()
                                    .enumerate()
                                    .map(|(i, p)| {
                                        if i == 0 {
                                            PathCommand::MoveTo(*p)
                                        } else {
                                            PathCommand::LineTo(*p)
                                        }
                                    })
                                    .collect(),
                                stroke,
                            }
                        };
                        out.push(item(primitive), std::mem::take(targets), request)?;
                        run.clear();
                        Ok(())
                    };
                    for (p, target) in points.iter().zip(&mark.targets) {
                        if let Some(p) = point(*p, target, 0.)? {
                            run.push(p);
                            targets.push(target.clone());
                        } else {
                            out.omitted += 1;
                            flush(&mut out, &mut run, &mut targets)?;
                        }
                    }
                    flush(&mut out, &mut run, &mut targets)?;
                }
                PreparedGeometry::Bar { from, to, width } => {
                    if let (Some(a), Some(b)) = (
                        point(*from, &mark.targets[0], 0.)?,
                        point(*to, &mark.targets[0], 0.)?,
                    ) {
                        let primitive = if a.y() == b.y() {
                            Primitive::Rule {
                                from: Point::new(a.x() - width / 2., a.y())?,
                                to: Point::new(a.x() + width / 2., a.y())?,
                                stroke,
                            }
                        } else {
                            Primitive::Rectangle {
                                bounds: Rect::new(
                                    a.x() - width / 2.,
                                    a.y().min(b.y()),
                                    *width,
                                    (b.y() - a.y()).abs(),
                                )?,
                                fill: mark.style.color,
                            }
                        };
                        out.push(item(primitive), mark.targets.clone(), request)?;
                    } else {
                        out.omitted += 1;
                    }
                }
                PreparedGeometry::Point(p) => {
                    if let Some(center) = point(*p, &mark.targets[0], 0.)? {
                        out.push(
                            item(Primitive::Point {
                                center,
                                radius: mark.style.radius,
                                fill: mark.style.color,
                            }),
                            mark.targets.clone(),
                            request,
                        )?;
                    } else {
                        out.omitted += 1;
                    }
                }
                PreparedGeometry::Rule { from, to } | PreparedGeometry::Rectangle { from, to } => {
                    let edge = if matches!(mark.geometry, PreparedGeometry::Rectangle { .. }) {
                        0.5
                    } else {
                        0.
                    };
                    if let (Some(from), Some(to)) = (
                        point(*from, &mark.targets[0], -edge)?,
                        point(*to, &mark.targets[0], edge)?,
                    ) {
                        let primitive = if matches!(mark.geometry, PreparedGeometry::Rule { .. }) {
                            Primitive::Rule { from, to, stroke }
                        } else {
                            Primitive::Rectangle {
                                bounds: Rect::new(
                                    from.x().min(to.x()),
                                    from.y().min(to.y()),
                                    (to.x() - from.x()).abs(),
                                    (to.y() - from.y()).abs(),
                                )?,
                                fill: mark.style.color,
                            }
                        };
                        out.push(item(primitive), mark.targets.clone(), request)?;
                    } else {
                        out.omitted += 1;
                    }
                }
            }
            if let Some(interaction) = layer.interactions().get(&mark_index)
                && out.items.len() == output_index + 1
            {
                use crate::grammar::HitGeometry;
                let map = |p| point(p, &mark.targets[0], 0.);
                let hit = match &interaction.hit {
                    HitGeometry::Point { center, radius } => {
                        map(*center)?.map(|center| HitGeometry::Point {
                            center,
                            radius: *radius,
                        })
                    }
                    HitGeometry::Rectangle { from, to } => match (map(*from)?, map(*to)?) {
                        (Some(from), Some(to)) => Some(HitGeometry::Rectangle { from, to }),
                        _ => None,
                    },
                    HitGeometry::Polygon(points) => points
                        .iter()
                        .map(|p| map(*p))
                        .collect::<ChartResult<Option<Vec<_>>>>()?
                        .map(HitGeometry::Polygon),
                };
                if let Some(hit) = hit {
                    out.interactions.insert(
                        output_index,
                        crate::grammar::GeometryInteraction {
                            hit,
                            ..interaction.clone()
                        },
                    );
                }
            }
        }
    }
    Ok(out)
}
