//! Explicitly installed transforms with immutable prepared state.
use super::{
    ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions,
    registry::{VersionedMap, reject_native_only},
};
use crate::{ChartResult, DiagnosticCode, interpolate::Number};
use std::sync::Arc;

/// A versioned transform selection. Deserialization does not install executable code.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformOperation {
    /// Exact installed identity.
    pub operation: OperationRef,
    /// Bounded declarative configuration.
    pub parameters: serde_json::Value,
}

/// Prepared pure arithmetic with owned, length-preserving population operations.
/// Implementations retain IEEE missing/infinite values and own all required state.
/// Core cannot preempt trusted native code or undo its side effects.
pub trait PreparedTransform: Send + Sync {
    /// Forward arithmetic, including exceptional values.
    fn forward(&self, value: f64) -> f64;
    /// Inverse arithmetic, including exceptional values.
    fn inverse(&self, value: f64) -> f64;
    /// Whether arithmetic is independent of the surrounding population.
    /// Population-dependent kernels must return false and override both batch methods.
    fn is_pointwise(&self) -> bool {
        true
    }
    /// Forward an ordered population, retaining missing and infinite entries.
    /// Output must have exactly the input length, including for empty input.
    fn forward_batch(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
        Ok(values.iter().map(|v| self.forward(*v)).collect())
    }
    /// Invert one ordered population; this need not invert a different-sized batch.
    /// Output must have exactly the input length, including for empty input.
    fn inverse_batch(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
        Ok(values.iter().map(|v| self.inverse(*v)).collect())
    }
    /// Transform NULL. None preserves NULL; Some([]) is a typed empty vector.
    /// The result must remain empty when present.
    fn forward_null(&self) -> ChartResult<Option<Vec<f64>>> {
        self.forward_batch(&[]).map(Some)
    }
    /// Invert an untrained NULL domain. None preserves NULL; Some([]) is a
    /// typed empty numeric vector. The result must remain empty when present.
    fn inverse_null(&self) -> ChartResult<Option<Vec<f64>>> {
        self.inverse_batch(&[]).map(Some)
    }
    /// Ordered source domain used for guide generation; infinite endpoints are allowed.
    fn domain(&self) -> [Number; 2];
    /// Declare whether this source interval has a continuous one-to-one branch.
    fn monotone_on(&self, bounds: [f64; 2]) -> bool;
    /// Optional whole-population rejection before missing values are removed.
    fn validate_population(&self, _values: &[f64]) -> ChartResult<()> {
        Ok(())
    }
    /// Default source-space major breaks. None selects the shared extended algorithm.
    fn breaks(&self, _limits: [f64; 2], _count: f64) -> ChartResult<Option<Vec<Number>>> {
        Ok(None)
    }
    /// Default minor breaks in transformed coordinates, including expanded limits.
    /// Receives finite major breaks and the requested subdivisions (currently two).
    /// None selects shared regular minor breaks; compositions reset this callback.
    fn minor_breaks(
        &self,
        _major: &[Number],
        _limits: [Number; 2],
        _subdivisions: usize,
    ) -> ChartResult<Option<Vec<Number>>> {
        Ok(None)
    }
    /// Default source-space labels. None selects the shared reference number formatter.
    fn labels(&self, _values: &[Number]) -> ChartResult<Option<Vec<Option<String>>>> {
        Ok(None)
    }
}

/// Source-compatible name for existing pointwise kernel implementations.
pub use PreparedTransform as PointwiseTransform;

