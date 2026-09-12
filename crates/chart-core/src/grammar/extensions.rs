//! Explicit versioned native implementations behind portable data descriptors.
//! Registries never load code, resolve libraries, perform I/O or contain host objects.
use super::*;
use crate::data::{DatasetSnapshot, InvalidPolicy, RowView};
use crate::provenance::Target;
use crate::{ChartResult, DiagnosticCode, Revision};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

/// A registered operation's declarative parameters and statistical population policy.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExtensionParameters {
    /// Bounded JSON values interpreted only by a registered implementation.
    pub values: serde_json::Value,
    /// Stable grouping; facet/chart scope overrides follow the builtin contract.
    pub grouping: Grouping,
    /// Explicit calculation space, independent of axis transforms and viewports.
    pub space: StatSpace,
}
impl ExtensionParameters {
    /// Whole-population data-space parameters.
    pub fn new(values: serde_json::Value) -> Self {
        Self {
            values,
            grouping: Grouping::All,
            space: StatSpace::Data,
        }
    }
}
/// Registration metadata, immutable for the lifetime of a compiler/snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtensionDescriptor {
    /// Non-builtin qualified operation name and exact version.
    pub operation: OperationRef,
    /// Whether a descriptor can execute in a portable session with this registration.
    pub portable: bool,
    /// Actual supported update paths. Alpha extensions use exact batch recomputation.
    pub incremental: IncrementalCapabilities,
}
impl ExtensionDescriptor {
    /// An exact batch implementation, optionally portable when explicitly registered.
    pub fn batch(id: impl Into<String>, version: Revision, portable: bool) -> Self {
        Self {
            operation: OperationRef::new(id, version),
            portable,
            incremental: IncrementalCapabilities {
                append: false,
                window: false,
                correction: false,
                full_recompute: true,
            },
        }
    }
}
/// Immutable filtered population given to an extension. It is never a viewport crop.
pub struct CustomStatInput<'a> {
    /// Original normalized source and schema, pinned by preparation.
    pub data: &'a DatasetSnapshot,
    /// Eligible source keys after source/facet filters.
    pub rows: &'a [SourceRow],
    /// Unique layer/transform and facet scope for target identities.
    pub scope: &'a str,
    /// Exact operation identity/version.
    pub operation: &'a OperationRef,
    /// Declared scope after facet/chart grouping resolution.
    pub parameters: &'a ExtensionParameters,
    /// Invalid required input policy; exclusions must be returned in `invalid_rows`.
    pub invalid: InvalidPolicy,
    /// Remaining shared preparation budget.
    pub limits: CompileLimits,
}
impl CustomStatInput<'_> {
    /// Resolve a source number with the same null/precision and pre-stat transform policy.
    pub fn number(&self, row: RowView<'_>, value: &Numeric) -> Option<f64> {
        stats::number(row, value).and_then(|v| match &self.parameters.space {
            StatSpace::Data => Some(v),
            StatSpace::Transformed(t) => {
                let v = t.factor.mul_add(v, t.offset);
                v.is_finite().then_some(v)
            }
        })
    }
    /// Resolve exact grouping without narrowing integer identities.
    pub fn group(&self, row: RowView<'_>) -> Option<GroupValue> {
        stats::group_value(row, &self.parameters.grouping)
    }
}
/// Generated typed rows and invalid-input accounting returned by a custom statistic.
#[derive(Clone, Debug, Default)]
pub struct CustomStatOutput {
    /// Generated values against the operation's own schema. Source accessors cannot bind these.
    pub rows: Vec<StatisticalRow>,
    /// Distinct filtered source keys excluded for invalid required inputs.
    pub invalid_rows: Vec<crate::RowKey>,
}
/// A native statistic normalized before scales/layout/rendering. Implementations are immutable
/// and thread-safe so hosts may schedule them explicitly; core always supports synchronous use.
pub trait CustomStat: Send + Sync {
    /// Stable identity, portability and verified update capabilities.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate parameters/source fields and declare generated fields before evaluation.
    /// Numeric custom fields use `StatField::Custom`; count/group retain their exact slots.
    fn schema(
        &self,
        data: &DatasetSnapshot,
        parameters: &ExtensionParameters,
        limits: CompileLimits,
    ) -> ChartResult<Vec<StatColumn>>;
    /// Evaluate the supplied filtered population, respecting the remaining budget.
    fn evaluate(&self, input: CustomStatInput<'_>) -> ChartResult<CustomStatOutput>;
}
/// Obtain the validated calculation-space metadata for a custom numeric input.
pub fn extension_input_space(
    data: &DatasetSnapshot,
    value: &Numeric,
    space: &StatSpace,
) -> ChartResult<ValueSpace> {
    let input = stats::numeric_space(data, value)?;
    Ok(match space {
        StatSpace::Data => input,
        StatSpace::Transformed(t) => ValueSpace::Transformed {
            input: Box::new(input),
            transform: t.clone(),
        },
    })
}
/// Immutable-by-ownership versioned registry. Install trusted implementations explicitly;
/// portable JSON can only select an existing entry and never supplies executable code.
#[derive(Clone, Default)]
pub struct ExtensionRegistry {
    pub(crate) limits_function: Arc<super::scale_limit_extensions::ScaleLimitRegistrations>,
    pub(crate) hierarchies: Arc<super::hierarchy_extensions::HierarchyRegistrations>,
    pub(crate) interpolations: Arc<super::interpolation_extensions::InterpolationRegistrations>,
    pub(crate) guides: Arc<super::guide_extensions::GuideRegistrations>,
    pub(crate) scales: Arc<super::scale_extensions::ScaleRegistrations>,
    stats: BTreeMap<(String, u64), Arc<dyn CustomStat>>,
    geoms: BTreeMap<(String, u64), Arc<dyn CustomGeom>>,
    pub(crate) shapes: super::shape_extensions::ShapeRegistrations,
}
impl ExtensionRegistry {
    /// Empty registry; builtins continue to use the common compiler without registration.
    pub fn new() -> Self {
        Self::default()
    }
    /// Register one exact version. Duplicate entries and builtin namespace overrides reject.
    pub fn register_stat(&mut self, stat: Arc<dyn CustomStat>) -> ChartResult<()> {
        let d = stat.descriptor();
        validate_descriptor(&d)?;
        let key = (d.operation.id.clone(), d.operation.version.get());
        if self.stats.contains_key(&key) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "An extension version is already registered.",
            ));
        }
        if self.stats.len() >= 64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "At most 64 stat extensions can be registered.",
            ));
        }
        self.stats.insert(key, stat);
        Ok(())
    }
    /// Register a custom geometry without replacing builtin geometry/renderer code.
    pub fn register_geom(&mut self, geom: Arc<dyn CustomGeom>) -> ChartResult<()> {
        let d = geom.descriptor();
        validate_descriptor(&d)?;
        let key = (d.operation.id.clone(), d.operation.version.get());
        if self.geoms.contains_key(&key) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "A geometry extension version is already registered.",
            ));
        }
        if self.geoms.len() >= 64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "At most 64 geometry extensions can be registered.",
            ));
        }
        self.geoms.insert(key, geom);
        Ok(())
    }
    /// Resolve a known geometry version; unknown/native callbacks never load from JSON.
    pub fn geometry_descriptor(&self, op: &OperationRef) -> ChartResult<ExtensionDescriptor> {
        Ok(self.geom(op)?.descriptor())
    }
    pub(crate) fn geom(&self, op: &OperationRef) -> ChartResult<&dyn CustomGeom> {
        self.geoms
            .get(&(op.id.clone(), op.version.get()))
            .map(Arc::as_ref)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "Geometry {} version {} is not registered.",
                        op.id,
                        op.version.get()
                    ),
                )
            })
    }
    /// Resolve metadata without invoking the implementation.
    pub fn stat_descriptor(&self, operation: &OperationRef) -> ChartResult<ExtensionDescriptor> {
        Ok(self.stat(operation)?.descriptor())
    }
    pub(crate) fn stat(&self, operation: &OperationRef) -> ChartResult<&dyn CustomStat> {
        self.stats.get(&(operation.id.clone(),operation.version.get())).map(Arc::as_ref).ok_or_else(||error(DiagnosticCode::UnsupportedCapability,format!("Extension {} version {} is not registered; install a known implementation or use a builtin.",operation.id,operation.version.get())))
    }
    pub(crate) fn validate_portable(&self, definition: &ChartDefinition) -> ChartResult<()> {
        self.validate_portable_hierarchies(definition)?;
        self.validate_scale_selections(definition, true)?;
        self.validate_guide_selections(definition, true)?;
        self.validate_interpolation_selections(definition, true)?;
        for layer in &definition.layers {
            for (family, selection) in &layer.shape_protocols {
                self.resolve_portable_shape(selection, *family)?;
            }
            if let Some(g) = &layer.geometry_extension
                && !self.geometry_descriptor(&g.operation)?.portable
            {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "A native-only geometry/painter cannot execute or serialize as a portable chart.",
                ));
            }
        }
        for stat in definition
            .transforms
            .iter()
            .map(|t| &t.statistic)
            .chain(definition.layers.iter().map(|l| &l.statistic))
        {
            if matches!(stat.parameters, StatParameters::Custom(_))
                && !self.stat_descriptor(&stat.operation)?.portable
            {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "A native-only statistic cannot execute or serialize as a portable chart.",
                ));
            }
        }
        Ok(())
    }
}
pub(super) fn validate_descriptor(d: &ExtensionDescriptor) -> ChartResult<()> {
    validate_name(&d.operation.id)?;
    if !d.operation.id.contains('.')
        || d.operation.id.starts_with("chart.")
        || d.operation.version == Revision::INITIAL
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Extensions require a non-builtin qualified name and a positive version.",
        ));
    }
    if d.incremental
        != (IncrementalCapabilities {
            append: false,
            window: false,
            correction: false,
            full_recompute: true,
        })
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "The alpha custom-stat executor supports exact batch recomputation only; specialized update declarations need a verified executor.",
        ));
    }
    Ok(())
}
pub(crate) fn validate_name(name: &str) -> ChartResult<()> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Extension names must be 1..128 ASCII letters, digits, dots, underscores or hyphens.",
        ));
    }
    Ok(())
}
pub(crate) fn parameter_size(value: &serde_json::Value) -> ChartResult<usize> {
    let mut todo = vec![(value, 0)];
    let mut bytes = 0usize;
    let mut nodes = 0usize;
    while let Some((v, depth)) = todo.pop() {
        nodes += 1;
        if depth > 24 || nodes > 4096 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Extension parameters exceed depth/node limits.",
            ));
        }
        let children = match v {
            serde_json::Value::Array(a) => a.len(),
            serde_json::Value::Object(o) => o.len(),
            _ => 0,
        };
        if children.saturating_add(todo.len()).saturating_add(nodes) > 4096 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Extension parameters exceed the node limit.",
            ));
        }
        match v {
            serde_json::Value::String(s) => bytes = bytes.saturating_add(s.len()),
            serde_json::Value::Array(a) => todo.extend(a.iter().map(|v| (v, depth + 1))),
            serde_json::Value::Object(o) => {
                bytes = bytes.saturating_add(o.keys().map(String::len).sum::<usize>());
                todo.extend(o.values().map(|v| (v, depth + 1)));
            }
            _ => {
                bytes = bytes.saturating_add(16);
            }
        }
        if bytes > 65536 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Extension parameters exceed 64 KiB.",
            ));
        }
    }
    Ok(bytes)
}
pub(crate) fn validate_parameters(
    p: &ExtensionParameters,
    limits: CompileLimits,
) -> ChartResult<()> {
    parameter_size(&p.values)?;
    // Reuse the builtin transform validation, including version/finite checks.
    let mut b = BinSpec::new(Numeric::Literal(0.), vec![0., 1.]);
    b.space = p.space.clone();
    stats::validate_stat(&Statistic::bin(b), limits)?;
    Ok(())
}
pub(crate) fn custom_schema(
    registry: &ExtensionRegistry,
    stat: &Statistic,
    data: &DatasetSnapshot,
    p: &ExtensionParameters,
    limits: CompileLimits,
) -> ChartResult<Vec<StatColumn>> {
    validate_parameters(p, limits)?;
    stats::validate_group(data, &p.grouping)?;
    let fields = registry.stat(&stat.operation)?.schema(data, p, limits)?;
    if fields.is_empty() || fields.len() > 64 || fields.len() > limits.max_prepared_rows {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Custom generated schema requires 1..64 bounded fields.",
        ));
    }
    let mut names = BTreeSet::new();
    for f in &fields {
        let name = match &f.field {
            StatField::Custom(n) => {
                validate_name(n)?;
                n.clone()
            }
            StatField::Count => "@count".into(),
            StatField::Group => "@group".into(),
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "A custom schema must use named custom fields or count/group.",
                ));
            }
        };
        if !names.insert(name) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Custom generated fields must be unique.",
            ));
        }
        validate_custom_space(&f.space, f.kind)?;
        match (&f.field, f.kind) {
            (StatField::Custom(_), GeneratedKind::Float64)
            | (StatField::Count, GeneratedKind::UInt64)
            | (StatField::Group, GeneratedKind::Categorical) => {}
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Custom field kind disagrees with its declared generated accessor.",
                ));
            }
        }
    }
    Ok(fields)
}
fn validate_custom_space(space: &ValueSpace, kind: GeneratedKind) -> ChartResult<()> {
    if matches!(
        space,
        ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. }
    ) != (kind == GeneratedKind::Categorical)
        || (kind == GeneratedKind::UInt64 && !matches!(space, ValueSpace::Data))
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Custom field kind and calculation space disagree.",
        ));
    }
    let mut current = space;
    let mut depth = 0;
    loop {
        match current {
            ValueSpace::Scaled { input, scale } => {
                depth += 1;
                if depth > 24 {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Custom scale space exceeds its depth budget.",
                    ));
                }
                scale.validate()?;
                current = input;
            }
            ValueSpace::Data => return Ok(()),
            ValueSpace::Timestamp { representation, .. } => {
                if representation.timezone.trim().is_empty() || representation.timezone.len() > 1024
                {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Custom timestamps require bounded explicit timezone metadata.",
                    ));
                }
                return Ok(());
            }
            ValueSpace::NullableCategorical { categories } => {
                if depth != 0
                    || categories.iter().collect::<BTreeSet<_>>().len() != categories.len()
                {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Nullable category catalogs must be unique and untransformed.",
                    ));
                }
                crate::limits::require_within(
                    categories.len() <= 4096
                        && categories
                            .iter()
                            .flatten()
                            .fold(0usize, |n, s| n.saturating_add(s.len()))
                            <= 65536,
                    "nullable category metadata",
                )?;
                return Ok(());
            }
            ValueSpace::Categorical { categories } => {
                if depth != 0 {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Numeric transforms cannot wrap category spaces.",
                    ));
                }
                if categories.len() > 4096
                    || categories
                        .iter()
                        .fold(0usize, |n, s| n.saturating_add(s.len()))
                        > 65536
                {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Custom category metadata exceeds its budget.",
                    ));
                }
                if categories.iter().collect::<BTreeSet<_>>().len() != categories.len() {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Custom category catalogs must be unique.",
                    ));
                }
                return Ok(());
            }
            ValueSpace::Transformed { input, transform } => {
                depth += 1;
                if depth > 24 {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Custom calculation space exceeds its depth budget.",
                    ));
                }
                super::statistics::validate_space(&StatSpace::Transformed(transform.clone()))?;
                current = input;
            }
        }
    }
}
pub(crate) fn run_custom(
    registry: &ExtensionRegistry,
    table: &mut PreparedTable,
    data: &DatasetSnapshot,
    request: &stats::StatRequest<'_>,
    limits: CompileLimits,
    counts: &mut PopulationCounts,
    diagnostics: &mut Vec<crate::Diagnostic>,
) -> ChartResult<()> {
    let StatParameters::Custom(p) = &request.stat.parameters else {
        return Err(error(
            DiagnosticCode::Validation,
            "Custom executor requires custom parameters.",
        ));
    };
    let PreparedRows::Source(rows) = &table.rows else {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Custom statistics require source observations; generated values have their own schema.",
        ));
    };
    let fields = custom_schema(registry, request.stat, data, p, limits)?;
    let output = registry
        .stat(&request.stat.operation)?
        .evaluate(CustomStatInput {
            data,
            rows,
            scope: request.scope,
            operation: &request.stat.operation,
            parameters: p,
            invalid: request.policy,
            limits,
        })?;
    if output.rows.len().saturating_mul(fields.len()) > limits.max_prepared_rows
        || output.invalid_rows.len() > rows.len()
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Custom statistic exceeded the prepared output budget.",
        ));
    }
    let eligible: BTreeSet<_> = rows.iter().map(|r| r.key).collect();
    let invalid: BTreeSet<_> = output.invalid_rows.iter().copied().collect();
    if invalid.len() != output.invalid_rows.len() || !invalid.is_subset(&eligible) {
        return Err(error(
            DiagnosticCode::Validation,
            "Custom invalid-row accounting must name distinct eligible source keys.",
        ));
    }
    let mut targets = BTreeSet::new();
    let mut memberships = 0usize;
    for r in &output.rows {
        memberships = memberships.saturating_add(r.members.len());
        if memberships > limits.max_vertices {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Custom memberships exceed the preparation budget.",
            ));
        }
        let members: BTreeSet<_> = r.members.iter().copied().collect();
        if members.len() != r.members.len()
            || !members.is_subset(&eligible)
            || !members.is_disjoint(&invalid)
            || r.count != r.members.len() as u64
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Custom rows must retain accurate distinct eligible membership and counts.",
            ));
        }
        let identity = match &r.target {
            Target::Aggregate {
                id,
                group,
                input,
                members: m,
            } if *input == table.input && !group.is_empty() && m == &r.members => {
                format!("a/{}/{group}", id.get())
            }
            Target::Derived {
                id,
                model,
                model_version,
                inputs,
            } if model == &request.stat.operation.id
                && model_version == &request.stat.operation.version
                && inputs == &[table.input] =>
            {
                format!("d/{}/{model}", id.get())
            }
            _ => {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Custom generated provenance must be an accurate aggregate or the registered derived model, never a source row.",
                ));
            }
        };
        if !targets.insert(identity) {
            return Err(error(
                DiagnosticCode::Validation,
                "Custom output targets must have distinct identities.",
            ));
        }
        let numeric: Vec<_> = fields
            .iter()
            .filter(|f| matches!(f.field, StatField::Custom(_)))
            .collect();
        if r.values.len() != numeric.len() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Custom values do not match their generated schema.",
            ));
        }
        for f in numeric {
            let values: Vec<_> = r.values.iter().filter(|v| v.field == f.field).collect();
            if values.len() != 1
                || values[0].value.is_some_and(|v| !v.is_finite())
                || (!f.nullable && values[0].value.is_none())
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Custom values must be unique, finite and honor declared nullability.",
                ));
            }
        }
    }
    counts.invalid_stat = invalid.len();
    stats::warning(
        request.policy,
        invalid.len(),
        output.invalid_rows.into_iter().take(32).collect(),
        format!(
            "Custom statistic excluded {} invalid required rows.",
            invalid.len()
        ),
        diagnostics,
    )?;
    table.rows = PreparedRows::Statistical(output.rows.into());
    table.schema = OutputSchema::Custom {
        operation: request.stat.operation.clone(),
        fields,
    };
    Ok(())
}
