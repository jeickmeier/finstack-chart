//! Reference numeric ranges with infinite data endpoints and finite destinations.
use super::{Bounds, GgplotRescaler, OutsidePolicy, ScaleTransform, error};
use crate::{ChartResult, DiagnosticCode, interpolate::Number};

/// Retained reference range, including infinite data or inverse endpoints.
/// Inversion is available only for a finite, one-to-one built-in viewport.
#[derive(Clone, Debug)]
pub struct GgplotUnboundedScale {
    limits: [Number; 2],
    viewport: [Number; 2],
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
        if limits.iter().any(|v| v.0.is_nan()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Reference projection limits must be comparable.",
            ));
        }
        Ok(Self {
            limits,
            viewport: limits,
            range: range.distinct()?,
            transform,
            outside,
        })
    }
    /// Panel endpoints in transformed units, encoded without nonfinite JSON numbers.
    pub fn viewport(&self) -> [Number; 2] {
        self.viewport
    }
    /// Retained scale limits, independent of a finite coordinate viewport.
    pub fn limits(&self) -> [Number; 2] {
        self.limits
    }
    /// Use transformed coordinate bounds without retraining the population.
    /// The caller owns coordinate expansion and transformation. Infinite source
    /// positions reach the corresponding viewport edge; finite positions remain
    /// available for destination clipping.
    pub fn with_transformed_viewport(mut self, viewport: [Number; 2]) -> ChartResult<Self> {
        if viewport.iter().any(|v| v.0.is_nan()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Coordinate viewport endpoints must be comparable.",
            ));
        }
        self.viewport = viewport;
        Ok(self)
    }
    /// Retained raw-to-transformed coordinate operation.
    pub(crate) fn transform(&self) -> Option<ScaleTransform> {
        self.transform.clone()
    }
    /// Finite destination endpoints.
    pub fn range(&self) -> Bounds {
        self.range
    }
    /// Finite source viewport when the built-in transform has a valid inverse branch.
    pub(crate) fn invertible_viewport(&self) -> Option<Bounds> {
        self.invertible_bounds(self.viewport)
    }
    /// Finite source training range on the same valid transform branch.
    pub(crate) fn invertible_domain(&self) -> Option<Bounds> {
        self.invertible_bounds(self.limits)
    }
    fn invertible_bounds(&self, bounds: [Number; 2]) -> Option<Bounds> {
        let Some(ScaleTransform::Ggplot { ref transform }) = self.transform else {
            return None;
        };
        let transformed = bounds.map(|v| v.0);
        if transformed.iter().any(|v| !v.is_finite())
            || super::ggplot::zero_range(transformed[0], transformed[1])
        {
            return None;
        }
        let source = transformed.map(|v| transform.inverse(v));
        if !transform.monotone_on(&source)
            || source.iter().zip(transformed).any(|(x, expected)| {
                (transform.forward(*x) - expected).abs() > 4e-14 * expected.abs().max(1.)
            })
        {
            return None;
        }
        Bounds::new(source[0], source[1]).ok()
    }
    /// Whether destination positions have a finite numeric inverse on this viewport.
    pub fn has_numeric_inverse(&self) -> bool {
        self.invertible_viewport().is_some()
    }
    /// Recover a raw numeric value through the retained transformed viewport.
    pub fn invert(&self, position: f64) -> ChartResult<f64> {
        if !self.has_numeric_inverse() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "This reference viewport has no finite one-to-one numeric inverse.",
            ));
        }
        let transformed = super::numeric::legacy_map(
            self.range,
            Bounds::new(self.viewport[0].0, self.viewport[1].0)?,
            position,
        )?;
        self.transform
            .as_ref()
            .expect("validated built-in transform")
            .inverse(transformed)
    }
    /// Map transformed data using the shared reference rescaler and coordinate policy.
    pub fn map_transformed(&self, value: f64) -> ChartResult<Option<f64>> {
        if !value.is_finite() {
            return self.project(value);
        }
        let [a, b] = self.limits.map(|v| v.0);
        let value = match self.outside {
            OutsidePolicy::Omit if value < a.min(b) || value > a.max(b) => return Ok(None),
            OutsidePolicy::Clamp => value.clamp(a.min(b), a.max(b)),
            _ => value,
        };
        self.project(value)
    }

    fn project(&self, value: f64) -> ChartResult<Option<f64>> {
        coordinate_position(self.viewport, self.range, value)
    }

    /// Transform and map a raw guide value; undefined guide positions are omitted.
    pub fn map(&self, value: f64) -> ChartResult<Option<f64>> {
        let value = match &self.transform {
            Some(t) => t.forward_raw(value),
            None => value,
        };
        // Guides retain scale rescaling; only geometry receives the final
        // Cartesian squishing of infinite normalized positions.
        if !value.is_finite() {
            return reference_position(self.viewport, self.range, value);
        }
        self.map_transformed(value)
    }
}

