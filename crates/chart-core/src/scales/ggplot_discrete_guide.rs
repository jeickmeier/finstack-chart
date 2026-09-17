//! Reference discrete guide selection. Mapping and domain training remain in the
//! ordinal and ggplot scale owners; labels never change aesthetic values.
use super::{ScaleKey, error};
use crate::{ChartResult, DiagnosticCode};

/// Scale label arguments, including discrete named replacement.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GgplotGuideLabels {
    /// Reference vector-wide numeric text, or discrete break names/category text.
    #[default]
    Automatic,
    /// Keep guide keys without labels.
    Hidden,
    /// Labels indexed by the original break vector, before discrete intersection/deduplication.
    Explicit(Vec<Option<String>>),
    /// Replace matching discrete labels; unmatched categories retain their text.
    /// Continuous scales use the label vector positionally and ignore the names.
    Named(Vec<(ScaleKey, Option<String>)>),
    /// Registered vector function, evaluated after reference break selection.
    Registered {
        /// Exact installed implementation identity.
        operation: crate::grammar::OperationRef,
        /// Bounded declarative configuration.
        parameters: serde_json::Value,
    },
}
/// Portable discrete break and label arguments for ggplot2 aesthetic scales.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GgplotDiscreteGuide {
    /// None uses the domain; an empty vector suppresses the guide keys.
    pub breaks: Option<Vec<ScaleKey>>,
    /// Optional names for authored breaks, used by automatic labels.
    pub break_names: Option<Vec<String>>,
    /// Label policy, independent of palette mapping.
    pub labels: GgplotGuideLabels,
}
/// A selected key and its optional presentation label.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GgplotDiscreteGuideEntry {
    /// Original domain key used to sample the aesthetic mapping.
    pub key: ScaleKey,
    /// None is a missing/hidden label, not a missing aesthetic key.
    pub label: Option<String>,
}
fn text(key: &ScaleKey) -> Option<String> {
    match key {
        ScaleKey::Null => None,
        ScaleKey::Boolean(v) => Some(if *v { "TRUE" } else { "FALSE" }.into()),
        ScaleKey::Number(v) => Some(crate::number::ecmascript(v.0)),
        ScaleKey::Integer(v) | ScaleKey::Timestamp(v) => Some(v.to_string()),
        ScaleKey::Unsigned(v) => Some(v.to_string()),
        ScaleKey::Text(v) => Some(v.clone()),
    }
}
impl GgplotGuideLabels {
    pub(super) fn registered_values(
        &self,
        values: &[crate::composition::ScaleValue],
        names: Option<&[String]>,
        temporal: Option<crate::grammar::GuideTemporalContext<'_>>,
        registry: &crate::grammar::guide_extensions::GuideRegistrations,
        label_budget: usize,
    ) -> ChartResult<Vec<Option<String>>> {
        let Self::Registered {
            operation,
            parameters,
        } = self
        else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "A registered label policy is required.",
            ));
        };
        registry.label_vector(
            operation,
            crate::grammar::GuideLabelsInput {
                facet: None,
                values,
                names,
                temporal,
                parameters,
                limits: crate::Limits {
                    max_items: crate::interpolate::MAX_VALUES,
                    max_text_bytes: label_budget,
                    ..Default::default()
                },
            },
            false,
        )
    }
    pub(super) fn validate(&self, breaks: Option<usize>) -> ChartResult<()> {
        if let Self::Registered { parameters, .. } = self {
            crate::grammar::guide_extensions::validate_parameters(parameters)?;
        }
        let count = match self {
            Self::Explicit(v) => Some(v.len()),
            Self::Named(v) => Some(v.len()),
            _ => None,
        };
        if let (Some(n), Some(labels)) = (breaks, count)
            && n != labels
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Breaks and labels must have the same length.",
            ));
        }
        if breaks.unwrap_or(0) > crate::interpolate::MAX_VALUES
            || count.unwrap_or(0) > crate::interpolate::MAX_VALUES
        {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Guide exceeds the value budget.",
            ));
        }
        let long_labels = match self {
            Self::Explicit(v) => v.iter().flatten().any(|s| s.len() > 4096),
            Self::Named(v) => v
                .iter()
                .filter_map(|(_, v)| v.as_ref())
                .any(|s| s.len() > 4096),
            _ => false,
        };
        if long_labels {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Guide labels exceed 4096 bytes.",
            ));
        }
        Ok(())
    }
}
impl GgplotDiscreteGuide {
    pub(super) fn validate(&self) -> ChartResult<()> {
        let n = self.breaks.as_ref().map(Vec::len);
        if self
            .break_names
            .as_ref()
            .is_some_and(|names| Some(names.len()) != n)
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Break names require one name per authored break.",
            ));
        }
        if self.break_names.iter().flatten().any(|s| s.len() > 4096) {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Guide labels exceed 4096 bytes.",
            ));
        }
        self.labels.validate(n)
    }
    /// Intersect in authored order, retaining the first position of duplicates.
    /// The supplied domain must come from the prepared scale's shared training.
    pub fn resolve(&self, domain: &[ScaleKey]) -> ChartResult<Vec<GgplotDiscreteGuideEntry>> {
        self.resolve_with_registry(domain, &crate::grammar::ExtensionRegistry::new())
    }
    /// Resolve with an explicit vector-label registry. Guide keys use the reference
    /// data-frame recycling rule; the raw callback result is not a named replacement.
    pub fn resolve_with_registry(
        &self,
        domain: &[ScaleKey],
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Vec<GgplotDiscreteGuideEntry>> {
        self.resolve_using(domain, &registry.guides)
    }
    pub(crate) fn resolve_using(
        &self,
        domain: &[ScaleKey],
        registry: &crate::grammar::guide_extensions::GuideRegistrations,
    ) -> ChartResult<Vec<GgplotDiscreteGuideEntry>> {
        self.resolve_using_context(
            domain,
            registry,
            !domain.is_empty(),
            crate::Limits {
                max_items: crate::interpolate::MAX_VALUES,
                max_text_bytes: 1_048_576,
                ..Default::default()
            },
        )
    }
    pub(crate) fn resolve_using_context(
        &self,
        domain: &[ScaleKey],
        registry: &crate::grammar::guide_extensions::GuideRegistrations,
        trained: bool,
        limits: crate::Limits,
    ) -> ChartResult<Vec<GgplotDiscreteGuideEntry>> {
        self.resolve_context(domain, registry, trained, limits, false)
    }
    pub(crate) fn resolve_break_function_using(
        &self,
        domain: &[ScaleKey],
        registry: &crate::grammar::guide_extensions::GuideRegistrations,
    ) -> ChartResult<Vec<GgplotDiscreteGuideEntry>> {
        self.resolve_context(
            domain,
            registry,
            !domain.is_empty(),
            crate::Limits {
                max_items: crate::interpolate::MAX_VALUES,
                max_text_bytes: 1_048_576,
                ..Default::default()
            },
            true,
        )
    }
    fn resolve_context(
        &self,
        domain: &[ScaleKey],
        registry: &crate::grammar::guide_extensions::GuideRegistrations,
        trained: bool,
        limits: crate::Limits,
        skip_empty: bool,
    ) -> ChartResult<Vec<GgplotDiscreteGuideEntry>> {
        self.validate()?;
        if domain.len() > crate::interpolate::MAX_VALUES {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Discrete guide domain exceeds the value budget.",
            ));
        }
        let lookup: std::collections::BTreeMap<_, _> =
            domain.iter().rev().map(|k| (text(k), k)).collect();
        if matches!(self.labels, GgplotGuideLabels::Registered { .. }) {
            if !trained {
                return Ok(vec![]);
            }
            let mut selection = self.clone();
            selection.labels = GgplotGuideLabels::Hidden;
            let source = self.breaks.as_deref().unwrap_or(domain);
            let mut entries = selection.select(source, &lookup)?;
            if skip_empty && entries.is_empty() {
                return Ok(entries);
            }
            let values = entries
                .iter()
                .map(|e| {
                    text(&e.key).map_or(
                        crate::composition::ScaleValue::MissingCategory,
                        crate::composition::ScaleValue::Category,
                    )
                })
                .collect::<Vec<_>>();
            let names = self.break_names.as_ref().map(|names| {
                entries
                    .iter()
                    .map(|e| {
                        let index = source
                            .iter()
                            .position(|k| text(k) == text(&e.key))
                            .expect("selected break");
                        names[index].clone()
                    })
                    .collect::<Vec<_>>()
            });
            let GgplotGuideLabels::Registered {
                operation,
                parameters,
            } = &self.labels
            else {
                unreachable!()
            };
            let labels = registry.label_vector(
                operation,
                crate::grammar::GuideLabelsInput {
                    facet: None,
                    values: &values,
                    names: names.as_deref(),
                    temporal: None,
                    parameters,
                    limits,
                },
                false,
            )?;
            if entries.is_empty() {
                return Ok(entries);
            }
            if labels.is_empty() || !entries.len().is_multiple_of(labels.len()) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Label function output cannot be recycled to the selected guide keys.",
                ));
            }
            for (i, entry) in entries.iter_mut().enumerate() {
                entry.label = labels[i % labels.len()].clone();
            }
            return Ok(entries);
        }
        self.select(self.breaks.as_deref().unwrap_or(domain), &lookup)
    }

    fn select(
        &self,
        source: &[ScaleKey],
        domain: &std::collections::BTreeMap<Option<String>, &ScaleKey>,
    ) -> ChartResult<Vec<GgplotDiscreteGuideEntry>> {
        let named: std::collections::BTreeMap<_, _> = match &self.labels {
            GgplotGuideLabels::Named(labels) => labels.iter().map(|(k, v)| (text(k), v)).collect(),
            _ => Default::default(),
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut out = vec![];
        for (i, key) in source.iter().enumerate() {
            let category = text(key);
            let Some(original) = domain.get(&category) else {
                continue;
            };
            if !seen.insert(category.clone()) {
                continue;
            }
            let label = match &self.labels {
                GgplotGuideLabels::Automatic => self
                    .break_names
                    .as_ref()
                    .map(|names| Some(names[i].clone()))
                    .unwrap_or_else(|| category.clone()),
                GgplotGuideLabels::Hidden => None,
                GgplotGuideLabels::Explicit(labels) => labels.get(i).cloned().flatten(),
                GgplotGuideLabels::Named(_) => named
                    .get(&category)
                    .map(|v| (*v).clone())
                    .unwrap_or_else(|| category.clone()),
                GgplotGuideLabels::Registered { .. } => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Registered scale labels require an explicit guide registry.",
                    ));
                }
            };
            out.push(GgplotDiscreteGuideEntry {
                key: (*original).clone(),
                label,
            });
        }
        Ok(out)
    }
}
