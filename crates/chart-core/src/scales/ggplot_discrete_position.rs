//! Typed reference positional categories. Domain and guide rules share aesthetic owners.
use super::{
    Bounds, GgplotDiscreteGuide, GgplotDiscreteGuideEntry, GgplotExpansion, ScaleKey, error,
};
use crate::{ChartResult, DiagnosticCode, interpolate::Number};
use std::collections::BTreeMap;

/// Character/factor positional training with explicit missing-category identity.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GgplotDiscretePosition {
    /// Authored limits override trained order, including explicit null positions.
    pub limits: Option<Vec<ScaleKey>>,
    /// Materialized numeric palette in trained-domain order; absent uses one-based indices.
    /// Like a fixed-vector reference palette, it must cover the complete resolved domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette: Option<Vec<Number>>,
    /// Factor levels; absent uses reference character ordering.
    pub levels: Option<Vec<ScaleKey>>,
    /// Remove unobserved factor levels.
    pub drop: bool,
    /// Retain a missing level in the automatic domain.
    pub na_translate: bool,
    /// Independent candidate selection and labels.
    pub guide: GgplotDiscreteGuide,
    /// Expansion in one-based category units.
    pub expansion: GgplotExpansion,
}
impl Default for GgplotDiscretePosition {
    fn default() -> Self {
        Self {
            limits: None,
            palette: None,
            levels: None,
            drop: true,
            na_translate: true,
            guide: GgplotDiscreteGuide::default(),
            expansion: GgplotExpansion::discrete_default(),
        }
    }
}
impl GgplotDiscretePosition {
    /// Train one eligible population without converting null to a display string.
    pub fn train(&self, values: &[ScaleKey]) -> ChartResult<PreparedGgplotDiscretePosition> {
        self.train_with_continuous_limits(values, None)
    }

    /// Train with authored continuous category-index limits using shared expansion rules.
    pub fn train_with_continuous_limits(
        &self,
        values: &[ScaleKey],
        continuous_limits: Option<&[Number]>,
    ) -> ChartResult<PreparedGgplotDiscretePosition> {
        if let Some(limits) = continuous_limits {
            crate::limits::require_within(
                limits.len() <= crate::interpolate::MAX_VALUES,
                "continuous category limits",
            )?;
        }
        for keys in [
            Some(values),
            self.limits.as_deref(),
            self.levels.as_deref(),
            self.guide.breaks.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            crate::limits::require_within(
                keys.len() <= crate::interpolate::MAX_VALUES,
                "positional category",
            )?;
            for key in keys {
                match key {
                    ScaleKey::Null => {}
                    ScaleKey::Text(value) => crate::limits::require_within(
                        value.len() <= 4096,
                        "positional category label",
                    )?,
                    _ => {
                        return Err(error(
                            DiagnosticCode::SchemaConflict,
                            "Discrete position keys must be text or null.",
                        ));
                    }
                }
            }
        }
        self.guide.validate()?;
        if values.is_empty() && self.limits.as_ref().is_some_and(Vec::is_empty) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Reference discrete positions reject empty limits on a zero-row population.",
            ));
        }
        let domain = super::ggplot::discrete_domain(
            values,
            self.limits.as_deref(),
            self.levels.as_deref(),
            self.drop,
            self.na_translate,
        );
        let index: BTreeMap<_, _> = domain
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, key)| (key, i))
            .collect();
        if let Some(palette) = &self.palette {
            crate::limits::require_within(
                palette.len() <= crate::interpolate::MAX_VALUES,
                "positional palette",
            )?;
            if palette.len() < domain.len() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "The positional palette must cover the complete category domain.",
                ));
            }
        }
        let numeric = |i: usize| self.palette.as_ref().map_or(1. + i as f64, |p| p[i].0);
        let mut observed = super::spacing::observed_extent(
            values.iter().filter_map(|key| index.get(key).copied()),
            !values.is_empty(),
        );
        if self.palette.is_some() && !values.is_empty() {
            let extent = values
                .iter()
                .filter_map(|key| index.get(key).map(|i| numeric(*i)))
                .filter(|v| v.is_finite())
                .fold([f64::INFINITY, f64::NEG_INFINITY], |[a, b], v| {
                    [a.min(v), b.max(v)]
                });
            observed = Some(extent.map(Number));
        }
        let discrete = (!domain.is_empty()).then(|| {
            if (0..domain.len()).any(|i| numeric(i).is_nan()) {
                [Number(f64::NAN); 2]
            } else {
                (0..domain.len())
                    .map(numeric)
                    .fold([f64::INFINITY, f64::NEG_INFINITY], |[a, b], v| {
                        [a.min(v), b.max(v)]
                    })
                    .map(Number)
            }
        });
        let viewport =
            self.expansion
                .discrete_mapped_viewport(discrete, observed, continuous_limits)?;
        let entries = self.guide.resolve(&domain)?;
        Ok(PreparedGgplotDiscretePosition {
            domain,
            index,
            palette: self.palette.clone(),
            viewport,
            entries,
            hidden_labels: matches!(self.guide.labels, super::GgplotGuideLabels::Hidden),
        })
    }
}
/// Immutable typed category mapping and reference viewport, including unbounded empty training.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedGgplotDiscretePosition {
    domain: Vec<ScaleKey>,
    index: BTreeMap<ScaleKey, usize>,
    palette: Option<Vec<Number>>,
    viewport: [Number; 2],
    entries: Vec<GgplotDiscreteGuideEntry>,
    hidden_labels: bool,
}
impl PreparedGgplotDiscretePosition {
    /// Exact category identities; an untrained scale has an empty categorical domain.
    pub fn domain(&self) -> &[ScaleKey] {
        &self.domain
    }
    /// Reference panel bounds in one-based category units; may be infinite.
    pub fn viewport(&self) -> [Number; 2] {
        self.viewport
    }
    /// Selected typed guide keys and nullable raw labels.
    pub fn entries(&self) -> &[GgplotDiscreteGuideEntry] {
        &self.entries
    }
    /// Numeric palette coordinate; the default is one-based. Unknown keys never extend training.
    pub fn map(&self, key: &ScaleKey) -> Option<f64> {
        self.index
            .get(key)
            .map(|i| self.palette.as_ref().map_or(1. + *i as f64, |p| p[*i].0))
            .filter(|v| !v.is_nan())
    }
    /// Project a category into finite destination bounds using the shared reference kernel.
    pub fn project(&self, key: &ScaleKey, range: Bounds) -> ChartResult<Option<f64>> {
        range.distinct()?;
        self.map(key).map_or(Ok(None), |value| {
            super::ggplot_unbounded::reference_position(self.viewport, range, value)
        })
    }
}

