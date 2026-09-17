use super::{AesBuilder, Data, Mapping, error, fresh_id};
use crate::grammar::*;
use crate::{ChartResult, DiagnosticCode, LayerId};

/// Stable layer identity, preserved by builder/Plot clones and edits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayerHandle(pub(super) LayerId);
impl LayerHandle {
    /// Exact identity for immutable specialist inspection and runtime actions.
    pub fn id(self) -> LayerId {
        self.0
    }
}
/// One composable layer; unresolved fields are temporary until Plot build.
#[derive(Clone)]
pub struct LayerBuilder {
    recipe: Option<BuiltinRecipe>,
    geography: Option<(GeoFeatureCollection, Mapping, GeoOperation)>,
    recipe_aes: std::collections::BTreeMap<RecipeAesthetic, NumericScaleInput>,
    legend: Option<LayerLegend>,
    text: Option<TextGeom>,
    pub(super) default_text: bool,
    annotation: Option<RowAnnotation>,
    pub(super) id: ChartResult<LayerId>,
    pub(super) name: Option<String>,
    pub(super) data: Option<Data>,
    pub(super) input: Option<super::TransformRef>,
    pub(super) mappings: AesBuilder,
    pub(super) inherit: bool,
    pub(super) geom: Geom,
    pub(super) hierarchy: Option<HierarchyRecipe<Mapping>>,
    pub(super) orientation: Option<Orientation>,
    pub(super) style: Style<crate::color::Paint>,
    numeric_scales: std::collections::BTreeMap<
        crate::grammar::NumericAesthetic,
        (NumericScaleInput, crate::scales::MappedScaleSpec),
    >,
    value_scales: std::collections::BTreeMap<
        ValueAesthetic,
        (NumericScaleInput, crate::scales::MappedScaleSpec),
    >,
    aesthetic_values: std::collections::BTreeMap<ValueAesthetic, crate::interpolate::Value>,
    symbol: Option<(Option<Mapping>, SymbolEncoding)>,
    symbol_size_guide: Option<SymbolSizeGuide>,
    shape_protocols: std::collections::BTreeMap<ShapeFamily, ShapeOperation>,
    pub(super) explicit_size: bool,
    pub(super) explicit_radius: bool,
    pub(super) explicit_line_width: bool,
    pub(super) explicit_color: bool,
    pub(super) after_scale:
        std::collections::BTreeMap<AfterScaleAesthetic, Expression<AfterScaleRead>>,
    pub(super) histogram: Option<usize>,
    pub(super) edges: Option<Vec<f64>>,
    pub(super) stat: Option<super::StatBuilder>,
    pub(super) generated: Option<Mappings>,
    pub(super) generated_color: Option<ColorInput>,
    bar_baseline: Option<f64>,
    pub(super) generated_color_scale: Option<String>,
    pub(super) filters: Vec<super::FilterBuilder>,
    pub(super) position: Option<super::PositionBuilder>,
    pub(super) axes: Option<(String, String)>,
    pub(super) scope: StatScope,
    pub(super) facet: FacetTarget,
    pub(super) clip: ClipPolicy,
    pub(super) invalid: crate::data::InvalidPolicy,
    pub(super) extension: Option<GeometryExtension>,
    pub(super) candle_colors: Option<CandleColors<crate::color::Paint>>,
    pub(super) theme: super::StyleBuilder,
    pub(super) failure: Option<crate::Diagnostic>,
}
impl LayerBuilder {
    /// Join typed geographic features to an ordinary source field in the shared compiler.
    pub fn geography(mut self, collection: GeoFeatureCollection, join: impl Into<Mapping>) -> Self {
        self.geography = Some((collection, join.into(), GeoOperation::Geometry));
        self
    }
    /// Select a geographic geometry-derived operation without host preprocessing.
    pub fn geography_operation(mut self, operation: GeoOperation) -> Self {
        if let Some((_, _, current)) = &mut self.geography {
            *current = operation;
        } else {
            self.failure = Some(error(
                DiagnosticCode::Validation,
                "Geographic operation requires a feature collection.",
            ));
        }
        self
    }

    /// Select a built-in recipe over common source/statistical channels.
    pub fn recipe(mut self, recipe: BuiltinRecipe) -> Self {
        self.recipe = Some(recipe);
        self
    }
    /// Bind a source field/expression or constant to a recipe channel.
    pub fn recipe_value(mut self, channel: RecipeAesthetic, value: impl Into<Mapping>) -> Self {
        self.recipe_aes
            .insert(channel, NumericScaleInput::Source(value.into()));
        self
    }
    /// Bind a generated statistical field to a recipe channel.
    pub fn recipe_stat_value(mut self, channel: RecipeAesthetic, value: StatField) -> Self {
        self.recipe_aes
            .insert(channel, NumericScaleInput::Statistical(value));
        self
    }

    /// Position a portable custom vector or raster at every retained row anchor.
    pub fn annotation(mut self, annotation: RowAnnotation) -> Self {
        self.annotation = Some(annotation);
        self
    }

    /// Render retained source/statistical rows as text using mapped Label aesthetics.
    pub fn text_geom(mut self, options: TextGeom) -> Self {
        self.default_text = false;
        self.text = Some(options);
        self
    }

    /// Render text with inherited geom theme defaults; mapped row controls remain authoritative.
    pub fn text_defaults(mut self) -> Self {
        self.text = Some(TextGeom::default());
        self.default_text = true;
        self
    }

    /// Map source text or exact scalar identities without an artificial label palette.
    pub fn text_label(self, field: impl Into<Mapping>) -> Self {
        let field = field.into();
        let scale = if matches!(
            field,
            Mapping::Expression(_) | Mapping::Literal(_) | Mapping::Scaled { .. }
        ) {
            crate::scales::ScaleFunctionSpec::GgplotNumericIdentity(Default::default())
        } else {
            crate::scales::ScaleFunctionSpec::GgplotDiscreteIdentity(Default::default())
        };
        self.value_scale(
            ValueAesthetic::Label,
            field,
            crate::scales::MappedScaleSpec::authored(scale),
        )
    }
    /// Format a finite generated statistic as a label while retaining its derived provenance.
    pub fn text_stat_label(self, field: StatField) -> Self {
        self.value_scale(
            ValueAesthetic::Label,
            NumericScaleInput::Statistical(field),
            crate::scales::MappedScaleSpec::authored(
                crate::scales::ScaleFunctionSpec::GgplotNumericIdentity(Default::default()),
            ),
        )
    }

    /// Select a versioned registered key while retaining guide inclusion controls.
    pub fn key_glyph(
        mut self,
        operation: impl Into<String>,
        version: crate::Revision,
        parameters: serde_json::Value,
    ) -> Self {
        self.legend
            .get_or_insert_with(Default::default)
            .registered_key = Some(crate::grammar::KeyGlyphSelection {
            operation: crate::grammar::OperationRef::new(operation, version),
            parameters,
        });
        self
    }

    /// Configure layer guide inclusion and key topology without changing marks.
    pub fn legend(mut self, policy: LayerLegend) -> Self {
        self.legend = Some(policy);
        self
    }

