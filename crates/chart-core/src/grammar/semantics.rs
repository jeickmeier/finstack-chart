//! Versioned compatibility provenance and resolved execution policies.
use super::*;
use crate::ChartResult;

/// Source-stage context retained when a statistic replaces the visible mappings.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerGrammar {
    /// Text defaults were requested without an explicit TextGeom descriptor.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default_text: bool,
    /// Independent radius default; absent uses the legacy coupled size flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_radius: Option<bool>,
    /// Independent linewidth default; absent uses the legacy coupled size flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_line_width: Option<bool>,
    /// Apply the profile's geometry size when no constant size was authored.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default_size: bool,
    /// Apply the profile's geometry color when no constant color was authored.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default_color: bool,
    /// Authored source aesthetics before statistical fields replace the visible mappings.
    pub source: SourceAes,
    /// An explicit stat-level population override; absence permits profile inference.
    pub stat_grouping: Option<Grouping>,
}

/// Source aesthetics and population overrides for a shared statistical computation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransformGrammar {
    /// Authored positional and grouping inputs, before generated mappings.
    pub source: SourceAes,
    /// Discrete color participates in inferred grouping.
    pub color: Option<crate::FieldId>,
    /// Explicit operation grouping overrides inferred aesthetics.
    pub stat_grouping: Option<Grouping>,
}

/// Semantic compatibility identity; each profile resolves explicit canonical policies.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Profile {
    /// Existing explicit grouping and post-stat coordinate-scale behavior.
    #[default]
    LibraryV1,
    /// Pinned ggplot2 4.0.3 behavior; capability acceptance is tracked by GG-00–19.
    Ggplot2_4_0_3,
}

/// Default population grouping, overridden by explicitly authored groups.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GroupPolicy {
    /// Whole population unless grouping is authored explicitly.
    Explicit,
    /// Interaction of eligible discrete mapped aesthetics.
    DiscreteInteraction,
}

/// Ordering of positional scale transforms, limits and statistics.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ScaleStage {
    /// Existing scale mapping after statistical preparation.
    AfterStatistics,
    /// Scale transformation and out-of-bounds policy precede statistical preparation.
    BeforeStatistics,
}

/// Immutable canonical semantics, retained by definitions and every captured snapshot.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSemantics {
    /// Profile identity independent of destination layout/device settings.
    pub profile: Profile,
    /// Policy schema version, independent of source/definition revisions.
    pub version: u32,
    /// SHA-256 of the pinned upstream source archive, absent for LibraryV1.
    pub reference_sha256: Option<String>,
    /// Resolved grouping rule; provenance alone never selects behavior in an adapter.
    pub grouping: GroupPolicy,
    /// Resolved scale/stat order, used by the shared grammar engine.
    pub scale_stage: ScaleStage,
}
impl ExecutionSemantics {
    /// Resolve one supported profile into concrete policies and pinned provenance.
    pub fn for_profile(profile: Profile) -> Self {
        let (reference_sha256, grouping, scale_stage) = match profile {
            Profile::LibraryV1 => (None, GroupPolicy::Explicit, ScaleStage::AfterStatistics),
            Profile::Ggplot2_4_0_3 => (
                Some("690224bd61642b6222adb109470988e87f786e193cca77a15c0923cf9da73fa5".into()),
                GroupPolicy::DiscreteInteraction,
                ScaleStage::BeforeStatistics,
            ),
        };
        Self {
            profile,
            version: 1,
            reference_sha256,
            grouping,
            scale_stage,
        }
    }
    /// Reject forged provenance, unknown policy versions and mismatched resolved defaults.
    /// Explicit per-layer policies remain separate authored overrides.
    pub fn validate(&self) -> ChartResult<()> {
        if *self != Self::for_profile(self.profile) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Compatibility provenance or resolved policies do not match the supported profile.",
            ));
        }
        Ok(())
    }
}
impl ChartDefinition {
    /// Canonical semantic profile, with the unchanged default for legacy definitions.
    pub fn profile(&self) -> Profile {
        self.semantics
            .as_ref()
            .map_or(Profile::LibraryV1, |s| s.profile)
    }
    /// Resolve a semantic profile into this definition. Revision owners must advance
    /// their existing definition fence on effective changes.
    pub fn with_profile(mut self, profile: Profile) -> Self {
        self.semantics =
            (profile != Profile::LibraryV1).then(|| ExecutionSemantics::for_profile(profile));
        self
    }
}

