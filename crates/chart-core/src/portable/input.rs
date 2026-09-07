//! Bounded input query DTOs; decoding never bypasses finite coordinate constructors.
use crate::grammar::PanelKey;
use crate::inspection::{InspectionMode, SelectionRegion};
use crate::navigation::{Navigation, NavigationBoundary};
use crate::state::AxisWindow;
use crate::{ChartResult, Point, Rect, ScaleId, SceneStamp};
use serde::Deserialize;
/// Pure input query fenced to the caller's actual presented or pinned gesture scene.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputQuery {
    /// Exact scene stamp.
    pub scene: SceneStamp,
    /// True uses the active gesture basis; false uses the currently presented scene.
    #[serde(default)]
    pub gesture: bool,
    /// Bounded query operation. Returned windows/targets become common reducer actions.
    pub query: InputOperation,
}
/// Portable input query families, with no host-specific computations.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub enum InputOperation {
    /// Bounded semantic table and focus descriptions, without native/interpreter objects.
    Describe {
        /// Starting meaningful target offset.
        offset: usize,
        /// Page size 1..256.
        limit: usize,
    },
    /// Constrain/snap an authored annotation over this presented or active gesture basis.
    EditAnnotation {
        /// Existing annotation identity.
        id: String,
        /// Editable endpoint or joint movement.
        part: crate::editing::AnnotationPart,
        /// Typed movement/snap/range constraints.
        constraints: crate::editing::EditConstraints,
        /// Total destination displacement since gesture begin.
        delta: [f64; 2],
    },
    /// Capture a root-origin message after an effective event; linked echoes produce null.
    LinkCapture {
        /// Stable application view name.
        origin: String,
        /// Event returned by common dispatch.
        event: crate::state::StateEvent,
        /// Sender axes, empty for selection-only linking.
        axes: Vec<ScaleId>,
        /// Optional source facet.
        panel: Option<PanelKey>,
        /// Include the effective selection, including an explicit clear.
        selection: bool,
    },
    /// Resolve semantic links to a common receiving action without dispatching it.
    LinkResolve {
        /// Unmodified root-origin message.
        message: crate::linking::LinkMessage,
        /// Explicit sender/receiver axis pairing.
        mappings: Vec<crate::linking::AxisLink>,
        /// Optional receiving facet.
        panel: Option<PanelKey>,
        /// Missing-key policy with explicit unmatched reporting.
        missing: crate::linking::MissingMatch,
    },
    /// Read nearest-x/point/shape hits and exact semantic targets.
    Inspect {
        /// Destination coordinates.
        point: [f64; 2],
        /// Positive finite scatter radius.
        radius: f64,
        /// Group cap 1..128.
        max_grouped: usize,
        /// Explicit hit policy.
        mode: InspectionMode,
    },
    /// Return provenance-aware selection targets without altering state.
    Select {
        /// Geometry predicate in presented coordinates.
        region: SelectionWire,
        /// Result cap 1..4096; overflow rejects the full result.
        limit: usize,
    },
    /// Produce typed windows from a shared navigation operation.
    Navigate {
        /// Primary scale identities.
        axes: Vec<ScaleId>,
        /// Stable facet identity, absent for one panel.
        panel: Option<PanelKey>,
        /// Pointer/range navigation.
        action: NavigationWire,
        /// Explicit clamp/extend policy.
        boundary: NavigationBoundary,
    },
    /// Validate one explicit semantic window and return a complete window map.
    SetRange {
        /// Primary scale identity.
        axis: ScaleId,
        /// Stable facet identity.
        panel: Option<PanelKey>,
        /// Type-correct requested window.
        window: AxisWindow,
    },
}
/// Wire selection coordinates, checked before constructing native core geometry.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub enum SelectionWire {
    /// One destination point.
    Point([f64; 2]),
    /// All visible targets in a panel/layer.
    Series {
        /// Layer identity.
        layer: crate::LayerId,
        /// Panel identity.
        panel: Option<PanelKey>,
    },
    /// Horizontal interval.
    XRange(f64, f64),
    /// Vertical interval.
    YRange(f64, f64),
    /// Origin and width/height in destination units.
    Rectangle([f64; 4]),
    /// Closed polygon vertices (3..4096).
    Lasso(Vec<[f64; 2]>),
}
impl SelectionWire {
    pub(super) fn region(self) -> ChartResult<SelectionRegion> {
        Ok(match self {
            Self::Point(p) => SelectionRegion::Point(point(p)?),
            Self::Series { layer, panel } => SelectionRegion::Series { layer, panel },
            Self::XRange(a, b) => SelectionRegion::XRange(a, b),
            Self::YRange(a, b) => SelectionRegion::YRange(a, b),
            Self::Rectangle([x, y, w, h]) => SelectionRegion::Rectangle(Rect::new(x, y, w, h)?),
            Self::Lasso(p) => {
                if p.len() > 4096 {
                    return Err(super::error(
                        crate::DiagnosticCode::ResourceLimit,
                        "Lasso vertex limit is 4096.",
                    ));
                }
                SelectionRegion::Lasso(p.into_iter().map(point).collect::<ChartResult<_>>()?)
            }
        })
    }
}
/// Wire navigation with finite constructors and explicit operation semantics.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub enum NavigationWire {
    /// Pointer-anchored scale factor.
    Zoom {
        /// Destination point.
        anchor: [f64; 2],
        /// Positive zoom factor.
        factor: f64,
    },
    /// Destination displacement from the pinned gesture start.
    Pan {
        /// Horizontal displacement.
        dx: f64,
        /// Vertical displacement.
        dy: f64,
    },
    /// Zoom to a rectangle.
    Region {
        /// First corner.
        from: [f64; 2],
        /// Opposite corner.
        to: [f64; 2],
    },
    /// Restore trained domains for the requested axes.
    Reset,
}
impl NavigationWire {
    pub(super) fn navigation(self) -> ChartResult<Navigation> {
        Ok(match self {
            Self::Zoom { anchor, factor } => Navigation::Zoom {
                anchor: point(anchor)?,
                factor,
            },
            Self::Pan { dx, dy } => Navigation::Pan { dx, dy },
            Self::Region { from, to } => Navigation::Region {
                from: point(from)?,
                to: point(to)?,
            },
            Self::Reset => Navigation::Reset,
        })
    }
}
pub(super) fn point([x, y]: [f64; 2]) -> ChartResult<Point> {
    Point::new(x, y)
}
