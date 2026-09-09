//! Nonpositional numeric style/shape outputs use the same readers and mapping engines as color.
use super::{compiler::EncodedRow, *};
use crate::{
    ChartResult, DiagnosticCode, ScaleId,
    data::DatasetSnapshot,
    interpolate::{Number, Value},
    scales::{MappedScale, MappedScaleSpec, ScaleKey},
};
use std::collections::BTreeMap;

/// Numeric style output, separate from the source field's values and positional training.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum NumericAesthetic {
    /// Area/stroke-size symbol measure, separate from the legacy point radius channel.
    AreaSize,
    /// Point radius; for the established rule-size mapping, stroke width.
    Size,
    /// Multiply the resulting paint alpha by a value in the closed unit interval.
    Opacity,
    /// Explicit stroke width, independent of point radius.
    StrokeWidth,
    /// Shared radial angle in radians clockwise from twelve o'clock.
    Angle,
    /// Shared signed radial radius in destination units.
    Radius,
    /// Arc or radial area/link inner radius in destination units.
    InnerRadius,
    /// Arc outer radius in destination units.
    OuterRadius,
    /// Arc start angle, radians clockwise from twelve o'clock.
    StartAngle,
    /// Arc end angle in radians.
    EndAngle,
    /// Arc gap angle in radians.
    PadAngle,
    /// Explicit arc padding radius in destination units.
    PadRadius,
    /// Arc corner radius in destination units.
    CornerRadius,
    /// Pie weight; nonpositive values receive zero angular weight.
    PieValue,
}
/// Stage-aware numeric output scale; adapters never implement its mapping arithmetic.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericEncoding {
    /// Shared population identity.
    pub id: ScaleId,
    /// Source, group or statistical input.
    pub input: ColorInput,
    /// Common typed scale descriptor, required to produce numeric/missing values.
    pub scale: MappedScaleSpec,
}
pub(super) fn apply(
    layer: &Layer,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
    samples: &BTreeMap<ScaleId, crate::scales::ScalePopulation>,
) -> ChartResult<()> {
    for (aesthetic, encoding) in &layer.numeric_scales {
        if *aesthetic == NumericAesthetic::Size
            && (layer.geom.run().is_some()
                || matches!(
                    layer.geom,
                    Geom::Rectangle
                        | Geom::ShapeArc { .. }
                        | Geom::ShapePie { .. }
                        | Geom::ShapeSymbol { .. }
                ))
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Mapped size supports points and rules; use explicit stroke width for runs and rectangles.",
            ));
        }
        super::shape_encoding::validate_channel(layer.geom, *aesthetic)?;
        let input = super::colors::read_inputs(&encoding.input, data, table, rows, limits, None)?;
        let scale = MappedScale::for_numbers(
            encoding
                .scale
                .trained_population(samples.get(&encoding.id))?,
        )?;
        for (i, row) in rows.iter_mut().enumerate() {
            let result = if matches!(
                encoding.input,
                ColorInput::Category(_) | ColorInput::Group | ColorInput::GroupField(_)
            ) {
                let key = input.keys[i].clone().or_else(|| {
                    input.categories[i]
                        .as_ref()
                        .map(|s| ScaleKey::Text(s.clone()))
                });
                scale.category(key.as_ref())?
            } else {
                scale.numeric(input.values[i])?
            };
            let value = match result {
                Value::Missing | Value::Null => {
                    row.x = None;
                    row.y = None;
                    continue;
                }
                Value::Number(Number(v)) if v.is_finite() => v,
                _ => {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Numeric aesthetic scales must produce finite numbers or explicit missing values.",
                    ));
                }
            };
            if super::shape_encoding::set(layer.geom, row, *aesthetic, value)? {
                continue;
            }
            if *aesthetic == NumericAesthetic::Opacity {
                if !(0. ..=1.).contains(&value) {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Mapped opacity must lie in the closed unit interval.",
                    ));
                }
                row.opacity = Some(value);
            } else if value <= 0. {
                row.x = None;
                row.y = None;
            } else if *aesthetic == NumericAesthetic::Size {
                row.size = Some(value);
            } else {
                row.stroke_width = Some(value);
            }
        }
    }
    Ok(())
}
pub(super) fn apply_opacity(
    paint: crate::color::Paint,
    opacity: Option<f64>,
) -> crate::scene::Color {
    opacity.map_or_else(
        || paint.resolve(),
        |opacity| {
            let color = paint.value();
            color.with_opacity(color.opacity() * opacity).to_paint()
        },
    )
}
