use super::{Data, FieldHandle, error};
use crate::data::{ColumnValues, FieldKind};
use crate::grammar::{Numeric, SourceAes};
use crate::{ChartResult, DiagnosticCode, FieldId};

/// Source-stage field reference or explicit calculation-space literal.
/// Generated stat fields intentionally do not implement conversion to this type.
#[derive(Clone, Debug)]
pub enum Mapping {
    /// Resolve a name in the selected layer dataset.
    Field(String),
    /// Require the same owning dataset as the supplied handle.
    Handle(FieldHandle),
    /// Explicit coordinate/baseline constant, not a mapped style.
    Literal(f64),
    /// Exact timestamp field with a deliberate integer origin.
    Timestamp {
        /// Authored field name.
        field: String,
        /// Exact source-unit origin.
        origin: i64,
    },
}
impl From<&str> for Mapping {
    fn from(value: &str) -> Self {
        Self::Field(value.into())
    }
}
impl From<String> for Mapping {
    fn from(value: String) -> Self {
        Self::Field(value)
    }
}
impl From<FieldHandle> for Mapping {
    fn from(value: FieldHandle) -> Self {
        Self::Handle(value)
    }
}
impl From<f64> for Mapping {
    fn from(value: f64) -> Self {
        Self::Literal(value)
    }
}
impl Mapping {
    pub(super) fn field(&self, data: &Data) -> ChartResult<FieldId> {
        match self {
            Self::Field(name) | Self::Timestamp { field: name, .. } => Ok(data.field(name)?.id()),
            Self::Handle(handle) if handle.dataset == data.id => Ok(handle.field),
            Self::Handle(_) => Err(error(
                DiagnosticCode::SchemaConflict,
                format!(
                    "A field handle belongs to another dataset than '{}'.",
                    data.name
                ),
            )),
            Self::Literal(_) => Err(error(
                DiagnosticCode::SchemaConflict,
                "This mapping requires a source field, not a literal.",
            )),
        }
    }
    pub(super) fn resolve(&self, data: &Data) -> ChartResult<Numeric> {
        if let Self::Literal(value) = self {
            return Ok(Numeric::Literal(*value));
        }
        let id = self.field(data)?;
        let (index, field) = data.batch.schema().field(id).ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                "Field handle is absent from its schema.",
            )
        })?;
        Ok(match (&field.kind, self) {
            (FieldKind::Timestamp(_), Self::Timestamp { origin, .. }) => Numeric::Timestamp {
                field: id,
                origin: *origin,
            },
            (FieldKind::Timestamp(kind), _) => {
                let column = &data.batch.columns()[index];
                let ColumnValues::Timestamp(values) = column.values() else {
                    unreachable!("validated timestamp")
                };
                let origin = data
                    .timestamp_origins
                    .iter()
                    .find(|(known, _)| known == kind)
                    .map(|(_, origin)| *origin)
                    .unwrap_or_else(|| {
                        values
                            .iter()
                            .zip(column.validity())
                            .find(|(_, valid)| **valid)
                            .map_or(0, |(v, _)| *v)
                    });
                Numeric::Timestamp { field: id, origin }
            }
            (_, Self::Timestamp { .. }) => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    format!("Field '{}' is not a timestamp.", field.name),
                ));
            }
            (FieldKind::Categorical, _) => Numeric::Category(id),
            _ => Numeric::Field(id),
        })
    }
}
/// Aesthetic mappings inherited by layers, resolved against each layer's selected data.
#[derive(Clone, Debug, Default)]
pub struct AesBuilder {
    pub(super) x: Option<Mapping>,
    pub(super) y: Option<Mapping>,
    pub(super) x2: Option<Mapping>,
    pub(super) y2: Option<Mapping>,
    pub(super) low: Option<Mapping>,
    pub(super) high: Option<Mapping>,
    pub(super) size: Option<Mapping>,
    pub(super) group: Option<Mapping>,
    pub(super) all_groups: bool,
    pub(super) color: Option<Mapping>,
    pub(super) color_scale: Option<String>,
}
/// Begin source-stage mappings; constants belong on layer/style builders.
///
/// ```compile_fail
/// use chart_core::{plot::aes, grammar::StatField};
/// let invalid = aes().x(StatField::Count);
/// ```
pub fn aes() -> AesBuilder {
    AesBuilder::default()
}
macro_rules! mapping_methods {
    ($($name:ident => $doc:literal),* $(,)?) => {$ (
        #[doc = $doc]
        pub fn $name(mut self, value: impl Into<Mapping>) -> Self { self.$name = Some(value.into()); self }
    )*};
}
impl AesBuilder {
    /// Explicitly use one group while retaining every other inherited aesthetic.
    pub fn group_all(mut self) -> Self {
        self.group = None;
        self.all_groups = true;
        self
    }
    /// Partition observations using an exact source field, independently of appearance.
    pub fn group(mut self, value: impl Into<Mapping>) -> Self {
        self.group = Some(value.into());
        self.all_groups = false;
        self
    }

    /// Select an explicitly authored color scale independently of the mapped field name.
    pub fn color_scale(mut self, name: impl Into<String>) -> Self {
        self.color_scale = Some(name.into());
        self
    }
    mapping_methods!(x => "Map the horizontal coordinate.", y => "Map the vertical coordinate.", x2 => "Map a second horizontal endpoint.", y2 => "Map a second vertical endpoint/baseline or OHLC close.", low => "Map the supplied OHLC low.", high => "Map the supplied OHLC high.", size => "Map baseline point-radius/stroke-width units.", color => "Map source values to a named color scale; does not implicitly group lines.");
    pub(super) fn merged(&self, base: &Self, inherit: bool) -> Self {
        if !inherit {
            return self.clone();
        }
        let mut result = self.clone();
        macro_rules! inherit { ($($name:ident),*) => { $(if result.$name.is_none() { result.$name = base.$name.clone(); })* }; }
        inherit!(x, y, x2, y2, low, high, size, color, color_scale);
        if result.group.is_none() && !result.all_groups {
            result.group = base.group.clone();
            result.all_groups = base.all_groups;
        }
        result
    }
    pub(super) fn resolve(&self, data: &Data) -> ChartResult<SourceAes> {
        Ok(SourceAes {
            x: self.x.as_ref().map(|v| v.resolve(data)).transpose()?,
            y: self.y.as_ref().map(|v| v.resolve(data)).transpose()?,
            x2: self.x2.as_ref().map(|v| v.resolve(data)).transpose()?,
            y2: self.y2.as_ref().map(|v| v.resolve(data)).transpose()?,
            low: self.low.as_ref().map(|v| v.resolve(data)).transpose()?,
            high: self.high.as_ref().map(|v| v.resolve(data)).transpose()?,
            size: self.size.as_ref().map(|v| v.resolve(data)).transpose()?,
            group: self.group.as_ref().map(|v| v.field(data)).transpose()?,
        })
    }
}
