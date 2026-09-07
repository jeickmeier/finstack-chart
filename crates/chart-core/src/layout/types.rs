use crate::grammar::{PreparedChart, ValueSpace};
use crate::provenance::Target;
use crate::scales::{
    BandOptions, BandScale, Bounds, ContinuousDomain, LinearScale, NonlinearScale, OutsidePolicy,
    PointOptions, PointScale, ScaleTransform, SessionCalendar, SessionScale, TimeBounds,
    UtcInterval, UtcScale,
};
use crate::scene::Scene;
use crate::services::{ResourceDescriptor, Units};
use crate::{Diagnostic, Limits, Rect, Revision, ScaleId};
use std::{collections::BTreeMap, sync::Arc};

/// Hard cap on axis measurement/layout passes (plus at most one compact-state label).
pub const MAX_LAYOUT_PASSES: usize = 4;
/// One independent guide per side; secondary unit mappings are a separate future construct.
#[derive(
    serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd,
)]
#[serde(deny_unknown_fields)]
pub enum AxisSide {
    /// Horizontal lower guide.
    Bottom,
    /// Vertical left guide.
    Left,
    /// Independent horizontal upper guide.
    Top,
    /// Independent vertical right guide.
    Right,
}
impl AxisSide {
    /// Whether the bound scale is horizontal.
    pub fn horizontal(self) -> bool {
        matches!(self, Self::Bottom | Self::Top)
    }
}
/// Declared family and training policy; automatic selection follows the encoded value space.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum AxisScale {
    /// Linear numeric, band categorical, or UTC timestamp with automatic ticks.
    #[default]
    Auto,
    /// Linear domain precedence/baseline/padding/nice policy.
    Linear(ContinuousDomain),
    /// Invertible nonlinear numeric mapping with data-space domain policies.
    Nonlinear {
        /// Coordinate transformation.
        transform: ScaleTransform,
        /// Domain policy applied in transformed coordinates.
        domain: ContinuousDomain,
    },
    /// Categorical centers without band extents.
    Point(PointOptions),
    /// Supplied active sessions compressed into contiguous time.
    Session(SessionCalendar),
    /// Alternate-unit guide over another numeric axis; cannot bind layer coordinates.
    Secondary {
        /// Existing primary numeric scale identity.
        source: ScaleId,
        /// Finite nonzero unit multiplier.
        factor: f64,
        /// Finite unit offset.
        offset: f64,
    },
    /// Stable labels with optional exact explicit domain/order.
    Band(BandOptions),
    /// UTC integer domain and optional calendar tick interval.
    Utc {
        /// Exact source-unit endpoints; absent derives post-stat endpoints.
        domain: Option<TimeBounds>,
        /// Calendar interval; absent selects from the visible interval.
        interval: Option<UtcInterval>,
    },
}
/// One named positional scale and optional plain guide.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AxisSpec {
    /// Explicit bounded semantic tick positions/labels, replacing automatic guide candidates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guide_ticks: Option<Vec<CustomGuideTick>>,
    /// Optional portable numeric formatting; incompatible category/time guides reject it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_format: Option<crate::typography::NumberFormat>,
    /// Optional explicit rich axis title, measured with this destination's text service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<crate::typography::RichText>,
    /// Optional rich tick style; its text is replaced by each logical tick label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typography: Option<crate::typography::RichRun>,
    /// Clockwise tick-label rotation in degrees; uses exact shaped outlines when nonzero.
    #[serde(default)]
    pub label_rotation: f64,
    /// Must match a layer scale binding (or an empty primary guide).
    pub id: ScaleId,
    /// Guide side and scale orientation.
    pub side: AxisSide,
    /// Domain family and policies.
    pub scale: AxisScale,
    /// Visible interval in layer calculation units, relative ticks for UTC.
    /// Overrides captured primary state viewport when supplied.
    pub viewport: Option<Bounds>,
    /// Exact destination range override; default is the oriented plot span.
    pub range: Option<Bounds>,
    /// Outside visible-domain treatment, independent of plot clipping.
    pub outside: OutsidePolicy,
    /// Whether to paint and reserve margin for this guide.
    pub visible: bool,
}
impl AxisSpec {
    /// Automatic family, plot range, visible guide and finite extrapolation for clipping.
    pub fn new(id: ScaleId, side: AxisSide) -> Self {
        Self {
            id,
            guide_ticks: None,
            side,
            typography: None,
            number_format: None,
            title: None,
            label_rotation: 0.,
            scale: AxisScale::Auto,
            viewport: None,
            range: None,
            outside: OutsidePolicy::Extend,
            visible: true,
        }
    }
}
/// Explicit bounded layout request; all dimensions share the destination units.
#[derive(Clone, Debug)]
pub struct LayoutRequest {
    /// Host tokens, preceding named theme and plot overrides.
    pub host_theme: crate::theme::ThemePatch,
    /// Explicit applicable interaction styling by layer.
    pub interaction_theme: BTreeMap<crate::LayerId, crate::theme::ThemePatch>,
    /// Destination output overrides, separate from the authored theme.
    pub output_theme: crate::theme::ThemePatch,
    /// Optional enclosing figure clip for layers explicitly allowing figure overflow.
    /// Facet layout supplies the outer figure; plot clips remain panel-local.
    pub figure_bounds: Option<Rect>,
    /// Finite destination figure.
    pub bounds: Rect,
    /// Destination measurement/painting convention.
    pub units: Units,
    /// Owner revision for bounds, font, profile or layout policy changes.
    pub revision: Revision,
    /// Exact font bytes identity/revision, supplied to measurement and scene painting.
    pub font: ResourceDescriptor,
    /// Positive plain-label size in destination units.
    pub font_size: f64,
    /// At most four independent named axes; at most one visible guide on each side.
    pub axes: Vec<AxisSpec>,
    /// Nonnegative figure inset.
    pub padding: f64,
    /// Minimum useful plot width and height, both positive.
    pub minimum_plot: (f64, f64),
    /// Nonnegative guide tick length.
    pub tick_length: f64,
    /// Nonnegative label/tick and inter-label separation.
    pub label_gap: f64,
    /// Desired numeric/time tick count, between two and 128.
    pub target_ticks: usize,
    /// Maximum generated/measured ticks per axis per pass, between two and 4096.
    pub max_ticks: usize,
    /// Bound catalog work before band resolution and label cloning.
    pub max_categories: usize,
    /// Bound projected geometry work before any destination callbacks.
    pub max_vertices: usize,
    /// Scene/resource/text limits, also enforced before measurement/geometry allocation.
    /// The text budget additionally bounds all trained and explicit category label bytes.
    pub limits: Limits,
}
impl LayoutRequest {
    /// Two automatic primary axes, explicit destination font, four-pass bounded solving.
    pub fn new(bounds: Rect, units: Units, font: ResourceDescriptor) -> Self {
        Self {
            bounds,
            host_theme: crate::theme::ThemePatch::default(),
            interaction_theme: BTreeMap::new(),
            output_theme: crate::theme::ThemePatch::default(),
            figure_bounds: None,
            units,
            font,
            revision: Revision::INITIAL,
            font_size: 12.,
            axes: vec![
                AxisSpec::new(ScaleId::new(0), AxisSide::Bottom),
                AxisSpec::new(ScaleId::new(1), AxisSide::Left),
            ],
            padding: 8.,
            minimum_plot: (24., 24.),
            tick_length: 4.,
            label_gap: 4.,
            target_ticks: 6,
            max_ticks: 128,
            max_categories: 10_000,
            max_vertices: 1_000_000,
            limits: Limits::default(),
        }
    }
}
/// Concrete scale exposes only its valid inversion/category capabilities.
#[derive(Clone, Debug)]
pub enum ResolvedScale {
    /// Numeric mapping/inversion in calculation space.
    Linear(LinearScale),
    /// Numeric nonlinear mapping/inversion.
    Nonlinear(NonlinearScale),
    /// Category centers without band widths.
    Point(PointScale),
    /// Supplied active-session timestamp mapping.
    Session(SessionScale),
    /// Guide-only alternate units over a primary numeric domain.
    Secondary {
        /// Source scale identity.
        source: ScaleId,
        /// Alternate-unit represented domain.
        domain: Bounds,
    },
    /// Exact label lookup and band extent; no numeric inverse.
    Band(BandScale),
    /// Exact source timestamp mapping/inversion.
    Utc(UtcScale),
}
/// Logical label and final destination coordinate retained for host inspection.
#[derive(Clone, Debug, PartialEq)]
pub struct GuideTick {
    /// Destination coordinate along the axis.
    pub position: f64,
    /// Preserved logical label; deterministic thinning can omit other candidates.
    pub label: String,
}
/// Resolved transform, domain interpretation and guide shared by destinations.
#[derive(Clone, Debug)]
pub struct ResolvedAxis {
    /// Authored guide and scale identity.
    pub spec: AxisSpec,
    /// Trained semantic value space, before viewport filtering.
    pub space: ValueSpace,
    /// Concrete checked transform and capabilities.
    pub scale: ResolvedScale,
    /// Final visible labels after deterministic thinning.
    pub ticks: Vec<GuideTick>,
}
/// Explicit compact outcomes; no zero-width scale or invalid geometry is fabricated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutStatus {
    /// Useful plot with visible data marks.
    Ready,
    /// Useful plot without visible data marks.
    NoData,
    /// Bounds/metrics cannot accommodate a minimum useful plot.
    NoSpace,
}
/// One coherent immutable layout, retaining the exact prepared/stat/source snapshot.
#[derive(Clone, Debug)]
pub struct LaidOutChart {
    pub(crate) interactions: BTreeMap<usize, crate::grammar::GeometryInteraction>,
    pub(crate) insets: Vec<LaidOutInset>,
    pub(crate) panels: Vec<LaidOutPanel>,
    pub(crate) item_panels: Vec<Option<crate::grammar::PanelKey>>,
    pub(crate) prepared: Arc<PreparedChart>,
    pub(crate) scene: Scene,
    pub(crate) plot: Option<Rect>,
    pub(crate) axes: BTreeMap<ScaleId, ResolvedAxis>,
    pub(crate) targets: Vec<Vec<Target>>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) status: LayoutStatus,
    pub(crate) passes: usize,
}
/// One facet destination retaining exact panel bounds, scales and source preparation.
#[derive(Clone, Debug)]
pub struct LaidOutPanel {
    /// Stable semantic identity.
    pub key: crate::grammar::PanelKey,
    /// Authored row position.
    pub row: usize,
    /// Authored column position.
    pub column: usize,
    /// Full panel cell, including the facet header.
    pub bounds: Rect,
    /// Resolved panel scene with common figure coordinates.
    pub chart: Arc<LaidOutChart>,
}
/// Alternate prepared-data view with its own coherent ranges and destination scene.
#[derive(Clone, Debug)]
pub struct LaidOutInset {
    /// Authored inset view identity.
    pub id: String,
    /// Parent facet, absent for a single-panel figure.
    pub panel: Option<crate::grammar::PanelKey>,
    /// Full inset destination bounds.
    pub bounds: Rect,
    /// Exact inset scene/scales and reused prepared data.
    pub chart: Arc<LaidOutChart>,
}
impl LaidOutChart {
    /// Explicit custom hit/semantic/selection/keyboard metadata by final scene item index.
    pub fn interactions(&self) -> &BTreeMap<usize, crate::grammar::GeometryInteraction> {
        &self.interactions
    }
    /// Alternate prepared-data views, in inset paint order.
    pub fn insets(&self) -> &[LaidOutInset] {
        &self.insets
    }

