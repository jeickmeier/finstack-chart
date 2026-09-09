//! Pinned named schemes and interpolators, sharing the core color/interpolation math.
//! Tables and catalog recipes are adapted from d3-scale-chromatic 3.1.0; see LICENSE.
mod data;
use crate::{ChartResult, Diagnostic, DiagnosticCode, scene::Color};
pub use data::{InterpolatorId, SchemeId};
use serde::{Deserialize, Serialize};

fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use a supported chromatic identity, size and finite parameter.",
    )
}
/// Discrete catalog family; interpolator sampling does not create named schemes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SchemeFamily {
    /// One fixed categorical table.
    Categorical,
    /// Every separately authored 3–9 color table.
    Sequential,
    /// Every separately authored 3–11 color table.
    Diverging,
}
/// Immutable scheme metadata, with each supported array cardinality.
#[derive(Clone, Debug, Serialize)]
pub struct SchemeInfo {
    /// Stable scheme identity.
    pub id: SchemeId,
    /// Catalog family.
    pub family: SchemeFamily,
    /// All actual table lengths; categorical queries omit the size argument.
    pub sizes: &'static [usize],
}
impl SchemeId {
    /// Describe the exact fixed or sized catalog entry.
    pub fn info(self) -> SchemeInfo {
        let sizes = data::sizes(self);
        let family = if sizes.len() == 1 {
            SchemeFamily::Categorical
        } else if sizes.last() == Some(&11) {
            SchemeFamily::Diverging
        } else {
            SchemeFamily::Sequential
        };
        SchemeInfo {
            id: self,
            family,
            sizes,
        }
    }
}
fn rgba(v: u32) -> Color {
    Color {
        red: (v >> 16) as u8,
        green: (v >> 8) as u8,
        blue: v as u8,
        alpha: 255,
    }
}
/// Query an exact independently authored table, optionally reversing its order.
/// Categorical entries require `None`; Brewer entries require an explicitly supported size.
pub fn scheme(id: SchemeId, size: Option<usize>, reverse: bool) -> ChartResult<Vec<Color>> {
    let table = data::table(id, size).ok_or_else(|| {
        error(
            DiagnosticCode::Validation,
            format!(
                "Unsupported size {size:?} for chromatic scheme {}",
                id.name()
            ),
        )
    })?;
    let mut colors: Vec<_> = table.iter().copied().map(rgba).collect();
    if reverse {
        colors.reverse();
    }
    Ok(colors)
}

