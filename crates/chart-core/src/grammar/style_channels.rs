//! Non-numeric style channels use the same typed mapping engine as numeric aesthetics.
use super::{compiler::EncodedRow, *};
use crate::{
    ChartResult, DiagnosticCode,
    data::DatasetSnapshot,
    interpolate::Value,
    scales::{MappedScale, ScaleKey},
};
use std::collections::BTreeMap;

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
    pub(super) fn validate(self, value: &Value) -> ChartResult<()> {
        use crate::interpolate::Number;
        let valid = match (self, value) {
            (Self::Shape, Value::Number(Number(value))) => {
                value.is_finite() && value.fract() == 0. && (0. ..=25.).contains(value)
            }
            (Self::LineType, value) => return line_type(value).map(|_| ()),
            (Self::Label | Self::FontFamily, Value::Text(value)) => value.len() <= 4096,
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
pub(super) fn apply(
    layer: &Layer,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
    samples: &BTreeMap<crate::ScaleId, crate::scales::ScalePopulation>,
    registry: &ExtensionRegistry,
) -> ChartResult<BTreeMap<ValueAesthetic, NumericEncoding>> {
    if (layer.value_scales.contains_key(&ValueAesthetic::Shape)
        || layer.aesthetic_values.contains_key(&ValueAesthetic::Shape))
        && layer.geom != Geom::Point
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
                .trained_population(samples.get(&encoding.id), registry)?,
            registry,
        )?;
        scale.validate_prepared_sampling()?;
        trained.insert(
            *channel,
            NumericEncoding {
                id: encoding.id,
                input: encoding.input.clone(),
                scale: scale.spec().clone(),
            },
        );
        for (i, row) in rows.iter_mut().enumerate() {
            let value = if encoding.scale.categorical() {
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