    fn new(geom: Geom) -> Self {
        Self {
            id: fresh_id().map(LayerId::new),
            name: None,
            recipe: None,
            geography: None,
            recipe_aes: Default::default(),
            legend: None,
            text: None,
            default_text: false,
            annotation: None,
            data: None,
            input: None,
            mappings: AesBuilder::default(),
            inherit: true,
            geom,
            hierarchy: None,
            orientation: None,
            style: Style::default(),
            numeric_scales: Default::default(),
            value_scales: Default::default(),
            aesthetic_values: Default::default(),
            symbol: None,
            symbol_size_guide: None,
            shape_protocols: Default::default(),
            explicit_size: false,
            explicit_radius: false,
            explicit_line_width: false,
            explicit_color: false,
            after_scale: Default::default(),
            histogram: None,
            edges: None,
            stat: None,
            generated: None,
            generated_color: None,
            bar_baseline: None,
            generated_color_scale: None,
            filters: vec![],
            position: None,
            axes: None,
            scope: StatScope::Group,
            facet: FacetTarget::Match,
            clip: ClipPolicy::Plot,
            invalid: crate::data::InvalidPolicy::Exclude,
            extension: None,
            candle_colors: None,
            theme: super::style(),
            failure: None,
        }
    }
    /// Select one versioned shape protocol; known Rust code must be explicitly registered.
    pub fn shape_protocol(mut self, family: ShapeFamily, selection: ShapeOperation) -> Self {
        match family {
            ShapeFamily::Curve => self = self.curve(crate::shape::CurveSpec::Linear),
            ShapeFamily::Symbol => {
                self = self.symbol_kind(crate::shape::SymbolKind::Circle);
                self.symbol = None;
            }
            ShapeFamily::StackOrder => {
                if let Some(position) = self.position.take() {
                    self.position = Some(position.stack_order(crate::shape::StackOrder::None));
                }
            }
            _ => {}
        }
        self.shape_protocols.insert(family, selection);
        self
    }
    /// Select a curve on an explicit shape route; ordinary recipes retain their policies.
    pub fn curve(mut self, value: crate::shape::CurveSpec) -> Self {
        self.shape_protocols.remove(&ShapeFamily::Curve);
        match &mut self.geom {
            Geom::ShapeLine { curve, .. }
            | Geom::ShapeArea { curve, .. }
            | Geom::ShapeLineRadial { curve, .. }
            | Geom::ShapeAreaRadial { curve, .. }
            | Geom::ShapeLink { curve } => *curve = value,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Curve selection requires a shape line, area or Cartesian link route.",
                ))
            }
        }
        self
    }
    /// Resolve this layer's identity before composing a plot.
    pub fn handle(&self) -> ChartResult<LayerHandle> {
        self.id.clone().map(LayerHandle)
    }
    /// Assign a discoverable name without replacing its stable identity.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    /// Override individual inherited source mappings.
    pub fn aes(mut self, mappings: AesBuilder) -> Self {
        self.mappings = mappings;
        self
    }
    /// Supply an independently owned dataset; inherited names resolve against its schema.
    pub fn data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }
    /// Consume one shared transform by name or stable handle.
    pub fn from_transform(mut self, input: impl Into<super::TransformRef>) -> Self {
        self.input = Some(input.into());
        self
    }
    /// Disable inherited mappings for a layer with independent coordinates/schema.
    pub fn independent(mut self) -> Self {
        self.inherit = false;
        self
    }
    /// Set the independent-axis direction for oriented statistics and geometry.
    pub fn orientation(mut self, value: Orientation) -> Self {
        self.orientation = Some(value);
        self
    }
    /// Set a constant point radius or line stroke width in baseline destination units.
    pub fn size(mut self, value: f64) -> Self {
        self.explicit_size = true;
        self.style.radius = value;
        self.style.stroke_width = value;
        self
    }
    /// Set a constant fill/stroke color; mapped color remains authoritative.
    pub fn color(mut self, color: impl Into<crate::color::Paint>) -> Self {
        self.explicit_color = true;
        self.style.color = color.into();
        self
    }
    /// Set a constant independent fill; this overrides any fill mapping.
    pub fn fill(mut self, paint: impl Into<crate::color::Paint>) -> Self {
        self.style.fill = Some(paint.into());
        self
    }
    /// Set a constant independent outline; this overrides any stroke mapping.
    pub fn stroke(mut self, paint: impl Into<crate::color::Paint>) -> Self {
        self.style.stroke = Some(paint.into());
        self
    }
    /// Set point radius without changing stroke width.
    pub fn radius(mut self, value: f64) -> Self {
        self.style.radius = value;
        self.explicit_radius = true;
        self
    }
    /// Set stroke width without changing point radius.
    pub fn linewidth(mut self, value: f64) -> Self {
        self.explicit_line_width = true;
        self.style.stroke_width = value;
        self
    }
    /// Set explicit physical units for point, symbol, stroke and text dimensions.
    pub fn aesthetic_units(mut self, units: AestheticUnits) -> Self {
        self.style.units = Some(units);
        self
    }
    /// Set stroke endpoint geometry through the shared core stroke renderer.
    pub fn lineend(mut self, value: crate::grammar::LineEnd) -> Self {
        self.style.line_end = Some(value);
        self
    }
    /// Set stroke joins; miter joins use the reference limit of ten stroke radii.
    pub fn linejoin(mut self, value: crate::grammar::LineJoin) -> Self {
        self.style.line_join = Some(value);
        self
    }
    /// Set an independent constant line type.
    pub fn line_type(mut self, line_type: LineType) -> Self {
        self.style.line_type = Some(line_type);
        self
    }
    /// Map line types or text channels through the common typed scale engine.
    pub fn value_scale(
        mut self,
        target: ValueAesthetic,
        input: impl Into<NumericScaleInput>,
        scale: crate::scales::MappedScaleSpec,
    ) -> Self {
        self.value_scales.insert(target, (input.into(), scale));
        self
    }
    /// Set a typed text or line-type constant, overriding its corresponding mapping.
    pub fn aesthetic_value(
        mut self,
        target: ValueAesthetic,
        value: crate::interpolate::Value,
    ) -> Self {
        self.aesthetic_values.insert(target, value);
        self
    }
    /// Replace paint alpha independently of the paint's embedded alpha.
    pub fn alpha(mut self, value: f64) -> Self {
        self.style.alpha = Some(value);
        self
    }
    /// Map resolved group identities through an explicit named color scale.
    pub fn color_group(mut self, scale: impl Into<String>) -> Self {
        self.generated_color = Some(ColorInput::Group);
        self.generated_color_scale = Some(scale.into());
        self
    }

    /// Set post-scale size/color expressions evaluated against resolved aesthetic values.
    pub fn after_scale(mut self, mappings: super::AfterScaleAesBuilder) -> Self {
        self.after_scale = mappings.mappings;
        self
    }
    /// Configure a shared statistic; generated defaults follow that statistic's declared schema.
    pub fn stat(mut self, stat: super::StatBuilder) -> Self {
        self.stat = Some(stat);
        self
    }
    /// Explicitly map generated statistical output, never a source field name.
    pub fn after_stat(mut self, mappings: super::StatAesBuilder) -> Self {
        self.generated_color = mappings.color;
        self.generated_color_scale = mappings.color_scale;
        self.generated = Some(Mappings::Statistical(mappings.aes));
        self
    }
    /// Explicitly map generated bin endpoints/counts.
    pub fn after_bin(mut self, mappings: super::BinAesBuilder) -> Self {
        self.generated_color = mappings.color_scale.as_ref().map(|_| ColorInput::Group);
        self.generated_color_scale = mappings.color_scale;
        self.generated = Some(Mappings::Binned(mappings.aes));
        self
    }
    /// Filter source observations before statistics and domain training.
    pub fn filter(mut self, filter: super::FilterBuilder) -> Self {
        self.filters.push(filter);
        self
    }
    /// Apply an existing semantic position kernel.
    pub fn position(mut self, position: super::PositionBuilder) -> Self {
        self.shape_protocols.remove(&ShapeFamily::StackOrder);
        self.shape_protocols.remove(&ShapeFamily::StackOffset);
        self.position = Some(position);
        self
    }
    /// Bind this layer to independently named positional scales.
    pub fn axes(mut self, x: impl Into<String>, y: impl Into<String>) -> Self {
        self.axes = Some((x.into(), y.into()));
        self
    }
    /// Set statistical population scope independently of presentation grouping.
    pub fn scope(mut self, scope: StatScope) -> Self {
        self.scope = scope;
        self
    }
    /// Explicitly match, broadcast or select facet panels.
    pub fn facet_target(mut self, target: FacetTarget) -> Self {
        self.facet = target;
        self
    }
    /// Choose plot or figure clipping.
    pub fn clip(mut self, clip: ClipPolicy) -> Self {
        self.clip = clip;
        self
    }
    /// Exclude invalid required values with counts, or reject preparation.
    pub fn invalid(mut self, policy: crate::data::InvalidPolicy) -> Self {
        self.invalid = policy;
        self
    }
    /// Set direction-dependent OHLC colors.
    pub fn candle_colors<P: Into<crate::color::Paint>>(mut self, colors: CandleColors<P>) -> Self {
        self.candle_colors = Some(colors.map_colors(Into::into));
        self
    }
    /// Configure layer presentation tokens, including constant symbol and dash pattern.
    pub fn style(mut self, style: super::StyleBuilder) -> Self {
        self.theme = style;
        self
    }
    /// Select a registered geometry after the shared encoding/position stage.
    pub fn geometry(
        mut self,
        id: impl Into<String>,
        version: crate::Revision,
        parameters: serde_json::Value,
    ) -> Self {
        self.extension = Some(GeometryExtension {
            operation: OperationRef::new(id, version),
            parameters,
        });
        self
    }
    /// Set bar/OHLC destination width.
    pub fn width(mut self, value: f64) -> Self {
        match &mut self.geom {
            Geom::Bar { width, .. } | Geom::Ohlc { width } => *width = value,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Width requires bar/OHLC geometry.",
                ))
            }
        };
        self
    }
    /// Set run order for line/area/ribbon geometry.
    pub fn order(mut self, value: LineOrder) -> Self {
        match &mut self.geom {
            Geom::ShapeLineRadial { order, .. }
            | Geom::ShapeAreaRadial { order, .. }
            | Geom::ShapeLine { order, .. }
            | Geom::ShapeArea { order, .. }
            | Geom::Line { order, .. }
            | Geom::Area { order, .. }
            | Geom::Ribbon { order, .. } => *order = value,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Run ordering requires line/area/ribbon geometry.",
                ))
            }
        };
        self
    }
    /// Explicitly bridge missing values in line/area/ribbon runs.
    pub fn connect_gaps(mut self, enabled: bool) -> Self {
        match &mut self.geom {
            Geom::ShapeLineRadial { connect_gaps, .. }
            | Geom::ShapeAreaRadial { connect_gaps, .. }
            | Geom::ShapeLine { connect_gaps, .. }
            | Geom::ShapeArea { connect_gaps, .. }
            | Geom::Line { connect_gaps, .. }
            | Geom::Area { connect_gaps, .. }
            | Geom::Ribbon { connect_gaps, .. } => *connect_gaps = enabled,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Gap bridging requires line/area/ribbon geometry.",
                ))
            }
        };
        self
    }
    /// Set the requested equal-width histogram bin count.
    pub fn bins(mut self, bins: usize) -> Self {
        if self.histogram.is_some() {
            self.histogram = Some(bins);
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Bins configure a histogram; use .stat(bin()) for another geometry.",
            ));
        }
        self
    }
    /// Select explicit histogram edges with existing left-closed/final-inclusive semantics.
    pub fn breaks(mut self, edges: Vec<f64>) -> Self {
        if self.histogram.is_some() {
            self.edges = Some(edges);
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Breaks configure a histogram; use .stat(bin()) for another geometry.",
            ));
        }
        self
    }
    /// Set an explicit calculation-space baseline.
    pub fn baseline(mut self, value: f64) -> Self {
        match &mut self.geom {
            Geom::Area { baseline, .. } => *baseline = value,
            Geom::Bar { .. } => self.bar_baseline = Some(value),
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Baseline applies to areas or bars.",
                ))
            }
        };
        self
    }
    pub(super) fn lower(
        &self,
        data: &Data,
        inherited: &AesBuilder,
        profile: Profile,
    ) -> ChartResult<(Layer, AesBuilder)> {
        if let Some(e) = &self.failure {
            return Err(e.clone());
        }
        if self.data.is_some() && self.input.is_some() {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "A layer takes either source data or a transform dependency.",
            ));
        }
        let mut mapped = self.mappings.merged(inherited, self.inherit);
        if matches!(
            self.geom,
            Geom::ShapeArc { .. }
                | Geom::ShapePie { .. }
                | Geom::ShapeLineRadial { .. }
                | Geom::ShapeAreaRadial { .. }
                | Geom::ShapeLinkRadial { .. }
        ) {
            mapped.x.get_or_insert(Mapping::Literal(0.));
            mapped.y.get_or_insert(Mapping::Literal(0.));
            if matches!(self.geom, Geom::ShapePie { .. })
                && mapped.group.is_none()
                && mapped.grouping.is_none()
            {
                mapped.all_groups = true;
            }
        }

        let orientation = self.orientation.unwrap_or_else(|| {
            let category = |value: &Option<Mapping>| {
                value
                    .as_ref()
                    .is_some_and(|v| matches!(v.resolve(data), Ok(Numeric::Category(_))))
            };
            if profile == Profile::Ggplot2_4_0_3
                && ((self.histogram.is_some() && mapped.x.is_none() && mapped.y.is_some())
                    || (matches!(self.geom, Geom::Bar { .. })
                        && category(&mapped.y)
                        && !category(&mapped.x)))
            {
                Orientation::Horizontal
            } else {
                Orientation::Vertical
            }
        });
        let transpose = |a: &mut AesBuilder| {
            std::mem::swap(&mut a.x, &mut a.y);
            std::mem::swap(&mut a.x2, &mut a.y2);
        };
        if orientation == Orientation::Horizontal {
            transpose(&mut mapped);
        }
        if mapped.y2.is_none() {
            mapped.y2 = self.bar_baseline.map(Mapping::Literal);
        }
        let id = self.id.clone()?;
        let mut layer = if let Some(bins) = self.histogram {
            let input = mapped
                .x
                .as_ref()
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::SchemaConflict,
                        "Histogram requires an x mapping.",
                    )
                })?
                .resolve(data)?;
            let mut layer = Layer::binned(id, data.id, Geom::Rectangle, BinAes::histogram());
            let grouping = mapped.resolved_grouping(data)?.unwrap_or(Grouping::All);
            layer.statistic = if let Some(edges) = &self.edges {
                let mut spec = BinSpec::new(input, edges.clone());
                spec.grouping = grouping;
                Statistic::bin(spec)
            } else {
                let mut spec = AutoBinSpec::new(input);
                spec.bins = bins;
                spec.grouping = grouping;
                Statistic::auto_bin(spec)
            };
            layer
        } else {
            Layer::new(id, data.id, self.geom, mapped.resolve(data)?)
        };
        layer.hierarchy = self
            .hierarchy
            .clone()
            .map(|recipe| recipe.try_map_fields(|field| field.field(data)))
            .transpose()?;
        layer.geography = self
            .geography
            .as_ref()
            .map(|(collection, join, operation)| {
                Ok(GeoLayerSpec {
                    collection: collection.clone(),
                    join: join.field(data)?,
                    operation: *operation,
                    default_color: !self.explicit_color,
                    default_line_width: !self.explicit_line_width,
                })
            })
            .transpose()?;
        layer.style = self.style;
        if matches!(
            self.recipe,
            Some(
                BuiltinRecipe::Polygon(_)
                    | BuiltinRecipe::Tile(_)
                    | BuiltinRecipe::Hexagon(_)
                    | BuiltinRecipe::Raster(_)
            )
        ) {
            if self.explicit_color && layer.style.stroke.is_none() {
                layer.style.stroke = Some(layer.style.color);
            }
            if !self.explicit_line_width {
                layer.style.stroke_width = if matches!(self.recipe, Some(BuiltinRecipe::Tile(_))) {
                    0.2
                } else {
                    0.5
                };
            }
        }
        layer.after_scale = self.after_scale.clone();
        layer.symbol = self
            .symbol
            .as_ref()
            .map(|(input, encoding)| {
                let mut encoding = encoding.clone();
                if let Some(input) = input {
                    encoding.input = ColorInput::Category(input.field(data)?);
                }
                Ok(encoding)
            })
            .transpose()?;
        layer.symbol_size_guide = self.symbol_size_guide.clone();
        layer.shape_protocols = self.shape_protocols.clone();
        for (target, (input, scale)) in &self.numeric_scales {
            let input = match input {
                NumericScaleInput::Source(input) => {
                    if scale.categorical() {
                        ColorInput::Category(input.field(data)?)
                    } else {
                        ColorInput::Numeric(input.resolve(data)?)
                    }
                }
                NumericScaleInput::Statistical(field) => ColorInput::Statistical(field.clone()),
            };
            layer.numeric_scales.insert(
                *target,
                crate::grammar::NumericEncoding {
                    id: crate::ScaleId::new(fresh_id()?),
                    input,
                    scale: scale.clone(),
                },
            );
        }
        layer.text = self.text.clone();
        layer.annotation = self.annotation.clone();
        layer.aesthetic_values = self.aesthetic_values.clone();
        for (target, (input, scale)) in &self.value_scales {
            let input = match input {
                NumericScaleInput::Source(input) if scale.categorical() => {
                    ColorInput::Category(input.field(data)?)
                }
                NumericScaleInput::Source(input) => ColorInput::Numeric(input.resolve(data)?),
                NumericScaleInput::Statistical(field) => ColorInput::Statistical(field.clone()),
            };
            layer.value_scales.insert(
                *target,
                NumericEncoding {
                    id: crate::ScaleId::new(fresh_id()?),
                    input,
                    scale: scale.clone(),
                },
            );
        }
        let implicit_sum =
            if self.stat.is_none() && matches!(self.recipe, Some(BuiltinRecipe::Count(_))) {
                Some(
                    super::count()
                        .sum_count()
                        .x(mapped.x.clone().ok_or_else(|| {
                            error(DiagnosticCode::Validation, "Count recipe requires x.")
                        })?)
                        .y(mapped.y.clone().ok_or_else(|| {
                            error(DiagnosticCode::Validation, "Count recipe requires y.")
                        })?),
                )
            } else {
                None
            };
        let implicit_align = (profile == Profile::Ggplot2_4_0_3
            && matches!(self.geom, Geom::Area { .. })
            && self.stat.is_none())
        .then(super::align_stat);
        let implicit_count = (profile == Profile::Ggplot2_4_0_3
            && matches!(self.geom, Geom::Bar { .. })
            && mapped.y.is_none()
            && self.stat.is_none())
        .then(super::count);
        if let Some(stat) = self
            .stat
            .as_ref()
            .or(implicit_sum.as_ref())
            .or(implicit_count.as_ref())
            .or(implicit_align.as_ref())
        {
            layer.statistic = stat.lower(data, &mapped)?;
            if self.generated.is_none()
                && let Some(mappings) = stat.default_mappings(layer.geom)?
            {
                layer.mappings = mappings;
            }
        }
        if orientation == Orientation::Horizontal {
            crate::grammar::orientation::transpose_mappings(&mut layer.mappings);
            transpose(&mut mapped);
        }
        layer.orientation = orientation;
        if let Some(mappings) = &self.generated {
            layer.mappings = mappings.clone();
        }
        if matches!(self.recipe, Some(BuiltinRecipe::Smooth)) {
            for (channel, field) in [
                (RecipeAesthetic::Lower, StatField::Lower),
                (RecipeAesthetic::Upper, StatField::Upper),
            ] {
                layer
                    .recipe_aes
                    .entry(channel)
                    .or_insert(ColorInput::Statistical(field));
            }
        }
        if matches!(layer.statistic.parameters, StatParameters::Spatial(_)) {
            let fields: &[(RecipeAesthetic, StatField)] = match self.recipe {
                Some(BuiltinRecipe::Tile(_) | BuiltinRecipe::Hexagon(_)) => &[
                    (RecipeAesthetic::Width, StatField::Width),
                    (RecipeAesthetic::Height, StatField::Height),
                ],
                Some(BuiltinRecipe::Polygon(_)) => {
                    &[(RecipeAesthetic::Subgroup, StatField::Subgroup)]
                }
                _ => &[],
            };
            for (channel, field) in fields {
                layer
                    .recipe_aes
                    .entry(*channel)
                    .or_insert_with(|| ColorInput::Statistical(field.clone()));
            }
        }
        if matches!(layer.statistic.parameters, StatParameters::Distribution(_)) {
            let fields: &[(RecipeAesthetic, StatField)] = match self.recipe {
                Some(BuiltinRecipe::Boxplot(_)) => &[
                    (RecipeAesthetic::Lower, StatField::Lower),
                    (RecipeAesthetic::Upper, StatField::Upper),
                    (RecipeAesthetic::Middle, StatField::Middle),
                    (RecipeAesthetic::WhiskerLower, StatField::WhiskerLower),
                    (RecipeAesthetic::WhiskerUpper, StatField::WhiskerUpper),
                    (RecipeAesthetic::NotchLower, StatField::NotchLower),
                    (RecipeAesthetic::NotchUpper, StatField::NotchUpper),
                    (RecipeAesthetic::Width, StatField::Width),
                    (RecipeAesthetic::RelativeWidth, StatField::RelativeWidth),
                ],
                Some(BuiltinRecipe::Violin(_)) => &[
                    (RecipeAesthetic::Width, StatField::Width),
                    (RecipeAesthetic::ViolinWidth, StatField::ViolinWidth),
                    (RecipeAesthetic::QuantileFlag, StatField::QuantileFlag),
                ],
                Some(BuiltinRecipe::Dotplot(_)) => &[
                    (RecipeAesthetic::Count, StatField::WeightedCount),
                    (RecipeAesthetic::BinWidth, StatField::BinWidth),
                    (RecipeAesthetic::Width, StatField::Width),
                ],
                _ => &[],
            };
            for (channel, field) in fields {
                layer
                    .recipe_aes
                    .entry(*channel)
                    .or_insert_with(|| ColorInput::Statistical(field.clone()));
            }
        }
        layer.filters = self
            .filters
            .iter()
            .map(|f| f.lower(data))
            .collect::<ChartResult<_>>()?;
        if let Some(position) = &self.position {
            layer.position = position.lower()?;
        } else if profile == Profile::Ggplot2_4_0_3
            && matches!(self.geom, Geom::Area { .. })
            && !matches!(self.recipe, Some(BuiltinRecipe::Density(_)))
        {
            layer.position = Position::GgplotStack(GgplotStackSpec::default());
        }
        layer.recipe = self.recipe.clone();
        if self.recipe.is_none()
            && matches!(layer.geom, Geom::Line { .. })
            && let StatParameters::Univariate(spec) = &layer.statistic.parameters
            && let UnivariateKind::Connect { connection } = &spec.kind
        {
            let direction = match connection {
                Connection::Hv => Some(StepDirection::Hv),
                Connection::Vh => Some(StepDirection::Vh),
                Connection::Mid => Some(StepDirection::Mid),
                Connection::Matrix(_) => None,
            };
            if let Some(direction) = direction {
                let recipe = step(direction);
                layer.geom = recipe.geom;
                layer.recipe = recipe.recipe;
            }
        }

        for (channel, input) in &self.recipe_aes {
            let input = match input {
                NumericScaleInput::Source(mapping) => {
                    if *channel == RecipeAesthetic::Subgroup {
                        ColorInput::Category(mapping.field(data)?)
                    } else {
                        ColorInput::Numeric(mapping.resolve(data)?)
                    }
                }
                NumericScaleInput::Statistical(field) => ColorInput::Statistical(field.clone()),
            };
            layer.recipe_aes.insert(*channel, input);
        }
        layer.scope = self.scope;
        layer.facet = self.facet.clone();
        layer.clip = self.clip;
        layer.invalid = self.invalid;
        layer.legend = self.legend.clone();
        layer.geometry_extension = self.extension.clone();
        layer.candle_colors = self.candle_colors;
        layer.inherit = false;
        Ok((layer, mapped))
    }
}
/// Train mapped scales without painting marks; x and y mappings are optional.
pub fn blank() -> LayerBuilder {
    LayerBuilder::new(Geom::Blank)
}
/// Circular source points.
pub fn points() -> LayerBuilder {
    LayerBuilder::new(Geom::Point)
}
/// Lines ordered by x, split at missing values, with explicit grouping.
pub fn line() -> LayerBuilder {
    LayerBuilder::new(Geom::line())
}
/// Filled runs against a zero baseline.
pub fn area() -> LayerBuilder {
    LayerBuilder::new(Geom::area())
}
/// Filled intervals between mapped y and y2.
pub fn ribbon() -> LayerBuilder {
    LayerBuilder::new(Geom::ribbon())
}
/// Signed interval bars with a zero baseline and width 8 destination units.
pub fn bars() -> LayerBuilder {
    LayerBuilder::new(Geom::Bar {
        width: 8.,
        nonnegative: false,
    })
    .baseline(0.)
}
/// Nonnegative volume bars with a zero baseline.
pub fn volume() -> LayerBuilder {
    LayerBuilder::new(Geom::Bar {
        width: 8.,
        nonnegative: true,
    })
    .baseline(0.)
}
/// Supplied open/high/low/close candles; y=open, y2=close.
pub fn ohlc() -> LayerBuilder {
    LayerBuilder::new(Geom::Ohlc { width: 8. })
}
/// A line segment between mapped x/y and x2/y2 endpoints.
pub fn rule() -> LayerBuilder {
    LayerBuilder::new(Geom::Rule)
}
/// Rectangles between mapped x/y and x2/y2 endpoints.
pub fn rectangle() -> LayerBuilder {
    LayerBuilder::new(Geom::Rectangle)
}
/// Colored rectangular cells using the same rectangle and color engines.
pub fn cells() -> LayerBuilder {
    rectangle()
}
/// Thirty equal-width bins using the existing shared histogram statistic.
pub fn histogram() -> LayerBuilder {
    let mut layer = LayerBuilder::new(Geom::Rectangle);
    layer.histogram = Some(30);
    layer
}

