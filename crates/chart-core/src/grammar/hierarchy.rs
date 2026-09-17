//! Hierarchy preparation owns topology/aggregation; destination layout owns geometry.
use super::{compiler::EncodedRow, *};
use crate::{
    ChartResult, DiagnosticCode, FieldId, HierarchyId, HierarchyNodeId, RowKey,
    data::{DatasetSnapshot, ValueRef},
    hierarchy::*,
    provenance::Target,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};

/// Source fields used to construct the topology; row keys remain occurrence identities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HierarchySource<F = FieldId> {
    /// ID/parent labels. Missing parent labels identify roots.
    Table {
        /// Optional ID label field.
        id: Option<F>,
        /// Optional parent-label field.
        parent: Option<F>,
    },
    /// Slash paths with explicit inferred ancestor semantics.
    Paths(F),
}
/// Explicit aggregation before layout. Internal own source values participate in sum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HierarchyAggregation<F = FieldId> {
    /// One unit per leaf.
    Count,
    /// Numeric source field; null is zero and negative/nonfinite values reject.
    Sum(F),
    /// Registered native own-value accessor over source payloads.
    Registered(RegisteredOperation),
}
/// Sibling ordering is explicit and precedes all numeric layout work.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum HierarchyOrder {
    /// Original source order.
    #[default]
    Input,
    /// Stable ascending aggregate value.
    ValueAscending,
    /// Stable descending aggregate value.
    ValueDescending,
    /// Registered native comparator.
    Registered(RegisteredOperation),
}
/// Radius interpretation for partition depth bands in a sunburst.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum HierarchyRadius {
    /// Equal depth steps have equal radius thickness.
    Linear,
    /// Equal depth steps have equal squared-radius area.
    Area,
}
/// Destination projection; all lengths use the destination's declared units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HierarchyProjection {
    /// Top-to-bottom tree/cluster or ordinary rectangle/circle layout.
    #[default]
    Cartesian,
    /// Left-to-right tree/cluster or transposed icicle.
    Horizontal,
    /// Tree/cluster angles in radians clockwise from twelve o'clock.
    Radial,
    /// Partition angles with an explicit hole and depth/radius interpretation.
    Sunburst {
        /// Nonnegative hole radius in destination units.
        inner_radius: f64,
        /// Depth/radius mapping.
        radius: HierarchyRadius,
    },
}
/// A checked hierarchy recipe, shared by primary authoring, grammar and proof hosts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HierarchyRecipe<F = FieldId> {
    /// Stable owner identity, independent of source revision and destination.
    pub identity: HierarchyId,
    /// Topology source fields.
    pub source: HierarchySource<F>,
    /// Explicit own-value aggregation.
    pub aggregation: HierarchyAggregation<F>,
    /// Optional source label for semantic inspection; inferred paths retain their own label.
    pub label: Option<F>,
    /// Stable sibling order.
    #[serde(default)]
    pub order: HierarchyOrder,
    /// All standalone layout controls. Extent sizes are replaced by the allocated panel;
    /// fixed tree node spacing remains in destination units (radians for radial x).
    pub layout: LayoutSpec,
    /// Destination projection using shared arc/link kernels.
    #[serde(default)]
    pub projection: HierarchyProjection,
    /// Explicit node/payload/work limits.
    #[serde(default)]
    pub limits: HierarchyLimits,
}
impl<F> HierarchyRecipe<F> {
    /// Resolve source selectors once while retaining all other authored controls.
    pub fn try_map_fields<G>(
        self,
        mut map: impl FnMut(F) -> ChartResult<G>,
    ) -> ChartResult<HierarchyRecipe<G>> {
        Ok(HierarchyRecipe {
            identity: self.identity,
            source: match self.source {
                HierarchySource::Table { id, parent } => HierarchySource::Table {
                    id: id.map(&mut map).transpose()?,
                    parent: parent.map(&mut map).transpose()?,
                },
                HierarchySource::Paths(path) => HierarchySource::Paths(map(path)?),
            },
            aggregation: match self.aggregation {
                HierarchyAggregation::Count => HierarchyAggregation::Count,
                HierarchyAggregation::Sum(field) => HierarchyAggregation::Sum(map(field)?),
                HierarchyAggregation::Registered(op) => HierarchyAggregation::Registered(op),
            },
            label: self.label.map(&mut map).transpose()?,
            order: self.order,
            layout: self.layout,
            projection: self.projection,
            limits: self.limits,
        })
    }
}
/// Prepared immutable topology and one shared source membership table.
#[derive(Clone)]
pub struct PreparedHierarchy {
    pub(crate) tree: Hierarchy,
    pub(crate) recipe: HierarchyRecipe,
    pub(crate) registry: Arc<ExtensionRegistry>,
    pub(crate) source_keys: Arc<[RowKey]>,
    pub(crate) targets: BTreeMap<HierarchyNodeId, Target>,
    pub(crate) labels: BTreeMap<HierarchyNodeId, String>,
}
impl PreparedHierarchy {
    /// Exact prepared topology/aggregation; coordinates do not exist before panel allocation.
    pub fn hierarchy(&self) -> &Hierarchy {
        &self.tree
    }
    /// Exact authored recipe and stable source selectors.
    pub fn recipe(&self) -> &HierarchyRecipe {
        &self.recipe
    }
    /// One linear source-key table shared by all ancestor targets.
    pub fn source_keys(&self) -> &Arc<[RowKey]> {
        &self.source_keys
    }
    /// Prepared subtree target for one owner-scoped occurrence.
    pub fn target(&self, node: NodeHandle) -> ChartResult<&Target> {
        self.tree.get(node)?;
        self.targets.get(&node.node).ok_or_else(|| {
            error(
                DiagnosticCode::Validation,
                "Hierarchy node target is absent.",
            )
        })
    }
    /// Source label or inferred path, retained for semantic inspection.
    pub fn label(&self, node: NodeHandle) -> ChartResult<&str> {
        self.tree.get(node)?;
        Ok(self.labels.get(&node.node).map_or("", String::as_str))
    }
}
pub(super) fn validate(
    layer: &Layer,
    data: &DatasetSnapshot,
    inherited: &SourceAes,
) -> ChartResult<()> {
    let Some(recipe) = &layer.hierarchy else {
        if layer.geom == Geom::Hierarchy {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Hierarchy geometry requires a recipe.",
            ));
        }
        return Ok(());
    };
    if layer.geom != Geom::Hierarchy
        || !matches!(layer.statistic.parameters, StatParameters::Identity)
        || layer.geometry_extension.is_some()
        || layer.position != Position::Identity
        || layer.orientation != Orientation::Vertical
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Hierarchy recipes require identity source statistics/positioning and their explicit projection control.",
        ));
    }
    let Mappings::Source(authored) = &layer.mappings else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Hierarchy recipes require source rows.",
        ));
    };
    let aes = if layer.inherit {
        authored.inherit(inherited)
    } else {
        authored.clone()
    };
    if [
        &aes.x, &aes.y, &aes.x2, &aes.y2, &aes.low, &aes.high, &aes.size,
    ]
    .iter()
    .any(|v| v.is_some())
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Hierarchy layout owns coordinates and radius; use style controls and explicit recipe fields.",
        ));
    }
    if !layer.numeric_scales.is_empty()
        || layer.symbol.is_some()
        || layer.symbol_size_guide.is_some()
        || !layer.shape_protocols.is_empty()
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Hierarchy geometry uses its explicit layout radius/shape and shared color/style controls.",
        ));
    }
    let field = |id: FieldId| {
        data.schema().field(id).map(|_| ()).ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Hierarchy field is absent from its dataset.",
            )
        })
    };
    match &recipe.source {
        HierarchySource::Table { id, parent } => {
            for id in id.iter().chain(parent) {
                field(*id)?;
            }
        }
        HierarchySource::Paths(id) => field(*id)?,
    }
    if let HierarchyAggregation::Sum(id) = recipe.aggregation {
        super::stats::numeric_space(data, &Numeric::Field(id))?;
    }
    if let Some(label) = recipe.label {
        field(label)?;
    }
    match (&recipe.layout, recipe.projection) {
        (
            LayoutSpec::Tree { .. } | LayoutSpec::Cluster { .. },
            HierarchyProjection::Cartesian
            | HierarchyProjection::Horizontal
            | HierarchyProjection::Radial,
        )
        | (
            LayoutSpec::Partition(_),
            HierarchyProjection::Cartesian
            | HierarchyProjection::Horizontal
            | HierarchyProjection::Sunburst { .. },
        )
        | (LayoutSpec::Treemap { .. } | LayoutSpec::Pack { .. }, HierarchyProjection::Cartesian) => {
        }
        _ => {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Hierarchy layout and destination projection are incompatible.",
            ));
        }
    }
    if let HierarchyProjection::Sunburst { inner_radius, .. } = recipe.projection
        && (!inner_radius.is_finite() || inner_radius < 0.)
    {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Sunburst hole radius must be finite and nonnegative.",
        ));
    }
    Ok(())
}
fn payload(row: crate::data::RowView<'_>, data: &DatasetSnapshot) -> Value {
    Value::Object(
        data.schema()
            .fields()
            .iter()
            .map(|f| {
                let value = match row.value(f.id) {
                    None => Value::Null,
                    Some(ValueRef::Float64(v)) if v.is_finite() => serde_json::json!(v),
                    Some(ValueRef::Float64(v)) => serde_json::json!({"number":v.to_string()}),
                    Some(ValueRef::Int64(v) | ValueRef::Timestamp(v)) => {
                        Value::String(v.to_string())
                    }
                    Some(ValueRef::UInt64(v)) => Value::String(v.to_string()),
                    Some(ValueRef::Boolean(v)) => Value::Bool(v),
                    Some(ValueRef::Utf8(v) | ValueRef::Category(v)) => Value::String(v.to_owned()),
                };
                (f.name.clone(), value)
            })
            .collect(),
    )
}
fn label(row: crate::data::RowView<'_>, field: Option<FieldId>) -> ChartResult<Option<String>> {
    match field.and_then(|f| row.value(f)) {
        None => Ok(None),
        Some(ValueRef::Utf8(v) | ValueRef::Category(v)) => Ok(Some(v.into())),
        Some(ValueRef::Int64(v)) => Ok(Some(v.to_string())),
        Some(ValueRef::UInt64(v)) => Ok(Some(v.to_string())),
        _ => Err(error(
            DiagnosticCode::SchemaConflict,
            "Hierarchy labels require text or exact integer fields.",
        )),
    }
}
pub(super) fn prepare(
    layer: &Layer,
    table: &PreparedTable,
    data: &DatasetSnapshot,
    inherited: &SourceAes,
    registry: &Arc<ExtensionRegistry>,
    limits: CompileLimits,
) -> ChartResult<(Option<Arc<PreparedHierarchy>>, Vec<EncodedRow>)> {
    validate(layer, data, inherited)?;
    let recipe = layer.hierarchy.as_ref().expect("validated recipe");
    let PreparedRows::Source(rows) = &table.rows else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Hierarchy requires source rows.",
        ));
    };
    if rows.is_empty() {
        return Ok((None, vec![]));
    }
    crate::limits::require_within(
        rows.len().saturating_mul(data.schema().fields().len()) <= recipe.limits.max_work,
        "hierarchy source payload field",
    )?;
    crate::limits::require_within(
        rows.len() <= recipe.limits.max_nodes,
        "hierarchy source node",
    )?;
    data.prepare_lookup();
    let source: BTreeMap<_, _> = rows
        .iter()
        .map(|r| {
            data.row(r.key).map(|row| (r.key, row)).ok_or_else(|| {
                error(
                    DiagnosticCode::RevisionConflict,
                    "Prepared hierarchy row is absent from its source.",
                )
            })
        })
        .collect::<ChartResult<_>>()?;
    // Charge borrowed source payloads before allocating their owned JSON representation.
    let mut bytes = 0usize;
    for row in source.values() {
        for field in data.schema().fields() {
            let value_bytes = match row.value(field.id) {
                Some(ValueRef::Utf8(v) | ValueRef::Category(v)) => v.len(),
                // Exact integer strings and tagged nonfinite floats fit this conservative bound.
                _ => 32,
            };
            bytes = bytes
                .saturating_add(field.name.len())
                .saturating_add(value_bytes);
            crate::limits::require_within(
                bytes <= recipe.limits.max_payload_bytes,
                "hierarchy source payload byte",
            )?;
        }
    }
    let payloads = rows
        .iter()
        .map(|r| {
            (
                HierarchyNodeId::new(r.key.get()),
                Arc::new(payload(source[&r.key], data)),
            )
        })
        .collect::<Vec<_>>();
    let mut tree = match &recipe.source {
        HierarchySource::Table { id, parent } => Hierarchy::stratify_with(
            recipe.identity,
            payloads,
            |_, i, _| label(source[&rows[i].key], *id),
            |_, i, _| label(source[&rows[i].key], *parent),
            recipe.limits,
        )?,
        HierarchySource::Paths(path) => Hierarchy::from_paths(
            recipe.identity,
            payloads
                .into_iter()
                .enumerate()
                .map(|(i, (key, payload))| {
                    Ok((
                        key,
                        payload,
                        label(source[&rows[i].key], Some(*path))?.ok_or_else(|| {
                            error(DiagnosticCode::Validation, "Hierarchy path is null.")
                        })?,
                    ))
                })
                .collect::<ChartResult<_>>()?,
            recipe.limits,
        )?,
    };
    tree = match &recipe.aggregation {
        HierarchyAggregation::Count => tree.count()?,
        HierarchyAggregation::Sum(field) => tree.sum(|n| {
            if n.synthetic() {
                Ok(None)
            } else {
                let row = source[&RowKey::new(n.handle().node.get())];
                match row.value(*field) {
                    None => Ok(None),
                    Some(ValueRef::Float64(v)) => Ok(Some(v)),
                    Some(ValueRef::Int64(v)) if v.unsigned_abs() <= 1_u64 << 53 => {
                        Ok(Some(v as f64))
                    }
                    Some(ValueRef::UInt64(v)) if v <= 1_u64 << 53 => Ok(Some(v as f64)),
                    _ => Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Hierarchy sum requires numeric values.",
                    )),
                }
            }
        })?,
        HierarchyAggregation::Registered(op) => {
            let scalar = ScalarAccessor::Registered(op.clone());
            let scalar = scalar.compile_with(registry, false)?;
            tree.sum(|n| scalar.evaluate(n, HierarchyScalar::Value))?
        }
    };
    tree = match &recipe.order {
        HierarchyOrder::Input => tree,
        HierarchyOrder::ValueAscending => {
            tree.sort(|a, b| Ok(a.value().partial_cmp(&b.value()).expect("finite values")))?
        }
        HierarchyOrder::ValueDescending => {
            tree.sort(|a, b| Ok(b.value().partial_cmp(&a.value()).expect("finite values")))?
        }
        HierarchyOrder::Registered(op) => {
            let operation = registry.hierarchy_operation(&op.operation, &op.parameters, false)?;
            tree.sort(|a, b| operation.implementation.compare(a, b, &op.parameters))?
        }
    };
    crate::limits::require_within(
        tree.len() <= limits.max_prepared_rows
            && tree.len().saturating_mul(2) <= limits.max_vertices,
        "prepared hierarchy node/edge",
    )?;
    let mut keys = Vec::with_capacity(rows.len());
    let mut starts = BTreeMap::new();
    let mut counts = BTreeMap::new();
    let mut labels = BTreeMap::new();
    tree.visit(tree.root().handle(), VisitOrder::PreOrder, |n, _, _| {
        let key = n.handle().node;
        starts.insert(key, keys.len());
        if !n.synthetic() {
            keys.push(RowKey::new(key.get()));
        }
        labels.insert(
            key,
            if n.synthetic() {
                n.id().unwrap_or("").into()
            } else {
                label(source[&RowKey::new(key.get())], recipe.label)?
                    .or_else(|| n.id().map(str::to_owned))
                    .unwrap_or_else(|| key.get().to_string())
            },
        );
        Ok(())
    })?;
    tree.visit(tree.root().handle(), VisitOrder::PostOrder, |n, _, _| {
        let count = usize::from(!n.synthetic())
            + n.children()
                .map(|c| counts[&c.handle().node])
                .sum::<usize>();
        counts.insert(n.handle().node, count);
        Ok(())
    })?;
    let source_keys: Arc<[RowKey]> = keys.into();
    let targets = tree
        .iter()?
        .map(|n| {
            let key = n.handle().node;
            let identity = if n.synthetic() {
                HierarchyTargetKey::Synthetic(n.id().expect("imputed path").into())
            } else {
                HierarchyTargetKey::Source(RowKey::new(key.get()))
            };
            Ok((
                key,
                Target::HierarchyNode {
                    hierarchy: tree.identity(),
                    node: identity,
                    input: table.input,
                    membership: HierarchyMembership::new(
                        source_keys.clone(),
                        starts[&key],
                        starts[&key] + counts[&key],
                    )?,
                },
            ))
        })
        .collect::<ChartResult<BTreeMap<_, _>>>()?;
    let Mappings::Source(aes) = &layer.mappings else {
        unreachable!()
    };
    let aes = if layer.inherit {
        aes.inherit(inherited)
    } else {
        aes.clone()
    };
    let grouping = aes
        .grouping
        .clone()
        .unwrap_or_else(|| aes.group.map_or(Grouping::All, Grouping::Field));
    let mut encoded = Vec::with_capacity(tree.len());
    tree.visit(tree.root().handle(), VisitOrder::PreOrder, |n, i, _| {
        let key = (!n.synthetic()).then(|| RowKey::new(n.handle().node.get()));
        encoded.push(EncodedRow {
            geo_feature: None,
            stat_outliers: vec![],
            outlier_anchor_y: None,
            recipe_values: Default::default(),
            missing_aesthetics: 0,
            values: Default::default(),
            x: None,
            y: None,
            x2: None,
            y2: None,
            low: None,
            high: None,
            size: None,
            color: None,
            fill: None,
            stroke: None,
            shape: None,
            opacity: None,
            alpha: None,
            stroke_width: None,
            group: key
                .and_then(|k| super::stats::group_value(source[&k], &grouping))
                .or(Some(GroupValue::All)),
            ordinal: i as u64,
            target: targets[&n.handle().node].clone(),
            key,
        });
        Ok(())
    })?;
    Ok((
        Some(Arc::new(PreparedHierarchy {
            tree,
            recipe: recipe.clone(),
            registry: registry.clone(),
            source_keys,
            targets,
            labels,
        })),
        encoded,
    ))
}