/// Trusted factory, compiled once per resolved descriptor and retained by owned copies.
pub trait CustomTransformFactory: Send + Sync {
    /// Immutable identity and portable capability captured at registration.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate configuration without executing arithmetic.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Create an owned transform kernel without retaining borrowed host state.
    fn compile(&self, parameters: &serde_json::Value) -> ChartResult<Arc<dyn PointwiseTransform>>;
}
#[derive(Clone, Debug, Default)]
pub(crate) struct TransformRegistrations(VersionedMap<Arc<dyn CustomTransformFactory>>);
impl TransformRegistrations {
    fn registration(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<&super::registry::VersionedEntry<Arc<dyn CustomTransformFactory>>> {
        self.0.get_fmt(operation, || {
            format!(
                "Transform {} version {} is not registered.",
                operation.id,
                operation.version.get()
            )
        })
    }
    pub(crate) fn validate(&self, call: &TransformOperation, portable: bool) -> ChartResult<()> {
        extensions::parameter_size(&call.parameters)?;
        let registration = self.registration(&call.operation)?;
        reject_native_only(
            portable,
            registration.descriptor.portable,
            "A native-only transform cannot execute in portable publication.",
        )?;
        registration.implementation.validate(&call.parameters)
    }
    pub(crate) fn compile(
        &self,
        call: &TransformOperation,
        portable: bool,
    ) -> ChartResult<RegisteredTransform> {
        self.validate(call, portable)?;
        let registration = self.registration(&call.operation)?;
        let kernel = registration.implementation.compile(&call.parameters)?;
        let domain = kernel.domain();
        if domain.iter().any(|v| v.0.is_nan()) || domain[0].0 > domain[1].0 {
            return Err(super::error(
                DiagnosticCode::NumericalDomain,
                "Registered transform domain must be ordered and comparable.",
            ));
        }
        Ok(RegisteredTransform {
            descriptor: registration.descriptor.clone(),
            kernel,
            domain,
        })
    }
}
/// Captured execution state, never transported as code.
#[derive(Clone)]
pub(crate) struct RegisteredTransform {
    descriptor: ExtensionDescriptor,
    pub(crate) kernel: Arc<dyn PointwiseTransform>,
    pub(crate) domain: [Number; 2],
}
impl std::fmt::Debug for RegisteredTransform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}
impl RegisteredTransform {
    pub(crate) fn portable(&self) -> bool {
        self.descriptor.portable
    }
}
impl ExtensionRegistry {
    /// Install one exact transform version; previously captured registries stay unchanged.
    pub fn register_transform(
        &mut self,
        implementation: Arc<dyn CustomTransformFactory>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        Arc::make_mut(&mut self.transforms_function).0.insert(
            descriptor,
            implementation,
            "A transform version is already registered.",
            "registered transform",
        )
    }
    /// Read a captured descriptor without executing its factory.
    pub fn transform_descriptor(
        &self,
        operation: &OperationRef,
    ) -> ChartResult<ExtensionDescriptor> {
        Ok(self
            .transforms_function
            .registration(operation)?
            .descriptor
            .clone())
    }
}

/// Portable selection plus private captured state. Decoded selections require resolution.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformSelection {
    /// Authored operation and parameters.
    pub(crate) call: TransformOperation,
    #[serde(skip)]
    pub(crate) prepared: Option<RegisteredTransform>,
}
impl PartialEq for TransformSelection {
    fn eq(&self, other: &Self) -> bool {
        self.call == other.call
    }
}
impl TransformSelection {
    /// Author a selection without executing a factory.
    pub fn new(call: TransformOperation) -> Self {
        Self {
            call,
            prepared: None,
        }
    }
    /// Read the immutable authored selection.
    pub fn call(&self) -> &TransformOperation {
        &self.call
    }
    pub(crate) fn validate_authoring(&self) -> ChartResult<()> {
        extensions::parameter_size(&self.call.parameters)?;
        extensions::validate_descriptor(&ExtensionDescriptor::batch(
            &self.call.operation.id,
            self.call.operation.version,
            true,
        ))
    }
    pub(crate) fn checked(&self) -> ChartResult<&RegisteredTransform> {
        self.prepared.as_ref().ok_or_else(||super::error(DiagnosticCode::UnsupportedCapability,"Registered transform must be resolved through its explicit registry before execution."))
    }
}
impl ExtensionRegistry {
    /// Resolve all transform selections in an owned transform, including compositions.
    pub fn resolve_transform(
        &self,
        transform: &crate::scales::GgplotTransform,
    ) -> ChartResult<crate::scales::GgplotTransform> {
        transform.validate_authoring()?;
        let mut transform = transform.clone();
        transform.resolve_registrations(&self.transforms_function, false)?;
        transform.validate()?;
        Ok(transform)
    }
}
