//! Non-numeric style channels use the same typed mapping engine as numeric aesthetics.
use super::{compiler::EncodedRow, *};
use crate::{
    ChartResult, DiagnosticCode,
    data::DatasetSnapshot,
    interpolate::Value,
    scales::{MappedScale, ScaleKey},
};
use std::collections::BTreeMap;

/// R's filled-circle radius from ggplot size and stroke, in matching units.
pub(crate) fn reference_point_radius(size: f64, stroke: f64) -> f64 {
    (size * 0.37640625 + stroke * 0.25).max(0.)
}

/// Units for explicitly authored aesthetic dimensions, independent of positional coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AestheticUnits {
    /// Values already use the destination's declared scene units.
    Destination,
    /// Physical millimeters: 96 logical pixels or 72 publication points per inch.
    Millimeters,
    /// Physical publication points, 72 per inch.
    Points,
}
impl AestheticUnits {
    /// Exact dimension conversion; areas use the square of this factor.
    pub fn factor(self, destination: crate::services::Units) -> f64 {
        let per_inch = match destination {
            crate::services::Units::LogicalPixels => 96.,
            crate::services::Units::Points => 72.,
        };
        match self {
            Self::Destination => 1.,
            Self::Millimeters => per_inch / 25.4,
            Self::Points => per_inch / 72.,
        }
    }
}
// R graphics linewidth uses TeX points per millimeter, then the device's
// 1/96-inch line unit. Preserve the PDF device's physical zero-width hairline.
pub(crate) fn reference_linewidth(width: f64, units: crate::services::Units) -> f64 {
    let pixels = if width == 0. {
        0.01 * 96. / 72.
    } else {
        width * 72.27 / 25.4
    };
    pixels
        * if units == crate::services::Units::Points {
            72. / 96.
        } else {
            1.
        }
}
/// R/ggplot line types with widths expressed in multiples of the resolved line width.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LineType {
    /// No stroke.
    Blank,
    /// Uninterrupted stroke.
    Solid,
    /// Four widths on, four off.
    Dashed,
    /// One width on, three off.
    Dotted,
    /// Alternating dots and dashes.
    DotDash,
    /// Seven widths on, three off.
    LongDash,
    /// Alternating two-width and six-width gaps.
    TwoDash,
    /// Two, four, six or eight nonzero hexadecimal pattern digits.
    Custom(u32),
}
impl LineType {
    /// Parse reference names or nonzero hexadecimal dash strings.
    pub fn parse(value: &str) -> ChartResult<Self> {
        let result = match value {
            "blank" => Self::Blank,
            "solid" => Self::Solid,
            "dashed" => Self::Dashed,
            "dotted" => Self::Dotted,
            "dotdash" => Self::DotDash,
            "longdash" => Self::LongDash,
            "twodash" => Self::TwoDash,
            value => {
                if !matches!(value.len(), 2 | 4 | 6 | 8) || value.contains('0') {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Line type requires a reference name or two, four, six or eight nonzero hex digits.",
                    ));
                }
                Self::Custom(u32::from_str_radix(value, 16).map_err(|_| {
                    error(DiagnosticCode::Validation, "Invalid hexadecimal line type.")
                })?)
            }
        };
        result.pattern(1.)?;
        Ok(result)
    }
    /// Resolve the pattern using the same physical stroke width as the mark.
    pub fn pattern(self, width: f64) -> ChartResult<Vec<f64>> {
        if !width.is_finite() || width <= 0. {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Line width must be positive and finite.",
            ));
        }
        let pattern = match self {
            Self::Blank | Self::Solid => return Ok(vec![]),
            Self::Dashed => 0x44,
            Self::Dotted => 0x13,
            Self::DotDash => 0x1343,
            Self::LongDash => 0x73,
            Self::TwoDash => 0x2262,
            Self::Custom(value) => value,
        };
        let digits = format!("{pattern:X}");
        if !matches!(digits.len(), 2 | 4 | 6 | 8) || digits.contains('0') {
            return Err(error(
                DiagnosticCode::Validation,
                "Invalid hexadecimal line type.",
            ));
        }
        digits
            .chars()
            .map(|d| {
                let value = f64::from(d.to_digit(16).expect("hex digit")) * width;
                if value.is_finite() {
                    Ok(value)
                } else {
                    Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Dash length overflow.",
                    ))
                }
            })
            .collect()
    }
}
/// Typed non-numeric outputs and text dimensions, separate from positional training.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum ValueAesthetic {
    /// Reference point-shape integer code zero through 25.
    Shape,
    /// Reference line type name or numeric code zero through six.
    LineType,
    /// UTF-8 label content, retained for the shared text geometry consumer.
    Label,
    /// Explicit font family selector; resolution requires caller-owned resources.
    FontFamily,
    /// Plain, bold, italic or bold.italic.
    FontFace,
    /// Positive text size in the authored aesthetic units.
    TextSize,
    /// Finite clockwise text angle in degrees.
    TextAngle,
    /// Finite horizontal justification relative to the text extent.
    HJust,
    /// Finite vertical justification relative to the text extent.
    VJust,
    /// Positive line-height multiplier.
    LineHeight,
}
impl ValueAesthetic {
    pub(crate) fn validate(self, value: &Value) -> ChartResult<()> {
        use crate::interpolate::Number;
        let valid = match (self, value) {
            (Self::Shape, Value::Number(Number(value))) => {
                value.is_finite() && value.fract() == 0. && (0. ..=25.).contains(value)
            }
            (Self::LineType, value) => return line_type(value).map(|_| ()),
            (Self::Label | Self::FontFamily, Value::Text(value)) => value.len() <= 4096,
            (Self::Label, Value::Number(Number(value))) => value.is_finite(),
            (Self::Label, Value::Boolean(_)) => true,
            (Self::FontFace, Value::Text(value)) => {
                matches!(value.as_str(), "plain" | "bold" | "italic" | "bold.italic")
            }
            (Self::TextSize | Self::LineHeight, Value::Number(Number(value))) => {
                value.is_finite() && *value > 0.
            }
            (Self::TextAngle | Self::HJust | Self::VJust, Value::Number(Number(value))) => {
                value.is_finite()
            }
            (_, Value::Missing | Value::Null) => true,
            _ => false,
        };
        if valid {
            Ok(())
        } else {
            Err(error(
                DiagnosticCode::SchemaConflict,
                "Style channel received an incompatible or out-of-range value.",
            ))
        }
    }
}
pub(super) fn line_type(value: &Value) -> ChartResult<LineType> {
    match value {
        Value::Text(value) => LineType::parse(value),
        Value::Number(crate::interpolate::Number(value))
            if value.is_finite() && value.fract() == 0. && (0. ..=6.).contains(value) =>
        {
            Ok([
                LineType::Blank,
                LineType::Solid,
                LineType::Dashed,
                LineType::Dotted,
                LineType::DotDash,
                LineType::LongDash,
                LineType::TwoDash,
            ][*value as usize])
        }
        Value::Missing | Value::Null => Ok(LineType::Blank),
        _ => Err(error(
            DiagnosticCode::SchemaConflict,
            "Line type requires a name, pattern or integer code zero through six.",
        )),
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct ValueGuides {
    pub samples: BTreeMap<crate::ScaleId, Vec<Value>>,
    pub discrete: BTreeMap<crate::ScaleId, Vec<crate::scales::GgplotDiscreteGuideEntry>>,
    pub numeric: BTreeMap<crate::ScaleId, Vec<crate::scales::GgplotContinuousGuideEntry>>,
}
impl ValueGuides {
    pub(super) fn insert(
        &mut self,
        id: crate::ScaleId,
        scale: &MappedScale,
        limits: CompileLimits,
    ) -> ChartResult<()> {
        if matches!(
            scale.spec().guide.as_deref(),
            Some(
                crate::scales::GgplotScaleGuide::Colorbar(_)
                    | crate::scales::GgplotScaleGuide::TemporalColorbar(_)
                    | crate::scales::GgplotScaleGuide::ContinuousSteps(_)
                    | crate::scales::GgplotScaleGuide::TemporalSteps(_)
            )
        ) {
            return Ok(());
        }
        if self.discrete.contains_key(&id) || self.numeric.contains_key(&id) {
            return Ok(());
        }
        if let Some(entries) = scale.discrete_guide_entries()? {
            if entries.len() > limits.max_groups {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Discrete aesthetic guide exceeds the category budget.",
                ));
            }
            self.samples.insert(
                id,
                entries
                    .iter()
                    .map(|e| scale.category(Some(&e.key)))
                    .collect::<ChartResult<_>>()?,
            );
            self.discrete.insert(id, entries);
        } else if let Some(entries) =
            scale.binned_value_guide_entries(limits.max_groups, 1_048_576)?
        {
            self.samples.insert(
                id,
                entries
                    .iter()
                    .map(|e| {
                        e.mapped.clone().map_or_else(
                            || {
                                if e.visible {
                                    scale.numeric(Some(e.value.0))
                                } else {
                                    Ok(Value::Missing)
                                }
                            },
                            Ok,
                        )
                    })
                    .collect::<ChartResult<_>>()?,
            );
            self.numeric.insert(id, entries);
        } else if let Some(entries) =
            scale.continuous_guide_entries(limits.max_groups, 1_048_576)?
        {
            self.samples.insert(
                id,
                entries
                    .iter()
                    .map(|e| {
                        e.mapped.clone().map_or_else(
                            || {
                                if e.visible {
                                    scale.numeric(Some(e.value.0))
                                } else {
                                    Ok(Value::Missing)
                                }
                            },
                            Ok,
                        )
                    })
                    .collect::<ChartResult<_>>()?,
            );
            self.numeric.insert(id, entries);
        }
        Ok(())
    }
}
pub(super) fn apply(
    layer: &Layer,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    context: super::numeric_aesthetics::NumericContext<'_>,
) -> ChartResult<BTreeMap<ValueAesthetic, NumericEncoding>> {
    let super::numeric_aesthetics::NumericContext {
        limits,
        samples,
        registry,
        guides,
        ..
    } = context;
    if (layer.value_scales.contains_key(&ValueAesthetic::Shape)
        || layer.aesthetic_values.contains_key(&ValueAesthetic::Shape))
        && !layer.reference_point()
        && !matches!(
            layer.recipe,
            Some(BuiltinRecipe::Interval(IntervalRecipe {
                kind: IntervalKind::PointRange,
                ..
            }))
        )
    {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Reference shape value mappings require point geometry.",
        ));
    }
    let mut trained = BTreeMap::new();
    for (channel, encoding) in &layer.value_scales {
        if layer.aesthetic_values.contains_key(channel)
            || (*channel == ValueAesthetic::LineType && layer.style.line_type.is_some())
        {
            continue;
        }
        if super::colors::dropped(&encoding.input, table) {
            continue;
        }
        let input = super::colors::read_inputs(
            &encoding.input,
            data,
            table,
            rows,
            limits,
            None,
            encoding.scale.has_ggplot(),
        )?;
        let scale = MappedScale::new_with_registry(
            encoding
                .scale
                .trained_value_population(samples.get(&encoding.id), registry)?,
            registry,
        )?;
        scale.validate_prepared_sampling()?;
        guides.insert(encoding.id, &scale, limits)?;
        trained.insert(
            *channel,
            NumericEncoding {
                id: encoding.id,
                input: encoding.input.clone(),
                scale: scale.spec().clone(),
            },
        );
        let source = super::colors::layer_batch(
            scale.spec(),
            samples.get(&encoding.id),
            layer.id,
            &encoding.input,
        );
        let batch =
            super::colors::sample_layer_batch(source, table, rows, &input.values, |values| {
                scale.row_palette_batch(values)
            })?;
        if batch.as_ref().is_some_and(|batch| batch.values.is_none()) {
            continue;
        }
        for (i, row) in rows.iter_mut().enumerate() {
            let value = if let Some(batch) = &batch {
                batch.values.as_ref().unwrap()[batch.indices[i]].clone()
            } else if encoding.scale.categorical() {
                let key = input.keys[i].clone().or_else(|| {
                    input.categories[i]
                        .as_ref()
                        .map(|s| ScaleKey::Text(s.clone()))
                });
                scale.category(key.as_ref())?
            } else {
                scale.numeric(input.values[i])?
            };
            channel.validate(&value)?;
            row.values.insert(*channel, value);
        }
    }
    for (channel, value) in &layer.aesthetic_values {
        channel.validate(value)?;
        for row in rows.iter_mut() {
            row.values.insert(*channel, value.clone());
        }
    }
    Ok(trained)
}
