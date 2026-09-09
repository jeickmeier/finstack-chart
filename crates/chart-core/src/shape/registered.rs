//! Registry adapters invoke the public native protocols without a second shape engine.
use super::*;
use crate::{
    ChartResult,
    grammar::{ExtensionRegistry, ShapeFamily, ShapeOperation, ShapeProtocol},
    path::Path,
};

fn curve(
    registry: &ExtensionRegistry,
    selection: &ShapeOperation,
) -> ChartResult<Box<dyn CurveFactory + Send + Sync>> {
    match registry.resolve_portable_shape(selection, ShapeFamily::Curve)? {
        ShapeProtocol::Curve(factory) => Ok(factory),
        _ => unreachable!("registry validates the family"),
    }
}
impl Line {
    /// Generate materialized rows using an explicitly installed portable curve registration.
    pub fn generate_registered<R: AsRef<[f64]>>(
        &self,
        data: &[R],
        registry: &ExtensionRegistry,
        selection: &ShapeOperation,
    ) -> ChartResult<Path> {
        self.generate_materialized_with(data, curve(registry, selection)?.as_ref())
    }
}
impl Area {
    /// Generate paired boundaries with a registered area-capable curve, preserving gaps.
    pub fn generate_registered<R: AsRef<[f64]>>(
        &self,
        data: &[R],
        registry: &ExtensionRegistry,
        selection: &ShapeOperation,
    ) -> ChartResult<Path> {
        self.generate_materialized_with(data, curve(registry, selection)?.as_ref())
    }
}
impl LineRadial {
    /// Apply a registered Cartesian curve after the common radial point transformation.
    pub fn generate_registered<R: AsRef<[f64]>>(
        &self,
        data: &[R],
        registry: &ExtensionRegistry,
        selection: &ShapeOperation,
    ) -> ChartResult<Path> {
        self.0.generate_materialized_with(
            data,
            &radial::RadialFactory(curve(registry, selection)?.as_ref()),
        )
    }
}
impl AreaRadial {
    /// Apply a registered area curve after transforming each radial boundary point.
    pub fn generate_registered<R: AsRef<[f64]>>(
        &self,
        data: &[R],
        registry: &ExtensionRegistry,
        selection: &ShapeOperation,
    ) -> ChartResult<Path> {
        self.0.generate_materialized_with(
            data,
            &radial::RadialFactory(curve(registry, selection)?.as_ref()),
        )
    }
}
impl Link {
    /// Generate the selected endpoints through a registered curve lifecycle.
    pub fn generate_registered(
        &self,
        data: &LinkDatum,
        registry: &ExtensionRegistry,
        selection: &ShapeOperation,
    ) -> ChartResult<Path> {
        self.generate_with(data, curve(registry, selection)?.as_ref(), |d| {
            self.points(d)
        })
    }
}
impl Symbol {
    /// Draw the configured size with a registered symbol, retaining checked output bounds.
    pub fn generate_registered(
        &self,
        registry: &ExtensionRegistry,
        selection: &ShapeOperation,
    ) -> ChartResult<Path> {
        match registry.resolve_portable_shape(selection, ShapeFamily::Symbol)? {
            ShapeProtocol::Symbol(symbol) => self.generate_with(symbol.as_ref(), self.size),
            _ => unreachable!("registry validates the family"),
        }
    }
}
impl Pie {
    /// Allocate pie angles using a registered datum/value comparator and exact owned metadata.
    pub fn layout_registered(
        &self,
        data: &[serde_json::Value],
        values: &[f64],
        registry: &ExtensionRegistry,
        selection: &ShapeOperation,
    ) -> ChartResult<Vec<PieSlice<serde_json::Value>>> {
        match registry.resolve_portable_shape(selection, ShapeFamily::PieComparator)? {
            ShapeProtocol::PieComparator(compare) => {
                self.layout_materialized_with(data, values, compare.as_ref())
            }
            _ => unreachable!("registry validates the family"),
        }
    }
}
impl Stack {
    /// Use registered ordering and/or offsets, falling back to configured builtins for None.
    pub fn layout_registered<T: Clone>(
        &self,
        data: &[T],
        values: &[Vec<Option<f64>>],
        registry: &ExtensionRegistry,
        order: Option<&ShapeOperation>,
        offset: Option<&ShapeOperation>,
    ) -> ChartResult<Vec<StackSeries<T>>> {
        if data.len() != values.len() || values.iter().any(|r| r.len() != self.keys.len()) {
            return Err(invalid(
                "Stack values must match source sample count and configured key count.",
            ));
        }
        let ordering = order
            .map(|s| registry.resolve_portable_shape(s, ShapeFamily::StackOrder))
            .transpose()?;
        let offsetting = offset
            .map(|s| registry.resolve_portable_shape(s, ShapeFamily::StackOffset))
            .transpose()?;
        let ordering: &dyn StackOrdering = match &ordering {
            Some(ShapeProtocol::StackOrder(p)) => p.as_ref(),
            None => &self.order,
            _ => unreachable!("registry validates the family"),
        };
        let offsetting: &dyn StackOffsetting = match &offsetting {
            Some(ShapeProtocol::StackOffset(p)) => p.as_ref(),
            None => &self.offset,
            _ => unreachable!("registry validates the family"),
        };
        self.generate_with(data, |_, k, s, _| Ok(values[s][k]), ordering, offsetting)
    }
}
