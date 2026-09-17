//! Explicit Cartesian coordinate capabilities for extensions and host tools.
use super::{ResolvedAxis, ResolvedScale};
use crate::composition::ScaleValue;
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect};
/// Coordinate behavior that callers can inspect before enabling an operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoordinateCapabilities {
    /// Both axes support a semantic inverse (numeric or exact timestamp).
    pub inverse: bool,
    /// Rectangular clipping is available and shared with the scene.
    pub rectangular_clip: bool,
    /// Automatic subdivision of arbitrary data-space curves is implemented.
    pub path_subdivision: bool,
}
/// Borrow the actual resolved named axes; custom tools never invent a second transform.
pub struct Cartesian<'a> {
    x: &'a ResolvedAxis,
    y: &'a ResolvedAxis,
    clip: Rect,
}
impl<'a> Cartesian<'a> {
    /// Bind one horizontal and one vertical coordinate scale plus the existing plot clip.
    pub fn new(x: &'a ResolvedAxis, y: &'a ResolvedAxis, clip: Rect) -> ChartResult<Self> {
        if !x.spec.side.horizontal()
            || y.spec.side.horizontal()
            || matches!(
                x.scale,
                ResolvedScale::Secondary { .. }
                    | ResolvedScale::SecondaryTime { .. }
                    | ResolvedScale::SecondaryDiscrete { .. }
            )
            || matches!(
                y.scale,
                ResolvedScale::Secondary { .. }
                    | ResolvedScale::SecondaryTime { .. }
                    | ResolvedScale::SecondaryDiscrete { .. }
            )
        {
            return Err(unsupported(
                "Cartesian projection requires horizontal/vertical primary coordinate axes.",
            ));
        }
        Ok(Self { x, y, clip })
    }
    /// Report supported operations; category lookup is never disguised as numeric inversion.
    pub fn capabilities(&self) -> CoordinateCapabilities {
        CoordinateCapabilities {
            inverse: self.x.capabilities().numeric_inverse && self.y.capabilities().numeric_inverse,
            rectangular_clip: true,
            path_subdivision: false,
        }
    }
    /// Project declared values with the same omission/clamp/extrapolation policies as marks.
    pub fn project(&self, x: &ScaleValue, y: &ScaleValue) -> ChartResult<Option<Point>> {
        match (self.x.map_value(x)?, self.y.map_value(y)?) {
            (Some(x), Some(y)) => Ok(Some(Point::new(x, y)?)),
            _ => Ok(None),
        }
    }
    /// Recover numeric calculation values or exact source-unit timestamps; categories reject.
    pub fn inverse(&self, p: Point) -> ChartResult<(ScaleValue, ScaleValue)> {
        Ok((self.x.invert_value(p.x())?, self.y.invert_value(p.y())?))
    }
    /// Exact rectangle shared with painter/hit/export clipping; projection alone does not crop data.
    pub fn clip(&self) -> Rect {
        self.clip
    }
}
impl ResolvedAxis {
    /// Actual capabilities of the resolved scale, including guide-only secondary axes.
    pub fn capabilities(&self) -> crate::scales::ScaleCapabilities {
        use crate::scales::ScaleCapabilities;
        match &self.scale {
            ResolvedScale::Unbounded(s) => ScaleCapabilities {
                numeric_inverse: s.has_numeric_inverse(),
                category_lookup: false,
            },
            ResolvedScale::Provider(s) => s.capabilities(),
            ResolvedScale::Linear(_)
            | ResolvedScale::Numeric(_)
            | ResolvedScale::Nonlinear(_)
            | ResolvedScale::Utc(_)
            | ResolvedScale::Calendar(_)
            | ResolvedScale::Session(_) => ScaleCapabilities {
                numeric_inverse: true,
                category_lookup: false,
            },
            ResolvedScale::Band(_) | ResolvedScale::Point(_) => ScaleCapabilities {
                numeric_inverse: false,
                category_lookup: true,
            },
            ResolvedScale::Secondary { .. }
            | ResolvedScale::SecondaryTime { .. }
            | ResolvedScale::SecondaryDiscrete { .. } => ScaleCapabilities {
                numeric_inverse: false,
                category_lookup: false,
            },
        }
    }
    /// Invert a destination position with the resolved numeric/time policy.
    pub fn invert_value(&self, p: f64) -> ChartResult<ScaleValue> {
        match &self.scale {
            ResolvedScale::Unbounded(s) => s.invert(p).map(ScaleValue::Number),
            ResolvedScale::Provider(s) => s.invert(p),
            ResolvedScale::Linear(s) => s.invert(p).map(ScaleValue::Number),
            ResolvedScale::Numeric(s) => s.invert(p).map(ScaleValue::Number),
            ResolvedScale::Nonlinear(s) => s.invert(p).map(ScaleValue::Number),
            ResolvedScale::Utc(s) => Ok(ScaleValue::Timestamp {
                value: s.invert(p)?,
                unit: s.unit(),
            }),
            ResolvedScale::Calendar(s) => Ok(ScaleValue::Timestamp {
                value: s.invert(p)?,
                unit: s.unit(),
            }),
            ResolvedScale::Session(s) => Ok(ScaleValue::Timestamp {
                value: s.invert(p)?,
                unit: s.calendar().unit,
            }),
            ResolvedScale::Band(_) | ResolvedScale::Point(_) => Err(unsupported(
                "Category scales provide category lookup, not a numeric inverse.",
            )),
            ResolvedScale::Secondary { .. }
            | ResolvedScale::SecondaryTime { .. }
            | ResolvedScale::SecondaryDiscrete { .. } => Err(unsupported(
                "A secondary guide cannot be used as an independent coordinate inverse.",
            )),
        }
    }
}
fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::UnsupportedCapability,
        message,
        "Check resolved scale/coordinate capabilities and use an appropriate primary axis or category lookup.",
    )
}

