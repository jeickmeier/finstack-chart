use super::encode::encode_layer;
use super::position_stage::position_layer;
use super::recipe_emit::finish_layer;
use super::stats::{numeric_space, validate_group};
use super::*;
use crate::data::{DatasetSnapshot, SnapshotHandle, StoreSnapshot};
use crate::provenance::Target;
use crate::state::ChartState;
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId, Point, RowKey, TransformId};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

#[derive(Clone)]
struct CachedOutput {
    table: Arc<PreparedTable>,
    diagnostics: Vec<Diagnostic>,
}
struct CachedGraph {
    semantics: Option<ExecutionSemantics>,
    population_axes: Vec<crate::layout::AxisSpec>,
    scope: Option<facets::PanelScope>,
    source: SnapshotHandle<StoreSnapshot>,
    definitions: Vec<TransformDefinition>,
    limits: CompileLimits,
    outputs: BTreeMap<TransformId, CachedOutput>,
    layers: BTreeMap<LayerId, (Layer, CachedOutput)>,
}

pub(crate) struct PreparedScope {
    graph: CachedGraph,
    tables: BTreeMap<TransformId, Arc<PreparedTable>>,
    diagnostics: Vec<Diagnostic>,
    metrics: PreparationMetrics,
    encoded: BTreeMap<LayerId, EncodedLayer>,
}
/// Positioned rows retained until all panels sharing an axis can train together.
pub(crate) struct PositionedScope {
    population: PreparedScope,
    layers: Vec<(Layer, PositionedLayer)>,
    color_domains: BTreeMap<crate::ScaleId, Vec<String>>,
    vertices: usize,
    positional_limits: BTreeMap<crate::ScaleId, Vec<crate::interpolate::Number>>,
    positional_empty: std::collections::BTreeSet<crate::ScaleId>,
}

pub(crate) fn synchronize_position_populations(
    definitions: &mut [ChartDefinition],
    populations: &[&PreparedScope],
) {
    let mut counts = std::collections::BTreeMap::<crate::LayerId, (usize, Vec<f64>)>::new();
    for (definition, population) in definitions.iter().zip(populations) {
        for layer in &definition.layers {
            if !matches!(
                layer.position,
                Position::GgplotDodge(_) | Position::GgplotDodge2(_) | Position::JitterDodge(_)
            ) {
                continue;
            }
            if let Some(encoded) = population.encoded.get(&layer.id) {
                let count =
                    super::ggplot_position::collision_population(&layer.position, &encoded.encoded);
                let entry = counts.entry(layer.id).or_default();
                entry.0 = entry.0.max(count);
                if matches!(layer.position, Position::JitterDodge(_)) {
                    entry.1.extend(encoded.encoded.iter().filter_map(|r| r.x));
                }
            }
        }
    }
    let counts = counts
        .into_iter()
        .map(|(id, (count, values))| (id, (count, super::ggplot_position::resolution(values))))
        .collect::<BTreeMap<_, _>>();
    for definition in definitions {
        for layer in &mut definition.layers {
            if let Some((count, resolution)) = counts.get(&layer.id) {
                super::ggplot_position::resolve_population(
                    &mut layer.position,
                    *count,
                    *resolution,
                );
            }
        }
    }
}

pub(crate) fn transform_generated_scopes(
    scopes: &mut [(&ChartDefinition, &mut PreparedScope)],
) -> ChartResult<()> {
    let ids = scopes
        .iter()
        .flat_map(|(_, s)| s.encoded.keys().copied())
        .collect::<BTreeSet<_>>();
    for id in ids {
        let mut groups = scopes
            .iter_mut()
            .filter_map(|(definition, scope)| {
                let layer = definition.layers.iter().find(|l| l.id == id)?;
                let encoded = scope.encoded.get_mut(&id)?;
                Some((
                    layer,
                    population_axes(definition),
                    &mut encoded.domains,
                    &mut encoded.encoded,
                ))
            })
            .collect::<Vec<_>>();
        super::scale_stage::transform_generated_populations(&mut groups)?;
    }
    Ok(())
}

/// Keep pre-stat positional contributions for reference shrink=false without painting source rows.
pub(crate) fn retain_facet_raw_domains(
    definition: &ChartDefinition,
    panels: &[ChartDefinition],
    scopes: &mut [PositionedScope],
    source: &StoreSnapshot,
    limits: CompileLimits,
) -> ChartResult<()> {
    let spec = definition.facets.as_ref().expect("facet source training");
    if spec.reference.as_ref().is_none_or(|p| p.shrink) {
        return Ok(());
    }
    let mut remaining = limits.max_prepared_rows;
    for (index, (panel, scope)) in panels.iter().zip(scopes).enumerate() {
        let panel_scope = super::facets::PanelScope::new(spec, spec.order[index].clone());
        for (layer, positioned) in &mut scope.layers {
            let data = source.dataset(positioned.prepared.table.input.dataset)?;
            let mut mappings = match &layer.mappings {
                Mappings::Source(a) => a.clone(),
                _ => layer
                    .grammar
                    .as_ref()
                    .map(|g| g.source.clone())
                    .unwrap_or_default(),
            };
            match &layer.statistic.parameters {
                StatParameters::Distribution(s) => {
                    if s.sample_axis() == 0 {
                        mappings.x = Some(s.input.clone());
                        mappings.y = s.position.clone();
                    } else {
                        mappings.x = s.position.clone();
                        mappings.y = Some(s.input.clone());
                    }
                }
                StatParameters::Univariate(s) => {
                    if s.sample_axis() == 0 {
                        mappings.x = Some(s.input.clone());
                        mappings.y = s.second.clone();
                    } else {
                        mappings.y = Some(s.input.clone());
                    }
                }
                StatParameters::Summary(s) => {
                    mappings.x = s
                        .ggplot
                        .as_ref()
                        .and_then(|g| g.position.clone())
                        .or(mappings.x);
                    mappings.y = Some(s.input.clone());
                }
                StatParameters::Count(s) => {
                    mappings.x = s
                        .ggplot
                        .as_ref()
                        .and_then(|g| g.position.clone())
                        .or(mappings.x);
                }
                StatParameters::Bin(s) => mappings.x = Some(s.input.clone()),
                StatParameters::AutoBin(s) => mappings.x = Some(s.input.clone()),
                StatParameters::Ols(s) => {
                    mappings.x = Some(s.x.clone());
                    mappings.y = Some(s.y.clone());
                }
                StatParameters::Model(s) => {
                    mappings.x = Some(s.x.clone());
                    mappings.y = Some(s.y.clone());
                }
                StatParameters::Spatial(s) => {
                    mappings.x = Some(s.x.clone());
                    mappings.y = Some(s.y.clone());
                }
                _ => {}
            }
            let projections = super::scale_stage::layer_projections(layer, &panel.axes);
            let values = [mappings.x, mappings.y]
                .into_iter()
                .enumerate()
                .map(|(i, n)| {
                    n.map(|n| {
                        if matches!(n, Numeric::Scaled { .. } | Numeric::Category(_)) {
                            n
                        } else {
                            projections[i]
                                .as_ref()
                                .map_or_else(|| n.clone(), |p| p.mapping(n.clone()))
                        }
                    })
                })
                .collect::<Vec<_>>();
            let mut filters = layer.filters.clone();
            let mut input = layer.data;
            for _ in 0..=panel.transforms.len() {
                let DataRef::Transform(id) = input else { break };
                let node = panel
                    .transforms
                    .iter()
                    .find(|n| n.id == id)
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::MissingResource,
                            "Facet source transform is absent.",
                        )
                    })?;
                filters.extend(node.filters.clone());
                input = node.input;
            }
            for row in data.rows().filter(|row| {
                filters
                    .iter()
                    .all(|f| stats::filter_matches(*row, f) == Some(true))
                    && (layer.scope == StatScope::Chart
                        || layer.facet != FacetTarget::Match
                        || super::facets::row_matches(*row, &panel_scope))
            }) {
                charge(&mut remaining, 1, "facet raw positional row")?;
                let numbers = [
                    values[0].as_ref().and_then(|v| stats::number(row, v)),
                    values[1].as_ref().and_then(|v| stats::number(row, v)),
                ];
                if let Some(v) = numbers[0] {
                    Extent::include(&mut positioned.prepared.domains.x, v);
                }
                if let Some(v) = numbers[1] {
                    Extent::include(&mut positioned.prepared.domains.y, v);
                }
                positioned.raw_training.push(numbers);
            }
        }
    }
    Ok(())
}

pub(crate) fn train_facet_positioned_scopes(
    definition: &ChartDefinition,
    panel_definitions: &[ChartDefinition],
    scopes: &mut [PositionedScope],
    registry: &ExtensionRegistry,
    limits: CompileLimits,
) -> ChartResult<()> {
    let facets = definition.facets.as_ref().expect("facet training");
    let Some(first) = panel_definitions.first() else {
        return Ok(());
    };
    for horizontal in [true, false] {
        let groups = facets
            .order
            .iter()
            .map(|key| super::facet_policy::sharing_group(facets, key, horizontal))
            .collect::<Vec<_>>();
        for group in groups.iter().collect::<BTreeSet<_>>() {
            let mut shared = first.clone();
            shared.axes.retain(|a| a.side.horizontal() == horizontal);
            let mut empty = BTreeSet::new();
            let resolved = train_positioned_limits(
                &shared,
                &mut scopes
                    .iter_mut()
                    .enumerate()
                    .filter(|(i, _)| &groups[*i] == group)
                    .flat_map(|(_, s)| s.layers.iter_mut().map(|(l, p)| (&*l, p)))
                    .collect::<Vec<_>>(),
                registry,
                limits,
                &mut empty,
                false,
            )?;
            for (i, scope) in scopes.iter_mut().enumerate() {
                if &groups[i] == group {
                    scope.positional_limits.extend(resolved.clone());
                    scope.positional_empty.extend(empty.iter().copied());
                }
            }
        }
    }
    for axis in definition.axes.iter().filter(|a| {
        a.oob_function.is_some() && !matches!(a.scale, crate::layout::AxisScale::Binned { .. })
    }) {
        let identities = facets
            .order
            .iter()
            .map(|key| super::facet_policy::sharing_group(facets, key, axis.side.horizontal()))
            .collect::<Vec<_>>();
        let axes = scopes
            .iter()
            .zip(panel_definitions)
            .map(|(scope, panel)| {
                let mut axis = panel
                    .axes
                    .iter()
                    .find(|a| a.id == axis.id)
                    .expect("panel axis")
                    .clone();
                axis.resolved_limits = scope.positional_limits.get(&axis.id).cloned().map(Box::new);
                axis
            })
            .collect::<Vec<_>>();
        let ids = scopes
            .iter()
            .flat_map(|s| &s.layers)
            .map(|(l, _)| l.id)
            .collect::<BTreeSet<_>>();
        for id in ids {
            let layer = scopes
                .iter()
                .flat_map(|s| &s.layers)
                .find(|(l, _)| l.id == id)
                .expect("layer")
                .0
                .clone();
            let mut grouped = BTreeMap::<GroupValue, (_, Vec<_>)>::new();
            for (i, scope) in scopes.iter_mut().enumerate() {
                let entry = grouped
                    .entry(identities[i].clone())
                    .or_insert_with(|| (&axes[i], vec![]));
                entry.1.extend(
                    scope
                        .layers
                        .iter_mut()
                        .filter(|(l, _)| l.id == id)
                        .flat_map(|(_, p)| p.encoded.iter_mut()),
                );
            }
            let mut groups = grouped.into_values().collect::<Vec<_>>();
            super::positional_vectors::generated_populations(
                definition,
                &layer,
                &mut groups,
                registry,
            )?;
        }
    }
    Ok(())
}