pub(super) fn emit(
    prepared: &mut PreparedLayer,
    layer: &Layer,
    rows: Vec<EncodedRow>,
    vertices: &mut usize,
) -> ChartResult<()> {
    let Some(hierarchy) = &prepared.hierarchy else {
        return Ok(());
    };
    let mut nodes = Vec::with_capacity(rows.len());
    hierarchy.tree.visit(
        hierarchy.tree.root().handle(),
        VisitOrder::PreOrder,
        |n, _, _| {
            nodes.push(n.handle());
            Ok(())
        },
    )?;
    let styles = rows
        .iter()
        .map(|row| super::compiler::row_style(layer, row))
        .collect::<ChartResult<Vec<_>>>()?;
    let by_key: BTreeMap<_, _> = nodes.iter().enumerate().map(|(i, n)| (n.node, i)).collect();
    let mut marks = Vec::with_capacity(nodes.len().saturating_mul(2));
    if matches!(
        hierarchy.recipe.layout,
        LayoutSpec::Tree { .. } | LayoutSpec::Cluster { .. }
    ) {
        for (parent, child) in hierarchy.tree.links(hierarchy.tree.root().handle())? {
            let index = by_key[&child.node];
            // Exact composite model scope avoids hashed parent collisions. Reparenting
            // changes the edge identity while the child node's selection stays stable.
            let parent_identity = match &hierarchy.targets[&parent.node] {
                Target::HierarchyNode { node, .. } => serde_json::to_string(node)
                    .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?,
                _ => unreachable!(),
            };
            let target = Target::Derived {
                id: crate::DerivedId::new(child.node.get()),
                model: format!(
                    "chart.hierarchy.link/{}/{}",
                    hierarchy.tree.identity().get(),
                    parent_identity
                ),
                model_version: crate::Revision::new(1),
                inputs: vec![prepared.table.input],
            };
            marks.push(PreparedMark {
                aesthetics: Default::default(),
                geometry: PreparedGeometry::HierarchyLink { parent, child },
                targets: vec![target],
                group: rows[index].group.clone().unwrap_or(GroupValue::All),
                style: styles[index],
            });
        }
    }
    for ((node, row), style) in nodes.into_iter().zip(rows).zip(styles) {
        marks.push(PreparedMark {
            aesthetics: Default::default(),
            geometry: PreparedGeometry::HierarchyNode(node),
            targets: vec![row.target],
            group: row.group.unwrap_or(GroupValue::All),
            style,
        });
    }
    if marks.len() > *vertices {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Prepared hierarchy geometry budget exceeded.",
        ));
    }
    *vertices -= marks.len();
    prepared.marks = Arc::new(marks);
    Ok(())
}

impl std::fmt::Debug for PreparedHierarchy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedHierarchy")
            .field("tree", &self.tree)
            .field("recipe", &self.recipe)
            .field("source_keys", &self.source_keys)
            .finish_non_exhaustive()
    }
}

impl ExtensionRegistry {
    /// Validate every selected hierarchy operation before portable capture or serialization.
    pub fn validate_portable_hierarchies(&self, definition: &ChartDefinition) -> ChartResult<()> {
        self.validate_hierarchy_selections(definition, true)
    }
    pub(crate) fn validate_hierarchy_selections(
        &self,
        definition: &ChartDefinition,
        portable: bool,
    ) -> ChartResult<()> {
        for layer in &definition.layers {
            if let Some(recipe) = &layer.hierarchy {
                recipe.layout.validate_with(self, portable)?;
                if let HierarchyAggregation::Registered(op) = &recipe.aggregation {
                    self.hierarchy_operation(&op.operation, &op.parameters, portable)?;
                }
                if let HierarchyOrder::Registered(op) = &recipe.order {
                    self.hierarchy_operation(&op.operation, &op.parameters, portable)?;
                }
            }
        }
        Ok(())
    }
}
