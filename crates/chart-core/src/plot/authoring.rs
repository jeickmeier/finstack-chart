//! Explicit typed recipe dispatch; callbacks compose the existing primary builders.
use super::*;
use crate::grammar::{ExtensionDescriptor, OperationRef};
/// An installed authoring recipe and its immutable declarative parameters.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoringSelection {
    /// Exact registered identity/version, never executable code.
    pub operation: OperationRef,
    /// Parameters interpreted only by the installed implementation.
    pub parameters: serde_json::Value,
}
/// Source/schema context for one explicit autolayer dispatch.
pub struct AuthoringInput<'a> {
    /// Immutable, already normalized source and its exact field handles.
    pub data: &'a Data,
    /// Validated declarative recipe controls.
    pub parameters: &'a serde_json::Value,
    /// Work limits inherited by the resulting common compiler.
    pub limits: CompileLimits,
}
/// Application-installed data adapter and layer recipe. Calls execute during authoring,
/// before preparation, and must not retain host objects or use implicit I/O/global state.
pub trait CustomAuthoring: Send + Sync {
    /// Exact identity, portability and batch invalidation policy.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate the recipe parameter schema independently of source observations.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Convert an explicitly supplied typed payload using the ordinary Data builder.
    /// Payloads are bounded JSON descriptions, never a language object to evaluate.
    fn materialize(&self, _payload: &serde_json::Value, _limits: DataLimits) -> ChartResult<Data> {
        Err(error(
            DiagnosticCode::UnsupportedCapability,
            "This recipe does not register a materializer.",
        ))
    }
    /// Compose ordinary layers over owner-scoped fields; core resolves and validates them.
    fn layers(&self, input: AuthoringInput<'_>) -> ChartResult<Vec<LayerBuilder>>;
}
#[derive(Clone)]
struct Entry {
    descriptor: ExtensionDescriptor,
    implementation: Arc<dyn CustomAuthoring>,
}
#[derive(Clone, Default)]
pub(crate) struct AuthoringRegistrations {
    entries: BTreeMap<(String, u64), Entry>,
}
impl ExtensionRegistry {
    /// Add an immutable authoring adapter; JSON cannot create registrations.
    pub fn register_authoring(
        &mut self,
        implementation: Arc<dyn CustomAuthoring>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        crate::grammar::validate_extension_descriptor(&descriptor)?;
        let entries = &mut Arc::make_mut(&mut self.authoring).entries;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        if entries.contains_key(&key) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Authoring version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "authoring registrations")?;
        entries.insert(
            key,
            Entry {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }
    fn authoring_entry(&self, operation: &OperationRef, portable: bool) -> ChartResult<&Entry> {
        let entry = self
            .authoring
            .entries
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| {
                error(
                    DiagnosticCode::UnsupportedCapability,
                    "Authoring recipe is not registered.",
                )
            })?;
        if portable && !entry.descriptor.portable {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Native-only authoring cannot dispatch in a portable host.",
            ));
        }
        Ok(entry)
    }
    /// Materialize an explicit application payload once, validating the returned data again.
    pub fn materialize(
        &self,
        operation: &OperationRef,
        payload: &serde_json::Value,
        limits: DataLimits,
        portable: bool,
    ) -> ChartResult<Data> {
        let size = serde_json::to_vec(payload)
            .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?
            .len();
        crate::limits::require_within(
            size <= limits.max_batch_bytes,
            "materialization payload bytes",
        )?;
        let result = self
            .authoring_entry(operation, portable)?
            .implementation
            .materialize(payload, limits)?;
        result.batch().validate(limits)?;
        Ok(result)
    }
    /// Resolve an explicit recipe to ordinary layers; source ownership is pinned on each layer.
    pub fn autolayers(
        &self,
        data: &Data,
        selection: &AuthoringSelection,
        limits: CompileLimits,
        portable: bool,
    ) -> ChartResult<Vec<LayerBuilder>> {
        crate::grammar::extension_parameter_size(&selection.parameters)?;
        let entry = self.authoring_entry(&selection.operation, portable)?;
        entry.implementation.validate(&selection.parameters)?;
        let layers = entry.implementation.layers(AuthoringInput {
            data,
            parameters: &selection.parameters,
            limits,
        })?;
        crate::limits::require_within(layers.len() <= limits.max_layers, "autolayer output")?;
        Ok(layers.into_iter().map(|l| l.data(data.clone())).collect())
    }
}
impl PlotBuilder {
    /// Append the registered recipe to this builder without replacing existing components.
    pub fn autolayer(mut self, selection: AuthoringSelection, portable: bool) -> ChartResult<Self> {
        let layers =
            self.extensions
                .autolayers(&self.data, &selection, self.compile_limits, portable)?;
        crate::limits::require_within(
            self.layers.len().saturating_add(layers.len()) <= self.compile_limits.max_layers,
            "composed autolayers",
        )?;
        self.layers.extend(layers);
        Ok(self)
    }
}
/// Start the ordinary Plot builder with an explicitly registered recipe and source.
pub fn autoplot(
    data: Data,
    selection: AuthoringSelection,
    registry: Arc<ExtensionRegistry>,
    portable: bool,
) -> ChartResult<PlotBuilder> {
    plot(data)
        .extensions(registry)
        .autolayer(selection, portable)
}
