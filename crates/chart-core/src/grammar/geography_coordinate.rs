//! Explicit geographic panel projection, source CRS and graticule policy.
use super::{CartesianCoordinate, GeoCrs, MapprojProjection};
/// Geographic destination selection; named mapproj methods are not PROJ aliases.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum GeoProjectionSelection {
    /// A complete destination CRS, with portable forward and inverse transformations.
    Crs(GeoCrs),
    /// Source-compatible spherical map projection and captured immutable orientation.
    Mapproj(MapprojProjection),
}
impl Default for GeoProjectionSelection {
    fn default() -> Self {
        Self::Crs(GeoCrs::Wgs84)
    }
}
/// sf-compatible method for mapping coordinate limits into the projected panel.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GeoLimitsMethod {
    /// Twenty samples on each centre crossing, matching coord_sf's default.
    #[default]
    Cross,
    /// Twenty samples along each of the four bounding-box edges.
    Box,
    /// Transform the two opposite authored limit corners.
    Orthogonal,
    /// Use the transformed geometry bounding box rather than authored limit crossings.
    GeometryBounds,
}
/// Which geographic graticules receive labels at one panel edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GeoAxisLabel {
    /// No graticule labels on this edge.
    #[default]
    None,
    /// Longitude graticules.
    Longitude,
    /// Latitude graticules.
    Latitude,
    /// Both graticule families.
    Both,
}
/// Explicit geographic graticule resources; ordinary axes retain their shared tick engine.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeoGraticule {
    /// Datum CRS; None suppresses geographic graticules.
    pub datum: Option<GeoCrs>,
    /// Explicit longitude degree breaks; None requests the shared automatic tick engine.
    pub longitude: Option<Vec<f64>>,
    /// Explicit latitude degree breaks; None requests the shared automatic tick engine.
    pub latitude: Option<Vec<f64>>,
    /// Number of source samples per graticule before bounded destination refinement.
    pub subdivisions: usize,
    /// Label policy for bottom, left, top and right panel edges.
    pub label_axes: [GeoAxisLabel; 4],
}
impl Default for GeoGraticule {
    fn default() -> Self {
        Self {
            datum: Some(GeoCrs::Wgs84),
            longitude: None,
            latitude: None,
            subdivisions: 100,
            label_axes: [
                GeoAxisLabel::Longitude,
                GeoAxisLabel::Latitude,
                GeoAxisLabel::None,
                GeoAxisLabel::None,
            ],
        }
    }
}
/// Shared geographic coordinate policy for feature geometry and ordinary overlay marks.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GeographicCoordinate {
    /// Explicit destination projection.
    pub projection: GeoProjectionSelection,
    /// Source CRS of ordinary x/y overlays and limits. None uses destination CRS units;
    /// mapproj destinations use longitude/latitude degrees when absent.
    pub default_crs: Option<GeoCrs>,
    /// Coordinate limits, reversal, expansion, clipping and explicit aspect override.
    pub view: CartesianCoordinate,
    /// Source-compatible projected limit calculation policy.
    pub limits_method: GeoLimitsMethod,
    /// Geographic grid and edge label controls.
    pub graticule: GeoGraticule,
}
impl GeographicCoordinate {
    /// Calculation CRS shared by feature geometry, overlays and authored limits.
    pub fn source_crs(&self) -> &GeoCrs {
        self.default_crs.as_ref().unwrap_or(match &self.projection {
            GeoProjectionSelection::Crs(crs) => crs,
            GeoProjectionSelection::Mapproj(_) => &GeoCrs::Wgs84,
        })
    }
    pub(crate) fn validate(&self) -> crate::ChartResult<()> {
        self.source_crs().validate()?;
        match &self.projection {
            GeoProjectionSelection::Crs(crs) => crs.validate()?,
            GeoProjectionSelection::Mapproj(p) => p.validate()?,
        }
        if let Some(crs) = &self.graticule.datum {
            crs.validate()?;
        }
        if self.graticule.subdivisions < 2
            || self.graticule.subdivisions > 100_000
            || self
                .graticule
                .longitude
                .iter()
                .chain(self.graticule.latitude.iter())
                .flatten()
                .any(|v| !v.is_finite())
        {
            return Err(super::error(
                crate::DiagnosticCode::Validation,
                "Geographic graticule breaks and bounded subdivisions must be finite and valid.",
            ));
        }
        super::CoordinateSpec::Cartesian(self.view.clone()).validate()
    }
}
