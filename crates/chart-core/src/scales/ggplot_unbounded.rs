//! Reference numeric ranges with infinite data endpoints and finite destinations.
use super::{Bounds, GgplotRescaler, OutsidePolicy, ScaleTransform, error};
use crate::{ChartResult, DiagnosticCode, interpolate::Number};

/// Retained unbounded reference range. Undefined projections are omitted; inversion
/// is unavailable because finite destination positions do not identify finite data.
#[derive(Clone, Debug)]
pub struct GgplotUnboundedScale {
    limits: [Number; 2],
    range: Bounds,
    transform: Option<ScaleTransform>,
    outside: OutsidePolicy,
}
impl GgplotUnboundedScale {
    pub(crate) fn new(
        limits: [Number; 2],
        range: Bounds,
        transform: Option<ScaleTransform>,
        outside: OutsidePolicy,
    ) -> ChartResult<Self> {
        if limits.iter().any(|v| v.0.is_nan()) || limits.iter().all(|v| v.0.is_finite()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Unbounded reference ranges require comparable limits and an infinite endpoint.",
            ));
        }
        Ok(Self {
            limits,
            range: range.distinct()?,
            transform,
            outside,
        })
    }
    /// Panel endpoints in transformed units, encoded without nonfinite JSON numbers.
    pub fn viewport(&self) -> [Number; 2] {
        self.limits
    }
    /// Finite destination endpoints.
    pub fn range(&self) -> Bounds {
        self.range
    }
    /// Map finite transformed data using the shared reference rescaler.
    pub fn map_transformed(&self, value: f64) -> ChartResult<Option<f64>> {
        if !value.is_finite() {
            return Ok(None);
        }
        let [a, b] = self.limits.map(|v| v.0);
        let value = match self.outside {
            OutsidePolicy::Omit if value < a.min(b) || value > a.max(b) => return Ok(None),
            OutsidePolicy::Clamp => value.clamp(a.min(b), a.max(b)),
            _ => value,
        };
        reference_position(self.limits, self.range, value)
    }

    /// Transform and map a raw guide value; undefined guide positions are omitted.
    pub fn map(&self, value: f64) -> ChartResult<Option<f64>> {
        let value = match self.transform {
            Some(t) => match t.forward(value)? {
                Some(v) => v,
                None => return Ok(None),
            },
            None => value,
        };
        self.map_transformed(value)
    }
}

/// Shared reference projection for finite or infinite endpoints. Undefined
/// coordinates are omitted before constructing any destination geometry.
pub(crate) fn reference_position(
    limits: [Number; 2],
    range: Bounds,
    value: f64,
) -> ChartResult<Option<f64>> {
    if value.is_nan()
        || (!value.is_finite() && !super::ggplot::zero_range(limits[0].0, limits[1].0))
    {
        return Ok(None);
    }
    let t = GgplotRescaler::Range.parameter(value, limits[0].0, limits[1].0);
    if !t.is_finite() {
        return Ok(None);
    }
    super::linear::interpolate(range, t).map(Some)
}
