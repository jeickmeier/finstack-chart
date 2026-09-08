use super::*;
use chart_core::{
    data::TimeUnit,
    plot::{ColumnData, ColumnsBuilder, Data},
};
handle!(_Column, _Column, ColumnData);
#[wasm_bindgen]
impl _Column {
    pub fn float64(values: Vec<f64>) -> Self {
        Self::wrap(values.into())
    }
    pub fn int64(values: Vec<i64>) -> Self {
        Self::wrap(values.into())
    }
    pub fn uint64(values: Vec<u64>) -> Self {
        Self::wrap(values.into())
    }
    pub fn boolean(values: Vec<u8>) -> Result<Self, JsError> {
        if values.iter().any(|v| *v > 1) {
            return Err(JsError::new("Boolean payload must contain zero or one."));
        }
        Ok(Self::wrap(
            values
                .into_iter()
                .map(|v| v != 0)
                .collect::<Vec<_>>()
                .into(),
        ))
    }
    pub fn strings(values: Vec<String>) -> Self {
        Self::wrap(values.into())
    }
    pub fn categorical(values: Vec<String>) -> Self {
        Self::wrap(plot::categorical(values))
    }
    pub fn timestamp(values: Vec<i64>, unit: &str, timezone: &str) -> Result<Self, JsError> {
        let unit = match unit {
            "s" => TimeUnit::Seconds,
            "ms" => TimeUnit::Milliseconds,
            "us" => TimeUnit::Microseconds,
            "ns" => TimeUnit::Nanoseconds,
            _ => return Err(JsError::new("Timestamp unit must be s, ms, us or ns.")),
        };
        Ok(Self::wrap(plot::timestamps(values, unit, timezone)))
    }
    pub fn nullable(&self, value: bool) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone().nullable(value)))
    }
    pub fn validity(&self, values: Vec<u8>) -> Result<Self, JsError> {
        if values.iter().any(|v| *v > 1) {
            return Err(JsError::new("Validity must contain zero or one."));
        }
        Ok(Self::wrap(
            self.get()?
                .clone()
                .validity(values.into_iter().map(|v| v != 0).collect()),
        ))
    }
    pub fn formatted(&self, values: Vec<String>, validity: Vec<u8>) -> Result<Self, JsError> {
        if values.len() != validity.len() || validity.iter().any(|v| *v > 1) {
            return Err(JsError::new("Formatted values and validity must agree."));
        }
        Ok(Self::wrap(
            self.get()?.clone().formatted(
                values
                    .into_iter()
                    .zip(validity)
                    .map(|(s, v)| (v != 0).then_some(s))
                    .collect(),
            ),
        ))
    }
    pub fn unit(&self, value: &str) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone().unit(value)))
    }
    pub fn label(&self, value: &str) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone().label(value)))
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_Columns, _Columns, ColumnsBuilder);
#[wasm_bindgen]
impl _Columns {
    pub fn identity(&self, identity: u64) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone().identity(identity)))
    }
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::wrap(Data::columns())
    }
    pub fn name(&self, value: &str) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone().name(value)))
    }
    pub fn column(&self, name: &str, column: &_Column) -> Result<Self, JsError> {
        Ok(Self::wrap(
            self.get()?.clone().column(name, column.get()?.clone()),
        ))
    }
    pub fn keys(&self, values: Vec<u64>) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone().keys(values)))
    }
    pub fn limits(&self, input: &str) -> Result<Self, JsError> {
        Ok(Self::wrap(
            self.get()?
                .clone()
                .limits(portable::decode(input).map_err(failure)?),
        ))
    }
    pub fn schema_version(&self, version: u64) -> Result<Self, JsError> {
        Ok(Self::wrap(
            self.get()?
                .clone()
                .schema_version(chart_core::SchemaVersion::new(version)),
        ))
    }
    pub fn build(&self) -> Result<_Data, JsError> {
        self.get()?
            .clone()
            .build()
            .map(_Data::wrap)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_Data, _Data, Data);
#[wasm_bindgen]
impl _Data {
    pub fn field(&self, name: &str) -> Result<_Field, JsError> {
        self.get()?.field(name).map(_Field::wrap).map_err(failure)
    }
    pub fn name(&self) -> Result<String, JsError> {
        Ok(self.get()?.name().into())
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_Field, _Field, plot::FieldHandle);
#[wasm_bindgen]
impl _Field {
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
