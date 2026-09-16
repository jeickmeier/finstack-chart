//! Reference count/column/rug/curve/spoke recipe controls and data-space setup.
use super::compiler::EncodedRow;
use super::*;
use crate::{ChartResult, Point};

/// Data-space column widths, separate from legacy display-unit bars.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ColumnRecipe {
    /// Width in positional calculation units; omitted uses 0.9 times resolution.
    pub width: Option<f64>,
    /// Fraction of width to the left of the center; default 0.5.
    pub just: f64,
}
impl Default for ColumnRecipe {
    fn default() -> Self {
        Self {
            width: None,
            just: 0.5,
        }
    }
}
/// Joint-position count mark; the default statistic supplies count-scaled size.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CountRecipe {}
/// Marginal tick marks with physical panel-relative lengths.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RugRecipe {
    /// Any combination of bottom/left/top/right: b/l/t/r.
    pub sides: String,
    /// Fraction of the perpendicular physical panel dimension.
    pub length: f64,
    /// Draw outward from the panel border instead of inward.
    pub outside: bool,
}
impl Default for RugRecipe {
    fn default() -> Self {
        Self {
            sides: "bl".into(),
            length: 0.03,
            outside: false,
        }
    }
}
/// grid-compatible rational X-spline curve, constructed after coordinate projection.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CurveRecipe {
    /// Signed curvature; zero is a straight segment.
    pub curvature: f64,
    /// Control-polygon skew angle in degrees; default 90.
    pub angle: f64,
    /// Number of internal control points; default five.
    pub ncp: usize,
    /// Optional endpoint arrows.
    pub arrow: Option<ArrowSpec>,
}
impl Default for CurveRecipe {
    fn default() -> Self {
        Self {
            curvature: 0.5,
            angle: 90.,
            ncp: 5,
            arrow: None,
        }
    }
}
/// Data-space radial segment, measured counterclockwise from positive x.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpokeRecipe {
    /// Constant angle in radians unless mapped.
    pub angle: f64,
    /// Signed radius in data units unless mapped.
    pub radius: f64,
    /// Optional endpoint arrows.
    pub arrow: Option<ArrowSpec>,
}
impl Default for SpokeRecipe {
    fn default() -> Self {
        Self {
            angle: 0.,
            radius: 1.,
            arrow: None,
        }
    }
}
/// Deferred marks retain original source endpoints and one semantic target.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub enum PreparedMarkRecipe {
    /// Project endpoints, then construct the physical curve.
    Curve {
        /// Original start.
        from: Point,
        /// Original end.
        to: Point,
        /// Physical curve controls.
        controls: CurveRecipe,
    },
    /// One source center; generated marginal ticks share its target.
    Rug {
        /// Optional source horizontal coordinate.
        x: Option<f64>,
        /// Optional source vertical coordinate.
        y: Option<f64>,
        /// Marginal tick controls.
        controls: RugRecipe,
    },
    /// Source-space spoke, with shared segment arrow semantics.
    Spoke {
        /// Original center.
        from: Point,
        /// Calculated data endpoint.
        to: Point,
        /// Arrow controls.
        controls: SpokeRecipe,
    },
}
impl PreparedMarkRecipe {
    /// Positional training anchors; curve control points/rug edges are display-only.
    pub fn points(&self) -> Vec<Point> {
        match self {
            Self::Curve { from, to, .. } | Self::Spoke { from, to, .. } => vec![*from, *to],
            Self::Rug {
                x: Some(x),
                y: Some(y),
                ..
            } => Point::new(*x, *y).into_iter().collect(),
            Self::Rug { .. } => vec![],
        }
    }
    /// Swap physical axes after normalized position processing.
    pub fn transpose(&mut self) -> ChartResult<()> {
        fn swap(p: &mut Point) -> ChartResult<()> {
            *p = Point::new(p.y(), p.x())?;
            Ok(())
        }
        match self {
            Self::Curve { from, to, .. } | Self::Spoke { from, to, .. } => {
                swap(from)?;
                swap(to)?
            }
            Self::Rug { x, y, .. } => std::mem::swap(x, y),
        }
        Ok(())
    }
}
fn invalid(message: &str) -> crate::Diagnostic {
    super::error(crate::DiagnosticCode::NumericalDomain, message)
}
pub(super) fn validate(recipe: &BuiltinRecipe) -> ChartResult<()> {
    match recipe {
        BuiltinRecipe::Column(v)
            if !v.just.is_finite() || v.width.is_some_and(|w| !w.is_finite() || w < 0.) =>
        {
            Err(invalid(
                "Column width must be nonnegative and justification finite.",
            ))
        }
        BuiltinRecipe::Rug(v)
            if !v.length.is_finite()
                || v.length < 0.
                || v.sides.chars().any(|c| !"bltr".contains(c)) =>
        {
            Err(invalid(
                "Rug length must be nonnegative and sides contain only b/l/t/r.",
            ))
        }
        BuiltinRecipe::Curve(v)
            if !v.curvature.is_finite() || !v.angle.is_finite() || v.ncp == 0 || v.ncp > 4096 =>
        {
            Err(invalid(
                "Curve controls must be finite and ncp in 1..=4096.",
            ))
        }
        BuiltinRecipe::Spoke(v) if !v.angle.is_finite() || !v.radius.is_finite() => {
            Err(invalid("Spoke angle and radius must be finite."))
        }
        _ => Ok(()),
    }
}
pub(super) fn setup(
    layer: &Layer,
    rows: &mut [EncodedRow],
    _limits: CompileLimits,
) -> ChartResult<()> {
    let Some(recipe) = &layer.recipe else {
        return Ok(());
    };
    validate(recipe)?;
    match recipe {
        BuiltinRecipe::Column(v) => {
            let default = v
                .width
                .unwrap_or(0.9 * super::recipe_emit::resolution(rows.iter().filter_map(|r| r.x)));
            for row in rows {
                let Some(width) =
                    super::recipe_emit::number_or(row, RecipeAesthetic::Width, default)
                else {
                    row.x2 = None;
                    row.y2 = None;
                    continue;
                };
                if !width.is_finite() || width < 0. {
                    return Err(invalid("Column widths must be finite and nonnegative."));
                }
                if let (Some(x), Some(y)) = (row.x, row.y) {
                    row.x = Some(x - width * v.just);
                    row.x2 = Some(x + width * (1. - v.just));
                    row.y = Some(y);
                    row.y2 = Some(0.);
                }
            }
        }
        BuiltinRecipe::Spoke(v) => {
            for row in rows {
                let angle = super::recipe_emit::number_or(row, RecipeAesthetic::Angle, v.angle);
                let radius = super::recipe_emit::number_or(row, RecipeAesthetic::Radius, v.radius);
                let (Some(angle), Some(radius)) = (angle, radius) else {
                    row.x2 = None;
                    row.y2 = None;
                    continue;
                };
                if let (Some(x), Some(y)) = (row.x, row.y) {
                    row.x2 = Some(x + radius * libm::cos(angle));
                    row.y2 = Some(y + radius * libm::sin(angle));
                }
            }
        }
        _ => {}
    }
    Ok(())
}
pub(super) fn emit(
    layer: &Layer,
    rows: &[EncodedRow],
    prepared: &mut PreparedLayer,
    remaining: &mut usize,
) -> ChartResult<bool> {
    let Some(recipe) = &layer.recipe else {
        return Ok(false);
    };
    if matches!(recipe, BuiltinRecipe::Column(_)) {
        for row in rows {
            let (Some(x), Some(y), Some(x2), Some(y2)) = (row.x, row.y, row.x2, row.y2) else {
                super::recipe_emit::retain_positions(prepared, row);
                continue;
            };
            super::recipe_emit::push(
                prepared,
                row,
                layer,
                PreparedGeometry::Rectangle {
                    from: Point::new(x, y)?,
                    to: Point::new(x2, y2)?,
                },
                remaining,
            )?;
            if let Some(grammar) = &layer.grammar {
                let mark = std::sync::Arc::make_mut(&mut prepared.marks)
                    .last_mut()
                    .unwrap();
                mark.style = super::recipe_surfaces::surface_style(layer, row)?;
                if mark.style.stroke.is_none() && (row.color.is_some() || !grammar.default_color) {
                    mark.style.stroke = Some(mark.style.color);
                }
            }
        }
        return Ok(true);
    }
    if matches!(recipe, BuiltinRecipe::Count(_)) {
        let mapped_size = layer.numeric_scales.contains_key(&NumericAesthetic::Size)
            || layer
                .numeric_scales
                .contains_key(&NumericAesthetic::AreaSize)
            || layer.after_scale.contains_key(&AfterScaleAesthetic::Size)
            || match &layer.mappings {
                Mappings::Source(a) => a.size.is_some(),
                Mappings::Statistical(a) => a.size.is_some(),
                Mappings::Binned(a) => a.size.is_some(),
            };
        for row in rows {
            let (Some(x), Some(y)) = (row.x, row.y) else {
                continue;
            };
            if mapped_size && row.size.is_none_or(f64::is_nan) {
                continue;
            }
            super::recipe_emit::push(
                prepared,
                row,
                layer,
                PreparedGeometry::Point(Point::new(x, y)?),
                remaining,
            )?;
            if let Some(size) = row.size {
                std::sync::Arc::make_mut(&mut prepared.marks)
                    .last_mut()
                    .unwrap()
                    .style
                    .radius = size;
            }
        }
        return Ok(true);
    }
    if !matches!(
        recipe,
        BuiltinRecipe::Curve(_) | BuiltinRecipe::Rug(_) | BuiltinRecipe::Spoke(_)
    ) {
        return Ok(false);
    }
    for row in rows {
        if let BuiltinRecipe::Rug(v) = recipe {
            if row.x.is_some() || row.y.is_some() {
                super::recipe_emit::push(
                    prepared,
                    row,
                    layer,
                    PreparedGeometry::Recipe(Box::new(PreparedRecipe::Mark(
                        PreparedMarkRecipe::Rug {
                            x: row.x,
                            y: row.y,
                            controls: v.clone(),
                        },
                    ))),
                    remaining,
                )?;
            }
            continue;
        }
        let (Some(x), Some(y)) = (row.x, row.y) else {
            continue;
        };
        let center = Point::new(x, y)?;
        let shape = match recipe {
            BuiltinRecipe::Curve(v) => {
                let (Some(x2), Some(y2)) = (row.x2, row.y2) else {
                    super::recipe_emit::retain_positions(prepared, row);
                    continue;
                };
                PreparedMarkRecipe::Curve {
                    from: center,
                    to: Point::new(x2, y2)?,
                    controls: v.clone(),
                }
            }
            BuiltinRecipe::Spoke(v) => {
                let (Some(x2), Some(y2)) = (row.x2, row.y2) else {
                    super::recipe_emit::retain_positions(prepared, row);
                    continue;
                };
                PreparedMarkRecipe::Spoke {
                    from: center,
                    to: Point::new(x2, y2)?,
                    controls: v.clone(),
                }
            }
            _ => unreachable!(),
        };
        super::recipe_emit::push(
            prepared,
            row,
            layer,
            PreparedGeometry::Recipe(Box::new(PreparedRecipe::Mark(shape))),
            remaining,
        )?;
    }
    Ok(true)
}
