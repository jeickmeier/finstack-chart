//! Navigation uses scale-transformed coordinates and exact presented ranges. No source work runs.
use crate::grammar::PanelKey;
use crate::layout::{LaidOutChart, ResolvedAxis, ResolvedScale};
use crate::scales::{Bounds, LinearScale, OutsidePolicy, ScaleTransform};
use crate::state::{AxisWindow, AxisWindows};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, ScaleId, SceneStamp};
use std::sync::Arc;

/// Explicit viewport boundary policy; categorical windows always clamp to the finite catalog.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationBoundary {
    /// Preserve interval width while clamping to the trained domain; zoom-out stops at full domain.
    ClampToDomain,
    /// Numeric/time viewports may extend beyond training; session calendars remain finite.
    Extend,
}
/// Host-independent pointer/range navigation request in presented destination units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Navigation {
    /// Factor greater than one zooms in; positive factors below one zoom out.
    Zoom {
        /// Pointer anchor in the pinned scene.
        anchor: Point,
        /// Finite positive zoom factor.
        factor: f64,
    },
    /// Drag displacement from the gesture start, in destination units.
    Pan {
        /// Horizontal displacement.
        dx: f64,
        /// Vertical displacement.
        dy: f64,
    },
    /// Zoom to the inclusive region, preserving each axis's orientation.
    Region {
        /// First corner.
        from: Point,
        /// Opposite corner.
        to: Point,
    },
    /// Reset named windows to the complete trained domains.
    Reset,
}
/// Cheap retained coordinate basis for one gesture. New presentation cannot silently rebase it.
#[derive(Clone, Debug)]
pub struct Navigator {
    presented: Arc<LaidOutChart>,
}
impl Navigator {
    /// Pin an acknowledged presented chart.
    pub fn new(presented: Arc<LaidOutChart>) -> Self {
        Self { presented }
    }
    /// Exact immutable basis, shared with inspection and gesture lifetime ownership.
    pub fn presented(&self) -> &Arc<LaidOutChart> {
        &self.presented
    }
    fn axis(&self, id: ScaleId, panel: Option<&PanelKey>) -> ChartResult<&ResolvedAxis> {
        let chart = match panel {
            Some(key) => {
                &self
                    .presented
                    .panels()
                    .iter()
                    .find(|p| &p.key == key)
                    .ok_or_else(|| error("Navigation panel is absent from the presented scene."))?
                    .chart
            }
            None => &self.presented,
        };
        chart
            .axes()
            .get(&id)
            .ok_or_else(|| error("Navigation axis is absent from the presented panel."))
    }
    /// Produce a complete window map, preserving unrelated axes. The reducer commits or previews it.
    /// Named scale windows affect all panels bound to that identity; independent facets need distinct IDs.
    pub fn navigate(
        &self,
        stamp: SceneStamp,
        axes: &[ScaleId],
        panel: Option<&PanelKey>,
        action: Navigation,
        boundary: NavigationBoundary,
    ) -> ChartResult<AxisWindows> {
        if stamp != self.presented.scene().stamp() {
            return Err(Diagnostic::error(
                DiagnosticCode::Superseded,
                "Navigation belongs to another presented scene.",
                "Use the pinned gesture basis.",
            ));
        }
        if axes.is_empty()
            || axes.len() > 64
            || axes.iter().collect::<std::collections::BTreeSet<_>>().len() != axes.len()
        {
            return Err(error("Navigation needs 1..64 distinct named axes."));
        }
        let mut windows = self
            .presented
            .prepared()
            .state()
            .axis_windows()
            .into_owned();
        for id in axes {
            windows.insert(
                *id,
                navigate_axis(self.axis(*id, panel)?, action, boundary)?,
            );
        }
        crate::state::validate_navigation_windows(&windows)?;
        Ok(windows)
    }
    /// Validate one explicit semantic range against this presented scale, returning the complete map.
    pub fn set_range(
        &self,
        stamp: SceneStamp,
        id: ScaleId,
        panel: Option<&PanelKey>,
        window: AxisWindow,
    ) -> ChartResult<AxisWindows> {
        if stamp != self.presented.scene().stamp() {
            return Err(error("Range setter requires the pinned presented scene."));
        }
        let axis = self.axis(id, panel)?;
        match (&axis.scale, &window) {
            (ResolvedScale::Linear(_), AxisWindow::Numeric(a, b)) => {
                Bounds::new(*a, *b)?.distinct()?;
            }
            (ResolvedScale::Numeric(s), AxisWindow::Numeric(a, b)) => {
                Bounds::new(s.coordinate(*a)?, s.coordinate(*b)?)?.distinct()?;
            }
            (ResolvedScale::Nonlinear(s), AxisWindow::Numeric(a, b)) => {
                transformed(s.transform(), Bounds::new(*a, *b)?)?.distinct()?;
            }
            (ResolvedScale::Calendar(s), AxisWindow::Timestamp(a, b)) => {
                crate::scales::TimeAxisScale::resolve(
                    s.spec().clone(),
                    s.range(),
                    Some(crate::scales::TimeBounds { start: *a, end: *b }),
                    OutsidePolicy::Extend,
                )?;
            }
            (ResolvedScale::Utc(s), AxisWindow::Timestamp(a, b)) => {
                crate::scales::UtcScale::new(
                    s.domain(),
                    Some(crate::scales::TimeBounds { start: *a, end: *b }),
                    s.unit(),
                    s.range(),
                    OutsidePolicy::Extend,
                )?;
            }
            (ResolvedScale::Session(s), AxisWindow::Timestamp(a, b)) => {
                crate::scales::SessionScale::new(
                    s.calendar().clone(),
                    s.range(),
                    Some(crate::scales::TimeBounds { start: *a, end: *b }),
                    OutsidePolicy::Extend,
                )?;
            }
            (ResolvedScale::Point(s), AxisWindow::Category { first, last }) => {
                s.clone().with_window(first, last)?;
            }
            (ResolvedScale::Band(s), AxisWindow::Category { first, last }) => {
                s.clone().with_window(first, last)?;
            }
            _ => {
                return Err(error(
                    "Range value type must match a navigable primary axis.",
                ));
            }
        }
        let mut windows = self
            .presented
            .prepared()
            .state()
            .axis_windows()
            .into_owned();
        windows.insert(id, window);
        Ok(windows)
    }
}
fn error(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use finite nondegenerate coordinates and a navigable primary scale from the pinned presented scene.",
    )
}
fn transformed(t: ScaleTransform, b: Bounds) -> ChartResult<Bounds> {
    Bounds::new(
        t.forward(b.start())?
            .ok_or_else(|| error("Viewport falls outside the scale transform."))?,
        t.forward(b.end())?
            .ok_or_else(|| error("Viewport falls outside the scale transform."))?,
    )
}
fn relative(t: i64, origin: i64) -> ChartResult<f64> {
    let n = i128::from(t) - i128::from(origin);
    if n.abs() > 1_i128 << 53 {
        return Err(error("Time navigation exceeds exact binary64 ticks."));
    }
    Ok(n as f64)
}
fn absolute(t: f64, origin: i64) -> ChartResult<i64> {
    if !t.is_finite() || t.abs() > (1_u64 << 53) as f64 {
        return Err(error("Time navigation exceeds exact binary64 ticks."));
    }
    i64::try_from(i128::from(origin) + t.round() as i128)
        .map_err(|_| error("Time navigation overflows source timestamps."))
}
fn navigate_axis(
    axis: &ResolvedAxis,
    action: Navigation,
    boundary: NavigationBoundary,
) -> ChartResult<AxisWindow> {
    let horizontal = axis.spec.side.horizontal();
    let (domain, view, range) = match &axis.scale {
        ResolvedScale::Unbounded(_) => {
            return Err(Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Unbounded reference ranges do not support finite pan/zoom inversion.",
                "Choose finite scale limits before navigating.",
            ));
        }
        ResolvedScale::Provider(_) => {
            return Err(Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Registered providers do not declare a generic pan/zoom coordinate metric.",
                "Use an explicit provider-supported viewport; an inverse alone does not define navigation.",
            ));
        }
        ResolvedScale::Linear(s) => (s.domain(), s.viewport(), s.range()),
        ResolvedScale::Numeric(s) => (s.coordinate_domain()?, s.coordinate_viewport(), s.range()),
        ResolvedScale::Nonlinear(s) => (
            transformed(s.transform(), s.domain())?,
            transformed(s.transform(), s.viewport())?,
            s.range(),
        ),
        ResolvedScale::Calendar(s) => (s.coordinate_domain()?, s.coordinate_viewport(), s.range()),
        ResolvedScale::Utc(s) => {
            let convert = |b: crate::scales::TimeBounds| {
                Bounds::new(relative(b.start, s.origin())?, relative(b.end, s.origin())?)
            };
            (convert(s.domain())?, convert(s.viewport())?, s.range())
        }
        ResolvedScale::Session(s) => {
            let (d, v) = s.navigation_bounds();
            (d, v, s.range())
        }
        ResolvedScale::Point(s) => {
            return categories(
                s.domain(),
                s.visible_domain(),
                s.range(),
                horizontal,
                action,
            );
        }
        ResolvedScale::Band(s) => {
            return categories(
                s.domain(),
                s.visible_domain(),
                s.range(),
                horizontal,
                action,
            );
        }
        ResolvedScale::Secondary { .. }
        | ResolvedScale::SecondaryTime { .. }
        | ResolvedScale::SecondaryDiscrete { .. } => {
            return Err(error(
                "Guide-only secondary axes navigate through their primary axis.",
            ));
        }
    };
    let next = interval(domain, view, range, horizontal, action, boundary)?;
    match &axis.scale {
        ResolvedScale::Linear(_) => Ok(AxisWindow::Numeric(next.start(), next.end())),
        ResolvedScale::Numeric(s) => Ok(AxisWindow::Numeric(
            s.coordinate_inverse(next.start())?,
            s.coordinate_inverse(next.end())?,
        )),
        ResolvedScale::Nonlinear(s) => Ok(AxisWindow::Numeric(
            s.transform().inverse(next.start())?,
            s.transform().inverse(next.end())?,
        )),
        ResolvedScale::Calendar(s) => Ok(AxisWindow::Timestamp(
            s.coordinate_inverse(next.start())?,
            s.coordinate_inverse(next.end())?,
        )),
        ResolvedScale::Utc(s) => Ok(AxisWindow::Timestamp(
            absolute(next.start(), s.origin())?,
            absolute(next.end(), s.origin())?,
        )),
        ResolvedScale::Session(s) => Ok(AxisWindow::Timestamp(
            s.timestamp_at_offset(next.start())?,
            s.timestamp_at_offset(next.end())?,
        )),
        _ => unreachable!(),
    }
}
fn interval(
    domain: Bounds,
    view: Bounds,
    range: Bounds,
    horizontal: bool,
    action: Navigation,
    boundary: NavigationBoundary,
) -> ChartResult<Bounds> {
    let p = |p: Point| if horizontal { p.x() } else { p.y() };
    let linear = LinearScale::from_domains(domain, view, range, OutsidePolicy::Extend)?;
    let result = match action {
        Navigation::Zoom { anchor, factor } => {
            if !factor.is_finite() || factor <= 0. {
                return Err(error("Zoom factor must be finite and positive."));
            }
            let anchor = p(anchor);
            Bounds::new(
                linear.invert(anchor + (range.start() - anchor) / factor)?,
                linear.invert(anchor + (range.end() - anchor) / factor)?,
            )?
        }
        Navigation::Pan { dx, dy } => {
            if !dx.is_finite() || !dy.is_finite() {
                return Err(error("Pan displacement must be finite."));
            }
            let delta = if horizontal { dx } else { dy };
            Bounds::new(
                linear.invert(range.start() - delta)?,
                linear.invert(range.end() - delta)?,
            )?
        }
        Navigation::Region { from, to } => {
            let (a, b) = (p(from), p(to));
            let (low, high) = (a.min(b), a.max(b));
            let (a, b) = if range.end() > range.start() {
                (low, high)
            } else {
                (high, low)
            };
            Bounds::new(linear.invert(a)?, linear.invert(b)?)?
        }
        Navigation::Reset => domain,
    }
    .distinct()?;
    if boundary == NavigationBoundary::Extend {
        return Ok(result);
    }
    let (low, high) = (
        domain.start().min(domain.end()),
        domain.start().max(domain.end()),
    );
    let width = (result.end() - result.start()).abs();
    if !width.is_finite() || width >= high - low {
        return Ok(domain);
    }
    let start = result.start().min(result.end()).clamp(low, high - width);
    Bounds::new(
        if result.end() > result.start() {
            start
        } else {
            start + width
        },
        if result.end() > result.start() {
            start + width
        } else {
            start
        },
    )?
    .distinct()
}
fn categories(
    labels: &[String],
    visible: &[String],
    range: Bounds,
    horizontal: bool,
    action: Navigation,
) -> ChartResult<AxisWindow> {
    let Some(first) = visible.first() else {
        return Err(error("Cannot navigate an empty category domain."));
    };
    let start = labels
        .iter()
        .position(|s| s == first)
        .expect("window belongs to domain");
    let domain = Bounds::new(0., labels.len() as f64)?;
    let view = Bounds::new(start as f64, (start + visible.len()) as f64)?;
    let next = interval(
        domain,
        view,
        range,
        horizontal,
        action,
        NavigationBoundary::ClampToDomain,
    )?;
    let count = ((next.end() - next.start()).round() as usize).clamp(1, labels.len());
    let start = (next.start().round() as usize).min(labels.len() - count);
    Ok(AxisWindow::Category {
        first: labels[start].clone(),
        last: labels[start + count - 1].clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Revision;
    use crate::data::TimeUnit;
    use crate::grammar::ValueSpace;
    use crate::layout::{AxisSide, AxisSpec};
    use crate::scales::*;
    fn b(a: f64, z: f64) -> Bounds {
        Bounds::new(a, z).unwrap()
    }
    fn axis(scale: ResolvedScale) -> ResolvedAxis {
        ResolvedAxis {
            spec: AxisSpec::new(ScaleId::new(0), AxisSide::Bottom),
            space: ValueSpace::Data,
            scale,
            ticks: vec![],
        }
    }
    fn zoom(x: f64, factor: f64) -> Navigation {
        Navigation::Zoom {
            anchor: Point::new(x, 0.).unwrap(),
            factor,
        }
    }
    #[test]
    fn piecewise_numeric_zoom_retains_knots_and_pointer_anchor() {
        let original = NumericScale::linear()
            .with_domain([0., 10., 100.])
            .unwrap()
            .with_range([0., 50., 100.])
            .unwrap();
        let scale = NumericAxisScale::resolve(
            original.spec().clone(),
            b(0., 100.),
            None,
            OutsidePolicy::Clamp,
        )
        .unwrap();
        let basis = axis(ResolvedScale::Numeric(scale));
        let next = navigate_axis(&basis, zoom(50., 2.), NavigationBoundary::Extend).unwrap();
        numeric(next.clone(), 5., 55.);
        let AxisWindow::Numeric(start, stop) = next else {
            panic!("numeric window")
        };
        let updated = NumericAxisScale::resolve(
            original.spec().clone(),
            b(0., 100.),
            Some(b(start, stop)),
            OutsidePolicy::Clamp,
        )
        .unwrap();
        assert_eq!(updated.map(10.).unwrap(), Some(50.));
        assert_eq!(updated.invert(50.).unwrap(), 10.);
        assert_eq!(updated.domain(), b(0., 100.));
        assert_eq!(updated.map(-10.).unwrap(), Some(0.));
        assert_eq!(updated.invert(-10.).unwrap(), 5.);
    }
    fn numeric(w: AxisWindow, a: f64, z: f64) {
        let AxisWindow::Numeric(x, y) = w else {
            panic!("numeric")
        };
        assert!((x - a).abs() <= 1e-12 * a.abs().max(1.), "{x} != {a}");
        assert!((y - z).abs() <= 1e-12 * z.abs().max(1.), "{y} != {z}");
    }
    #[test]
    fn pointer_anchor_uses_log_and_symlog_coordinates() {
        let log = axis(ResolvedScale::Nonlinear(
            NonlinearScale::resolve(
                None,
                ContinuousDomain::explicit(b(1., 1000.)),
                ScaleTransform::Log { base: 10. },
                b(0., 300.),
                None,
                OutsidePolicy::Extend,
            )
            .unwrap(),
        ));
        numeric(
            navigate_axis(&log, zoom(100., 2.), NavigationBoundary::Extend).unwrap(),
            10_f64.sqrt(),
            100.,
        );
        numeric(
            navigate_axis(
                &log,
                Navigation::Pan { dx: 100., dy: 0. },
                NavigationBoundary::Extend,
            )
            .unwrap(),
            0.1,
            100.,
        );
        let symlog = axis(ResolvedScale::Nonlinear(
            NonlinearScale::resolve(
                None,
                ContinuousDomain::explicit(b(-99., 99.)),
                ScaleTransform::Symlog { threshold: 1. },
                b(0., 100.),
                None,
                OutsidePolicy::Extend,
            )
            .unwrap(),
        ));
        numeric(
            navigate_axis(&symlog, zoom(50., 2.), NavigationBoundary::Extend).unwrap(),
            -9.,
            9.,
        );
    }
    #[test]
    fn reversed_ranges_domain_clamp_region_and_invalid_inputs() {
        let a = axis(ResolvedScale::Linear(
            LinearScale::from_domains(b(10., 0.), b(10., 0.), b(100., 0.), OutsidePolicy::Clamp)
                .unwrap(),
        ));
        numeric(
            navigate_axis(&a, zoom(100., 2.), NavigationBoundary::Extend).unwrap(),
            10.,
            5.,
        );
        numeric(
            navigate_axis(
                &a,
                Navigation::Region {
                    from: Point::new(80., 0.).unwrap(),
                    to: Point::new(20., 0.).unwrap(),
                },
                NavigationBoundary::Extend,
            )
            .unwrap(),
            8.,
            2.,
        );
        numeric(
            navigate_axis(&a, zoom(0., 0.5), NavigationBoundary::ClampToDomain).unwrap(),
            10.,
            0.,
        );
        for factor in [0., -1., f64::NAN, f64::INFINITY] {
            assert!(navigate_axis(&a, zoom(50., factor), NavigationBoundary::Extend).is_err());
        }
        assert!(
            navigate_axis(
                &a,
                Navigation::Region {
                    from: Point::new(1., 0.).unwrap(),
                    to: Point::new(1., 10.).unwrap()
                },
                NavigationBoundary::Extend
            )
            .is_err()
        );
    }
    #[test]
    fn timestamp_navigation_preserves_large_integer_identity() {
        let base = 9_000_000_000_000_000_000_i64;
        let a = axis(ResolvedScale::Utc(
            UtcScale::new(
                TimeBounds {
                    start: base,
                    end: base + 1000,
                },
                None,
                TimeUnit::Microseconds,
                b(0., 100.),
                OutsidePolicy::Extend,
            )
            .unwrap(),
        ));
        assert_eq!(
            navigate_axis(&a, zoom(25., 2.), NavigationBoundary::Extend).unwrap(),
            AxisWindow::Timestamp(base + 125, base + 625)
        );
        assert_eq!(
            navigate_axis(
                &a,
                Navigation::Pan { dx: 50., dy: 0. },
                NavigationBoundary::Extend
            )
            .unwrap(),
            AxisWindow::Timestamp(base - 500, base + 500)
        );
        // Sub-tick collapse is rejected by range validation rather than silently accepted.
        assert!(super::absolute(f64::INFINITY, base).is_err());
    }
    #[test]
    fn calendar_navigation_uses_piecewise_mapping_and_exact_timestamps() {
        use crate::{
            interpolate::Value,
            scales::{TimeAxisScale, TimeScaleSpec},
        };
        let start = 1_700_000_000_000_000_001;
        let spec = TimeScaleSpec {
            unit: TimeUnit::Nanoseconds,
            domain: vec![start, start + 200, start + 1000],
            range: vec![Value::number(0.), Value::number(50.), Value::number(100.)],
            ..Default::default()
        };
        let scale =
            TimeAxisScale::resolve(spec.clone(), b(0., 100.), None, OutsidePolicy::Extend).unwrap();
        let a = axis(ResolvedScale::Calendar(Box::new(scale)));
        let result = navigate_axis(&a, zoom(50., 2.), NavigationBoundary::Extend).unwrap();
        assert_eq!(result, AxisWindow::Timestamp(start + 100, start + 600));
        let next = TimeAxisScale::resolve(
            spec,
            b(0., 100.),
            Some(TimeBounds {
                start: start + 100,
                end: start + 600,
            }),
            OutsidePolicy::Extend,
        )
        .unwrap();
        assert_eq!(next.map(start + 200).unwrap(), Some(50.));
        assert_eq!(next.invert(50.).unwrap(), start + 200);
        let region = navigate_axis(
            &a,
            Navigation::Region {
                from: Point::new(25., 0.).unwrap(),
                to: Point::new(75., 0.).unwrap(),
            },
            NavigationBoundary::Extend,
        )
        .unwrap();
        assert_eq!(region, result);
    }
    #[test]
    fn sessions_zoom_by_active_time_and_reject_unknown_calendar_extent() {
        let scale = SessionScale::new(
            SessionCalendar {
                id: "split".into(),
                revision: Revision::INITIAL,
                unit: TimeUnit::Milliseconds,
                sessions: vec![
                    TimeBounds { start: 0, end: 100 },
                    TimeBounds {
                        start: 1000,
                        end: 1100,
                    },
                ],
                closed: ClosedSessionPolicy::Omit,
            },
            b(0., 100.),
            None,
            OutsidePolicy::Extend,
        )
        .unwrap();
        let a = axis(ResolvedScale::Session(scale));
        assert_eq!(
            navigate_axis(&a, zoom(50., 2.), NavigationBoundary::Extend).unwrap(),
            AxisWindow::Timestamp(50, 1050)
        );
        assert!(
            navigate_axis(
                &a,
                Navigation::Pan { dx: 100., dy: 0. },
                NavigationBoundary::Extend
            )
            .is_err()
        );
        assert_eq!(
            navigate_axis(
                &a,
                Navigation::Pan { dx: 100., dy: 0. },
                NavigationBoundary::ClampToDomain
            )
            .unwrap(),
            AxisWindow::Timestamp(0, 1100)
        );
    }
    #[test]
    fn category_windows_preserve_catalog_lookup_and_singletons() {
        let labels: Vec<String> = ["A", "B", "C", "D", "E"].map(str::to_string).to_vec();
        let point = PointScale::resolve(&labels, &PointOptions::default(), b(100., 0.))
            .unwrap()
            .with_window("B", "D")
            .unwrap();
        assert_eq!(point.domain(), labels);
        assert_eq!(point.visible_domain(), ["B", "C", "D"]);
        assert_eq!(point.center("A").unwrap(), None);
        assert_eq!(point.category_at(50.).unwrap(), Some("C"));
        let a = axis(ResolvedScale::Point(point));
        assert_eq!(
            navigate_axis(&a, zoom(50., 3.), NavigationBoundary::Extend).unwrap(),
            AxisWindow::Category {
                first: "C".into(),
                last: "C".into()
            }
        );
        let band = BandScale::resolve(&labels, &BandOptions::default(), b(0., 100.))
            .unwrap()
            .with_window("C", "C")
            .unwrap();
        assert_eq!(band.domain(), labels);
        assert_eq!(band.category_at(50.).unwrap(), Some("C"));
        assert_eq!(band.extent("A").unwrap(), None);
        assert!(band.clone().with_window("D", "B").is_err());
        assert!(band.with_window("C", "missing").is_err());
    }
}