/// Source or generated input to a numeric aesthetic scale.
#[derive(Clone, Debug)]
pub enum NumericScaleInput {
    /// Resolve a named/source expression against the layer dataset.
    Source(super::Mapping),
    /// Read a field produced by the layer statistic.
    Statistical(crate::grammar::StatField),
}
impl From<&str> for NumericScaleInput {
    fn from(v: &str) -> Self {
        Self::Source(v.into())
    }
}
impl From<super::Mapping> for NumericScaleInput {
    fn from(v: super::Mapping) -> Self {
        Self::Source(v)
    }
}
impl From<crate::grammar::Expression<super::Mapping>> for NumericScaleInput {
    fn from(value: crate::grammar::Expression<super::Mapping>) -> Self {
        Self::Source(value.into())
    }
}
impl From<crate::grammar::StatField> for NumericScaleInput {
    fn from(v: crate::grammar::StatField) -> Self {
        Self::Statistical(v)
    }
}
impl LayerBuilder {
    /// Map size, opacity or stroke width through a shared typed scale.
    pub fn numeric_scale(
        mut self,
        target: crate::grammar::NumericAesthetic,
        input: impl Into<NumericScaleInput>,
        scale: crate::scales::MappedScaleSpec,
    ) -> Self {
        self.numeric_scales.insert(target, (input.into(), scale));
        self
    }
}

