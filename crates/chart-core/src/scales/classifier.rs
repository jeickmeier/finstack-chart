//! Prepared classifiers. Reference arithmetic follows d3-scale 4.0.2 / d3-array 3.2.4.
use super::error;
use crate::{
    ChartResult, DiagnosticCode,
    interpolate::{MAX_VALUES, Number},
};
use serde::{Deserialize, Serialize};

/// Inverse interval, retaining absent output membership separately from unbounded ends.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleExtent<K> {
    /// Whether the queried output occurs in the range (first occurrence wins).
    pub found: bool,
    /// Inclusive lower bound, or an undefined/unbounded endpoint.
    pub lower: Option<K>,
    /// Exclusive upper bound, or an undefined/unbounded endpoint.
    pub upper: Option<K>,
}

/// Authored ordered cutpoints and arbitrary owned output values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThresholdSpec<K, V> {
    /// Ordered cuts. Equality maps into the following interval.
    pub domain: Vec<K>,
    /// Range values; extra cuts are retained for inverse extent queries.
    pub range: Vec<V>,
    /// Output for absent or non-comparable inputs.
    pub unknown: Option<V>,
}
/// Frozen threshold mapping; input keys need not be numeric or stringified.
#[derive(Clone, Debug, PartialEq)]
pub struct ThresholdScale<K, V> {
    spec: ThresholdSpec<K, V>,
}
impl<K: PartialOrd, V> ThresholdScale<K, V> {
    /// Retain authored order, including the reference's behavior for unsorted cutpoints.
    pub fn new(spec: ThresholdSpec<K, V>) -> ChartResult<Self> {
        budget(spec.domain.len(), spec.range.len())?;
        Ok(Self { spec })
    }
    /// Immutable authored configuration.
    pub fn spec(&self) -> &ThresholdSpec<K, V> {
        &self.spec
    }
    /// Pure lookup with no catalog mutation or per-input allocation.
    pub fn map(&self, input: Option<&K>) -> Option<&V> {
        self.map_by(input.filter(|x| x.partial_cmp(x).is_some()), |d, x| d <= x)
    }
    pub(super) fn map_by(&self, input: Option<&K>, before: impl Fn(&K, &K) -> bool) -> Option<&V> {
        let Some(x) = input else {
            return self.spec.unknown.as_ref();
        };
        let n = self
            .spec
            .domain
            .len()
            .min(self.spec.range.len().saturating_sub(1));
        self.spec
            .range
            .get(bisect_right(&self.spec.domain[..n], |d| before(d, x), 0))
    }
}
impl<K: PartialOrd + Clone, V: PartialEq> ThresholdScale<K, V> {
    /// Inverse extent of the first equal range value; tails have undefined endpoints.
    pub fn invert_extent(&self, value: &V) -> ScaleExtent<K> {
        let index = self.spec.range.iter().position(|v| v == value);
        ScaleExtent {
            found: index.is_some(),
            lower: index
                .and_then(|i| i.checked_sub(1))
                .and_then(|i| self.spec.domain.get(i))
                .cloned(),
            upper: index.and_then(|i| self.spec.domain.get(i)).cloned(),
        }
    }
}
impl<V> ThresholdScale<super::ScaleKey, V> {
    /// Typed lookup with IEEE comparisons for numeric cuts and missing NaN inputs.
    pub fn map_key(&self, key: Option<&super::ScaleKey>) -> Option<&V> {
        use super::ScaleKey;
        self.map_by(
            key.filter(|k| !matches!(k, ScaleKey::Number(Number(x)) if x.is_nan())),
            |d, x| match (d, x) {
                (ScaleKey::Number(Number(d)), ScaleKey::Number(Number(x))) => d <= x,
                _ => d <= x,
            },
        )
    }
}

