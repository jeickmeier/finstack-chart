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
    /// Replace paint alpha with a value in the closed unit interval.
    Alpha,
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
pub(super) struct NumericContext<'a> {
    pub limits: CompileLimits,
    pub samples: &'a BTreeMap<ScaleId, crate::scales::ScalePopulation>,
    pub registry: &'a ExtensionRegistry,
    pub profile: Profile,
}
pub(super) fn apply(
    layer: &Layer,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    context: NumericContext<'_>,
) -> ChartResult<BTreeMap<NumericAesthetic, NumericEncoding>> {
    let NumericContext {
        limits,
        samples,
        registry,
        profile,
    } = context;
    let mut trained = BTreeMap::new();
    for (aesthetic, encoding) in &layer.numeric_scales {
        if *aesthetic == NumericAesthetic::Alpha && layer.style.alpha.is_some() {
            continue;
        }
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
        let input = super::colors::read_inputs(
            &encoding.input,
            data,
            table,
            rows,
            limits,
            None,
            encoding.scale.has_ggplot(),
        )?;
        let spec = encoding
            .scale
            .trained_population(samples.get(&encoding.id), registry)?;
        if spec.guide_entries() > limits.max_groups {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Trained numeric scale exceeds category budget.",
            ));
        }
        let scale = MappedScale::for_numbers_with_registry(spec, registry)?;
        trained.insert(
            *aesthetic,
            NumericEncoding {
                id: encoding.id,
                input: encoding.input.clone(),
                scale: scale.spec().clone(),
            },
        );
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
            // Reference identity output palettes may retain IEEE NaN. Treat it
            // as a missing aesthetic at the grammar boundary, just like NA from
            // other palettes. Alpha retains its existing pre-paint handling.
            let result = if profile == Profile::Ggplot2_4_0_3
                && *aesthetic != NumericAesthetic::Alpha
                && matches!(result, Value::Number(Number(v)) if v.is_nan())
            {
                Value::Missing
            } else {
                result
            };
            let value = match result {
                Value::Number(Number(v))
                    if profile == Profile::Ggplot2_4_0_3
                        && *aesthetic == NumericAesthetic::Alpha =>
                {
                    // Reference alpha is an aesthetic until paint lowering:
                    // after_scale must see finite values outside [0,1] unchanged.
                    row.alpha = (!v.is_nan()).then_some(v);
                    row.set_missing(AfterScaleAesthetic::Alpha, v.is_nan());
                    continue;
                }
                Value::Missing | Value::Null => {
                    let channel = match aesthetic {
                        NumericAesthetic::Size | NumericAesthetic::AreaSize => {
                            Some(AfterScaleAesthetic::Size)
                        }
                        NumericAesthetic::Alpha => Some(AfterScaleAesthetic::Alpha),
                        NumericAesthetic::StrokeWidth => Some(AfterScaleAesthetic::LineWidth),
                        _ => None,
                    };
                    if profile == Profile::Ggplot2_4_0_3
                        && let Some(channel) = channel
                    {
                        row.set_missing(channel, true);
                        continue;
                    }
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
            if matches!(
                aesthetic,
                NumericAesthetic::Opacity | NumericAesthetic::Alpha
            ) {
                if !(0. ..=1.).contains(&value) {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Mapped opacity must lie in the closed unit interval.",
                    ));
                }
                if *aesthetic == NumericAesthetic::Alpha {
                    row.alpha = Some(value);
                } else {
                    row.opacity = Some(value);
                }
            } else if (value < 0.
                && !(profile == Profile::Ggplot2_4_0_3
                    && layer.geom == Geom::Point
                    && *aesthetic == NumericAesthetic::Size))
                || (value == 0.
                    && !(profile == Profile::Ggplot2_4_0_3
                        && (layer.geom == Geom::Point
                            || (layer.geom.reference_linewidth()
                                && matches!(
                                    aesthetic,
                                    NumericAesthetic::Size | NumericAesthetic::StrokeWidth
                                )))))
            {
                row.x = None;
                row.y = None;
            } else if *aesthetic == NumericAesthetic::Size {
                row.size = Some(value);
            } else {
                row.stroke_width = Some(value);
            }
        }
    }
    Ok(trained)
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

pub(super) fn apply_alpha(
    mut paint: crate::scene::Color,
    alpha: Option<f64>,
) -> crate::scene::Color {
    if let Some(alpha) = alpha.filter(|v| !v.is_nan()) {
        // Reference farver conversion retains original coverage for NA/NaN,
        // saturates finite values, and lowers either infinity to zero coverage.
        paint.alpha = if alpha.is_infinite() {
            0
        } else {
            (alpha * 255.).round() as u8
        };
    }
    paint
}