pub(super) fn resolve<'a>(
    definition: &'a ChartDefinition,
    source: &crate::data::StoreSnapshot,
    limits: CompileLimits,
    registry: &ExtensionRegistry,
    execute_vectors: bool,
) -> ChartResult<std::borrow::Cow<'a, ChartDefinition>> {
    if needs_panel_training(definition) {
        return Ok(std::borrow::Cow::Borrowed(definition));
    }
    resolve_scoped(definition, source, limits, registry, None, execute_vectors)
}

pub(super) fn needs_panel_training(definition: &ChartDefinition) -> bool {
    definition.facets.as_ref().is_some_and(|facets| {
        definition.axes.iter().any(|a| {
            a.numeric_limits.is_some()
                || a.temporal_limits.is_some()
                || a.oob_function.is_some()
                || a.limits_function.is_some()
                || matches!(a.scale, crate::layout::AxisScale::Binned { .. })
                    && if a.side.horizontal() {
                        facets.scales.free_x
                    } else {
                        facets.scales.free_y
                    }
        })
    })
}

pub(super) fn resolve_scoped<'a>(
    definition: &'a ChartDefinition,
    source: &crate::data::StoreSnapshot,
    limits: CompileLimits,
    registry: &ExtensionRegistry,
    scope: Option<&super::facets::PanelScope>,
    execute_vectors: bool,
) -> ChartResult<std::borrow::Cow<'a, ChartDefinition>> {
    for axis in &definition.axes {
        if axis.population_missing.is_some()
            && (definition.profile() != Profile::Ggplot2_4_0_3
                || axis.scale_stage == Some(ScaleStage::AfterStatistics)
                || !matches!(
                    axis.scale,
                    crate::layout::AxisScale::Auto
                        | crate::layout::AxisScale::Linear(_)
                        | crate::layout::AxisScale::Duration(_)
                        | crate::layout::AxisScale::Nonlinear { .. }
                ))
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Positional missing replacement requires a ggplot2 pre-statistic numeric or duration scale.",
            ));
        }
    }
    let Some(policy) = &definition.semantics else {
        if definition.axes.iter().any(|a| {
            a.numeric_limits.is_some()
                || a.temporal_limits.is_some()
                || a.oob_function.is_some()
                || a.limits_function.is_some()
                || matches!(a.scale, crate::layout::AxisScale::Binned { .. })
        }) {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Population-dependent positional scales require the ggplot2 pre-statistic scale stage.",
            ));
        }
        return Ok(std::borrow::Cow::Borrowed(definition));
    };
    policy.validate()?;
    if definition.layers.len() > limits.max_layers
        || definition.transforms.len() > limits.max_transforms
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Compatibility preparation exceeds definition budgets.",
        ));
    }
    let mut resolved = definition.clone();
    if let Some(theme) = &mut resolved.theme
        && theme.hierarchy.is_some()
        && theme.geometry.is_none()
    {
        theme.geometry = Some(theme.geometry_defaults()?);
    }
    if policy.profile == Profile::Ggplot2_4_0_3 {
        for statistic in resolved
            .layers
            .iter_mut()
            .map(|l| &mut l.statistic)
            .chain(resolved.transforms.iter_mut().map(|t| &mut t.statistic))
        {
            match &mut statistic.parameters {
                StatParameters::Bin(s) => {
                    s.ggplot.get_or_insert_with(Default::default);
                }
                StatParameters::AutoBin(s) => {
                    s.ggplot.get_or_insert_with(Default::default);
                }
                _ => {}
            }
        }
    }
    let theme_elements = definition
        .theme
        .as_ref()
        .map(|t| t.resolved_elements())
        .transpose()?
        .flatten();
    for layer in &mut resolved.layers {
        let mut input = layer.data;
        for _ in 0..=definition.transforms.len() {
            let DataRef::Transform(id) = input else {
                break;
            };
            input = definition
                .transforms
                .iter()
                .find(|t| t.id == id)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        "Layer transform is absent.",
                    )
                })?
                .input;
        }
        let DataRef::Dataset(id) = input else {
            return Err(error(
                DiagnosticCode::Validation,
                "Transform dependency cycle.",
            ));
        };
        let data = source.dataset(id)?;
        if policy.profile == Profile::Ggplot2_4_0_3
            && layer.reference_point()
            && !layer
                .grammar
                .as_ref()
                .is_some_and(|g| g.default_radius == Some(false))
        {
            layer
                .style
                .units
                .get_or_insert(super::AestheticUnits::Millimeters);
        }
        if policy.profile == Profile::Ggplot2_4_0_3
            && let Some(grammar) = &layer.grammar
        {
            let theme = definition
                .theme
                .as_ref()
                .map(|t| t.geometry_defaults())
                .transpose()?
                .unwrap_or_default();
            theme.validate()?;
            if grammar.default_radius.unwrap_or(grammar.default_size) {
                layer.style.radius = theme.point_size;
            }
            if grammar.default_line_width.unwrap_or(grammar.default_size) {
                layer.style.stroke_width = theme.line_width;
            }
            // Fixed parameters replace mapped aesthetics before scale training and
            // group inference. Keep the authored mapping in the immutable definition.
            if grammar.default_line_width == Some(false) {
                layer.numeric_scales.remove(&NumericAesthetic::StrokeWidth);
            }
            if grammar.default_color {
                layer.style.color = theme.ink;
            } else {
                layer.color = None;
            }
            if let Some(elements) = &theme_elements {
                if grammar.default_text && layer.text.is_some() {
                    if !layer.value_scales.contains_key(&ValueAesthetic::TextSize)
                        && let Some(size) = elements.number("geom", "fontsize")
                    {
                        layer
                            .aesthetic_values
                            .entry(ValueAesthetic::TextSize)
                            .or_insert(crate::interpolate::Value::Number(
                                crate::interpolate::Number(size),
                            ));
                    }
                    if !layer.value_scales.contains_key(&ValueAesthetic::FontFamily)
                        && let Some(crate::theme::ThemeValue::Text(family)) =
                            elements.value("geom", "family")
                        && !family.is_empty()
                    {
                        layer
                            .aesthetic_values
                            .entry(ValueAesthetic::FontFamily)
                            .or_insert_with(|| crate::interpolate::Value::Text(family.clone()));
                    }
                    if !layer.value_scales.contains_key(&ValueAesthetic::FontFace)
                        && let Some(crate::theme::ThemeValue::Text(face)) =
                            elements.value("geom", "fontface")
                        && face != "plain"
                    {
                        layer
                            .aesthetic_values
                            .entry(ValueAesthetic::FontFace)
                            .or_insert_with(|| crate::interpolate::Value::Text(face.clone()));
                    }
                }
                if grammar.default_color
                    && layer.color.is_none()
                    && !layer.paint_scales.contains_key(&PaintAesthetic::Stroke)
                    && let Some(color) = elements.paint("geom", "colour")?
                {
                    layer.style.color = color;
                }
                if layer.style.fill.is_none()
                    && !layer.paint_scales.contains_key(&PaintAesthetic::Fill)
                {
                    layer.style.fill = elements.paint("geom", "fill")?;
                }
                if layer.reference_point()
                    && layer.symbol.is_none()
                    && !layer.value_scales.contains_key(&ValueAesthetic::Shape)
                    && !layer.aesthetic_values.contains_key(&ValueAesthetic::Shape)
                    && let Some(shape) = elements.number("geom", "pointshape")
                    && shape != 19.
                {
                    layer.aesthetic_values.insert(
                        ValueAesthetic::Shape,
                        crate::interpolate::Value::Number(crate::interpolate::Number(shape)),
                    );
                }
            }
            if matches!(layer.recipe, Some(super::BuiltinRecipe::Column(_)))
                && layer.style.fill.is_none()
                && !layer
                    .paint_scales
                    .contains_key(&super::PaintAesthetic::Fill)
            {
                // geom_col defaults to a 35% paper blend, independent of outline colour.
                let ink = theme.ink.resolve();
                let paper = theme.paper.resolve();
                let mix = |ink: u8, paper: u8| {
                    (f64::from(ink) * 0.65 + f64::from(paper) * 0.35).round() as u8
                };
                layer.style.fill = Some(
                    crate::scene::Color {
                        red: mix(ink.red, paper.red),
                        green: mix(ink.green, paper.green),
                        blue: mix(ink.blue, paper.blue),
                        alpha: mix(ink.alpha, paper.alpha),
                    }
                    .into(),
                );
            }
            if layer.reference_point() {
                if grammar.default_radius != Some(false) && grammar.default_line_width.is_none() {
                    layer.style.stroke_width = theme.line_width;
                }
                let constant = !grammar.default_radius.unwrap_or(grammar.default_size);
                if constant {
                    layer.numeric_scales.remove(&NumericAesthetic::Size);
                }
                if (constant || layer.numeric_scales.contains_key(&NumericAesthetic::Size))
                    && let Mappings::Source(source) = &mut layer.mappings
                {
                    if layer.inherit {
                        *source = source.inherit(&definition.mappings);
                        layer.inherit = false;
                    }
                    source.size = None;
                }
            }
            super::recipe_distributions::apply_defaults(layer, &theme);
            super::recipe_models::apply_defaults(layer, &theme);
        }
        if policy.grouping == GroupPolicy::DiscreteInteraction {
            let authored = match &layer.mappings {
                Mappings::Source(a) => Some(if layer.inherit {
                    a.inherit(&definition.mappings)
                } else {
                    a.clone()
                }),
                _ => layer.grammar.as_ref().map(|g| g.source.clone()),
            };
            if let Some(aes) = authored {
                let color = super::colors::encodings(layer)
                    .map(|c| &c.input)
                    .chain(layer.symbol.iter().map(|c| &c.input))
                    .chain(
                        layer
                            .numeric_scales
                            .iter()
                            .filter(|(a, _)| {
                                **a != NumericAesthetic::Alpha || layer.style.alpha.is_none()
                            })
                            .map(|(_, c)| &c.input),
                    )
                    .chain(
                        layer
                            .value_scales
                            .iter()
                            .filter(|(a, _)| {
                                **a != ValueAesthetic::Label
                                    && !layer.aesthetic_values.contains_key(a)
                                    && (**a != ValueAesthetic::LineType
                                        || layer.style.line_type.is_none())
                            })
                            .map(|(_, c)| &c.input),
                    )
                    .filter_map(|input| match *input {
                        ColorInput::Category(f) | ColorInput::GroupField(f) => Some(f),
                        _ => None,
                    });
                let group = resolve_group(&aes, color, data)?;
                if let Mappings::Source(mapped) = &mut layer.mappings {
                    // Execution-local resolved mappings; the returned snapshot retains authored semantics.
                    *mapped = aes.clone().grouped(group.clone());
                    layer.inherit = false;
                }
                if let Some(grammar) = &layer.grammar {
                    let stat_group = grammar
                        .stat_grouping
                        .as_ref()
                        .map(keep_missing)
                        .unwrap_or(group);
                    set_group(&mut layer.statistic, stat_group);
                }
            }
        }
    }
    super::scale_stage::train_function_axes(&mut resolved, source, limits, registry, scope)?;
    super::scale_stage::train_binned_axes(&mut resolved, source, limits, registry, scope)?;
    if policy.scale_stage == ScaleStage::BeforeStatistics {
        let vector_context = execute_vectors
            .then_some(&resolved)
            .filter(|_| {
                resolved
                    .axes
                    .iter()
                    .any(super::positional_vectors::selected)
            })
            .cloned();
        let mut vector_cache = super::positional_vectors::SourceCache::default();
        for layer in &mut resolved.layers {
            super::scale_stage::source_layer(layer, &resolved.axes)?;
            if let Some(context) = &vector_context {
                super::positional_vectors::source_layer(
                    layer,
                    context,
                    source,
                    registry,
                    limits,
                    None,
                    &mut vector_cache,
                )?;
            }
        }
    }
    resolve_transforms(&mut resolved, definition, source, policy)?;
    if execute_vectors && policy.scale_stage == ScaleStage::BeforeStatistics {
        let context = resolved.clone();
        super::positional_vectors::source_transforms(
            &mut resolved,
            &context,
            source,
            registry,
            limits,
            None,
            &mut std::collections::BTreeMap::new(),
        )?;
    }
    Ok(std::borrow::Cow::Owned(resolved))
}
fn keep_missing(group: &Grouping) -> Grouping {
    match group {
        Grouping::Field(f) => Grouping::Interaction(vec![*f]),
        _ => group.clone(),
    }
}
fn resolve_group(
    aes: &SourceAes,
    color: impl IntoIterator<Item = crate::FieldId>,
    data: &crate::data::DatasetSnapshot,
) -> ChartResult<Grouping> {
    if let Some(group) = &aes.grouping {
        return Ok(keep_missing(group));
    }
    if let Some(field) = aes.group {
        return Ok(Grouping::Interaction(vec![field]));
    }
    let mut fields = vec![];
    let mut add = |field| -> ChartResult<()> {
        let (_, column) = data.schema().field(field).ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                "Inferred grouping field is absent.",
            )
        })?;
        if matches!(
            column.kind,
            crate::data::FieldKind::Categorical
                | crate::data::FieldKind::Utf8
                | crate::data::FieldKind::Boolean
        ) && !fields.contains(&field)
        {
            fields.push(field);
        }
        Ok(())
    };
    for mapping in [
        &aes.x, &aes.y, &aes.x2, &aes.y2, &aes.low, &aes.high, &aes.size,
    ]
    .into_iter()
    .flatten()
    {
        if let Numeric::Field(field) | Numeric::Category(field) = mapping {
            add(*field)?;
        }
    }
    for field in color {
        add(field)?;
    }
    Ok(if fields.is_empty() {
        Grouping::All
    } else {
        Grouping::Interaction(fields)
    })
}
fn set_group(stat: &mut Statistic, group: Grouping) {
    match &mut stat.parameters {
        StatParameters::Distribution(s) => s.grouping = group,
        StatParameters::Spatial(s) => s.grouping = group,
        StatParameters::Model(s) => s.grouping = group,
        StatParameters::Univariate(s) => s.grouping = group,
        StatParameters::Identity => {}
        StatParameters::Custom(s) => s.grouping = group,
        StatParameters::AutoBin(s) => s.grouping = group,
        StatParameters::Bin(s) => s.grouping = group,
        StatParameters::Count(s) => s.grouping = group,
        StatParameters::Summary(s) => s.grouping = group,
        StatParameters::Ols(s) => s.grouping = group,
    }
}