impl PreparedScope {
    pub(crate) fn work_units(&self) -> usize {
        self.tables
            .values()
            .map(|t| t.work_units())
            .chain(
                self.graph
                    .layers
                    .values()
                    .map(|(_, o)| o.table.work_units()),
            )
            .sum()
    }
    pub(crate) fn layer_tables(&self) -> impl Iterator<Item = (&Layer, &PreparedTable)> {
        self.graph
            .layers
            .values()
            .map(|(l, o)| (l, o.table.as_ref()))
    }
}

/// One synchronous preparation route for typed-normalized authoring, recipes and layers.
/// A bounded graph cache retains each authored panel scope for the current source/graph.
/// New source or transform definitions release prior cache entries; prepared owners stay valid.
#[derive(Default)]
pub struct Compiler {
    pub(crate) extensions: Arc<ExtensionRegistry>,
    cache: BTreeMap<Option<PanelKey>, CachedGraph>,
    presentation: Option<(PreparedChart, CompileLimits)>,
    bin_cache: super::incremental_bins::BinCache,
}
impl Compiler {
    /// Empty compiler; no host services, threads or I/O are needed for data preparation.
    pub fn new() -> Self {
        Self::default()
    }
    /// Use explicitly supplied immutable extension implementations.
    pub fn with_extensions(extensions: Arc<ExtensionRegistry>) -> Self {
        Self {
            extensions,
            ..Self::default()
        }
    }
    /// Shared registry identity used by coherent capture and portable validation.
    pub fn extensions(&self) -> &Arc<ExtensionRegistry> {
        &self.extensions
    }
    /// Validate definition, source schemas, stages and registered parameters without running
    /// statistics, geometry generation or layout. Native-only extensions remain valid here;
    /// serialization and destinations apply their own capability checks.
    pub fn validate(
        &self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        limits: CompileLimits,
    ) -> ChartResult<()> {
        let captured = super::transform_resolution::resolve(definition, &self.extensions, false)?;
        let oriented = super::orientation::resolve(captured.as_ref())?;
        let resolved = semantics::resolve(
            oriented.as_ref(),
            source.get()?,
            limits,
            &self.extensions,
            false,
        )?;
        let definition = resolved.as_ref();
        self.extensions.validate_key_selections(definition, false)?;
        self.extensions
            .validate_coordinate_selection(definition, false)?;
        self.extensions
            .validate_facet_selection(definition, false)?;
        self.extensions.validate_guide_drawing(definition, false)?;
        let mut custom_ids = std::collections::BTreeSet::new();
        for guide in &definition.custom_legends {
            guide.validate(crate::Limits {
                max_items: limits.max_prepared_rows,
                max_path_commands: limits.max_vertices,
                ..Default::default()
            })?;
            if !custom_ids.insert(guide.id) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Duplicate custom guide identity.",
                ));
            }
        }
        for options in definition.legends.values() {
            options.validate()?;
        }
        if let Some(semantics) = &definition.semantics {
            semantics.validate()?;
        }
        let snapshot = source.get()?;
        crate::layout::validate_definition_axes(definition)?;
        if let Some(figure) = &definition.figure {
            figure.validate_references(definition)?;
        }
        validate_definition(definition, snapshot, limits, &self.extensions)?;
        if definition.facets.is_some() {
            let planned =
                super::facet_policy::resolve(definition, snapshot, limits, &self.extensions)?;
            facets::validate_facets(planned.as_ref(), snapshot, limits, &self.extensions)?;
        }
        Ok(())
    }
    /// Release cached graph/source ownership; existing prepared charts remain valid.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.presentation = None;
        self.bin_cache.clear();
    }
    /// Exact explicit-bin contribution work from the most recent preparation.
    pub fn update_metrics(&self) -> StatUpdateMetrics {
        self.bin_cache.metrics
    }
    /// Validate, filter/map, compute stats, bind outputs, position, collect domains and emit
    /// immutable data-space geometry. Scale/range/layout preparation follows in WP-06.
    pub fn prepare(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        limits: CompileLimits,
    ) -> ChartResult<PreparedChart> {
        let captured = super::transform_resolution::resolve(definition, &self.extensions, false)?;
        let definition = captured.as_ref();
        let staged = if super::expression_stage::has_expressions(definition) {
            self.validate(definition, source, limits)?;
            super::expression_stage::specialize(definition, source.get()?)?
        } else {
            std::borrow::Cow::Borrowed(definition)
        };
        let oriented = super::orientation::resolve(staged.as_ref())?;
        let resolved = semantics::resolve(
            oriented.as_ref(),
            source.get()?,
            limits,
            &self.extensions,
            true,
        )?;
        let palettes = super::palette_theme::resolve(resolved.as_ref())?;
        let mut result = self.prepare_resolved(palettes.as_ref(), source, state, limits)?;
        let mut retained = definition.clone();
        if definition
            .facets
            .as_ref()
            .and_then(|f| f.reference.as_ref())
            .is_some_and(|p| p.registered.is_some())
        {
            // Destination layout consumes the checked plan from this coherent source snapshot.
            retained.facets = result.definition.facets.clone();
        }
        result.definition = Arc::new(retained);
        Ok(result)
    }
    fn prepare_resolved(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        limits: CompileLimits,
    ) -> ChartResult<PreparedChart> {
        if let Some(semantics) = &definition.semantics {
            semantics.validate()?;
        }
        let snapshot = source.get()?;
        self.bin_cache.begin(limits.max_prepared_rows);
        if let Some((previous, old_limits)) = &self.presentation {
            let old = previous.definition();
            if !semantics::needs_panel_training(definition)
                && *old_limits == limits
                && previous
                    .source
                    .get()
                    .is_ok_and(|p| std::ptr::eq(p, snapshot))
                && old.mappings == definition.mappings
                && old.transforms == definition.transforms
                && old.layers == definition.layers
                && old.facets == definition.facets
            {
                validate_definition(definition, snapshot, limits, &self.extensions)?;
                let mut result = previous.clone();
                rebind_presentation(&mut result, &Arc::new(definition.clone()), state);
                self.presentation = Some((result.clone(), limits));
                return Ok(result);
            }
        }
        self.presentation = None;
        if self.cache.values().any(|c| {
            c.definitions != definition.transforms
                || c.semantics != definition.semantics
                || c.population_axes != population_axes(definition)
                || !c
                    .source
                    .get()
                    .is_ok_and(|old| old.epoch() == snapshot.epoch())
        }) {
            self.cache.clear();
        }
        let result = if definition.facets.is_some() {
            self.cache.retain(|key, _| {
                key.as_ref().is_some_and(|key| {
                    definition
                        .facets
                        .as_ref()
                        .is_some_and(|spec| spec.order.contains(key))
                })
            });
            facets::prepare_facets(self, definition, source, state, limits)
        } else {
            self.cache.retain(|key, _| key.is_none());
            if super::ggplot_bin_training::needed(definition) {
                let mut trained = definition.clone();
                super::ggplot_bin_training::resolve(
                    std::slice::from_mut(&mut trained),
                    snapshot,
                    limits,
                    None,
                )
                .and_then(|()| self.prepare_scoped(&trained, source, state, limits, None))
            } else {
                self.prepare_scoped(definition, source, state, limits, None)
            }
        };
        let result = match result {
            Ok(result) => result,
            Err(e) => {
                self.bin_cache.clear();
                return Err(e);
            }
        };
        self.bin_cache.finish();
        self.presentation = Some((result.clone(), limits));
        Ok(result)
    }
    pub(crate) fn prepare_scoped(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        limits: CompileLimits,
        scope: Option<&facets::PanelScope>,
    ) -> ChartResult<PreparedChart> {
        let population = self.prepare_scope(definition, source, limits, scope)?;
        let samples = super::colors::shared_samples(
            definition,
            source.get()?,
            population.layer_tables(),
            limits,
        )?;
        self.finish_scope(definition, source, state, limits, population, &samples)
    }
    pub(crate) fn prepare_scope(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        limits: CompileLimits,
        scope: Option<&facets::PanelScope>,
    ) -> ChartResult<PreparedScope> {
        if let Some(semantics) = &definition.semantics {
            semantics.validate()?;
        }
        let snapshot = source.get()?;
        let order = validate_definition(definition, snapshot, limits, &self.extensions)?;
        let cache_key = scope.map(|s| s.key.clone());
        let cached = self.cache.get(&cache_key).filter(|c| {
            c.scope.as_ref() == scope
                && c.semantics == definition.semantics
                && c.population_axes == population_axes(definition)
                && c.definitions == definition.transforms
                && c.limits == limits
        });
        let reusable = |c: &CachedGraph, output: &CachedOutput| {
            c.source
                .get()
                .and_then(|old| old.dataset(output.table.input.dataset))
                .ok()
                .zip(snapshot.dataset(output.table.input.dataset).ok())
                .is_some_and(|(old, new)| std::ptr::eq(old, new))
        };
        let mut metrics = PreparationMetrics::default();
        let mut remaining = limits.max_prepared_rows;
        let mut tables = BTreeMap::new();
        let mut outputs = BTreeMap::new();
        let mut diagnostics = vec![];
        for id in order {
            let node = &definition.transforms[id];
            if !facets::targeted(&node.facet, scope) {
                continue;
            }
            let output = if let Some(old) =
                cached.and_then(|c| c.outputs.get(&node.id).filter(|o| reusable(c, o)))
            {
                metrics.reused_transforms += 1;
                old.clone()
            } else {
                metrics.evaluated_transforms += 1;
                let input = resolve_input(
                    node.input,
                    snapshot,
                    &tables,
                    remaining,
                    scope,
                    &node.facet,
                    node.scope,
                )?;
                let data = snapshot.dataset(input.input.dataset)?;
                let input = facets::filter_panel(input, data, scope, &node.facet, node.scope)?;
                let statistic = facets::scoped_stat(&node.statistic, node.scope);
                let mut errors = vec![];
                let table = stats::run(
                    &self.extensions,
                    &mut self.bin_cache,
                    input,
                    data,
                    stats::StatRequest {
                        stat: &statistic,
                        population: node.scope,
                        panel: facets::population_panel(scope, &node.facet, node.scope),
                        filters: &node.filters,
                        policy: node.invalid,
                        scope: &facets::operation_scope(
                            "transform",
                            node.id.get(),
                            node.scope,
                            scope,
                        ),
                    },
                    CompileLimits {
                        max_prepared_rows: remaining,
                        ..limits
                    },
                    &mut errors,
                )
                .map_err(|e| context(e, data, None))?;
                for e in &mut errors {
                    *e = context(e.clone(), data, None);
                }
                CachedOutput {
                    table,
                    diagnostics: errors,
                }
            };
            charge(&mut remaining, output.table.work_units(), "prepared value")?;
            diagnostics.extend(output.diagnostics.clone());
            tables.insert(node.id, output.table.clone());
            outputs.insert(node.id, output);
        }
        let mut cached_layers = BTreeMap::new();
        for layer in &definition.layers {
            if !facets::targeted(&layer.facet, scope) {
                continue;
            }
            let output = if let Some((_, old)) = cached.and_then(|c| {
                c.layers
                    .get(&layer.id)
                    .filter(|(old, o)| same_population(old, layer) && reusable(c, o))
            }) {
                metrics.reused_layers += 1;
                old.clone()
            } else {
                metrics.evaluated_layers += 1;
                let input = resolve_input(
                    layer.data,
                    snapshot,
                    &tables,
                    remaining,
                    scope,
                    &layer.facet,
                    layer.scope,
                )?;
                let data = snapshot.dataset(input.input.dataset)?;
                let input = facets::filter_panel(input, data, scope, &layer.facet, layer.scope)?;
                let statistic = facets::scoped_stat(&layer.statistic, layer.scope);
                let mut errors = vec![];
                let table = stats::run(
                    &self.extensions,
                    &mut self.bin_cache,
                    input,
                    data,
                    stats::StatRequest {
                        stat: &statistic,
                        population: layer.scope,
                        panel: facets::population_panel(scope, &layer.facet, layer.scope),
                        filters: &layer.filters,
                        policy: layer.invalid,
                        scope: &facets::operation_scope(
                            "layer",
                            layer.id.get(),
                            layer.scope,
                            scope,
                        ),
                    },
                    CompileLimits {
                        max_prepared_rows: remaining,
                        ..limits
                    },
                    &mut errors,
                )
                .map_err(|e| context(e, data, Some(layer.id)))?;
                for e in &mut errors {
                    *e = context(e.clone(), data, Some(layer.id));
                }
                CachedOutput {
                    table,
                    diagnostics: errors,
                }
            };
            charge(&mut remaining, output.table.work_units(), "prepared value")?;
            diagnostics.extend(output.diagnostics.clone());
            cached_layers.insert(layer.id, (layer.clone(), output));
        }
        Ok(PreparedScope {
            graph: CachedGraph {
                semantics: definition.semantics.clone(),
                population_axes: population_axes(definition).to_vec(),
                scope: scope.cloned(),
                source: source.clone(),
                definitions: definition.transforms.clone(),
                limits,
                outputs,
                layers: cached_layers,
            },
            tables,
            diagnostics,
            metrics,
            encoded: BTreeMap::new(),
        })
    }
    pub(crate) fn finish_scope(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        limits: CompileLimits,
        mut population: PreparedScope,
        samples: &BTreeMap<crate::ScaleId, crate::scales::ScalePopulation>,
    ) -> ChartResult<PreparedChart> {
        self.encode_scope(definition, source, limits, &mut population)?;
        transform_generated_scopes(&mut [(definition, &mut population)])?;
        let mut positioned =
            self.position_scope(definition, source, state, limits, population, samples)?;
        positioned.positional_limits = train_positioned_limits(
            definition,
            &mut positioned
                .layers
                .iter_mut()
                .map(|(l, p)| (&*l, p))
                .collect::<Vec<_>>(),
            &self.extensions,
            limits,
            &mut positioned.positional_empty,
            true,
        )?;
        self.finish_positioned_scope(definition, source, state, limits, positioned, samples)
    }
    pub(crate) fn encode_scope(
        &self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        limits: CompileLimits,
        population: &mut PreparedScope,
    ) -> ChartResult<()> {
        let snapshot = source.get()?;
        for layer in &definition.layers {
            let Some((_, output)) = population.graph.layers.get(&layer.id) else {
                continue;
            };
            let data = snapshot.dataset(output.table.input.dataset)?;
            let encoded = encode_layer(
                layer,
                &output.table,
                data,
                &definition.mappings,
                definition.profile(),
                &self.extensions,
                limits,
            )
            .map_err(|e| context(e, data, Some(layer.id)))?;
            population.encoded.insert(layer.id, encoded);
        }
        Ok(())
    }
    pub(crate) fn position_scope(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        limits: CompileLimits,
        mut population: PreparedScope,
        samples: &BTreeMap<crate::ScaleId, crate::scales::ScalePopulation>,
    ) -> ChartResult<PositionedScope> {
        let snapshot = source.get()?;
        let mut budget = GeometryBudget {
            geography_coordinate: definition.coordinate.as_ref().and_then(|v| {
                if let CoordinateSpec::Geographic(v) = v {
                    Some(v)
                } else {
                    None
                }
            }),
            profile: definition.profile(),
            geometry_theme: definition.theme.as_ref().and_then(|t| t.geometry.as_ref()),
            population_axes: population_axes(definition),
            extensions: &self.extensions,
            limits,
            vertices: limits.max_vertices,
            color_domains: BTreeMap::new(),
            color_samples: samples,
        };
        budget.color_domains = super::colors::shared_catalogs(
            definition,
            snapshot,
            &population
                .graph
                .layers
                .iter()
                .map(|(id, (_, output))| (*id, output.table.as_ref()))
                .collect(),
            limits,
        )?;
        let mut positioned = Vec::new();
        for layer in &definition.layers {
            let Some((_, output)) = population.graph.layers.get(&layer.id) else {
                continue;
            };
            let table = output.table.clone();
            let data = snapshot.dataset(table.input.dataset)?;
            let prepared = position_layer(
                layer,
                table,
                data,
                population.encoded.remove(&layer.id).expect("encoded layer"),
                state,
                &mut budget,
            )
            .map_err(|e| context(e, data, Some(layer.id)))?;
            positioned.push((layer.clone(), prepared));
        }
        Ok(PositionedScope {
            population,
            layers: positioned,
            color_domains: budget.color_domains,
            vertices: budget.vertices,
            positional_limits: BTreeMap::new(),
            positional_empty: Default::default(),
        })
    }
    pub(crate) fn finish_positioned_scope(
        &mut self,
        definition: &ChartDefinition,
        source: &SnapshotHandle<StoreSnapshot>,
        state: &ChartState,
        limits: CompileLimits,
        positioned: PositionedScope,
        samples: &BTreeMap<crate::ScaleId, crate::scales::ScalePopulation>,
    ) -> ChartResult<PreparedChart> {
        let snapshot = source.get()?;
        let PositionedScope {
            population,
            layers: positioned,
            color_domains,
            vertices,
            positional_limits,
            positional_empty,
        } = positioned;
        let mut diagnostics = population.diagnostics;
        let mut layers = vec![];
        let mut colors = BTreeMap::new();
        let mut scale_domains = BTreeMap::new();
        let mut budget = GeometryBudget {
            geography_coordinate: definition.coordinate.as_ref().and_then(|v| {
                if let CoordinateSpec::Geographic(v) = v {
                    Some(v)
                } else {
                    None
                }
            }),
            profile: definition.profile(),
            geometry_theme: definition.theme.as_ref().and_then(|t| t.geometry.as_ref()),
            population_axes: population_axes(definition),
            extensions: &self.extensions,
            limits,
            vertices: vertices.min(limits.max_vertices),
            color_domains,
            color_samples: samples,
        };
        for (layer, positioned) in positioned {
            let data = snapshot.dataset(positioned.prepared.table.input.dataset)?;
            let start = diagnostics.len();
            let prepared = finish_layer(&layer, positioned, &mut budget, &mut diagnostics)
                .map_err(|e| context(e, data, Some(layer.id)))?;
            for e in &mut diagnostics[start..] {
                *e = context(e.clone(), data, Some(layer.id));
            }
            if layer.geom != Geom::Hierarchy {
                merge_named(
                    &mut scale_domains,
                    layer.scales,
                    &eligible_domains(&prepared),
                )
                .map_err(|e| context(e, data, Some(layer.id)))?;
            }
            if let Some(legend) = &prepared.color_legend {
                if colors.get(&legend.id).is_some_and(|old| old != legend) {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Layers sharing a color scale ID must resolve compatible domains and palettes; supply an explicit shared domain.",
                    ));
                }
                colors.insert(legend.id, legend.clone());
            }
            layers.push(prepared);
        }
        let mut domains = DomainContributions::default();
        if let Some(d) = scale_domains.get(&crate::ScaleId::new(0)) {
            domains.x = d.x;
            domains.x_space = d.x_space.clone();
        }
        if let Some(d) = scale_domains.get(&crate::ScaleId::new(1)) {
            domains.y = d.y;
            domains.y_space = d.y_space.clone();
        }
        let result = PreparedChart {
            positional_limits,
            positional_empty,
            scale_registrations: self.extensions.scales.clone(),
            key_registrations: self.extensions.keys.clone(),
            coordinate_registrations: self.extensions.coordinates.clone(),
            guide_drawing: self.extensions.guide_drawing.clone(),
            palette_registrations: self.extensions.palette_function.clone(),
            break_registrations: self.extensions.breaks_function.clone(),
            guide_registrations: self.extensions.guides.clone(),
            panels: vec![],
            shared_training: None,
            definition: Arc::new(definition.clone()),
            source: source.clone(),
            state: state.clone(),
            layers,
            transforms: population.tables,
            domains,
            scale_domains,
            diagnostics,
            metrics: population.metrics,
        };
        self.cache.insert(
            population.graph.scope.as_ref().map(|s| s.key.clone()),
            population.graph,
        );
        Ok(result)
    }
}
fn population_axes(definition: &ChartDefinition) -> &[crate::layout::AxisSpec] {
    if definition
        .semantics
        .as_ref()
        .is_some_and(|s| s.scale_stage == ScaleStage::BeforeStatistics)
    {
        &definition.axes
    } else {
        &[]
    }
}
fn context(mut e: Diagnostic, data: &DatasetSnapshot, layer: Option<LayerId>) -> Diagnostic {
    e.context.dataset = Some(data.version().dataset);
    e.context.dataset_revision = Some(data.version().revision);
    e.context.schema_version = Some(data.version().schema_version);
    e.context.layer = layer;
    e
}
pub(super) fn charge(remaining: &mut usize, count: usize, kind: &str) -> ChartResult<()> {
    *remaining = remaining.checked_sub(count).ok_or_else(|| {
        error(
            DiagnosticCode::ResourceLimit,
            format!("Compiler {kind} budget exceeded."),
        )
    })?;
    Ok(())
}
fn resolve_input(
    input: DataRef,
    snapshot: &StoreSnapshot,
    tables: &BTreeMap<TransformId, Arc<PreparedTable>>,
    remaining: usize,
    scope: Option<&facets::PanelScope>,
    target: &FacetTarget,
    stat: StatScope,
) -> ChartResult<Arc<PreparedTable>> {
    match input {
        DataRef::Dataset(id) => {
            facets::source_table(snapshot.dataset(id)?, remaining, scope, target, stat)
        }
        DataRef::Transform(id) => tables.get(&id).cloned().ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("Transform {} is unavailable.", id.get()),
            )
        }),
    }
}
pub(super) fn validate_definition(
    definition: &ChartDefinition,
    snapshot: &StoreSnapshot,
    limits: CompileLimits,
    extensions: &ExtensionRegistry,
) -> ChartResult<Vec<usize>> {
    extensions.validate_hierarchy_selections(definition, false)?;
    extensions.validate_scale_selections(definition, false)?;
    extensions.validate_guide_selections(definition, false)?;
    extensions.validate_interpolation_selections(definition, false)?;
    if let Some(theme) = &definition.theme {
        theme.resolve(&crate::theme::ThemePatch::<crate::scene::Color>::default())?;
        if theme
            .layers
            .keys()
            .any(|id| !definition.layers.iter().any(|l| &l.id == id))
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Theme names an absent layer.",
            ));
        }
    }
    if let Some(figure) = &definition.figure {
        figure.validate(crate::Limits::default())?;
    }
    if definition.layers.len() > limits.max_layers
        || definition.transforms.len() > limits.max_transforms
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Definition layer/transform count budget exceeded.",
        ));
    }
    let mut layers = BTreeSet::new();
    let mut nodes = BTreeMap::new();
    for (i, node) in definition.transforms.iter().enumerate() {
        if nodes.insert(node.id, i).is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Transform IDs must be unique.",
            ));
        }
        stats::validate_stat(&node.statistic, limits)?;
        if let StatParameters::Model(spec) = &node.statistic.parameters {
            super::model_extensions::validate(extensions, &spec.options.method, false)?;
        }
        if let StatParameters::Univariate(spec) = &node.statistic.parameters {
            super::univariate_stage::validate_registry(spec, extensions, false)?;
        }
        if matches!(node.statistic.parameters, StatParameters::Custom(_)) {
            extensions.stat_descriptor(&node.statistic.operation)?;
        }
        stats::validate_filters(&node.filters, limits)?;
    }
    for layer in &definition.layers {
        super::text_geom::validate_layer(layer)?;
        super::row_annotation::validate_layer(layer, limits.max_vertices)?;
        super::recipe_emit::validate(layer)?;
        super::shape_encoding::validate(layer)?;
        match layer.geom {
            Geom::ShapeLine { curve, .. }
            | Geom::ShapeLineRadial { curve, .. }
            | Geom::ShapeLink { curve } => curve.validate()?,
            Geom::ShapeArea { curve, .. } | Geom::ShapeAreaRadial { curve, .. } => {
                crate::shape::Area::new().curve(curve)?;
            }
            _ => {}
        }
        if layer.candle_colors.is_some() && !matches!(layer.geom, Geom::Ohlc { .. }) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Candle colors require supplied OHLC geometry.",
            ));
        }
        super::shape_extensions::resolve_layer(layer, extensions)?;
        if let Some(g) = &layer.geometry_extension {
            extensions::parameter_size(&g.parameters)?;
            extensions
                .geom(&g.operation)?
                .validate(layer, &g.parameters)?;
        }
        if !layers.insert(layer.id) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Layer IDs must be unique.",
            ));
        }
        if let Geom::Bar { width, .. } | Geom::Ohlc { width } = layer.geom
            && (!width.is_finite() || width <= 0.)
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Bar/candle width must be finite and positive.",
            ));
        }
        stats::validate_stat(&layer.statistic, limits)?;
        if let StatParameters::Model(spec) = &layer.statistic.parameters {
            super::model_extensions::validate(extensions, &spec.options.method, false)?;
        }
        if let StatParameters::Univariate(spec) = &layer.statistic.parameters {
            super::univariate_stage::validate_registry(spec, extensions, false)?;
        }
        if matches!(layer.statistic.parameters, StatParameters::Custom(_)) {
            extensions.stat_descriptor(&layer.statistic.operation)?;
        }
        stats::validate_filters(&layer.filters, limits)?;
        if layer
            .style
            .alpha
            .is_some_and(|v| !v.is_finite() || !(0. ..=1.).contains(&v))
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Alpha must lie in the closed unit interval.",
            ));
        }
        let reference_line =
            definition.profile() == Profile::Ggplot2_4_0_3 && layer.reference_linewidth();
        if let Some(line_type) = layer.style.line_type {
            line_type.pattern(if reference_line && layer.style.stroke_width == 0. {
                1.
            } else {
                layer.style.stroke_width
            })?;
        }
        for (channel, value) in &layer.aesthetic_values {
            channel.validate(value)?;
        }
        let reference_point = definition.profile() == Profile::Ggplot2_4_0_3
            && layer.reference_point()
            && !layer
                .grammar
                .as_ref()
                .is_some_and(|g| g.default_radius == Some(false));
        if !layer.style.radius.is_finite()
            || (!reference_point && layer.style.radius < 0.)
            || (!reference_point && layer.style.radius == 0.)
            || !layer.style.stroke_width.is_finite()
            || layer.style.stroke_width < 0.
            || (!reference_point && !reference_line && layer.style.stroke_width == 0.)
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Constant radii and stroke widths must be finite and positive; reference point sizes may be nonpositive and reference stroke widths may be zero.",
            ));
        }
    }
    for input in definition
        .transforms
        .iter()
        .map(|t| t.input)
        .chain(definition.layers.iter().map(|l| l.data))
    {
        match input {
            DataRef::Dataset(id) => {
                snapshot.dataset(id)?;
            }
            DataRef::Transform(id) if !nodes.contains_key(&id) => {
                return Err(error(
                    DiagnosticCode::MissingResource,
                    format!("Unknown transform {}.", id.get()),
                ));
            }
            _ => {}
        }
    }
    // Iterative topological order avoids call-stack dependence on user graph depth.
    let mut visited = BTreeSet::new();
    let mut order = vec![];
    while order.len() < nodes.len() {
        let before = order.len();
        for (i, node) in definition.transforms.iter().enumerate() {
            if visited.contains(&node.id) {
                continue;
            }
            if matches!(node.input, DataRef::Dataset(_))
                || matches!(node.input,DataRef::Transform(id) if visited.contains(&id))
            {
                visited.insert(node.id);
                order.push(i);
            }
        }
        if order.len() == before {
            return Err(error(
                DiagnosticCode::Validation,
                "Transform dependency cycle: layout/viewport feedback and cyclic statistical inputs are unsupported.",
            ));
        }
    }
    preflight_schemas(definition, snapshot, &order, limits, extensions)?;
    Ok(order)
}