/// D3 line route: authored order, defined gaps and exact curve degeneracy.
pub fn shape_line() -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeLine {
        order: LineOrder::Authored,
        connect_gaps: false,
        curve: crate::shape::CurveSpec::Linear,
    })
}
/// D3 general area route: explicit x/y lower and x2/y2 upper boundary mappings.
pub fn shape_area() -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeArea {
        order: LineOrder::Authored,
        connect_gaps: false,
        curve: crate::shape::CurveSpec::Linear,
    })
}

/// Circular arc marks centered on x/y (zero by default); radii use destination units.
pub fn shape_arc() -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeArc {
        parameters: default_arc_parameters(),
    })
}
/// Pie weights mapped by PieValue, with source grouping independent of slice color.
pub fn shape_pie() -> LayerBuilder {
    LayerBuilder::new(Geom::ShapePie {
        parameters: default_arc_parameters(),
        angles: crate::shape::PieAngles::default(),
        order: crate::shape::PieOrder::default(),
        grouped: true,
    })
}
fn default_arc_parameters() -> crate::shape::ArcParameters {
    crate::shape::ArcParameters {
        datum: crate::shape::ArcDatum {
            inner_radius: 0.,
            outer_radius: 40.,
            start_angle: 0.,
            end_angle: std::f64::consts::TAU,
            pad_angle: 0.,
        },
        corner_radius: 0.,
        pad_radius: None,
    }
}
impl From<super::FieldHandle> for NumericScaleInput {
    fn from(value: super::FieldHandle) -> Self {
        Self::Source(Mapping::Handle(value))
    }
}
impl From<f64> for NumericScaleInput {
    fn from(value: f64) -> Self {
        Self::Source(Mapping::Literal(value))
    }
}
impl LayerBuilder {
    /// Map a finite shape parameter without normalization; named numeric scales remain available.
    pub fn shape_value(
        self,
        target: NumericAesthetic,
        input: impl Into<NumericScaleInput>,
    ) -> Self {
        self.numeric_scale(
            target,
            input,
            crate::scales::MappedScaleSpec::authored(crate::scales::ScaleFunctionSpec::Continuous(
                crate::scales::ContinuousScaleSpec::d3(crate::scales::NumericFamily::Linear),
            )),
        )
    }
    /// Set constant arc parameters; a pie layout owns the resulting start/end/pad angles.
    pub fn arc_parameters(mut self, parameters: crate::shape::ArcParameters) -> Self {
        match &mut self.geom {
            Geom::ShapeArc { parameters: p } | Geom::ShapePie { parameters: p, .. } => {
                *p = parameters
            }
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Arc parameters require shape_arc or shape_pie.",
                ))
            }
        }
        self
    }
    /// Set pie-wide sweep and padding, replacing the default full turn.
    pub fn pie_angles(mut self, angles: crate::shape::PieAngles) -> Self {
        match &mut self.geom {
            Geom::ShapePie { angles: a, .. } => *a = angles,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Pie angles require shape_pie.",
                ))
            }
        }
        self
    }
    /// Partition weights by row group; false combines generated category counts into one pie.
    pub fn pie_grouped(mut self, grouped: bool) -> Self {
        match &mut self.geom {
            Geom::ShapePie { grouped: g, .. } => *g = grouped,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Pie grouping requires shape_pie.",
                ))
            }
        }
        self
    }
    /// Set stable angular ordering, preserving source-array output and identity.
    pub fn pie_order(mut self, order: crate::shape::PieOrder) -> Self {
        self.shape_protocols.remove(&ShapeFamily::PieComparator);
        match &mut self.geom {
            Geom::ShapePie { order: o, .. } => *o = order,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Pie order requires shape_pie.",
                ))
            }
        }
        self
    }
}

