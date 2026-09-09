use super::{
    ChartDefinition, Grouping, NumericTransform, OperationRef, SourceFilter, StatSpace, Style,
};
use crate::data::{DatasetVersion, Schema, SnapshotHandle, StoreSnapshot, TimestampType};
use crate::provenance::Target;
use crate::state::ChartState;
use crate::{Diagnostic, LayerId, Point, Revision, RowKey, SchemaVersion, TransformId};
use std::{collections::BTreeMap, sync::Arc};

/// Finite numeric group identity with total ordering; both zero signs identify one group.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct GroupNumber(f64);
impl TryFrom<f64> for GroupNumber {
    type Error = crate::Diagnostic;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(super::error(
                crate::DiagnosticCode::NumericalDomain,
                "Numeric groups must be finite.",
            ));
        }
        Ok(Self(if value == 0. { 0. } else { value }))
    }
}
impl From<GroupNumber> for f64 {
    fn from(value: GroupNumber) -> Self {
        value.0
    }
}
impl PartialEq for GroupNumber {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for GroupNumber {}
impl Ord for GroupNumber {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}
impl PartialOrd for GroupNumber {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Exact group values; labels/codes and numeric IDs are never narrowed to f64.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GroupValue {
    /// Whole population.
    All,
    /// Categorical or UTF-8 group label.
    Text(String),
    /// Signed integer group.
    Int(#[serde(with = "crate::portable::signed")] i64),
    /// Unsigned integer group.
    UInt(#[serde(with = "crate::portable::unsigned")] u64),
    /// Boolean group.
    Boolean(bool),
    /// Exact finite binary64 explicit group identity.
    Number(GroupNumber),
    /// Explicit missing component of an inferred/declared interaction.
    Missing,
    /// Ordered field/value identity; controls and rendered row order do not define groups.
    Interaction(Vec<(crate::FieldId, GroupValue)>),
}
impl GroupValue {
    /// Display label, distinct from the exact structural group identity.
    pub fn label(&self) -> String {
        match self {
            Self::All => "All".into(),
            Self::Text(s) => s.clone(),
            Self::Int(v) => v.to_string(),
            Self::UInt(v) => v.to_string(),
            Self::Boolean(v) => v.to_string(),
            Self::Number(v) => v.0.to_string(),
            Self::Missing => "NA".into(),
            Self::Interaction(values) => values
                .iter()
                .map(|(_, v)| v.label())
                .collect::<Vec<_>>()
                .join("."),
        }
    }
    /// Retained component of an interaction, independent of rendered row/control order.
    pub fn component(&self, field: crate::FieldId) -> Option<&Self> {
        if let Self::Interaction(values) = self {
            values.iter().find(|(f, _)| *f == field).map(|(_, v)| v)
        } else {
            None
        }
    }
}

/// Declared axis calculation space, preventing silent double transforms or mixed origins.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum ValueSpace {
    /// Positional scale already evaluated before this coordinate was produced.
    Scaled {
        /// Prior source representation.
        input: Box<ValueSpace>,
        /// Exact scale stage applied once by the shared compiler.
        scale: super::ScaleProjection,
    },
    /// Stable label catalog; geometry stores a checked ordinal into this layer catalog.
    Categorical {
        /// Labels in retained source order; never dictionary codes or source identities.
        categories: Vec<String>,
    },
    /// Numeric source units or a generated count.
    Data,
    /// Relative ticks, preserving the exact source timestamp representation and origin.
    Timestamp {
        /// Source unit/timezone.
        representation: TimestampType,
        /// Checked integer origin.
        #[serde(with = "crate::portable::signed")]
        origin: i64,
    },
    /// Explicit pre-stat transform; interval coordinates already occupy this space.
    Transformed {
        /// Input representation, including timestamp origin when relevant.
        input: Box<ValueSpace>,
        /// Exact operation and parameters.
        transform: NumericTransform,
    },
}

/// Exact generated schema field kind.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneratedKind {
    /// Exact group label with an explicit categorical catalog.
    Categorical,
    /// Finite binary64 endpoint.
    Float64,
    /// Exact unsigned membership count.
    UInt64,
}

/// Generated schema descriptor, separate from any original source schema.
#[derive(serde::Serialize, Clone, Debug, Eq, PartialEq)]
pub struct GeneratedField {
    /// Generated accessor identity.
    pub field: super::BinField,
    /// Portable physical kind.
    pub kind: GeneratedKind,
}

/// Prepared output schema; source accessors and generated accessors are disjoint.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum OutputSchema {
    /// Registered custom output identity and its separate generated field schema.
    Custom {
        /// Exact schema/operation identity and version.
        operation: OperationRef,
        /// Checked generated fields, separate from original observations.
        fields: Vec<super::StatColumn>,
    },
    /// Count/summary/model schema with per-field nullability and calculation space.
    Statistical {
        /// Schema definition version.
        version: SchemaVersion,
        /// Exact generated field descriptors.
        fields: Vec<super::StatColumn>,
    },
    /// Original source fields, preserved by identity.
    Source(Arc<Schema>),
    /// Versioned explicit-bin output: start/end/midpoint f64 and count u64.
    Binned {
        /// Schema definition version.
        version: SchemaVersion,
        /// Declared generated fields.
        fields: Vec<GeneratedField>,
    },
}

/// An identity-stat row references its owning immutable source snapshot.
#[derive(serde::Serialize, Clone, Debug, Eq, PartialEq)]
pub struct SourceRow {
    /// Exact source key.
    pub key: RowKey,
    /// Stable insertion ordinal, independent of authored order.
    #[serde(with = "crate::portable::unsigned")]
    pub ordinal: u64,
}

/// Typed bin output; no source-row accessor receives this generated row.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct BinnedRow {
    /// Left edge in the declared output space.
    pub start: f64,
    /// Right edge, final edge included by the bin computation.
    pub end: f64,
    /// Exact number of source members.
    #[serde(with = "crate::portable::unsigned")]
    pub count: u64,
    /// Declared source group.
    pub group: GroupValue,
    /// Aggregate identity and complete compact source membership.
    pub target: Target,
}

