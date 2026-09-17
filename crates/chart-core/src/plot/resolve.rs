use super::*;
#[derive(Clone)]
pub(super) struct AutomaticPaintScale {
    pub id: ScaleId,
    pub title: String,
    pub scale: ColorScale<crate::color::Paint>,
}
pub(super) struct LayerContext<'a> {
    pub axes: &'a BTreeMap<String, ScaleId>,
    pub color_ids: &'a mut BTreeMap<String, ScaleId>,
    pub color_scales: &'a BTreeMap<String, ColorScale<crate::color::Paint>>,
    pub ggplot_paint_scales:
        &'a mut BTreeMap<Option<crate::grammar::PaintAesthetic>, AutomaticPaintScale>,
    pub ggplot_numeric_ids:
        &'a mut BTreeMap<(crate::grammar::NumericAesthetic, crate::FieldId), ScaleId>,
    pub ggplot_style_ids:
        &'a mut BTreeMap<(crate::grammar::ValueAesthetic, crate::FieldId), ScaleId>,
}
pub(super) fn temporal_guide(
    spec: &mut crate::scales::MappedScaleSpec,
    input: &crate::grammar::ColorInput,
    data: &Data,
) -> ChartResult<()> {
    use crate::{
        grammar::{ColorInput, Numeric},
        scales::*,
    };
    if spec.guide.is_some() {
        return Ok(());
    }
    if let ColorInput::Numeric(Numeric::Timestamp { field, origin }) = input {
        let (_, column) = data.batch.schema().field(*field).ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                "Timestamp guide field is absent.",
            )
        })?;
        if let crate::data::FieldKind::Timestamp(kind) = &column.kind {
            spec.timestamp_normalization(*origin, kind.unit, false)?;
            spec.guide = Some(Box::new(GgplotScaleGuide::Temporal(GgplotTemporalGuide {
                origin: *origin,
                unit: kind.unit,
                zone: CalendarZone::Utc,
                arguments: Default::default(),
            })));
        }
    }
    Ok(())
}

pub(super) fn default_numeric_scale(
    kind: crate::scales::GgplotNumericPalette,
    input: &crate::grammar::ColorInput,
    data: &Data,
) -> ChartResult<crate::scales::MappedScaleSpec> {
    let mut scale = crate::scales::ggplot_numeric_default(kind)?;
    temporal_guide(&mut scale, input, data)?;
    Ok(scale)
}