// One shared output has one population/coordinate contract. Compare the actual
// resolved operation, so unused consumer axes do not introduce false conflicts.
fn resolve_transforms(
    resolved: &mut ChartDefinition,
    authored: &ChartDefinition,
    source: &crate::data::StoreSnapshot,
    policy: &ExecutionSemantics,
) -> ChartResult<()> {
    let mut operations = std::collections::BTreeMap::new();
    for consumer in &authored.layers {
        let mut input = consumer.data;
        for _ in 0..=authored.transforms.len() {
            let DataRef::Transform(id) = input else {
                break;
            };
            let node = authored
                .transforms
                .iter()
                .find(|n| n.id == id)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        "Shared statistic is absent.",
                    )
                })?;
            let mut layer = consumer.clone();
            layer.statistic = node.statistic.clone();
            layer.data = node.input;
            layer.filters = node.filters.clone();
            layer.facet = node.facet.clone();
            layer.scope = node.scope;
            // Consumer mappings are generated fields; only node source aesthetics
            // may determine the shared statistic's grouping and input axis.
            layer.mappings = Mappings::Source(SourceAes::default());
            let default_text = layer.grammar.as_ref().is_some_and(|g| g.default_text);
            layer.grammar = node.grammar.as_ref().map(|g| LayerGrammar {
                default_text,
                source: g.source.clone(),
                stat_grouping: g.stat_grouping.clone(),
                default_radius: None,
                default_line_width: None,
                default_size: false,
                default_color: false,
            });
            if layer.orientation == Orientation::Horizontal
                && let Some(g) = &mut layer.grammar
            {
                super::orientation::transpose_source(&mut g.source);
            }
            if policy.grouping == GroupPolicy::DiscreteInteraction
                && let Some(grammar) = &node.grammar
            {
                let mut root = node.input;
                for _ in 0..=authored.transforms.len() {
                    let DataRef::Transform(parent) = root else {
                        break;
                    };
                    root = authored
                        .transforms
                        .iter()
                        .find(|n| n.id == parent)
                        .ok_or_else(|| {
                            error(
                                DiagnosticCode::MissingResource,
                                "Shared statistic dependency is absent.",
                            )
                        })?
                        .input;
                }
                let DataRef::Dataset(root) = root else {
                    return Err(error(
                        DiagnosticCode::Validation,
                        "Shared statistic dependency cycle.",
                    ));
                };
                let group = grammar
                    .stat_grouping
                    .as_ref()
                    .map(keep_missing)
                    .map(Ok)
                    .unwrap_or_else(|| {
                        resolve_group(&grammar.source, grammar.color, source.dataset(root)?)
                    })?;
                set_group(&mut layer.statistic, group);
            }
            if policy.scale_stage == ScaleStage::BeforeStatistics {
                super::scale_stage::source_layer(&mut layer, &resolved.axes)?;
            }
            if let Some(previous) = operations.insert(id, layer.statistic.clone())
                && previous != layer.statistic
            {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Shared statistic consumers require incompatible positional scale contexts; author separate transforms.",
                ));
            }
            input = node.input;
        }
    }
    for node in &mut resolved.transforms {
        if let Some(operation) = operations.remove(&node.id) {
            node.statistic = operation;
        }
    }
    Ok(())
}