/// Result of coordinate inversion, retaining ambiguity rather than selecting a hidden branch.
#[derive(Clone, Debug, PartialEq)]
pub enum CoordinateInverse {
    /// The position lies outside the coordinate panel or clipping sector.
    Outside,
    /// The coordinate is singular or a positional scale has no semantic inverse.
    Unavailable,
    /// All valid semantic branches, including repeated angular turns.
    Values(Vec<(ScaleValue, ScaleValue)>),
}
/// The panel's post-statistical coordinate map, shared by projection and inspection.
pub struct Coordinates<'a> {
    x: &'a ResolvedAxis,
    y: &'a ResolvedAxis,
    map: super::coordinate_map::CoordinateMap,
}
impl<'a> Coordinates<'a> {
    /// Resolve a coordinate policy over existing primary axes without retraining data.
    pub fn new(
        spec: &crate::grammar::CoordinateSpec,
        x: &'a ResolvedAxis,
        y: &'a ResolvedAxis,
        plot: Rect,
    ) -> ChartResult<Self> {
        Cartesian::new(x, y, plot)?;
        Ok(Self {
            x,
            y,
            map: super::coordinate_resolve::resolve(spec, [x, y], plot)?,
        })
    }
    /// Resolve the exact retained chart mapping, including explicitly registered coordinates.
    pub fn for_chart(
        chart: &crate::grammar::PreparedChart,
        x: &'a ResolvedAxis,
        y: &'a ResolvedAxis,
        plot: Rect,
    ) -> ChartResult<Self> {
        let spec = chart
            .definition()
            .coordinate
            .as_ref()
            .ok_or_else(|| unsupported("Chart has no post-stat coordinate view."))?;
        Cartesian::new(x, y, plot)?;
        let map = super::coordinate_resolve::resolve_with_windows(
            spec,
            [x, y],
            plot,
            Some(&chart.state().axis_windows()),
        )?
        .with_chart_resources(chart)?;
        Ok(Self { x, y, map })
    }
    /// Project semantic values through scale calculation and the post-stat coordinate map.
    pub fn project(&self, x: &ScaleValue, y: &ScaleValue) -> ChartResult<Option<Point>> {
        match (self.x.map_value(x)?, self.y.map_value(y)?) {
            (Some(x), Some(y)) => self.map.project(Point::new(x, y)?),
            _ => Ok(None),
        }
    }
    /// Test the exact panel/annulus/sector boundary, including a filled radial center.
    pub fn contains(&self, position: Point) -> bool {
        self.map.contains(position)
    }
    /// Recover every valid source branch under an explicit maximum branch budget.
    pub fn inverse(&self, position: Point, max_branches: usize) -> ChartResult<CoordinateInverse> {
        if !self.contains(position) {
            return Ok(CoordinateInverse::Outside);
        }
        if !self.map.inverse_available()
            || !self.x.capabilities().numeric_inverse
            || !self.y.capabilities().numeric_inverse
        {
            return Ok(CoordinateInverse::Unavailable);
        }
        let Some(values) = self.map.inverse(position, max_branches)? else {
            return Ok(CoordinateInverse::Unavailable);
        };
        if values.is_empty() {
            return Ok(CoordinateInverse::Outside);
        }
        values
            .into_iter()
            .map(|p| Ok((self.x.invert_value(p.x())?, self.y.invert_value(p.y())?)))
            .collect::<ChartResult<Vec<_>>>()
            .map(CoordinateInverse::Values)
    }
    /// Required physical panel height divided by width, when the policy fixes aspect.
    pub fn aspect(&self) -> Option<f64> {
        self.map.aspect()
    }
}

