//! Positional bin classification before statistics and interval mapping afterward.
use super::*;
use crate::interpolate::{Number, Value};

/// Reference positional binned scale, sharing numeric population and break policies.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GgplotBinnedPosition {
    /// Registered cuts shared by classification and guides (wire v35).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breaks_function: Option<Box<crate::grammar::ScaleBreaksOperation>>,
    /// Shared limits, cuts, out-of-bounds and closure policy.
    pub bins: GgplotBinnedPolicy,
    /// Transformation before classification; absent selects identity.
    pub transform: Option<ScaleTransform>,
    /// Guide labels for the selected cuts.
    pub labels: GgplotGuideLabels,
    /// Replacement in transformed units after OOB handling and before classification.
    pub missing: Option<Number>,
    /// Add the final scale limits to the guide candidates.
    pub show_limits: bool,
}
impl Default for GgplotBinnedPosition {
    fn default() -> Self {
        Self {
            breaks_function: None,
            bins: GgplotBinnedPolicy {
                breaks: GgplotBreaks::Nice(10.),
                ..Default::default()
            },
            transform: None,
            labels: GgplotGuideLabels::Automatic,
            missing: None,
            show_limits: false,
        }
    }
}
impl GgplotBinnedPosition {
    fn family(&self) -> (NumericFamily, bool) {
        super::ggplot_numeric_limits::positional_coordinates(self.transform.clone())
    }
    /// Check authored parameters without selecting cuts on a placeholder population.
    pub fn validate(&self) -> ChartResult<()> {
        if self.breaks_function.is_some() && matches!(self.bins.breaks, GgplotBreaks::Explicit(_)) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Binned position break functions cannot compete with fixed cuts.",
            ));
        }
        if self.bins.palette.is_some() {
            return Err(error(
                DiagnosticCode::Validation,
                "Position bins do not accept aesthetic palettes.",
            ));
        }
        if let Some(transform) = &self.transform {
            transform.validate()?;
        }
        if self.breaks_function.is_some() {
            let count = match self.bins.breaks {
                GgplotBreaks::Nice(n) | GgplotBreaks::Equal(n) => n,
                _ => unreachable!("fixed cuts rejected above"),
            };
            if !count.is_finite() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Break function counts must be finite.",
                ));
            }
            let mut selected = self.bins.clone();
            selected.breaks = GgplotBreaks::Explicit(vec![]);
            selected.validate_break_budget(4096)?;
        } else {
            self.bins.validate_break_budget(4096)?;
        }
        self.labels.validate(match &self.bins.breaks {
            GgplotBreaks::Explicit(v) if !self.show_limits => Some(v.len()),
            _ => None,
        })
    }
    /// Train the complete pre-statistic population using the shared numeric owner.
    pub fn train(&self, values: &[Option<Number>]) -> ChartResult<PreparedGgplotBinnedPosition> {
        self.train_with_registry(values, &crate::grammar::ExtensionRegistry::new())
    }
    /// Train with explicitly installed pure cut selectors.
    pub fn train_with_registry(
        &self,
        values: &[Option<Number>],
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<PreparedGgplotBinnedPosition> {
        self.validate()?;
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "positional bin population",
        )?;
        let (family, reverse) = self.family();
        let spec = MappedScaleSpec {
            colorbar_options: None,
            palette_theme_aesthetics: vec![],
            oob_function: None,
            rescaler_function: None,
            palette_fallback_indices: vec![],
            missing_paint_is_na: false,
            palette_function: None,
            resolved_numeric_limits: None,
            resolved_discrete_limits_null: false,
            trained_transformed_bounds: None,
            limits_function: None,
            breaks_function: self.breaks_function.clone(),
            training: ScaleTraining::Eligible,
            ggplot: Some(Box::new(GgplotScalePolicy::Binned(Box::new(
                self.bins.clone(),
            )))),
            guide: Some(Box::new(GgplotScaleGuide::Binned(if self.show_limits {
                GgplotGuideLabels::Automatic
            } else {
                self.labels.clone()
            }))),
            catalog: None,
            function: ScaleFunctionSpec::Interpolated(InterpolatedScaleSpec {
                normalization: NormalizationSpec::Ggplot {
                    timestamp: None,
                    family: family.clone(),
                    domain: [Number(1.), Number(10.)],
                    reverse,
                    rescaler: GgplotRescaler::Range,
                },
                output: ScaleRangeFunction::Identity,
                unknown: Value::Missing,
            }),
        }
        .trained_with_registry(values, registry)?;
        let ScaleFunctionSpec::Interpolated(function) = &spec.function else {
            unreachable!()
        };
        let NormalizationSpec::Ggplot { domain, .. } = function.normalization else {
            unreachable!()
        };
        let Some(GgplotScalePolicy::Binned(policy)) = spec.ggplot.as_deref() else {
            unreachable!()
        };
        let limits = policy
            .population_bounds(family.clone(), reverse)
            .unwrap_or_else(|| {
                domain.map(|v| ggplot_continuous_guide::forward(&family, reverse, v.0))
            });
        let selected_policy = self.breaks_function.as_ref().map(|_| policy.clone());
        let mut entries = MappedScale::new_with_registry(spec, registry)?
            .binned_guide_entries(4096, 4096)?
            .unwrap();
        let selected_breaks = entries.iter().map(|e| e.transformed).collect();
        let selected_break_names = entries.iter().any(|e| e.name.is_some()).then(|| {
            entries
                .iter()
                .map(|e| e.name.clone().unwrap_or_default())
                .collect()
        });
        if self.show_limits {
            entries = self.entries_with_limits(entries, limits, &self.labels, 4096)?;
        }
        let mut boundaries = entries
            .iter()
            .map(|v| v.transformed)
            .chain(limits.map(Number))
            .filter(|v| !v.0.is_nan())
            .collect::<Vec<_>>();
        boundaries.sort_by(|a, b| a.0.total_cmp(&b.0));
        boundaries.dedup_by(|a, b| a.0 == b.0);
        let mut authored = self.clone();
        // Prepared caches carry selected cuts, never an executable selector identity.
        authored.breaks_function = None;
        if let Some(policy) = selected_policy {
            authored.bins = *policy;
        }
        authored.bins.empty_population = values.is_empty();
        let result = PreparedGgplotBinnedPosition {
            spec: authored,
            limits: limits.map(Number),
            entries,
            boundaries,
            source_boundaries: None,
            statistic_boundaries: None,
            selected_breaks,
            selected_break_names,
            function_limits: None,
        };
        result.validate()?;
        Ok(result)
    }
    fn entries_with_limits(
        &self,
        entries: Vec<GgplotContinuousGuideEntry>,
        limits: [f64; 2],
        labels: &GgplotGuideLabels,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        let named = entries.iter().any(|e| e.name.is_some());
        let mut cuts = limits
            .into_iter()
            .map(|value| (value, None))
            .chain(entries.into_iter().map(|e| (e.transformed.0, e.name)))
            .filter(|(value, _)| !value.is_nan())
            .collect::<Vec<_>>();
        cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
        cuts.dedup_by(|a, b| a.0 == b.0);
        let (family, reverse) = self.family();
        let mut result = ggplot_continuous_guide::numeric_guide_entries(
            cuts.iter().map(|(value, _)| *value).collect(),
            limits,
            family,
            reverse,
            labels,
            label_budget,
        )?;
        if named {
            for (entry, (_, name)) in result.iter_mut().zip(cuts) {
                entry.name = Some(name.unwrap_or_default());
            }
        }
        Ok(result)
    }
    /// Resolve callback limits once and cache the initial cuts before scale reset.
    pub(crate) fn train_function(
        &self,
        limits: Option<Vec<Number>>,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<(PreparedGgplotBinnedPosition, bool)> {
        self.validate()?;
        let (family, reverse) = self.family();
        let null_limits = limits.is_none();
        let limits = limits.unwrap_or_default();
        let pair = [
            limits.first().map_or(f64::NAN, |v| v.0),
            limits.get(1).map_or(f64::NAN, |v| v.0),
        ];
        let mut policy = self.bins.clone();
        policy.limits = Some([None, None]);
        let (raw_cuts, selected_break_names, null_cuts) = if let Some(call) = &self.breaks_function
        {
            let count = match self.bins.breaks {
                GgplotBreaks::Nice(n) | GgplotBreaks::Equal(n) => n,
                _ => unreachable!("validated function cuts"),
            };
            let selected = ggplot_binned_breaks::select(
                call,
                &registry.breaks_function,
                (!null_limits).then_some(pair),
                family.clone(),
                reverse,
                count,
                4096,
            )?;
            (selected.values, selected.names, selected.is_null)
        } else {
            (
                policy
                    .resolve_transformed(pair, family.clone(), reverse, 4096)?
                    .1,
                None,
                false,
            )
        };
        let mut authored = self.clone();
        if authored.breaks_function.take().is_some() {
            authored.bins.breaks = GgplotBreaks::Explicit(raw_cuts.clone());
        }
        let selected_breaks: Vec<_> = raw_cuts
            .into_iter()
            .map(|v| Number(ggplot_continuous_guide::forward(&family, reverse, v.0)))
            .collect();
        let mut boundaries: Vec<_> = selected_breaks
            .iter()
            .copied()
            .chain(pair.map(Number))
            .filter(|v| !v.0.is_nan())
            .collect();
        boundaries.sort_by(|a, b| a.0.total_cmp(&b.0));
        boundaries.dedup_by(|a, b| a.0 == b.0);
        Ok((
            PreparedGgplotBinnedPosition {
                spec: authored,
                limits: pair.map(Number),
                entries: vec![],
                boundaries,
                source_boundaries: None,
                statistic_boundaries: None,
                selected_breaks,
                selected_break_names,
                function_limits: Some(limits),
            },
            null_cuts,
        ))
    }
}

