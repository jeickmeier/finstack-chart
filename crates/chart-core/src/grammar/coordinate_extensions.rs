//! Registered paired coordinate maps compose with the common post-stat projection.
use super::{
    ExtensionDescriptor, ExtensionRegistry, OperationRef,
    registry::{VersionedMap, reject_native_only},
};
use crate::{ChartResult, DiagnosticCode};
use std::sync::Arc;
/// Explicit versioned coordinate implementation with bounded parameters.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinateSelection {
    /// Registered identity and version.
    pub operation: OperationRef,
    /// Implementation-specific checked schema.
    pub parameters: serde_json::Value,
}
/// Training context in scale calculation units; the mapping itself uses normalized coordinates.
pub struct CoordinateTrainInput<'a> {
    /// Complete trained x/y domains.
    pub domains: [[f64; 2]; 2],
    /// Current x/y views, after explicit coordinate limits.
    pub views: [[f64; 2]; 2],
    /// Checked parameter payload.
    pub parameters: &'a serde_json::Value,
}
/// Immutable trained paired mapping. Forward and inverse act in normalized panel units.
/// Implementations must have no ambient state and perform bounded work per point.
pub trait TrainedCoordinate: Send + Sync + std::fmt::Debug {
    /// Map x/y together; None explicitly omits points outside the mathematical domain.
    fn forward(&self, point: [f64; 2]) -> ChartResult<Option<[f64; 2]>>;
    /// Conservative output bounds for every point in the input rectangle.
    /// Endpoints are ordered minimum/maximum in normalized panel units. None means
    /// the whole rectangle cannot be bounded on one valid domain branch. This bound
    /// drives the shared path subdivision; corner-only samples are insufficient.
    fn bounds(&self, input: [[f64; 2]; 2]) -> ChartResult<Option<[[f64; 2]; 2]>>;
    /// True only when a unique inverse implementation is supplied.
    fn has_inverse(&self) -> bool;
    /// Recover the unique input, or None outside the inverse domain.
    fn inverse(&self, point: [f64; 2]) -> ChartResult<Option<[f64; 2]>>;
}
/// Trusted coordinate factory; all destinations use the same trained map.
pub trait CustomCoordinate: Send + Sync {
    /// Exact identity, portability and batch invalidation contract.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate parameters before compiling any layer.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Train an immutable paired map from explicit domains and views.
    fn train(&self, input: CoordinateTrainInput<'_>) -> ChartResult<Arc<dyn TrainedCoordinate>>;
}
#[derive(Clone, Debug, Default)]
pub(crate) struct CoordinateRegistrations(VersionedMap<Arc<dyn CustomCoordinate>>);
impl CoordinateRegistrations {
    fn get(
        &self,
        s: &CoordinateSelection,
    ) -> ChartResult<&super::registry::VersionedEntry<Arc<dyn CustomCoordinate>>> {
        self.0
            .get(&s.operation, "Coordinate implementation is not registered.")
    }
    pub(crate) fn validate(&self, s: &CoordinateSelection, portable: bool) -> ChartResult<()> {
        super::extensions::parameter_size(&s.parameters)?;
        let e = self.get(s)?;
        reject_native_only(
            portable,
            e.descriptor.portable,
            "Native-only coordinate cannot execute portably.",
        )?;
        e.implementation.validate(&s.parameters)
    }
    pub(crate) fn train(
        &self,
        s: &CoordinateSelection,
        input: CoordinateTrainInput<'_>,
    ) -> ChartResult<Arc<dyn TrainedCoordinate>> {
        self.validate(s, false)?;
        self.get(s)?.implementation.train(input)
    }
}
impl ExtensionRegistry {
    /// Install a trusted immutable coordinate implementation.
    pub fn register_coordinate(
        &mut self,
        implementation: Arc<dyn CustomCoordinate>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        Arc::make_mut(&mut self.coordinates).0.insert(
            descriptor,
            implementation,
            "Coordinate version is already registered.",
            "coordinate registrations",
        )
    }
    pub(crate) fn validate_coordinate_selection(
        &self,
        definition: &super::ChartDefinition,
        portable: bool,
    ) -> ChartResult<()> {
        if let Some(selection) = &definition.coordinate_extension {
            self.coordinates.validate(selection, portable)?;
            if !matches!(
                definition.coordinate,
                Some(super::CoordinateSpec::Cartesian(_))
            ) {
                return Err(super::error(
                    DiagnosticCode::UnsupportedCapability,
                    "Registered paired coordinates require a Cartesian base view; nonlinear behavior belongs to the paired map.",
                ));
            }
        }
        Ok(())
    }
}
