//! Versioned standalone operations shared by native and proof hosts.
use super::*;
use crate::grammar::{ExtensionRegistry, HierarchyScalar, OperationRef};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};

/// Registered native implementation and bounded declarative parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisteredOperation {
    /// Exact registered version.
    pub operation: OperationRef,
    /// Declarative configuration; never executable host code.
    #[serde(default)]
    pub parameters: Value,
}
impl RegisteredOperation {
    pub(super) fn resolve(
        &self,
        registry: &ExtensionRegistry,
    ) -> ChartResult<crate::grammar::hierarchy_extensions::HierarchyRegistration> {
        self.resolve_with(registry, true)
    }
    fn resolve_with(
        &self,
        registry: &ExtensionRegistry,
        portable: bool,
    ) -> ChartResult<crate::grammar::hierarchy_extensions::HierarchyRegistration> {
        registry.hierarchy_operation(&self.operation, &self.parameters, portable)
    }
}
/// Scalar selections for aggregation, radius and padding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScalarAccessor {
    /// Finite constant.
    Constant(f64),
    /// Numeric payload field, with absent/null interpreted as missing.
    Field(String),
    /// Previously aggregated node value.
    Value,
    /// Node depth.
    Depth,
    /// Captured native operation.
    Registered(RegisteredOperation),
}
impl ScalarAccessor {
    pub(crate) fn compile<'a>(
        &'a self,
        registry: &ExtensionRegistry,
    ) -> ChartResult<CompiledScalar<'a>> {
        self.compile_with(registry, true)
    }
    pub(crate) fn compile_with<'a>(
        &'a self,
        registry: &ExtensionRegistry,
        portable: bool,
    ) -> ChartResult<CompiledScalar<'a>> {
        let registered = match self {
            Self::Registered(op) => Some(op.resolve_with(registry, portable)?),
            Self::Constant(v) if !v.is_finite() => {
                return Err(numerical("Hierarchy constant must be finite."));
            }
            _ => None,
        };
        Ok(CompiledScalar {
            accessor: self,
            registered,
        })
    }
}
pub(crate) struct CompiledScalar<'a> {
    accessor: &'a ScalarAccessor,
    registered: Option<crate::grammar::hierarchy_extensions::HierarchyRegistration>,
}
impl CompiledScalar<'_> {
    pub(crate) fn evaluate(
        &self,
        node: NodeView<'_>,
        purpose: HierarchyScalar,
    ) -> ChartResult<Option<f64>> {
        let result =
            match self.accessor {
                ScalarAccessor::Constant(v) => Some(*v),
                ScalarAccessor::Value => node.value(),
                ScalarAccessor::Depth => Some(node.depth() as f64),
                ScalarAccessor::Field(field) => match node.data().get(field) {
                    None | Some(Value::Null) => None,
                    Some(value) => Some(value.as_f64().ok_or_else(|| {
                        invalid("Hierarchy scalar field must be numeric or null.")
                    })?),
                },
                ScalarAccessor::Registered(operation) => self
                    .registered
                    .as_ref()
                    .expect("compiled registration")
                    .implementation
                    .scalar(node, purpose, &operation.parameters)?,
            };
        if result.is_some_and(|n| !n.is_finite()) {
            return Err(numerical("Hierarchy accessor must return a finite number."));
        }
        Ok(result)
    }
}
/// Stable declarative sibling comparison. Missing scalar values sort first.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Comparator {
    /// Numeric ascending or descending comparison.
    Scalar {
        /// Scalar selection.
        accessor: ScalarAccessor,
        /// Reverse non-equal comparisons.
        descending: bool,
    },
    /// Text payload comparison; missing/null sorts first.
    Text {
        /// Payload field.
        field: String,
        /// Reverse non-equal comparisons.
        descending: bool,
    },
    /// Captured native comparator.
    Registered(RegisteredOperation),
}
impl Comparator {
    pub(super) fn sort(
        &self,
        tree: &Hierarchy,
        registry: &ExtensionRegistry,
    ) -> ChartResult<Hierarchy> {
        match self {
            Self::Scalar {
                accessor,
                descending,
            } => {
                let accessor = accessor.compile(registry)?;
                tree.sort(|a, b| {
                    let order = accessor
                        .evaluate(a, HierarchyScalar::Value)?
                        .partial_cmp(&accessor.evaluate(b, HierarchyScalar::Value)?)
                        .ok_or_else(|| numerical("Unordered hierarchy comparator."))?;
                    Ok(if *descending { order.reverse() } else { order })
                })
            }
            Self::Text { field, descending } => tree.sort(|a, b| {
                fn text<'a>(node: NodeView<'a>, field: &str) -> ChartResult<Option<&'a str>> {
                    match node.data().get(field) {
                        None | Some(Value::Null) => Ok(None),
                        Some(Value::String(s)) => Ok(Some(s)),
                        _ => Err(invalid("Hierarchy text comparator requires text or null.")),
                    }
                }
                let order = text(a, field)?.cmp(&text(b, field)?);
                Ok(if *descending { order.reverse() } else { order })
            }),
            Self::Registered(op) => {
                let implementation = op.resolve(registry)?;
                tree.sort(|a, b| implementation.implementation.compare(a, b, &op.parameters))
            }
        }
    }
}
/// Explicit source occurrence and original payload.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadNode {
    /// Durable key, encoded as a decimal string.
    pub key: HierarchyNodeId,
    /// Original data.
    pub data: Arc<Value>,
}
/// Versioned construction independent of charts and rendering.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HierarchyEnvelope {
    /// Current standalone hierarchy envelope version: 1.
    pub version: u32,
    /// Explicit owner identity.
    pub identity: HierarchyId,
    /// Per-operation bounds.
    #[serde(default)]
    pub limits: HierarchyLimits,
    /// Input representation.
    pub input: HierarchyInput,
}
/// All standalone construction families.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HierarchyInput {
    /// Explicit keyed parent occurrences.
    Rows(Vec<NodeInput>),
    /// Explicit nested occurrences.
    Nested(NestedNode),
    /// Standalone node.
    Node(PayloadNode),
    /// Ordered grouped entries under a synthetic root.
    Grouped {
        /// Explicit root key.
        root: HierarchyNodeId,
        /// Ordered entries.
        entries: Vec<GroupedEntry>,
    },
    /// Field-based ID/parent or path stratification.
    Stratify {
        /// Keyed input records.
        rows: Vec<PayloadNode>,
        /// Accessor configuration, including disabled fields.
        options: StratifyOptions,
    },
    /// Native registered children accessor.
    Children {
        /// Root payload and stable key.
        root: PayloadNode,
        /// Children operation.
        accessor: RegisteredOperation,
    },
}
impl HierarchyEnvelope {
    /// Construct and validate with a captured registry, before publishing a session.
    pub fn build(self, registry: &ExtensionRegistry) -> ChartResult<Hierarchy> {
        if self.version != 1 {
            return Err(invalid(
                "Unsupported hierarchy envelope version; expected 1.",
            ));
        }
        let (id, limits) = (self.identity, self.limits);
        match self.input {
            HierarchyInput::Rows(rows) => Hierarchy::from_rows(id, rows, limits),
            HierarchyInput::Nested(root) => Hierarchy::from_nested(id, root, limits),
            HierarchyInput::Node(node) => Hierarchy::node(id, node.key, node.data, limits),
            HierarchyInput::Grouped { root, entries } => {
                Hierarchy::from_grouped(id, root, entries, limits)
            }
            HierarchyInput::Stratify { rows, options } => Hierarchy::stratify(
                id,
                rows.into_iter().map(|r| (r.key, r.data)).collect(),
                &options,
                limits,
            ),
            HierarchyInput::Children { root, accessor } => {
                let operation = accessor.resolve(registry)?;
                Hierarchy::with_children(
                    id,
                    (root.key, root.data),
                    |key, data| {
                        let children =
                            operation
                                .implementation
                                .children(key, data, &accessor.parameters)?;
                        within(
                            children.len() <= limits.max_nodes,
                            "Registered children exceed node budget.",
                        )?;
                        Ok(children)
                    },
                    limits,
                )
            }
        }
    }
}
/// Layout configuration retained for exact readback.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LayoutSpec {
    /// Tidy tree.
    Tree {
        /// Size and built-in separation.
        options: TreeOptions,
        /// Optional native separation override.
        separation: Option<RegisteredOperation>,
    },
    /// Leaf-aligned cluster.
    Cluster {
        /// Size and built-in separation.
        options: TreeOptions,
        /// Optional native separation override.
        separation: Option<RegisteredOperation>,
    },
    /// Adjacency rectangles.
    Partition(PartitionOptions),
    /// Treemap with explicit optional history and native accessors.
    Treemap {
        /// All constant controls.
        options: TreemapOptions,
        /// Reuse compatible rows.
        history: bool,
        /// Optional padding override for all slots.
        padding: Option<ScalarAccessor>,
        /// Per-side accessors take precedence over the all-slot accessor and numeric defaults.
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        padding_sides: BTreeMap<PaddingSide, ScalarAccessor>,
        /// Optional custom tiler.
        tiler: Option<RegisteredOperation>,
    },
    /// Circle packing.
    Pack {
        /// Size, radius mode and default padding.
        options: PackOptions,
        /// Explicit leaf radius accessor.
        radius: Option<ScalarAccessor>,
        /// Optional padding override.
        padding: Option<ScalarAccessor>,
    },
}
struct Accessors<'a> {
    padding_sides: BTreeMap<PaddingSide, CompiledScalar<'a>>,
    padding: Option<CompiledScalar<'a>>,
    radius: Option<CompiledScalar<'a>>,
    tiler: Option<(
        &'a RegisteredOperation,
        crate::grammar::hierarchy_extensions::HierarchyRegistration,
    )>,
}
impl TreemapAccessors for Accessors<'_> {
    fn padding(
        &mut self,
        node: NodeView<'_>,
        side: PaddingSide,
        configured: f64,
    ) -> ChartResult<f64> {
        self.padding_sides
            .get(&side)
            .or(self.padding.as_ref())
            .map_or(Ok(configured), |a| {
                a.evaluate(node, HierarchyScalar::TreemapPadding(side))?
                    .ok_or_else(|| invalid("Padding accessor returned missing."))
            })
    }
    fn tile(&mut self, parent: NodeView<'_>, bounds: [f64; 4]) -> ChartResult<Vec<[f64; 4]>> {
        let (spec, op) = self
            .tiler
            .as_ref()
            .ok_or_else(|| invalid("Custom tiler requires a registered operation."))?;
        op.implementation.tile(parent, bounds, &spec.parameters)
    }
}
impl PackAccessors for Accessors<'_> {
    fn radius(&mut self, node: NodeView<'_>) -> ChartResult<f64> {
        self.radius
            .as_ref()
            .ok_or_else(|| invalid("Explicit packing requires a radius accessor."))?
            .evaluate(node, HierarchyScalar::Radius)?
            .ok_or_else(|| invalid("Radius accessor returned missing."))
    }
    fn padding(&mut self, node: NodeView<'_>, configured: f64) -> ChartResult<f64> {
        self.padding.as_ref().map_or(Ok(configured), |a| {
            a.evaluate(node, HierarchyScalar::PackPadding)?
                .ok_or_else(|| invalid("Padding accessor returned missing."))
        })
    }
}
impl LayoutSpec {
    /// Validate all controls and registrations without visiting nodes or running layout.
    pub fn validate(&self, registry: &ExtensionRegistry) -> ChartResult<()> {
        self.validate_with(registry, false)
    }
    pub(crate) fn validate_with(
        &self,
        registry: &ExtensionRegistry,
        portable: bool,
    ) -> ChartResult<()> {
        match self {
            Self::Tree {
                options,
                separation,
            }
            | Self::Cluster {
                options,
                separation,
            } => {
                options.validate()?;
                if let Some(op) = separation {
                    op.resolve_with(registry, portable)?;
                }
            }
            Self::Partition(options) => {
                super::layout::extent(options.size)?;
                if !options.padding.is_finite() {
                    return Err(numerical("Partition padding must be finite."));
                }
            }
            Self::Treemap {
                options,
                padding,
                padding_sides,
                tiler,
                ..
            } => {
                options.validate()?;
                if tiler.is_some() != matches!(options.tile, Tiler::Custom) {
                    return Err(invalid(
                        "A registered tiler requires the Custom tiler mode and vice versa.",
                    ));
                }
                if let Some(op) = tiler {
                    op.resolve_with(registry, portable)?;
                }
                if let Some(a) = padding {
                    a.compile_with(registry, portable)?;
                }
                for a in padding_sides.values() {
                    a.compile_with(registry, portable)?;
                }
            }
            Self::Pack {
                options,
                radius,
                padding,
            } => {
                super::layout::extent(options.size)?;
                if !options.padding.is_finite() {
                    return Err(numerical("Pack padding must be finite."));
                }
                if radius.is_some() != matches!(options.radius, PackRadius::Explicit) {
                    return Err(invalid(
                        "Radius accessor and Explicit packing mode must be selected together.",
                    ));
                }
                if let Some(a) = radius {
                    a.compile_with(registry, portable)?;
                }
                if let Some(a) = padding {
                    a.compile_with(registry, portable)?;
                }
            }
        }
        Ok(())
    }