/// Physically distinct source and generated row vectors.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum PreparedRows {
    /// Count, summary or fitted-model rows.
    Statistical(Arc<[super::StatisticalRow]>),
    /// Source-key references, not cloned original rows or type-erased callbacks.
    Source(Arc<[SourceRow]>),
    /// Typed generated rows with explicit aggregate provenance.
    Binned(Arc<[BinnedRow]>),
}
impl PreparedRows {
    /// Prepared output row count.
    pub fn len(&self) -> usize {
        match self {
            Self::Statistical(rows) => rows.len(),
            Self::Source(rows) => rows.len(),
            Self::Binned(rows) => rows.len(),
        }
    }
    /// Whether this prepared output has no rows.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Per-stage population accounting; counts describe this operation, not the whole graph.
#[derive(serde::Serialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct PopulationCounts {
    /// Rows received from the dependency.
    pub input: usize,
    /// Finite rows rejected by explicit source filter bounds.
    pub filtered: usize,
    /// Null/non-finite/imprecise required filter inputs.
    pub invalid_filter: usize,
    /// Null/non-finite/imprecise statistic/group inputs.
    pub invalid_stat: usize,
    /// Finite inputs below the first explicit bin edge.
    pub below: usize,
    /// Finite inputs above the last explicit bin edge.
    pub above: usize,
    /// Rows produced by the stat (including empty explicit bins).
    pub output: usize,
}

/// Auditable stat invocation with exact update capability and population accounting.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct OperationRecord {
    /// Explicit grouped, per-facet or whole-chart statistical population.
    pub scope: super::StatScope,
    /// Matched panel population, absent for chart-wide or broadcast inputs.
    pub panel: Option<super::PanelKey>,
    /// Registered operation identity/version.
    pub operation: OperationRef,
    /// Exact builtin parameters, including explicit edges and numeric input mapping.
    pub parameters: super::StatParameters,
    /// Immutable source input revision/schema.
    pub input: DatasetVersion,
    /// Explicit ordered filters; viewport/highlighting never appears here.
    pub filters: Vec<SourceFilter>,
    /// Grouping defines the computation scope.
    pub grouping: Grouping,
    /// Statistical calculation-space policy.
    pub space: StatSpace,
    /// Exact specialized update support and declared full-recompute fallback.
    pub incremental: super::IncrementalCapabilities,
    /// Population accounting for this stage.
    pub counts: PopulationCounts,
}

