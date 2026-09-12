//! One prepared categorical spacing kernel for explicit legacy and D3 policies.
use super::{Bounds, ScaleCompatibility, error};
use crate::{ChartResult, DiagnosticCode};
use std::collections::BTreeMap;

/// Complete D3 band configuration; chart axes supply their destination range separately.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BandSpec<K = String> {
    /// None uses the chart's stable trained catalog; standalone None means empty.
    pub domain: Option<Vec<K>>,
    /// Inner step fraction, capped at one as in D3; negative finite values are supported.
    pub padding_inner: f64,
    /// Outer step fractions, including defined finite negative padding.
    pub padding_outer: f64,
    /// Leftover-space alignment, clamped to `0..=1`.
    pub align: f64,
    /// Floor the step and round starts/widths to integer destination units.
    pub round: bool,
}
impl<K> Default for BandSpec<K> {
    fn default() -> Self {
        Self {
            domain: None,
            padding_inner: 0.,
            padding_outer: 0.,
            align: 0.5,
            round: false,
        }
    }
}
/// Complete D3 point configuration, with zero default padding and zero bandwidth.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PointSpec<K = String> {
    /// None uses the trained chart catalog or an empty standalone catalog.
    pub domain: Option<Vec<K>>,
    /// Outer step fractions.
    pub padding: f64,
    /// Leftover-space alignment, clamped to `0..=1`.
    pub align: f64,
    /// Integer spacing and starts.
    pub round: bool,
}
impl<K> Default for PointSpec<K> {
    fn default() -> Self {
        Self {
            domain: None,
            padding: 0.,
            align: 0.5,
            round: false,
        }
    }
}
impl<K: Clone> PointSpec<K> {
    pub(crate) fn band(&self) -> BandSpec<K> {
        BandSpec {
            domain: self.domain.clone(),
            padding_inner: 1.,
            padding_outer: self.padding,
            align: self.align,
            round: self.round,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum ReferenceSpacing {
    Finite(Bounds),
    Unbounded([crate::interpolate::Number; 2]),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Spacing {
    pub(crate) range: Bounds,
    pub(crate) count: usize,
    inner: f64,
    outer: f64,
    align: f64,
    pub(super) round: bool,
    compatibility: ScaleCompatibility,
    point: bool,
    denominator: f64,
    reference_view: Option<ReferenceSpacing>,
    start: f64,
    step: f64,
    bandwidth: f64,
}
impl Spacing {
    pub(crate) fn new<K>(
        n: usize,
        range: Bounds,
        spec: &BandSpec<K>,
        compatibility: ScaleCompatibility,
        point: bool,
    ) -> ChartResult<Self> {
        if ![spec.padding_inner, spec.padding_outer, spec.align]
            .iter()
            .all(|x| x.is_finite())
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Categorical spacing needs finite padding and alignment.",
            ));
        }
        let inner = spec.padding_inner.min(1.);
        let denominator = if compatibility == ScaleCompatibility::Legacy && point && n != 1 {
            n as f64 - 1. + 2. * spec.padding_outer
        } else {
            (n as f64 - inner + 2. * spec.padding_outer).max(1.)
        };
        let mut value = Self {
            range,
            count: n,
            inner,
            outer: spec.padding_outer,
            align: spec.align.clamp(0., 1.),
            round: spec.round,
            compatibility,
            point,
            denominator,
            reference_view: None,
            start: 0.,
            step: 0.,
            bandwidth: 0.,
        };
        if compatibility == ScaleCompatibility::D3 {
            let span = range.maximum() - range.minimum();
            value.step = span / denominator;
            if value.round {
                value.step = value.step.floor();
            }
            value.start = range.minimum() + (span - value.step * (n as f64 - inner)) * value.align;
            value.bandwidth = value.step * (1. - inner);
            if value.round {
                value.start = crate::interpolate::js_round(value.start);
                value.bandwidth = crate::interpolate::js_round(value.bandwidth);
            }
            if ![
                denominator,
                value.step,
                value.start,
                value.bandwidth,
                value.start + value.step * n.saturating_sub(1) as f64 + value.bandwidth,
            ]
            .iter()
            .all(|x| x.is_finite())
            {
                return Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Categorical spacing cannot preserve finite extents.",
                ));
            }
        } else if !denominator.is_finite() {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Categorical spacing denominator overflow.",
            ));
        }
        Ok(value)
    }
    pub(crate) fn with_reference_expansion(
        self,
        expansion: super::GgplotExpansion,
        limits: Option<&[crate::interpolate::Number]>,
        continuous: Option<[crate::interpolate::Number; 2]>,
    ) -> ChartResult<Self> {
        let view = expansion.discrete_position_viewport(self.count, continuous, limits)?;
        self.with_reference_viewport(view)
    }
    pub(crate) fn with_reference_viewport(
        mut self,
        view: [crate::interpolate::Number; 2],
    ) -> ChartResult<Self> {
        self.reference_view = Some(if view.iter().all(|v| v.0.is_finite()) {
            ReferenceSpacing::Finite(Bounds::new(view[0].0, view[1].0)?)
        } else {
            ReferenceSpacing::Unbounded(view)
        });
        Ok(self)
    }
    pub(crate) fn reference_viewport(&self) -> Option<[crate::interpolate::Number; 2]> {
        match self.reference_view {
            Some(ReferenceSpacing::Finite(view)) => Some([
                crate::interpolate::Number(view.start()),
                crate::interpolate::Number(view.end()),
            ]),
            Some(ReferenceSpacing::Unbounded(limits)) => Some(limits),
            None => None,
        }
    }
    pub(crate) fn reference_extent(&self, value: f64) -> ChartResult<Option<Bounds>> {
        let view = self.reference_viewport().ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "Materialized position palettes require reference spacing.",
            )
        })?;
        let half = if self.point {
            0.
        } else {
            (1. - self.inner) / 2.
        };
        let a = super::ggplot_unbounded::reference_position(view, self.range, value - half)?;
        let b = super::ggplot_unbounded::reference_position(view, self.range, value + half)?;
        a.zip(b).map(|(a, b)| Bounds::new(a, b)).transpose()
    }
    pub(crate) fn reference_minor(&self, value: f64) -> ChartResult<Option<f64>> {
        let limits = self.reference_viewport().ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "Numeric categorical minors require reference category spacing.",
            )
        })?;
        if !value.is_finite() || value < limits[0].0 || value > limits[1].0 {
            return Ok(None);
        }
        super::ggplot_unbounded::reference_position(limits, self.range, value)
    }
    fn reference_map(&self, view: Bounds, value: f64) -> ChartResult<f64> {
        super::ggplot_unbounded::reference_position(
            [
                crate::interpolate::Number(view.start()),
                crate::interpolate::Number(view.end()),
            ],
            self.range,
            value,
        )?
        .ok_or_else(|| {
            error(
                DiagnosticCode::PrecisionLoss,
                "Finite category reference coordinate is undefined.",
            )
        })
    }
    pub(crate) fn supports_lookup(&self) -> bool {
        !matches!(self.reference_view, Some(ReferenceSpacing::Unbounded(_)))
    }
    pub(crate) fn center(&self, i: usize) -> ChartResult<Option<f64>> {
        if let Some(ReferenceSpacing::Unbounded(limits)) = self.reference_view {
            return super::ggplot_unbounded::reference_position(limits, self.range, 1. + i as f64);
        }
        self.finite_center(i).map(Some)
    }
    pub(crate) fn extent(&self, i: usize) -> ChartResult<Option<Bounds>> {
        if let Some(ReferenceSpacing::Unbounded(limits)) = self.reference_view {
            let center = 1. + i as f64;
            let half = if self.point {
                0.
            } else {
                (1. - self.inner) / 2.
            };
            let a = super::ggplot_unbounded::reference_position(limits, self.range, center - half)?;
            let b = super::ggplot_unbounded::reference_position(limits, self.range, center + half)?;
            return a.zip(b).map(|(a, b)| Bounds::new(a, b)).transpose();
        }
        self.finite_extent(i).map(Some)
    }
    pub(crate) fn resize(&self, n: usize) -> ChartResult<Self> {
        Self::new(
            n,
            self.range,
            &BandSpec::<String> {
                domain: None,
                padding_inner: self.inner,
                padding_outer: self.outer,
                align: self.align,
                round: self.round,
            },
            self.compatibility,
            self.point,
        )
    }
    fn finite_extent(&self, i: usize) -> ChartResult<Bounds> {
        if let Some(ReferenceSpacing::Finite(view)) = self.reference_view {
            let center = 1. + i as f64;
            let half = (1. - self.inner) / 2.;
            return Bounds::new(
                self.reference_map(view, center - half)?,
                self.reference_map(view, center + half)?,
            );
        }
        if self.compatibility == ScaleCompatibility::D3 {
            let index = if self.range.end() < self.range.start() {
                self.count - 1 - i
            } else {
                i
            };
            let start = self.start + self.step * index as f64;
            if self.range.end() < self.range.start() {
                Bounds::new(start + self.bandwidth, start)
            } else {
                Bounds::new(start, start + self.bandwidth)
            }
        } else {
            let a = if self.point && self.count == 1 {
                0.5
            } else {
                (self.outer + i as f64) / self.denominator
            };
            let b = if self.point {
                a
            } else {
                (self.outer + i as f64 + 1. - self.inner) / self.denominator
            };
            Bounds::new(
                super::linear::interpolate(self.range, a)?,
                super::linear::interpolate(self.range, b)?,
            )
        }
    }
    fn finite_center(&self, i: usize) -> ChartResult<f64> {
        if let Some(ReferenceSpacing::Finite(view)) = self.reference_view {
            return self.reference_map(view, 1. + i as f64);
        }
        if self.compatibility == ScaleCompatibility::Legacy {
            let t = if self.point && self.count == 1 {
                0.5
            } else {
                (self.outer + i as f64 + (1. - self.inner) * 0.5) / self.denominator
            };
            super::linear::interpolate(self.range, t)
        } else {
            let e = self.finite_extent(i)?;
            Ok(e.minimum() + self.bandwidth * 0.5)
        }
    }
    fn validate_position(&self, p: f64) -> ChartResult<bool> {
        if !p.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Category lookup needs a finite position.",
            ));
        }
        Ok(self.count > 0 && self.range.contains(p))
    }
    pub(crate) fn band_at(&self, p: f64) -> ChartResult<Option<usize>> {
        if !self.validate_position(p)? || !self.supports_lookup() {
            return Ok(None);
        }
        if let Some(ReferenceSpacing::Finite(view)) = self.reference_view {
            if view.start() == view.end() {
                return Ok((p == self.finite_center(0)?).then_some(0));
            }
            let value = super::linear::interpolate(view, super::linear::fraction(self.range, p)?)?;
            let slot = value - 1. + (1. - self.inner) / 2.;
            if slot < 0. {
                return Ok(None);
            }
            let i = (slot.floor() as usize).min(self.count - 1);
            return Ok(self.finite_extent(i)?.contains(p).then_some(i));
        }
        if self.compatibility == ScaleCompatibility::Legacy {
            let slot = super::linear::fraction(self.range, p)? * self.denominator - self.outer;
            if slot < 0. {
                return Ok(None);
            }
            let i = slot.floor() as usize;
            if i >= self.count {
                return Ok(
                    (p == self.finite_extent(self.count - 1)?.end()).then_some(self.count - 1)
                );
            }
            return Ok((slot - i as f64 <= 1. - self.inner).then_some(i));
        }
        if self.step == 0. {
            return Ok(self.finite_extent(0)?.contains(p).then_some(0));
        }
        let slot = ((p - self.start) / self.step).floor();
        if slot < 0. {
            return Ok(None);
        }
        let physical = (slot as usize).min(self.count - 1);
        let i = if self.range.end() < self.range.start() {
            self.count - 1 - physical
        } else {
            physical
        };
        Ok(self.finite_extent(i)?.contains(p).then_some(i))
    }
    pub(crate) fn point_at(&self, p: f64) -> ChartResult<Option<usize>> {
        if !self.validate_position(p)? || !self.supports_lookup() {
            return Ok(None);
        }
        if self.finite_center(0)? == self.finite_center(self.count - 1)? {
            return Ok(Some(0));
        }
        // Binary search the prepared centers; use the original normalized distance for
        // legacy tie behavior, avoiding a per-category scan or repeated string lookup.
        let reverse = self.range.end() < self.range.start();
        let mut lo = 0;
        let mut hi = self.count;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let center = self.finite_center(mid)?;
            if if reverse { center > p } else { center < p } {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        let distance = |i| -> ChartResult<f64> {
            let c = self.finite_center(i)?;
            Ok(if self.compatibility == ScaleCompatibility::Legacy {
                (super::linear::fraction(self.range, c)? - super::linear::fraction(self.range, p)?)
                    .abs()
            } else {
                (c - p).abs()
            })
        };
        let selected = if lo == 0 {
            0
        } else if lo == self.count || distance(lo - 1)? <= distance(lo)? {
            lo - 1
        } else {
            lo
        };
        // Large destination origins can collapse adjacent starts even with nonzero
        // step. Retain the earliest category for all equal-distance candidates.
        let best = distance(selected)?;
        let mut lo = 0;
        let mut hi = selected;
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if distance(mid)? > best {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        Ok(Some(lo))
    }

    pub(crate) fn step(&self) -> f64 {
        if !self.supports_lookup() {
            return 0.;
        }
        if let Some(ReferenceSpacing::Finite(view)) = self.reference_view {
            return if view.start() == view.end() {
                0.
            } else {
                ((self.range.end() - self.range.start()) / (view.end() - view.start())).abs()
            };
        }
        if self.compatibility == ScaleCompatibility::D3 {
            self.step
        } else {
            (self.range.end() - self.range.start()).abs() / self.denominator
        }
    }
    pub(crate) fn bandwidth(&self) -> f64 {
        if self.compatibility == ScaleCompatibility::D3 {
            self.bandwidth
        } else {
            self.step() * (1. - self.inner)
        }
    }
}
/// Generic immutable D3 band/point mapping, retaining exact key identity.
#[derive(Clone, Debug, PartialEq)]
pub struct CategoryScale<K: Ord> {
    spec: BandSpec<K>,
    domain: Vec<K>,
    index: BTreeMap<K, usize>,
    spacing: Spacing,
}
impl<K: Ord + Clone> CategoryScale<K> {
    /// Resolve D3 bands, deduplicating the domain in stable order.
    pub fn band(spec: BandSpec<K>, range: Bounds) -> ChartResult<Self> {
        let mut index = BTreeMap::new();
        let mut domain = Vec::new();
        for key in spec.domain.iter().flatten() {
            if !index.contains_key(key) {
                index.insert(key.clone(), domain.len());
                domain.push(key.clone());
            }
        }
        let spacing = Spacing::new(domain.len(), range, &spec, ScaleCompatibility::D3, false)?;
        let spec = BandSpec {
            domain: Some(domain.clone()),
            padding_inner: spec.padding_inner.min(1.),
            align: spec.align.clamp(0., 1.),
            ..spec
        };
        Ok(Self {
            spec,
            domain,
            index,
            spacing,
        })
    }
    /// Resolve D3 points through the same spacing kernel with inner padding one.
    pub fn point(spec: PointSpec<K>, range: Bounds) -> ChartResult<Self> {
        Self::band(spec.band(), range)
    }
    /// Normalized configuration (points use inner padding one).
    pub fn spec(&self) -> &BandSpec<K> {
        &self.spec
    }
    /// Exact stable category order.
    pub fn domain(&self) -> &[K] {
        &self.domain
    }
    /// Authored destination endpoints.
    pub fn range(&self) -> Bounds {
        self.spacing.range
    }
    /// D3's lower band start, even when the range is reversed.
    pub fn map(&self, key: &K) -> ChartResult<Option<f64>> {
        self.extent(key).map(|e| e.map(Bounds::minimum))
    }
    /// Center of a known band or point.
    pub fn center(&self, key: &K) -> ChartResult<Option<f64>> {
        self.index
            .get(key)
            .map(|&i| self.spacing.center(i))
            .transpose()
            .map(Option::flatten)
    }
    /// Edges oriented with the authored range.
    pub fn extent(&self, key: &K) -> ChartResult<Option<Bounds>> {
        self.index
            .get(key)
            .map(|&i| self.spacing.extent(i))
            .transpose()
            .map(Option::flatten)
    }
    /// Nonnegative interval between starts.
    pub fn step(&self) -> f64 {
        self.spacing.step()
    }
    /// Nonnegative band width (zero for points).
    pub fn bandwidth(&self) -> f64 {
        self.spacing.bandwidth()
    }
}

/// Numeric positions trained by observed categories, independent of retained levels.
pub(crate) fn observed_extent(
    indices: impl Iterator<Item = usize>,
    has_observations: bool,
) -> Option<[crate::interpolate::Number; 2]> {
    use crate::interpolate::Number;
    let mut indices = indices;
    let Some(first) = indices.next() else {
        return has_observations.then_some([Number(f64::INFINITY), Number(f64::NEG_INFINITY)]);
    };
    let (low, high) = indices.fold((first, first), |(low, high), i| (low.min(i), high.max(i)));
    Some([Number(1. + low as f64), Number(1. + high as f64)])
}
