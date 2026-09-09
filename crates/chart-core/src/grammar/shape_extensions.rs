//! Versioned selections for trusted implementations of the shared shape protocols.
use super::{ExtensionDescriptor, ExtensionRegistry, OperationRef, extensions};
use crate::{ChartResult, DiagnosticCode, shape::*};
use std::{collections::BTreeMap, sync::Arc};

/// Protocol selected by a registration. Families cannot substitute for one another.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ShapeFamily {
    /// Line/area lifecycle, also used by radial generators and links.
    Curve,
    /// Drawing into the checked numeric path context.
    Symbol,
    /// Stable comparison of owned pie data and its resolved value.
    PieComparator,
    /// Complete stack rank permutation.
    StackOrder,
    /// Stack baseline and endpoint transformation.
    StackOffset,
}

/// Declarative selection only: no executable callback or dynamically loaded code.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShapeOperation {
    /// Exact registered operation identity and version.
    pub operation: OperationRef,
    /// Bounded, implementation-specific data parameters.
    pub parameters: serde_json::Value,
}

/// A resolved immutable native protocol. It deliberately has no serialization API.
pub enum ShapeProtocol {
    /// Reusable line/area factory; each invocation owns its lifecycle state.
    Curve(Box<dyn CurveFactory + Send + Sync>),
    /// Symbol drawing through the shared checked path.
    Symbol(Box<dyn SymbolDraw + Send + Sync>),
    /// Fallible pie comparison, preserving exact JSON datum fields.
    PieComparator(Box<dyn PieComparator + Send + Sync>),
    /// Validated by the stack generator before offsets execute.
    StackOrder(Box<dyn StackOrdering + Send + Sync>),
    /// Validated by the stack generator before results are returned.
    StackOffset(Box<dyn StackOffsetting + Send + Sync>),
}
impl ShapeProtocol {
    /// The protocol implemented by this resolved value.
    pub fn family(&self) -> ShapeFamily {
        match self {
            Self::Curve(_) => ShapeFamily::Curve,
            Self::Symbol(_) => ShapeFamily::Symbol,
            Self::PieComparator(_) => ShapeFamily::PieComparator,
            Self::StackOrder(_) => ShapeFamily::StackOrder,
            Self::StackOffset(_) => ShapeFamily::StackOffset,
        }
    }
}

/// Explicit trusted registration; portable sessions must opt into the same Rust code.
/// Implementations validate their parameter schema and never retain interpreter objects.
pub trait CustomShape: Send + Sync {
    /// Exact version and portability, captured at registration time.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Protocol identity, captured at registration time.
    fn family(&self) -> ShapeFamily;
    /// Validate bounded parameters and construct a reusable immutable protocol.
    fn resolve(&self, parameters: &serde_json::Value) -> ChartResult<ShapeProtocol>;
}

#[derive(Clone)]
struct Registration {
    descriptor: ExtensionDescriptor,
    family: ShapeFamily,
    implementation: Arc<dyn CustomShape>,
}
#[derive(Clone, Default)]
pub(crate) struct ShapeRegistrations(BTreeMap<(String, u64), Registration>);
impl ExtensionRegistry {
    /// Install one shape implementation, with at most 64 exact shape versions per registry.
    pub fn register_shape(&mut self, implementation: Arc<dyn CustomShape>) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        extensions::validate_descriptor(&descriptor)?;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        if self.shapes.0.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "A shape extension version is already registered.",
            ));
        }
        if self.shapes.0.len() >= 64 {
            return Err(super::error(
                DiagnosticCode::ResourceLimit,
                "At most 64 shape extensions can be registered.",
            ));
        }
        self.shapes.0.insert(
            key,
            Registration {
                descriptor,
                family: implementation.family(),
                implementation,
            },
        );
        Ok(())
    }
    /// Read captured metadata without invoking native implementation code.
    pub fn shape_descriptor(&self, operation: &OperationRef) -> ChartResult<ExtensionDescriptor> {
        Ok(self.shape_registration(operation)?.descriptor.clone())
    }
    fn shape_registration(&self, operation: &OperationRef) -> ChartResult<&Registration> {
        self.shapes
            .0
            .get(&(operation.id.clone(), operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "Shape {} version {} is not registered.",
                        operation.id,
                        operation.version.get()
                    ),
                )
            })
    }
    /// Resolve a native selection, including explicitly native-only implementations.
    pub fn resolve_shape(
        &self,
        selection: &ShapeOperation,
        family: ShapeFamily,
    ) -> ChartResult<ShapeProtocol> {
        self.resolve_shape_impl(selection, family, false)
    }
    /// Resolve for a portable host or publication; native-only selections reject before callbacks.
    pub fn resolve_portable_shape(
        &self,
        selection: &ShapeOperation,
        family: ShapeFamily,
    ) -> ChartResult<ShapeProtocol> {
        self.resolve_shape_impl(selection, family, true)
    }
    fn resolve_shape_impl(
        &self,
        selection: &ShapeOperation,
        family: ShapeFamily,
        portable: bool,
    ) -> ChartResult<ShapeProtocol> {
        extensions::parameter_size(&selection.parameters)?;
        let registration = self.shape_registration(&selection.operation)?;
        if registration.family != family {
            return Err(super::error(
                DiagnosticCode::Validation,
                "Registered shape protocol does not match the requested family.",
            ));
        }
        if portable && !registration.descriptor.portable {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "A native-only shape cannot execute or serialize in a portable session.",
            ));
        }
        let protocol = registration.implementation.resolve(&selection.parameters)?;
        if protocol.family() != family {
            return Err(super::error(
                DiagnosticCode::Validation,
                "Shape registration returned the wrong protocol family.",
            ));
        }
        Ok(protocol)
    }
    /// Validate portability and parameter schema before serializing a shape selection.
    pub fn shape_to_json(
        &self,
        selection: &ShapeOperation,
        family: ShapeFamily,
    ) -> ChartResult<String> {
        self.resolve_portable_shape(selection, family)?;
        serde_json::to_string(selection)
            .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
    }
}

