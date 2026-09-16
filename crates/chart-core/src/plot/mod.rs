//! Primary composable authoring API over the shared normalized chart engine.
//! Builders resolve names once; immutable plots retain exact data and registrations.
mod axis;
mod color;
mod composition;
mod data;
mod edit;
mod facet;
/// Thin dynamic-language dispatch into these same typed builders; not a grammar interpreter.
#[doc(hidden)]
pub mod host;
mod layer;
mod mapping;
mod options;
mod resolve;
mod stat;
mod text;
mod theme;
mod transaction;
mod transform;
mod wire;
use crate::data::{DataLimits, SnapshotHandle, StoreSnapshot};
use crate::grammar::{
    ChartDefinition, ColorEncoding, ColorInput, CompileLimits, Compiler, ExtensionRegistry,
    Mappings,
};
use crate::scales::ColorScale;
use crate::transaction::DataStore;
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId, Revision, ScaleId, SourceEpoch};
pub use axis::*;
pub use color::*;
pub use composition::*;
pub use data::*;
pub use facet::*;
pub use layer::*;
pub use mapping::*;
pub use options::*;
pub use stat::*;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
pub use text::*;
pub use theme::*;
pub use transaction::*;
pub use transform::*;

static NEXT_ID: AtomicU64 = AtomicU64::new(1024);
pub(super) fn fresh_id() -> ChartResult<u64> {
    NEXT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| {
            error(
                DiagnosticCode::RevisionOverflow,
                "Authoring identity allocation is exhausted.",
            )
        })
}
pub(super) fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Check the named data/layer/aesthetic and its type, or provide an explicit supported component policy.",
    )
}
pub(super) fn validate_name(name: &str) -> ChartResult<()> {
    if name.is_empty() || name.len() > 4096 {
        Err(error(
            DiagnosticCode::Validation,
            "Names must contain 1..4096 UTF-8 bytes.",
        ))
    } else {
        Ok(())
    }
}
pub use crate::grammar::Profile;
/// Immutable authored chart. Cloning retains identities/data without running the engine.
#[derive(Clone)]
pub struct Plot {
    pub(crate) owner: u64,
    pub(super) definition: ChartDefinition,
    pub(super) source: SnapshotHandle<StoreSnapshot>,
    pub(super) data: Vec<Data>,
    pub(super) layers: BTreeMap<String, LayerId>,
    pub(super) extensions: Arc<ExtensionRegistry>,
    pub(super) data_limits: DataLimits,
    pub(super) compile_limits: CompileLimits,
    pub(super) axes: BTreeMap<String, ScaleId>,
    pub(super) guides: BTreeMap<String, crate::GuideId>,
    pub(super) transforms: BTreeMap<String, crate::TransformId>,
    pub(super) colors: BTreeMap<String, ScaleId>,
}
impl std::fmt::Debug for Plot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Plot")
            .field("definition", &self.definition)
            .field("profile", &self.profile())
            .finish_non_exhaustive()
    }
}
impl Plot {
    /// Resolve a default or additional guide by its authored name.
    pub fn guide(&self, name: &str) -> ChartResult<GuideHandle> {
        self.guides
            .get(name)
            .copied()
            .map(GuideHandle)
            .or_else(|| self.axes.get(name).map(|id| AxisHandle(*id).guide()))
            .ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("No guide named '{name}'."),
                )
            })
    }
    /// Resolve the positional scale used by a default or additional named guide.
    pub fn guide_axis(&self, name: &str) -> ChartResult<AxisHandle> {
        if let Some(id) = self.guides.get(name) {
            return self
                .definition
                .guides
                .iter()
                .find(|g| g.id == *id)
                .map(|g| AxisHandle(g.scale))
                .ok_or_else(|| error(DiagnosticCode::MissingResource, "Named guide is absent."));
        }
        self.axis(name)
    }
    /// Default and additional guide names with independent stable identities.
    pub fn named_guides(&self) -> impl Iterator<Item = (&str, GuideHandle)> {
        self.axes
            .iter()
            .map(|(name, id)| (name.as_str(), AxisHandle(*id).guide()))
            .chain(
                self.guides
                    .iter()
                    .map(|(name, id)| (name.as_str(), GuideHandle(*id))),
            )
    }

    /// Authored layer names with stable handles, without exposing mutable grammar ownership.
    pub fn named_layers(&self) -> impl ExactSizeIterator<Item = (&str, LayerHandle)> {
        self.layers
            .iter()
            .map(|(name, id)| (name.as_str(), LayerHandle(*id)))
    }
    /// Authored axis names with stable handles for host navigation and linking.
    pub fn named_axes(&self) -> impl ExactSizeIterator<Item = (&str, AxisHandle)> {
        self.axes
            .iter()
            .map(|(name, id)| (name.as_str(), AxisHandle(*id)))
    }
    /// Read the normalized definition; edits use immutable builders or revision-fenced Chart APIs.
    pub fn definition(&self) -> &ChartDefinition {
        &self.definition
    }
    /// Cheap coherent source handle retained independently of live updates.
    pub fn source(&self) -> SnapshotHandle<StoreSnapshot> {
        self.source.clone()
    }
    /// Explicit numeric-preparation budgets retained by destinations and live charts.
    pub fn compile_limits(&self) -> CompileLimits {
        self.compile_limits
    }
    /// Explicit default semantic profile.
    pub fn profile(&self) -> Profile {
        self.definition.profile()
    }
    /// Exact immutable registry used by native/export destinations.
    pub fn extensions(&self) -> &Arc<ExtensionRegistry> {
        &self.extensions
    }
    /// Start an immutable definition-only edit, retaining this plot's data/identities.
    pub fn edit(&self) -> PlotEditBuilder {
        PlotEditBuilder {
            original: self.clone(),
            definition: self.definition.clone(),
            failure: None,
        }
    }
    /// Resolve an authored axis name.
    pub fn axis(&self, name: &str) -> ChartResult<AxisHandle> {
        self.axes.get(name).copied().map(AxisHandle).ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("No axis named '{name}'."),
            )
        })
    }
    /// Resolve named data without losing exact scalar metadata.
    pub fn data(&self, name: &str) -> ChartResult<&Data> {
        self.data.iter().find(|d| d.name == name).ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("No dataset named '{name}'."),
            )
        })
    }
    /// Resolve a named shared computation.
    pub fn transform(&self, name: &str) -> ChartResult<TransformHandle> {
        self.transforms
            .get(name)
            .copied()
            .map(TransformHandle)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("No transform named '{name}'."),
                )
            })
    }
    /// Resolve a named authored layer to its stable handle.
    pub fn layer(&self, name: &str) -> ChartResult<LayerHandle> {
        self.layers
            .get(name)
            .copied()
            .map(LayerHandle)
            .ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("No layer named '{name}'."),
                )
            })
    }
    /// Construct an owned typed runtime with independent view/ingestion state.
    pub fn chart(&self) -> ChartResult<crate::runtime::Chart> {
        let mut chart = crate::runtime::Chart::from_store_with_limits(
            self.definition.clone(),
            DataStore::new(
                SourceEpoch::new(fresh_id()?),
                self.data
                    .iter()
                    .map(|d| (d.id, d.batch.as_ref().clone()))
                    .collect(),
                self.data_limits,
            )?,
            self.extensions.clone(),
            self.compile_limits,
        )?;
        chart.plot_owner = Some(self.owner);
        chart.data_names = self
            .data
            .iter()
            .map(|data| (data.name.clone(), data.id))
            .collect();
        Ok(chart)
    }
}
/// Collect a plot's data and components before one atomic structural build.
#[derive(Clone)]
pub struct PlotBuilder {
    data: Data,
    datasets: Vec<Data>,
    mappings: AesBuilder,
    layers: Vec<LayerBuilder>,
    extensions: Arc<ExtensionRegistry>,
    profile: Profile,
    data_limits: DataLimits,
    compile_limits: CompileLimits,
    figure: crate::composition::FigureComposition,
    theme: Option<ThemeBuilder>,
    axes: Vec<AxisBuilder>,
    guides: Vec<GuideBuilder>,
    colors: Vec<ColorScaleBuilder>,
    legends: Vec<LegendBuilder>,
    annotations: Vec<LabelsBuilder>,
    insets: Vec<InsetBuilder>,
    facet: Option<FacetBuilder>,
    transforms: Vec<TransformBuilder>,
}
/// Start the main chart-authoring route with a default owned dataset.
pub fn plot(data: Data) -> PlotBuilder {
    PlotBuilder {
        data,
        datasets: vec![],
        mappings: aes(),
        layers: vec![],
        extensions: Arc::default(),
        profile: Profile::LibraryV1,
        data_limits: DataLimits::default(),
        compile_limits: CompileLimits::default(),
        figure: Default::default(),
        theme: None,
        axes: vec![],
        guides: vec![],
        colors: vec![],
        legends: vec![],
        annotations: vec![],
        insets: vec![],
        facet: None,
        transforms: vec![],
    }
}
/// A plotted data layer or one fixed annotation, sharing the same composition entry point.
pub enum PlotLayer {
    /// Data/statistic geometry with ordinary source or generated provenance.
    Marks(Box<LayerBuilder>),
    /// One fixed annotation with annotation identity, not repeated per source row.
    Annotation(Box<LabelsBuilder>),
    /// Fixed retained path with stable annotation identity.
    VectorPath(Box<VectorPathBuilder>),
}
impl From<VectorPathBuilder> for PlotLayer {
    fn from(path: VectorPathBuilder) -> Self {
        Self::VectorPath(Box::new(path))
    }
}
impl From<LayerBuilder> for PlotLayer {
    fn from(layer: LayerBuilder) -> Self {
        Self::Marks(Box::new(layer))
    }
}
impl From<LabelsBuilder> for PlotLayer {
    fn from(label: LabelsBuilder) -> Self {
        Self::Annotation(Box::new(label))
    }
}
impl From<CalloutBuilder> for PlotLayer {
    fn from(callout: CalloutBuilder) -> Self {
        Self::Annotation(Box::new(callout.label))
    }
}
macro_rules! figure_methods {
    () => {
        /// Set a separate plot title component.
        pub fn title(mut self, title: TitleBuilder) -> Self {
            self.figure_mut().title = Some(title.text);
            self
        }
        /// Set a separate plot subtitle component.
        pub fn subtitle(mut self, subtitle: SubtitleBuilder) -> Self {
            self.figure_mut().subtitle = Some(subtitle.text);
            self
        }
        /// Set the below-panel caption.
        pub fn caption(mut self, caption: CaptionBuilder) -> Self {
            self.figure_mut().caption = Some(caption.text);
            self
        }
        /// Append an ordered source note.
        pub fn source_note(mut self, note: SourceNoteBuilder) -> Self {
            self.figure_mut().source_notes.push(note.text);
            self
        }
        /// Append an ordered footnote.
        pub fn footnote(mut self, note: FootnoteBuilder) -> Self {
            self.figure_mut().footnotes.push(note.text);
            self
        }
        /// Append a stable panel letter using the shared figure-text layout.
        pub fn panel_letter(mut self, letter: PanelLetterBuilder) -> Self {
            self.figure_mut().panel_letters.push(letter.value);
            self
        }
    };
}
impl PlotBuilder {
    fn figure_mut(&mut self) -> &mut crate::composition::FigureComposition {
        &mut self.figure
    }
    figure_methods!();
    /// Configure shared theme tokens and layer overrides.
    pub fn theme(mut self, theme: ThemeBuilder) -> Self {
        self.theme = Some(theme);
        self
    }
    /// Configure the primary horizontal axis.
    pub fn x_axis(mut self, axis: AxisBuilder) -> Self {
        self.axes.retain(|a| a.name != axis.name);
        self.axes.push(axis);
        self
    }
    /// Configure the primary vertical axis.
    pub fn y_axis(mut self, axis: AxisBuilder) -> Self {
        self.axes.retain(|a| a.name != axis.name);
        self.axes.push(axis);
        self
    }
    /// Add an independent guide over an existing positional scale.
    pub fn guide(mut self, guide: GuideBuilder) -> Self {
        self.guides.push(guide);
        self
    }
    /// Add a named independent or secondary axis.
    pub fn axis(mut self, axis: AxisBuilder) -> Self {
        self.axes.push(axis);
        self
    }
    /// Add a named non-positional color scale; axes own their positional scale components.
    pub fn scale(mut self, scale: ColorScaleBuilder) -> Self {
        self.colors.push(scale);
        self
    }
    /// Override an existing named color legend, independently of annotations and axes.
    pub fn legend(mut self, legend: LegendBuilder) -> Self {
        self.legends.push(legend);
        self
    }
    /// Set mappings inherited and individually overridden by layers.
    pub fn aes(mut self, mappings: AesBuilder) -> Self {
        self.mappings = mappings;
        self
    }
    /// Append a layer in paint order.
    pub fn layer(mut self, layer: impl Into<PlotLayer>) -> Self {
        match layer.into() {
            PlotLayer::Marks(layer) => self.layers.push(*layer),
            PlotLayer::Annotation(label) => self.annotations.push(*label),
            PlotLayer::VectorPath(path) => {
                self.figure.version = 2;
                self.figure.paths.push(path.0);
            }
        };
        self
    }
    /// Add an inset over existing prepared layer handles.
    pub fn inset(mut self, inset: InsetBuilder) -> Self {
        self.insets.push(inset);
        self
    }
    /// Configure shared wrap/grid facets with an inferred or explicit fixed catalog.
    pub fn facet(mut self, facet: FacetBuilder) -> Self {
        self.facet = Some(facet);
        self
    }
    /// Add one shared named transform to the ordinary acyclic computation graph.
    pub fn transform(mut self, transform: TransformBuilder) -> Self {
        self.transforms.push(transform);
        self
    }
    /// Register another named dataset, including one used by later updates/transforms.
    pub fn data(mut self, data: Data) -> Self {
        self.datasets.push(data);
        self
    }
    /// Retain explicit immutable stat/geometry implementations, including native-only operations.
    pub fn extensions(mut self, extensions: Arc<ExtensionRegistry>) -> Self {
        self.extensions = extensions;
        self
    }
    /// Set coherent-store data budgets; individual Data batches retain their materialization budgets.
    pub fn data_limits(mut self, limits: DataLimits) -> Self {
        self.data_limits = limits;
        self
    }
    /// Set structural/preparation budgets retained by this plot and its live Chart.
    pub fn compile_limits(mut self, limits: CompileLimits) -> Self {
        self.compile_limits = limits;
        self
    }
    /// Select explicit supported semantic defaults.
    pub fn profile(mut self, profile: Profile) -> Self {
        self.profile = profile;
        self
    }
    /// Resolve identities, names and stages without executing stats, geometry or layout.
    pub fn build(self) -> ChartResult<Plot> {
        let mut datasets = vec![self.data.clone()];
        for data in self
            .datasets
            .iter()
            .chain(self.layers.iter().filter_map(|l| l.data.as_ref()))
            .chain(self.transforms.iter().filter_map(|t| t.data.as_ref()))
        {
            if let Some(existing) = datasets.iter().find(|d| d.id == data.id) {
                if existing.name != data.name || !Arc::ptr_eq(&existing.batch, &data.batch) {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "One dataset identity has distinct data owners or names; reuse the same Data clone for shared references.",
                    ));
                }
                continue;
            }
            if datasets.iter().any(|d| d.name == data.name) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    format!(
                        "Duplicate dataset name '{}'; give independent data distinct names.",
                        data.name
                    ),
                ));
            }
            datasets.push(data.clone());
        }
        data::align_timestamp_origins(&mut datasets);
        let canonical_data = |data: &Data| {
            &datasets[datasets
                .iter()
                .position(|d| d.id == data.id)
                .expect("registered authoring data")]
        };
        let mut definition = ChartDefinition::new(Revision::INITIAL).with_profile(self.profile);
        definition.facets = self
            .facet
            .as_ref()
            .map(|f| f.lower_sources(&self.data, &datasets, self.profile))
            .transpose()?;
        if self.figure != crate::composition::FigureComposition::default() {
            definition.figure = Some(self.figure);
        }
        definition.theme = self.theme.map(|t| t.spec);
        let mut axis_builders = self.axes;
        if axis_builders.is_empty()
            && !self.layers.is_empty()
            && self
                .layers
                .iter()
                .all(|l| l.geom == crate::grammar::Geom::Hierarchy)
        {
            axis_builders = vec![x_axis().visible(false), y_axis().visible(false)];
        }
        if !axis_builders.is_empty() {
            if !axis_builders.iter().any(|a| a.name == "x") {
                axis_builders.push(x_axis());
            }
            if !axis_builders.iter().any(|a| a.name == "y") {
                axis_builders.push(y_axis());
            }
        }
        let mut axes =
            BTreeMap::from([("x".into(), ScaleId::new(0)), ("y".into(), ScaleId::new(1))]);
        let mut declared = std::collections::BTreeSet::new();
        for axis in &axis_builders {
            validate_name(&axis.name)?;
            if !declared.insert(axis.name.clone()) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    format!("Duplicate axis name '{}'.", axis.name),
                ));
            }
            axes.insert(axis.name.clone(), axis.handle()?.id());
        }
        for axis in axis_builders {
            definition.axes.push(axis.lower(&axes)?);
        }
        let mut guide_names = BTreeMap::new();
        for guide in self.guides {
            validate_name(&guide.name)?;
            if axes.contains_key(&guide.name)
                || guide_names
                    .insert(guide.name.clone(), guide.handle()?.id())
                    .is_some()
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    format!("Duplicate guide name '{}'.", guide.name),
                ));
            }
            definition.guides.push(guide.lower(&axes)?);
        }

        for annotation in self.annotations {
            definition
                .figure
                .get_or_insert_with(Default::default)
                .annotations
                .push(annotation.lower(&axes)?);
        }
        for inset in self.insets {
            if let Some(e) = inset.failure {
                return Err(e);
            }
            definition
                .figure
                .get_or_insert_with(Default::default)
                .insets
                .push(inset.value);
        }
        let mut transform_names = BTreeMap::new();
        for t in &self.transforms {
            validate_name(&t.name)?;
            if transform_names
                .insert(t.name.clone(), t.id.clone()?)
                .is_some()
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    format!("Duplicate transform '{}'.", t.name),
                ));
            }
        }
        let mut transform_data = BTreeMap::new();
        let mut transform_defaults =
            BTreeMap::<crate::TransformId, Option<crate::grammar::Mappings>>::new();
        while transform_data.len() < self.transforms.len() {
            let before = transform_data.len();
            for t in &self.transforms {
                let id = t.id.clone()?;
                if transform_data.contains_key(&id) {
                    continue;
                }
                let dependency = t
                    .input
                    .as_ref()
                    .map(|i| i.resolve(&transform_names))
                    .transpose()?;
                let root = if let Some(dep) = dependency {
                    let Some(data) = transform_data.get(&dep) else {
                        continue;
                    };
                    data
                } else {
                    canonical_data(t.data.as_ref().unwrap_or(&self.data))
                };
                let input = dependency.map_or(
                    crate::grammar::DataRef::Dataset(root.id),
                    crate::grammar::DataRef::Transform,
                );
                let node = t.lower(root, input, &self.mappings, self.profile)?;
                let default = if matches!(
                    node.statistic.parameters,
                    crate::grammar::StatParameters::Custom(_)
                ) {
                    None
                } else {
                    t.statistic.default_mappings(crate::grammar::Geom::Point)?
                };
                let default = default.or_else(|| {
                    dependency.and_then(|id| transform_defaults.get(&id).cloned().flatten())
                });
                transform_defaults.insert(id, default);
                let root = root.clone();
                transform_data.insert(id, root);
                definition.transforms.push(node);
            }
            if before == transform_data.len() {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Transform dependency cycle.",
                ));
            }
        }
        let mut names = BTreeMap::new();
        let mut color_ids = BTreeMap::<String, ScaleId>::new();
        let mut ggplot_paint_scales = BTreeMap::new();
        let mut ggplot_numeric_ids = BTreeMap::new();
        let mut ggplot_style_ids = BTreeMap::new();
        let mut color_scales = BTreeMap::new();
        for color in self.colors {
            validate_name(&color.name)?;
            let scale = color.scale?;
            scale.validate_with_registry(&self.extensions)?;
            let id = color.id?;
            if color_scales.insert(color.name.clone(), scale).is_some() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    format!("Duplicate color scale '{}'.", color.name),
                ));
            }
            color_ids.insert(color.name, id);
        }
        for (index, builder) in self.layers.iter().enumerate() {
            let dependency = builder
                .input
                .as_ref()
                .map(|i| i.resolve(&transform_names))
                .transpose()?;
            let data = dependency.map_or_else(
                || canonical_data(builder.data.as_ref().unwrap_or(&self.data)),
                |id| &transform_data[&id],
            );
            let name = builder
                .name
                .clone()
                .unwrap_or_else(|| format!("layer_{}", index + 1));
            validate_name(&name)?;
            let (mut layer, mapping) =
                builder
                    .lower(data, &self.mappings, self.profile)
                    .map_err(|mut e| {
                        e.message =
                            format!("Dataset '{}', layer '{name}': {}", data.name, e.message);
                        e
                    })?;
            if let Some(id) = dependency {
                layer.data = crate::grammar::DataRef::Transform(id);
                if builder.generated.is_none()
                    && builder.stat.is_none()
                    && let Some(mappings) = &transform_defaults[&id]
                {
                    layer.mappings = mappings.clone();
                    if layer.orientation == crate::grammar::Orientation::Horizontal {
                        crate::grammar::orientation::transpose_mappings(&mut layer.mappings);
                    }
                    if matches!(layer.geom, crate::grammar::Geom::Bar { .. })
                        && let Mappings::Statistical(aes) = &mut layer.mappings
                    {
                        if layer.orientation == crate::grammar::Orientation::Horizontal {
                            aes.x2 = Some(crate::grammar::StatNumeric::Literal(0.));
                        } else {
                            aes.y2 = Some(crate::grammar::StatNumeric::Literal(0.));
                        }
                    }
                }
            }
            if names.insert(name.clone(), layer.id).is_some() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    format!("Duplicate layer name '{name}'."),
                ));
            }
            resolve::LayerContext {
                axes: &axes,
                color_ids: &mut color_ids,
                color_scales: &color_scales,
                ggplot_paint_scales: &mut ggplot_paint_scales,
                ggplot_numeric_ids: &mut ggplot_numeric_ids,
                ggplot_style_ids: &mut ggplot_style_ids,
            }
            .apply(&mut definition, &mut layer, builder, &mapping, data)?;
            definition.layers.push(layer);
        }
        for legend in self.legends {
            legend.apply(&mut definition, &color_ids)?;
        }
        for name in color_scales.keys() {
            if !definition.layers.iter().any(|l| {
                l.color
                    .iter()
                    .chain(l.paint_scales.values())
                    .any(|c| c.id == color_ids[name])
            }) {
                return Err(error(
                    DiagnosticCode::MissingResource,
                    format!("Declared color scale '{name}' has no mapped layer."),
                ));
            }
        }
        facet::validate_dataset_fields(&definition, &datasets)?;
        let store = DataStore::new(
            SourceEpoch::new(fresh_id()?),
            datasets
                .iter()
                .map(|d| (d.id, d.batch.as_ref().clone()))
                .collect(),
            self.data_limits,
        )?;
        let source = store.snapshot();
        Compiler::with_extensions(self.extensions.clone()).validate(
            &definition,
            &source,
            self.compile_limits,
        )?;
        Ok(Plot {
            owner: fresh_id()?,
            definition,
            source,
            data: datasets,
            layers: names,
            extensions: self.extensions,
            data_limits: self.data_limits,
            compile_limits: self.compile_limits,
            axes,
            guides: guide_names,
            transforms: transform_names,
            colors: color_ids,
        })
    }
}
/// Immutable definition edits. Applying the result to a live Chart never replaces its data.
#[derive(Clone)]
pub struct PlotEditBuilder {
    original: Plot,
    definition: ChartDefinition,
    failure: Option<Diagnostic>,
}
impl PlotEditBuilder {
    /// Change canonical execution semantics while retaining data and immutable old snapshots.
    pub fn profile(mut self, profile: Profile) -> Self {
        self.definition = self.definition.with_profile(profile);
        self
    }
    fn figure_mut(&mut self) -> &mut crate::composition::FigureComposition {
        self.definition.figure.get_or_insert_with(Default::default)
    }
    figure_methods!();
    /// Replace the theme while preserving source/layer identities.
    pub fn theme(mut self, theme: ThemeBuilder) -> Self {
        self.definition.theme = Some(theme.spec);
        self
    }
    /// Validate the candidate without running stats; only effective changes advance revision.
    pub fn build(mut self) -> ChartResult<Plot> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        self.original.colors.retain(|_, id| {
            self.definition.layers.iter().any(|l| {
                l.color
                    .iter()
                    .chain(l.paint_scales.values())
                    .any(|c| c.id == *id)
            })
        });
        if self.definition == self.original.definition {
            return Ok(self.original);
        }
        self.definition.revision = self.original.definition.revision.checked_next()?;
        facet::validate_dataset_fields(&self.definition, &self.original.data)?;
        Compiler::with_extensions(self.original.extensions.clone()).validate(
            &self.definition,
            &self.original.source,
            self.original.compile_limits,
        )?;
        self.original.definition = self.definition;
        Ok(self.original)
    }
}
fn default_color_scale() -> ColorScale<crate::color::Paint> {
    ColorScale::Discrete {
        domain: None,
        palette: vec![
            crate::theme::rgb(31, 119, 180).into(),
            crate::theme::rgb(255, 127, 14).into(),
            crate::theme::rgb(44, 160, 44).into(),
            crate::theme::rgb(214, 39, 40).into(),
            crate::theme::rgb(148, 103, 189).into(),
            crate::theme::rgb(140, 86, 75).into(),
        ],
        missing: crate::theme::rgb(128, 128, 128).into(),
    }
}

fn generated_group(
    definition: &ChartDefinition,
    layer: &crate::grammar::Layer,
) -> Option<crate::grammar::Grouping> {
    use crate::grammar::{DataRef, Grouping, StatScope};
    let (mut statistic, mut input, mut scope) = (&layer.statistic, layer.data, layer.scope);
    for _ in 0..=definition.transforms.len() {
        if scope != StatScope::Group {
            return Some(Grouping::All);
        }
        if let Some(group) = statistic.grouping() {
            return Some(group.clone());
        }
        let DataRef::Transform(id) = input else {
            return None;
        };
        let node = definition.transforms.iter().find(|t| t.id == id)?;
        statistic = &node.statistic;
        input = node.input;
        scope = node.scope;
    }
    None
}