#[derive(Clone)]
pub(super) struct EncodedRow {
    pub(super) geo_feature: Option<usize>,
    pub(super) stat_outliers: Vec<StatOutlier>,
    pub(super) outlier_anchor_y: Option<f64>,
    pub(super) recipe_values: BTreeMap<RecipeAesthetic, crate::interpolate::Value>,
    pub(super) missing_aesthetics: u8,
    pub(super) values: BTreeMap<ValueAesthetic, crate::interpolate::Value>,
    pub(super) shape: Option<Box<super::shape_encoding::ShapeRow>>,
    pub(super) x: Option<f64>,
    pub(super) y: Option<f64>,
    pub(super) x2: Option<f64>,
    pub(super) y2: Option<f64>,
    pub(super) color: Option<crate::color::Paint>,
    pub(super) fill: Option<crate::color::Paint>,
    pub(super) stroke: Option<crate::color::Paint>,
    pub(super) opacity: Option<f64>,
    pub(super) alpha: Option<f64>,
    pub(super) stroke_width: Option<f64>,
    pub(super) low: Option<f64>,
    pub(super) high: Option<f64>,
    pub(super) size: Option<f64>,
    pub(super) group: Option<GroupValue>,
    pub(super) ordinal: u64,
    pub(super) target: Target,
    pub(super) key: Option<RowKey>,
}
impl EncodedRow {
    pub(super) fn is_missing(&self, aesthetic: AfterScaleAesthetic) -> bool {
        self.missing_aesthetics & (1 << aesthetic as u8) != 0
    }
    pub(super) fn set_missing(&mut self, aesthetic: AfterScaleAesthetic, missing: bool) {
        let mask = 1 << aesthetic as u8;
        if missing {
            self.missing_aesthetics |= mask;
        } else {
            self.missing_aesthetics &= !mask;
        }
    }
}
fn coordinate_space(
    data: &DatasetSnapshot,
    value: &Numeric,
    profile: Profile,
) -> ChartResult<ValueSpace> {
    if let Numeric::Category(id) = value {
        return data
            .categories(*id)
            .map(|v| {
                if profile == Profile::Ggplot2_4_0_3
                    && data.rows().any(|row| row.value(*id).is_none())
                {
                    ValueSpace::NullableCategorical {
                        categories: v
                            .iter()
                            .cloned()
                            .map(Some)
                            .chain(std::iter::once(None))
                            .collect(),
                    }
                } else {
                    ValueSpace::Categorical {
                        categories: v.to_vec(),
                    }
                }
            })
            .ok_or_else(|| {
                error(
                    DiagnosticCode::SchemaConflict,
                    "Band encoding requires a categorical source field.",
                )
            });
    }
    numeric_space(data, value)
}
fn source_space(
    data: &DatasetSnapshot,
    value: &Numeric,
    profile: Profile,
) -> ChartResult<Option<ValueSpace>> {
    let space = coordinate_space(data, value, profile)?;
    Ok(if matches!(value, Numeric::Literal(_)) {
        None
    } else {
        Some(space)
    })
}
fn bin_space(space: &ValueSpace, value: &BinNumeric) -> ChartResult<Option<ValueSpace>> {
    match value {
        BinNumeric::Expression(expr) => {
            numeric_expression_type(expr, |_| Ok(ExpressionType::Number))?;
            Ok(Some(ValueSpace::Data))
        }
        BinNumeric::Literal(v) if !v.is_finite() => Err(error(
            DiagnosticCode::NumericalDomain,
            "Generated literal mappings must be finite.",
        )),
        BinNumeric::Literal(_) => Ok(None),
        BinNumeric::Field(
            BinField::Count
            | BinField::Width
            | BinField::Density
            | BinField::NCount
            | BinField::NDensity,
        ) => Ok(Some(ValueSpace::Data)),
        BinNumeric::Field(_) => Ok(Some(space.clone())),
    }
}
pub(super) fn bin_number(row: &BinnedRow, value: &BinNumeric) -> Option<f64> {
    match value {
        BinNumeric::Literal(v) => Some(*v),
        BinNumeric::Field(BinField::Start) => Some(row.start),
        BinNumeric::Field(BinField::End) => Some(row.end),
        BinNumeric::Field(BinField::Midpoint) => Some(row.start.midpoint(row.end)),
        BinNumeric::Field(BinField::Width) => Some(row.end - row.start),
        BinNumeric::Field(BinField::Density) => row.statistics.as_ref().and_then(|s| s.density),
        BinNumeric::Field(BinField::NCount) => row.statistics.as_ref().and_then(|s| s.ncount),
        BinNumeric::Field(BinField::NDensity) => row.statistics.as_ref().and_then(|s| s.ndensity),
        BinNumeric::Field(BinField::Count) if row.statistics.is_some() => {
            row.statistics.as_ref().map(|s| s.count)
        }
        BinNumeric::Field(BinField::Count) if row.count <= 1_u64 << 53 => Some(row.count as f64),
        _ => None,
    }
    .filter(|v| v.is_finite())
}
fn merge_space(a: &mut Option<ValueSpace>, b: &Option<ValueSpace>) -> ChartResult<()> {
    if let Some(space) = b {
        if a.as_ref().is_some_and(|prior| prior != space) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Incompatible calculation spaces or timestamp origins share an axis; use compatible mappings or independently named scales.",
            ));
        }
        *a = Some(space.clone());
    }
    Ok(())
}
// Train categorical membership from eligible post-stat geometry, ordered by the retained
// source catalog. Keep the layer catalog unchanged so geometry ordinals still decode exactly.
pub(super) fn eligible_domains(layer: &PreparedLayer) -> DomainContributions {
    let mut d = layer.domains.clone();
    for horizontal in [true, false] {
        let space = if horizontal {
            &mut d.x_space
        } else {
            &mut d.y_space
        };
        if space.as_ref().is_some_and(ValueSpace::is_categorical) {
            let mut used = layer
                .unpainted_categories
                .as_ref()
                .map_or_else(BTreeSet::new, |categories| {
                    categories[usize::from(!horizontal)].clone()
                });
            let mut include = |x: f64, y: f64| {
                let value = if horizontal { x } else { y };
                if value.is_finite() {
                    used.insert(value as usize);
                }
            };
            for mark in layer.marks.iter() {
                match &mark.geometry {
                    PreparedGeometry::Recipe(recipe) => {
                        for p in recipe.points() {
                            include(p.x(), p.y());
                        }
                    }
                    PreparedGeometry::UnboundedPoint(p) => {
                        include(p[0].0, p[1].0);
                    }
                    PreparedGeometry::HierarchyNode(_) | PreparedGeometry::HierarchyLink { .. } => {
                    }
                    PreparedGeometry::Point(p)
                    | PreparedGeometry::ShapePath { center: p, .. }
                    | PreparedGeometry::ShapePathRun { center: p, .. } => include(p.x(), p.y()),
                    PreparedGeometry::BandRun { lower, upper }
                    | PreparedGeometry::StackBandRun { lower, upper, .. } => {
                        for p in lower.iter().chain(upper) {
                            include(p.x(), p.y());
                        }
                    }
                    PreparedGeometry::LineRun(points) | PreparedGeometry::Polygon(points) => {
                        for p in points {
                            include(p.x(), p.y());
                        }
                    }
                    PreparedGeometry::Rule { from, to }
                    | PreparedGeometry::Rectangle { from, to }
                    | PreparedGeometry::Bar { from, to, .. }
                    | PreparedGeometry::NativePaint { from, to, .. } => {
                        include(from.x(), from.y());
                        include(to.x(), to.y());
                    }
                }
            }
            match space.as_mut().unwrap() {
                ValueSpace::Categorical { categories } => {
                    let mut i = 0;
                    categories.retain(|_| {
                        let keep = used.contains(&i);
                        i += 1;
                        keep
                    });
                }
                ValueSpace::NullableCategorical { categories } => {
                    let mut i = 0;
                    categories.retain(|_| {
                        let keep = used.contains(&i);
                        i += 1;
                        keep
                    });
                }
                _ => unreachable!(),
            }
        }
    }
    d
}
pub(super) fn merge_named(
    all: &mut BTreeMap<crate::ScaleId, DomainContributions>,
    bindings: ScaleBindings,
    d: &DomainContributions,
) -> ChartResult<()> {
    for (id, horizontal) in [(bindings.x, true), (bindings.y, false)] {
        merge_axis(all, id, horizontal, d)?;
    }
    Ok(())
}
pub(super) fn merge_axis(
    all: &mut BTreeMap<crate::ScaleId, DomainContributions>,
    id: crate::ScaleId,
    horizontal: bool,
    d: &DomainContributions,
) -> ChartResult<()> {
    // A train-only layer may omit either positional aesthetic. Absence contributes
    // no space; inventing a numeric space here conflicts with categorical peers.
    if if horizontal {
        d.x_space.is_none() && d.x.is_none()
    } else {
        d.y_space.is_none() && d.y.is_none()
    } {
        return Ok(());
    }
    let a = all.entry(id).or_default();
    if (horizontal && a.y_space.is_some()) || (!horizontal && a.x_space.is_some()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "A named scale cannot serve both x and y.",
        ));
    }
    let b = if horizontal {
        DomainContributions {
            x: d.x,
            x_space: Some(d.x_space.clone().unwrap_or(ValueSpace::Data)),
            ..Default::default()
        }
    } else {
        DomainContributions {
            y: d.y,
            y_space: Some(d.y_space.clone().unwrap_or(ValueSpace::Data)),
            ..Default::default()
        }
    };
    // Union layer catalogs only here: each layer retains its own ordinal-to-label mapping.
    let (prior, next) = if horizontal {
        (&mut a.x_space, &b.x_space)
    } else {
        (&mut a.y_space, &b.y_space)
    };
    let nullable = matches!(prior, Some(ValueSpace::NullableCategorical { .. }))
        || matches!(next, Some(ValueSpace::NullableCategorical { .. }));
    if nullable
        && prior.as_ref().is_some_and(ValueSpace::is_categorical)
        && next.as_ref().is_some_and(ValueSpace::is_categorical)
    {
        let keys = |space: &ValueSpace| match space {
            ValueSpace::Categorical { categories } => {
                categories.iter().cloned().map(Some).collect::<Vec<_>>()
            }
            ValueSpace::NullableCategorical { categories } => categories.clone(),
            _ => unreachable!(),
        };
        let mut p = keys(prior.as_ref().unwrap());
        let mut seen: BTreeSet<_> = p.iter().cloned().collect();
        p.extend(
            keys(next.as_ref().unwrap())
                .into_iter()
                .filter(|key| seen.insert(key.clone())),
        );
        *prior = Some(ValueSpace::NullableCategorical { categories: p });
        let extent = if horizontal { &mut a.x } else { &mut a.y };
        if let Some(e) = if horizontal { b.x } else { b.y } {
            Extent::include(extent, e.minimum);
            Extent::include(extent, e.maximum);
        }
    } else if let (
        Some(ValueSpace::Categorical { categories: p }),
        Some(ValueSpace::Categorical { categories: n }),
    ) = (prior, next)
    {
        let mut seen: BTreeSet<String> = p.iter().cloned().collect();
        p.extend(n.iter().filter(|v| seen.insert((*v).clone())).cloned());
        let extent = if horizontal { &mut a.x } else { &mut a.y };
        if let Some(e) = if horizontal { b.x } else { b.y } {
            Extent::include(extent, e.minimum);
            Extent::include(extent, e.maximum);
        }
    } else {
        merge_domains(a, &b)?;
    }

    Ok(())
}

