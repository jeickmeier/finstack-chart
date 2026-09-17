//! Registered facet planning feeds the existing population, training and layout engine.
use super::{CompileLimits, ExtensionDescriptor, ExtensionRegistry, FacetSpec, OperationRef};
use crate::{ChartResult, DiagnosticCode};
use std::{collections::BTreeMap, sync::Arc};
/// Explicit immutable facet planning selection.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacetSelection {
    /// Installed identity and version.
    pub operation: OperationRef,
    /// Bounded declarative parameters.
    pub parameters: serde_json::Value,
}
/// Coherent source and already resolved ordinary facet catalog.
pub struct FacetPlanInput<'a> {
    /// Built-in catalog includes exact field and missing/margin identities.
    pub base: &'a FacetSpec,
    /// Source snapshot used by this preparation, without host objects.
    pub source: &'a crate::data::StoreSnapshot,
    /// Checked explicit configuration.
    pub parameters: &'a serde_json::Value,
    /// Resource budgets applied again to the returned plan.
    pub limits: CompileLimits,
}
/// Trusted facet planner; returned plans share ordinary population and update semantics.
pub trait CustomFacet: Send + Sync {
    /// Exact registered identity, portability and batch invalidation.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate parameters independently of source data.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Return a concrete plan over the same ordered facet variables.
    fn plan(&self, input: FacetPlanInput<'_>) -> ChartResult<FacetSpec>;
}
#[derive(Clone)]
struct Entry {
    descriptor: ExtensionDescriptor,
    implementation: Arc<dyn CustomFacet>,
}
#[derive(Clone, Default)]
pub(crate) struct FacetRegistrations {
    entries: BTreeMap<(String, u64), Entry>,
}
impl FacetRegistrations {
    fn get(&self, s: &FacetSelection) -> ChartResult<&Entry> {
        self.entries
            .get(&(s.operation.id.clone(), s.operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    "Selected facet implementation is not registered.",
                )
            })
    }
    pub(crate) fn validate(&self, s: &FacetSelection, portable: bool) -> ChartResult<()> {
        super::extensions::parameter_size(&s.parameters)?;
        let e = self.get(s)?;
        if portable && !e.descriptor.portable {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "Native-only facet planner cannot execute portably.",
            ));
        }
        e.implementation.validate(&s.parameters)
    }
}
impl ExtensionRegistry {
    /// Install one immutable facet version. Replacing a registry cannot mutate old snapshots.
    pub fn register_facet(&mut self, implementation: Arc<dyn CustomFacet>) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        super::extensions::validate_descriptor(&descriptor)?;
        let entries = &mut Arc::make_mut(&mut self.facets).entries;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        if entries.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "Facet version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "facet registrations")?;
        entries.insert(
            key,
            Entry {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }
    pub(crate) fn validate_facet_selection(
        &self,
        definition: &super::ChartDefinition,
        portable: bool,
    ) -> ChartResult<()> {
        if let Some(selection) = definition
            .facets
            .as_ref()
            .and_then(|f| f.reference.as_ref())
            .and_then(|p| p.registered.as_ref())
        {
            self.facets.validate(selection, portable)?;
        }
        Ok(())
    }
}
pub(crate) fn resolve<'a>(
    definition: std::borrow::Cow<'a, super::ChartDefinition>,
    source: &crate::data::StoreSnapshot,
    limits: CompileLimits,
    registry: &ExtensionRegistry,
) -> ChartResult<std::borrow::Cow<'a, super::ChartDefinition>> {
    let Some(base) = definition.facets.as_ref() else {
        return Ok(definition);
    };
    let Some(selection) = base.reference.as_ref().and_then(|p| p.registered.as_ref()) else {
        return Ok(definition);
    };
    registry.facets.validate(selection, false)?;
    let plan = registry
        .facets
        .get(selection)?
        .implementation
        .plan(FacetPlanInput {
            base,
            source,
            parameters: &selection.parameters,
            limits,
        })?;
    if plan.fields != base.fields
        || plan.reference.as_ref().map(|p| &p.field_names)
            != base.reference.as_ref().map(|p| &p.field_names)
        || plan.reference.as_ref().and_then(|p| p.registered.as_ref()) != Some(selection)
    {
        return Err(super::error(
            DiagnosticCode::SchemaConflict,
            "Facet planner must preserve field identities, names and its registered selection.",
        ));
    }
    // Full shared validation checks unique keys, population coverage and grid topology.
    let mut result = definition.into_owned();
    result.facets = Some(plan);
    super::facets::validate_facets(&result, source, limits, registry)?;
    Ok(std::borrow::Cow::Owned(result))
}