    /// Resolved facets in explicit order; empty on a single-panel chart.
    pub fn panels(&self) -> &[LaidOutPanel] {
        &self.panels
    }
    /// Panel identity for every scene item; figure furniture has no panel.
    pub fn item_panels(&self) -> &[Option<crate::grammar::PanelKey>] {
        &self.item_panels
    }
    /// Exact immutable prepared source/stat inputs; layout never reruns them.
    pub fn prepared(&self) -> &Arc<PreparedChart> {
        &self.prepared
    }
    /// Finite destination scene with explicit plot/figure clips.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    /// Useful plot, absent for the compact no-space state.
    pub fn plot(&self) -> Option<Rect> {
        self.plot
    }
    /// Named scales and final plain guides.
    pub fn axes(&self) -> &BTreeMap<ScaleId, ResolvedAxis> {
        &self.axes
    }
    /// One entry per scene item; decorations are empty, line targets follow vertices.
    pub fn targets(&self) -> &[Vec<Target>] {
        &self.targets
    }
    /// Aggregate layout pressure/omission diagnostics plus preparation diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    /// Explicit usable/empty/compact outcome.
    pub fn status(&self) -> LayoutStatus {
        self.status
    }
    /// Actual destination measurement passes, never above the documented cap.
    pub fn passes(&self) -> usize {
        self.passes
    }
}