fn merge_domains(a: &mut DomainContributions, b: &DomainContributions) -> ChartResult<()> {
    merge_space(&mut a.x_space, &b.x_space)?;
    merge_space(&mut a.y_space, &b.y_space)?;
    if let Some(v) = b.x {
        Extent::include(&mut a.x, v.minimum);
        Extent::include(&mut a.x, v.maximum);
    }
    if let Some(v) = b.y {
        Extent::include(&mut a.y, v.minimum);
        Extent::include(&mut a.y, v.maximum);
    }
    Ok(())
}
pub(super) fn source_binding(
    layer: &Layer,
    authored: &SourceAes,
    inherited: &SourceAes,
    data: &DatasetSnapshot,
    profile: Profile,
) -> ChartResult<(SourceAes, DomainContributions)> {
    let endpoints = matches!(
        layer.geom,
        Geom::Rule | Geom::ShapeLink { .. } | Geom::Rectangle | Geom::ShapeArea { .. }
    );
    let mut domains = DomainContributions::default();
    let aes = if layer.inherit {
        authored.inherit(inherited)
    } else {
        authored.clone()
    };
    if layer.geom == Geom::Hierarchy || layer.hierarchy.is_some() {
        super::hierarchy::validate(layer, data, inherited)?;
        if let Some(group) = &aes.grouping {
            validate_group(data, group)?;
        }
        if let Some(group) = aes.group {
            validate_group(data, &Grouping::Field(group))?;
        }
        return Ok((aes, DomainContributions::default()));
    }
    if layer.geom != Geom::Blank
        && layer.geography.is_none()
        && !matches!(layer.recipe, Some(BuiltinRecipe::Rug(_)))
        && (aes.x.is_none()
            || (aes.y.is_none() && !matches!(layer.recipe, Some(BuiltinRecipe::Interval(_)))))
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Geometry requires x and y source mappings.",
        ));
    }
    if layer.recipe.is_none()
        && layer.geography.is_none()
        && endpoints
        && (aes.x2.is_none() || aes.y2.is_none())
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Rules/rectangles require both second endpoints; baselines must be explicit.",
        ));
    }
    if matches!(layer.geom, Geom::Bar { .. } | Geom::Ohlc { .. }) && aes.y2.is_none() {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Bars/candles require an explicit y2 baseline/close.",
        ));
    }
    if matches!(layer.geom, Geom::Ohlc { .. }) && (aes.low.is_none() || aes.high.is_none()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "OHLC requires low and high numeric bounds.",
        ));
    }
    // Validate even unused inherited fields so unrelated schemas require explicit overrides.
    for value in [&aes.x, &aes.y, &aes.x2, &aes.y2, &aes.low, &aes.high]
        .into_iter()
        .flatten()
    {
        coordinate_space(data, value, profile)?;
    }
    if let Some(size) = &aes.size {
        numeric_space(data, size)?;
    }
    let grouping = aes
        .grouping
        .clone()
        .unwrap_or_else(|| aes.group.map_or(Grouping::All, Grouping::Field));
    validate_group(data, &grouping)?;
    domains.x_space = aes
        .x
        .as_ref()
        .map(|v| source_space(data, v, profile))
        .transpose()?
        .flatten();
    domains.y_space = aes
        .y
        .as_ref()
        .map(|v| source_space(data, v, profile))
        .transpose()?
        .flatten();
    if endpoints || layer.geom == Geom::Blank {
        if let Some(value) = &aes.x2 {
            merge_space(&mut domains.x_space, &source_space(data, value, profile)?)?;
        }
        if let Some(value) = &aes.y2 {
            merge_space(&mut domains.y_space, &source_space(data, value, profile)?)?;
        }
    }
    if matches!(
        layer.geom,
        Geom::Ribbon { .. } | Geom::Bar { .. } | Geom::Ohlc { .. }
    ) {
        let y2 = aes.y2.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Ribbon requires a y2 upper-bound mapping.",
            )
        })?;
        merge_space(&mut domains.y_space, &source_space(data, y2, profile)?)?;
    }
    for bound in [&aes.low, &aes.high].into_iter().flatten() {
        merge_space(&mut domains.y_space, &source_space(data, bound, profile)?)?;
    }
    if matches!(layer.geom, Geom::Ohlc { .. } | Geom::Bar { .. })
        && matches!(
            domains.y_space,
            Some(
                ValueSpace::Categorical { .. }
                    | ValueSpace::NullableCategorical { .. }
                    | ValueSpace::Timestamp { .. }
            )
        )
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Bar/OHLC values must be numeric.",
        ));
    }
    validate_line_size(layer, aes.size.is_some())?;
    Ok((aes, domains))
}
pub(super) fn bin_binding(
    layer: &Layer,
    aes: &BinAes,
    space: &ValueSpace,
) -> ChartResult<DomainContributions> {
    if matches!(layer.geom, Geom::Ohlc { .. }) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "OHLC requires source open/close/low/high mappings.",
        ));
    }
    let endpoints = matches!(
        layer.geom,
        Geom::Rule | Geom::ShapeLink { .. } | Geom::Rectangle | Geom::ShapeArea { .. }
    );
    let mut domains = DomainContributions::default();
    if layer.recipe.is_none() && endpoints && (aes.x2.is_none() || aes.y2.is_none()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Generated rule/rectangle mappings require both endpoints.",
        ));
    }
    domains.x_space = bin_space(space, &aes.x)?;
    domains.y_space = bin_space(space, &aes.y)?;
    for value in [&aes.x2, &aes.y2, &aes.size].into_iter().flatten() {
        bin_space(space, value)?;
    }
    if endpoints {
        if let Some(value) = &aes.x2 {
            merge_space(&mut domains.x_space, &bin_space(space, value)?)?;
        }
        if let Some(value) = &aes.y2 {
            merge_space(&mut domains.y_space, &bin_space(space, value)?)?;
        }
    }
    if matches!(layer.geom, Geom::Ribbon { .. } | Geom::Bar { .. }) {
        let y2 = aes.y2.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Ribbon requires a generated y2 upper-bound mapping.",
            )
        })?;
        merge_space(&mut domains.y_space, &bin_space(space, y2)?)?;
    }
    validate_line_size(layer, aes.size.is_some())?;
    Ok(domains)
}
fn validate_line_size(layer: &Layer, mapped: bool) -> ChartResult<()> {
    super::after_scale::validate(layer)?;
    if (layer.geom.run().is_some() || matches!(layer.geom, Geom::Rectangle)) && mapped {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Mapped size currently supports point radius and rule stroke width; use constant styling for lines and rectangles.",
        ));
    }
    Ok(())
}

