//! Resolved geographic coordinate resources shared by geometry, guides and inverse queries.
use crate::grammar::{
    GeoAxisOrder, GeoCrs, GeoLimitsMethod, GeoProjectionSelection, GeoTransform,
    GeographicCoordinate, MapprojProjection,
};
use crate::{ChartResult, DiagnosticCode};
#[derive(Clone)]
pub(super) struct GeographicMap {
    pub vertices_only: bool,
    forward: GeoTransform,
    inverse: Option<GeoTransform>,
    mapproj: Option<MapprojProjection>,
}
impl std::fmt::Debug for GeographicMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GeographicMap")
            .field("mapproj", &self.mapproj)
            .finish_non_exhaustive()
    }
}
impl GeographicMap {
    pub fn new(spec: &GeographicCoordinate) -> ChartResult<Self> {
        spec.validate()?;
        let source = spec.source_crs();
        let destination = match &spec.projection {
            GeoProjectionSelection::Crs(crs) => crs,
            GeoProjectionSelection::Mapproj(_) => &GeoCrs::Wgs84,
        };
        Ok(Self {
            vertices_only: false,
            forward: GeoTransform::new(source, destination, GeoAxisOrder::XY)?,
            inverse: if matches!(spec.projection, GeoProjectionSelection::Crs(_)) {
                Some(GeoTransform::new(destination, source, GeoAxisOrder::XY)?)
            } else {
                None
            },
            mapproj: if let GeoProjectionSelection::Mapproj(p) = &spec.projection {
                Some(p.clone())
            } else {
                None
            },
        })
    }
    pub fn project(&self, p: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        let p = match self.forward.transform(p) {
            Ok(p) => p,
            Err(e) if e.code == DiagnosticCode::NumericalDomain => return Ok(None),
            Err(e) => return Err(e),
        };
        let p = if let Some(projection) = &self.mapproj {
            projection.project(p)?
        } else {
            Some(p)
        };
        Ok(p.filter(|p| p.iter().all(|v| v.is_finite())))
    }
    pub fn angular_step(&self) -> Option<f64> {
        self.forward.source_is_geographic().then(|| {
            1. / self
                .mapproj
                .as_ref()
                .map_or(1., |p| {
                    if p.method == crate::grammar::MapprojMethod::Lune {
                        p.parameters.get(1).copied().unwrap_or(180.).abs() / 180.
                    } else {
                        1.
                    }
                })
                .max(1.)
        })
    }
    pub fn aspect_correction(&self, latitude: [f64; 2]) -> f64 {
        if self.mapproj.is_none() && self.forward.destination_is_geographic() {
            libm::cos((latitude[0] / 2. + latitude[1] / 2.) * std::f64::consts::PI / 180.).abs()
        } else {
            1.
        }
    }
    pub fn inverse(&self, p: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        let Some(inverse) = &self.inverse else {
            return Ok(None);
        };
        match inverse.transform(p) {
            Ok(p) => Ok(Some(p)),
            Err(e) if e.code == DiagnosticCode::NumericalDomain => Ok(None),
            Err(e) => Err(e),
        }
    }
    pub fn bounds(
        &self,
        view: [[f64; 2]; 2],
        method: GeoLimitsMethod,
    ) -> ChartResult<[[f64; 2]; 2]> {
        let mut points = vec![];
        let [x, y] = view;
        if self.mapproj.is_some() {
            for ix in 0..50 {
                for iy in 0..50 {
                    points.push([
                        x[0] + (x[1] - x[0]) * ix as f64 / 49.,
                        y[0] + (y[1] - y[0]) * iy as f64 / 49.,
                    ]);
                }
            }
        } else {
            match method {
                GeoLimitsMethod::Orthogonal => {
                    points.push([x[0], y[0]]);
                    points.push([x[1], y[1]]);
                }
                GeoLimitsMethod::Box | GeoLimitsMethod::GeometryBounds => {
                    for i in 0..20 {
                        let t = i as f64 / 19.;
                        let xx = x[0] + (x[1] - x[0]) * t;
                        let yy = y[0] + (y[1] - y[0]) * t;
                        points.extend([[x[0], yy], [x[1], yy], [xx, y[0]], [xx, y[1]]]);
                    }
                }
                GeoLimitsMethod::Cross => {
                    for i in 0..20 {
                        let t = i as f64 / 19.;
                        points.extend([
                            [x[0] / 2. + x[1] / 2., y[0] + (y[1] - y[0]) * t],
                            [x[0] + (x[1] - x[0]) * t, y[0] / 2. + y[1] / 2.],
                        ]);
                    }
                }
            }
        }
        let mut bounds = [[f64::INFINITY, f64::NEG_INFINITY]; 2];
        for p in points {
            if let Some(p) = self.project(p)? {
                for i in 0..2 {
                    bounds[i][0] = bounds[i][0].min(p[i]);
                    bounds[i][1] = bounds[i][1].max(p[i]);
                }
            }
        }
        if bounds.iter().flatten().any(|v| !v.is_finite()) {
            return Err(crate::scales::error(
                DiagnosticCode::NumericalDomain,
                "Geographic view has no finite projected bounds.",
            ));
        }
        Ok(bounds)
    }
}

impl GeographicMap {
    pub fn geometry_bounds(
        &self,
        chart: &crate::grammar::PreparedChart,
    ) -> ChartResult<Option<[[f64; 2]; 2]>> {
        let mut bounds = [[f64::INFINITY, f64::NEG_INFINITY]; 2];
        for layer in chart.layers() {
            if !chart
                .definition()
                .layers
                .iter()
                .any(|l| l.id == layer.id() && l.geography.is_some())
            {
                continue;
            }
            for p in &layer.geography_vertices {
                if let Some(p) = self.project([p.x(), p.y()])? {
                    for i in 0..2 {
                        bounds[i][0] = bounds[i][0].min(p[i]);
                        bounds[i][1] = bounds[i][1].max(p[i]);
                    }
                }
            }
        }
        Ok(bounds
            .iter()
            .flatten()
            .all(|v| v.is_finite())
            .then_some(bounds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_coord_map_uses_expanded_source_grid_bounds() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/geography-controls.json"
        ))
        .unwrap();
        for case in fixture["map_coordinates"].as_array().unwrap() {
            let method = serde_json::from_value(case["method"].clone()).unwrap();
            let map = GeographicMap::new(&GeographicCoordinate {
                projection: GeoProjectionSelection::Mapproj(
                    MapprojProjection::resolve(method, vec![], Some([90., 0., 0.]), [-20., 35.])
                        .unwrap(),
                ),
                default_crs: Some(GeoCrs::Wgs84),
                ..Default::default()
            })
            .unwrap();
            let view = [
                serde_json::from_value(case["x_range"].clone()).unwrap(),
                serde_json::from_value(case["y_range"].clone()).unwrap(),
            ];
            let actual = map.bounds(view, GeoLimitsMethod::Cross).unwrap();
            for (axis, key) in ["x_projected", "y_projected"].into_iter().enumerate() {
                let expected: [f64; 2] = serde_json::from_value(case[key].clone()).unwrap();
                for (v, e) in actual[axis].iter().zip(expected) {
                    assert!(
                        (v - e).abs() < 2e-10 * (1. + e.abs()),
                        "{} {v} {e}",
                        case["method"]
                    );
                }
            }
        }
    }
}
