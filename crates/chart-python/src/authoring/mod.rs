//! Thin owned primary handles; all grammar and runtime behavior remains in shared Rust.
mod color;
mod data;
mod interpolate;
mod output;
mod path;
mod runtime;
mod scale;
mod shape;
mod shape_arc_pie;
mod shape_radial;
mod shape_registry;
mod shape_stack;
mod shape_symbol;
mod time;
use super::failure;
use chart_core::{
    plot::{
        self,
        host::{Component, Draft},
    },
    portable,
};
use data::{DataHandle, FieldHandle};
use pyo3::prelude::*;

fn disposed() -> PyErr {
    failure(chart_core::Diagnostic::error(
        chart_core::DiagnosticCode::DisposedHandle,
        "This primary handle has been disposed.",
        "Create a new handle.",
    ))
}
macro_rules! handle {
    ($name:ident, $python:literal, $ty:ty) => {
        #[pyclass(name = $python, module = "chart_python")]
        pub(super) struct $name {
            inner: Option<$ty>,
        }
        impl $name {
            pub(super) fn get(&self) -> PyResult<&$ty> {
                self.inner.as_ref().ok_or_else(disposed)
            }
            #[allow(
                dead_code,
                reason = "Only mutable owners use this shared macro accessor."
            )]
            pub(super) fn get_mut(&mut self) -> PyResult<&mut $ty> {
                self.inner.as_mut().ok_or_else(disposed)
            }
            pub(super) fn wrap(inner: $ty) -> Self {
                Self { inner: Some(inner) }
            }
        }
    };
}
pub(super) use handle;
handle!(ComponentHandle, "_Component", Component);
#[pymethods]
impl ComponentHandle {
    #[new]
    fn new(name: &str, arguments: &str) -> PyResult<Self> {
        Component::new(name, arguments)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn set(&self, method: &str, arguments: &str) -> PyResult<Self> {
        self.get()?
            .set(method, arguments)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn with_component(&self, method: &str, other: &Self) -> PyResult<Self> {
        self.get()?
            .with(method, other.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn field(&self, method: &str, field: &FieldHandle) -> PyResult<Self> {
        self.get()?
            .field(method, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn numeric_scale_field(
        &self,
        target: &str,
        field: &FieldHandle,
        scale: &str,
    ) -> PyResult<Self> {
        self.get()?
            .numeric_scale_field(
                portable::decode(target).map_err(failure)?,
                *field.get()?,
                portable::decode(scale).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    fn numeric_scale_expression(
        &self,
        target: &str,
        input: &ComponentHandle,
        scale: &str,
    ) -> PyResult<Self> {
        self.get()?
            .numeric_scale_expression(
                portable::decode(target).map_err(failure)?,
                input.get()?,
                portable::decode(scale).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    fn symbol_types_field(
        &self,
        field: &FieldHandle,
        domain: &str,
        palette: &str,
    ) -> PyResult<Self> {
        self.get()?
            .symbol_types_field(
                *field.get()?,
                portable::decode(domain).map_err(failure)?,
                portable::decode(palette).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    fn shape_value_field(&self, target: &str, field: &FieldHandle) -> PyResult<Self> {
        self.get()?
            .shape_value_field(portable::decode(target).map_err(failure)?, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn shape_value_expression(&self, target: &str, input: &ComponentHandle) -> PyResult<Self> {
        self.get()?
            .shape_value_expression(portable::decode(target).map_err(failure)?, input.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn field_parameter(&self, name: &str, field: &FieldHandle) -> PyResult<Self> {
        self.get()?
            .field_parameter(name, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn data(&self, data: &DataHandle) -> PyResult<Self> {
        self.get()?
            .data(data.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    #[staticmethod]
    fn source_expression(field: &FieldHandle) -> PyResult<Self> {
        Ok(Self::wrap(Component::source_expression(*field.get()?)))
    }
    #[staticmethod]
    fn transform(name: &str, stat: &Self) -> PyResult<Self> {
        Component::transform(name, stat.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn theme_layer(&self, layer: &Self, style: &Self) -> PyResult<Self> {
        self.get()?
            .theme_layer(layer.get()?, style.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn candle_volume(&self, candle: &Self, volume: &FieldHandle) -> PyResult<Self> {
        self.get()?
            .candle_volume(candle.get()?, *volume.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn link_axis(&self, source: &Self, target: &Self) -> PyResult<Self> {
        self.get()?
            .link_axis(source.get()?, target.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(DraftHandle, "_Draft", Draft);
#[pymethods]
impl DraftHandle {
    fn with_shape_registry(
        &self,
        registry: &shape_registry::ShapeRegistryHandle,
    ) -> PyResult<Self> {
        self.get()?
            .extensions(registry.get()?.clone())
            .map(Self::wrap)
            .map_err(failure)
    }
    #[cfg(feature = "extension-proof")]
    fn with_example_extensions(&self) -> PyResult<Self> {
        self.get()?
            .extensions(chart_extension_example::registry().map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }
    #[new]
    fn new(data: &DataHandle) -> PyResult<Self> {
        Ok(Self::wrap(Draft::new(data.get()?)))
    }
    fn with_component(&self, slot: &str, value: &ComponentHandle) -> PyResult<Self> {
        self.get()?
            .with(slot, value.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn dataset(&self, data: &DataHandle) -> PyResult<Self> {
        self.get()?
            .dataset(data.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn layer(&self, name: &str, value: &ComponentHandle) -> PyResult<Self> {
        self.get()?
            .layer(name, value.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn set(&self, method: &str, arguments: &str) -> PyResult<Self> {
        self.get()?
            .set(method, arguments)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn build(&self, py: Python<'_>) -> PyResult<PlotHandle> {
        let draft = self.get()?;
        py.detach(|| draft.build())
            .map(PlotHandle::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(PlotHandle, "_Plot", plot::Plot);
#[pymethods]
impl PlotHandle {
    #[staticmethod]
    fn from_json_with_registry(
        input: &str,
        registry: &shape_registry::ShapeRegistryHandle,
    ) -> PyResult<Self> {
        plot::Plot::from_json_with_extensions(input, registry.get()?.clone())
            .map(Self::wrap)
            .map_err(failure)
    }
    fn edit(&self) -> PyResult<DraftHandle> {
        Ok(DraftHandle::wrap(Draft::edit(self.get()?)))
    }
    fn to_json(&self, py: Python<'_>) -> PyResult<String> {
        let plot = self.get()?;
        py.detach(|| plot.to_json()).map_err(failure)
    }
    #[cfg(feature = "extension-proof")]
    #[staticmethod]
    fn from_json_with_example_extensions(input: &str) -> PyResult<Self> {
        plot::Plot::from_json_with_extensions(
            input,
            chart_extension_example::registry().map_err(failure)?,
        )
        .map(Self::wrap)
        .map_err(failure)
    }
    #[staticmethod]
    fn from_json(py: Python<'_>, input: String) -> PyResult<Self> {
        py.detach(|| plot::Plot::from_json(&input))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ComponentHandle>()?;
    module.add_class::<DraftHandle>()?;
    module.add_class::<PlotHandle>()?;
    module.add_class::<data::ColumnHandle>()?;
    module.add_class::<data::ColumnsHandle>()?;
    module.add_class::<data::DataHandle>()?;
    module.add_class::<data::FieldHandle>()?;
    module.add_class::<path::PathHandle>()?;
    module.add_class::<shape_registry::ShapeRegistryHandle>()?;
    module.add_class::<shape::ShapeLineHandle>()?;
    module.add_class::<shape::ShapeAreaHandle>()?;
    module.add_class::<shape_radial::ShapeLineRadialHandle>()?;
    module.add_class::<shape_radial::ShapeAreaRadialHandle>()?;
    module.add_class::<shape_radial::ShapeLinkHandle>()?;
    module.add_class::<shape_radial::ShapeLinkRadialHandle>()?;
    module.add_class::<shape_arc_pie::ShapeArcHandle>()?;
    module.add_class::<shape_arc_pie::ShapePieHandle>()?;
    module.add_class::<shape_symbol::ShapeSymbolHandle>()?;
    module.add_class::<shape_stack::ShapeStackHandle>()?;
    module.add_class::<color::ColorHandle>()?;
    module.add_class::<interpolate::InterpolatorHandle>()?;
    module.add_class::<time::TimeScaleHandle>()?;
    module.add_class::<scale::ScaleHandle>()?;
    module.add_class::<runtime::EditorHandle>()?;
    module.add_class::<runtime::RuntimeHandle>()?;
    module.add_class::<runtime::UpdatesHandle>()?;
    module.add_class::<runtime::TransactionHandle>()?;
    module.add_class::<output::OptionsHandle>()?;
    module.add_class::<output::OutputHandle>()?;
    module.add_class::<output::RequestHandle>()?;
    module.add_class::<output::FrameHandle>()?;
    module.add_class::<output::QueueHandle>()?;
    module.add_class::<output::JobHandle>()
}
