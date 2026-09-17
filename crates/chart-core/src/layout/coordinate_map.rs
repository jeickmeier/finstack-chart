//! One resolved post-stat map shared by marks, guide curves and coordinate inspection.
use crate::grammar::{
    CoordinateClip, CoordinateReverse, CoordinateSpec, RadialMode, RadialReverse, ThetaAxis,
};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, Rect};

const TAU: f64 = std::f64::consts::TAU;
/// Existing axis state expressed in scale calculation units before coordinate transforms.
#[derive(Clone, Copy, Debug)]
pub(super) struct CoordinateDomain {
    pub domain: [f64; 2],
    pub viewport: [f64; 2],
    pub range: [f64; 2],
}
/// Resolved panel mapping. Input points use the existing Cartesian axis destination ranges.
#[derive(Clone, Debug)]
pub(super) struct CoordinateMap {
    pub spec: CoordinateSpec,
    extension: Option<std::sync::Arc<dyn crate::grammar::TrainedCoordinate>>,
    pub geographic: Option<super::geographic::GeographicMap>,
    pub plot: Rect,
    pub axes: [CoordinateDomain; 2],
    pub view: [[f64; 2]; 2],
    transformed: [[f64; 2]; 2],
    pub arc: [f64; 2],
    pub radii: [f64; 2],
    pub bbox: [[f64; 2]; 2],
}
fn error(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::NumericalDomain,
        message,
        "Supply finite coordinate parameters and a supported nondegenerate view, or inspect the explicit inverse capability.",
    )
}
fn rescale(value: f64, from: [f64; 2], to: [f64; 2]) -> f64 {
    if from[0] == from[1] {
        (to[0] + to[1]) / 2.
    } else {
        to[0] + (value - from[0]) / (from[1] - from[0]) * (to[1] - to[0])
    }
}
fn reverse(spec: CoordinateReverse, index: usize) -> bool {
    spec == CoordinateReverse::Both
        || (index == 0 && spec == CoordinateReverse::X)
        || (index == 1 && spec == CoordinateReverse::Y)
}
impl CoordinateMap {
    /// View endpoints are resolved with existing typed-axis readers before construction.
    pub fn new(
        spec: CoordinateSpec,
        axes: [CoordinateDomain; 2],
        view: [[f64; 2]; 2],
        plot: Rect,
    ) -> ChartResult<Self> {
        Self::with_transformed_bounds(spec, axes, view, None, plot)
    }
    /// Retain expansion in transformed space even outside the transform's inverse branch.
    pub fn with_transformed_bounds(
        spec: CoordinateSpec,
        axes: [CoordinateDomain; 2],
        view: [[f64; 2]; 2],
        transformed_view: Option<[[f64; 2]; 2]>,
        plot: Rect,
    ) -> ChartResult<Self> {
        let mut transformed = view;
        let mut geographic = None;
        let mut arc = [0., TAU];
        let mut radii = [0., 0.4];
        let mut bbox = [[0., 1.], [0., 1.]];
        match &spec {
            CoordinateSpec::Geographic(v) => {
                let map = super::geographic::GeographicMap::new(v)?;
                transformed = match transformed_view {
                    Some(bounds) => bounds,
                    None => map.bounds(view, v.limits_method)?,
                };
                for (i, pair) in transformed.iter_mut().enumerate() {
                    if reverse(v.view.reverse, i) {
                        pair.swap(0, 1);
                    }
                }
                geographic = Some(map);
            }
            CoordinateSpec::Cartesian(v) => {
                for (index, pair) in transformed.iter_mut().enumerate() {
                    if reverse(v.reverse, index) {
                        pair.swap(0, 1)
                    }
                }
            }
            CoordinateSpec::Transformed(v) => {
                for (index, transform) in [&v.x, &v.y].iter().enumerate() {
                    if !transform.monotone_on(&view[index]) {
                        return Err(error(
                            "Coordinate transform requires one finite monotone inverse branch over the trained view.",
                        ));
                    }
                    transformed[index] = transformed_view.map_or_else(
                        || view[index].map(|x| transform.forward(x)),
                        |values| values[index],
                    );
                    if reverse(v.view.reverse, index) {
                        transformed[index].swap(0, 1)
                    }
                }
            }
            CoordinateSpec::Radial(v) => {
                arc = [v.start, v.end.unwrap_or(v.start + TAU)];
                if arc[0] > arc[1] {
                    arc[0] -= (libm::floor((arc[0] - arc[1]) / TAU) + 1.) * TAU;
                }
                if matches!(v.reverse, RadialReverse::Theta | RadialReverse::Both) {
                    arc.swap(0, 1)
                }
                radii = [v.inner_radius * 0.4, 0.4];
                if matches!(v.reverse, RadialReverse::Radius | RadialReverse::Both) {
                    radii.swap(0, 1)
                }
                bbox = if v.mode == RadialMode::Polar {
                    [[0., 1.], [0., 1.]]
                } else {
                    radial_bbox(arc, radii)
                };
            }
        }
        if view
            .iter()
            .flatten()
            .chain(transformed.iter().flatten())
            .chain(arc.iter())
            .chain(radii.iter())
            .chain(bbox.iter().flatten())
            .any(|v| !v.is_finite())
        {
            return Err(error(
                "Coordinate view or transform does not have finite destination bounds.",
            ));
        }
        Ok(Self {
            spec,
            geographic,
            extension: None,
            plot,
            axes,
            view,
            transformed,
            arc,
            radii,
            bbox,
        })
    }
    pub(super) fn with_chart_resources(
        mut self,
        chart: &crate::grammar::PreparedChart,
    ) -> ChartResult<Self> {
        if let CoordinateSpec::Geographic(spec) = &self.spec
            && matches!(
                spec.projection,
                crate::grammar::GeoProjectionSelection::Crs(_)
            )
            && chart.state().axis_windows().is_empty()
            && let Some(map) = &self.geographic
            && let Some(geometry) = map.geometry_bounds(chart)?
        {
            let previous = map.bounds(self.view, spec.limits_method)?;
            for i in 0..2 {
                let use_geometry =
                    spec.limits_method == crate::grammar::GeoLimitsMethod::GeometryBounds;
                let limits = if i == 0 {
                    &spec.view.xlim
                } else {
                    &spec.view.ylim
                };
                if !use_geometry && limits.is_some() {
                    continue;
                }
                let mut next = if use_geometry {
                    geometry[i]
                } else {
                    [
                        geometry[i][0].min(previous[i][0]),
                        geometry[i][1].max(previous[i][1]),
                    ]
                };
                let reversed = self.transformed[i][0] > self.transformed[i][1];
                let old = if reversed {
                    [self.transformed[i][1], self.transformed[i][0]]
                } else {
                    self.transformed[i]
                };
                let span = previous[i][1] - previous[i][0];
                if span != 0. {
                    let extent = next[1] - next[0];
                    next[0] -= (previous[i][0] - old[0]) / span * extent;
                    next[1] += (old[1] - previous[i][1]) / span * extent;
                }
                if reversed {
                    next.swap(0, 1);
                }
                self.transformed[i] = next;
            }
            if spec.limits_method == crate::grammar::GeoLimitsMethod::GeometryBounds {
                self.view = self.axes.map(|a| a.domain);
            }
        }
        if let Some(selection) = &chart.definition().coordinate_extension {
            self.extension = Some(chart.coordinate_registrations.train(
                selection,
                crate::grammar::CoordinateTrainInput {
                    domains: self.axes.map(|a| a.domain),
                    views: self.view,
                    parameters: &selection.parameters,
                },
            )?);
        }
        Ok(self)
    }
    pub(super) fn inverse_available(&self) -> bool {
        self.extension.as_ref().is_none_or(|e| e.has_inverse())
    }
    fn inverse_extension(&self, normalized: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        let Some(extension) = &self.extension else {
            return Ok(Some(normalized));
        };
        if !extension.has_inverse() {
            return Err(error("Registered coordinate does not declare an inverse."));
        }
        let value = extension.inverse(normalized)?;
        if value.is_some_and(|v| v.iter().any(|x| !x.is_finite())) {
            return Err(error(
                "Registered coordinate returned a nonfinite inverse; use None outside its domain.",
            ));
        }
        Ok(value)
    }
    pub fn clip(&self) -> CoordinateClip {
        match &self.spec {
            CoordinateSpec::Cartesian(v) => v.clip,
            CoordinateSpec::Geographic(v) => v.view.clip,
            CoordinateSpec::Transformed(v) => v.view.clip,
            CoordinateSpec::Radial(v) => v.clip.unwrap_or(if v.mode == RadialMode::Polar {
                CoordinateClip::On
            } else {
                CoordinateClip::Off
            }),
        }
    }
    pub fn affine(&self) -> bool {
        self.extension.is_none() && matches!(self.spec, CoordinateSpec::Cartesian(_))
    }
    pub fn aspect(&self) -> Option<f64> {
        match &self.spec {
            CoordinateSpec::Geographic(v) => {
                let aspect = v.view.ratio.unwrap_or(1.)
                    * ((self.transformed[1][1] - self.transformed[1][0])
                        / (self.transformed[0][1] - self.transformed[0][0]))
                        .abs()
                    / self
                        .geographic
                        .as_ref()
                        .expect("resolved geography")
                        .aspect_correction(self.transformed[1]);
                Some(if v.view.flip { 1. / aspect } else { aspect })
            }
            CoordinateSpec::Radial(_) => {
                Some((self.bbox[1][1] - self.bbox[1][0]) / (self.bbox[0][1] - self.bbox[0][0]))
            }
            CoordinateSpec::Cartesian(v) => v.ratio.map(|r| {
                let aspect = r
                    * ((self.view[1][1] - self.view[1][0]) / (self.view[0][1] - self.view[0][0]))
                        .abs();
                if v.flip { 1. / aspect } else { aspect }
            }),
            CoordinateSpec::Transformed(v) => v.view.ratio.map(|r| {
                let aspect = r
                    * ((self.transformed[1][1] - self.transformed[1][0])
                        / (self.transformed[0][1] - self.transformed[0][0]))
                        .abs();
                if v.view.flip { 1. / aspect } else { aspect }
            }),
        }
    }
    /// Calculation-space endpoints used by the existing guide tick selector.
    /// Display expansion beyond an inverse branch clamps to that branch endpoint.
    pub fn guide_view(&self, index: usize) -> [f64; 2] {
        let CoordinateSpec::Transformed(v) = &self.spec else {
            return self.view[index];
        };
        let transform = if index == 0 { &v.x } else { &v.y };
        let domain = transform.domain();
        let output = domain.map(|x| transform.forward(x));
        let mut bounds = self.transformed[index];
        if reverse(v.view.reverse, index) {
            bounds.swap(0, 1);
        }
        bounds.map(|n| {
            if !output[0].is_nan() && !output[1].is_nan() {
                if n < output[0].min(output[1]) {
                    return if output[0] < output[1] {
                        domain[0]
                    } else {
                        domain[1]
                    };
                }
                if n > output[0].max(output[1]) {
                    return if output[0] > output[1] {
                        domain[0]
                    } else {
                        domain[1]
                    };
                }
            }
            transform.inverse(n)
        })
    }
    /// Project a point already mapped by the shared positional axes.
    pub(super) fn source_values(&self, p: Point) -> [f64; 2] {
        let mut source = [
            rescale(p.x(), self.axes[0].range, self.axes[0].viewport),
            rescale(p.y(), self.axes[1].range, self.axes[1].viewport),
        ];
        // Undoing a prior finite axis projection can cross a closed transform
        // endpoint by a few arithmetic ulps. Restore only that endpoint, within
        // the affine round-trip error; direct source projection is never snapped.
        if let CoordinateSpec::Transformed(v) = &self.spec {
            for (i, t) in [&v.x, &v.y].into_iter().enumerate() {
                let tolerance = 8.
                    * f64::EPSILON
                    * (self.axes[i].viewport[0].abs() + self.axes[i].viewport[1].abs());
                for boundary in t.domain() {
                    if boundary.is_finite() && (source[i] - boundary).abs() <= tolerance {
                        source[i] = boundary;
                    }
                }
            }
        }
        source
    }
    pub fn project(&self, p: Point) -> ChartResult<Option<Point>> {
        self.project_values(self.source_values(p))
    }
    /// Project scale calculation values; infinities attach to the appropriate view edge.
    pub fn project_values(&self, mut source: [f64; 2]) -> ChartResult<Option<Point>> {
        for (index, value) in source.iter_mut().enumerate() {
            if *value == f64::NEG_INFINITY {
                *value = self.view[index][0].min(self.view[index][1]);
            }
            if *value == f64::INFINITY {
                *value = self.view[index][0].max(self.view[index][1]);
            }
        }
        let normalized = match &self.spec {
            CoordinateSpec::Geographic(v) => {
                let Some(values) = self
                    .geographic
                    .as_ref()
                    .expect("resolved geography")
                    .project(source)?
                else {
                    return Ok(None);
                };
                let mut p = [
                    rescale(values[0], self.transformed[0], [0., 1.]),
                    rescale(values[1], self.transformed[1], [0., 1.]),
                ];
                if v.view.flip {
                    p.swap(0, 1);
                }
                p
            }
            CoordinateSpec::Cartesian(v) => {
                let mut p = [
                    rescale(source[0], self.transformed[0], [0., 1.]),
                    rescale(source[1], self.transformed[1], [0., 1.]),
                ];
                if v.flip {
                    p.swap(0, 1);
                }
                p
            }
            CoordinateSpec::Transformed(v) => {
                let mut p = [
                    rescale(v.x.forward(source[0]), self.transformed[0], [0., 1.]),
                    rescale(v.y.forward(source[1]), self.transformed[1], [0., 1.]),
                ];
                if v.view.flip {
                    p.swap(0, 1);
                }
                p
            }
            CoordinateSpec::Radial(v) => {
                let (theta, r) = if v.theta == ThetaAxis::X {
                    (0, 1)
                } else {
                    (1, 0)
                };
                let angle = mod_turn(rescale(source[theta], self.view[theta], self.arc))
                    * if v.mode == RadialMode::Polar {
                        f64::from(v.direction)
                    } else {
                        1.
                    };
                let radius = rescale(source[r], self.view[r], self.radii);
                [
                    rescale(radius * libm::sin(angle) + 0.5, self.bbox[0], [0., 1.]),
                    rescale(radius * libm::cos(angle) + 0.5, self.bbox[1], [0., 1.]),
                ]
            }
        };
        if normalized.iter().any(|v| !v.is_finite()) {
            return Ok(None);
        }
        let normalized = if let Some(extension) = &self.extension {
            let Some(value) = extension.forward(normalized)? else {
                return Ok(None);
            };
            if value.iter().any(|v| !v.is_finite()) {
                return Err(error(
                    "Registered coordinate returned nonfinite output; use None outside its domain.",
                ));
            }
            value
        } else {
            normalized
        };
        Point::new(
            self.plot.origin().x() + normalized[0] * self.plot.width(),
            self.plot.max_y() - normalized[1] * self.plot.height(),
        )
        .map(Some)
    }
    /// Invert to prior Cartesian destination coordinates. Multiple angular turns return
    /// each valid source branch; the center/degenerate radius has no unique inverse.
    pub fn inverse(&self, p: Point, max_branches: usize) -> ChartResult<Option<Vec<Point>>> {
        let normalized = [
            (p.x() - self.plot.origin().x()) / self.plot.width(),
            (self.plot.max_y() - p.y()) / self.plot.height(),
        ];
        let Some(normalized) = self.inverse_extension(normalized)? else {
            return Ok(None);
        };
        let values = match &self.spec {
            CoordinateSpec::Geographic(v) => {
                let mut p = normalized;
                if v.view.flip {
                    p.swap(0, 1);
                }
                let p = [
                    rescale(p[0], [0., 1.], self.transformed[0]),
                    rescale(p[1], [0., 1.], self.transformed[1]),
                ];
                let Some(source) = self
                    .geographic
                    .as_ref()
                    .expect("resolved geography")
                    .inverse(p)?
                else {
                    return Ok(None);
                };
                vec![source]
            }
            CoordinateSpec::Cartesian(v) => {
                let mut p = normalized;
                if v.flip {
                    p.swap(0, 1)
                };
                vec![[
                    rescale(p[0], [0., 1.], self.transformed[0]),
                    rescale(p[1], [0., 1.], self.transformed[1]),
                ]]
            }
            CoordinateSpec::Transformed(v) => {
                let mut p = normalized;
                if v.view.flip {
                    p.swap(0, 1)
                };
                let transformed = [
                    rescale(p[0], [0., 1.], self.transformed[0]),
                    rescale(p[1], [0., 1.], self.transformed[1]),
                ];
                let source = [v.x.inverse(transformed[0]), v.y.inverse(transformed[1])];
                for (i, t) in [&v.x, &v.y].into_iter().enumerate() {
                    let restored = t.forward(source[i]);
                    if !restored.is_finite()
                        || (restored - transformed[i]).abs()
                            > 16. * f64::EPSILON * transformed[i].abs().max(1.)
                    {
                        return Ok(None);
                    }
                }
                vec![source]
            }
            CoordinateSpec::Radial(v) => {
                let p = [
                    rescale(normalized[0], [0., 1.], self.bbox[0]) - 0.5,
                    rescale(normalized[1], [0., 1.], self.bbox[1]) - 0.5,
                ];
                let radius = libm::hypot(p[0], p[1]);
                if radius == 0. || self.radii[0] == self.radii[1] || self.arc[0] == self.arc[1] {
                    return Ok(None);
                }
                if v.mode == RadialMode::Radial
                    && self.clip() == CoordinateClip::On
                    && (radius < self.radii[0].min(self.radii[1]) - 1e-12
                        || radius > self.radii[0].max(self.radii[1]) + 1e-12)
                {
                    return Ok(Some(vec![]));
                }
                let (theta, r) = if v.theta == ThetaAxis::X {
                    (0, 1)
                } else {
                    (1, 0)
                };
                let direction = if v.mode == RadialMode::Polar {
                    f64::from(v.direction)
                } else {
                    1.
                };
                let angle = libm::atan2(p[0], p[1]) * direction;
                let low = self.arc[0].min(self.arc[1]);
                let high = self.arc[0].max(self.arc[1]);
                let first = libm::ceil((low - angle) / TAU - 1e-12);
                let last = libm::floor((high - angle) / TAU + 1e-12);
                if last - first + 1. > max_branches as f64 {
                    return Err(error(
                        "Coordinate inverse exceeds its explicit angular branch budget.",
                    ));
                }
                let mut result = Vec::new();
                let mut k = first;
                while k <= last {
                    let mut value = [0.; 2];
                    value[theta] = rescale(angle + k * TAU, self.arc, self.view[theta]);
                    value[r] = rescale(radius, self.radii, self.view[r]);
                    result.push(value);
                    k += 1.;
                }
                result
            }
        };
        if values.iter().flatten().any(|v| !v.is_finite()) {
            return Ok(None);
        }
        values
            .into_iter()
            .map(|v| {
                Point::new(
                    rescale(v[0], self.axes[0].viewport, self.axes[0].range),
                    rescale(v[1], self.axes[1].viewport, self.axes[1].range),
                )
            })
            .collect::<ChartResult<Vec<_>>>()
            .map(Some)
    }
    /// Invert within a finite source rectangle, including angular overflow and
    /// reflected negative radii. Image sampling uses this window rather than
    /// silently restricting visible clip-off geometry to the coordinate limits.
    pub fn inverse_in(
        &self,
        p: Point,
        input: Rect,
        max_branches: usize,
    ) -> ChartResult<Option<Vec<Point>>> {
        let CoordinateSpec::Radial(v) = &self.spec else {
            return self.inverse(p, max_branches);
        };
        let normalized = [
            (p.x() - self.plot.origin().x()) / self.plot.width(),
            (self.plot.max_y() - p.y()) / self.plot.height(),
        ];
        let Some(normalized) = self.inverse_extension(normalized)? else {
            return Ok(None);
        };
        let xy = [
            rescale(normalized[0], [0., 1.], self.bbox[0]) - 0.5,
            rescale(normalized[1], [0., 1.], self.bbox[1]) - 0.5,
        ];
        let radius = libm::hypot(xy[0], xy[1]);
        if radius == 0. || self.radii[0] == self.radii[1] || self.arc[0] == self.arc[1] {
            return Ok(None);
        }
        let theta = usize::from(v.theta == ThetaAxis::Y);
        let r = 1 - theta;
        let bounds = [
            [input.origin().x(), input.max_x()],
            [input.origin().y(), input.max_y()],
        ];
        let angle_window = bounds[theta].map(|a| {
            rescale(
                rescale(a, self.axes[theta].range, self.axes[theta].viewport),
                self.view[theta],
                self.arc,
            )
        });
        let direction = if v.mode == RadialMode::Polar {
            f64::from(v.direction)
        } else {
            1.
        };
        let mut result = vec![];
        for sign in [1., -1.] {
            let radial_value = rescale(sign * radius, self.radii, self.view[r]);
            let radial_pixel = rescale(radial_value, self.axes[r].viewport, self.axes[r].range);
            if radial_pixel < bounds[r][0] - 1e-10 || radial_pixel > bounds[r][1] + 1e-10 {
                continue;
            }
            let angle = libm::atan2(xy[0] * sign, xy[1] * sign) * direction;
            let first = libm::ceil((angle_window[0].min(angle_window[1]) - angle) / TAU - 1e-12);
            let last = libm::floor((angle_window[0].max(angle_window[1]) - angle) / TAU + 1e-12);
            crate::limits::require_within(
                last - first + 1. <= max_branches.saturating_sub(result.len()) as f64,
                "coordinate inverse source-window branch",
            )?;
            let mut k = first;
            while k <= last {
                let angle_value = rescale(angle + k * TAU, self.arc, self.view[theta]);
                let mut pixel = [0.; 2];
                pixel[theta] = rescale(
                    angle_value,
                    self.axes[theta].viewport,
                    self.axes[theta].range,
                );
                pixel[r] = radial_pixel;
                result.push(Point::new(pixel[0], pixel[1])?);
                k += 1.;
            }
        }
        Ok(Some(result))
    }
    pub fn contains(&self, p: Point) -> bool {
        match &self.spec {
            CoordinateSpec::Radial(v) if v.mode == RadialMode::Radial => {
                let x = rescale(
                    (p.x() - self.plot.origin().x()) / self.plot.width(),
                    [0., 1.],
                    self.bbox[0],
                ) - 0.5;
                let y = rescale(
                    (self.plot.max_y() - p.y()) / self.plot.height(),
                    [0., 1.],
                    self.bbox[1],
                ) - 0.5;
                let radius = libm::hypot(x, y);
                let low = self.radii[0].min(self.radii[1]);
                let high = self.radii[0].max(self.radii[1]);
                radius >= low - 1e-12
                    && radius <= high + 1e-12
                    && (radius == 0.
                        || in_arc(
                            libm::atan2(x, y)
                                * if v.mode == RadialMode::Polar {
                                    f64::from(v.direction)
                                } else {
                                    1.
                                },
                            self.arc,
                        ))
            }
            _ => {
                p.x() >= self.plot.origin().x()
                    && p.x() <= self.plot.max_x()
                    && p.y() >= self.plot.origin().y()
                    && p.y() <= self.plot.max_y()
            }
        }
    }
    /// Conservative destination enclosure of a Cartesian control hull. Scalar
    /// coordinate transforms are monotone on their declared inverse branch;
    /// radial extrema additionally include every cardinal direction in the arc.
    pub fn hull(&self, points: &[Point]) -> ChartResult<Option<Rect>> {
        let mut low = [f64::INFINITY; 2];
        let mut high = [f64::NEG_INFINITY; 2];
        for p in points {
            for (i, v) in [p.x(), p.y()].into_iter().enumerate() {
                low[i] = low[i].min(v);
                high[i] = high[i].max(v);
            }
        }
        if let Some(extension) = &self.extension {
            let mut base = self.clone();
            base.extension = None;
            let mut input = [[f64::INFINITY, f64::NEG_INFINITY]; 2];
            for x in [low[0], high[0]] {
                for y in [low[1], high[1]] {
                    let Some(p) = base.project(Point::new(x, y)?)? else {
                        return Ok(None);
                    };
                    let values = [
                        (p.x() - self.plot.origin().x()) / self.plot.width(),
                        (self.plot.max_y() - p.y()) / self.plot.height(),
                    ];
                    for i in 0..2 {
                        input[i][0] = input[i][0].min(values[i]);
                        input[i][1] = input[i][1].max(values[i]);
                    }
                }
            }
            let Some(output) = extension.bounds(input)? else {
                return Ok(None);
            };
            if output
                .iter()
                .any(|v| v.iter().any(|n| !n.is_finite()) || v[0] > v[1])
            {
                return Err(error(
                    "Registered coordinate returned invalid enclosure bounds.",
                ));
            }
            for x in input[0] {
                for y in input[1] {
                    if let Some(point) = extension.forward([x, y])? {
                        for i in 0..2 {
                            let tolerance = 32. * f64::EPSILON * point[i].abs().max(1.);
                            if !point[i].is_finite()
                                || point[i] < output[i][0] - tolerance
                                || point[i] > output[i][1] + tolerance
                            {
                                return Err(error(
                                    "Registered coordinate enclosure excludes a mapped corner.",
                                ));
                            }
                        }
                    }
                }
            }
            return Rect::new(
                self.plot.origin().x() + output[0][0] * self.plot.width(),
                self.plot.max_y() - output[1][1] * self.plot.height(),
                ((output[0][1] - output[0][0]) * self.plot.width()).max(f64::EPSILON),
                ((output[1][1] - output[1][0]) * self.plot.height()).max(f64::EPSILON),
            )
            .map(Some);
        }
        if let CoordinateSpec::Transformed(v) = &self.spec {
            let a = self.source_values(Point::new(low[0], low[1])?);
            let b = self.source_values(Point::new(high[0], high[1])?);
            for (i, t) in [&v.x, &v.y].into_iter().enumerate() {
                if a[i] != b[i] && !t.monotone_on(&[a[i], b[i]]) {
                    return Ok(None);
                }
            }
        }
        let mut samples = vec![];
        for x in [low[0], high[0]] {
            for y in [low[1], high[1]] {
                samples.push(Point::new(x, y)?);
            }
        }
        if let CoordinateSpec::Radial(v) = &self.spec {
            let theta = usize::from(v.theta == ThetaAxis::Y);
            let input = [low[theta], high[theta]];
            let angles = input.map(|n| {
                rescale(
                    rescale(n, self.axes[theta].range, self.axes[theta].viewport),
                    self.view[theta],
                    self.arc,
                )
            });
            for cardinal in [
                0.,
                std::f64::consts::FRAC_PI_2,
                std::f64::consts::PI,
                3. * std::f64::consts::FRAC_PI_2,
            ] {
                if in_arc(cardinal, angles) {
                    let angle =
                        cardinal + TAU * libm::ceil((angles[0].min(angles[1]) - cardinal) / TAU);
                    let value = rescale(
                        rescale(angle, self.arc, self.view[theta]),
                        self.axes[theta].viewport,
                        self.axes[theta].range,
                    );
                    for r in [low[1 - theta], high[1 - theta]] {
                        samples.push(if theta == 0 {
                            Point::new(value, r)?
                        } else {
                            Point::new(r, value)?
                        });
                    }
                }
            }
        }
        let mut minimum = [f64::INFINITY; 2];
        let mut maximum = [f64::NEG_INFINITY; 2];
        for p in samples {
            let Some(p) = self.project(p)? else {
                return Ok(None);
            };
            for (i, v) in [p.x(), p.y()].into_iter().enumerate() {
                minimum[i] = minimum[i].min(v);
                maximum[i] = maximum[i].max(v);
            }
        }
        Rect::new(
            minimum[0],
            minimum[1],
            (maximum[0] - minimum[0]).max(f64::EPSILON),
            (maximum[1] - minimum[1]).max(f64::EPSILON),
        )
        .map(Some)
    }
    /// Logical angle before text upright adjustment; independent of font units.
    pub fn theta_angle(&self, p: Point) -> Option<f64> {
        let CoordinateSpec::Radial(v) = &self.spec else {
            return None;
        };
        let p = [
            rescale(p.x(), self.axes[0].range, self.axes[0].viewport),
            rescale(p.y(), self.axes[1].range, self.axes[1].viewport),
        ];
        let index = usize::from(v.theta == ThetaAxis::Y);
        Some(
            mod_turn(rescale(p[index], self.view[index], self.arc))
                * if v.mode == RadialMode::Polar {
                    f64::from(v.direction)
                } else {
                    1.
                },
        )
    }
}
fn mod_turn(x: f64) -> f64 {
    x - TAU * libm::floor(x / TAU)
}
fn in_arc(angle: f64, arc: [f64; 2]) -> bool {
    let low = arc[0].min(arc[1]);
    let high = arc[0].max(arc[1]);
    libm::ceil((low - angle) / TAU) <= libm::floor((high - angle) / TAU)
}
/// Source polar_bbox includes guide margin; it is not merely the mark disk envelope.
fn radial_bbox(arc: [f64; 2], radii: [f64; 2]) -> [[f64; 2]; 2] {
    if (arc[1] - arc[0]).abs() >= TAU {
        return [[0., 1.], [0., 1.]];
    }
    let inner = radii[0].min(radii[1]);
    let sin = arc.map(libm::sin);
    let cos = arc.map(libm::cos);
    let xmax = (0.5 * sin[0] + 0.5)
        .max(0.5 * sin[1] + 0.5)
        .max((inner * sin[0] + 0.5).max(inner * sin[1] + 0.5) + 0.05);
    let xmin = (0.5 * sin[0] + 0.5)
        .min(0.5 * sin[1] + 0.5)
        .min((inner * sin[0] + 0.5).min(inner * sin[1] + 0.5) - 0.05);
    let ymax = (0.5 * cos[0] + 0.5)
        .max(0.5 * cos[1] + 0.5)
        .max((inner * cos[0] + 0.5).max(inner * cos[1] + 0.5) + 0.05);
    let ymin = (0.5 * cos[0] + 0.5)
        .min(0.5 * cos[1] + 0.5)
        .min((inner * cos[0] + 0.5).min(inner * cos[1] + 0.5) - 0.05);
    [
        [
            if in_arc(1.5 * std::f64::consts::PI, arc) {
                0.
            } else {
                xmin
            },
            if in_arc(0.5 * std::f64::consts::PI, arc) {
                1.
            } else {
                xmax
            },
        ],
        [
            if in_arc(std::f64::consts::PI, arc) {
                0.
            } else {
                ymin
            },
            if in_arc(0., arc) { 1. } else { ymax },
        ],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{CartesianCoordinate, RadialCoordinate, TransformedCoordinate};
    use crate::scales::GgplotTransform;
    #[test]
    fn coordinate_inverse_and_path_bounds_reject_discontinuous_transform_branch() {
        let axes = [CoordinateDomain {
            domain: [-1., 1.],
            viewport: [-1., 1.],
            range: [0., 100.],
        }; 2];
        let result = CoordinateMap::new(
            CoordinateSpec::Transformed(TransformedCoordinate {
                x: GgplotTransform::Reciprocal,
                ..Default::default()
            }),
            axes,
            [[-1., 1.], [-1., 1.]],
            Rect::new(0., 0., 100., 100.).unwrap(),
        );
        assert!(result.is_err());
    }
    #[test]
    fn projected_controls_match_all_pinned_coordinate_cases() {
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/coordinate-controls.json"
        ))
        .unwrap();
        for case in source["cases"].as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            let panel = &case["panels"][0];
            let pair = |name: &str| {
                let a = panel[name].as_array().unwrap();
                [a[0].as_f64().unwrap(), a[1].as_f64().unwrap()]
            };
            let (spec, view) = if name.starts_with("polar") || name.starts_with("radial") {
                let theta = if name == "polar-y" {
                    ThetaAxis::Y
                } else {
                    ThetaAxis::X
                };
                let v = RadialCoordinate {
                    mode: if name.starts_with("polar") {
                        RadialMode::Polar
                    } else {
                        RadialMode::Radial
                    },
                    theta,
                    start: if name == "polar-start-direction" {
                        std::f64::consts::FRAC_PI_2
                    } else if name == "radial-partial-hole" {
                        std::f64::consts::FRAC_PI_4
                    } else {
                        0.
                    },
                    end: if name == "radial-partial-hole" {
                        Some(1.5 * std::f64::consts::PI)
                    } else {
                        None
                    },
                    inner_radius: match name {
                        "radial-partial-hole" => 0.3,
                        "radial-secondary-guides" => 0.4,
                        _ => 0.,
                    },
                    direction: if name == "polar-start-direction" {
                        -1
                    } else {
                        1
                    },
                    reverse: if name == "radial-reverse" {
                        RadialReverse::Both
                    } else {
                        RadialReverse::None
                    },
                    ..Default::default()
                };
                let view = if theta == ThetaAxis::X {
                    [pair("theta.range"), pair("r.range")]
                } else {
                    [pair("r.range"), pair("theta.range")]
                };
                (CoordinateSpec::Radial(v), view)
            } else {
                let flip = name == "flip-interval" || name == "secondary-flip";
                let view = if flip {
                    [pair("y.range"), pair("x.range")]
                } else {
                    [pair("x.range"), pair("y.range")]
                };
                let transform = match name {
                    "transform-log" => {
                        Some((GgplotTransform::Log { base: 10. }, GgplotTransform::Sqrt))
                    }
                    "transform-reverse" => {
                        Some((GgplotTransform::Reverse, GgplotTransform::Identity))
                    }
                    "coord-log-stat-summary" => Some((
                        GgplotTransform::Log { base: 10. },
                        GgplotTransform::Identity,
                    )),
                    "secondary-transform" => {
                        Some((GgplotTransform::Identity, GgplotTransform::Sqrt))
                    }
                    _ => None,
                };
                if let Some((x, y)) = transform {
                    let source_view =
                        [view[0].map(|v| x.inverse(v)), view[1].map(|v| y.inverse(v))];
                    (
                        CoordinateSpec::Transformed(TransformedCoordinate {
                            x,
                            y,
                            view: Default::default(),
                        }),
                        source_view,
                    )
                } else {
                    (
                        CoordinateSpec::Cartesian(CartesianCoordinate {
                            flip,
                            ratio: match name {
                                "fixed-ratio-one" => Some(1.),
                                "fixed-ratio-two" => Some(2.),
                                _ => None,
                            },
                            ..Default::default()
                        }),
                        view,
                    )
                }
            };
            let axes = [
                CoordinateDomain {
                    domain: view[0],
                    viewport: view[0],
                    range: [0., 1.],
                },
                CoordinateDomain {
                    domain: view[1],
                    viewport: view[1],
                    range: [1., 0.],
                },
            ];
            let map =
                CoordinateMap::new(spec, axes, view, Rect::new(0., 0., 1., 1.).unwrap()).unwrap();
            for (layer, projected) in case["data"]
                .as_array()
                .unwrap()
                .iter()
                .zip(case["projected"].as_array().unwrap())
            {
                let expected = projected["1"].as_array().unwrap();
                for (row, expected) in layer.as_array().unwrap().iter().zip(expected) {
                    let (Some(x), Some(y), Some(ex), Some(ey)) = (
                        row["x"].as_f64(),
                        row["y"].as_f64(),
                        expected["x"].as_f64(),
                        expected["y"].as_f64(),
                    ) else {
                        continue;
                    };
                    let p = map.project_values([x, y]).unwrap().unwrap();
                    assert!(
                        (p.x() - ex).abs() < 1e-12 && (p.y() - (1. - ey)).abs() < 1e-12,
                        "{name}: {row}: {p:?} expected{ex},{ey}"
                    );
                }
            }
            if let Some(expected) = case["coord"]["aspect"][0].as_f64() {
                assert!((map.aspect().unwrap() - expected).abs() < 1e-12, "{name}");
            }
        }
    }
}
