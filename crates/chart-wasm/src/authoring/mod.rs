//! Owned primary authoring handles; source columns cross typed WASM vectors.
mod color;
mod data;
mod hierarchy;
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
mod vector;
use super::failure;
use chart_core::{
    plot::{
        self,
        host::{Component, Draft},
    },
    portable,
};
use data::{_Data, _Field};
use wasm_bindgen::prelude::*;
fn disposed() -> JsError {
    failure(chart_core::Diagnostic::error(
        chart_core::DiagnosticCode::DisposedHandle,
        "This primary handle has been disposed.",
        "Create a new handle.",
    ))
}
macro_rules! handle {
    ($name:ident, $ty:ty) => {
        #[wasm_bindgen]
        pub struct $name {
            inner: Option<$ty>,
        }
        impl $name {
            pub(super) fn get(&self) -> Result<&$ty, JsError> {
                self.inner.as_ref().ok_or_else(disposed)
            }
            #[allow(
                dead_code,
                reason = "Only mutable owners use this shared macro accessor."
            )]
            pub(super) fn get_mut(&mut self) -> Result<&mut $ty, JsError> {
                self.inner.as_mut().ok_or_else(disposed)
            }
            pub(super) fn wrap(inner: $ty) -> Self {
                Self { inner: Some(inner) }
            }
        }
    };
}
pub(super) use handle;
handle!(_Component, Component);
#[wasm_bindgen]
impl _Component {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, args: &str) -> Result<Self, JsError> {
        Component::new(name, args).map(Self::wrap).map_err(failure)
    }
    pub fn set(&self, name: &str, args: &str) -> Result<Self, JsError> {
        self.get()?.set(name, args).map(Self::wrap).map_err(failure)
    }
    pub fn with_component(&self, name: &str, other: &Self) -> Result<Self, JsError> {
        self.get()?
            .with(name, other.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn field(&self, name: &str, field: &_Field) -> Result<Self, JsError> {
        self.get()?
            .field(name, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn value_scale_field(
        &self,
        target: &str,
        field: &_Field,
        scale: &str,
    ) -> Result<Self, JsError> {
        self.get()?
            .value_scale_field(
                portable::decode(target).map_err(failure)?,
                *field.get()?,
                portable::decode(scale).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn numeric_scale_field(
        &self,
        target: &str,
        field: &_Field,
        scale: &str,
    ) -> Result<Self, JsError> {
        self.get()?
            .numeric_scale_field(
                portable::decode(target).map_err(failure)?,
                *field.get()?,
                portable::decode(scale).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn value_scale_expression(
        &self,
        target: &str,
        input: &_Component,
        scale: &str,
    ) -> Result<Self, JsError> {
        self.get()?
            .value_scale_expression(
                portable::decode(target).map_err(failure)?,
                input.get()?,
                portable::decode(scale).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn numeric_scale_expression(
        &self,
        target: &str,
        input: &_Component,
        scale: &str,
    ) -> Result<Self, JsError> {
        self.get()?
            .numeric_scale_expression(
                portable::decode(target).map_err(failure)?,
                input.get()?,
                portable::decode(scale).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn symbol_types_field(
        &self,
        field: &_Field,
        domain: &str,
        palette: &str,
    ) -> Result<Self, JsError> {
        self.get()?
            .symbol_types_field(
                *field.get()?,
                portable::decode(domain).map_err(failure)?,
                portable::decode(palette).map_err(failure)?,
            )
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn shape_value_field(&self, target: &str, field: &_Field) -> Result<Self, JsError> {
        self.get()?
            .shape_value_field(portable::decode(target).map_err(failure)?, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn shape_value_expression(
        &self,
        target: &str,
        input: &_Component,
    ) -> Result<Self, JsError> {
        self.get()?
            .shape_value_expression(portable::decode(target).map_err(failure)?, input.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn recipe_value_field(&self, target: &str, field: &_Field) -> Result<Self, JsError> {
        self.get()?
            .recipe_value_field(portable::decode(target).map_err(failure)?, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn recipe_value_expression(
        &self,
        target: &str,
        input: &_Component,
    ) -> Result<Self, JsError> {
        self.get()?
            .recipe_value_expression(portable::decode(target).map_err(failure)?, input.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn field_parameter(&self, name: &str, field: &_Field) -> Result<Self, JsError> {
        self.get()?
            .field_parameter(name, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn data(&self, data: &_Data) -> Result<Self, JsError> {
        self.get()?
            .data(data.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn source_expression(field: &_Field) -> Result<Self, JsError> {
        Ok(Self::wrap(Component::source_expression(*field.get()?)))
    }
    pub fn transform(name: &str, stat: &Self) -> Result<Self, JsError> {
        Component::transform(name, stat.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn theme_layer(&self, layer: &Self, style: &Self) -> Result<Self, JsError> {
        self.get()?
            .theme_layer(layer.get()?, style.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn candle_volume(&self, layer: &Self, field: &_Field) -> Result<Self, JsError> {
        self.get()?
            .candle_volume(layer.get()?, *field.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn link_axis(&self, source: &Self, target: &Self) -> Result<Self, JsError> {
        self.get()?
            .link_axis(source.get()?, target.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_Draft, Draft);
#[wasm_bindgen]
impl _Draft {
    pub fn autolayer(&self, selection: &str) -> Result<Self, JsError> {
        self.get()?
            .autolayer(portable::decode(selection).map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }

    pub fn with_shape_registry(
        &self,
        registry: &shape_registry::_ShapeRegistry,
    ) -> Result<Self, JsError> {
        self.get()?
            .extensions(registry.get()?.clone())
            .map(Self::wrap)
            .map_err(failure)
    }
    #[cfg(feature = "extension-proof")]
    pub fn with_example_extensions(&self) -> Result<Self, JsError> {
        self.get()?
            .extensions(chart_extension_example::registry().map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }
    #[wasm_bindgen(constructor)]
    pub fn new(data: &_Data) -> Result<Self, JsError> {
        Ok(Self::wrap(Draft::new(data.get()?)))
    }
    pub fn with_component(&self, slot: &str, value: &_Component) -> Result<Self, JsError> {
        self.get()?
            .with(slot, value.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn dataset(&self, data: &_Data) -> Result<Self, JsError> {
        self.get()?
            .dataset(data.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn layer(&self, name: &str, value: &_Component) -> Result<Self, JsError> {
        self.get()?
            .layer(name, value.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn set(&self, name: &str, args: &str) -> Result<Self, JsError> {
        self.get()?.set(name, args).map(Self::wrap).map_err(failure)
    }
    pub fn build(&self) -> Result<_Plot, JsError> {
        self.get()?.build().map(_Plot::wrap).map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_Plot, plot::Plot);
#[wasm_bindgen]
impl _Plot {
    pub fn from_json_with_registry(
        input: &str,
        registry: &shape_registry::_ShapeRegistry,
    ) -> Result<Self, JsError> {
        plot::Plot::from_json_with_extensions(input, registry.get()?.clone())
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn edit(&self) -> Result<_Draft, JsError> {
        Ok(_Draft::wrap(Draft::edit(self.get()?)))
    }
    pub fn to_json(&self) -> Result<String, JsError> {
        self.get()?.to_json().map_err(failure)
    }
    #[cfg(feature = "extension-proof")]
    pub fn from_json_with_example_extensions(input: &str) -> Result<Self, JsError> {
        plot::Plot::from_json_with_extensions(
            input,
            chart_extension_example::registry().map_err(failure)?,
        )
        .map(Self::wrap)
        .map_err(failure)
    }
    pub fn from_json(input: &str) -> Result<Self, JsError> {
        plot::Plot::from_json(input)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
