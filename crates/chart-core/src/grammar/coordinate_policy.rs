//! GG13 chart-level coordinate policies. Scale training remains owned by axes/statistics.
use crate::composition::ScaleValue;
use crate::scales::GgplotTransform;

/// Post-stat panel coordinate mapping; absence on a definition preserves legacy Cartesian behavior.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum CoordinateSpec {
    /// Cartesian views, flips, reversals and fixed data aspect.
    Cartesian(CartesianCoordinate),
    /// Apply the existing transform descriptors after statistics and positions.
    Transformed(TransformedCoordinate),
    /// Full or partial polar/radial panels through one shared projection.
    Radial(RadialCoordinate),
    /// Explicit geographic projection, CRS resources and geographic panel furniture.
    Geographic(super::GeographicCoordinate),
}
/// Coordinate clipping controls geometry after statistical population processing.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CoordinateClip {
    /// Clip to the resolved panel boundary, including radial holes and partial sectors.
    #[default]
    On,
    /// Preserve overflow up to the enclosing figure clip.
    Off,
}
/// Cartesian reverse controls in the original positional channels, before an optional flip.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CoordinateReverse {
    /// Neither channel.
    #[default]
    None,
    /// Original horizontal channel.
    X,
    /// Original vertical channel.
    Y,
    /// Both channels.
    Both,
}
/// Optional exact view endpoints; missing endpoints retain trained values.
pub type CoordinateLimits = Option<[Option<ScaleValue>; 2]>;
/// Cartesian post-stat view controls.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct CartesianCoordinate {
    /// Original horizontal channel view; never filters source rows.
    pub xlim: CoordinateLimits,
    /// Original vertical channel view; never filters source rows.
    pub ylim: CoordinateLimits,
    /// Preserve scale expansion on top/right/bottom/left, respectively.
    pub expand: [bool; 4],
    /// Reverse before a possible channel flip.
    pub reverse: CoordinateReverse,
    /// Swap horizontal and vertical positional channels and their guides.
    pub flip: bool,
    /// Physical length of one y unit divided by one x unit; None leaves aspect unconstrained.
    pub ratio: Option<f64>,
    /// Post-stat clipping policy.
    pub clip: CoordinateClip,
}
impl Default for CartesianCoordinate {
    fn default() -> Self {
        Self {
            xlim: None,
            ylim: None,
            expand: [true; 4],
            reverse: CoordinateReverse::None,
            flip: false,
            ratio: None,
            clip: CoordinateClip::On,
        }
    }
}
/// Existing transform descriptors applied in coordinate space, after scale/stat operations.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct TransformedCoordinate {
    /// Original x channel transform.
    pub x: GgplotTransform,
    /// Original y channel transform.
    pub y: GgplotTransform,
    /// View, expansion, reverse, flip and clip controls.
    pub view: CartesianCoordinate,
}
impl Default for TransformedCoordinate {
    fn default() -> Self {
        Self {
            x: GgplotTransform::Identity,
            y: GgplotTransform::Identity,
            view: Default::default(),
        }
    }
}
/// Source compatibility policy for the shared radial projection.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadialMode {
    /// Legacy ggplot polar panel defaults, including clipping on.
    Polar,
    /// ggplot radial defaults, including clipping off and partial-circle furniture.
    #[default]
    Radial,
}
/// Positional channel mapped to angle; the other channel maps to radius.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThetaAxis {
    /// Original x channel.
    #[default]
    X,
    /// Original y channel.
    Y,
}
/// Independent angular/radial reversal.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadialReverse {
    /// Neither dimension.
    #[default]
    None,
    /// Angular direction.
    Theta,
    /// Radial direction.
    Radius,
    /// Both dimensions.
    Both,
}
/// Placement of the radial primary and optional secondary axes.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum RadialAxisPlacement {
    /// Place along the arc endpoints inside the panel.
    Inside,
    /// Place on eligible exterior cardinal directions.
    Outside,
    /// Source theta values for primary and secondary radial axes; one value is reused.
    At(Vec<ScaleValue>),
}
/// Full/partial radial coordinates and source-compatible polar behavior.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct RadialCoordinate {
    /// Select polar or radial default controls over the same geometry engine.
    pub mode: RadialMode,
    /// Angular channel.
    pub theta: ThetaAxis,
    /// Radians clockwise from the top before angular reversal.
    pub start: f64,
    /// Arc end; None means start plus one full turn.
    pub end: Option<f64>,
    /// Angular-channel post-stat view.
    pub thetalim: CoordinateLimits,
    /// Radial-channel post-stat view.
    pub rlim: CoordinateLimits,
    /// Preserve source expansion for both angular/radial ends.
    pub expand: bool,
    /// Override the source-mode clipping default.
    pub clip: Option<CoordinateClip>,
    /// Inner-to-outer radius ratio in `[0,1]`, including the degenerate circle at one.
    pub inner_radius: f64,
    /// Independent angular/radial reversal.
    pub reverse: RadialReverse,
    /// Polar direction, exactly -1 or 1; radial uses reverse instead.
    pub direction: i8,
    /// None selects source full/partial-circle axis placement.
    pub radial_axis: Option<RadialAxisPlacement>,
    /// Rotate mapped text angles with angular position using source upright rules.
    pub rotate_angle: bool,
}
impl Default for RadialCoordinate {
    fn default() -> Self {
        Self {
            mode: RadialMode::Radial,
            theta: ThetaAxis::X,
            start: 0.,
            end: None,
            thetalim: None,
            rlim: None,
            expand: true,
            clip: None,
            inner_radius: 0.,
            reverse: RadialReverse::None,
            direction: 1,
            radial_axis: None,
            rotate_angle: false,
        }
    }
}