/// Stable version-one ramp payload, independent of domain normalization and unknown policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChromaticSpec {
    /// Named reference interpolator.
    pub id: InterpolatorId,
    /// Evaluate at `1-t`; independent from descending scale domains.
    #[serde(default)]
    pub reverse: bool,
}
#[derive(Clone, Debug)]
enum Kernel {
    Color(crate::interpolate::ColorInterpolator),
    Lookup(&'static [u32]),
    Turbo,
    Cividis,
    Rainbow,
    Sinebow,
}
/// Prepared owned evaluator; shared palette resources are resolved once per configuration.
#[derive(Clone, Debug)]
pub struct ChromaticRamp {
    spec: ChromaticSpec,
    kernel: Kernel,
}
impl ChromaticRamp {
    /// Prepare a reference recipe or exact lookup table using shared core primitives.
    pub fn new(spec: ChromaticSpec) -> ChartResult<Self> {
        use crate::{
            color,
            interpolate::{ColorInterpolator, ColorRoute},
        };
        use InterpolatorId::*;
        let kernel = if let Some(table) = data::lookup(spec.id) {
            Kernel::Lookup(table)
        } else {
            match spec.id {
                Turbo => Kernel::Turbo,
                Cividis => Kernel::Cividis,
                Rainbow => Kernel::Rainbow,
                Sinebow => Kernel::Sinebow,
                CubehelixDefault | Warm | Cool => {
                    let (a, b) = match spec.id {
                        CubehelixDefault => (
                            color::cubehelix(300., 0.5, 0.),
                            color::cubehelix(-240., 0.5, 1.),
                        ),
                        Warm => (
                            color::cubehelix(-100., 0.75, 0.35),
                            color::cubehelix(80., 1.5, 0.8),
                        ),
                        _ => (
                            color::cubehelix(260., 0.75, 0.35),
                            color::cubehelix(80., 1.5, 0.8),
                        ),
                    };
                    Kernel::Color(ColorInterpolator::new(
                        ColorRoute::CubehelixLong,
                        a.into(),
                        b.into(),
                        None,
                    )?)
                }
                _ => {
                    let id: SchemeId = spec.id.name().parse()?;
                    let size = *id.info().sizes.last().expect("built-in table sizes");
                    let values = scheme(id, Some(size), false)?
                        .into_iter()
                        .map(|v| {
                            color::rgb(f64::from(v.red), f64::from(v.green), f64::from(v.blue))
                                .into()
                        })
                        .collect::<Vec<_>>();
                    Kernel::Color(ColorInterpolator::rgb_basis(&values, false)?)
                }
            }
        };
        Ok(Self { spec, kernel })
    }
    /// Original immutable catalog selection.
    pub fn spec(&self) -> ChromaticSpec {
        self.spec
    }
    /// Evaluate the pinned function and lower exactly once to canonical sRGB8.
    /// Finite outside inputs use the named recipe's own clamp or wrapping rules.
    pub fn evaluate(&self, t: f64) -> ChartResult<Color> {
        if !t.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Chromatic parameters must be finite.",
            ));
        }
        self.evaluate_scale(t)
    }
    // Finite source observations can normalize to IEEE exceptional parameters.
    // Preserve each reference recipe internally; direct calls retain finite validation.
    pub(crate) fn evaluate_scale(&self, t: f64) -> ChartResult<Color> {
        let t = if self.spec.reverse { 1. - t } else { t };
        if t.is_nan() && matches!(self.kernel, Kernel::Turbo | Kernel::Cividis) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "The named polynomial has no color for an undefined normalized parameter.",
            ));
        }
        use crate::color;
        let value: color::ColorValue = match &self.kernel {
            Kernel::Lookup(table) => {
                if t.is_nan() {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "The named lookup has no color for an undefined normalized parameter.",
                    ));
                }
                return Ok(rgba(
                    table[(t * table.len() as f64)
                        .floor()
                        .max(0.)
                        .min((table.len() - 1) as f64) as usize],
                ));
            }
            Kernel::Color(c) => c.scale_color(t)?,
            Kernel::Turbo => {
                let t = t.clamp(0., 1.);
                color::rgb(
                    34.61
                        + t * (1172.33
                            - t * (10793.56 - t * (33300.12 - t * (38394.49 - t * 14825.05)))),
                    23.31
                        + t * (557.33 + t * (1225.33 - t * (3574.96 - t * (1073.77 + t * 707.56)))),
                    27.2 + t
                        * (3211.1 - t * (15327.97 - t * (27814. - t * (22569.18 - t * 6838.66)))),
                )
                .into()
            }
            Kernel::Cividis => {
                let t = t.clamp(0., 1.);
                color::rgb(
                    -4.54
                        - t * (35.34 - t * (2381.73 - t * (6402.7 - t * (7024.72 - t * 2710.57)))),
                    32.49 + t * (170.73 + t * (52.82 - t * (131.46 - t * (176.58 - t * 67.37)))),
                    81.24
                        + t * (442.36
                            - t * (2482.43 - t * (6167.24 - t * (6614.94 - t * 2475.67)))),
                )
                .into()
            }
            Kernel::Rainbow => {
                let t = if !(0. ..=1.).contains(&t) {
                    t - t.floor()
                } else {
                    t
                };
                let ts = (t - 0.5).abs();
                color::cubehelix(360. * t - 100., 1.5 - 1.5 * ts, 0.8 - 0.9 * ts).into()
            }
            Kernel::Sinebow => {
                let t = (0.5 - t) * std::f64::consts::PI;
                let r = crate::color::trig::sin(t);
                let g = crate::color::trig::sin(t + std::f64::consts::PI / 3.);
                let b = crate::color::trig::sin(t + std::f64::consts::PI * 2. / 3.);
                color::rgb(255. * r * r, 255. * g * g, 255. * b * b).into()
            }
        };
        Ok(value.to_paint())
    }
}

/// Versioned named palette identity retained alongside its checked canonical range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemeSpec {
    /// Catalog revision; only version one is supported.
    #[serde(default = "catalog_version")]
    pub version: u32,
    /// Named discrete scheme.
    pub id: SchemeId,
    /// Required for Brewer sizes; omitted for fixed categorical tables.
    #[serde(default)]
    pub size: Option<usize>,
    /// Reverse the complete named table.
    #[serde(default)]
    pub reverse: bool,
}
fn catalog_version() -> u32 {
    1
}
impl SchemeSpec {
    /// Select the version-one discrete catalog.
    pub fn new(id: SchemeId, size: Option<usize>, reverse: bool) -> Self {
        Self {
            version: 1,
            id,
            size,
            reverse,
        }
    }
    /// Validate version/size and return an independent canonical palette.
    pub fn colors(self) -> ChartResult<Vec<Color>> {
        if self.version != 1 {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported chromatic catalog version.",
            ));
        }
        scheme(self.id, self.size, self.reverse)
    }
    /// Canonical color values for mapped scale ranges; no source parsing per mark.
    pub fn values(self) -> ChartResult<Vec<crate::interpolate::Value>> {
        Ok(self
            .colors()?
            .into_iter()
            .map(|c| crate::interpolate::Value::Color(crate::color::Paint::from(c).value()))
            .collect())
    }
}
/// List all checked scheme metadata and evaluator identities, without loading resources.
pub fn catalog_json() -> ChartResult<String> {
    crate::portable::encode(
        &serde_json::json!({"version":1,"schemes":SchemeId::ALL.into_iter().map(SchemeId::info).collect::<Vec<_>>(),"interpolators":InterpolatorId::ALL.as_slice()}),
    )
}
