//! External hierarchy callbacks used by native and actual portable proof hosts.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    grammar::{CustomHierarchyOperation, ExtensionDescriptor, ExtensionRegistry, HierarchyScalar},
    hierarchy::{HierarchyNodeId, NodeView},
};
use serde_json::Value;
use std::{cmp::Ordering, sync::Arc};
/// Register one portable accessor family and a native-only counterpart.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_hierarchy(Arc::new(Accessor { portable: true }))?;
    registry.register_hierarchy(Arc::new(Accessor { portable: false }))
}
/// Pure external operations, with no access to private topology fields.
pub struct Accessor {
    /// Captured portability at registration time.
    pub portable: bool,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    mode: Mode,
}
#[derive(serde::Deserialize)]
enum Mode {
    Children,
    Value,
    DepthPadding,
    DescendingValue,
    DepthSeparation,
    FindC,
    FindDepthOne,
    EqualTile,
    InvalidTile,
}
fn parameters(p: &Value) -> ChartResult<Parameters> {
    serde_json::from_value(p.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
fn wrong<T>() -> ChartResult<T> {
    Err(super::error(
        DiagnosticCode::UnsupportedCapability,
        "Hierarchy example mode does not match the invoked accessor.",
    ))
}
impl CustomHierarchyOperation for Accessor {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                "example.hierarchy"
            } else {
                "example.native_hierarchy"
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, p: &Value) -> ChartResult<()> {
        parameters(p).map(|_| ())
    }
    fn children(
        &self,
        _: HierarchyNodeId,
        data: &Arc<Value>,
        p: &Value,
    ) -> ChartResult<Vec<(HierarchyNodeId, Arc<Value>)>> {
        if !matches!(parameters(p)?.mode, Mode::Children) {
            return wrong();
        }
        let children = match data.get("children") {
            None | Some(Value::Null) => return Ok(vec![]),
            Some(Value::Array(a)) => a,
            _ => return wrong(),
        };
        children
            .iter()
            .map(|child| {
                let key = child.get("key").ok_or_else(|| {
                    super::error(
                        DiagnosticCode::Validation,
                        "Custom children require explicit stable keys.",
                    )
                })?;
                let key = serde_json::from_value(key.clone())
                    .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))?;
                Ok((key, Arc::new(child.clone())))
            })
            .collect()
    }
    fn scalar(
        &self,
        node: NodeView<'_>,
        _: HierarchyScalar,
        p: &Value,
    ) -> ChartResult<Option<f64>> {
        Ok(Some(match parameters(p)?.mode {
            Mode::Value => node
                .data()
                .get("value")
                .and_then(Value::as_f64)
                .unwrap_or(0.),
            Mode::DepthPadding => node.depth() as f64 + 1.,
            _ => return wrong(),
        }))
    }
    fn compare(&self, a: NodeView<'_>, b: NodeView<'_>, p: &Value) -> ChartResult<Ordering> {
        if !matches!(parameters(p)?.mode, Mode::DescendingValue) {
            return wrong();
        }
        b.value().partial_cmp(&a.value()).ok_or_else(|| {
            super::error(
                DiagnosticCode::NumericalDomain,
                "Invalid example comparison.",
            )
        })
    }
    fn separation(&self, a: NodeView<'_>, b: NodeView<'_>, p: &Value) -> ChartResult<f64> {
        if !matches!(parameters(p)?.mode, Mode::DepthSeparation) {
            return wrong();
        }
        Ok(
            if a.parent().map(NodeView::handle) == b.parent().map(NodeView::handle) {
                1.
            } else {
                2.
            } / a.depth().max(1) as f64,
        )
    }
    fn predicate(
        &self,
        node: NodeView<'_>,
        index: usize,
        root: NodeView<'_>,
        p: &Value,
    ) -> ChartResult<bool> {
        let mode = parameters(p)?.mode;
        if matches!(mode, Mode::FindDepthOne) {
            return Ok(index > 0 && node.handle() != root.handle() && node.depth() == 1);
        }
        if !matches!(mode, Mode::FindC) {
            return wrong();
        }
        // Exercise all callback context fields as well as the selected payload.
        Ok(index > 0
            && node.handle() != root.handle()
            && node.data().get("name") == Some(&Value::String("C".into())))
    }
    fn tile(
        &self,
        parent: NodeView<'_>,
        [x0, y0, x1, y1]: [f64; 4],
        p: &Value,
    ) -> ChartResult<Vec<[f64; 4]>> {
        match parameters(p)?.mode {
            Mode::InvalidTile => Ok(vec![]),
            Mode::EqualTile => {
                let count = parent.children().len();
                Ok((0..count)
                    .map(|i| {
                        [
                            x0 + (x1 - x0) * i as f64 / count as f64,
                            y0,
                            x0 + (x1 - x0) * (i + 1) as f64 / count as f64,
                            y1,
                        ]
                    })
                    .collect())
            }
            _ => wrong(),
        }
    }
}