/// Area/stroke-size points using the complete D3 symbol set; legacy points keep radius units.
pub fn shape_symbol() -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeSymbol {
        kind: crate::shape::SymbolKind::Circle,
        size: 64.,
        paint: crate::shape::SymbolPaint::Auto,
    })
}
impl LayerBuilder {
    /// Select one symbol type for this area-symbol layer.
    pub fn symbol_kind(mut self, kind: crate::shape::SymbolKind) -> Self {
        self.shape_protocols.remove(&ShapeFamily::Symbol);
        if let Geom::ShapeSymbol { kind: target, .. } = &mut self.geom {
            *target = kind;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Symbol type requires shape_symbol.",
            ));
        }
        self
    }
    /// Set constant area/stroke size, separate from legacy point radius.
    pub fn symbol_size(mut self, size: f64) -> Self {
        if let Geom::ShapeSymbol { size: target, .. } = &mut self.geom {
            *target = size;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Symbol area size requires shape_symbol.",
            ));
        }
        self
    }
    /// Choose automatic topology, explicit fill or explicit stroke painting.
    pub fn symbol_paint(mut self, paint: crate::shape::SymbolPaint) -> Self {
        if let Geom::ShapeSymbol { paint: target, .. } = &mut self.geom {
            *target = paint;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Symbol painting requires shape_symbol.",
            ));
        }
        self
    }
    /// Map exact source categories to an explicitly ordered symbol domain and palette.
    pub fn symbol_types(
        mut self,
        input: impl Into<Mapping>,
        domain: Vec<String>,
        palette: Vec<crate::shape::SymbolKind>,
    ) -> Self {
        self.shape_protocols.remove(&ShapeFamily::Symbol);
        self.symbol = Some((
            Some(input.into()),
            SymbolEncoding {
                input: ColorInput::Group,
                domain,
                palette,
                missing: None,
                title: None,
            },
        ));
        self
    }
    /// Map retained source/statistical group labels through the same explicit symbol catalog.
    pub fn symbol_groups(
        mut self,
        domain: Vec<String>,
        palette: Vec<crate::shape::SymbolKind>,
    ) -> Self {
        self.shape_protocols.remove(&ShapeFamily::Symbol);
        self.symbol = Some((
            None,
            SymbolEncoding {
                input: ColorInput::Group,
                domain,
                palette,
                missing: None,
                title: None,
            },
        ));
        self
    }
    /// Configure unknown/null category handling after selecting a symbol catalog.
    pub fn symbol_missing(mut self, kind: Option<crate::shape::SymbolKind>) -> Self {
        if let Some((_, mapping)) = &mut self.symbol {
            mapping.missing = kind;
        } else {
            self.failure = Some(error(
                DiagnosticCode::Validation,
                "Select a symbol catalog before its missing type.",
            ));
        }
        self
    }
    /// Set the mapped type guide title; an empty title omits only the title.
    pub fn symbol_title(mut self, title: impl Into<String>) -> Self {
        if let Some((_, mapping)) = &mut self.symbol {
            mapping.title = Some(title.into());
        } else {
            self.failure = Some(error(
                DiagnosticCode::Validation,
                "Select a symbol catalog before its title.",
            ));
        }
        self
    }
    /// Show domain samples evaluated through the actual AreaSize numeric mapping.
    pub fn symbol_size_guide(mut self, title: impl Into<String>, values: Vec<f64>) -> Self {
        self.symbol_size_guide = Some(SymbolSizeGuide {
            title: title.into(),
            values,
        });
        self
    }
}