// The existing checked provider owns guide centering and destination validation.
pub(crate) struct DiscretePositionProvider {
    domain: Vec<crate::composition::ScaleValue>,
    index: BTreeMap<ScaleKey, usize>,
    palette: Option<Vec<Number>>,
    spacing: super::spacing::Spacing,
    entries: Vec<GgplotDiscreteGuideEntry>,
    hidden_labels: bool,
}
impl PreparedGgplotDiscretePosition {
    pub(crate) fn into_provider(
        self,
        range: Bounds,
        inner: f64,
        outer: f64,
        point: bool,
    ) -> ChartResult<DiscretePositionProvider> {
        let spacing = super::spacing::Spacing::new(
            self.domain.len(),
            range,
            &super::BandSpec::<String> {
                domain: None,
                padding_inner: inner,
                padding_outer: outer,
                align: 0.5,
                round: false,
            },
            super::ScaleCompatibility::Legacy,
            point,
        )?
        .with_reference_viewport(self.viewport)?;
        let domain = self
            .domain
            .into_iter()
            .map(|key| match key {
                ScaleKey::Null => crate::composition::ScaleValue::MissingCategory,
                ScaleKey::Text(value) => crate::composition::ScaleValue::Category(value),
                _ => unreachable!("validated positional keys"),
            })
            .collect();
        Ok(DiscretePositionProvider {
            domain,
            index: self.index,
            palette: self.palette,
            spacing,
            entries: self.entries,
            hidden_labels: self.hidden_labels,
        })
    }
}
impl super::PositionalScale for DiscretePositionProvider {
    fn domain(&self) -> &[crate::composition::ScaleValue] {
        &self.domain
    }
    fn range(&self) -> Bounds {
        self.spacing.range
    }
    fn band(&self) -> Option<super::ProviderBand> {
        Some(super::ProviderBand {
            bandwidth: self.spacing.bandwidth(),
            round: false,
        })
    }
    fn category_coordinate(
        &self,
        value: &crate::composition::ScaleValue,
    ) -> ChartResult<Option<f64>> {
        let key = match value {
            crate::composition::ScaleValue::MissingCategory => ScaleKey::Null,
            crate::composition::ScaleValue::Category(value) => ScaleKey::Text(value.clone()),
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "A category key is required.",
                ));
            }
        };
        Ok(self
            .index
            .get(&key)
            .map(|i| self.palette.as_ref().map_or(1. + *i as f64, |p| p[*i].0))
            .filter(|v| !v.is_nan()))
    }
    fn category_viewport(&self) -> Option<[Number; 2]> {
        self.spacing.reference_viewport()
    }
    fn category_minor(&self, value: f64) -> ChartResult<Option<f64>> {
        self.spacing.reference_minor(value)
    }
    fn ticks(
        &self,
        _arguments: &super::GuideTickArguments,
        max_ticks: usize,
    ) -> ChartResult<Option<Vec<crate::composition::ScaleValue>>> {
        crate::limits::require_within(
            self.entries.len() <= max_ticks,
            "positional discrete guide",
        )?;
        Ok(Some(
            self.entries
                .iter()
                .map(|entry| match &entry.key {
                    ScaleKey::Null => crate::composition::ScaleValue::MissingCategory,
                    ScaleKey::Text(value) => {
                        crate::composition::ScaleValue::Category(value.clone())
                    }
                    _ => unreachable!(),
                })
                .collect(),
        ))
    }
    fn format(
        &self,
        value: &crate::composition::ScaleValue,
        _index: usize,
        _values: &[crate::composition::ScaleValue],
        _arguments: &super::GuideTickArguments,
    ) -> ChartResult<Option<String>> {
        if self.hidden_labels {
            return Ok(Some(String::new()));
        }
        let key = match value {
            crate::composition::ScaleValue::MissingCategory => ScaleKey::Null,
            crate::composition::ScaleValue::Category(value) => ScaleKey::Text(value.clone()),
            _ => return Ok(None),
        };
        Ok(self
            .entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.label.clone().unwrap_or_else(|| "NA".into())))
    }
    fn map(&self, value: &crate::composition::ScaleValue) -> ChartResult<Option<f64>> {
        let key = match value {
            crate::composition::ScaleValue::MissingCategory => ScaleKey::Null,
            crate::composition::ScaleValue::Category(value) => ScaleKey::Text(value.clone()),
            _ => {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Discrete position requires a category.",
                ));
            }
        };
        let Some(index) = self.index.get(&key) else {
            return Ok(None);
        };
        if let Some(palette) = &self.palette {
            self.spacing
                .reference_extent(palette[*index].0)
                .map(|v| v.map(Bounds::minimum))
        } else {
            self.spacing.extent(*index).map(|v| v.map(Bounds::minimum))
        }
    }
}