pub(super) struct GeometryBudget<'a> {
    pub(super) geography_coordinate: Option<&'a GeographicCoordinate>,
    pub(super) profile: Profile,
    pub(super) geometry_theme: Option<&'a crate::theme::GeometryTheme<crate::color::Paint>>,
    pub(super) population_axes: &'a [crate::layout::AxisSpec],
    pub(super) extensions: &'a Arc<ExtensionRegistry>,
    pub(super) limits: CompileLimits,
    pub(super) vertices: usize,
    pub(super) color_domains: BTreeMap<crate::ScaleId, Vec<String>>,
    pub(super) color_samples: &'a BTreeMap<crate::ScaleId, crate::scales::ScalePopulation>,
}
/// Encoded rows after statistics and positions, before final scale mapping and
/// geometry. Retaining this stage lets shared scales train across all layers once.
pub(super) struct PositionedLayer {
    pub(super) raw_training: Vec<[Option<f64>; 2]>,
    pub(super) prepared: PreparedLayer,
    pub(super) encoded: Vec<EncodedRow>,
    pub(super) mapped_size: bool,
    pub(super) area_size: bool,
    pub(super) stack: Option<super::stack_position::StackLayout>,
}
pub(super) struct EncodedLayer {
    pub(super) domains: DomainContributions,
    pub(super) encoded: Vec<EncodedRow>,
    pub(super) hierarchy: Option<Arc<super::PreparedHierarchy>>,
    pub(super) mapped_size: bool,
}
/// Resolve independent paint, alpha and linewidth once for every geometry consumer.
pub(super) fn row_style(layer: &Layer, row: &EncodedRow) -> ChartResult<Style> {
    let alpha = layer.style.alpha.or(row.alpha);
    let resolve = |paint| {
        super::numeric_aesthetics::apply_alpha(
            super::numeric_aesthetics::apply_opacity(paint, row.opacity),
            alpha,
        )
    };
    let default_fill = matches!(row.values.get(&ValueAesthetic::Shape), Some(crate::interpolate::Value::Number(crate::interpolate::Number(code))) if (21. ..=25.).contains(code))
        || matches!(
            row.shape.as_deref().and_then(|s| s.symbol).map(|s| s.0),
            Some(crate::shape::SymbolKind::Ggplot(21..=25))
        )
        || matches!(
            layer.geom,
            Geom::ShapeSymbol {
                kind: crate::shape::SymbolKind::Ggplot(21..=25),
                ..
            }
        );
    let mut style = Style {
        color: resolve(row.color.unwrap_or(layer.style.color)),
        fill: layer.style.fill.or(row.fill).map(resolve).or_else(|| {
            default_fill.then_some(crate::scene::Color {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            })
        }),
        stroke: layer.style.stroke.or(row.stroke).map(resolve),
        line_type: layer.style.line_type.or(row
            .values
            .get(&ValueAesthetic::LineType)
            .map(super::style_channels::line_type)
            .transpose()?),
        stroke_width: row.stroke_width.unwrap_or(layer.style.stroke_width),
        ..layer.style.resolve()
    };
    if matches!(
        layer.recipe,
        Some(super::BuiltinRecipe::Density(_) | super::BuiltinRecipe::Smooth)
    ) {
        style.color = super::numeric_aesthetics::apply_opacity(
            row.color.unwrap_or(layer.style.color),
            row.opacity,
        );
        style.stroke = Some(super::numeric_aesthetics::apply_opacity(
            layer
                .style
                .stroke
                .or(row.stroke)
                .unwrap_or(row.color.unwrap_or(layer.style.color)),
            row.opacity,
        ));
        style.alpha = None;
    }
    Ok(style)
}
pub(super) fn run_style<'a>(
    layer: &Layer,
    rows: impl Iterator<Item = &'a EncodedRow>,
) -> ChartResult<Style> {
    let mut result: Option<Style> = None;
    for row in rows.filter(|r| r.x.is_some() && r.y.is_some()) {
        let style = row_style(layer, row)?;
        let same = |old: Style| {
            if matches!(layer.geom, Geom::Line { .. } | Geom::ShapeLine { .. }) {
                old.stroke.unwrap_or(old.color) == style.stroke.unwrap_or(style.color)
                    && old.stroke_width == style.stroke_width
                    && old.line_type == style.line_type
                    && old.units == style.units
            } else {
                old == style
            }
        };
        if result.is_some_and(|old| !same(old)) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Line and filled runs require constant styling within each group.",
            ));
        }
        result = Some(style);
    }
    Ok(result.unwrap_or_else(|| layer.style.resolve()))
}