impl CoordinateSpec {
    pub(crate) fn validate(&self) -> crate::ChartResult<()> {
        let reject = |message: &str| {
            crate::Diagnostic::error(
                crate::DiagnosticCode::Validation,
                message,
                "Correct the coordinate descriptor; coordinate limits do not change source/statistical populations.",
            )
        };
        let limits = |values: &CoordinateLimits| -> crate::ChartResult<()> {
            if let Some(values) = values {
                for value in values.iter().flatten() {
                    if matches!(value,ScaleValue::Number(v) if v.is_nan()) {
                        return Err(reject("Coordinate view endpoints cannot be NaN."));
                    }
                }
            }
            Ok(())
        };
        let cartesian = |v: &CartesianCoordinate| -> crate::ChartResult<()> {
            limits(&v.xlim)?;
            limits(&v.ylim)?;
            if v.ratio.is_some_and(|r| !r.is_finite() || r <= 0.) {
                return Err(reject(
                    "Fixed coordinate ratio must be finite and positive.",
                ));
            }
            Ok(())
        };
        match self {
            Self::Cartesian(v) => cartesian(v),
            Self::Geographic(v) => v.validate(),
            Self::Transformed(v) => {
                cartesian(&v.view)?;
                v.x.validate_authoring()?;
                v.y.validate_authoring()
            }
            Self::Radial(v) => {
                if v.mode == RadialMode::Polar
                    && (v.end.is_some()
                        || v.inner_radius != 0.
                        || v.reverse != RadialReverse::None
                        || v.radial_axis.is_some()
                        || v.rotate_angle
                        || v.thetalim.is_some()
                        || v.rlim.is_some())
                {
                    return Err(reject(
                        "Polar coordinates use start and direction; partial arcs, holes, coordinate limits, angle rotation and radial guide placement require Radial mode.",
                    ));
                }
                if v.mode == RadialMode::Radial && v.direction != 1 {
                    return Err(reject(
                        "Radial direction is expressed by reverse; direction is a Polar control.",
                    ));
                }
                limits(&v.thetalim)?;
                limits(&v.rlim)?;
                if !v.start.is_finite()
                    || v.end.is_some_and(|x| !x.is_finite())
                    || !v.inner_radius.is_finite()
                    || !(0. ..=1.).contains(&v.inner_radius)
                    || ![-1, 1].contains(&v.direction)
                {
                    return Err(reject(
                        "Radial start/end must be finite, inner_radius must be in [0,1], and polar direction must be -1 or 1.",
                    ));
                }
                if let Some(RadialAxisPlacement::At(values)) = &v.radial_axis {
                    if !(1..=2).contains(&values.len()) {
                        return Err(reject(
                            "Radial axis placement requires one or two theta values.",
                        ));
                    }
                    if values
                        .iter()
                        .any(|v| matches!(v,ScaleValue::Number(x) if !x.is_finite()))
                    {
                        return Err(reject("Radial axis placement values must be finite."));
                    }
                }
                Ok(())
            }
        }
    }
}