/// Immutable prepared output, shareable by composed layers and successive chart preparations.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedTable {
    pub(crate) schema: OutputSchema,
    pub(crate) rows: PreparedRows,
    pub(crate) input: DatasetVersion,
    pub(crate) space: ValueSpace,
    pub(crate) operations: Vec<OperationRecord>,
}
impl PreparedTable {
    pub(crate) fn work_units(&self) -> usize {
        let fields = match &self.schema {
            OutputSchema::Statistical { fields, .. } | OutputSchema::Custom { fields, .. } => {
                fields.len()
            }
            _ => 1,
        };
        self.rows.len().saturating_mul(fields)
    }
    /// Explicit original/generated schema.
    pub fn schema(&self) -> &OutputSchema {
        &self.schema
    }
    /// Read-only typed prepared rows.
    pub fn rows(&self) -> &PreparedRows {
        &self.rows
    }
    /// Exact source input scope used by all target resolvers.
    pub fn input(&self) -> DatasetVersion {
        self.input
    }
    /// Numeric space of generated bin edges; source fields carry their own mapping space.
    pub fn space(&self) -> &ValueSpace {
        &self.space
    }
    /// Ordered operation metadata, including dependency operations.
    pub fn operations(&self) -> &[OperationRecord] {
        &self.operations
    }
}

/// Finite data-space domain extent. Empty axes have no extent; Scale resolution chooses documented fallbacks.
#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Extent {
    /// Minimum contributing endpoint.
    pub minimum: f64,
    /// Maximum contributing endpoint.
    pub maximum: f64,
}
impl Extent {
    pub(crate) fn include(target: &mut Option<Self>, value: f64) {
        if let Some(current) = target {
            current.minimum = current.minimum.min(value);
            current.maximum = current.maximum.max(value);
        } else {
            *target = Some(Self {
                minimum: value,
                maximum: value,
            });
        }
    }
}

/// Contributions after statistics and semantic positions, before scale policies or viewport.
#[derive(serde::Serialize, Clone, Debug, Default, PartialEq)]
pub struct DomainContributions {
    /// Eligible x coordinates and interval endpoints.
    pub x: Option<Extent>,
    /// Eligible y coordinates and interval endpoints/baselines.
    pub y: Option<Extent>,
    /// Declared x calculation space even for empty outputs.
    pub x_space: Option<ValueSpace>,
    /// Declared y calculation space even for empty outputs.
    pub y_space: Option<ValueSpace>,
}

/// Portable prepared geometry, explicitly in calculation/data units, never pixels.
#[derive(Clone, Debug, PartialEq)]
pub enum PreparedGeometry {
    /// Destination-unit radial run or edge translated to one data-space center.
    ShapePathRun {
        /// Explicit fill or stroke policy.
        paint: crate::shape::SymbolPaint,
        /// Data-space center projected through named axes.
        center: Point,
        /// Numeric local geometry from the common generator engine.
        geometry: crate::path::PathGeometry,
        /// Actual source locations, one per target; a link has two anchors for one edge identity.
        anchors: Vec<Point>,
    },
    /// Destination-unit shape translated to an encoded center, with one source focus anchor.
    ShapePath {
        /// Explicit filled/stroked geometry policy.
        paint: crate::shape::SymbolPaint,
        /// Data-space center projected by the named axes.
        center: Point,
        /// Checked local destination-unit geometry generated by the shared shape engine.
        geometry: crate::path::PathGeometry,
        /// Local focus location representing this source mark.
        anchor: Point,
    },
    /// Closed custom filled polygon, with one atomic semantic target.
    Polygon(Vec<Point>),
    /// Explicit native-only painter invocation; every headless renderer rejects it.
    NativePaint {
        /// First calculation-space corner.
        from: Point,
        /// Opposite calculation-space corner.
        to: Point,
        /// Registered native painter identity and version.
        painter: OperationRef,
        /// Bounded data, never executable code.
        parameters: serde_json::Value,
    },
    /// Circular point center; size stays in style as eventual destination units.
    Point(Point),
    /// Ordered straight run, including a one-vertex isolated run that must not bridge a gap.
    LineRun(Vec<Point>),
    /// Aligned lower/upper run vertices with one semantic target per pair.
    BandRun {
        /// First boundary in run order.
        lower: Vec<Point>,
        /// Second boundary in the same run order.
        upper: Vec<Point>,
    },
    /// Stack boundaries with optional source indexes for explicitly missing zero cells.
    StackBandRun {
        /// First boundary in ascending sample order.
        lower: Vec<Point>,
        /// Second boundary in the same sample order.
        upper: Vec<Point>,
        /// Index into the mark targets, or no observation for a virtual zero cell.
        sources: Vec<Option<usize>>,
    },
    /// Vertical interval with an explicit destination width.
    Bar {
        /// Value endpoint at the category/time/numeric center.
        from: Point,
        /// Baseline/second endpoint at the same center.
        to: Point,
        /// Positive destination width.
        width: f64,
    },
    /// Two independent endpoints.
    Rule {
        /// First endpoint.
        from: Point,
        /// Second endpoint.
        to: Point,
    },
    /// Exact data endpoints, before range projection constructs destination rectangle bounds.
    Rectangle {
        /// First encoded endpoint; may include a baseline.
        from: Point,
        /// Second encoded endpoint. Keeping endpoints avoids data-space width cancellation.
        to: Point,
    },
}