/// Authored radial line; x/y is its shared center, zero when absent.
pub fn shape_line_radial() -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeLineRadial {
        order: LineOrder::Authored,
        connect_gaps: false,
        curve: crate::shape::CurveSpec::Linear,
        parameters: crate::grammar::RadialParameters {
            inner_radius: 40.,
            outer_radius: None,
            ..Default::default()
        },
    })
}
/// Authored radial area with named inner/outer and start/end boundaries.
pub fn shape_area_radial() -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeAreaRadial {
        order: LineOrder::Authored,
        connect_gaps: false,
        curve: crate::shape::CurveSpec::Linear,
        parameters: Default::default(),
    })
}
/// One Cartesian source edge per row, with both endpoints projected before curving.
pub fn shape_link(curve: crate::shape::CurveSpec) -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeLink { curve })
}
/// Cartesian links with horizontal endpoint tangents.
pub fn shape_link_horizontal() -> LayerBuilder {
    shape_link(crate::shape::CurveSpec::BumpX)
}
/// Cartesian links with vertical endpoint tangents.
pub fn shape_link_vertical() -> LayerBuilder {
    shape_link(crate::shape::CurveSpec::BumpY)
}
/// One radial edge per row; named polar endpoints share the x/y center.
pub fn shape_link_radial() -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeLinkRadial {
        parameters: Default::default(),
    })
}
impl LayerBuilder {
    /// Set constant radial parameters; lines consume start_angle/inner_radius only.
    pub fn radial_parameters(mut self, parameters: crate::grammar::RadialParameters) -> Self {
        match &mut self.geom {
            Geom::ShapeLineRadial { parameters: p, .. }
            | Geom::ShapeAreaRadial { parameters: p, .. }
            | Geom::ShapeLinkRadial { parameters: p } => *p = parameters,
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Radial parameters require a radial shape route.",
                ))
            }
        }
        self
    }
}