/// Sample-trained quantiles or equal-width quantization of two authored endpoints.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ClassifierDomain {
    /// Missing/NaN samples are removed; infinities are retained as in the reference.
    Quantile(Vec<Option<Number>>),
    /// Endpoint direction and degenerate values retain reference arithmetic.
    Quantize([Number; 2]),
}
/// Portable classifier configuration with generic range outputs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassifierSpec<V> {
    /// Population or endpoints used to prepare breakpoints.
    pub domain: ClassifierDomain,
    /// Ordered outputs; duplicate values are legal.
    pub range: Vec<V>,
    /// Missing/NaN input output, independent of empty-range behavior.
    pub unknown: Option<V>,
}
impl ClassifierSpec<crate::interpolate::Value> {
    /// Reference quantile defaults: empty population/range and undefined unknown.
    pub fn quantile() -> Self {
        Self {
            domain: ClassifierDomain::Quantile(vec![]),
            range: vec![],
            unknown: None,
        }
    }
    /// Reference quantize defaults: unit input domain and numeric zero/one outputs.
    pub fn quantize() -> Self {
        Self {
            domain: ClassifierDomain::Quantize([Number(0.), Number(1.)]),
            range: vec![
                crate::interpolate::Value::number(0.),
                crate::interpolate::Value::number(1.),
            ],
            unknown: None,
        }
    }
}
impl ThresholdSpec<super::ScaleKey, crate::interpolate::Value> {
    /// Reference threshold defaults: a half-unit cut and numeric zero/one outputs.
    pub fn d3() -> Self {
        Self {
            domain: vec![super::ScaleKey::Number(Number(0.5))],
            range: vec![
                crate::interpolate::Value::number(0.),
                crate::interpolate::Value::number(1.),
            ],
            unknown: None,
        }
    }
}
/// Prepared quantile/quantize mapping sharing one breakpoint search.
#[derive(Clone, Debug, PartialEq)]
pub struct ClassifierScale<V> {
    spec: ClassifierSpec<V>,
    domain: Vec<Number>,
    cuts: Vec<Option<Number>>,
}
impl<V: Clone> ClassifierScale<V> {
    /// Rebuild quantize thresholds from niced endpoints, retaining the source and all output values.
    pub fn nice(&self, count: f64) -> ChartResult<Self> {
        let domain = self.tick_domain()?;
        let (a, b) = super::ticks::nice(domain[0].0, domain[1].0, count);
        let mut spec = self.spec.clone();
        spec.domain = ClassifierDomain::Quantize([Number(a), Number(b)]);
        Self::new(spec)
    }
}
impl<V> ClassifierScale<V> {
    /// Sort/train once. Rebuild from the complete eligible population after corrections.
    pub fn new(spec: ClassifierSpec<V>) -> ChartResult<Self> {
        let count = match &spec.domain {
            ClassifierDomain::Quantile(v) => v.len(),
            ClassifierDomain::Quantize(_) => 2,
        };
        budget(count, spec.range.len())?;
        let (domain, cuts) = match &spec.domain {
            ClassifierDomain::Quantile(samples) => {
                let domain = sorted_samples(samples);
                let n = spec.range.len().max(1);
                let cuts = (1..n)
                    .map(|i| quantile_sorted(&domain, i as f64 / n as f64))
                    .collect();
                (domain, cuts)
            }
            ClassifierDomain::Quantize([a, b]) => {
                if spec.range.is_empty() {
                    return Err(error(
                        DiagnosticCode::NumericalDomain,
                        "Quantize requires a nonempty range (the reference throws RangeError).",
                    ));
                }
                let n = spec.range.len() - 1;
                let cuts = (0..n)
                    .map(|i| {
                        Some(Number(
                            ((i + 1) as f64 * b.0 - (i as f64 - n as f64) * a.0) / (n + 1) as f64,
                        ))
                    })
                    .collect();
                (vec![*a, *b], cuts)
            }
        };
        Ok(Self { spec, domain, cuts })
    }
    /// Complete authored configuration, including original population order.
    pub fn spec(&self) -> &ClassifierSpec<V> {
        &self.spec
    }
    /// Prepared sorted sample population, or two quantize endpoints.
    pub fn domain(&self) -> &[Number] {
        &self.domain
    }
    fn tick_domain(&self) -> ChartResult<&[Number]> {
        if matches!(self.spec.domain, ClassifierDomain::Quantize(_)) {
            Ok(&self.domain)
        } else {
            Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Quantile classifiers expose thresholds rather than continuous ticks or nice.",
            ))
        }
    }
    /// Quantize's unthinned data-space ticks, independent of its output classes.
    pub fn ticks(&self, count: f64, budget: usize) -> ChartResult<Vec<f64>> {
        super::NumericFamily::Linear.ticks(self.tick_domain()?, count, budget)
    }
    /// Quantize's independently inferred tick labels.
    pub fn tick_format(
        &self,
        count: f64,
        specifier: Option<&str>,
        locale: crate::typography::NumericLocale,
    ) -> ChartResult<crate::typography::NumericFormatter> {
        super::NumericFamily::Linear.tick_format(self.tick_domain()?, count, specifier, locale)
    }
    /// Actual cuts; `None` represents the reference's undefined empty-sample quantile.
    pub fn thresholds(&self) -> &[Option<Number>] {
        &self.cuts
    }
    /// Missing/NaN inputs select unknown; equality belongs to the upper bucket.
    pub fn map(&self, input: Option<f64>) -> Option<&V> {
        let Some(x) = input.filter(|x| !x.is_nan()) else {
            return self.spec.unknown.as_ref();
        };
        self.spec
            .range
            .get(bisect_right(&self.cuts, |d| d.is_some_and(|d| d.0 <= x), 0))
    }
}
impl<V: PartialEq> ClassifierScale<V> {
    /// First matching range value's inverse interval. Absent values return tagged NaN ends.
    pub fn invert_extent(&self, value: &V) -> ScaleExtent<Number> {
        let Some(i) = self.spec.range.iter().position(|v| v == value) else {
            return ScaleExtent {
                found: false,
                lower: Some(Number(f64::NAN)),
                upper: Some(Number(f64::NAN)),
            };
        };
        let lower = if i == 0 {
            self.domain.first().copied()
        } else {
            self.cuts.get(i - 1).copied().flatten()
        };
        // Quantize's singleton range preserves its undefined upper cut (not x1).
        let upper = if matches!(self.spec.domain, ClassifierDomain::Quantize(_)) && i == 0 {
            self.cuts.first().copied().flatten()
        } else if i < self.cuts.len() {
            self.cuts[i]
        } else {
            self.domain.last().copied()
        };
        ScaleExtent {
            found: true,
            lower,
            upper,
        }
    }
}

