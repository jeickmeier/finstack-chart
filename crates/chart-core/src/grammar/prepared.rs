use super::{
    ChartDefinition, Grouping, NumericTransform, OperationRef, SourceFilter, StatSpace, Style,
};
use crate::data::{DatasetVersion, Schema, SnapshotHandle, StoreSnapshot, TimestampType};
use crate::provenance::Target;
use crate::state::ChartState;
use crate::{Diagnostic, LayerId, Point, Revision, RowKey, SchemaVersion, TransformId};
use std::{collections::BTreeMap, sync::Arc};

/// Exact group values; labels/codes and numeric IDs are never narrowed to f64.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum GroupValue {
    /// Whole population.
    All,
    /// Categorical or UTF-8 group label.
    Text(String),
    /// Signed integer group.
    Int(i64),
    /// Unsigned integer group.
    UInt(u64),
    /// Boolean group.
    Boolean(bool),
}

/// Declared axis calculation space, preventing silent double transforms or mixed origins.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueSpace {
    /// Numeric source units or a generated count.
    Data,
    /// Relative ticks, preserving the exact source timestamp representation and origin.
    Timestamp {
        /// Source unit/timezone.
        representation: TimestampType,
        /// Checked integer origin.
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneratedKind {
    /// Finite binary64 endpoint.
    Float64,
    /// Exact unsigned membership count.
    UInt64,
}

/// Generated schema descriptor, separate from any original source schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedField {
    /// Generated accessor identity.
    pub field: super::BinField,
    /// Portable physical kind.
    pub kind: GeneratedKind,
}

/// Prepared output schema; source accessors and generated accessors are disjoint.
#[derive(Clone, Debug, PartialEq)]
pub enum OutputSchema {
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceRow {
    /// Exact source key.
    pub key: RowKey,
    /// Stable insertion ordinal, independent of authored order.
    pub ordinal: u64,
}

/// Typed bin output; no source-row accessor receives this generated row.
#[derive(Clone, Debug, PartialEq)]
pub struct BinnedRow {
    /// Left edge in the declared output space.
    pub start: f64,
    /// Right edge, final edge included by the bin computation.
    pub end: f64,
    /// Exact number of source members.
    pub count: u64,
    /// Declared source group.
    pub group: GroupValue,
    /// Aggregate identity and complete compact source membership.
    pub target: Target,
}

/// Physically distinct source and generated row vectors.
#[derive(Clone, Debug, PartialEq)]
pub enum PreparedRows {
    /// Source-key references, not cloned original rows or type-erased callbacks.
    Source(Arc<[SourceRow]>),
    /// Typed generated rows with explicit aggregate provenance.
    Binned(Arc<[BinnedRow]>),
}
impl PreparedRows {
    /// Prepared output row count.
    pub fn len(&self) -> usize {
        match self {
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
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

/// Auditable stat invocation. All WP-05 operations use exact full recomputation.
#[derive(Clone, Debug, PartialEq)]
pub struct OperationRecord {
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

/// Finite data-space domain extent. Empty axes have no extent; WP-06 chooses fallbacks.
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Debug, Default, PartialEq)]
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
    /// Circular point center; size stays in style as eventual destination units.
    Point(Point),
    /// Ordered straight run, including a one-vertex isolated run that must not bridge a gap.
    LineRun(Vec<Point>),
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

/// One prepared layer in paint order.
#[derive(Clone, Debug)]
pub struct PreparedLayer {
    pub(crate) id: LayerId,
    pub(crate) table: Arc<PreparedTable>,
    pub(crate) marks: Vec<PreparedMark>,
    pub(crate) domains: DomainContributions,
    pub(crate) invalid_geometry: usize,
    pub(crate) visible: bool,
}
impl PreparedLayer {
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
}

/// Immutable prepared scene foundation. WP-06 adds scales, coordinates, guides and layout.
/// It owns no typed source rows or callbacks; its handle pins one coherent source snapshot.
#[derive(Clone, Debug)]
pub struct PreparedChart {
    pub(crate) definition: Arc<ChartDefinition>,
    pub(crate) source: SnapshotHandle<StoreSnapshot>,
    pub(crate) state: ChartState,
    pub(crate) layers: Vec<PreparedLayer>,
    pub(crate) transforms: BTreeMap<TransformId, Arc<PreparedTable>>,
    pub(crate) domains: DomainContributions,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) metrics: PreparationMetrics,
}
impl PreparedChart {
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
    /// Explicit named output, if declared.
    pub fn transform(&self, id: TransformId) -> Option<&Arc<PreparedTable>> {
        self.transforms.get(&id)
    }
    /// Combined compatible-axis contributions, including hidden layers and all endpoints.
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