/// Geometry and semantic provenance, separate from source payloads.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedMark {
    /// Numeric geometry to be projected by the later scale/layout stage.
    pub geometry: PreparedGeometry,
    /// One target per line vertex; one target per point/rule/rectangle.
    pub targets: Vec<Target>,
    /// Stable group identity.
    pub group: GroupValue,
    /// Explicit resolved constant/mapped size plus solid color.
    pub style: Style,
}
/// One stable semantic target before viewport clipping or presentation visibility.
/// Source edits/filtering/statistical scope can remove it; ordinary navigation cannot.
#[derive(Clone, Copy, Debug)]
pub struct PreparedTarget<'a> {
    /// Presentation layer identity.
    pub layer: LayerId,
    /// Exact facet identity, absent for a single panel.
    pub panel: Option<&'a super::PanelKey>,
    /// Actual provenance and retained input scope.
    pub target: &'a Target,
    /// Explicit custom selection capability for this prepared mark.
    pub selectable: bool,
}

/// One prepared layer in paint order.
#[derive(Clone, Debug)]
pub struct PreparedLayer {
    pub(crate) shape_protocols: super::shape_extensions::ResolvedShapes,
    pub(crate) orientation: super::Orientation,
    pub(crate) interactions: BTreeMap<usize, super::GeometryInteraction>,
    pub(crate) color_legend: Option<crate::scales::ColorLegend>,
    pub(crate) symbol_legends: Vec<super::SymbolLegend>,
    pub(crate) position: super::Position,
    pub(crate) id: LayerId,
    pub(crate) scales: super::ScaleBindings,
    pub(crate) clip: super::ClipPolicy,
    pub(crate) table: Arc<PreparedTable>,
    pub(crate) marks: Arc<Vec<PreparedMark>>,
    pub(crate) domains: DomainContributions,
    pub(crate) invalid_geometry: usize,
    pub(crate) visible: bool,
}
impl PreparedLayer {
    /// Independent-axis direction retained for destination bar/dodge geometry.
    pub fn orientation(&self) -> super::Orientation {
        self.orientation
    }
    /// Explicit custom interaction contract by prepared mark index.
    pub fn interactions(&self) -> &BTreeMap<usize, super::GeometryInteraction> {
        &self.interactions
    }
    /// Exact categorical types and evaluated area-size guide samples.
    pub fn symbol_legends(&self) -> &[super::SymbolLegend] {
        &self.symbol_legends
    }
    /// Exact color identity, palette and domain metadata.
    pub fn color_legend(&self) -> Option<&crate::scales::ColorLegend> {
        self.color_legend.as_ref()
    }
    /// Exact semantic/display position policy.
    pub fn position(&self) -> &super::Position {
        &self.position
    }
    /// Bound positional scale identities.
    pub fn scales(&self) -> super::ScaleBindings {
        self.scales
    }
    /// Shared destination clipping policy.
    pub fn clip(&self) -> super::ClipPolicy {
        self.clip
    }
    /// Stable source layer identity.
    pub fn id(&self) -> LayerId {
        self.id
    }
    /// Shared stat output, whose generated/source type is explicit.
    pub fn table(&self) -> &Arc<PreparedTable> {
        &self.table
    }
    /// Prepared geometry; hidden layers remain prepared and contribute to domains by default.
    pub fn marks(&self) -> &[PreparedMark] {
        &self.marks
    }
    /// All endpoint contributions, before viewport/scale policies.
    pub fn domains(&self) -> &DomainContributions {
        &self.domains
    }
    /// Count of invalid required geometry/group/size rows.
    pub fn invalid_geometry(&self) -> usize {
        self.invalid_geometry
    }
    /// Presentation visibility captured through the common state/action path.
    pub fn visible(&self) -> bool {
        self.visible
    }
}