/// Explicit hierarchy recipe with temporary primary source selectors.
pub fn hierarchy(recipe: HierarchyRecipe<Mapping>) -> LayerBuilder {
    let mut layer = LayerBuilder::new(Geom::Hierarchy);
    layer.hierarchy = Some(recipe);
    layer
}
fn hierarchy_default(
    id: Mapping,
    parent: Mapping,
    layout: crate::hierarchy::LayoutSpec,
    projection: HierarchyProjection,
) -> LayerBuilder {
    let mut layer = LayerBuilder::new(Geom::Hierarchy);
    let identity = crate::HierarchyId::new(layer.id.as_ref().map_or(0, |id| id.get()));
    layer.hierarchy = Some(HierarchyRecipe {
        identity,
        source: HierarchySource::Table {
            id: Some(id),
            parent: Some(parent),
        },
        aggregation: HierarchyAggregation::Count,
        label: None,
        order: HierarchyOrder::Input,
        layout,
        projection,
        limits: crate::hierarchy::HierarchyLimits::default(),
    });
    layer
}
/// Panel-fitted tidy tree with caller-stable row identities.
pub fn hierarchy_tree(id: impl Into<Mapping>, parent: impl Into<Mapping>) -> LayerBuilder {
    hierarchy_default(
        id.into(),
        parent.into(),
        crate::hierarchy::LayoutSpec::Tree {
            options: Default::default(),
            separation: None,
        },
        HierarchyProjection::Cartesian,
    )
}
/// Panel-fitted dendrogram with leaf-aligned depth coordinates.
pub fn hierarchy_cluster(id: impl Into<Mapping>, parent: impl Into<Mapping>) -> LayerBuilder {
    hierarchy_default(
        id.into(),
        parent.into(),
        crate::hierarchy::LayoutSpec::Cluster {
            options: Default::default(),
            separation: None,
        },
        HierarchyProjection::Cartesian,
    )
}
/// Partition rectangles with equal depth steps (explicit aggregation can replace leaf count).
pub fn hierarchy_icicle(id: impl Into<Mapping>, parent: impl Into<Mapping>) -> LayerBuilder {
    hierarchy_default(
        id.into(),
        parent.into(),
        crate::hierarchy::LayoutSpec::Partition(Default::default()),
        HierarchyProjection::Cartesian,
    )
}
/// Partition arcs with area-based depth bands and a 16-unit central hole.
pub fn hierarchy_sunburst(id: impl Into<Mapping>, parent: impl Into<Mapping>) -> LayerBuilder {
    hierarchy_icicle(id, parent).hierarchy_projection(HierarchyProjection::Sunburst {
        inner_radius: 16.,
        radius: HierarchyRadius::Area,
    })
}
/// Panel-fitted treemap using default squarify and explicit leaf counts.
pub fn hierarchy_treemap(id: impl Into<Mapping>, parent: impl Into<Mapping>) -> LayerBuilder {
    hierarchy_default(
        id.into(),
        parent.into(),
        crate::hierarchy::LayoutSpec::Treemap {
            options: Default::default(),
            history: true,
            padding_sides: Default::default(),
            padding: None,
            tiler: None,
        },
        HierarchyProjection::Cartesian,
    )
}
/// Panel-fitted circle packing; radius retains its layout meaning.
pub fn hierarchy_pack(id: impl Into<Mapping>, parent: impl Into<Mapping>) -> LayerBuilder {
    hierarchy_default(
        id.into(),
        parent.into(),
        crate::hierarchy::LayoutSpec::Pack {
            options: Default::default(),
            radius: None,
            padding: None,
        },
        HierarchyProjection::Cartesian,
    )
}
impl LayerBuilder {
    fn hierarchy_change(mut self, change: impl FnOnce(&mut HierarchyRecipe<Mapping>)) -> Self {
        if let Some(recipe) = &mut self.hierarchy {
            change(recipe);
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Hierarchy controls require a hierarchy layer.",
            ));
        }
        self
    }
    /// Sum this source field, including own internal-node values.
    pub fn hierarchy_value(self, field: impl Into<Mapping>) -> Self {
        self.hierarchy_change(|r| r.aggregation = HierarchyAggregation::Sum(field.into()))
    }
    /// Supply any typed aggregation, including a registered Rust accessor.
    pub fn hierarchy_aggregation(self, value: HierarchyAggregation<Mapping>) -> Self {
        self.hierarchy_change(|r| r.aggregation = value)
    }
    /// Source label for semantic node inspection.
    pub fn hierarchy_label(self, field: impl Into<Mapping>) -> Self {
        self.hierarchy_change(|r| r.label = Some(field.into()))
    }
    /// Explicit topology fields, including slash paths with inferred ancestors.
    pub fn hierarchy_source(self, value: HierarchySource<Mapping>) -> Self {
        self.hierarchy_change(|r| r.source = value)
    }
    /// Replace all numerical layout controls; extent sizing remains panel-relative.
    pub fn hierarchy_layout(self, value: crate::hierarchy::LayoutSpec) -> Self {
        self.hierarchy_change(|r| r.layout = value)
    }
    /// Explicit Cartesian, horizontal, radial or sunburst projection.
    pub fn hierarchy_projection(self, value: HierarchyProjection) -> Self {
        self.hierarchy_change(|r| r.projection = value)
    }
    /// Stable sibling order before layout.
    pub fn hierarchy_order(self, value: HierarchyOrder) -> Self {
        self.hierarchy_change(|r| r.order = value)
    }
    /// Override node/depth/payload/work budgets.
    pub fn hierarchy_limits(self, value: crate::hierarchy::HierarchyLimits) -> Self {
        self.hierarchy_change(|r| r.limits = value)
    }
}

