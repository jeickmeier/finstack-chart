//! Native-only painter callbacks are explicit host-owned resources, never core objects.
use chart_core::grammar::OperationRef;
use chart_core::scene::Color;
use chart_core::{ChartResult, Diagnostic, DiagnosticCode};
use gpui::{App, Bounds, Pixels, Window};
use std::{collections::BTreeMap, rc::Rc};

/// Immutable paint data prepared once for a frame. The chart applies its content clip.
pub trait PreparedNativePaint {
    /// Submit native drawing inside the current chart/item clip; no statistical work belongs here.
    fn paint(&self, window: &mut Window, cx: &mut App);
}
/// Caller-owned native implementation selected by an exact scene descriptor.
pub trait NativePainter {
    /// Stable implementation identity/version, independent of the data revision.
    fn operation(&self) -> OperationRef;
    /// Validate parameters and construct immutable frame-local paint data before any submission.
    fn prepare(
        &self,
        bounds: Bounds<Pixels>,
        parameters: &serde_json::Value,
        color: Color,
    ) -> ChartResult<Rc<dyn PreparedNativePaint>>;
}
/// Bounded host registry. Core stores only operation IDs and numeric/JSON paint descriptors.
#[derive(Clone, Default)]
pub struct NativePainterRegistry(BTreeMap<(String, u64), Rc<dyn NativePainter>>);
impl NativePainterRegistry {
    /// Empty native capability set.
    pub fn new() -> Self {
        Self::default()
    }
    /// Register an exact native implementation; duplicates and invalid identities reject.
    pub fn register(&mut self, painter: Rc<dyn NativePainter>) -> ChartResult<()> {
        let op = painter.operation();
        chart_core::grammar::validate_native_paint(&op, &serde_json::Value::Null)?;
        let key = (op.id, op.version.get());
        if self.0.contains_key(&key) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Native painter version is already registered.",
            ));
        }
        if self.0.len() >= 64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Native painter registry exceeds 64 entries.",
            ));
        }
        self.0.insert(key, painter);
        Ok(())
    }
    /// Query the exact installed host capability without calling it.
    pub fn contains(&self, operation: &OperationRef) -> bool {
        self.0
            .contains_key(&(operation.id.clone(), operation.version.get()))
    }
    pub(crate) fn get(&self, operation: &OperationRef) -> ChartResult<&dyn NativePainter> {
        self.0
            .get(&(operation.id.clone(), operation.version.get()))
            .map(Rc::as_ref)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "Native painter {} version {} is not installed in this host.",
                        operation.id,
                        operation.version.get()
                    ),
                )
            })
    }
}
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Register the exact native painter before mounting, or use portable geometry for this layer.",
    )
}
