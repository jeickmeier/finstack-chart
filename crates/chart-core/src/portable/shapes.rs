//! JSON ownership boundary for shape data that must retain arbitrary source metadata.
use super::{decode, encode};
use crate::{ChartResult, shape::Pie};
/// Layout materialized pie values and return owned source data without host arithmetic.
pub fn pie_layout_json(pie: &Pie, data: &str, values: &str) -> ChartResult<String> {
    let data: Vec<serde_json::Value> = decode(data)?;
    let values: Vec<f64> = decode(values)?;
    encode(&pie.layout_materialized(&data, &values)?)
}

/// Layout a sample-by-key matrix while preserving arbitrary source metadata.
pub fn stack_layout_json(
    stack: &crate::shape::Stack,
    data: &str,
    values: &str,
) -> ChartResult<String> {
    let data: Vec<serde_json::Value> = decode(data)?;
    let values: Vec<Vec<Option<f64>>> = decode(values)?;
    encode(&stack.layout_materialized(&data, &values)?)
}

/// Resolve a portable comparator before laying out exact JSON data in the shared core.
pub fn pie_layout_registered_json(
    pie: &Pie,
    data: &str,
    values: &str,
    registry: &crate::grammar::ExtensionRegistry,
    selection: &str,
) -> ChartResult<String> {
    let data: Vec<serde_json::Value> = decode(data)?;
    let values: Vec<f64> = decode(values)?;
    let selection = decode(selection)?;
    encode(&pie.layout_registered(&data, &values, registry, &selection)?)
}

/// Resolve optional portable stack protocols while preserving exact source metadata.
pub fn stack_layout_registered_json(
    stack: &crate::shape::Stack,
    data: &str,
    values: &str,
    registry: &crate::grammar::ExtensionRegistry,
    order: &str,
    offset: &str,
) -> ChartResult<String> {
    let data: Vec<serde_json::Value> = decode(data)?;
    let values: Vec<Vec<Option<f64>>> = decode(values)?;
    let order: Option<crate::grammar::ShapeOperation> = decode(order)?;
    let offset: Option<crate::grammar::ShapeOperation> = decode(offset)?;
    encode(&stack.layout_registered(&data, &values, registry, order.as_ref(), offset.as_ref())?)
}
