//! GG15 portable, explicitly owned geographic feature resources.
use super::GroupValue;
use crate::{ChartResult, DiagnosticCode, FieldId};
use std::collections::BTreeSet;

/// Two source coordinates in the declared CRS and axis order; no hidden altitude conversion.
pub type GeoPosition = [f64; 2];
/// Polygon contours: the first is the exterior, the rest are holes.
pub type GeoPolygon = Vec<Vec<GeoPosition>>;

/// Typed geometry without a GIS runtime, filesystem resource or interpreter object.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GeoGeometry {
    /// Explicit empty geometry, retained for joins but not painted.
    Empty,
    /// One position.
    Point(GeoPosition),
    /// Independent point components in source order.
    MultiPoint(Vec<GeoPosition>),
    /// Ordered connected positions.
    LineString(Vec<GeoPosition>),
    /// Independent lines; components never receive connecting edges.
    MultiLineString(Vec<Vec<GeoPosition>>),
    /// One exterior contour and its holes.
    Polygon(GeoPolygon),
    /// Independent polygon components, each with its own holes.
    MultiPolygon(Vec<GeoPolygon>),
    /// Ordered mixed geometry components.
    Collection(Vec<GeoGeometry>),
}
/// Source axis order is independent of the destination projection's axis convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GeoAxisOrder {
    /// Longitude/easting first, latitude/northing second.
    #[default]
    XY,
    /// Latitude/northing first, longitude/easting second.
    YX,
}
/// Explicit source CRS; projection strings are resources, never database lookups.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GeoCrs {
    /// WGS84 longitude and latitude in degrees (EPSG:4326 with explicit axis order).
    Wgs84,
    /// Spherical Web Mercator metres (EPSG:3857).
    WebMercator,
    /// Fully specified portable PROJ definition; implicit grids and init lookups are forbidden.
    Proj(String),
}
/// One stable feature, optionally overriding the collection's source CRS.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeoFeature {
    /// Exact join identity; integers are never narrowed through binary64.
    pub id: GroupValue,
    /// Owned geometry with ordered components and holes.
    pub geometry: GeoGeometry,
    /// Explicit mixed-CRS override; None inherits the collection resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crs: Option<GeoCrs>,
}
/// Immutable, serializable feature input shared by Rust and host adapters.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeoFeatureCollection {
    /// Features in source order; duplicate ids are rejected rather than silently overwritten.
    pub features: Vec<GeoFeature>,
    /// Explicit default source CRS.
    pub crs: GeoCrs,
    /// Coordinate tuple order for all features, including CRS overrides.
    #[serde(default)]
    pub axis_order: GeoAxisOrder,
}
/// Geometry-derived authoring operation; all use the same retained feature join.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GeoOperation {
    /// Filled polygon, line and point components.
    #[default]
    Geometry,
    /// Boundary lines, including hole contours.
    Borders,
    /// Interior label anchor (largest polygon component for multipolygons).
    PointOnSurface,
    /// Area/length-weighted planar centroid in the explicitly selected calculation CRS.
    Centroid,
    /// Ordered vertices, retaining feature and component provenance.
    Coordinates,
}
/// A feature resource joined to an ordinary typed dataset; aesthetics remain core mappings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeoLayerSpec {
    /// Owned feature resource.
    pub collection: GeoFeatureCollection,
    /// Dataset field whose exact typed values identify features.
    pub join: FieldId,
    /// Shared geometry or derived-location operation.
    #[serde(default)]
    pub operation: GeoOperation,
    /// Apply source geometry-family colour defaults unless a colour aesthetic is mapped.
    #[serde(default = "geo_default")]
    pub default_color: bool,
    /// Apply source geometry-family linewidth defaults unless a linewidth is mapped.
    #[serde(default = "geo_default")]
    pub default_line_width: bool,
}
fn geo_default() -> bool {
    true
}
/// Explicit geometry processing bounds, including recursive collections.
#[derive(Clone, Copy, Debug)]
pub struct GeoLimits {
    /// Maximum feature count.
    pub features: usize,
    /// Maximum total coordinate count.
    pub positions: usize,
    /// Maximum nested collection depth.
    pub depth: usize,
}
impl Default for GeoLimits {
    fn default() -> Self {
        Self {
            features: 100_000,
            positions: 1_000_000,
            depth: 32,
        }
    }
}
fn invalid(message: &str) -> crate::Diagnostic {
    super::error(DiagnosticCode::Validation, message)
}
impl GeoFeatureCollection {
    /// Validate finite tuples, contour structure, exact identities and bounded resource size.
    pub fn validate(&self, limits: GeoLimits) -> ChartResult<()> {
        if self.features.len() > limits.features {
            return Err(super::error(
                DiagnosticCode::ResourceLimit,
                "Geographic feature budget exceeded.",
            ));
        }
        self.crs.validate()?;
        let mut ids = BTreeSet::new();
        let mut remaining = limits.positions;
        for feature in &self.features {
            if matches!(
                feature.id,
                GroupValue::All | GroupValue::Missing | GroupValue::Interaction(_)
            ) || !ids.insert(&feature.id)
            {
                return Err(invalid(
                    "Geographic feature ids must be unique, nonmissing scalar values.",
                ));
            }
            if let Some(crs) = &feature.crs {
                crs.validate()?;
            }
            feature
                .geometry
                .validate_inner(0, limits.depth, &mut remaining)?;
        }
        Ok(())
    }
}
impl GeoCrs {
    /// Reject implicit resource acquisition in portable projection descriptions.
    pub fn validate(&self) -> ChartResult<()> {
        if let Self::Proj(definition) = self {
            if definition.trim().is_empty()
                || !definition
                    .split_whitespace()
                    .any(|v| v.starts_with("+proj="))
            {
                return Err(invalid(
                    "A geographic CRS requires an explicit +proj definition.",
                ));
            }
            for word in definition.split_whitespace() {
                if [
                    "+init=",
                    "+nadgrids=",
                    "+geoidgrids=",
                    "+file=",
                    "+catalog=",
                ]
                .iter()
                .any(|prefix| word.starts_with(prefix))
                {
                    return Err(invalid(
                        "Geographic CRS definitions cannot acquire external resources.",
                    ));
                }
            }
        }
        Ok(())
    }
}
impl GeoGeometry {
    fn validate_inner(
        &self,
        depth: usize,
        max_depth: usize,
        remaining: &mut usize,
    ) -> ChartResult<()> {
        if depth > max_depth {
            return Err(super::error(
                DiagnosticCode::ResourceLimit,
                "Geographic collection depth exceeded.",
            ));
        }
        let positions = |values: &[GeoPosition], remaining: &mut usize| -> ChartResult<()> {
            if values.len() > *remaining {
                return Err(super::error(
                    DiagnosticCode::ResourceLimit,
                    "Geographic position budget exceeded.",
                ));
            }
            *remaining -= values.len();
            if values.iter().flatten().any(|v| !v.is_finite()) {
                return Err(invalid(
                    "Geographic coordinates must be finite; use an explicit empty geometry.",
                ));
            }
            Ok(())
        };
        let line = |values: &[GeoPosition], remaining: &mut usize| -> ChartResult<()> {
            if values.len() == 1 {
                return Err(invalid(
                    "A nonempty geographic line requires at least two positions.",
                ));
            }
            positions(values, remaining)
        };
        let polygon = |rings: &GeoPolygon, remaining: &mut usize| -> ChartResult<()> {
            for ring in rings {
                if ring.len() < 4 || ring.first() != ring.last() {
                    return Err(invalid(
                        "Geographic polygon rings require at least four positions and explicit closure.",
                    ));
                }
                positions(ring, remaining)?;
            }
            Ok(())
        };
        match self {
            Self::Empty => Ok(()),
            Self::Point(p) => positions(std::slice::from_ref(p), remaining),
            Self::MultiPoint(p) => positions(p, remaining),
            Self::LineString(p) => line(p, remaining),
            Self::MultiLineString(lines) => {
                for p in lines {
                    line(p, remaining)?;
                }
                Ok(())
            }
            Self::Polygon(p) => polygon(p, remaining),
            Self::MultiPolygon(polygons) => {
                for p in polygons {
                    polygon(p, remaining)?;
                }
                Ok(())
            }
            Self::Collection(parts) => {
                for p in parts {
                    p.validate_inner(depth + 1, max_depth, remaining)?;
                }
                Ok(())
            }
        }
    }
}
