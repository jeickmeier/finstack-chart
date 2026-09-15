//! Identity aesthetic policy: transformation precedes raw output, while the
//! eligible population and limits describe guides only.
use super::*;
use crate::interpolate::{Number, Value};

/// Numeric ggplot2 identity mapping for size, alpha, linewidth and shape codes.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotNumericIdentity {
    /// Optional input transformation, shared with positional scale arithmetic.
    pub transform: Option<ScaleTransform>,
    /// Raw limits for guides. They never censor identity output.
    pub limits: Option<[Option<Number>; 2]>,
    /// Train guide metadata. Reference identity scales default to no guide.
    pub guide: bool,
    /// Whether guide training observed any rows, including missing/nonfinite rows.
    #[serde(default)]
    pub has_population: bool,
    /// Finite transformed extent retained after eligible population training.
    pub trained: Option<[Number; 2]>,
}
impl GgplotNumericIdentity {
    pub(super) fn validate(&self) -> ChartResult<()> {
        if let Some(transform) = &self.transform {
            transform.validate()?;
        }
        if self.trained.iter().flatten().any(|v| !v.0.is_finite()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Identity trained extent must be finite.",
            ));
        }
        Ok(())
    }
    fn transform(&self, value: f64) -> f64 {
        self.transform
            .as_ref()
            .map_or(value, |t| t.forward_raw(value))
    }
    pub(super) fn transform_values(&self, values: &[f64]) -> ChartResult<Vec<f64>> {
        match &self.transform {
            Some(ScaleTransform::Ggplot { transform }) => transform.forward_population(values),
            _ => Ok(values.iter().map(|v| self.transform(*v)).collect()),
        }
    }
    pub(super) fn train(&mut self, values: &[Option<Number>]) -> ChartResult<()> {
        self.train_batches(values, &[values.len()])
    }
    pub(super) fn train_batches(
        &mut self,
        values: &[Option<Number>],
        batches: &[usize],
    ) -> ChartResult<()> {
        self.validate()?;
        self.trained = None;
        self.has_population = self.guide && !values.is_empty();
        let mut offset = 0;
        for count in batches {
            let end = offset + count;
            let raw = values[offset..end]
                .iter()
                .map(|v| v.map_or(f64::NAN, |v| v.0))
                .collect::<Vec<_>>();
            if let Some(ScaleTransform::Ggplot { transform }) = &self.transform {
                transform.validate_population(raw.iter().copied())?;
            }
            if self.guide {
                for value in self
                    .transform_values(&raw)?
                    .into_iter()
                    .filter(|v| v.is_finite())
                {
                    let extent = self.trained.get_or_insert([Number(value); 2]);
                    extent[0].0 = extent[0].0.min(value);
                    extent[1].0 = extent[1].0.max(value);
                }
            }
            offset = end;
        }
        debug_assert_eq!(offset, values.len());
        Ok(())
    }
    /// Guide extent in transformed coordinates; independent from raw mapping.
    pub fn domain(&self) -> ChartResult<[Number; 2]> {
        self.validate()?;
        let mut domain =
            if self.guide { self.trained } else { None }.unwrap_or([Number(0.), Number(1.)]);
        if let Some(limits) = self.limits {
            let transformed =
                self.transform_values(&limits.map(|v| v.map_or(f64::NAN, |v| v.0)))?;
            let mut limits = [transformed[0], transformed[1]].map(|v| (!v.is_nan()).then_some(v));
            if let [Some(a), Some(b)] = limits {
                limits = [Some(a.min(b)), Some(a.max(b))];
            }
            if (!self.guide || self.trained.is_none())
                && !limits.iter().all(|v| v.is_some_and(f64::is_finite))
            {
                return Ok([Number(0.), Number(1.)]);
            }
            for (i, limit) in limits.into_iter().enumerate() {
                if let Some(value) = limit {
                    domain[i] = Number(value);
                }
            }
        }
        Ok(domain)
    }
    /// Preserve missing input and IEEE numbers without rescaling or OOB censoring.
    pub fn map(&self, input: Option<f64>) -> ChartResult<Value> {
        self.validate()?;
        Ok(input.map_or(Value::Missing, |v| Value::Number(Number(self.transform(v)))))
    }
}

/// Raw discrete identity mapping for reference colour, fill and linetype values.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotDiscreteIdentity {
    /// Explicit guide domain; values outside it still map to themselves.
    pub limits: Option<Vec<ScaleKey>>,
    /// Ordered factor levels for guide training.
    pub levels: Option<Vec<ScaleKey>>,
    /// Drop unobserved factor levels from guides.
    pub drop: bool,
    /// Include an observed missing guide level.
    pub na_translate: bool,
    /// Train and emit guide metadata; false by default.
    pub guide: bool,
    /// Complete observed keys, retained for pooled colour preparation.
    pub observed: Vec<ScaleKey>,
}
impl Default for GgplotDiscreteIdentity {
    fn default() -> Self {
        Self {
            limits: None,
            levels: None,
            drop: true,
            na_translate: true,
            guide: false,
            observed: vec![],
        }
    }
}
impl GgplotDiscreteIdentity {
    pub(super) fn validate(&self) -> ChartResult<()> {
        if self.guide
            && self.observed.iter().any(|key| {
                !matches!(
                    key,
                    ScaleKey::Text(_) | ScaleKey::Boolean(_) | ScaleKey::Null
                )
            })
        {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Continuous values cannot train a discrete identity guide.",
            ));
        }
        for keys in [
            &self.observed[..],
            self.limits.as_deref().unwrap_or_default(),
            self.levels.as_deref().unwrap_or_default(),
        ] {
            if keys.len() > crate::interpolate::MAX_VALUES {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Identity keys exceed the scale value budget.",
                ));
            }
            for key in keys {
                Self::map(Some(key))?.validate()?;
            }
        }
        Ok(())
    }
    pub(super) fn train(&mut self, keys: &[ScaleKey]) -> ChartResult<()> {
        self.observed = keys
            .iter()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        self.validate()
    }
    /// Guide domain; an untrained reference scale exposes the conventional `[0,1]`.
    pub fn domain(&self) -> ChartResult<Vec<ScaleKey>> {
        self.validate()?;
        if !self.guide {
            return Ok(self.limits.clone().unwrap_or_else(|| {
                vec![ScaleKey::Number(Number(0.)), ScaleKey::Number(Number(1.))]
            }));
        }
        Ok(super::ggplot::discrete_domain(
            &self.observed,
            self.limits.as_deref(),
            self.levels.as_deref(),
            self.drop,
            self.na_translate,
        ))
    }

    /// Return a raw typed aesthetic value, including an unseen category.
    pub fn map(key: Option<&ScaleKey>) -> ChartResult<Value> {
        Ok(match key {
            None | Some(ScaleKey::Null) => Value::Missing,
            Some(ScaleKey::Text(v)) => Value::Text(v.clone()),
            Some(ScaleKey::Number(v)) => Value::Number(*v),
            Some(ScaleKey::Boolean(v)) => Value::Boolean(*v),
            Some(ScaleKey::Integer(v)) if v.unsigned_abs() <= 9_007_199_254_740_992 => {
                Value::Number(Number(*v as f64))
            }
            Some(ScaleKey::Unsigned(v)) if *v <= 9_007_199_254_740_992 => {
                Value::Number(Number(*v as f64))
            }
            _ => {
                return Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Identity aesthetic values require an exactly representable number or text.",
                ));
            }
        })
    }
}