pub(super) fn sorted_samples(samples: &[Option<Number>]) -> Vec<Number> {
    let mut domain: Vec<_> = samples
        .iter()
        .flatten()
        .copied()
        .filter(|v| !v.0.is_nan())
        .collect();
    domain.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("NaNs removed"));
    domain
}
pub(super) fn quantile_sorted(domain: &[Number], p: f64) -> Option<Number> {
    if domain.is_empty() || p.is_nan() {
        return None;
    }
    if p <= 0. || domain.len() < 2 {
        return domain.first().copied();
    }
    if p >= 1. {
        return domain.last().copied();
    }
    let i = (domain.len() - 1) as f64 * p;
    let j = i.floor() as usize;
    let a = domain[j].0;
    Some(Number(a + (domain[j + 1].0 - a) * (i - j as f64)))
}
// Explicit binary search also preserves reference behavior for undefined/unsorted cuts.
pub(super) fn bisect_right<T>(values: &[T], before: impl Fn(&T) -> bool, mut lo: usize) -> usize {
    let mut hi = values.len();
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if before(&values[mid]) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}
fn budget(domain: usize, range: usize) -> ChartResult<()> {
    if domain.saturating_add(range) > MAX_VALUES {
        Err(error(
            DiagnosticCode::ResourceLimit,
            "Classifier exceeds its aggregate value budget.",
        ))
    } else {
        Ok(())
    }
}
