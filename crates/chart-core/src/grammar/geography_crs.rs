//! Explicit portable CRS resources; no database, network, grids or process-global lookup.
use super::{GeoAxisOrder, GeoCrs, GeoPosition};
use crate::{ChartResult, DiagnosticCode};
use proj4rs::Proj;
/// Resolved immutable two-dimensional CRS transformation, with explicit source tuple order.
/// Geographic coordinates use degrees at this boundary; ellipsoidal height is explicitly zero.
#[derive(Clone)]
pub struct GeoTransform {
    source: Proj,
    destination: Proj,
    order: GeoAxisOrder,
    identity: bool,
}
impl GeoTransform {
    /// Resolve both complete CRS definitions without resource acquisition.
    pub fn new(source: &GeoCrs, destination: &GeoCrs, order: GeoAxisOrder) -> ChartResult<Self> {
        Ok(Self {
            source: resolve(source)?,
            destination: resolve(destination)?,
            order,
            identity: source == destination,
        })
    }
    /// Whether source tuples are longitude/latitude degrees.
    pub fn source_is_geographic(&self) -> bool {
        self.source.is_latlong()
    }
    /// Whether destination tuples are longitude/latitude degrees.
    pub fn destination_is_geographic(&self) -> bool {
        self.destination.is_latlong()
    }
    /// Transform a finite source tuple, returning a typed domain error on failure on every host.
    pub fn transform(&self, input: GeoPosition) -> ChartResult<GeoPosition> {
        if input.iter().any(|v| !v.is_finite()) {
            return Err(super::error(
                DiagnosticCode::NumericalDomain,
                "Geographic transform requires finite coordinates.",
            ));
        }
        let [mut x, mut y] = if self.order == GeoAxisOrder::YX {
            [input[1], input[0]]
        } else {
            input
        };
        // Identical explicit CRS resources retain exact supplied vertices, as sf does.
        if self.identity {
            return Ok([x, y]);
        }
        if self.source.is_latlong() {
            x = x.to_radians();
            y = y.to_radians();
        }
        let mut p = (x, y, 0.);
        proj4rs::transform::transform(&self.source, &self.destination, &mut p).map_err(|e| {
            super::error(
                DiagnosticCode::NumericalDomain,
                format!("Geographic CRS transform failed: {e}"),
            )
        })?;
        if self.destination.is_latlong() {
            p.0 = p.0.to_degrees();
            p.1 = p.1.to_degrees();
        }
        if !p.0.is_finite() || !p.1.is_finite() {
            return Err(super::error(
                DiagnosticCode::NumericalDomain,
                "Geographic CRS transform produced nonfinite coordinates.",
            ));
        }
        Ok([p.0, p.1])
    }
}
fn resolve(crs: &GeoCrs) -> ChartResult<Proj> {
    crs.validate()?;
    let definition = match crs {
        GeoCrs::Wgs84 => "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs",
        GeoCrs::WebMercator => {
            "+proj=merc +a=6378137 +b=6378137 +lat_ts=0 +lon_0=0 +x_0=0 +y_0=0 +k=1 +units=m +no_defs"
        }
        GeoCrs::Proj(definition) => definition,
    };
    Proj::from_proj_string(definition).map_err(|e| {
        super::error(
            DiagnosticCode::Validation,
            format!("Invalid portable CRS definition: {e}"),
        )
    })
}