/// Resolved implementations retained by prepared layers and immutable snapshots.
#[derive(Clone, Default)]
pub(crate) struct ResolvedShapes(BTreeMap<ShapeFamily, Arc<ShapeProtocol>>);
impl std::fmt::Debug for ResolvedShapes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.0.keys()).finish()
    }
}
impl ResolvedShapes {
    pub(crate) fn curve(&self) -> Option<&(dyn CurveFactory + Send + Sync)> {
        match self.0.get(&ShapeFamily::Curve)?.as_ref() {
            ShapeProtocol::Curve(p) => Some(p.as_ref()),
            _ => None,
        }
    }
    pub(crate) fn symbol(&self) -> Option<&(dyn SymbolDraw + Send + Sync)> {
        match self.0.get(&ShapeFamily::Symbol)?.as_ref() {
            ShapeProtocol::Symbol(p) => Some(p.as_ref()),
            _ => None,
        }
    }
    pub(super) fn pie_comparator(&self) -> Option<&(dyn PieComparator + Send + Sync)> {
        match self.0.get(&ShapeFamily::PieComparator)?.as_ref() {
            ShapeProtocol::PieComparator(p) => Some(p.as_ref()),
            _ => None,
        }
    }
    pub(super) fn stack_order(&self) -> Option<&(dyn StackOrdering + Send + Sync)> {
        match self.0.get(&ShapeFamily::StackOrder)?.as_ref() {
            ShapeProtocol::StackOrder(p) => Some(p.as_ref()),
            _ => None,
        }
    }
    pub(super) fn stack_offset(&self) -> Option<&(dyn StackOffsetting + Send + Sync)> {
        match self.0.get(&ShapeFamily::StackOffset)?.as_ref() {
            ShapeProtocol::StackOffset(p) => Some(p.as_ref()),
            _ => None,
        }
    }
}
pub(super) fn resolve_layer(
    layer: &super::Layer,
    registry: &ExtensionRegistry,
) -> ChartResult<ResolvedShapes> {
    use super::{Geom, Position};
    let mut result = ResolvedShapes::default();
    if !layer.shape_protocols.is_empty() && layer.geometry_extension.is_some() {
        return Err(super::error(
            DiagnosticCode::UnsupportedCapability,
            "Shape protocols cannot also replace the layer geometry.",
        ));
    }
    for (family, selection) in &layer.shape_protocols {
        let allowed = match family {
            ShapeFamily::Curve => matches!(
                layer.geom,
                Geom::ShapeLine { .. }
                    | Geom::ShapeArea { .. }
                    | Geom::ShapeLineRadial { .. }
                    | Geom::ShapeAreaRadial { .. }
                    | Geom::ShapeLink { .. }
            ),
            ShapeFamily::Symbol => matches!(layer.geom, Geom::ShapeSymbol { .. }),
            ShapeFamily::PieComparator => matches!(layer.geom, Geom::ShapePie { .. }),
            ShapeFamily::StackOrder | ShapeFamily::StackOffset => {
                matches!(layer.position, Position::ShapeStack(_))
            }
        };
        if !allowed {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "Shape protocol is incompatible with this layer geometry or position.",
            ));
        }
        if *family == ShapeFamily::Symbol && layer.symbol.is_some() {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "A custom symbol drawing cannot also select a builtin symbol palette.",
            ));
        }
        let protocol = registry.resolve_shape(selection, *family)?;
        if let ShapeProtocol::Curve(curve) = &protocol
            && curve.command_bound(0).is_none()
        {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "Registered chart curves must declare a command bound for layout preflight.",
            ));
        }
        if matches!(
            layer.geom,
            Geom::ShapeArea { .. } | Geom::ShapeAreaRadial { .. }
        ) && let ShapeProtocol::Curve(curve) = &protocol
            && !curve.supports_area()
        {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "Registered curve does not support the area lifecycle.",
            ));
        }
        result.0.insert(*family, Arc::new(protocol));
    }
    Ok(result)
}