pub(super) fn include_geometry(domains: &mut DomainContributions, geometry: &PreparedGeometry) {
    let mut include = |p: Point| {
        Extent::include(&mut domains.x, p.x());
        Extent::include(&mut domains.y, p.y());
    };
    match geometry {
        PreparedGeometry::Recipe(recipe) => {
            for p in recipe.points() {
                Extent::include(&mut domains.x, p.x());
                Extent::include(&mut domains.y, p.y());
            }
        }
        PreparedGeometry::UnboundedPoint(p) => {
            if p[0].0.is_finite() {
                Extent::include(&mut domains.x, p[0].0);
            }
            if p[1].0.is_finite() {
                Extent::include(&mut domains.y, p[1].0);
            }
        }
        PreparedGeometry::HierarchyNode(_) | PreparedGeometry::HierarchyLink { .. } => {}
        PreparedGeometry::Point(p)
        | PreparedGeometry::ShapePath { center: p, .. }
        | PreparedGeometry::ShapePathRun { center: p, .. } => include(*p),
        PreparedGeometry::BandRun { lower, upper }
        | PreparedGeometry::StackBandRun { lower, upper, .. } => {
            for p in lower.iter().chain(upper) {
                include(*p);
            }
        }
        PreparedGeometry::LineRun(points) | PreparedGeometry::Polygon(points) => {
            for p in points {
                include(*p);
            }
        }
        PreparedGeometry::Rule { from, to } => {
            include(*from);
            include(*to);
        }
        PreparedGeometry::Rectangle { from, to }
        | PreparedGeometry::Bar { from, to, .. }
        | PreparedGeometry::NativePaint { from, to, .. } => {
            include(*from);
            include(*to);
        }
    }
}