/// Vertical dependent-axis interval stem; lower/upper recipe channels are required.
pub fn linerange() -> LayerBuilder {
    LayerBuilder::new(Geom::Rule).recipe(BuiltinRecipe::Interval(IntervalRecipe {
        kind: IntervalKind::LineRange,
        width: None,
        ..Default::default()
    }))
}
/// Interval stem with a point at the mapped y anchor.
pub fn pointrange() -> LayerBuilder {
    LayerBuilder::new(Geom::Rule).recipe(BuiltinRecipe::Interval(IntervalRecipe {
        kind: IntervalKind::PointRange,
        width: None,
        ..Default::default()
    }))
}
/// Interval stem and caps with reference resolution-based widths.
pub fn errorbar() -> LayerBuilder {
    LayerBuilder::new(Geom::Rule).recipe(BuiltinRecipe::Interval(IntervalRecipe {
        kind: IntervalKind::ErrorBar,
        width: None,
        ..Default::default()
    }))
}
/// Interval box with a central mapped y rule.
pub fn crossbar() -> LayerBuilder {
    LayerBuilder::new(Geom::Rule).recipe(BuiltinRecipe::Interval(IntervalRecipe {
        kind: IntervalKind::Crossbar,
        width: None,
        ..Default::default()
    }))
}
/// Data-space sloped reference line clipped by the resolved panel.
pub fn abline(slope: f64, intercept: f64) -> LayerBuilder {
    LayerBuilder::new(Geom::Point)
        .independent()
        .recipe(BuiltinRecipe::Reference(ReferenceRecipe {
            kind: ReferenceKind::Abline,
            slope,
            intercept,
            arrow: None,
        }))
        .aes(super::aes().x(0.).y(0.))
}
/// Data-space horizontal reference line.
pub fn hline(intercept: f64) -> LayerBuilder {
    LayerBuilder::new(Geom::Point)
        .independent()
        .recipe(BuiltinRecipe::Reference(ReferenceRecipe {
            kind: ReferenceKind::Horizontal,
            slope: 0.,
            intercept,
            arrow: None,
        }))
        .aes(super::aes().x(0.).y(0.))
}
/// Data-space vertical reference line.
pub fn vline(intercept: f64) -> LayerBuilder {
    LayerBuilder::new(Geom::Point)
        .independent()
        .recipe(BuiltinRecipe::Reference(ReferenceRecipe {
            kind: ReferenceKind::Vertical,
            slope: 0.,
            intercept,
            arrow: None,
        }))
        .aes(super::aes().x(0.).y(0.))
}
/// Reference step path over the existing checked step kernel.
pub fn step(direction: StepDirection) -> LayerBuilder {
    LayerBuilder::new(Geom::ShapeLine {
        order: LineOrder::X,
        connect_gaps: false,
        curve: match direction {
            StepDirection::Hv => crate::shape::CurveSpec::StepAfter,
            StepDirection::Vh => crate::shape::CurveSpec::StepBefore,
            StepDirection::Mid => crate::shape::CurveSpec::Step,
        },
    })
    .recipe(BuiltinRecipe::Step(direction))
}
/// Straight mapped endpoints, optionally decorated with shared arrows.
pub fn segment() -> LayerBuilder {
    LayerBuilder::new(Geom::Rule).recipe(BuiltinRecipe::Segment { arrow: None })
}
/// Boxplot with actual grouped hinges, whiskers, notches and observed outliers.
pub fn boxplot() -> LayerBuilder {
    rule()
        .recipe(BuiltinRecipe::Boxplot(Box::default()))
        .stat(super::boxplot_stat())
}
/// Reference density curve over the shared area geometry.
pub fn density() -> LayerBuilder {
    area()
        .recipe(BuiltinRecipe::Density(
            crate::grammar::DensityRecipe::default(),
        ))
        .stat(super::density_stat())
}
/// Violin with actual shared kernel density and panel normalization.
pub fn violin() -> LayerBuilder {
    line()
        .recipe(BuiltinRecipe::Violin(ViolinRecipe::default()))
        .stat(super::violin_stat())
}
/// Integer-weight dot bins with reference stacking geometry.
pub fn dotplot() -> LayerBuilder {
    points()
        .recipe(BuiltinRecipe::Dotplot(DotplotRecipe::default()))
        .stat(super::dotplot_stat())
}
/// Empirical cumulative distribution, drawn as horizontal-then-vertical steps.
pub fn ecdf() -> LayerBuilder {
    step(StepDirection::Hv).stat(super::ecdf_stat())
}
/// Sorted sample quantiles against a standard normal reference distribution.
pub fn qq() -> LayerBuilder {
    points().stat(super::qq_stat())
}
/// Reference quantile line through the first and third quartiles.
pub fn qq_line() -> LayerBuilder {
    line().stat(super::qq_line_stat())
}
/// Sample and draw a portable pure numeric function.
pub fn function_curve(function: AnalyticFunction) -> LayerBuilder {
    line().stat(super::function_stat(function))
}

/// Reference model smoother with a confidence ribbon when estimates are available.
pub fn smooth() -> LayerBuilder {
    line()
        .recipe(BuiltinRecipe::Smooth)
        .stat(super::smooth_stat())
}
/// Quantile-regression curves from the shared statistical model owner.
pub fn quantile() -> LayerBuilder {
    line().stat(super::quantile_stat())
}

/// Rectangular weighted spatial bins with shared tile geometry.
pub fn bin2d() -> LayerBuilder {
    points()
        .recipe(BuiltinRecipe::Tile(TileRecipe::default()))
        .stat(super::bin2d_stat())
}
/// Hexagonal weighted spatial bins with shared polygon geometry.
pub fn hex() -> LayerBuilder {
    points()
        .recipe(BuiltinRecipe::Hexagon(TileRecipe::default()))
        .stat(super::hex_stat())
}
/// Product Gaussian density contour paths.
pub fn density2d() -> LayerBuilder {
    shape_line().stat(super::density2d_stat())
}
/// Contour paths from source grid response values supplied through stat input.
pub fn contour() -> LayerBuilder {
    shape_line().stat(super::contour_stat())
}
/// Filled contour bands preserving compound holes.
pub fn contour_filled() -> LayerBuilder {
    points()
        .recipe(BuiltinRecipe::Polygon(PolygonRecipe::default()))
        .stat(super::contour_filled_stat())
}
/// Weighted covariance ellipse paths.
pub fn ellipse() -> LayerBuilder {
    shape_line().stat(super::ellipse_stat())
}