    /// Execute a destination-independent numeric layout through the canonical kernels.
    pub fn layout(
        &self,
        tree: &Hierarchy,
        registry: &ExtensionRegistry,
        history_state: &mut TreemapHistory,
    ) -> ChartResult<HierarchyLayout> {
        self.layout_with(tree, registry, history_state, false)
    }
    pub(crate) fn layout_with(
        &self,
        tree: &Hierarchy,
        registry: &ExtensionRegistry,
        history_state: &mut TreemapHistory,
        portable: bool,
    ) -> ChartResult<HierarchyLayout> {
        match self {
            Self::Tree {
                options,
                separation,
            }
            | Self::Cluster {
                options,
                separation,
            } => {
                let registered = separation
                    .as_ref()
                    .map(|s| s.resolve_with(registry, portable))
                    .transpose()?;
                let separation_fn =
                    |a: NodeView<'_>, b: NodeView<'_>| match (&registered, separation) {
                        (Some(op), Some(spec)) => {
                            op.implementation.separation(a, b, &spec.parameters)
                        }
                        _ => Ok(options.separation.evaluate(a, b)),
                    };
                if matches!(self, Self::Tree { .. }) {
                    tree.tree_with(*options, separation_fn)
                } else {
                    tree.cluster_with(*options, separation_fn)
                }
            }
            Self::Partition(options) => tree.partition(*options),
            Self::Treemap {
                options,
                history,
                padding,
                padding_sides,
                tiler,
            } => {
                if tiler.is_some() != matches!(options.tile, Tiler::Custom) {
                    return Err(invalid(
                        "A registered tiler requires the Custom tiler mode and vice versa.",
                    ));
                }
                tree.treemap_with(
                    *options,
                    if *history { Some(history_state) } else { None },
                    &mut Accessors {
                        padding_sides: padding_sides
                            .iter()
                            .map(|(side, accessor)| {
                                Ok((*side, accessor.compile_with(registry, portable)?))
                            })
                            .collect::<ChartResult<_>>()?,
                        padding: padding
                            .as_ref()
                            .map(|a| a.compile_with(registry, portable))
                            .transpose()?,
                        radius: None,
                        tiler: tiler
                            .as_ref()
                            .map(|op| Ok((op, op.resolve_with(registry, portable)?)))
                            .transpose()?,
                    },
                )
            }
            Self::Pack {
                options,
                radius,
                padding,
            } => {
                if radius.is_some() != matches!(options.radius, PackRadius::Explicit) {
                    return Err(invalid(
                        "Radius accessor and Explicit packing mode must be selected together.",
                    ));
                }
                tree.pack_with(
                    *options,
                    &mut Accessors {
                        padding_sides: BTreeMap::new(),
                        padding: padding
                            .as_ref()
                            .map(|a| a.compile_with(registry, portable))
                            .transpose()?,
                        radius: radius
                            .as_ref()
                            .map(|a| a.compile_with(registry, portable))
                            .transpose()?,
                        tiler: None,
                    },
                )
            }
        }
    }
}