#[derive(Clone)]
struct OutputShape {
    dataset: crate::DatasetId,
    bins: Option<ValueSpace>,
    statistical: Option<Vec<StatColumn>>,
}
fn preflight_schemas(
    definition: &ChartDefinition,
    snapshot: &StoreSnapshot,
    order: &[usize],
    limits: CompileLimits,
    extensions: &ExtensionRegistry,
) -> ChartResult<()> {
    let mut shapes = BTreeMap::<TransformId, OutputShape>::new();
    for &i in order {
        let node = &definition.transforms[i];
        let input = input_shape(node.input, &shapes)?;
        let data = snapshot.dataset(input.dataset)?;
        let output = stat_shape(
            input,
            data,
            &facets::scoped_stat(&node.statistic, node.scope),
            &node.filters,
            limits,
            extensions,
        )
        .map_err(|e| context(e, data, None))?;
        shapes.insert(node.id, output);
    }
    let mut domains = BTreeMap::new();
    for layer in &definition.layers {
        let input = input_shape(layer.data, &shapes)?;
        let data = snapshot.dataset(input.dataset)?;
        let output = stat_shape(
            input,
            data,
            &facets::scoped_stat(&layer.statistic, layer.scope),
            &layer.filters,
            limits,
            extensions,
        )
        .map_err(|e| context(e, data, Some(layer.id)))?;
        let mut binding = match (&output.bins, &output.statistical, &layer.mappings) {
            (None, None, Mappings::Source(aes)) => {
                source_binding(layer, aes, &definition.mappings, data, definition.profile())
                    .map(|(_, domain)| domain)
            }
            (Some(space), None, Mappings::Binned(aes)) => bin_binding(layer, aes, space),
            (None, Some(fields), Mappings::Statistical(aes)) => {
                statistical_binding(layer, aes, fields)
            }
            _ => Err(error(
                DiagnosticCode::SchemaConflict,
                "Aesthetic stage does not match the declared source/generated output schema.",
            )),
        }
        .map_err(|e| context(e, data, Some(layer.id)))?;
        for color in super::colors::encodings(layer) {
            super::colors::preflight(
                color,
                data,
                output.bins.is_none() && output.statistical.is_none(),
                output.statistical.as_deref(),
                limits,
                extensions,
            )
            .map_err(|e| context(e, data, Some(layer.id)))?;
        }
        super::scale_stage::generated_spaces(layer, population_axes(definition), &mut binding);
        super::positions::validate(layer, &binding, limits)?;
        super::positions::output_space(layer, &mut binding);
        if layer.orientation == Orientation::Horizontal {
            super::orientation::domains(&mut binding);
        }
        if layer.geom != Geom::Hierarchy {
            merge_named(&mut domains, layer.scales, &binding)
                .map_err(|e| context(e, data, Some(layer.id)))?;
        }
    }
    Ok(())
}
fn input_shape(
    input: DataRef,
    shapes: &BTreeMap<TransformId, OutputShape>,
) -> ChartResult<OutputShape> {
    match input {
        DataRef::Dataset(dataset) => Ok(OutputShape {
            dataset,
            bins: None,
            statistical: None,
        }),
        DataRef::Transform(id) => shapes.get(&id).cloned().ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                "Dependency schema is absent.",
            )
        }),
    }
}
fn stat_shape(
    mut input: OutputShape,
    data: &DatasetSnapshot,
    stat: &Statistic,
    filters: &[SourceFilter],
    limits: CompileLimits,
    extensions: &ExtensionRegistry,
) -> ChartResult<OutputShape> {
    if !filters.is_empty() && (input.bins.is_some() || input.statistical.is_some()) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Filter source observations before generating statistical rows.",
        ));
    }
    for filter in filters {
        numeric_space(data, &filter.value)?;
    }
    let automatic = if let StatParameters::AutoBin(s) = &stat.parameters {
        Some(BinSpec {
            ggplot: s.ggplot.clone(),
            input: s.input.clone(),
            edges: vec![],
            grouping: s.grouping.clone(),
            space: s.space.clone(),
            outliers: OutlierPolicy::Exclude,
        })
    } else {
        None
    };
    let bin = match &stat.parameters {
        StatParameters::Bin(s) => Some(s),
        _ => automatic.as_ref(),
    };
    if !matches!(stat.parameters, StatParameters::Identity)
        && (input.bins.is_some() || input.statistical.is_some())
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Nonidentity statistics require source observations.",
        ));
    }
    if matches!(
        stat.parameters,
        StatParameters::Distribution(_)
            | StatParameters::Spatial(_)
            | StatParameters::Model(_)
            | StatParameters::Univariate(_)
            | StatParameters::Count(_)
            | StatParameters::Summary(_)
            | StatParameters::Ols(_)
    ) {
        input.statistical = Some(super::statistics::schema(stat, data, limits)?);
    }
    if let StatParameters::Custom(p) = &stat.parameters {
        input.statistical = Some(extensions::custom_schema(
            extensions, stat, data, p, limits,
        )?);
    }
    if let Some(spec) = bin {
        if input.bins.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Bins require source observations rather than generated bins.",
            ));
        }
        let space = numeric_space(data, &spec.input)?;
        validate_group(data, &spec.grouping)?;
        input.bins = Some(match &spec.space {
            StatSpace::Data => space,
            StatSpace::Transformed(transform) => ValueSpace::Transformed {
                input: Box::new(space),
                transform: transform.clone(),
            },
        });
    }
    Ok(input)
}

pub(super) fn statistical_binding(
    layer: &Layer,
    aes: &StatAes,
    fields: &[StatColumn],
) -> ChartResult<DomainContributions> {
    if matches!(layer.geom, Geom::Ohlc { .. }) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "OHLC requires source open/close/low/high mappings.",
        ));
    }
    let space = |value: &StatNumeric| -> ChartResult<Option<ValueSpace>> {
        match value {
            StatNumeric::Expression(expr) => {
                numeric_expression_type(expr, |field| stat_expression_type(fields, field))?;
                Ok(Some(ValueSpace::Data))
            }
            StatNumeric::Literal(v) if v.is_finite() => Ok(None),
            StatNumeric::Literal(_) => Err(error(
                DiagnosticCode::NumericalDomain,
                "Generated literal must be finite.",
            )),
            StatNumeric::Field(f) => fields
                .iter()
                .find(|c| &c.field == f)
                .map(|c| Some(c.space.clone()))
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "Generated field is absent from this statistic's schema.",
                    )
                }),
        }
    };
    validate_line_size(layer, aes.size.is_some())?;
    let mut d = DomainContributions {
        x_space: space(&aes.x)?,
        y_space: space(&aes.y)?,
        ..Default::default()
    };
    for v in [&aes.x2, &aes.y2].into_iter().flatten() {
        space(v)?;
    }
    if let Some(v) = &aes.size
        && matches!(
            space(v)?,
            Some(ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. })
        )
    {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Mapped generated size requires a numeric field.",
        ));
    }
    if matches!(
        layer.geom,
        Geom::Rule | Geom::ShapeLink { .. } | Geom::Rectangle | Geom::ShapeArea { .. }
    ) && layer.recipe.is_none()
    {
        let (Some(x2), Some(y2)) = (&aes.x2, &aes.y2) else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Interval geometry requires both generated endpoints.",
            ));
        };
        merge_space(&mut d.x_space, &space(x2)?)?;
        merge_space(&mut d.y_space, &space(y2)?)?;
    }
    if matches!(layer.geom, Geom::Ribbon { .. } | Geom::Bar { .. }) {
        let y2 = aes.y2.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::SchemaConflict,
                "Ribbon requires a generated y2 upper-bound mapping.",
            )
        })?;
        merge_space(&mut d.y_space, &space(y2)?)?;
    }
    Ok(d)
}

// Theme/furniture/guide changes reuse immutable rows, marks and provenance without recomputation.
fn rebind_presentation(
    chart: &mut PreparedChart,
    definition: &Arc<ChartDefinition>,
    state: &ChartState,
) {
    chart.state = state.clone();
    for layer in &mut chart.layers {
        layer.visible = state.is_visible(layer.id);
    }
    chart.definition = definition.clone();
    chart.metrics = PreparationMetrics {
        evaluated_transforms: 0,
        reused_transforms: chart.metrics.evaluated_transforms + chart.metrics.reused_transforms,
        evaluated_layers: 0,
        reused_layers: chart.metrics.evaluated_layers + chart.metrics.reused_layers,
    };
    for panel in &mut chart.panels {
        rebind_presentation(Arc::make_mut(&mut panel.chart), definition, state);
    }
}

pub(super) fn apply_custom_geometry(
    registry: &ExtensionRegistry,
    extension: &GeometryExtension,
    layer: &Layer,
    prepared: &mut PreparedLayer,
    vertices: &mut usize,
) -> ChartResult<()> {
    let implementation = registry.geom(&extension.operation)?;
    let outputs = implementation.prepare(CustomGeomInput {
        marks: prepared.marks(),
        table: prepared.table(),
        domains: prepared.domains(),
        parameters: &extension.parameters,
        max_vertices: *vertices,
    })?;
    if outputs.len() > *vertices {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Custom geometry exceeded its remaining mark budget.",
        ));
    }
    let mut marks = Vec::with_capacity(outputs.len());
    let mut interactions = BTreeMap::new();
    let mut order = BTreeSet::new();
    let mut domains = DomainContributions {
        x_space: prepared.domains.x_space.clone(),
        y_space: prepared.domains.y_space.clone(),
        ..Default::default()
    };
    for output in outputs {
        let source = prepared.marks().get(output.input_mark).ok_or_else(|| {
            error(
                DiagnosticCode::Validation,
                "Custom geometry references an absent input mark.",
            )
        })?;
        if source.targets.len() != 1 {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Atomic custom geometry requires one target per input mark; split runs into explicit rules or points.",
            ));
        }
        let n = match &output.geometry {
            PreparedGeometry::Point(_) => 1,
            PreparedGeometry::Rule { .. } | PreparedGeometry::Rectangle { .. } => 2,
            PreparedGeometry::Polygon(p) if p.len() >= 3 => p.len(),
            PreparedGeometry::NativePaint {
                painter,
                parameters,
                ..
            } => {
                if implementation.descriptor().portable {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "A portable geometry cannot emit a native-only painter.",
                    ));
                }
                extensions::validate_name(&painter.id)?;
                if painter.version == crate::Revision::INITIAL {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Native painters require a positive version.",
                    ));
                }
                extensions::parameter_size(parameters)?;
                2
            }
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Custom atomic paint supports points, rules, rectangles, polygons and explicit native painters.",
                ));
            }
        };
        charge(vertices, n, "custom paint vertex")?;
        let hit = output.interaction.validate(*vertices)?;
        charge(vertices, hit, "custom hit vertex")?;
        if !order.insert(output.interaction.keyboard_order) {
            return Err(error(
                DiagnosticCode::Validation,
                "Custom keyboard order must be unique within a layer/panel.",
            ));
        }
        include_geometry(&mut domains, &output.geometry);
        interactions.insert(marks.len(), output.interaction);
        marks.push(PreparedMark {
            aesthetics: BTreeMap::new(),
            geometry: output.geometry,
            targets: source.targets.clone(),
            group: source.group.clone(),
            style: source.style,
        });
    }
    // Semantic positions precede this custom lowering; geometry cannot change encoding spaces.
    let _ = layer;
    prepared.marks = Arc::new(marks);
    prepared.domains = domains;
    prepared.interactions = interactions;
    Ok(())
}