/// Observable graph reuse, not a runtime or performance benchmark.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PreparationMetrics {
    /// Named transforms evaluated in this preparation.
    pub evaluated_transforms: usize,
    /// Named outputs reused from the compiler's last valid immutable graph.
    pub reused_transforms: usize,
    /// Layer statistics evaluated (geometry may be rebuilt separately).
    pub evaluated_layers: usize,
    /// Exact layer statistic tables reused.
    pub reused_layers: usize,
}

/// Immutable grammar output consumed by the shared scale/layout stage.
/// It owns no typed source rows or callbacks; its handle pins one coherent source snapshot.
#[derive(Clone, Debug)]
pub struct PreparedChart {
    pub(crate) scale_registrations: Arc<super::scale_extensions::ScaleRegistrations>,
    pub(crate) panels: Vec<super::PreparedPanel>,
    pub(crate) shared_training: Option<Arc<PreparedChart>>,
    pub(crate) definition: Arc<ChartDefinition>,
    pub(crate) source: SnapshotHandle<StoreSnapshot>,
    pub(crate) state: ChartState,
    pub(crate) layers: Vec<PreparedLayer>,
    pub(crate) scale_domains: BTreeMap<crate::ScaleId, DomainContributions>,
    pub(crate) transforms: BTreeMap<TransformId, Arc<PreparedTable>>,
    pub(crate) domains: DomainContributions,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) metrics: PreparationMetrics,
}
impl PreparedChart {
    /// Explicit ordered facet populations; empty for a single-panel chart.
    pub fn panels(&self) -> &[super::PreparedPanel] {
        &self.panels
    }
    /// Independently trained named scales; each entry occupies exactly one orientation.
    pub fn scale_domains(&self) -> &BTreeMap<crate::ScaleId, DomainContributions> {
        &self.scale_domains
    }
    /// Exact captured normalized definition.
    pub fn definition(&self) -> &ChartDefinition {
        &self.definition
    }
    /// Coherent source snapshot; external handle disposal cannot invalidate this owner.
    pub fn source(&self) -> &SnapshotHandle<StoreSnapshot> {
        &self.source
    }
    /// Definition revision used for preparation.
    pub fn definition_revision(&self) -> Revision {
        self.definition.revision
    }
    /// Captured presentation state; source filtering is not a state viewport action.
    pub fn state(&self) -> &ChartState {
        &self.state
    }
    /// Layers in authored paint order.
    pub fn layers(&self) -> &[PreparedLayer] {
        &self.layers
    }
    /// Borrow stable targets from the presented preparation, including marks outside its viewport.
    /// Use the laid-out Inspector for visible hit/focus order; linking/selection use this catalog.
    pub fn semantic_targets(&self) -> impl Iterator<Item = PreparedTarget<'_>> {
        std::iter::once((None, self))
            .filter(|_| self.panels.is_empty())
            .chain(self.panels.iter().map(|p| (Some(&p.key), p.chart.as_ref())))
            .flat_map(|(panel, chart)| {
                chart.layers.iter().flat_map(move |layer| {
                    layer
                        .marks
                        .iter()
                        .enumerate()
                        .flat_map(move |(index, mark)| {
                            let selectable = layer
                                .interactions
                                .get(&index)
                                .is_none_or(|v| v.selection != super::SelectionPolicy::Disabled);
                            mark.targets.iter().map(move |target| PreparedTarget {
                                layer: layer.id,
                                panel,
                                target,
                                selectable,
                            })
                        })
                })
            })
    }
    /// Explicit named output, if declared.
    pub fn transform(&self, id: TransformId) -> Option<&Arc<PreparedTable>> {
        self.transforms.get(&id)
    }
    /// Primary x/y contributions, including hidden layers and all eligible endpoints.
    /// Other named axes are available through `scale_domains`.
    pub fn domains(&self) -> &DomainContributions {
        &self.domains
    }
    /// Bounded aggregate warnings; no per-row warning flood.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    /// Graph evaluation/reuse counters for validation and diagnosis.
    pub fn metrics(&self) -> PreparationMetrics {
        self.metrics
    }
}
