use super::*;
use chart_core::{
    data::TimeUnit,
    plot::{ColumnData, ColumnsBuilder, Data},
};
/// Decode a typed `Uint8Array` of zero/one flags; any other byte is rejected.
fn flags(values: Vec<u8>, what: &str) -> Result<Vec<bool>, JsError> {
    if values.iter().any(|v| *v > 1) {
        return Err(JsError::new(&format!("{what} must contain zero or one.")));
    }
    Ok(values.into_iter().map(|v| v != 0).collect())
}
handle!(_Column, ColumnData);
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
        Ok(Self::wrap(flags(values, "Boolean payload")?.into()))
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
        let validity = flags(values, "Validity")?;
        Ok(Self::wrap(self.get()?.clone().validity(validity)))
    }
    pub fn formatted(&self, values: Vec<String>, validity: Vec<u8>) -> Result<Self, JsError> {
        if values.len() != validity.len() {
            return Err(JsError::new("Formatted values and validity must agree."));
        }
        let validity = flags(validity, "Validity")?;
        Ok(Self::wrap(
            self.get()?.clone().formatted(
                values
                    .into_iter()
                    .zip(validity)
                    .map(|(s, valid)| valid.then_some(s))
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
handle!(_Columns, ColumnsBuilder);
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
handle!(_Data, Data);
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
handle!(_Field, plot::FieldHandle);
#[wasm_bindgen]
impl _Field {
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