fn same_population(a: &Layer, b: &Layer) -> bool {
    a.data == b.data
        && a.statistic == b.statistic
        && a.filters == b.filters
        && a.facet == b.facet
        && a.scope == b.scope
        && a.invalid == b.invalid
}

pub(super) fn expression_number(value: Option<f64>) -> ExpressionValue {
    value.map_or(
        ExpressionValue::Missing(ExpressionType::Number),
        ExpressionValue::Number,
    )
}
fn numeric_expression_type<R>(
    expr: &Expression<R>,
    read: impl FnMut(&R) -> ChartResult<ExpressionType>,
) -> ChartResult<()> {
    if expr.validate(ExpressionLimits::default(), read)? != ExpressionType::Number {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Numeric mappings require a numeric expression result.",
        ));
    }
    Ok(())
}
pub(super) fn stat_expression_type(
    fields: &[StatColumn],
    field: &StatField,
) -> ChartResult<ExpressionType> {
    if !fields.iter().any(|c| {
        &c.field == field
            && !matches!(
                c.space,
                ValueSpace::Categorical { .. } | ValueSpace::NullableCategorical { .. }
            )
    }) {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Expression requires a numeric field in this statistic's generated schema.",
        ));
    }
    Ok(ExpressionType::Number)
}
pub(super) fn backtransform(value: f64, space: &ValueSpace) -> Option<f64> {
    match space {
        ValueSpace::Scaled { scale, .. } => scale
            .transform
            .as_ref()
            .map_or(Some(value), |t| t.inverse(value).ok()),
        _ => Some(value),
    }
    .filter(|v| v.is_finite())
}

fn train_positioned_limits(
    definition: &ChartDefinition,
    layers: &mut [(&Layer, &mut PositionedLayer)],
    registry: &ExtensionRegistry,
    limits: CompileLimits,
    empty: &mut std::collections::BTreeSet<crate::ScaleId>,
    map_vectors: bool,
) -> ChartResult<BTreeMap<crate::ScaleId, Vec<crate::interpolate::Number>>> {
    use crate::interpolate::Number;
    let mut result = BTreeMap::new();
    let mut remaining = limits.max_prepared_rows;
    for axis in &definition.axes {
        if axis.limits_function.is_none()
            && axis.numeric_limits.is_none()
            && axis.temporal_limits.is_none()
            && axis.oob_function.is_none()
        {
            let automatic = match &axis.scale {
                crate::layout::AxisScale::Auto => true,
                crate::layout::AxisScale::Date { domain }
                | crate::layout::AxisScale::Utc { domain, .. } => domain.is_none(),
                crate::layout::AxisScale::Linear(domain)
                | crate::layout::AxisScale::Duration(domain)
                | crate::layout::AxisScale::Nonlinear { domain, .. } => {
                    *domain == crate::scales::ContinuousDomain::default()
                }
                crate::layout::AxisScale::Calendar { spec, .. } => spec.domain.is_empty(),
                _ => false,
            };
            if definition.profile() == Profile::Ggplot2_4_0_3 && automatic {
                let mut observed = false;
                let mut finite = false;
                for (layer, positioned) in layers.iter() {
                    let horizontal = layer.scales.x == axis.id;
                    if !horizontal && layer.scales.y != axis.id {
                        continue;
                    }
                    for row in &positioned.encoded {
                        observed = true;
                        let values = if horizontal {
                            [row.x, row.x2, None, None]
                        } else {
                            [row.y, row.y2, row.low, row.high]
                        };
                        finite |= values.into_iter().flatten().any(f64::is_finite);
                    }
                }
                if !observed
                    && matches!(
                        axis.scale,
                        crate::layout::AxisScale::Date { .. }
                            | crate::layout::AxisScale::Utc { .. }
                            | crate::layout::AxisScale::Calendar { .. }
                    )
                {
                    empty.insert(axis.id);
                }
                if observed && !finite {
                    result.insert(
                        axis.id,
                        vec![Number(f64::INFINITY), Number(f64::NEG_INFINITY)],
                    );
                }
            }
            continue;
        }
        if let crate::layout::AxisScale::Binned {
            prepared: Some(bins),
            ..
        } = &axis.scale
        {
            if let Some(limits) = bins.function_limits() {
                result.insert(axis.id, limits.to_vec());
            }
            continue;
        }
        let mut values = vec![];
        let mut projection =
            super::scale_stage::projection(axis).expect("validated numeric function axis");
        for (layer, positioned) in layers.iter() {
            let dimension = if layer.scales.x == axis.id {
                0
            } else if layer.scales.y == axis.id {
                1
            } else {
                continue;
            };
            for raw in &positioned.raw_training {
                charge(&mut remaining, 1, "facet raw limit population")?;
                values.push(raw[dimension].map(Number));
            }
            for row in &positioned.encoded {
                // A built-in summary with no usable observations has no reference
                // population at the post-stat stage, even though the library keeps
                // its typed empty aggregate for inspection.
                if positioned
                    .prepared
                    .table
                    .population_operation()
                    .is_some_and(|op| matches!(op.parameters, StatParameters::Summary(_)))
                    && let PreparedRows::Statistical(rows) = &positioned.prepared.table.rows
                    && rows.get(row.ordinal as usize).is_some_and(|r| r.count == 0)
                {
                    continue;
                }
                let inputs = if dimension == 0 {
                    [row.x, row.x2, None, None]
                } else {
                    [row.y, row.y2, row.low, row.high]
                };
                for (_, input) in inputs
                    .into_iter()
                    .enumerate()
                    .filter(|(i, v)| *i == 0 || v.is_some())
                {
                    charge(&mut remaining, 1, "positioned limit population")?;
                    values.push(input.map(Number));
                }
            }
        }
        let (family, reverse) = crate::scales::ggplot_numeric_limits::positional_coordinates(
            projection.transform.clone(),
        );
        let resolved: Vec<_> = if let Some(limits) = axis.authored_population_limits()? {
            if values.is_empty()
                && limits.iter().any(|v| {
                    v.is_none_or(|v| {
                        !projection
                            .transform
                            .as_ref()
                            .map_or(v.0, |t| t.forward_raw(v.0))
                            .is_finite()
                    })
                })
            {
                empty.insert(axis.id);
            }

            crate::scales::ggplot_numeric_limits::authored_transformed(
                limits,
                projection.transform.clone(),
                values.iter().flatten().map(|v| v.0),
                !values.is_empty(),
            )?
            .to_vec()
        } else if axis.limits_function.is_none() && axis.oob_function.is_some() {
            let authored = super::positional_vectors::authored_limits(axis)?;
            if values.is_empty() && authored.iter().all(Option::is_none) {
                empty.insert(axis.id);
            }
            super::positional_vectors::train_limits(
                axis,
                projection.transform.clone(),
                values.iter().flatten().map(|v| v.0),
                !values.is_empty(),
            )?
        } else {
            crate::scales::ggplot_numeric_limits::evaluate_transformed(
                axis.limits_function.as_ref().expect("selected function"),
                values.iter().flatten().map(|v| v.0),
                family,
                reverse,
                axis.resolved_temporal.as_deref().copied(),
                !values.is_empty(),
                registry,
            )?
            .into_iter()
            .map(|v| {
                Number(
                    projection
                        .transform
                        .as_ref()
                        .map_or(v.0, |t| t.forward_raw(v.0)),
                )
            })
            .collect()
        };
        projection.function_limits = Some(Box::new(resolved.clone()));
        if axis.oob_function.is_some() {
            // Reference panel break preparation precedes the second positional
            // map. Default temporal break generation rejects unbounded extents.
            if (axis.resolved_temporal.is_some()
                || matches!(axis.scale, crate::layout::AxisScale::Duration(_)))
                && axis.tick_values.is_none()
                && axis.guide_ticks.is_none()
                && axis.breaks_function.is_none()
                && resolved.iter().any(|v| !v.0.is_finite())
            {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Default temporal breaks require a finite trained range.",
                ));
            }
            if !map_vectors {
                result.insert(axis.id, resolved);
                continue;
            }
            let mut prepared_axis = axis.clone();
            prepared_axis.resolved_limits = Some(Box::new(resolved.clone()));
            let ids = layers.iter().map(|(l, _)| l.id).collect::<BTreeSet<_>>();
            for id in ids {
                let layer = layers
                    .iter()
                    .find(|(l, _)| l.id == id)
                    .expect("selected layer")
                    .0;
                let mut rows = layers
                    .iter_mut()
                    .filter(|(l, _)| l.id == id)
                    .flat_map(|(_, p)| p.encoded.iter_mut())
                    .collect::<Vec<_>>();
                // Identity rows retain original insertion order across interleaved
                // panels. Statistical outputs retain panel and group order.
                if super::positional_vectors::matched_source(definition, layer)
                    && matches!(layer.statistic.parameters, StatParameters::Identity)
                {
                    rows.sort_by_key(|r| r.ordinal);
                }
                super::positional_vectors::generated_rows(
                    definition,
                    layer,
                    std::slice::from_ref(&prepared_axis),
                    &mut rows,
                    registry,
                )?;
            }
            result.insert(axis.id, resolved);
            continue;
        }
        for (layer, positioned) in layers.iter_mut() {
            let [x2, y2, low, high] = super::scale_stage::mapped_endpoints(layer);
            for row in &mut positioned.encoded {
                if layer.scales.x == axis.id {
                    row.x = projection.project_transformed_optional(row.x);
                    if x2 || row.x2.is_some() {
                        row.x2 = projection.project_transformed_optional(row.x2);
                    }
                }
                if layer.scales.y == axis.id {
                    row.y = projection.project_transformed_optional(row.y);
                    if y2 || row.y2.is_some() {
                        row.y2 = projection.project_transformed_optional(row.y2);
                    }
                    if low || row.low.is_some() {
                        row.low = projection.project_transformed_optional(row.low);
                    }
                    if high || row.high.is_some() {
                        row.high = projection.project_transformed_optional(row.high);
                    }
                }
            }
        }
        result.insert(axis.id, resolved);
    }
    Ok(result)
}
