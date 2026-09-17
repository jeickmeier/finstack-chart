//! Immutable mapproj-compatible projection descriptors; no process-global orientation.
use super::GeoPosition;
use crate::{ChartResult, DiagnosticCode};
/// Named mapproj 1.2.12 projection family, independent from general-purpose CRS identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MapprojMethod {
    /// The pinned mercator projection.
    #[serde(rename = "mercator")]
    Mercator,
    /// The pinned sinusoidal projection.
    #[serde(rename = "sinusoidal")]
    Sinusoidal,
    /// The pinned cylequalarea projection.
    #[serde(rename = "cylequalarea")]
    Cylequalarea,
    /// The pinned cylindrical projection.
    #[serde(rename = "cylindrical")]
    Cylindrical,
    /// The pinned rectangular projection.
    #[serde(rename = "rectangular")]
    Rectangular,
    /// The pinned gall projection.
    #[serde(rename = "gall")]
    Gall,
    /// The pinned mollweide projection.
    #[serde(rename = "mollweide")]
    Mollweide,
    /// The pinned gilbert projection.
    #[serde(rename = "gilbert")]
    Gilbert,
    /// The pinned azequidistant projection.
    #[serde(rename = "azequidistant")]
    Azequidistant,
    /// The pinned azequalarea projection.
    #[serde(rename = "azequalarea")]
    Azequalarea,
    /// The pinned gnomonic projection.
    #[serde(rename = "gnomonic")]
    Gnomonic,
    /// The pinned perspective projection.
    #[serde(rename = "perspective")]
    Perspective,
    /// The pinned orthographic projection.
    #[serde(rename = "orthographic")]
    Orthographic,
    /// The pinned stereographic projection.
    #[serde(rename = "stereographic")]
    Stereographic,
    /// The pinned laue projection.
    #[serde(rename = "laue")]
    Laue,
    /// The pinned fisheye projection.
    #[serde(rename = "fisheye")]
    Fisheye,
    /// The pinned newyorker projection.
    #[serde(rename = "newyorker")]
    Newyorker,
    /// The pinned conic projection.
    #[serde(rename = "conic")]
    Conic,
    /// The pinned simpleconic projection.
    #[serde(rename = "simpleconic")]
    Simpleconic,
    /// The pinned lambert projection.
    #[serde(rename = "lambert")]
    Lambert,
    /// The pinned albers projection.
    #[serde(rename = "albers")]
    Albers,
    /// The pinned bonne projection.
    #[serde(rename = "bonne")]
    Bonne,
    /// The pinned polyconic projection.
    #[serde(rename = "polyconic")]
    Polyconic,
    /// The pinned aitoff projection.
    #[serde(rename = "aitoff")]
    Aitoff,
    /// The pinned lagrange projection.
    #[serde(rename = "lagrange")]
    Lagrange,
    /// The pinned bicentric projection.
    #[serde(rename = "bicentric")]
    Bicentric,
    /// The pinned elliptic projection.
    #[serde(rename = "elliptic")]
    Elliptic,
    /// The pinned globular projection.
    #[serde(rename = "globular")]
    Globular,
    /// The pinned vandergrinten projection.
    #[serde(rename = "vandergrinten")]
    Vandergrinten,
    /// The pinned eisenlohr projection.
    #[serde(rename = "eisenlohr")]
    Eisenlohr,
    /// The pinned guyou projection.
    #[serde(rename = "guyou")]
    Guyou,
    /// The pinned square projection.
    #[serde(rename = "square")]
    Square,
    /// The pinned tetra projection.
    #[serde(rename = "tetra")]
    Tetra,
    /// The pinned hex projection.
    #[serde(rename = "hex")]
    Hex,
    /// The pinned harrison projection.
    #[serde(rename = "harrison")]
    Harrison,
    /// The pinned trapezoidal projection.
    #[serde(rename = "trapezoidal")]
    Trapezoidal,
    /// The pinned lune projection.
    #[serde(rename = "lune")]
    Lune,
    /// The pinned mecca projection.
    #[serde(rename = "mecca")]
    Mecca,
    /// The pinned homing projection.
    #[serde(rename = "homing")]
    Homing,
    /// The pinned sp_mercator projection.
    #[serde(rename = "sp_mercator")]
    SpMercator,
    /// The pinned sp_albers projection.
    #[serde(rename = "sp_albers")]
    SpAlbers,
}
impl MapprojMethod {
    /// Required positional parameter count from the pinned projection inventory.
    pub const fn parameter_count(self) -> usize {
        use MapprojMethod::*;
        match self {
            Simpleconic | Lambert | Albers | Harrison | Trapezoidal | Lune | SpAlbers => 2,
            Cylequalarea | Rectangular | Gall | Perspective | Fisheye | Newyorker | Conic
            | Bonne | Bicentric | Elliptic | Mecca | Homing => 1,
            _ => 0,
        }
    }
}
/// Source-compatible map projection with an explicitly resolved orientation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapprojProjection {
    /// Exact projection family.
    pub method: MapprojMethod,
    /// Ordered parameters in the documented source units.
    pub parameters: Vec<f64>,
    /// Pole latitude, pole longitude and clockwise twist, all in degrees.
    pub orientation: [f64; 3],
}
impl MapprojProjection {
    /// Resolve the source's data-dependent orientation once, without inherited mutable state.
    pub fn resolve(
        method: MapprojMethod,
        parameters: Vec<f64>,
        orientation: Option<[f64; 3]>,
        longitude_extent: [f64; 2],
    ) -> ChartResult<Self> {
        let projection = Self {
            method,
            parameters,
            orientation: orientation.unwrap_or([
                90.,
                0.,
                longitude_extent[0] / 2. + longitude_extent[1] / 2.,
            ]),
        };
        projection.validate()?;
        Ok(projection)
    }
    /// Validate descriptor arity and finiteness; kernel-specific source defaults remain explicit.
    pub fn validate(&self) -> ChartResult<()> {
        if self.parameters.len() != self.method.parameter_count()
            || self
                .parameters
                .iter()
                .chain(self.orientation.iter())
                .any(|v| !v.is_finite())
        {
            return Err(super::error(
                DiagnosticCode::Validation,
                "Map projection requires its documented finite parameters and a finite three-angle orientation.",
            ));
        }
        Ok(())
    }
    /// Project one longitude/latitude degree pair; None denotes the source's clipped output.
    pub fn project(&self, longitude_latitude: GeoPosition) -> ChartResult<Option<GeoPosition>> {
        self.validate()?;
        super::geography_mapproj::project(self, longitude_latitude)
    }
}
