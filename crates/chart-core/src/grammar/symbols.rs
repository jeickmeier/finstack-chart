//! Area-symbol mappings and guide samples share canonical readers and scale evaluation.
use super::{compiler::EncodedRow, *};
use crate::{
    ChartResult, DiagnosticCode,
    data::DatasetSnapshot,
    shape::{SymbolKind, SymbolPaint},
};
use std::collections::{BTreeMap, BTreeSet};
/// Exact categorical symbol mapping; explicit domains keep updates and facets stable.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolEncoding {
    /// Source category or prepared group component.
    pub input: ColorInput,
    /// Ordered category labels; duplicates reject.
    pub domain: Vec<String>,
    /// Nonempty range, cycled over the declared domain.
    pub palette: Vec<SymbolKind>,
    /// Unknown/null type; None omits the mark.
    pub missing: Option<SymbolKind>,
    /// Optional guide title; an empty title hides only the title.
    pub title: Option<String>,
}
/// Requested input-domain samples for an area-size scale's guide.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolSizeGuide {
    /// Guide title; empty omits the title.
    pub title: String,
    /// Input values evaluated through the actual area-size scale.
    pub values: Vec<f64>,
}
/// An exact glyph sample, independent of legend placement.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SymbolLegendEntry {
    /// Retained registered glyph, when a custom drawing replaces the builtin type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<crate::path::PathGeometry>,
    /// User-facing category or numeric input label.
    pub label: String,
    /// Resolved symbol type.
    pub kind: SymbolKind,
    /// Actual output area/stroke size.
    pub size: f64,
    /// Explicit resolved paint mode.
    pub paint: SymbolPaint,
}
/// Prepared type/size guide, separate from the unchanged color-guide contract.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SymbolLegend {
    /// Explicit guide title.
    pub title: String,
    /// Ordered glyph samples.
    pub entries: Vec<SymbolLegendEntry>,
    /// Base layer color, with destination theme conversion applied later.
    pub color: crate::scene::Color,
    /// Actual symbol stroke width.
    pub stroke_width: f64,
}
pub(super) fn validate(layer: &Layer, limits: CompileLimits) -> ChartResult<()> {
    let Geom::ShapeSymbol { kind, size, paint } = layer.geom else {
        if layer.symbol.is_some() || layer.symbol_size_guide.is_some() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Symbol mappings and guides require area-symbol geometry.",
            ));
        }
        return Ok(());
    };
    crate::shape::Symbol::new()
        .kind(kind)
        .size(size)
        .validate()?;
    paint.resolve(kind)?;
    if let Some(s) = &layer.symbol {
        if !matches!(
            s.input,
            ColorInput::Category(_) | ColorInput::Group | ColorInput::GroupField(_)
        ) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Symbol types require a category or prepared group input.",
            ));
        }
        if s.palette.is_empty() {
            return Err(error(
                DiagnosticCode::Validation,
                "Symbol palette must not be empty.",
            ));
        }
        if s.domain.len() > limits.max_groups || s.palette.len() > limits.max_groups {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Symbol catalog exceeds category budget.",
            ));
        }
        if s.domain.iter().collect::<BTreeSet<_>>().len() != s.domain.len() {
            return Err(error(
                DiagnosticCode::Validation,
                "Symbol domain labels must be unique.",
            ));
        }
        if s.title.as_ref().is_some_and(|t| t.len() > 4096) {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Symbol guide title exceeds 4096 bytes.",
            ));
        }
        for kind in s.palette.iter().copied().chain(s.missing) {
            paint.resolve(kind)?;
        }
    }
    if let Some(g) = &layer.symbol_size_guide {
        if g.values.len() > limits.max_groups || g.title.len() > 4096 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Symbol size guide exceeds its entry or title budget.",
            ));
        }
        if g.values.iter().any(|v| !v.is_finite()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Symbol size guide inputs must be finite.",
            ));
        }
        if !layer
            .numeric_scales
            .contains_key(&NumericAesthetic::AreaSize)
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "A symbol size guide requires an AreaSize scale mapping.",
            ));
        }
    }
    Ok(())
}
pub(super) fn apply(
    layer: &Layer,
    data: &DatasetSnapshot,
    table: &PreparedTable,
    rows: &mut [EncodedRow],
    limits: CompileLimits,
    samples: &BTreeMap<crate::ScaleId, crate::scales::ScalePopulation>,
) -> ChartResult<Vec<SymbolLegend>> {
    validate(layer, limits)?;
    let Geom::ShapeSymbol { kind, size, paint } = layer.geom else {
        return Ok(vec![]);
    };
    let mut legends = vec![];
    if let Some(s) = &layer.symbol {
        let inputs = super::colors::read_inputs(&s.input, data, table, rows, limits, None)?;
        let catalog: BTreeMap<_, _> = s
            .domain
            .iter()
            .enumerate()
            .map(|(i, k)| (k.as_str(), s.palette[i % s.palette.len()]))
            .collect();
        for (row, label) in rows.iter_mut().zip(inputs.categories) {
            match label
                .as_deref()
                .and_then(|s| catalog.get(s))
                .copied()
                .or(s.missing)
            {
                Some(kind) => super::shape_encoding::set_symbol(layer.geom, row, kind),
                None => {
                    row.x = None;
                    row.y = None;
                }
            }
        }
        legends.push(SymbolLegend {
            title: s.title.clone().unwrap_or_else(|| "Symbol".into()),
            entries: s
                .domain
                .iter()
                .enumerate()
                .map(|(i, label)| {
                    let kind = s.palette[i % s.palette.len()];
                    Ok(SymbolLegendEntry {
                        geometry: None,
                        label: label.clone(),
                        kind,
                        size,
                        paint: paint.resolve(kind)?,
                    })
                })
                .collect::<ChartResult<_>>()?,
            color: layer.style.color.resolve(),
            stroke_width: layer.style.stroke_width,
        });
    }
    if let Some(g) = &layer.symbol_size_guide {
        let encoding = &layer.numeric_scales[&NumericAesthetic::AreaSize];
        let scale = crate::scales::MappedScale::for_numbers(
            encoding
                .scale
                .trained_population(samples.get(&encoding.id))?,
        )?;
        let entries = g
            .values
            .iter()
            .map(|value| {
                let crate::interpolate::Value::Number(crate::interpolate::Number(size)) =
                    scale.numeric(Some(*value))?
                else {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Symbol size guide sample maps to missing or nonnumeric output.",
                    ));
                };
                crate::shape::Symbol::new().size(size).validate()?;
                Ok(SymbolLegendEntry {
                    geometry: None,
                    label: value.to_string(),
                    kind,
                    size,
                    paint: paint.resolve(kind)?,
                })
            })
            .collect::<ChartResult<_>>()?;
        legends.push(SymbolLegend {
            title: g.title.clone(),
            entries,
            color: layer.style.color.resolve(),
            stroke_width: layer.style.stroke_width,
        });
    }
    Ok(legends)
}

pub(super) fn custom_glyphs(
    legends: &mut [SymbolLegend],
    symbol: &(dyn crate::shape::SymbolDraw + Send + Sync),
    limits: CompileLimits,
    vertices: &mut usize,
) -> ChartResult<()> {
    for entry in legends.iter_mut().flat_map(|l| &mut l.entries) {
        let geometry = crate::shape::Symbol::new()
            .size(entry.size)
            .limits(crate::shape::ShapeLimits {
                max_points: 1,
                path: crate::path::PathLimits {
                    max_commands: limits.max_vertices.min(*vertices),
                    ..Default::default()
                },
            })
            .generate_with(symbol, entry.size)?
            .geometry();
        super::compiler::charge(vertices, geometry.commands().len(), "custom symbol guide")?;
        entry.geometry = Some(geometry);
    }
    Ok(())
}