pub(super) fn default_value_scale(
    target: crate::grammar::ValueAesthetic,
) -> ChartResult<crate::scales::MappedScaleSpec> {
    use crate::{grammar::ValueAesthetic as V, scales::*};
    let palette = match target {
        V::Shape => GgplotDiscretePalette::Shape { solid: true },
        V::LineType => GgplotDiscretePalette::LineType,
        _ => unreachable!("reference discrete style"),
    };
    MappedScaleSpec::authored(ScaleFunctionSpec::Ordinal(OrdinalSpec::default()))
        .with_ggplot(GgplotScalePolicy::Discrete {
            empty_population: false,
            limits: None,
            levels: None,
            drop: true,
            na_translate: true,
            palette,
        })?
        .with_theme_palette(vec![
            match target {
                V::Shape => "shape",
                V::LineType => "linetype",
                _ => unreachable!("reference discrete style"),
            }
            .into(),
        ])
}
impl LayerContext<'_> {
    pub fn apply(
        &mut self,
        definition: &mut ChartDefinition,
        layer: &mut crate::grammar::Layer,
        builder: &LayerBuilder,
        mapping: &AesBuilder,
        data: &Data,
    ) -> ChartResult<()> {
        if definition.profile() != Profile::Ggplot2_4_0_3
            && (mapping.shape.is_some()
                || mapping.linetype.is_some()
                || mapping.alpha.is_some()
                || mapping.linewidth.is_some())
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Automatic reference style mappings require the ggplot profile; use an explicit value scale with other profiles.",
            ));
        }
        if definition.profile() == Profile::Ggplot2_4_0_3 {
            use crate::grammar::ValueAesthetic as V;
            for (target, input) in [(V::Shape, &mapping.shape), (V::LineType, &mapping.linetype)] {
                let Some(input) = input else { continue };
                if (target == V::Shape
                    && !layer.reference_point()
                    && !matches!(
                        layer.recipe,
                        Some(crate::grammar::BuiltinRecipe::Interval(
                            crate::grammar::IntervalRecipe {
                                kind: crate::grammar::IntervalKind::PointRange,
                                ..
                            }
                        ))
                    ))
                    || (target == V::LineType
                        && matches!(
                            layer.geom,
                            crate::grammar::Geom::Point | crate::grammar::Geom::ShapeSymbol { .. }
                        ))
                {
                    continue;
                }
                if layer.value_scales.contains_key(&target)
                    || layer.aesthetic_values.contains_key(&target)
                    || (target == V::LineType && layer.style.line_type.is_some())
                {
                    continue;
                }
                let field = input.field(data)?;
                if !data.batch.schema().field(field).is_some_and(|(_, f)| {
                    matches!(
                        f.kind,
                        crate::data::FieldKind::Categorical
                            | crate::data::FieldKind::Utf8
                            | crate::data::FieldKind::Boolean
                    )
                }) {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Default shape and linetype scales require discrete fields.",
                    ));
                }
                let id = match self.ggplot_style_ids.entry((target, field)) {
                    std::collections::btree_map::Entry::Occupied(e) => *e.get(),
                    std::collections::btree_map::Entry::Vacant(e) => {
                        *e.insert(ScaleId::new(fresh_id()?))
                    }
                };
                layer.value_scales.insert(
                    target,
                    crate::grammar::NumericEncoding {
                        id,
                        input: ColorInput::Category(field),
                        scale: default_value_scale(target)?,
                    },
                );
            }
            use crate::grammar::NumericAesthetic as N;
            for (target, kind, input, constant) in [
                (
                    N::Alpha,
                    crate::scales::GgplotNumericPalette::Alpha,
                    &mapping.alpha,
                    layer.style.alpha.is_some(),
                ),
                (
                    N::StrokeWidth,
                    crate::scales::GgplotNumericPalette::Linewidth,
                    &mapping.linewidth,
                    builder.explicit_line_width,
                ),
            ] {
                let Some(input) = input else { continue };
                if constant || layer.numeric_scales.contains_key(&target) {
                    continue;
                }
                let field = input.field(data)?;
                let categorical = data.batch.schema().field(field).is_some_and(|(_, f)| {
                    matches!(
                        f.kind,
                        crate::data::FieldKind::Categorical
                            | crate::data::FieldKind::Utf8
                            | crate::data::FieldKind::Boolean
                    )
                });
                let id = match self.ggplot_numeric_ids.entry((target, field)) {
                    std::collections::btree_map::Entry::Occupied(e) => *e.get(),
                    std::collections::btree_map::Entry::Vacant(e) => {
                        *e.insert(ScaleId::new(fresh_id()?))
                    }
                };
                let input = if categorical {
                    ColorInput::Category(field)
                } else {
                    ColorInput::Numeric(input.resolve(data)?)
                };
                let scale = if categorical {
                    crate::scales::ggplot_numeric_ordinal(kind)?
                } else {
                    default_numeric_scale(kind, &input, data)?
                };
                layer
                    .numeric_scales
                    .insert(target, crate::grammar::NumericEncoding { id, input, scale });
            }
            if matches!(layer.recipe, Some(crate::grammar::BuiltinRecipe::Count(_)))
                && !builder.explicit_size
                && !builder.explicit_radius
                && mapping.size.is_none()
                && !layer
                    .numeric_scales
                    .contains_key(&crate::grammar::NumericAesthetic::Size)
            {
                let input = ColorInput::Statistical(crate::grammar::StatField::WeightedCount);
                let scale =
                    default_numeric_scale(crate::scales::GgplotNumericPalette::Size, &input, data)?;
                layer.numeric_scales.insert(
                    crate::grammar::NumericAesthetic::Size,
                    crate::grammar::NumericEncoding {
                        id: ScaleId::new(fresh_id()?),
                        input,
                        scale,
                    },
                );
            }
            let mut source = mapping.resolve(data)?;
            if mapping.all_groups {
                source = source.grouped(crate::grammar::Grouping::All);
            }
            if (layer.reference_point()
                || matches!(
                    layer.recipe,
                    Some(crate::grammar::BuiltinRecipe::Interval(
                        crate::grammar::IntervalRecipe {
                            kind: crate::grammar::IntervalKind::PointRange,
                            ..
                        }
                    ))
                ))
                && !builder.explicit_size
                && !builder.explicit_radius
                && !layer
                    .numeric_scales
                    .contains_key(&crate::grammar::NumericAesthetic::Size)
                && let Some(
                    input @ (crate::grammar::Numeric::Field(field)
                    | crate::grammar::Numeric::Category(field)
                    | crate::grammar::Numeric::Timestamp { field, .. }),
                ) = &source.size
            {
                let id = if let Some(id) = self
                    .ggplot_numeric_ids
                    .get(&(crate::grammar::NumericAesthetic::Size, *field))
                {
                    *id
                } else {
                    let id = ScaleId::new(fresh_id()?);
                    self.ggplot_numeric_ids
                        .insert((crate::grammar::NumericAesthetic::Size, *field), id);
                    id
                };
                let categorical = data
                    .batch
                    .schema()
                    .field(*field)
                    .is_some_and(|(_, column)| {
                        matches!(
                            column.kind,
                            crate::data::FieldKind::Categorical
                                | crate::data::FieldKind::Utf8
                                | crate::data::FieldKind::Boolean
                        )
                    });
                let input = if categorical {
                    ColorInput::Category(*field)
                } else {
                    ColorInput::Numeric(input.clone())
                };
                let scale = if categorical {
                    crate::scales::ggplot_numeric_ordinal(
                        crate::scales::GgplotNumericPalette::Size,
                    )?
                } else {
                    default_numeric_scale(crate::scales::GgplotNumericPalette::Size, &input, data)?
                };
                layer.numeric_scales.insert(
                    crate::grammar::NumericAesthetic::Size,
                    crate::grammar::NumericEncoding { id, input, scale },
                );
            }
            if let Mappings::Source(aes) = &mut layer.mappings {
                *aes = source.clone();
            }
            layer.grammar = Some(crate::grammar::LayerGrammar {
                default_text: builder.default_text,
                default_radius: builder.explicit_radius.then_some(false),
                default_line_width: (builder.explicit_line_width
                    || matches!(
                        layer.recipe,
                        Some(
                            crate::grammar::BuiltinRecipe::Polygon(_)
                                | crate::grammar::BuiltinRecipe::Tile(_)
                                | crate::grammar::BuiltinRecipe::Raster(_)
                        )
                    ))
                .then_some(false),
                default_size: !builder.explicit_size,
                default_color: !builder.explicit_color,
                source,
                stat_grouping: builder
                    .stat
                    .as_ref()
                    .map(|s| s.explicit_grouping(data))
                    .transpose()?
                    .flatten(),
            });
        }
        if let Some((x, y)) = &builder.axes {
            layer.scales = crate::grammar::ScaleBindings {
                x: *self.axes.get(x).ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        format!("No axis named '{x}'."),
                    )
                })?,
                y: *self.axes.get(y).ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        format!("No axis named '{y}'."),
                    )
                })?,
            };
        }
        if builder.theme.patch != crate::theme::ThemePatch::default() {
            let theme = definition.theme.get_or_insert_with(|| theme().spec);
            theme
                .layers
                .entry(layer.id)
                .or_default()
                .overlay(&builder.theme.patch);
        }
        if let Some(input) = &builder.generated_color {
            let name = builder.generated_color_scale.as_ref().ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    "Generated color requires an explicit color_scale name.",
                )
            })?;
            let scale = self.color_scales.get(name).ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("No color scale named '{name}'."),
                )
            })?;
            layer.color = Some(ColorEncoding {
                automatic: false,
                id: self.color_ids[name],
                title: Some(name.clone()),
                input: input.clone(),
                scale: scale.clone(),
            });
        }
        for (channel, color, named_scale) in [
            (None, &mapping.color, &mapping.color_scale),
            (
                Some(crate::grammar::PaintAesthetic::Fill),
                &mapping.fill,
                &mapping.fill_scale,
            ),
            (
                Some(crate::grammar::PaintAesthetic::Stroke),
                &mapping.stroke,
                &mapping.stroke_scale,
            ),
        ] {
            if channel.is_none() && builder.generated_color.is_some() {
                continue;
            }
            let Some(color) = color else {
                continue;
            };
            let field = color.field(data)?;
            let (_, column) =
                data.batch.schema().field(field).ok_or_else(|| {
                    error(DiagnosticCode::MissingResource, "Color field is absent.")
                })?;
            let automatic_name = match channel {
                None => column.name.clone(),
                Some(crate::grammar::PaintAesthetic::Fill) => format!("fill:{}", column.name),
                Some(crate::grammar::PaintAesthetic::Stroke) => format!("stroke:{}", column.name),
            };
            let scale_name = named_scale.as_deref().unwrap_or(&automatic_name);
            let explicit = self.color_scales.get(scale_name);
            if named_scale.is_some() && explicit.is_none() {
                return Err(error(
                    DiagnosticCode::MissingResource,
                    format!("No color scale named '{scale_name}'."),
                ));
            }
            if definition.profile() != Profile::Ggplot2_4_0_3
                && explicit.is_none()
                && matches!(
                    column.kind,
                    crate::data::FieldKind::Float64 | crate::data::FieldKind::Timestamp(_)
                )
            {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Numeric color requires an explicit continuous color scale.",
                ));
            }
            let automatic = definition.profile() == Profile::Ggplot2_4_0_3
                && named_scale.is_none()
                && explicit.is_none();
            let prior = self.ggplot_paint_scales.get(&channel).filter(|_| automatic);
            let (id, title) = if let Some(prior) = prior {
                (prior.id, prior.title.clone())
            } else if automatic {
                let id = ScaleId::new(fresh_id()?);
                self.color_ids.insert(automatic_name.clone(), id);
                (id, column.name.clone())
            } else if let Some(id) = self.color_ids.get(scale_name) {
                (*id, column.name.clone())
            } else {
                let id = ScaleId::new(fresh_id()?);
                self.color_ids.insert(scale_name.to_owned(), id);
                (id, column.name.clone())
            };
            let scale = if let Some(prior) = prior {
                prior.scale.clone()
            } else if let Some(explicit) = explicit {
                explicit.clone()
            } else if definition.profile() == Profile::Ggplot2_4_0_3 {
                let mut scale = crate::scales::ggplot_color_default(matches!(
                    column.kind,
                    crate::data::FieldKind::Float64
                        | crate::data::FieldKind::Int64
                        | crate::data::FieldKind::UInt64
                        | crate::data::FieldKind::Timestamp(_)
                ))?;
                if let crate::scales::ColorScale::Mapped { scale, .. } = &mut scale {
                    if matches!(column.kind, crate::data::FieldKind::Timestamp(_)) {
                        // Date/datetime paint constructors have explicit gradients.
                        scale.palette_theme_aesthetics.clear();
                    } else if channel == Some(crate::grammar::PaintAesthetic::Fill) {
                        scale.palette_theme_aesthetics = vec!["fill".into()];
                    }
                }
                scale
            } else {
                default_color_scale()
            };
            let input = if matches!(layer.mappings, Mappings::Source(_))
                || matches!(
                    layer.statistic.parameters,
                    crate::grammar::StatParameters::Spatial(_)
                        | crate::grammar::StatParameters::Model(_)
                        | crate::grammar::StatParameters::Distribution(_)
                        | crate::grammar::StatParameters::Univariate(_)
                )
                || matches!(layer.recipe, Some(crate::grammar::BuiltinRecipe::Count(_)))
            {
                if !scale.is_categorical() {
                    ColorInput::Numeric(color.resolve(data)?)
                } else {
                    ColorInput::Category(field)
                }
            } else if definition.profile() == Profile::Ggplot2_4_0_3 {
                ColorInput::GroupField(field)
            } else if generated_group(definition, layer)
                == Some(crate::grammar::Grouping::Field(field))
            {
                ColorInput::Group
            } else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Generated layers must map color to their group or an explicit generated field.",
                ));
            };
            let mut scale = scale;
            if explicit.is_none()
                && definition.profile() == Profile::Ggplot2_4_0_3
                && let crate::scales::ColorScale::Mapped { scale, .. } = &mut scale
            {
                temporal_guide(scale, &input, data)?;
            }
            let encoding = ColorEncoding {
                automatic,
                id,
                title: Some(title),
                input,
                scale,
            };
            if automatic {
                self.ggplot_paint_scales
                    .entry(channel)
                    .or_insert_with(|| AutomaticPaintScale {
                        id,
                        title: encoding.title.clone().unwrap_or_default(),
                        scale: encoding.scale.clone(),
                    });
            }
            if let Some(channel) = channel {
                layer.paint_scales.insert(channel, encoding);
            } else {
                layer.color = Some(encoding);
            }
        }
        if definition.profile() == Profile::Ggplot2_4_0_3
            && mapping.fill.is_none()
            && builder.style.fill.is_none()
        {
            use crate::grammar::{PaintAesthetic, SpatialKind, StatField, StatParameters};
            let default = if let StatParameters::Spatial(spec) = &layer.statistic.parameters {
                match &spec.kind {
                    SpatialKind::Density { contour: None, .. } => {
                        Some((StatField::Density, false, "density"))
                    }

                    SpatialKind::Rectangular { summary, .. }
                    | SpatialKind::Hexagonal { summary, .. } => Some((
                        if summary.is_some() {
                            StatField::Mean
                        } else {
                            StatField::WeightedCount
                        },
                        false,
                        if summary.is_some() { "value" } else { "count" },
                    )),
                    SpatialKind::Contour { filled: true, .. }
                    | SpatialKind::Density { filled: true, .. } => {
                        Some((StatField::Level, true, "level"))
                    }
                    _ => None,
                }
            } else {
                None
            };
            if let Some((field, ordered, title)) = default {
                let channel = Some(PaintAesthetic::Fill);
                let name = format!("fill:{title}");
                let explicit = self.color_scales.get(&name);
                let automatic = explicit.is_none();
                let (id, scale) =
                    if let Some(scale) = explicit {
                        let id = if let Some(id) = self.color_ids.get(&name) {
                            *id
                        } else {
                            let id = ScaleId::new(fresh_id()?);
                            self.color_ids.insert(name.clone(), id);
                            id
                        };
                        (id, scale.clone())
                    } else if let Some(prior) = self.ggplot_paint_scales.get(&channel) {
                        (prior.id, prior.scale.clone())
                    } else {
                        let mut scale = if ordered {
                            crate::scales::ggplot_color_ordinal()?
                        } else {
                            crate::scales::ggplot_color_default(true)?
                        };
                        if !ordered
                            && let crate::scales::ColorScale::Mapped { scale, .. } = &mut scale
                        {
                            scale.palette_theme_aesthetics = vec!["fill".into()];
                            *scale = scale.clone().with_guide(
                                crate::scales::GgplotScaleGuide::Colorbar(Default::default()),
                            )?;
                        }
                        let id = ScaleId::new(fresh_id()?);
                        self.color_ids.insert(format!("fill:{title}"), id);
                        self.ggplot_paint_scales.insert(
                            channel,
                            AutomaticPaintScale {
                                id,
                                title: title.into(),
                                scale: scale.clone(),
                            },
                        );
                        (id, scale)
                    };
                layer.paint_scales.insert(
                    PaintAesthetic::Fill,
                    ColorEncoding {
                        automatic,
                        id,
                        title: Some(title.into()),
                        input: ColorInput::Statistical(field),
                        scale,
                    },
                );
            }
        }
        complete_count_partitions(layer);
        Ok(())
    }
}