/// Authored guide label positioned by the same checked scale as its data geometry.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CustomGuideTick {
    /// Numeric calculation value, exact category, or exact timestamp with unit.
    pub value: crate::composition::ScaleValue,
    /// Logical label; no host formatter closure is serialized.
    pub label: String,
}

impl ResolvedAxis {
    /// Map a semantic value through its actual numeric/category/time capability.
    pub fn map_value(&self, v: &crate::composition::ScaleValue) -> crate::ChartResult<Option<f64>> {
        match (&self.scale, v) {
            (ResolvedScale::Linear(s), crate::composition::ScaleValue::Number(v)) => s.map(*v),
            (ResolvedScale::Nonlinear(s), crate::composition::ScaleValue::Number(v)) => s.map(*v),
            (ResolvedScale::Band(s), crate::composition::ScaleValue::Category(v)) => s.center(v),
            (ResolvedScale::Point(s), crate::composition::ScaleValue::Category(v)) => s.center(v),
            (ResolvedScale::Utc(s), crate::composition::ScaleValue::Timestamp { value, unit })
                if s.unit() == *unit =>
            {
                s.map(*value)
            }
            (
                ResolvedScale::Session(s),
                crate::composition::ScaleValue::Timestamp { value, unit },
            ) if s.calendar().unit == *unit => s.map(*value),
            _ => Err(crate::scales::error(
                crate::DiagnosticCode::SchemaConflict,
                "Value disagrees with its named scale family or timestamp unit.",
            )),
        }
    }
}