/// Captured reference bin state. Statistics consume indices; geometry consumes intervals.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparedGgplotBinnedPosition {
    spec: GgplotBinnedPosition,
    limits: [Number; 2],
    entries: Vec<GgplotContinuousGuideEntry>,
    boundaries: Vec<Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_boundaries: Option<Vec<Number>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    statistic_boundaries: Option<Vec<Number>>,
    selected_breaks: Vec<Number>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    selected_break_names: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    function_limits: Option<Vec<Number>>,
}
impl PreparedGgplotBinnedPosition {
    pub(crate) fn resolve_transform(
        &mut self,
        registrations: &crate::grammar::transform_extensions::TransformRegistrations,
        portable: bool,
    ) -> ChartResult<()> {
        if let Some(t) = self
            .spec
            .transform
            .as_mut()
            .and_then(super::ScaleTransform::ggplot_transform_mut)
        {
            t.resolve_registrations(registrations, portable)?;
        }
        Ok(())
    }
    /// Validate captured state before accepting it from a portable descriptor.
    pub fn validate(&self) -> ChartResult<()> {
        self.spec.validate()?;
        if self.spec.breaks_function.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Prepared positional cuts must materialize their break function.",
            ));
        }
        if let Some(names) = &self.selected_break_names {
            crate::limits::require_within(
                names.len() == self.selected_breaks.len(),
                "positional break names",
            )?;
            crate::limits::require_within(
                names.iter().fold(0usize, |n, s| n.saturating_add(s.len()))
                    <= crate::interpolate::MAX_VALUE_BYTES,
                "positional break name bytes",
            )?;
        }
        if let Some(limits) = &self.function_limits {
            crate::limits::require_within(limits.len() <= 4096, "positional function limits")?;
        }
        crate::limits::require_within(
            self.boundaries.len() <= 4098
                && self.entries.len() <= 4098
                && self.selected_breaks.len() <= 4096,
            "prepared positional bin",
        )?;
        if self.boundaries.iter().any(|v| v.0.is_nan())
            || self.boundaries.windows(2).any(|v| v[0].0 >= v[1].0)
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Positional bin boundaries must be comparable and strictly increasing.",
            ));
        }
        if let Some(cuts) = &self.source_boundaries {
            crate::limits::require_within(cuts.len() <= 4097, "scalar positional cut")?;
            if self.boundaries.len() != 1
                || cuts.len() < 2
                || cuts.iter().any(|v| !v.0.is_finite())
                || cuts.windows(2).any(|v| v[0].0 >= v[1].0)
            {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Captured scalar positional cuts are invalid.",
                ));
            }
        }
        if let Some(cuts) = &self.statistic_boundaries {
            crate::limits::require_within(cuts.len() <= 4098, "statistic positional cut")?;
            if cuts.iter().any(|v| v.0.is_nan()) || cuts.windows(2).any(|v| v[0].0 >= v[1].0) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Invalid statistic positional cuts.",
                ));
            }
        }
        Ok(())
    }
    pub(crate) fn reset_population(&self) -> Vec<Option<Number>> {
        let (family, reverse) = self.spec.family();
        self.selected_breaks
            .iter()
            .chain(self.limits.iter())
            .filter(|v| !v.0.is_nan())
            .map(|v| {
                Some(Number(ggplot_continuous_guide::inverse(
                    &family, reverse, v.0,
                )))
            })
            .collect()
    }
    pub(crate) fn has_break_names(&self) -> bool {
        self.selected_break_names.is_some()
    }
    fn entries_for_limits(&self, bounds: [f64; 2]) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        let (family, reverse) = self.spec.family();
        let mut entries = ggplot_continuous_guide::numeric_guide_entries(
            self.selected_breaks.iter().map(|v| v.0).collect(),
            bounds,
            family,
            reverse,
            &self.spec.labels,
            4096,
        )?;
        if let Some(names) = &self.selected_break_names {
            for (entry, name) in entries.iter_mut().zip(names) {
                entry.name = Some(name.clone());
            }
        }
        if self.spec.show_limits {
            entries = self
                .spec
                .entries_with_limits(entries, bounds, &self.spec.labels, 4096)?;
        }
        Ok(entries)
    }
    pub(crate) fn reset_function(mut self, limits: Vec<Number>) -> ChartResult<Self> {
        self.function_limits = Some(limits);
        self.entries = self.entries_for_limits(self.panel_limits().map(|v| v.0))?;
        Ok(self)
    }
    pub(crate) fn function_limits(&self) -> Option<&[Number]> {
        self.function_limits.as_deref()
    }
    pub(crate) fn bind_initial_source(mut self, values: &[Option<Number>]) -> ChartResult<Self> {
        self.spec.bins.empty_population = values.is_empty();
        if self.boundaries.is_empty() && !values.is_empty() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Positional classification requires bin boundaries.",
            ));
        }
        if self.boundaries.len() == 1 && !values.is_empty() && self.source_boundaries.is_none() {
            let input = values
                .iter()
                .map(|v| self.input(v.map(|v| v.0)))
                .collect::<Vec<_>>();
            self.source_boundaries = Some(cut_boundaries(&self.boundaries, &input)?);
        }
        Ok(self)
    }
    pub(crate) fn bind_source(self, values: &[Option<Number>]) -> ChartResult<Self> {
        self.bind_initial_source(values)?.finish_source()
    }
    pub(crate) fn bind_vector_population(mut self, empty: bool) -> ChartResult<Self> {
        self.spec.bins.empty_population = empty;
        self.finish_source()
    }
    fn finish_source(self) -> ChartResult<Self> {
        let mut next = self;
        // The reference reset retains all initial cuts as its trained range,
        // then reapplies authored limits before mapping statistic indices.
        let mut cuts = next
            .selected_breaks
            .iter()
            .copied()
            .chain(next.panel_limits())
            .filter(|v| !v.0.is_nan())
            .collect::<Vec<_>>();
        cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
        cuts.dedup_by(|a, b| a.0 == b.0);
        if next.spec.show_limits {
            next.entries = next.entries_for_limits(next.panel_limits().map(|v| v.0))?;
        }
        next.statistic_boundaries = Some(cuts);
        next.validate()?;
        Ok(next)
    }
    pub(crate) fn with_authored_labels(mut self, labels: GgplotGuideLabels) -> Self {
        self.spec.labels = labels;
        self
    }
    pub(crate) fn authored(&self) -> &GgplotBinnedPosition {
        &self.spec
    }
    fn input(&self, value: Option<f64>) -> Option<f64> {
        let (family, reverse) = self.spec.family();
        self.spec
            .bins
            .oob
            .apply(
                value.map(|v| ggplot_continuous_guide::forward(&family, reverse, v)),
                self.limits.map(|v| v.0),
            )
            .or(self.spec.missing.map(|v| v.0))
            .filter(|v| !v.is_nan())
    }
    /// Classify an already transformed OOB vector without reapplying the scalar policy.
    pub(crate) fn project_vector_source(&self, values: &[Number]) -> ChartResult<Vec<Number>> {
        if values.is_empty() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Positional bin classification requires a nonempty numeric callback result.",
            ));
        }
        let values = values
            .iter()
            .map(|v| {
                if v.0.is_nan() {
                    self.spec.missing.map(|v| v.0)
                } else {
                    Some(v.0)
                }
            })
            .collect::<Vec<_>>();
        let boundaries = cut_boundaries(&self.boundaries, &values)?;
        Ok(values
            .iter()
            .map(|v| {
                Number(
                    v.and_then(|v| classify(&boundaries, v, self.spec.bins.right))
                        .map_or(f64::NAN, |v| v as f64),
                )
            })
            .collect())
    }
    pub(crate) fn project_source(&self, value: Option<f64>) -> Option<f64> {
        classify(
            self.source_boundaries
                .as_deref()
                .unwrap_or(&self.boundaries),
            self.input(value)?,
            self.spec.bins.right,
        )
        .map(|i| i as f64)
    }
    pub(crate) fn project_statistic(&self, value: f64) -> Option<f64> {
        let boundaries = self
            .statistic_boundaries
            .as_deref()
            .unwrap_or(&self.boundaries);
        let n = boundaries.len().saturating_sub(1);
        if n == 0 || !value.is_finite() || value < 0.5 || value > n as f64 + 0.5 {
            return None;
        }
        let index = if self.spec.bins.right {
            (value - 0.5).ceil()
        } else {
            (value - 0.5).floor() + 1.
        }
        .max(1.) as usize;
        if index > n {
            return None;
        }
        let a = boundaries[index - 1].0;
        let b = boundaries[index].0;
        Some((value - index as f64 + 0.5) * (b - a) + a)
    }
    /// Pre-statistic limits in transformed units, preserving authored orientation.
    pub fn limits(&self) -> [Number; 2] {
        self.limits
    }
    pub(crate) fn panel_limits(&self) -> [Number; 2] {
        if let Some(limits) = &self.function_limits {
            let mut values = limits.iter().map(|v| v.0).filter(|v| !v.is_nan());
            let first = values.next().unwrap_or(f64::NAN);
            let (lo, hi) = values.fold((first, first), |(lo, hi), v| (lo.min(v), hi.max(v)));
            return [Number(lo), Number(hi)];
        }
        let (family, reverse) = self.spec.family();
        // Reset training follows the numeric range contract: infinite cut sentinels
        // remain classification boundaries but do not expand a finite panel range.
        let mut finite = self.boundaries.iter().filter(|v| v.0.is_finite());
        let first = finite.next().map(|v| v.0);
        let last = finite.next_back().map(|v| v.0).or(first);
        let fallback = [
            first.unwrap_or(self.limits[0].0),
            last.unwrap_or(self.limits[1].0),
        ];
        ggplot_continuous_guide::authored_bounds(self.spec.bins.limits, family, reverse, fallback)
            .map(Number)
    }
    /// Unthinned reference candidates and labels before axis composition.
    pub fn guide_entries(&self) -> &[GgplotContinuousGuideEntry] {
        &self.entries
    }
    /// An untrained primary view selects fresh cuts against its expanded panel,
    /// while the immutable source classification still has no trained cuts.
    pub(crate) fn empty_axis_guide_entries(
        &self,
        viewport: [f64; 2],
        budget: usize,
        labels: &GgplotGuideLabels,
        label_budget: usize,
        function: Option<(
            &crate::grammar::ScaleBreaksOperation,
            &crate::grammar::scale_break_extensions::ScaleBreakRegistrations,
            f64,
        )>,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        if let Some((call, registry, count)) = function {
            let (family, reverse) = self.spec.family();
            let selected = ggplot_binned_breaks::select(
                call,
                registry,
                Some(viewport),
                family.clone(),
                reverse,
                count,
                budget,
            )?;
            let mut entries = ggplot_continuous_guide::numeric_guide_entries(
                selected
                    .values
                    .into_iter()
                    .map(|v| ggplot_continuous_guide::forward(&family, reverse, v.0))
                    .collect(),
                viewport,
                family,
                reverse,
                labels,
                label_budget,
            )?;
            if let Some(names) = selected.names {
                for (entry, name) in entries.iter_mut().zip(names) {
                    entry.name = Some(name);
                }
            }
            if self.spec.show_limits {
                entries = self.spec.entries_with_limits(
                    entries,
                    self.panel_limits().map(|v| v.0),
                    labels,
                    label_budget,
                )?;
            }
            crate::limits::require_within(entries.len() <= budget, "positional bin guide tick")?;
            return Ok(entries);
        }
        let (family, reverse) = self.spec.family();
        let (bounds, cuts) =
            self.spec
                .bins
                .resolve_transformed(viewport, family.clone(), reverse, budget)?;
        let mut cuts = cuts
            .into_iter()
            .map(|v| ggplot_continuous_guide::forward(&family, reverse, v.0))
            .collect::<Vec<_>>();
        if self.spec.show_limits {
            let added = if self.spec.bins.limits.is_none()
                && !matches!(self.spec.bins.breaks, GgplotBreaks::Explicit(_))
            {
                bounds
            } else {
                self.panel_limits().map(|v| v.0)
            };
            cuts.extend(added);
            cuts.retain(|v| !v.is_nan());
            cuts.sort_by(f64::total_cmp);
            cuts.dedup_by(|a, b| *a == *b);
        }
        crate::limits::require_within(cuts.len() <= budget, "positional bin guide tick")?;
        ggplot_continuous_guide::numeric_guide_entries(
            cuts,
            viewport,
            family,
            reverse,
            labels,
            label_budget,
        )
    }
    /// Sorted interval boundaries in transformed units.
    pub fn boundaries(&self) -> &[Number] {
        &self.boundaries
    }
    /// Classify a complete raw source batch into one-based statistic inputs.
    pub fn map_before_statistics(
        &self,
        values: &[Option<Number>],
    ) -> ChartResult<Vec<Option<Number>>> {
        self.validate()?;
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "positional bin mapping",
        )?;
        let transformed = values
            .iter()
            .map(|v| self.input(v.map(|v| v.0)))
            .collect::<Vec<_>>();
        let boundaries = cut_boundaries(&self.boundaries, &transformed)?;
        Ok(transformed
            .into_iter()
            .map(|v| {
                v.and_then(|v| classify(&boundaries, v, self.spec.bins.right))
                    .map(|v| Number(v as f64))
            })
            .collect())
    }
    /// Map statistic indices and fractional offsets back into transformed intervals.
    pub fn map_after_statistics(
        &self,
        values: &[Option<Number>],
    ) -> ChartResult<Vec<Option<Number>>> {
        self.validate()?;
        crate::limits::require_within(
            values.len() <= crate::interpolate::MAX_VALUES,
            "post-statistic positional bin mapping",
        )?;
        Ok(values
            .iter()
            .map(|v| v.and_then(|v| self.project_statistic(v.0)).map(Number))
            .collect())
    }
}
fn classify(bounds: &[Number], x: f64, right: bool) -> Option<usize> {
    if bounds.len() < 2 || x < bounds[0].0 || x > bounds.last()?.0 || x.is_nan() {
        return None;
    }
    Some(
        super::classifier::bisect_right(
            &bounds[1..bounds.len() - 1],
            |v| if right { v.0 < x } else { v.0 <= x },
            0,
        ) + 1,
    )
}
fn cut_boundaries(bounds: &[Number], values: &[Option<f64>]) -> ChartResult<Vec<Number>> {
    if bounds.len() != 1 {
        return Ok(bounds.to_vec());
    }
    let count = bounds[0].0;
    if !count.is_finite() || count < 2. {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "A scalar positional cut count must be finite and at least two.",
        ));
    }
    crate::limits::require_within(count <= 4096., "scalar positional cut count")?;
    let [mut lo, mut hi] = values
        .iter()
        .flatten()
        .fold([f64::INFINITY, f64::NEG_INFINITY], |[lo, hi], v| {
            [lo.min(*v), hi.max(*v)]
        });
    if !lo.is_finite() || !hi.is_finite() {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "Scalar positional cuts require finite population endpoints.",
        ));
    }
    let count = (count + 1.).trunc() as usize;
    let span = hi - lo;
    let constant = span == 0.;
    let margin = if constant {
        if lo == 0. { 1. } else { lo.abs() }
    } else {
        span
    } / 1000.;
    if constant {
        lo -= margin;
        hi += margin;
    }
    let mut cuts = (0..count)
        .map(|i| Number(lo + (hi - lo) * i as f64 / (count - 1) as f64))
        .collect::<Vec<_>>();
    if !constant {
        cuts[0].0 -= margin;
        cuts[count - 1].0 += margin;
    }
    Ok(cuts)
}
