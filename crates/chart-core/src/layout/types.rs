use super::{GuideFormatter, GuideProfile};
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
    /// D3-compatible authored numeric knots and range, projected into destination units.
    Numeric(crate::scales::NumericScaleSpec),
    /// Explicitly registered numeric-output provider; never implies inversion.
    Registered {
        /// Exact installed provider identity and version.
        operation: crate::grammar::OperationRef,
        /// Bounded declarative provider inputs.
        parameters: serde_json::Value,
    },
    /// Invertible nonlinear numeric mapping with data-space domain policies.
    Nonlinear {
        /// Coordinate transformation.
        transform: ScaleTransform,
        /// Domain policy applied in transformed coordinates.
        domain: ContinuousDomain,
    },
    /// Categorical centers without band extents.
    Point(PointOptions),
    /// D3 point spacing with explicit alignment and rounding.
    D3Point(crate::scales::PointSpec),
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
    /// D3 band spacing with explicit alignment, rounding and zero-width support.
    D3Band(crate::scales::BandSpec),
    /// Integer-origin time knots with a supplied UTC/local calendar.
    Calendar {
        /// Authored timestamp knots and numerical outputs.
        spec: crate::scales::TimeScaleSpec,
        /// Explicit calendar interval, or automatic density selection.
        interval: Option<crate::scales::CalendarInterval>,
    },
    /// UTC integer domain and optional calendar tick interval.
    Utc {
        /// Exact source-unit endpoints; absent derives post-stat endpoints.
        domain: Option<TimeBounds>,
        /// Calendar interval; absent selects from the visible interval.
        interval: Option<UtcInterval>,
    },
}
/// Presentation shared by default and independently identified positional guides.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct GuideStyle {
    /// Explicit guide presentation policy, independent of scale/population semantics (wire v11).
    #[serde(default, skip_serializing_if = "GuideProfile::is_legacy")]
    pub profile: GuideProfile,
    /// Per-guide count/interval/format hints; absent uses the selected profile defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_arguments: Option<crate::scales::GuideTickArguments>,
    /// Independent typed values; None selects automatically, Some([]) selects no ticks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_values: Option<Vec<crate::composition::ScaleValue>>,
    /// Independent formatter override; None restores the scale formatter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tick_format: Option<GuideFormatter>,
    /// Explicit bounded semantic tick positions/labels, replacing automatic guide candidates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guide_ticks: Option<Vec<CustomGuideTick>>,
    /// Optional portable numeric formatting; incompatible category/time guides reject it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub number_format: Option<crate::typography::NumberFormat>,
    /// D3 numeric specifier with inferred tick precision and an explicit locale (wire v5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_format: Option<crate::typography::NumericFormat>,
    /// Conditional or custom time labels using the axis calendar (wire v5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_format: Option<crate::scales::TimeFormat>,
    /// Optional explicit rich axis title, measured with this destination's text service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<crate::typography::RichText>,
    /// Optional rich tick style; its text is replaced by each logical tick label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typography: Option<crate::typography::RichRun>,
    /// Clockwise tick-label rotation in degrees; uses exact shaped outlines when nonzero.
    #[serde(default)]
    pub label_rotation: f64,
    /// Whether to paint and reserve margin for this guide.
    pub visible: bool,
}
impl GuideStyle {
    /// Whether this guide uses the independent value/formatter contract (wire v11).
    pub fn uses_tick_configuration(&self) -> bool {
        self.profile != GuideProfile::LibraryV1
            || self.tick_arguments.is_some()
            || self.tick_values.is_some()
            || self.tick_format.is_some()
    }
}
impl Default for GuideStyle {
    fn default() -> Self {
        Self {
            profile: GuideProfile::LibraryV1,
            tick_arguments: None,
            tick_values: None,
            tick_format: None,
            guide_ticks: None,
            number_format: None,
            numeric_format: None,
            time_format: None,
            title: None,
            typography: None,
            label_rotation: 0.,
            visible: true,
        }
    }
}
/// One independently identified guide over an already declared positional scale.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GuideSpec {
    /// Stable guide identity, independent of the referenced scale identity.
    pub id: crate::GuideId,
    /// Existing scale to reuse without retraining or altering mark coordinates.
    pub scale: ScaleId,
    /// Guide side; orientation must match the referenced positional scale.
    pub side: AxisSide,
    /// Explicit finite translation in destination units, applied only to this guide.
    #[serde(default)]
    pub translation: [f64; 2],
    /// Shared presentation configuration.
    #[serde(flatten)]
    pub style: GuideStyle,
}
impl GuideSpec {
    /// A visible guide at its side's plot edge with legacy presentation defaults.
    pub fn new(id: crate::GuideId, scale: ScaleId, side: AxisSide) -> Self {
        Self {
            id,
            scale,
            side,
            translation: [0., 0.],
            style: GuideStyle::default(),
        }
    }
}
impl std::ops::Deref for GuideSpec {
    type Target = GuideStyle;
    fn deref(&self) -> &Self::Target {
        &self.style
    }
}
impl std::ops::DerefMut for GuideSpec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.style
    }
}
impl std::ops::Deref for AxisSpec {
    type Target = GuideStyle;
    fn deref(&self) -> &Self::Target {
        &self.guide
    }
}
impl std::ops::DerefMut for AxisSpec {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guide
    }
}
/// One named positional scale and its default guide.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AxisSpec {
    /// Explicit coordinate-only transform override; absence follows the canonical profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_stage: Option<crate::grammar::ScaleStage>,
    /// Population handling under pre-stat scale semantics, separate from viewport clipping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub population_oob: Option<crate::grammar::ScaleOob>,
    /// Presentation of the default guide; the scale has independent ownership.
    #[serde(flatten)]
    pub guide: GuideStyle,
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
}
impl AxisSpec {
    /// Default guide identity and presentation, preserving historical scale identity values.
    pub fn default_guide(&self) -> GuideSpec {
        GuideSpec {
            id: crate::GuideId::new(self.id.get()),
            scale: self.id,
            side: self.side,
            translation: [0., 0.],
            style: self.guide.clone(),
        }
    }
    /// Automatic family, plot range, visible guide and finite extrapolation for clipping.
    pub fn new(id: ScaleId, side: AxisSide) -> Self {
        Self {
            id,
            population_oob: None,
            scale_stage: None,
            side,
            guide: GuideStyle::default(),
            scale: AxisScale::Auto,
            viewport: None,
            range: None,
            outside: OutsidePolicy::Extend,
        }
    }
}
/// Explicit bounded layout request; all dimensions share the destination units.
#[derive(serde::Serialize, Clone, Debug)]
pub struct LayoutRequest {
    /// Host tokens, preceding named theme and plot overrides.
    pub host_theme: crate::theme::ThemePatch<crate::color::Paint>,
    /// Explicit applicable interaction styling by layer.
    pub interaction_theme: BTreeMap<crate::LayerId, crate::theme::ThemePatch<crate::color::Paint>>,
    /// Destination output overrides, separate from the authored theme.
    pub output_theme: crate::theme::ThemePatch<crate::color::Paint>,
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
    /// At most four independent named positional scales and their default guides.
    pub axes: Vec<AxisSpec>,
    /// Additional independent guides over existing positional scales.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub guides: Vec<GuideSpec>,
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
            guides: vec![],
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
    /// One checked immutable provider shared by all guides on this scale.
    Provider(crate::scales::CheckedPositionalScale),
    /// Numeric mapping/inversion in calculation space.
    Linear(LinearScale),
    /// Retained piecewise and transformed numeric domain.
    Numeric(crate::scales::NumericAxisScale),
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
    /// Shared calendar and retained numeric timestamp mapping.
    Calendar(Box<crate::scales::TimeAxisScale>),
}
/// Logical label and final destination coordinate retained for host inspection.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GuideTick {
    /// Original typed semantic value, retained without inverse reconstruction.
    pub value: crate::composition::ScaleValue,
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
    /// Retained ticks after label thinning; suppressed log minor labels are empty.
    pub ticks: Vec<GuideTick>,
}
/// One retained guide referencing an independently owned positional scale.
#[derive(Clone, Debug)]
pub struct ResolvedGuide {
    /// Stable identity, shared scale, placement and presentation.
    pub spec: GuideSpec,
    /// Original semantic values and final destination positions/labels.
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
    pub(crate) paint_themes: BTreeMap<crate::LayerId, crate::theme::ThemePatch>,
    pub(crate) interactions: BTreeMap<usize, crate::grammar::GeometryInteraction>,
    pub(crate) insets: Vec<LaidOutInset>,
    pub(crate) panels: Vec<LaidOutPanel>,
    pub(crate) item_panels: Vec<Option<crate::grammar::PanelKey>>,
    pub(crate) prepared: Arc<PreparedChart>,
    pub(crate) scene: Scene,
    pub(crate) plot: Option<Rect>,
    pub(crate) axes: BTreeMap<ScaleId, ResolvedAxis>,
    pub(crate) guides: BTreeMap<crate::GuideId, ResolvedGuide>,
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
    /// Owned semantic guide snapshots from this exact layout, including facet/inset scopes.
    /// Values and labels are captured together; no inverse mapping or relayout occurs.
    pub fn guide_snapshots(&self) -> Vec<super::GuideSnapshot> {
        fn visit(
            chart: &LaidOutChart,
            scope: &mut Vec<super::GuideScope>,
            out: &mut Vec<super::GuideSnapshot>,
        ) {
            out.extend(chart.guides.values().map(|guide| super::GuideSnapshot {
                scope: scope.clone(),
                spec: guide.spec.clone(),
                ticks: guide.ticks.clone(),
            }));
            for panel in &chart.panels {
                scope.push(super::GuideScope::Panel(panel.key.clone()));
                visit(&panel.chart, scope, out);
                scope.pop();
            }
            for inset in &chart.insets {
                scope.push(super::GuideScope::Inset(inset.id.clone()));
                visit(&inset.chart, scope, out);
                scope.pop();
            }
        }
        let mut snapshots = Vec::new();
        visit(self, &mut Vec::new(), &mut snapshots);
        snapshots
    }
    /// Independently identified default and additional guides over retained scales.
    pub fn guides(&self) -> &BTreeMap<crate::GuideId, ResolvedGuide> {
        &self.guides
    }
    /// Resolved per-layer paint cascade captured with this immutable destination.
    pub fn paint_theme(&self, layer: crate::LayerId) -> Option<&crate::theme::ThemePatch> {
        self.paint_themes.get(&layer)
    }
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
    /// Resolve an annotation coordinate through the exact layout projection and clipping policy.
    pub fn project_anchor(
        &self,
        anchor: &crate::composition::Anchor,
    ) -> crate::ChartResult<Option<(crate::Point, Rect)>> {
        super::composition::anchor(self, anchor, self.scene.bounds())
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
            (ResolvedScale::Provider(s), value) => s.map(value),
            (ResolvedScale::Linear(s), crate::composition::ScaleValue::Number(v)) => s.map(*v),
            (ResolvedScale::Numeric(s), crate::composition::ScaleValue::Number(v)) => s.map(*v),
            (ResolvedScale::Nonlinear(s), crate::composition::ScaleValue::Number(v)) => s.map(*v),
            (ResolvedScale::Band(s), crate::composition::ScaleValue::Category(v)) => s.center(v),
            (ResolvedScale::Point(s), crate::composition::ScaleValue::Category(v)) => s.center(v),
            (ResolvedScale::Utc(s), crate::composition::ScaleValue::Timestamp { value, unit })
                if s.unit() == *unit =>
            {
                s.map(*value)
            }
            (
                ResolvedScale::Calendar(s),
                crate::composition::ScaleValue::Timestamp { value, unit },
            ) if s.unit() == *unit => s.map(*value),
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
