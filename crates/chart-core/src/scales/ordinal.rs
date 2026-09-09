//! Typed, immutable category catalogs. D3 ordinal semantics; see scales/LICENSE.
use crate::interpolate::Number;
use std::{cmp::Ordering, collections::BTreeMap};

/// Portable category identity, with exact integers and no implicit string conversion.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScaleKey {
    /// Explicit null category (distinct from an absent input).
    Null,
    /// Boolean identity.
    Boolean(bool),
    /// IEEE number, using SameValueZero equality (NaNs and signed zeros coalesce).
    Number(Number),
    /// Exact signed integer, distinct from a floating-point number.
    Integer(#[serde(with = "crate::portable::signed")] i64),
    /// Exact unsigned integer.
    Unsigned(#[serde(with = "crate::portable::unsigned")] u64),
    /// Exact timestamp in the caller's declared common unit.
    Timestamp(#[serde(with = "crate::portable::signed")] i64),
    /// Text identity.
    Text(String),
}
impl ScaleKey {
    fn tag(&self) -> u8 {
        match self {
            Self::Null => 0,
            Self::Boolean(_) => 1,
            Self::Number(_) => 2,
            Self::Integer(_) => 3,
            Self::Unsigned(_) => 4,
            Self::Timestamp(_) => 5,
            Self::Text(_) => 6,
        }
    }
}
impl Ord for ScaleKey {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Boolean(a), Self::Boolean(b)) => a.cmp(b),
            (Self::Number(Number(a)), Self::Number(Number(b))) => {
                let normalize = |v: f64| {
                    if v.is_nan() {
                        f64::NAN
                    } else if v == 0. {
                        0.
                    } else {
                        v
                    }
                };
                normalize(*a).total_cmp(&normalize(*b))
            }
            (Self::Integer(a), Self::Integer(b)) | (Self::Timestamp(a), Self::Timestamp(b)) => {
                a.cmp(b)
            }
            (Self::Unsigned(a), Self::Unsigned(b)) => a.cmp(b),
            (Self::Text(a), Self::Text(b)) => a.cmp(b),
            _ => self.tag().cmp(&other.tag()),
        }
    }
}
impl PartialOrd for ScaleKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for ScaleKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl Eq for ScaleKey {}

/// Authoring-time unknown-category policy. Resolved lookup never changes a domain.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum OrdinalUnknown<V> {
    /// Explicit training may append previously unseen categories in first-seen order.
    Implicit,
    /// Missing categories yield this output; None denotes undefined.
    Explicit(Option<V>),
}
/// Generic ordinal configuration; ranges may hold any native output type.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrdinalSpec<K, V> {
    /// Authored category order; duplicate keys retain their first occurrence.
    pub domain: Vec<K>,
    /// Outputs cycled by category index; an empty range always yields undefined.
    pub range: Vec<V>,
    /// Explicit output or authoring-time implicit-domain behavior.
    pub unknown: OrdinalUnknown<V>,
}
impl<K, V> Default for OrdinalSpec<K, V> {
    fn default() -> Self {
        Self {
            domain: vec![],
            range: vec![],
            unknown: OrdinalUnknown::Implicit,
        }
    }
}
/// Frozen ordinal mapping with logarithmic lookup and independent reconfiguration.
#[derive(Clone, Debug, PartialEq)]
pub struct OrdinalScale<K: Ord, V> {
    spec: OrdinalSpec<K, V>,
    index: BTreeMap<K, usize>,
}
impl<K: Ord + Clone, V: Clone> OrdinalScale<K, V> {
    /// Deduplicate a caller's domain in stable first-seen order.
    pub fn new(mut spec: OrdinalSpec<K, V>) -> Self {
        let mut index = BTreeMap::new();
        spec.domain.retain(|k| {
            let i = index.len();
            if index.contains_key(k) {
                false
            } else {
                index.insert(k.clone(), i);
                true
            }
        });
        Self { spec, index }
    }
    /// Exact resolved configuration, including category order and unknown policy.
    pub fn spec(&self) -> &OrdinalSpec<K, V> {
        &self.spec
    }
    /// Authoring/training operation; explicit unknown policies do not add categories.
    pub fn train(&self, keys: impl IntoIterator<Item = K>) -> Self {
        let mut next = self.clone();
        if matches!(next.spec.unknown, OrdinalUnknown::Implicit) {
            for key in keys {
                if !next.index.contains_key(&key) {
                    next.index.insert(key.clone(), next.spec.domain.len());
                    next.spec.domain.push(key);
                }
            }
        }
        next
    }
    /// Pure lookup. Untrained implicit keys are undefined until explicitly trained.
    pub fn map(&self, key: &K) -> Option<&V> {
        if let Some(&i) = self.index.get(key) {
            return if self.spec.range.is_empty() {
                None
            } else {
                Some(&self.spec.range[i % self.spec.range.len()])
            };
        }
        match &self.spec.unknown {
            OrdinalUnknown::Explicit(value) => value.as_ref(),
            OrdinalUnknown::Implicit => None,
        }
    }
    /// Replace configuration without changing this prepared mapping.
    pub fn reconfigure(&self, spec: OrdinalSpec<K, V>) -> Self {
        Self::new(spec)
    }
}