/// Cartesian reference coordinates squish only infinite normalized positions.
/// Ordinary finite extrapolation remains available for destination clipping.
pub(crate) fn coordinate_position(
    viewport: [Number; 2],
    range: Bounds,
    value: f64,
) -> ChartResult<Option<f64>> {
    if value.is_infinite() && viewport.iter().all(|v| v.0.is_finite()) {
        let t = GgplotRescaler::Range.parameter(value, viewport[0].0, viewport[1].0);
        return super::linear::interpolate(range, t.clamp(0., 1.)).map(Some);
    }
    reference_position(viewport, range, value)
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

#[cfg(test)]
mod viewport_tests {
    use super::*;
    #[test]
    fn builtin_inverse_uses_retained_transformed_viewport() {
        use super::super::GgplotTransform as T;
        for (transform, endpoints, position, expected) in [
            (T::Asinh, [0., 2_f64.asinh()], 100., 0.),
            (T::Reverse, [-4., -2.], 320., 3.),
            (T::Exp { base: 2. }, [1., 4.], 320., 2.5_f64.log2()),
            (T::Reciprocal, [0.25, 1.], 320., 1.6),
        ] {
            let scale = GgplotUnboundedScale::new(
                endpoints.map(Number),
                Bounds::new(100., 540.).unwrap(),
                Some(ScaleTransform::Ggplot {
                    transform: transform.clone(),
                }),
                OutsidePolicy::Extend,
            )
            .unwrap();
            assert!(scale.has_numeric_inverse(), "{transform:?}");
            assert!((scale.invert(position).unwrap() - expected).abs() < 1e-13);
        }
        for (transform, endpoints) in [
            (T::Reciprocal, [-1., 1.]),
            (T::Exp { base: 2. }, [-1., 4.]),
            (T::Sqrt, [-1., 2.]),
            (T::Asinh, [0., f64::INFINITY]),
            (T::Identity, [1., 1.]),
        ] {
            let scale = GgplotUnboundedScale::new(
                endpoints.map(Number),
                Bounds::new(100., 540.).unwrap(),
                Some(ScaleTransform::Ggplot {
                    transform: transform.clone(),
                }),
                OutsidePolicy::Extend,
            )
            .unwrap();
            assert!(!scale.has_numeric_inverse(), "{transform:?}");
            assert_eq!(
                scale.invert(320.).unwrap_err().code,
                DiagnosticCode::UnsupportedCapability
            );
        }
    }
    fn number(v: &serde_json::Value) -> f64 {
        v.as_f64().unwrap_or_else(|| match v.as_str() {
            Some("Infinity") => f64::INFINITY,
            Some("-Infinity") => f64::NEG_INFINITY,
            _ => f64::NAN,
        })
    }
    #[test]
    fn finite_view_projection_matches_reference_coordinate_stage() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/positional-unbounded-viewports.json"
        ))
        .unwrap();
        let mut cases = 0;
        let mut infinite_positions = 0;
        for case in fixture["cases"].as_array().unwrap() {
            let result = &case["result"];
            let Some(limits) = result["limits"].as_array() else {
                continue;
            };
            let limits = [Number(number(&limits[0])), Number(number(&limits[1]))];
            let range = result["range"].as_array().unwrap();
            let range = [number(&range[0]), number(&range[1])];
            if limits.iter().any(|n| n.0.is_nan())
                || limits.iter().all(|n| n.0.is_finite())
                || range.iter().any(|n| !n.is_finite())
            {
                continue;
            }
            let scale = GgplotUnboundedScale::new(
                limits,
                Bounds::new(100., 540.).unwrap(),
                None,
                OutsidePolicy::Extend,
            )
            .unwrap()
            .with_transformed_viewport(range.map(Number))
            .unwrap();
            assert_eq!(scale.limits(), limits);
            for (value, expected) in result["values"]
                .as_array()
                .unwrap()
                .iter()
                .zip(result["point_positions"].as_array().unwrap())
            {
                let value = number(value);
                let expected = number(expected);
                let actual = scale.map_transformed(value).unwrap();
                if expected.is_finite() {
                    let actual =
                        (actual.unwrap_or_else(|| panic!("omitted {value}: {case}")) - 100.) / 440.;
                    assert!(
                        (actual - expected).abs() <= 3e-12,
                        "{actual} != {expected}: {case}"
                    );
                    infinite_positions += usize::from(value.is_infinite());
                } else {
                    assert!(actual.is_none(), "unexpected {actual:?}: {case}");
                }
            }
            cases += 1;
        }
        assert!(cases > 500, "only {cases} reference panels exercised");
        assert!(
            infinite_positions > 100,
            "only {infinite_positions} infinite positions exercised"
        );
        println!("{cases} finite views, {infinite_positions} infinite projections");
    }
    #[test]
    fn reversed_and_constant_views_preserve_reference_edges() {
        let scale = GgplotUnboundedScale::new(
            [Number(f64::NEG_INFINITY), Number(f64::INFINITY)],
            Bounds::new(100., 540.).unwrap(),
            None,
            OutsidePolicy::Extend,
        )
        .unwrap();
        let reversed = scale
            .clone()
            .with_transformed_viewport([Number(10.), Number(0.)])
            .unwrap();
        assert_eq!(
            reversed.map_transformed(f64::NEG_INFINITY).unwrap(),
            Some(540.)
        );
        assert_eq!(reversed.map_transformed(f64::INFINITY).unwrap(), Some(100.));
        let constant = scale
            .with_transformed_viewport([Number(1.), Number(1.)])
            .unwrap();
        assert_eq!(constant.map_transformed(f64::INFINITY).unwrap(), Some(320.));
        assert_eq!(constant.map_transformed(f64::NAN).unwrap(), None);
    }
}
