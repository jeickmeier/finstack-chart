use super::{AesBuilder, Data, FilterBuilder, StatBuilder, error, fresh_id, validate_name};
use crate::{ChartResult, DiagnosticCode, TransformId, data::InvalidPolicy, grammar::*};
use std::collections::BTreeMap;

/// Stable graph-node identity, preserved across clones and definition edits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransformHandle(pub(super) TransformId);
impl TransformHandle {
    /// Exact identity for immutable prepared graph inspection.
    pub fn id(self) -> TransformId {
        self.0
    }
}
/// A named or typed reference to another authored transform.
#[derive(Clone, Debug)]
pub enum TransformRef {
    /// Resolve a name within the plot.
    Name(String),
    /// Require this exact transform identity within the plot.
    Handle(TransformHandle),
}
impl From<&str> for TransformRef {
    fn from(value: &str) -> Self {
        Self::Name(value.into())
    }
}
impl From<String> for TransformRef {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}
impl From<TransformHandle> for TransformRef {
    fn from(value: TransformHandle) -> Self {
        Self::Handle(value)
    }
}
impl TransformRef {
    pub(super) fn resolve(
        &self,
        names: &BTreeMap<String, TransformId>,
    ) -> ChartResult<TransformId> {
        match self {
            Self::Name(name) => names.get(name).copied(),
            Self::Handle(handle) => names.values().find(|id| **id == handle.0).copied(),
        }
        .ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("Unknown transform {self:?}."),
            )
        })
    }
}
/// Shared acyclic computation that ordinary layers consume without repeating its kernel.
#[derive(Clone)]
pub struct TransformBuilder {
    pub(super) id: ChartResult<TransformId>,
    pub(super) name: String,
    pub(super) data: Option<Data>,
    pub(super) input: Option<TransformRef>,
    pub(super) statistic: StatBuilder,
    mappings: AesBuilder,
    filters: Vec<FilterBuilder>,
    facet: FacetTarget,
    scope: StatScope,
    invalid: InvalidPolicy,
}
/// Declare a shared statistic using default plot data, with an optional named dependency.
pub fn transform(name: impl Into<String>, statistic: StatBuilder) -> TransformBuilder {
    TransformBuilder {
        id: fresh_id().map(TransformId::new),
        name: name.into(),
        data: None,
        input: None,
        statistic,
        mappings: super::aes(),
        filters: vec![],
        facet: FacetTarget::Match,
        scope: StatScope::Group,
        invalid: InvalidPolicy::Exclude,
    }
}
impl TransformBuilder {
    /// Obtain its stable identity before composing dependent layers or transforms.
    pub fn handle(&self) -> ChartResult<TransformHandle> {
        self.id.clone().map(TransformHandle)
    }
    /// Select an owned source dataset instead of the plot default.
    pub fn data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }
    /// Consume a registered transform's prepared output; declaration order is irrelevant.
    pub fn from_transform(mut self, input: impl Into<TransformRef>) -> Self {
        self.input = Some(input.into());
        self
    }
    /// Override inherited source-input/group mappings for this operation.
    pub fn aes(mut self, mappings: AesBuilder) -> Self {
        self.mappings = mappings;
        self
    }
    /// Filter source populations before the statistic; generated-stage misuse rejects.
    pub fn filter(mut self, filter: FilterBuilder) -> Self {
        self.filters.push(filter);
        self
    }
    /// Choose explicit facet population scope.
    pub fn scope(mut self, scope: StatScope) -> Self {
        self.scope = scope;
        self
    }
    /// Select match/broadcast/exact-panel behavior.
    pub fn facet_target(mut self, target: FacetTarget) -> Self {
        self.facet = target;
        self
    }
    /// Choose exclusion with diagnostics or rejection for invalid observations.
    pub fn invalid(mut self, invalid: InvalidPolicy) -> Self {
        self.invalid = invalid;
        self
    }
    pub(super) fn lower(
        &self,
        root: &Data,
        input: DataRef,
        inherited: &AesBuilder,
        profile: Profile,
    ) -> ChartResult<TransformDefinition> {
        validate_name(&self.name)?;
        if self.data.is_some() && self.input.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "A transform takes either source data or a transform dependency.",
            ));
        }
        let mut node = TransformDefinition::new(
            self.id.clone()?,
            input,
            self.statistic
                .lower(root, &self.mappings.merged(inherited, true))?,
        );
        if profile == Profile::Ggplot2_4_0_3 {
            let mapping = self.mappings.merged(inherited, true);
            node.grammar = Some(TransformGrammar {
                source: mapping.resolve(root)?,
                color: mapping.color.as_ref().map(|c| c.field(root)).transpose()?,
                stat_grouping: self.statistic.explicit_grouping(root)?,
            });
        }
        node.filters = self
            .filters
            .iter()
            .map(|f| f.lower(root))
            .collect::<ChartResult<_>>()?;
        node.facet = self.facet.clone();
        node.scope = self.scope;
        node.invalid = self.invalid;
        Ok(node)
    }
}
