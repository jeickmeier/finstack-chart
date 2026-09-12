use super::{Data, FieldHandle, error};
use crate::data::{ColumnValues, FieldKind};
use crate::grammar::{Numeric, SourceAes};
use crate::{ChartResult, DiagnosticCode, FieldId};

/// Source-stage field reference or explicit calculation-space literal.
/// Generated stat fields intentionally do not implement conversion to this type.
#[derive(Clone, Debug)]
pub enum Mapping {
    /// Source-stage expression resolved against the selected layer dataset.
    Expression(crate::grammar::Expression<Mapping>),
    /// Explicit resolved source scale stage, preserved when editing portable mappings.
    Scaled {
        /// Owned underlying source mapping.
        input: Box<Mapping>,
        /// Checked scale policy.
        scale: crate::grammar::ScaleProjection,
    },
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
    pub(super) fn grouping(&self, data: &Data) -> ChartResult<crate::grammar::Grouping> {
        if let Self::Literal(value) = self {
            if !value.is_finite() {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "A constant grouping value must be finite.",
                ));
            }
            Ok(crate::grammar::Grouping::All)
        } else {
            self.field(data).map(crate::grammar::Grouping::Field)
        }
    }
    pub(super) fn field(&self, data: &Data) -> ChartResult<FieldId> {
        match self {
            Self::Expression(_) => Err(error(
                DiagnosticCode::SchemaConflict,
                "This aesthetic requires a source field rather than a numeric expression.",
            )),
            Self::Scaled { input, .. } => input.field(data),
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
        if let Self::Expression(expr) = self {
            return Ok(Numeric::Expression(expr.try_map_reads(|read| {
                if matches!(read,Self::Expression(_) | Self::Scaled {..}) {
                    return Err(error(DiagnosticCode::SchemaConflict,"Expression reads require source fields; compose expressions with typed operators."));
                }
                match read.resolve(data)? {
                    Numeric::Field(f) => Ok(crate::grammar::SourceRead::Field(f)),
                    Numeric::Timestamp {field,origin} => Ok(crate::grammar::SourceRead::Timestamp {field,origin}),
                    _ => Err(error(DiagnosticCode::SchemaConflict,"Source numeric expressions require numeric fields or relative timestamps.")),
                }
            })?));
        }
        if let Self::Scaled { input, scale } = self {
            return Ok(Numeric::Scaled {
                input: Box::new(input.resolve(data)?),
                scale: Box::new(scale.clone()),
            });
        }
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
    pub(super) grouping: Option<crate::grammar::Grouping>,
    pub(super) x: Option<Mapping>,
    pub(super) y: Option<Mapping>,
    pub(super) x2: Option<Mapping>,
    pub(super) y2: Option<Mapping>,
    pub(super) low: Option<Mapping>,
    pub(super) high: Option<Mapping>,
    pub(super) size: Option<Mapping>,
    pub(super) shape: Option<Mapping>,
    pub(super) alpha: Option<Mapping>,
    pub(super) linewidth: Option<Mapping>,
    pub(super) linetype: Option<Mapping>,
    pub(super) group: Option<Mapping>,
    pub(super) all_groups: bool,
    pub(super) color: Option<Mapping>,
    pub(super) color_scale: Option<String>,
    pub(super) fill: Option<Mapping>,
    pub(super) fill_scale: Option<String>,
    pub(super) stroke: Option<Mapping>,
    pub(super) stroke_scale: Option<String>,
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
    pub(super) fn resolved_grouping(
        &self,
        data: &Data,
    ) -> ChartResult<Option<crate::grammar::Grouping>> {
        if self.all_groups {
            Ok(Some(crate::grammar::Grouping::All))
        } else if let Some(group) = &self.grouping {
            Ok(Some(group.clone()))
        } else {
            self.group.as_ref().map(|g| g.grouping(data)).transpose()
        }
    }
    /// Explicitly use one group while retaining every other inherited aesthetic.
    pub fn group_all(mut self) -> Self {
        self.grouping = None;
        self.group = None;
        self.all_groups = true;
        self
    }
    /// Partition observations using an exact source field, independently of appearance.
    pub fn group(mut self, value: impl Into<Mapping>) -> Self {
        self.grouping = None;
        self.group = Some(value.into());
        self.all_groups = false;
        self
    }

    /// Select an explicitly authored color scale independently of the mapped field name.
    pub fn color_scale(mut self, name: impl Into<String>) -> Self {
        self.color_scale = Some(name.into());
        self
    }
    /// Select the fill scale independently of the source field and outline scale.
    pub fn fill_scale(mut self, name: impl Into<String>) -> Self {
        self.fill_scale = Some(name.into());
        self
    }
    /// Select the outline scale independently of the source field and fill scale.
    pub fn stroke_scale(mut self, name: impl Into<String>) -> Self {
        self.stroke_scale = Some(name.into());
        self
    }
    mapping_methods!(alpha => "Map reference alpha coverage under the ggplot profile.", linewidth => "Map reference line width under the ggplot profile.");
    mapping_methods!(fill => "Map interior paint independently.", stroke => "Map outline and line paint independently.", shape => "Map discrete reference point shapes under the ggplot profile.", linetype => "Map discrete reference line patterns under the ggplot profile.");
    mapping_methods!(x => "Map the horizontal coordinate.", y => "Map the vertical coordinate.", x2 => "Map a second horizontal endpoint.", y2 => "Map a second vertical endpoint/baseline or OHLC close.", low => "Map the supplied OHLC low.", high => "Map the supplied OHLC high.", size => "Map baseline point-radius/stroke-width units.", color => "Map source values to a named color scale; does not implicitly group lines.");
    pub(super) fn merged(&self, base: &Self, inherit: bool) -> Self {
        if !inherit {
            return self.clone();
        }
        let mut result = self.clone();
        macro_rules! inherit { ($($name:ident),*) => { $(if result.$name.is_none() { result.$name = base.$name.clone(); })* }; }
        inherit!(
            x,
            y,
            x2,
            y2,
            low,
            high,
            size,
            shape,
            alpha,
            linewidth,
            linetype,
            color,
            color_scale,
            fill,
            fill_scale,
            stroke,
            stroke_scale
        );
        if result.group.is_none() && !result.all_groups && result.grouping.is_none() {
            result.grouping = base.grouping.clone();
            result.group = base.group.clone();
            result.all_groups = base.all_groups;
        }
        result
    }
    pub(super) fn resolve(&self, data: &Data) -> ChartResult<SourceAes> {
        let explicit = self.group.as_ref().map(|g| g.grouping(data)).transpose()?;
        Ok(SourceAes {
            grouping: self.grouping.clone().or_else(|| {
                matches!(explicit, Some(crate::grammar::Grouping::All))
                    .then_some(crate::grammar::Grouping::All)
            }),
            x: self.x.as_ref().map(|v| v.resolve(data)).transpose()?,
            y: self.y.as_ref().map(|v| v.resolve(data)).transpose()?,
            x2: self.x2.as_ref().map(|v| v.resolve(data)).transpose()?,
            y2: self.y2.as_ref().map(|v| v.resolve(data)).transpose()?,
            low: self.low.as_ref().map(|v| v.resolve(data)).transpose()?,
            high: self.high.as_ref().map(|v| v.resolve(data)).transpose()?,
            size: self.size.as_ref().map(|v| v.resolve(data)).transpose()?,
            group: if let Some(crate::grammar::Grouping::Field(field)) = explicit {
                Some(field)
            } else {
                None
            },
        })
    }
}

impl From<crate::grammar::Expression<Mapping>> for Mapping {
    fn from(value: crate::grammar::Expression<Mapping>) -> Self {
        Self::Expression(value)
    }
}
/// Begin a typed source expression, resolved within each layer's selected dataset.
pub fn source_expr(field: impl Into<Mapping>) -> crate::grammar::Expression<Mapping> {
    crate::grammar::Expression::read(field.into())
}

/// Typed nonpositional mappings evaluated after scale mapping and positions.
#[derive(Clone, Debug, Default)]
pub struct AfterScaleAesBuilder {
    pub(super) mappings: std::collections::BTreeMap<
        crate::grammar::AfterScaleAesthetic,
        crate::grammar::Expression<crate::grammar::AfterScaleRead>,
    >,
}
/// Begin post-scale size/color modifiers.
pub fn scale_aes() -> AfterScaleAesBuilder {
    AfterScaleAesBuilder::default()
}
/// Read an aesthetic's resolved scale value, before any post-scale modifiers.
pub fn after_scale_expr(
    aesthetic: crate::grammar::AfterScaleAesthetic,
) -> crate::grammar::Expression<crate::grammar::AfterScaleRead> {
    crate::grammar::Expression::read(crate::grammar::AfterScaleRead::Aesthetic(aesthetic))
}
/// Read a geometry-theme token in the same typed expression kernel.
pub fn from_theme(
    token: crate::grammar::ThemeRead,
) -> crate::grammar::Expression<crate::grammar::AfterScaleRead> {
    crate::grammar::Expression::read(crate::grammar::AfterScaleRead::Theme(token))
}
impl AfterScaleAesBuilder {
    /// Derive an independent interior paint from the pre-modifier scale snapshot.
    pub fn fill(
        mut self,
        expr: crate::grammar::Expression<crate::grammar::AfterScaleRead>,
    ) -> Self {
        self.mappings
            .insert(crate::grammar::AfterScaleAesthetic::Fill, expr);
        self
    }
    /// Derive an independent outline paint from the pre-modifier scale snapshot.
    pub fn stroke(
        mut self,
        expr: crate::grammar::Expression<crate::grammar::AfterScaleRead>,
    ) -> Self {
        self.mappings
            .insert(crate::grammar::AfterScaleAesthetic::Stroke, expr);
        self
    }
    /// Derive replacement alpha from the pre-modifier scale snapshot.
    pub fn alpha(
        mut self,
        expr: crate::grammar::Expression<crate::grammar::AfterScaleRead>,
    ) -> Self {
        self.mappings
            .insert(crate::grammar::AfterScaleAesthetic::Alpha, expr);
        self
    }
    /// Derive independent linewidth from the pre-modifier scale snapshot.
    pub fn linewidth(
        mut self,
        expr: crate::grammar::Expression<crate::grammar::AfterScaleRead>,
    ) -> Self {
        self.mappings
            .insert(crate::grammar::AfterScaleAesthetic::LineWidth, expr);
        self
    }

    /// Set point/rule size from a numeric expression.
    pub fn size(
        mut self,
        expr: crate::grammar::Expression<crate::grammar::AfterScaleRead>,
    ) -> Self {
        self.mappings
            .insert(crate::grammar::AfterScaleAesthetic::Size, expr);
        self
    }
    /// Set geometry color from a color expression.
    pub fn color(
        mut self,
        expr: crate::grammar::Expression<crate::grammar::AfterScaleRead>,
    ) -> Self {
        self.mappings
            .insert(crate::grammar::AfterScaleAesthetic::Color, expr);
        self
    }
}