/// Rebase a gesture into the same retained axis coordinate space used by navigation.
pub(crate) fn navigation_basis(
    chart: &super::LaidOutChart,
    id: crate::ScaleId,
    action: crate::navigation::Navigation,
) -> ChartResult<(
    crate::navigation::Navigation,
    Option<(crate::scales::Bounds, crate::scales::Bounds)>,
)> {
    use crate::navigation::Navigation;
    let Some(spec) = &chart.prepared().definition().coordinate else {
        return Ok((action, None));
    };
    if matches!(action, Navigation::Reset) {
        return Ok((action, None));
    }
    let Some(plot) = chart.plot() else {
        return Err(unsupported(
            "Coordinate navigation requires a measured plot.",
        ));
    };
    let chosen = &chart.axes()[&id];
    let other = chart
        .axes()
        .values()
        .find(|a| {
            a.spec.side.horizontal() != chosen.spec.side.horizontal()
                && !matches!(
                    a.scale,
                    ResolvedScale::Secondary { .. }
                        | ResolvedScale::SecondaryDiscrete { .. }
                        | ResolvedScale::SecondaryTime { .. }
                )
        })
        .ok_or_else(|| {
            unsupported("Coordinate navigation requires both primary positional axes.")
        })?;
    let axes = if chosen.spec.side.horizontal() {
        [chosen, other]
    } else {
        [other, chosen]
    };
    let map = super::coordinate_resolve::resolve_with_windows(
        spec,
        axes,
        plot,
        Some(&chart.prepared().state().axis_windows()),
    )?
    .with_chart_resources(chart.prepared())?;
    let inverse = |p: Point| -> ChartResult<Point> {
        let Some(points) = map.inverse(p, 64)? else {
            return Err(unsupported(
                "The coordinate pointer has no unique inverse on this transform branch.",
            ));
        };
        if points.len() != 1 {
            return Err(unsupported(
                "The coordinate pointer has multiple angular inverse branches.",
            ));
        }
        Ok(points[0])
    };
    let dimension = usize::from(!chosen.spec.side.horizontal());
    let domain = map.axes[dimension];
    let view = map.guide_view(dimension);
    let range = view.map(|v| {
        domain.range[0]
            + (v - domain.viewport[0]) / (domain.viewport[1] - domain.viewport[0])
                * (domain.range[1] - domain.range[0])
    });
    let basis = Some((
        crate::scales::Bounds::new(view[0], view[1])?,
        crate::scales::Bounds::new(range[0], range[1])?,
    ));
    let radial = matches!(spec, crate::grammar::CoordinateSpec::Radial(_));
    let action = match action {
        Navigation::Zoom { anchor, factor } => {
            if !factor.is_finite() || factor <= 0. {
                return Err(unsupported(
                    "Coordinate zoom factor must be finite and positive.",
                ));
            }
            if radial {
                Navigation::Zoom {
                    anchor: inverse(anchor)?,
                    factor,
                }
            } else {
                let edge = |x: f64, y: f64| {
                    Point::new(
                        anchor.x() + (x - anchor.x()) / factor,
                        anchor.y() + (y - anchor.y()) / factor,
                    )
                };
                Navigation::Region {
                    from: inverse(edge(plot.origin().x(), plot.origin().y())?)?,
                    to: inverse(edge(plot.max_x(), plot.max_y())?)?,
                }
            }
        }
        Navigation::Pan { dx, dy } if !radial => Navigation::Region {
            from: inverse(Point::new(plot.origin().x() - dx, plot.origin().y() - dy)?)?,
            to: inverse(Point::new(plot.max_x() - dx, plot.max_y() - dy)?)?,
        },
        Navigation::Region { from, to } if !radial => Navigation::Region {
            from: inverse(from)?,
            to: inverse(to)?,
        },
        Navigation::Pan { .. } | Navigation::Region { .. } => {
            return Err(unsupported(
                "A radial coordinate does not define a unique rectangular pan or region inverse. Use anchored zoom or explicit semantic axis ranges.",
            ));
        }
        Navigation::Reset => unreachable!(),
    };
    Ok((action, basis))
}

impl super::LaidOutChart {
    /// Borrow this presented panel's exact coordinate policy, including runtime windows.
    /// Facet callers select the retained panel chart before requesting its axes.
    pub fn coordinates(
        &self,
        x: crate::ScaleId,
        y: crate::ScaleId,
    ) -> ChartResult<Coordinates<'_>> {
        let x = self
            .axes()
            .get(&x)
            .ok_or_else(|| unsupported("The horizontal coordinate scale is absent."))?;
        let y = self
            .axes()
            .get(&y)
            .ok_or_else(|| unsupported("The vertical coordinate scale is absent."))?;
        let plot = self
            .plot()
            .ok_or_else(|| unsupported("The presented chart has no coordinate plot."))?;
        Cartesian::new(x, y, plot)?;
        let default = crate::grammar::CoordinateSpec::Cartesian(
            crate::grammar::CartesianCoordinate::default(),
        );
        let spec = self
            .prepared()
            .definition()
            .coordinate
            .as_ref()
            .unwrap_or(&default);
        let map = super::coordinate_resolve::resolve_with_windows(
            spec,
            [x, y],
            plot,
            Some(&self.prepared().state().axis_windows()),
        )?;
        Ok(Coordinates { x, y, map })
    }
}
