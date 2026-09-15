use super::*;
use crate::grammar::{DataRef, Layer, Numeric, SourceAes};

fn source_mapping(value: &Numeric, data: &Data) -> Mapping {
    match value {
        Numeric::Expression(expr) => Mapping::Expression(
            expr.try_map_reads(|read| Ok(source_mapping(&read.numeric(), data)))
                .expect("infallible source read conversion"),
        ),
        Numeric::Scaled { input, scale, .. } => Mapping::Scaled {
            input: Box::new(source_mapping(input, data)),
            scale: scale.as_ref().clone(),
        },
        Numeric::Field(id) | Numeric::Category(id) => Mapping::Handle(FieldHandle {
            dataset: data.id,
            field: *id,
        }),
        Numeric::Timestamp { field, origin } => Mapping::Timestamp {
            field: data
                .batch
                .schema()
                .field(*field)
                .map_or(String::new(), |(_, f)| f.name.clone()),
            origin: *origin,
        },
        Numeric::Literal(value) => Mapping::Literal(*value),
    }
}

fn source_mappings(aes: &SourceAes, data: &Data) -> AesBuilder {
    let mapping = |value: &Numeric| source_mapping(value, data);
    AesBuilder {
        grouping: aes.grouping.clone(),
        x: aes.x.as_ref().map(mapping),
        y: aes.y.as_ref().map(mapping),
        x2: aes.x2.as_ref().map(mapping),
        y2: aes.y2.as_ref().map(mapping),
        low: aes.low.as_ref().map(mapping),
        high: aes.high.as_ref().map(mapping),
        size: aes.size.as_ref().map(mapping),
        group: aes.group.map(|field| {
            Mapping::Handle(FieldHandle {
                dataset: data.id,
                field,
            })
        }),
        ..Default::default()
    }
}
impl PlotEditBuilder {
    fn update(mut self, apply: impl FnOnce(&mut Self) -> ChartResult<()>) -> Self {
        if self.failure.is_none()
            && let Err(e) = apply(&mut self)
        {
            self.failure = Some(e);
        }
        self
    }
    fn root_data(&self, mut input: DataRef) -> ChartResult<&Data> {
        for _ in 0..=self.definition.transforms.len() {
            match input {
                DataRef::Dataset(id) => {
                    return self
                        .original
                        .data
                        .iter()
                        .find(|data| data.id == id)
                        .ok_or_else(|| {
                            error(
                                DiagnosticCode::MissingResource,
                                "Definition edit names an unregistered dataset.",
                            )
                        });
                }
                DataRef::Transform(id) => {
                    input = self
                        .definition
                        .transforms
                        .iter()
                        .find(|t| t.id == id)
                        .ok_or_else(|| {
                            error(
                                DiagnosticCode::MissingResource,
                                "Definition edit names an absent transform.",
                            )
                        })?
                        .input
                }
            }
        }
        Err(error(
            DiagnosticCode::Validation,
            "Transform dependency cycle.",
        ))
    }
    fn color_scales(&self) -> BTreeMap<String, ColorScale<crate::color::Paint>> {
        self.original
            .colors
            .iter()
            .filter_map(|(name, id)| {
                self.definition
                    .layers
                    .iter()
                    .flat_map(|l| l.color.iter().chain(l.paint_scales.values()))
                    .find(|c| c.id == *id && !c.automatic)
                    .map(|c| (name.clone(), c.scale.clone()))
            })
            .collect()
    }
    /// Add or replace a complete layer component, preserving an existing name's identity.
    /// Omitted source mappings inherit that layer's resolved source mappings. Data must already
    /// belong to the plot; live data replacement remains a separate transaction.
    pub fn layer(self, name: impl Into<String>, mut builder: LayerBuilder) -> Self {
        let name = name.into();
        self.update(|this| {
            validate_name(&name)?;
            let existing = this
                .original
                .layers
                .get(&name)
                .and_then(|id| this.definition.layers.iter().find(|l| l.id == *id))
                .cloned();
            if let Some(old) = &existing {
                builder.id = Ok(old.id);
            }
            let input = if let Some(input) = &builder.input {
                DataRef::Transform(input.resolve(&this.original.transforms)?)
            } else if let Some(data) = &builder.data {
                DataRef::Dataset(data.id)
            } else {
                existing
                    .as_ref()
                    .map_or(DataRef::Dataset(this.original.data[0].id), |old| old.data)
            };
            let data = this.root_data(input)?.clone();
            let mut inherited = if let Some(Layer {
                mappings: Mappings::Source(aes),
                ..
            }) = &existing
            {
                source_mappings(aes, &data)
            } else if let Some(grammar) = existing.as_ref().and_then(|l| l.grammar.as_ref()) {
                source_mappings(&grammar.source, &data)
            } else {
                aes()
            };
            if let Some(color) = existing.as_ref().and_then(|l| l.color.as_ref()) {
                inherited.color = match &color.input {
                    ColorInput::Category(field) | ColorInput::GroupField(field) => {
                        Some(Mapping::Handle(FieldHandle {
                            dataset: data.id,
                            field: *field,
                        }))
                    }
                    ColorInput::Numeric(value) => {
                        source_mappings(&SourceAes::new().x(value.clone()), &data).x
                    }
                    _ => None,
                };
                inherited.color_scale = this
                    .original
                    .colors
                    .iter()
                    .find(|(_, id)| **id == color.id && !color.automatic)
                    .map(|(name, _)| name.clone());
            }
            if let Some(existing) = &existing {
                for (channel, color) in &existing.paint_scales {
                    let input = match &color.input {
                        ColorInput::Category(field) | ColorInput::GroupField(field) => {
                            Some(Mapping::Handle(FieldHandle {
                                dataset: data.id,
                                field: *field,
                            }))
                        }
                        ColorInput::Numeric(value) => {
                            source_mappings(&SourceAes::new().x(value.clone()), &data).x
                        }
                        _ => None,
                    };
                    let scale = this
                        .original
                        .colors
                        .iter()
                        .find(|(_, id)| **id == color.id && !color.automatic)
                        .map(|(name, _)| name.clone());
                    match channel {
                        crate::grammar::PaintAesthetic::Fill => {
                            inherited.fill = input;
                            inherited.fill_scale = scale;
                        }
                        crate::grammar::PaintAesthetic::Stroke => {
                            inherited.stroke = input;
                            inherited.stroke_scale = scale;
                        }
                    }
                }
            }
            if let Some(existing) = &existing {
                for target in [
                    crate::grammar::ValueAesthetic::Shape,
                    crate::grammar::ValueAesthetic::LineType,
                ] {
                    if let Some(encoding) = existing.value_scales.get(&target)
                        && encoding.scale == resolve::default_value_scale(target)?
                        && let ColorInput::Category(field) = encoding.input
                    {
                        let input = Some(Mapping::Handle(FieldHandle {
                            dataset: data.id,
                            field,
                        }));
                        match target {
                            crate::grammar::ValueAesthetic::Shape => inherited.shape = input,
                            _ => inherited.linetype = input,
                        }
                    }
                }
            }
            if let Some(existing) = &existing {
                for (target, kind) in [
                    (
                        crate::grammar::NumericAesthetic::Alpha,
                        crate::scales::GgplotNumericPalette::Alpha,
                    ),
                    (
                        crate::grammar::NumericAesthetic::StrokeWidth,
                        crate::scales::GgplotNumericPalette::Linewidth,
                    ),
                ] {
                    if let Some(encoding) = existing.numeric_scales.get(&target)
                        && (encoding.scale
                            == resolve::default_numeric_scale(kind, &encoding.input, &data)?
                            || encoding.scale == crate::scales::ggplot_numeric_ordinal(kind)?)
                        && let ColorInput::Category(field)
                        | ColorInput::Numeric(Numeric::Field(field))
                        | ColorInput::Numeric(Numeric::Timestamp { field, .. }) = encoding.input
                    {
                        let input = Some(match &encoding.input {
                            ColorInput::Numeric(value) => source_mapping(value, &data),
                            _ => Mapping::Handle(FieldHandle {
                                dataset: data.id,
                                field,
                            }),
                        });
                        match target {
                            crate::grammar::NumericAesthetic::Alpha => inherited.alpha = input,
                            _ => inherited.linewidth = input,
                        }
                    }
                }
            }
            let (mut layer, mapping) =
                builder.lower(&data, &inherited, this.definition.profile())?;
            layer.data = input;
            if matches!(input, DataRef::Transform(_))
                && builder.generated.is_none()
                && builder.stat.is_none()
                && let Some(old) = &existing
            {
                layer.mappings = old.mappings.clone();
            }
            if builder.axes.is_none()
                && let Some(old) = &existing
            {
                layer.scales = old.scales;
            }
            let color_scales = this.color_scales();
            let mut ggplot_paint_scales = BTreeMap::new();
            for existing in &this.definition.layers {
                for (channel, encoding) in existing
                    .color
                    .iter()
                    .map(|color| (None, color))
                    .chain(
                        existing
                            .paint_scales
                            .iter()
                            .map(|(channel, color)| (Some(*channel), color)),
                    )
                    .filter(|(_, color)| color.automatic)
                {
                    ggplot_paint_scales.entry(channel).or_insert_with(|| {
                        resolve::AutomaticPaintScale {
                            id: encoding.id,
                            title: encoding.title.clone().unwrap_or_default(),
                            scale: encoding.scale.clone(),
                        }
                    });
                }
            }
            let ordinal_size =
                crate::scales::ggplot_numeric_ordinal(crate::scales::GgplotNumericPalette::Size)?;
            let mut ggplot_numeric_ids = BTreeMap::new();
            for existing in &this.definition.layers {
                if let Some(encoding) = existing
                    .numeric_scales
                    .get(&crate::grammar::NumericAesthetic::Size)
                    && (encoding.scale
                        == resolve::default_numeric_scale(
                            crate::scales::GgplotNumericPalette::Size,
                            &encoding.input,
                            this.root_data(existing.data)?,
                        )?
                        || encoding.scale == ordinal_size)
                    && let ColorInput::Numeric(Numeric::Field(field))
                    | ColorInput::Numeric(Numeric::Timestamp { field, .. })
                    | ColorInput::Category(field) = encoding.input
                {
                    ggplot_numeric_ids
                        .insert((crate::grammar::NumericAesthetic::Size, field), encoding.id);
                }
            }
            for existing in &this.definition.layers {
                for (target, kind) in [
                    (
                        crate::grammar::NumericAesthetic::Alpha,
                        crate::scales::GgplotNumericPalette::Alpha,
                    ),
                    (
                        crate::grammar::NumericAesthetic::StrokeWidth,
                        crate::scales::GgplotNumericPalette::Linewidth,
                    ),
                ] {
                    if let Some(encoding) = existing.numeric_scales.get(&target)
                        && (encoding.scale
                            == resolve::default_numeric_scale(
                                kind,
                                &encoding.input,
                                this.root_data(existing.data)?,
                            )?
                            || encoding.scale == crate::scales::ggplot_numeric_ordinal(kind)?)
                        && let ColorInput::Category(field)
                        | ColorInput::Numeric(Numeric::Field(field))
                        | ColorInput::Numeric(Numeric::Timestamp { field, .. }) = encoding.input
                    {
                        ggplot_numeric_ids.insert((target, field), encoding.id);
                    }
                }
            }
            let mut ggplot_style_ids = BTreeMap::new();
            for existing in &this.definition.layers {
                for target in [
                    crate::grammar::ValueAesthetic::Shape,
                    crate::grammar::ValueAesthetic::LineType,
                ] {
                    if let Some(encoding) = existing.value_scales.get(&target)
                        && encoding.scale == resolve::default_value_scale(target)?
                        && let ColorInput::Category(field) = encoding.input
                    {
                        ggplot_style_ids.insert((target, field), encoding.id);
                    }
                }
            }
            if let Some(theme) = &mut this.definition.theme {
                theme.layers.remove(&layer.id);
            }
            resolve::LayerContext {
                axes: &this.original.axes,
                color_ids: &mut this.original.colors,
                color_scales: &color_scales,
                ggplot_paint_scales: &mut ggplot_paint_scales,
                ggplot_numeric_ids: &mut ggplot_numeric_ids,
                ggplot_style_ids: &mut ggplot_style_ids,
            }
            .apply(&mut this.definition, &mut layer, &builder, &mapping, &data)?;
            if let Some(existing) = &existing {
                for (target, encoding) in &mut layer.numeric_scales {
                    if let Some(previous) = existing.numeric_scales.get(target)
                        && previous.input == encoding.input
                        && previous.scale == encoding.scale
                    {
                        // Reauthoring the same scale must retain explicit sharing.
                        encoding.id = previous.id;
                    }
                }
            }
            this.original.layers.insert(name, layer.id);
            if let Some(index) = this.definition.layers.iter().position(|l| l.id == layer.id) {
                this.definition.layers[index] = layer;
            } else {
                this.definition.layers.push(layer);
            }
            Ok(())
        })
    }
    /// Remove one named layer; dangling inset/transform references reject at build.
    pub fn remove_layer(self, name: &str) -> Self {
        self.update(|this| {
            let id = this.original.layers.remove(name).ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("No layer named '{name}'."),
                )
            })?;
            this.definition.layers.retain(|l| l.id != id);
            if let Some(theme) = &mut this.definition.theme {
                theme.layers.remove(&id);
            }
            Ok(())
        })
    }
    /// Configure the horizontal axis, preserving its authored name and identity.
    pub fn x_axis(self, axis: AxisBuilder) -> Self {
        self.axis(axis)
    }
    /// Configure the vertical axis, preserving its authored name and identity.
    pub fn y_axis(self, axis: AxisBuilder) -> Self {
        self.axis(axis)
    }
    /// Add or replace an independent guide while retaining its existing named identity.
    pub fn guide(self, mut guide: GuideBuilder) -> Self {
        self.update(|this| {
            super::validate_name(&guide.name)?;
            if this.original.axes.contains_key(&guide.name) {
                return Err(super::error(
                    crate::DiagnosticCode::SchemaConflict,
                    "An additional guide cannot replace a positional scale name.",
                ));
            }
            if let Some(id) = this.original.guides.get(&guide.name) {
                guide.spec.id = *id;
            }
            this.original
                .guides
                .insert(guide.name.clone(), guide.handle()?.id());
            let guide = guide.lower(&this.original.axes)?;
            if let Some(index) = this.definition.guides.iter().position(|g| g.id == guide.id) {
                this.definition.guides[index] = guide;
            } else {
                this.definition.guides.push(guide);
            }
            Ok(())
        })
    }
    /// Replace/add a named axis while preserving its existing identity.
    pub fn axis(self, mut axis: AxisBuilder) -> Self {
        self.update(|this| {
            if this.original.guides.contains_key(&axis.name) {
                return Err(super::error(
                    crate::DiagnosticCode::SchemaConflict,
                    "A positional scale cannot replace an additional guide name.",
                ));
            }
            validate_name(&axis.name)?;
            if let Some(id) = this.original.axes.get(&axis.name) {
                axis.spec.id = *id;
            }
            if this.definition.axes.is_empty() {
                this.definition.axes = vec![x_axis().spec, y_axis().spec];
            }
            this.original
                .axes
                .insert(axis.name.clone(), axis.handle()?.id());
            let axis = axis.lower(&this.original.axes)?;
            if let Some(index) = this.definition.axes.iter().position(|a| a.id == axis.id) {
                this.definition.axes[index] = axis;
            } else {
                this.definition.axes.push(axis);
            }
            Ok(())
        })
    }
    /// Replace wrap/grid policy using the retained default data and exact catalog rules.
    pub fn facet(self, facet: FacetBuilder) -> Self {
        self.update(|this| {
            this.definition.facets = Some(facet.lower(&this.original.data[0])?);
            Ok(())
        })
    }
    /// Remove facets; data/panel furniture must also be made compatible before build.
    pub fn clear_facets(mut self) -> Self {
        self.definition.facets = None;
        self
    }
    /// Replace one named color scale on every mapped layer, preserving shared scale identity.
    pub fn scale(self, scale: ColorScaleBuilder) -> Self {
        self.update(|this| {
            let id = *this.original.colors.get(&scale.name).ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    "Color edit names an absent mapped scale.",
                )
            })?;
            let value = scale.scale?;
            value.validate()?;
            for color in this
                .definition
                .layers
                .iter_mut()
                .flat_map(|l| l.color.iter_mut().chain(l.paint_scales.values_mut()))
                .filter(|c| c.id == id)
            {
                color.scale = value.clone();
            }
            Ok(())
        })
    }
    /// Override the title of an existing shared legend, independently of annotations.
    pub fn legend(self, legend: LegendBuilder) -> Self {
        self.update(|this| legend.apply(&mut this.definition, &this.original.colors))
    }
    /// Add or replace one stable annotation; use id() to target an existing annotation.
    pub fn annotation(self, label: impl Into<PlotLayer>) -> Self {
        self.update(|this| {
            let layer = label.into();
            if let PlotLayer::VectorPath(path) = layer {
                let figure = this.figure_mut();
                figure.version = 2;
                if let Some(index) = figure.paths.iter().position(|a| a.id == path.0.id) {
                    figure.paths[index] = path.0;
                } else {
                    figure.paths.push(path.0);
                }
                return Ok(());
            }
            let PlotLayer::Annotation(label) = layer else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Annotation edit requires labels, callout or a retained path.",
                ));
            };
            let label = label.lower(&this.original.axes)?;
            let figure = this.figure_mut();
            if let Some(index) = figure.annotations.iter().position(|a| a.id == label.id) {
                figure.annotations[index] = label;
            } else {
                figure.annotations.push(label);
            }
            Ok(())
        })
    }
    /// Remove a stable annotation without touching observed data.
    pub fn remove_annotation(mut self, id: &str) -> Self {
        self.figure_mut().annotations.retain(|a| a.id != id);
        self.figure_mut().paths.retain(|a| a.id != id);
        self
    }
    /// Add or replace a stable inset over existing prepared layer identities.
    pub fn inset(self, inset: InsetBuilder) -> Self {
        self.update(|this| {
            if let Some(e) = inset.failure {
                return Err(e);
            }
            let figure = this.figure_mut();
            if let Some(index) = figure.insets.iter().position(|i| i.id == inset.value.id) {
                figure.insets[index] = inset.value;
            } else {
                figure.insets.push(inset.value);
            }
            Ok(())
        })
    }
    /// Add/replace one complete shared transform, retaining a matching name's identity.
    pub fn transform(self, mut transform: TransformBuilder) -> Self {
        self.update(|this| {
            validate_name(&transform.name)?;
            let existing = this
                .original
                .transforms
                .get(&transform.name)
                .and_then(|id| this.definition.transforms.iter().find(|t| t.id == *id));
            if let Some(old) = existing {
                transform.id = Ok(old.id);
            }
            let input = if let Some(reference) = &transform.input {
                DataRef::Transform(reference.resolve(&this.original.transforms)?)
            } else if let Some(data) = &transform.data {
                DataRef::Dataset(data.id)
            } else {
                existing.map_or(DataRef::Dataset(this.original.data[0].id), |old| old.input)
            };
            let root = this.root_data(input)?;
            let node = transform.lower(root, input, &aes(), this.definition.profile())?;
            this.original.transforms.insert(transform.name, node.id);
            if let Some(index) = this
                .definition
                .transforms
                .iter()
                .position(|t| t.id == node.id)
            {
                this.definition.transforms[index] = node;
            } else {
                this.definition.transforms.push(node);
            }
            Ok(())
        })
    }
}