/// StatSum partitions the mapped input values, independently of its normalization group.
fn complete_count_partitions(layer: &mut crate::grammar::Layer) {
    use crate::grammar::{ColorInput, Numeric, StatParameters};
    let mut fields = std::collections::BTreeSet::new();
    let mut numeric = Vec::new();
    let inputs = layer
        .color
        .iter()
        .map(|e| &e.input)
        .chain(layer.paint_scales.values().map(|e| &e.input))
        .chain(layer.numeric_scales.values().map(|e| &e.input))
        .chain(layer.value_scales.values().map(|e| &e.input))
        .chain(layer.symbol.iter().map(|e| &e.input))
        .chain(layer.recipe_aes.values());
    for input in inputs {
        match input {
            ColorInput::Category(field) => {
                fields.insert(*field);
            }
            ColorInput::Numeric(
                Numeric::Field(field) | Numeric::Category(field) | Numeric::Timestamp { field, .. },
            ) => {
                fields.insert(*field);
            }
            ColorInput::Numeric(value @ (Numeric::Expression(_) | Numeric::Scaled { .. }))
                if !numeric.contains(value) =>
            {
                numeric.push(value.clone());
            }
            _ => {}
        }
    }
    match &mut layer.statistic.parameters {
        StatParameters::Model(spec) => {
            spec.retained_fields = fields.iter().copied().collect();
            spec.retained_numeric = numeric.clone();
        }
        StatParameters::Spatial(spec) => {
            spec.retained_fields = fields.iter().copied().collect();
            spec.retained_numeric = numeric.clone();
        }
        StatParameters::Distribution(spec) => {
            spec.retained_fields = fields.iter().copied().collect();
            spec.retained_numeric = numeric.clone();
        }
        StatParameters::Univariate(spec) => {
            spec.retained_fields = fields.iter().copied().collect();
            spec.retained_numeric = numeric.clone();
        }
        _ => {}
    }
    if let StatParameters::Count(count) = &mut layer.statistic.parameters
        && let Some(options) = &mut count.ggplot
        && options.joint_position.is_some()
    {
        for field in fields {
            if !options.joint_aesthetics.contains(&field) {
                options.joint_aesthetics.push(field);
            }
        }
        for value in numeric {
            if !options.joint_numeric.contains(&value) {
                options.joint_numeric.push(value);
            }
        }
    }
}
