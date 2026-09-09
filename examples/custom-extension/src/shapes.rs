//! FIX-S08 implementations outside core, reused verbatim by both proof hosts.
use chart_core::{ChartResult, DiagnosticCode, Revision, grammar::*, path::Path, shape::*};
use serde_json::Value;
use std::{cmp::Ordering, sync::Arc};

/// Install one version of each public shape protocol and a native-only curve.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    for family in [
        ShapeFamily::Curve,
        ShapeFamily::Symbol,
        ShapeFamily::PieComparator,
        ShapeFamily::StackOrder,
        ShapeFamily::StackOffset,
    ] {
        registry.register_shape(Arc::new(Example {
            family,
            portable: true,
        }))?;
    }
    registry.register_shape(Arc::new(Example {
        family: ShapeFamily::Curve,
        portable: false,
    }))
}
struct Example {
    family: ShapeFamily,
    portable: bool,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Amount {
    amount: f64,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Field {
    field: String,
}
fn parameter<T: serde::de::DeserializeOwned>(value: &Value) -> ChartResult<T> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
fn amount(value: &Value) -> ChartResult<f64> {
    let p: Amount = parameter(value)?;
    if !p.amount.is_finite() {
        return Err(super::error(
            DiagnosticCode::NumericalDomain,
            "Amount must be finite.",
        ));
    }
    Ok(p.amount)
}
impl CustomShape for Example {
    fn descriptor(&self) -> ExtensionDescriptor {
        let name = if !self.portable {
            "example.native_curve"
        } else {
            match self.family {
                ShapeFamily::Curve => "example.shift_curve",
                ShapeFamily::Symbol => "example.rectangle_symbol",
                ShapeFamily::PieComparator => "example.field_comparator",
                ShapeFamily::StackOrder => "example.first_value_order",
                ShapeFamily::StackOffset => "example.shift_offset",
            }
        };
        ExtensionDescriptor::batch(name, Revision::new(1), self.portable)
    }
    fn family(&self) -> ShapeFamily {
        self.family
    }
    fn resolve(&self, parameters: &Value) -> ChartResult<ShapeProtocol> {
        Ok(match self.family {
            ShapeFamily::Curve => ShapeProtocol::Curve(Box::new(ShiftCurve(amount(parameters)?))),
            ShapeFamily::Symbol => {
                let ratio = amount(parameters)?;
                if ratio <= 0. {
                    return Err(super::error(
                        DiagnosticCode::Validation,
                        "Rectangle aspect ratio must be positive.",
                    ));
                }
                ShapeProtocol::Symbol(Box::new(Rectangle(ratio)))
            }
            ShapeFamily::PieComparator => {
                let p: Field = parameter(parameters)?;
                if p.field.is_empty() || p.field.len() > 128 {
                    return Err(super::error(
                        DiagnosticCode::Validation,
                        "Comparator field requires 1..128 bytes.",
                    ));
                }
                ShapeProtocol::PieComparator(Box::new(FieldComparator(p.field)))
            }
            ShapeFamily::StackOrder => {
                if parameters != &serde_json::json!({}) {
                    return Err(super::error(
                        DiagnosticCode::Validation,
                        "First-value order accepts no parameters.",
                    ));
                }
                ShapeProtocol::StackOrder(Box::new(FirstValueOrder))
            }
            ShapeFamily::StackOffset => {
                ShapeProtocol::StackOffset(Box::new(ShiftOffset(amount(parameters)?)))
            }
        })
    }
}
/// Shift every Cartesian curve point, preserving the complete area lifecycle.
pub struct ShiftCurve(pub f64);
struct ShiftContext<'a> {
    inner: Box<dyn CurveProtocol + 'a>,
    shift: f64,
}
impl CurveFactory for ShiftCurve {
    fn command_bound(&self, max_points: usize) -> Option<usize> {
        max_points.checked_mul(2)
    }
    fn supports_area(&self) -> bool {
        true
    }
    fn create<'a>(
        &'a self,
        path: &'a mut Path,
        max_points: usize,
    ) -> ChartResult<Box<dyn CurveProtocol + 'a>> {
        Ok(Box::new(ShiftContext {
            inner: CurveSpec::Linear.create(path, max_points)?,
            shift: self.0,
        }))
    }
}
impl CurveProtocol for ShiftContext<'_> {
    fn area_start(&mut self) -> ChartResult<()> {
        self.inner.area_start()
    }
    fn area_end(&mut self) -> ChartResult<()> {
        self.inner.area_end()
    }
    fn line_start(&mut self) -> ChartResult<()> {
        self.inner.line_start()
    }
    fn line_end(&mut self) -> ChartResult<()> {
        self.inner.line_end()
    }
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()> {
        self.inner.point(x, y + self.shift)
    }
}
/// Centered rectangle, parameterized by width/height, with requested filled area.
pub struct Rectangle(pub f64);
impl SymbolDraw for Rectangle {
    fn draw(&self, path: &mut Path, size: f64) -> ChartResult<()> {
        let width = (size * self.0).sqrt();
        let height = (size / self.0).sqrt();
        path.rect(-width / 2., -height / 2., width, height)
    }
}
/// Exact unsigned integer field comparison; large source IDs are never narrowed to f64.
pub struct FieldComparator(pub String);
impl PieComparator for FieldComparator {
    fn compare(&self, left: &Value, _: f64, right: &Value, _: f64) -> ChartResult<Ordering> {
        let field = |value: &Value| {
            value
                .get(&self.0)
                .and_then(|v| {
                    v.as_u64()
                        .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
                })
                .ok_or_else(|| {
                    super::error(
                        DiagnosticCode::Validation,
                        "Comparator requires an exact unsigned integer field.",
                    )
                })
        };
        Ok(field(left)?.cmp(&field(right)?))
    }
}
/// Stable ascending first finite sample, with zero for empty or missing first samples.
pub struct FirstValueOrder;
impl StackOrdering for FirstValueOrder {
    fn work_units(&self, series: usize, _: usize) -> Option<usize> {
        series.checked_mul(series.checked_ilog2().unwrap_or(0) as usize + 1)
    }
    fn order(&self, series: &[Vec<[f64; 2]>]) -> ChartResult<Vec<usize>> {
        let first = |index: usize| {
            series[index]
                .first()
                .map(|p| p[1])
                .filter(|v| v.is_finite())
                .unwrap_or(0.)
        };
        let mut result: Vec<_> = (0..series.len()).collect();
        result.sort_by(|a, b| first(*a).partial_cmp(&first(*b)).expect("finite"));
        Ok(result)
    }
}
/// Translate both endpoints of the common cumulative stack without changing heights.
pub struct ShiftOffset(pub f64);
impl StackOffsetting for ShiftOffset {
    fn work_units(&self, series: usize, samples: usize) -> Option<usize> {
        series.checked_mul(samples)?.checked_mul(2)
    }
    fn offset(&self, series: &mut [Vec<[f64; 2]>], order: &[usize]) -> ChartResult<()> {
        StackOffset::None.offset(series, order)?;
        for sample in series.iter_mut().flatten() {
            sample[0] += self.0;
            sample[1] += self.0;
        }
        Ok(())
    }
}