/// Independent tiler invocation with optional owned resquarify history.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TilingRequest {
    /// Current tiler protocol version: 1.
    pub version: u32,
    /// Parent occurrence with children.
    pub parent: NodeHandle,
    /// Explicit left/top/right/bottom bounds.
    pub bounds: [f64; 4],
    /// Built-in tiler or Custom with a registered operation.
    pub tiler: Tiler,
    /// Whether to retain compatible row partitions.
    pub history: bool,
    /// Custom tiler implementation.
    pub operation: Option<RegisteredOperation>,
}
impl TilingRequest {
    pub(super) fn execute(
        &self,
        tree: &Hierarchy,
        registry: &ExtensionRegistry,
        history: &mut TreemapHistory,
    ) -> ChartResult<Vec<(NodeHandle, [f64; 4])>> {
        if self.version != 1 {
            return Err(invalid("Unsupported tiler request version."));
        }
        if self.operation.is_some() != matches!(self.tiler, Tiler::Custom) {
            return Err(invalid(
                "Custom tiler and registration must be selected together.",
            ));
        }
        let mut accessors = Accessors {
            padding_sides: BTreeMap::new(),
            radius: None,
            padding: None,
            tiler: self
                .operation
                .as_ref()
                .map(|op| Ok((op, op.resolve(registry)?)))
                .transpose()?,
        };
        tree.tile_children(
            self.parent,
            self.bounds,
            self.tiler,
            if self.history { Some(history) } else { None },
            &mut accessors,
        )
    }
}
