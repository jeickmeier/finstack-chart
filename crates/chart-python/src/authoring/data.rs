use super::*;
use chart_core::{
    data::TimeUnit,
    plot::{ColumnData, ColumnsBuilder, Data},
};
handle!(ColumnHandle, "_Column", ColumnData);
#[pymethods]
impl ColumnHandle {
    #[new]
    #[pyo3(signature=(kind, values, timezone="UTC"))]
    fn new(kind: &str, values: &Bound<'_, PyAny>, timezone: &str) -> PyResult<Self> {
        let column: ColumnData = match kind {
            "float64" => values.extract::<Vec<Option<f64>>>()?.into(),
            "int64" => values.extract::<Vec<Option<i64>>>()?.into(),
            "uint64" => values.extract::<Vec<Option<u64>>>()?.into(),
            "bool" => values.extract::<Vec<Option<bool>>>()?.into(),
            "string" => values.extract::<Vec<Option<String>>>()?.into(),
            "category" => {
                let values = values.extract::<Vec<Option<String>>>()?;
                let validity = values.iter().map(Option::is_some).collect();
                plot::categorical(values.into_iter().map(Option::unwrap_or_default))
                    .validity(validity)
            }
            "s" | "ms" | "us" | "ns" => plot::nullable_timestamps(
                values.extract::<Vec<Option<i64>>>()?,
                match kind {
                    "s" => TimeUnit::Seconds,
                    "ms" => TimeUnit::Milliseconds,
                    "us" => TimeUnit::Microseconds,
                    _ => TimeUnit::Nanoseconds,
                },
                timezone,
            ),
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Unknown column kind.",
                ));
            }
        };
        Ok(Self::wrap(column))
    }
    fn nullable(&self, value: bool) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone().nullable(value)))
    }
    fn validity(&self, values: Vec<bool>) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone().validity(values)))
    }
    fn formatted(&self, values: Vec<Option<String>>) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone().formatted(values)))
    }
    fn unit(&self, value: &str) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone().unit(value)))
    }
    fn label(&self, value: &str) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone().label(value)))
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(ColumnsHandle, "_Columns", ColumnsBuilder);
#[pymethods]
impl ColumnsHandle {
    fn identity(&self, identity: u64) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone().identity(identity)))
    }
    #[new]
    fn new() -> Self {
        Self::wrap(Data::columns())
    }
    fn name(&self, name: &str) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone().name(name)))
    }
    fn column(&self, name: &str, column: &ColumnHandle) -> PyResult<Self> {
        Ok(Self::wrap(
            self.get()?.clone().column(name, column.get()?.clone()),
        ))
    }
    fn keys(&self, keys: Vec<u64>) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.clone().keys(keys)))
    }
    fn limits(&self, input: &str) -> PyResult<Self> {
        Ok(Self::wrap(
            self.get()?
                .clone()
                .limits(portable::decode(input).map_err(failure)?),
        ))
    }
    fn schema_version(&self, version: u64) -> PyResult<Self> {
        Ok(Self::wrap(
            self.get()?
                .clone()
                .schema_version(chart_core::SchemaVersion::new(version)),
        ))
    }
    fn build(&self, py: Python<'_>) -> PyResult<DataHandle> {
        let b = self.get()?.clone();
        py.detach(|| b.build())
            .map(DataHandle::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(DataHandle, "_Data", Data);
#[pymethods]
impl DataHandle {
    fn field(&self, name: &str) -> PyResult<FieldHandle> {
        self.get()?
            .field(name)
            .map(FieldHandle::wrap)
            .map_err(failure)
    }
    fn name(&self) -> PyResult<String> {
        Ok(self.get()?.name().into())
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(FieldHandle, "_Field", plot::FieldHandle);
#[pymethods]
impl FieldHandle {
    fn dispose(&mut self) {
        self.inner.take();
    }
}
pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<ColumnHandle>()?;
    module.add_class::<ColumnsHandle>()?;
    module.add_class::<DataHandle>()?;
    module.add_class::<FieldHandle>()
}
