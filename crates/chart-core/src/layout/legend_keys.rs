//! Layer-aware nonpositional keys, using outputs retained by scale preparation.
use super::LayoutRequest;
use crate::{ChartResult, ScaleId, grammar::*, interpolate::Value, scene::Color};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct KeyGlyph {
    pub shape: crate::shape::SymbolKind,
    pub size: f64,
    pub color: Color,
    pub fill: Option<Color>,
    pub stroke: Color,
    pub width: f64,
    pub line: Option<LineType>,
    pub rectangle: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct KeyLegend {
    pub channels: Vec<LegendAesthetic>,
    pub inputs: Vec<ColorInput>,
    pub source_keys: Option<Vec<crate::scales::ScaleKey>>,
    pub binned: bool,
    pub show_limits: bool,
    pub id: ScaleId,
    pub options: LegendOptions,
    pub title: String,
    pub labels: Vec<String>,
    pub values: Vec<crate::composition::ScaleValue>,
    pub glyphs: Vec<Vec<KeyGlyph>>,
}
fn title(chart: &PreparedChart, layer: &Layer, input: &ColorInput) -> String {
    let field = match input {
        ColorInput::Category(f)
        | ColorInput::GroupField(f)
        | ColorInput::Numeric(
            Numeric::Field(f) | Numeric::Category(f) | Numeric::Timestamp { field: f, .. },
        ) => Some(*f),
        _ => None,
    };
    if let (Some(field), DataRef::Dataset(id)) = (field, layer.data)
        && let Ok(source) = chart.source().get()
        && let Ok(data) = source.dataset(id)
        && let Some((_, field)) = data.schema().field(field)
    {
        return field.name.clone();
    }
    match input {
        ColorInput::Group => "Group".into(),
        ColorInput::Statistical(f) => format!("{f:?}"),
        _ => "Value".into(),
    }
}
fn base(chart: &PreparedChart, layer: &Layer, r: &LayoutRequest) -> KeyGlyph {
    let style = &layer.style;
    let theme = chart
        .definition()
        .theme
        .as_ref()
        .and_then(|t| t.geometry.clone())
        .unwrap_or_default();
    let default_width = layer.grammar.as_ref().is_some_and(|g| {
        g.default_line_width.unwrap_or(g.default_size)
            || (layer.geom == Geom::Point
                && g.default_radius != Some(false)
                && g.default_line_width.is_none())
    });
    let width = if default_width {
        theme.line_width
    } else {
        style.stroke_width
    };
    let units = style
        .units
        .unwrap_or(AestheticUnits::Millimeters)
        .factor(r.units);
    let color = r
        .host_theme
        .mark
        .map(crate::color::Paint::resolve)
        .unwrap_or_else(|| {
            if layer.grammar.as_ref().is_some_and(|g| g.default_color) {
                theme.ink.resolve()
            } else {
                style.color.resolve()
            }
        });
    KeyGlyph {
        shape: match layer.aesthetic_values.get(&ValueAesthetic::Shape) {
            Some(Value::Number(n)) => crate::shape::SymbolKind::Ggplot(n.0 as u8),
            _ => match layer.geom {
                Geom::ShapeSymbol { kind, .. } => kind,
                _ => crate::shape::SymbolKind::Ggplot(19),
            },
        },
        size: if layer
            .grammar
            .as_ref()
            .is_some_and(|g| g.default_radius.unwrap_or(g.default_size))
        {
            theme.point_size
        } else {
            style.radius
        },
        color,
        fill: style.fill.map(crate::color::Paint::resolve),
        stroke: style
            .stroke
            .map(crate::color::Paint::resolve)
            .unwrap_or(color),
        width: if layer.geom.reference_linewidth() && style.units.is_none() {
            2. * crate::grammar::reference_linewidth(width, r.units)
        } else {
            width * units
        },
        line: layer
            .geom
            .reference_linewidth()
            .then_some(style.line_type.unwrap_or(LineType::Solid)),
        rectangle: matches!(
            layer.geom,
            Geom::Rectangle | Geom::Bar { .. } | Geom::Area { .. }
        ),
    }
}
fn add(result: &mut Vec<KeyLegend>, next: KeyLegend) {
    if let Some(old) = result.iter_mut().find(|l| {
        l.binned == next.binned
            && l.title == next.title
            && l.labels == next.labels
            && l.options == next.options
    }) {
        for (a, b) in old.glyphs.iter_mut().zip(next.glyphs) {
            for g in b {
                if !a.contains(&g) {
                    a.push(g);
                }
            }
        }
    } else {
        result.push(next);
    }
}
pub(super) fn collect(chart: &PreparedChart, r: &LayoutRequest) -> ChartResult<Vec<KeyLegend>> {
    let mut result = vec![];
    for prepared in chart.layers() {
        let Some(layer) = chart
            .definition()
            .layers
            .iter()
            .find(|l| l.id == prepared.id())
        else {
            continue;
        };
        if layer.geom == Geom::Blank {
            continue;
        }
        let include = |a| layer.legend.as_ref().is_none_or(|l| l.includes(a));
        let mut local: Vec<KeyLegend> = vec![];
        for (paint_channel, legend) in prepared
            .color_legend()
            .map(|l| (None, l))
            .into_iter()
            .chain(prepared.paint_legends().iter().map(|(c, l)| (Some(*c), l)))
        {
            if !include(match paint_channel {
                None => LegendAesthetic::Color,
                Some(PaintAesthetic::Fill) => LegendAesthetic::Fill,
                Some(PaintAesthetic::Stroke) => LegendAesthetic::Stroke,
            }) {
                continue;
            }
            if !legend.colorbar.is_empty()
                || !legend.colorsteps.is_empty()
                || legend.entries.is_empty()
            {
                continue;
            }
            let input = layer
                .color
                .iter()
                .chain(layer.paint_scales.values())
                .find(|e| e.id == legend.id)
                .map(|e| e.input.clone());
            let mut key = KeyLegend {
                channels: vec![match paint_channel {
                    None => LegendAesthetic::Color,
                    Some(PaintAesthetic::Fill) => LegendAesthetic::Fill,
                    Some(PaintAesthetic::Stroke) => LegendAesthetic::Stroke,
                }],
                inputs: input.into_iter().collect(),
                source_keys: (!legend.discrete_keys.is_empty())
                    .then(|| legend.discrete_keys.clone()),
                binned: legend.mapping.as_ref().is_some_and(|m| {
                    matches!(
                        m.guide.as_deref(),
                        Some(
                            crate::scales::GgplotScaleGuide::BinnedBins(_)
                                | crate::scales::GgplotScaleGuide::ContinuousBins(_)
                                | crate::scales::GgplotScaleGuide::TemporalBins(_)
                        )
                    )
                }),
                show_limits: legend
                    .mapping
                    .as_ref()
                    .and_then(|m| m.colorbar_options.as_ref())
                    .is_some_and(|o| o.show_limits),
                id: legend.id,
                options: chart
                    .definition()
                    .legends
                    .get(&legend.id)
                    .cloned()
                    .unwrap_or_default(),
                title: legend.title.clone().unwrap_or_else(|| "Color".into()),
                labels: vec![],
                values: vec![],
                glyphs: vec![],
            };
            for (index, (label, color)) in legend.entries.iter().enumerate() {
                let mut glyph = base(chart, layer, r);
                match paint_channel {
                    None => {
                        glyph.color = *color;
                        glyph.stroke = *color;
                    }
                    Some(PaintAesthetic::Fill) => glyph.fill = Some(*color),
                    Some(PaintAesthetic::Stroke) => glyph.stroke = *color,
                }
                let units = layer
                    .style
                    .units
                    .unwrap_or(AestheticUnits::Millimeters)
                    .factor(r.units);
                glyph.size =
                    crate::grammar::reference_point_radius(glyph.size * units, glyph.width);
                glyph.width *= 0.5;
                key.values.push(legend.discrete_keys.get(index).map_or_else(
                    || crate::composition::ScaleValue::Category(label.clone()),
                    key_identity,
                ));
                key.labels.push(label.clone());
                if key.binned
                    && let Some(entry) = legend.numeric_breaks.get(index)
                {
                    key.values[index] = crate::composition::ScaleValue::Number(entry.value.0);
                }
                key.glyphs.push(
                    if key.binned
                        && legend
                            .numeric_breaks
                            .get(index)
                            .is_some_and(|e| matches!(e.mapped, Some(Value::Missing | Value::Null)))
                    {
                        vec![]
                    } else {
                        vec![glyph]
                    },
                );
            }
            if let Some(old) = local.iter_mut().find(|l| {
                l.title == key.title && l.labels == key.labels && l.options == key.options
            }) {
                old.channels.extend(key.channels);
                old.inputs.extend(key.inputs);
                for (a, b) in old.glyphs.iter_mut().zip(key.glyphs) {
                    if a.is_empty() || b.is_empty() {
                        continue;
                    }
                    match paint_channel {
                        None => {
                            a[0].color = b[0].color;
                            a[0].stroke = b[0].stroke
                        }
                        Some(PaintAesthetic::Fill) => a[0].fill = b[0].fill,
                        Some(PaintAesthetic::Stroke) => a[0].stroke = b[0].stroke,
                    }
                }
            } else {
                local.push(key);
            }
        }
        for (channel, encoding) in prepared
            .numeric_scales()
            .iter()
            .filter(|(c, _)| {
                matches!(
                    c,
                    NumericAesthetic::Size
                        | NumericAesthetic::AreaSize
                        | NumericAesthetic::Alpha
                        | NumericAesthetic::Opacity
                        | NumericAesthetic::StrokeWidth
                )
            })
            .map(|(c, e)| (Some(*c), e))
            .chain(
                prepared
                    .value_scales()
                    .iter()
                    .filter(|(c, _)| matches!(c, ValueAesthetic::Shape | ValueAesthetic::LineType))
                    .map(|(_, e)| (None, e)),
            )
        {
            let a = match channel {
                Some(NumericAesthetic::Size) => LegendAesthetic::Size,
                Some(NumericAesthetic::AreaSize) => LegendAesthetic::AreaSize,
                Some(NumericAesthetic::Alpha) => LegendAesthetic::Alpha,
                Some(NumericAesthetic::Opacity) => LegendAesthetic::Opacity,
                Some(NumericAesthetic::StrokeWidth) => LegendAesthetic::StrokeWidth,
                _ => {
                    if prepared
                        .value_scales()
                        .get(&ValueAesthetic::Shape)
                        .is_some_and(|e| e.id == encoding.id)
                    {
                        LegendAesthetic::Shape
                    } else {
                        LegendAesthetic::LineType
                    }
                }
            };
            if !include(a) {
                continue;
            }
            let Some(samples) = prepared.value_guide_samples().get(&encoding.id) else {
                continue;
            };
            let labels: Vec<_> =
                if let Some(entries) = prepared.discrete_value_guides().get(&encoding.id) {
                    entries.iter().map(|e| e.label.clone()).collect()
                } else if let Some(entries) = prepared.numeric_value_guides().get(&encoding.id) {
                    entries.iter().map(|e| e.label.clone()).collect()
                } else {
                    continue;
                };
            let binned = (matches!(
                encoding.scale.ggplot.as_deref(),
                Some(crate::scales::GgplotScalePolicy::Binned(_))
            ) && matches!(
                encoding.scale.guide.as_deref(),
                None | Some(crate::scales::GgplotScaleGuide::Binned(_))
            )) || matches!(
                encoding.scale.guide.as_deref(),
                Some(
                    crate::scales::GgplotScaleGuide::BinnedBins(_)
                        | crate::scales::GgplotScaleGuide::ContinuousBins(_)
                        | crate::scales::GgplotScaleGuide::TemporalBins(_)
                )
            );
            let mut legend = KeyLegend {
                channels: vec![a],
                inputs: vec![encoding.input.clone()],
                source_keys: prepared
                    .discrete_value_guides()
                    .contains_key(&encoding.id)
                    .then(Vec::new),
                binned,
                show_limits: encoding
                    .scale
                    .colorbar_options
                    .as_ref()
                    .is_some_and(|o| o.show_limits),
                id: encoding.id,
                options: chart
                    .definition()
                    .legends
                    .get(&encoding.id)
                    .cloned()
                    .unwrap_or_default(),
                title: title(chart, layer, &encoding.input),
                labels: vec![],
                values: vec![],
                glyphs: vec![],
            };
            let identities: Vec<_> =
                if let Some(entries) = prepared.numeric_value_guides().get(&encoding.id) {
                    entries
                        .iter()
                        .map(|e| crate::composition::ScaleValue::Number(e.value.0))
                        .collect()
                } else {
                    prepared.discrete_value_guides()[&encoding.id]
                        .iter()
                        .map(|e| key_identity(&e.key))
                        .collect()
                };
            let value_channel = prepared
                .value_scales()
                .iter()
                .find(|(_, e)| e.id == encoding.id)
                .map(|(c, _)| *c);
            for (entry_index, ((label, value), identity)) in
                labels.into_iter().zip(samples).zip(identities).enumerate()
            {
                if let Some(channel) = value_channel {
                    channel.validate(value)?;
                }
                let mut glyph = base(chart, layer, r);
                let units = layer
                    .style
                    .units
                    .unwrap_or(AestheticUnits::Millimeters)
                    .factor(r.units);
                match (channel, value_channel, value) {
                    (_, _, Value::Missing | Value::Null) => {
                        if binned {
                            legend.values.push(identity);
                            legend.labels.push(label.unwrap_or_default());
                            legend.glyphs.push(vec![]);
                        }
                        continue;
                    }
                    (Some(NumericAesthetic::Size), _, Value::Number(n)) => glyph.size = n.0,
                    (Some(NumericAesthetic::AreaSize), _, Value::Number(n)) => {
                        glyph.size = (n.0 / std::f64::consts::PI).sqrt()
                    }
                    (Some(NumericAesthetic::StrokeWidth), _, Value::Number(n)) => {
                        glyph.width =
                            if layer.geom.reference_linewidth() && layer.style.units.is_none() {
                                2. * crate::grammar::reference_linewidth(n.0, r.units)
                            } else {
                                n.0 * units
                            }
                    }
                    (
                        Some(NumericAesthetic::Alpha | NumericAesthetic::Opacity),
                        _,
                        Value::Number(n),
                    ) => {
                        glyph.color.alpha = crate::color::d65::alpha_byte(n.0);
                        glyph.stroke.alpha = glyph.color.alpha;
                        if let Some(fill) = &mut glyph.fill {
                            fill.alpha = glyph.color.alpha;
                        }
                    }
                    (None, Some(ValueAesthetic::Shape), Value::Number(n)) => {
                        glyph.shape = crate::shape::SymbolKind::Ggplot(n.0 as u8)
                    }
                    (None, Some(ValueAesthetic::LineType), Value::Text(text)) => {
                        glyph.line = Some(LineType::parse(text)?)
                    }
                    (None, Some(ValueAesthetic::LineType), Value::Number(n)) => {
                        glyph.line = Some(
                            [
                                LineType::Blank,
                                LineType::Solid,
                                LineType::Dashed,
                                LineType::Dotted,
                                LineType::DotDash,
                                LineType::LongDash,
                                LineType::TwoDash,
                            ][n.0 as usize],
                        )
                    }
                    _ => continue,
                }
                if !glyph.size.is_finite() || !glyph.width.is_finite() {
                    continue;
                }
                glyph.size =
                    crate::grammar::reference_point_radius(glyph.size * units, glyph.width);
                glyph.width *= 0.5;
                if let Some(keys) = &mut legend.source_keys {
                    keys.push(
                        prepared.discrete_value_guides()[&encoding.id][entry_index]
                            .key
                            .clone(),
                    );
                }
                legend.values.push(identity);
                legend.labels.push(label.unwrap_or_default());
                legend.glyphs.push(vec![glyph]);
            }
            if !legend.labels.is_empty() {
                if let Some(old) = local.iter_mut().find(|l| {
                    l.title == legend.title
                        && l.labels == legend.labels
                        && l.options == legend.options
                }) {
                    old.channels.extend(legend.channels);
                    old.inputs.extend(legend.inputs);
                    for (a, b) in old.glyphs.iter_mut().zip(legend.glyphs) {
                        if a.is_empty() || b.is_empty() {
                            continue;
                        }
                        match (channel, value_channel) {
                            (Some(NumericAesthetic::Size | NumericAesthetic::AreaSize), _) => {
                                a[0].size = b[0].size
                            }
                            (Some(NumericAesthetic::StrokeWidth), _) => a[0].width = b[0].width,
                            (Some(NumericAesthetic::Alpha | NumericAesthetic::Opacity), _) => {
                                a[0].color.alpha = b[0].color.alpha;
                                a[0].stroke.alpha = b[0].stroke.alpha;
                                if let Some(fill) = &mut a[0].fill {
                                    fill.alpha = b[0].color.alpha
                                }
                            }
                            (_, Some(ValueAesthetic::Shape)) => a[0].shape = b[0].shape,
                            (_, Some(ValueAesthetic::LineType)) => a[0].line = b[0].line,
                            _ => {}
                        }
                    }
                } else {
                    local.push(legend);
                }
            }
        }
        for mut legend in local {
            if let Some(title) = &legend.options.title {
                legend.title = title.clone();
            }
            if legend.options.reverse.unwrap_or(false) {
                if let Some(keys) = &mut legend.source_keys {
                    keys.reverse();
                }
                legend.values.reverse();
                legend.labels.reverse();
                if legend.binned && legend.glyphs.last().is_some_and(Vec::is_empty) {
                    legend.glyphs.pop();
                    legend.glyphs.reverse();
                    legend.glyphs.push(vec![]);
                } else {
                    legend.glyphs.reverse();
                }
            }
            let forced = layer.legend.as_ref().is_some_and(|policy| {
                if policy.aesthetics.is_empty() {
                    return policy.show == Some(true);
                }
                let selected = legend
                    .channels
                    .iter()
                    .filter_map(|a| policy.aesthetics.get(a))
                    .collect::<Vec<_>>();
                selected.is_empty() || selected.into_iter().any(|show| *show)
            });
            if !forced && let Some(keys) = &legend.source_keys {
                let mut observed = std::collections::BTreeSet::new();
                for input in &legend.inputs {
                    observed.extend(crate::grammar::guide_key_population(
                        input,
                        layer,
                        chart,
                        prepared.table(),
                    )?);
                }
                let missing = observed.iter().any(|value| !keys.contains(value));
                for (key, glyphs) in keys.iter().zip(&mut legend.glyphs) {
                    if !observed.contains(key)
                        && !(missing && matches!(key, crate::scales::ScaleKey::Null))
                    {
                        glyphs.clear();
                    }
                }
            }
            for glyphs in &mut legend.glyphs {
                if layer
                    .legend
                    .as_ref()
                    .is_some_and(|l| l.key_glyph == crate::grammar::KeyGlyph::None)
                {
                    glyphs.clear();
                    continue;
                }
                for glyph in glyphs {
                    if let Some(policy) = &layer.legend {
                        match policy.key_glyph {
                            crate::grammar::KeyGlyph::Point => {
                                glyph.line = None;
                                glyph.rectangle = false;
                            }
                            crate::grammar::KeyGlyph::Line => {
                                glyph.line = Some(LineType::Solid);
                                glyph.rectangle = false;
                            }
                            crate::grammar::KeyGlyph::Rectangle => {
                                glyph.line = None;
                                glyph.rectangle = true;
                            }
                            _ => {}
                        }
                    }
                    if let Some(alpha) = layer.style.alpha {
                        let alpha = crate::color::d65::alpha_byte(alpha);
                        glyph.color.alpha = alpha;
                        glyph.stroke.alpha = alpha;
                        if let Some(fill) = &mut glyph.fill {
                            fill.alpha = alpha;
                        }
                    }
                    let o = &legend.options.override_aes;
                    if let Some(v) = o.color {
                        glyph.color = v.resolve();
                        glyph.stroke = v.resolve();
                    }
                    if let Some(v) = o.stroke {
                        glyph.stroke = v.resolve();
                    }
                    if let Some(v) = o.fill {
                        glyph.fill = Some(v.resolve());
                    }
                    if let Some(v) = o.size {
                        glyph.size = crate::grammar::reference_point_radius(
                            v * AestheticUnits::Millimeters.factor(r.units),
                            glyph.width * 2.,
                        );
                    }
                    if let Some(v) = o.width {
                        glyph.width = v * AestheticUnits::Millimeters.factor(r.units);
                    }
                    if let Some(v) = o.alpha {
                        let a = crate::color::d65::alpha_byte(v);
                        glyph.color.alpha = a;
                        glyph.stroke.alpha = a;
                        if let Some(fill) = &mut glyph.fill {
                            fill.alpha = a;
                        }
                    }
                    if let Some(v) = o.shape {
                        glyph.shape = crate::shape::SymbolKind::Ggplot(v);
                    }
                    if let Some(v) = o.line_type {
                        glyph.line = Some(v);
                    }
                }
            }
            add(&mut result, legend);
        }
    }
    result.sort_by_key(|l| l.options.order);
    Ok(result)
}

pub(super) fn paint(
    g: &KeyGlyph,
    items: &mut Vec<crate::scene::SceneItem>,
    bounds: crate::Rect,
    y: f64,
    height: f64,
    key_width: f64,
    r: &LayoutRequest,
) -> ChartResult<()> {
    use crate::scene::{Primitive, SceneItem, Stroke};
    crate::limits::require_within(items.len() < r.limits.max_items, "legend key item")?;
    let stroke = (g.width > 0.).then_some(Stroke {
        color: g.stroke,
        width: g.width,
    });
    let primitive = if let Some(line) = g.line {
        if line == LineType::Blank {
            return Ok(());
        }
        let Some(stroke) = stroke else {
            return Ok(());
        };
        let mut builder = crate::path::Path::new();
        builder.move_to(bounds.origin().x(), y + height / 2.)?;
        builder.line_to(bounds.origin().x() + 2. * r.font_size, y + height / 2.)?;
        Primitive::VectorPath {
            geometry: builder.geometry(),
            fill: None,
            stroke: Some(stroke),
            dashes: line.pattern(stroke.width)?,
        }
    } else if g.rectangle {
        Primitive::Rectangle {
            bounds: crate::Rect::new(bounds.origin().x(), y, r.font_size, r.font_size)?,
            fill: g.fill.unwrap_or(g.color),
        }
    } else {
        if g.size <= 0. {
            return Ok(());
        }
        let mode = crate::shape::SymbolPaint::Auto.resolve(g.shape)?;
        let geometry = crate::shape::Symbol::new()
            .kind(g.shape)
            .size(std::f64::consts::PI * g.size * g.size)
            .generate()?
            .geometry();
        let transform = crate::path::Affine::new([
            1.,
            0.,
            0.,
            1.,
            bounds.origin().x() + key_width / 2.,
            y + height / 2.,
        ])?;
        Primitive::VectorPath {
            geometry: geometry.transformed(transform, 0.01, r.limits.max_path_commands)?,
            fill: if mode.color_fill() {
                Some(g.color)
            } else if mode.fills() {
                g.fill
            } else {
                None
            },
            stroke: if mode.strokes() { stroke } else { None },
            dashes: vec![],
        }
    };
    items.push(SceneItem {
        guide: None,
        layer: None,
        clip: Some(bounds),
        primitive,
    });
    Ok(())
}

fn key_identity(key: &crate::scales::ScaleKey) -> crate::composition::ScaleValue {
    match key {
        crate::scales::ScaleKey::Null => crate::composition::ScaleValue::MissingCategory,
        crate::scales::ScaleKey::Number(n) => crate::composition::ScaleValue::Number(n.0),
        crate::scales::ScaleKey::Text(s) => crate::composition::ScaleValue::Category(s.clone()),
        other => crate::composition::ScaleValue::Category(
            serde_json::to_string(other).expect("serializable key"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Rect, ResourceId, Revision,
        prelude::*,
        services::{ResourceDescriptor, ResourceKind, Units},
    };
    #[test]
    fn renamed_layer_keys_and_forced_inclusion_match_source() {
        use crate::{Rect, ResourceId, Revision, prelude::*, scales::*, services::*};
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/legend-awareness.json"
        ))
        .unwrap();
        for case in source["cases"].as_array().unwrap() {
            let a = Data::columns()
                .column("x", vec![1.])
                .column("g", vec!["A"])
                .build()
                .unwrap();
            let b = Data::columns()
                .name("second")
                .column("x", vec![2.])
                .column("g", vec!["B"])
                .build()
                .unwrap();
            let ColorScale::Mapped { scale, .. } = ggplot_color_default(false).unwrap() else {
                panic!("mapped")
            };
            let scale = scale
                .with_guide(GgplotScaleGuide::Discrete(GgplotDiscreteGuide {
                    labels: GgplotGuideLabels::Explicit(vec![
                        Some("Alpha".into()),
                        Some("Beta".into()),
                    ]),
                    ..Default::default()
                }))
                .unwrap();
            let p = plot(a)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.).color("g").color_scale("g"))
                .scale(color_mapped("g", scale))
                .layer(
                    points().legend(LayerLegend {
                        show: case["force"].as_bool(),
                        aesthetics: case["force"]
                            .as_object()
                            .map(|values| {
                                values
                                    .iter()
                                    .map(|(name, value)| {
                                        (
                                            if name == "colour" {
                                                LegendAesthetic::Color
                                            } else {
                                                LegendAesthetic::Shape
                                            },
                                            value.as_bool().unwrap(),
                                        )
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        ..Default::default()
                    }),
                )
                .layer(points().data(b).aesthetic_value(
                    ValueAesthetic::Shape,
                    Value::Number(crate::interpolate::Number(17.)),
                ))
                .build()
                .unwrap();
            let request = LayoutRequest::new(
                Rect::new(0., 0., 600., 360.).unwrap(),
                Units::Points,
                ResourceDescriptor {
                    id: ResourceId::new(1),
                    revision: Revision::INITIAL,
                    kind: ResourceKind::Font,
                    byte_len: 1,
                },
            );
            let keys = collect(&p.chart().unwrap().prepare().unwrap(), &request).unwrap();
            assert_eq!(keys.len(), 1);
            let key = &keys[0];
            assert_eq!(key.labels, vec!["Alpha", "Beta"]);
            for (i, glyphs) in key.glyphs.iter().enumerate() {
                let expected = case["draw"]
                    .as_object()
                    .unwrap()
                    .values()
                    .filter(|draw| draw[i] == true)
                    .count();
                assert_eq!(glyphs.len(), expected, "{case}");
            }
            assert_eq!(
                key.values,
                vec![
                    crate::composition::ScaleValue::Category("A".into()),
                    crate::composition::ScaleValue::Category("B".into())
                ]
            );
        }
    }
    #[test]
    fn interval_glyphs_and_boundary_labels_match_source() {
        use crate::{
            Rect, ResourceId, Revision, interpolate::Number, prelude::*, scales::*, services::*,
        };
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/binned-key-layout.json"
        ))
        .unwrap();
        for case in source["cases"].as_array().unwrap() {
            let data = Data::columns()
                .column("x", vec![1., 2., 3., 4.])
                .column("v", vec![1., 4., 9., 16.])
                .build()
                .unwrap();
            let scale = ggplot_numeric_default(GgplotNumericPalette::Size)
                .unwrap()
                .with_ggplot(GgplotScalePolicy::Binned(Box::new(GgplotBinnedPolicy {
                    limits: Some([Some(Number(0.)), Some(Number(20.))]),
                    breaks: GgplotBreaks::Explicit(vec![Number(5.), Number(10.)]),
                    ..Default::default()
                })))
                .unwrap()
                .with_guide(GgplotScaleGuide::BinnedBins(GgplotGuideLabels::Automatic))
                .unwrap()
                .with_colorbar_options(GgplotColorbarOptions {
                    show_limits: case["show_limits"] == true,
                    ..Default::default()
                });
            let p = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(aes().x("x").y(1.))
                .layer(points().numeric_scale(NumericAesthetic::Size, "v", scale))
                .legend(
                    legend()
                        .aesthetic(LegendAesthetic::Size)
                        .options(LegendOptions {
                            reverse: Some(case["reverse"] == true),
                            direction: Some(if case["direction"] == "horizontal" {
                                crate::scene::GradientDirection::Horizontal
                            } else {
                                crate::scene::GradientDirection::Vertical
                            }),
                            ..Default::default()
                        }),
                )
                .build()
                .unwrap();
            let r = LayoutRequest::new(
                Rect::new(0., 0., 600., 360.).unwrap(),
                Units::Points,
                ResourceDescriptor {
                    id: ResourceId::new(1),
                    revision: Revision::INITIAL,
                    kind: ResourceKind::Font,
                    byte_len: 1,
                },
            );
            let keys = collect(&p.chart().unwrap().prepare().unwrap(), &r).unwrap();
            assert_eq!(keys.len(), 1);
            let key = &keys[0];
            assert!(key.binned);
            assert_eq!(
                key.labels,
                case["labels"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>()
            );
            struct Metrics;
            impl TextMeasurer for Metrics {
                fn measure(&self, r: TextRequest<'_>) -> crate::ChartResult<TextMetrics> {
                    TextMetrics::new(r.text.len() as f64 * 6., 8., 2.)
                }
            }
            let frame =
                crate::layout::layout(p.chart().unwrap().prepare().unwrap(), &r, &Metrics).unwrap();
            let roles = frame
                .scene()
                .items()
                .iter()
                .filter_map(|i| i.guide.as_ref().map(|g| g.role))
                .collect::<Vec<_>>();
            assert_eq!(
                roles
                    .iter()
                    .filter(|role| **role == crate::scene::GuideRole::LegendKey)
                    .count(),
                3
            );
            assert_eq!(
                roles
                    .iter()
                    .filter(|role| **role == crate::scene::GuideRole::LegendLabel)
                    .count(),
                if case["show_limits"] == true { 4 } else { 2 }
            );
            assert_eq!(
                roles
                    .iter()
                    .filter(|role| **role == crate::scene::GuideRole::LegendTick)
                    .count(),
                if case["show_limits"] == true { 4 } else { 2 }
            );
            for (glyphs, size) in key.glyphs.iter().zip(case["sizes"].as_array().unwrap()) {
                if let Some(size) = size.as_f64() {
                    assert_eq!(glyphs.len(), 1);
                    let expected = (size * 0.37640625 + 0.5 * 0.25) * 72. / 25.4;
                    assert!((glyphs[0].size - expected).abs() < 3e-12);
                } else {
                    assert!(glyphs.is_empty());
                }
            }
        }
    }
    #[test]
    fn reference_key_values_and_merging() {
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/legend-keys.json"
        ))
        .unwrap();
        for case in source["cases"].as_array().unwrap() {
            let mode = case["mode"].as_str().unwrap();
            let data = Data::columns()
                .column("x", vec![1., 2., 3., 4.])
                .column("y", vec![1., 2., 1., 2.])
                .column("g", vec!["A", "B", "A", "B"])
                .column("v", vec![1., 4., 9., 16.])
                .build()
                .unwrap();
            let mut mapping = aes().x("x").y("y");
            if mode.contains("colour") {
                mapping = mapping.color("g");
            }
            if mode.contains("shape") {
                mapping = mapping.shape("g");
            }
            if mode.contains("size") {
                mapping = mapping.size("v");
            }
            if mode.contains("alpha") {
                mapping = mapping.alpha("v");
            }
            if mode == "linetype" {
                mapping = mapping.linetype("g");
            }
            let mut builder = plot(data)
                .profile(Profile::Ggplot2_4_0_3)
                .aes(mapping)
                .layer(if mode == "linetype" { line() } else { points() });
            if case["override"] == true {
                for channel in [
                    LegendAesthetic::Color,
                    LegendAesthetic::Shape,
                    LegendAesthetic::Size,
                    LegendAesthetic::Alpha,
                    LegendAesthetic::LineType,
                ] {
                    let enabled = match channel {
                        LegendAesthetic::Color => mode.contains("colour"),
                        LegendAesthetic::Shape => mode.contains("shape"),
                        LegendAesthetic::Size => mode.contains("size"),
                        LegendAesthetic::Alpha => mode.contains("alpha"),
                        _ => mode == "linetype",
                    };
                    if enabled {
                        builder =
                            builder.legend(legend().aesthetic(channel).options(LegendOptions {
                                override_aes: KeyOverrides {
                                    size: Some(5.),
                                    alpha: Some(0.5),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }));
                    }
                }
            }
            let p = builder.build().unwrap();
            let prepared = p.chart().unwrap().prepare().unwrap();
            let request = LayoutRequest::new(
                Rect::new(0., 0., 600., 360.).unwrap(),
                Units::Points,
                ResourceDescriptor {
                    id: ResourceId::new(1),
                    revision: Revision::INITIAL,
                    kind: ResourceKind::Font,
                    byte_len: 1,
                },
            );
            let keys = collect(&prepared, &request).unwrap();
            let guides = case["guides"].as_object().unwrap();
            assert_eq!(keys.len(), guides.len(), "{case}");
            for (key, guide) in keys.iter().zip(guides.values()) {
                assert_eq!(key.title, guide["title"].as_str().unwrap(), "{case}");
                let labels: Vec<_> = guide["key"][".label"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect();
                assert_eq!(key.labels, labels, "{case}");
                let samples = guide["layers"]
                    .as_object()
                    .unwrap()
                    .values()
                    .next()
                    .unwrap();
                for (index, glyphs) in key.glyphs.iter().enumerate() {
                    assert_eq!(glyphs.len(), 1, "{case}");
                    let glyph = &glyphs[0];
                    if let Some(size) = samples["size"][index]
                        .as_f64()
                        .filter(|_| mode != "linetype")
                    {
                        let stroke = samples["stroke"][index].as_f64().unwrap();
                        let radius = (size * 0.37640625 + stroke * 0.25) * 72. / 25.4;
                        assert!(
                            (glyph.size - radius).abs() < 3e-12,
                            "{case}: {} != {}",
                            glyph.size,
                            radius
                        );
                    }
                    if let Some(color) = samples["colour"][index].as_str() {
                        let mut expected = crate::color::parse_r(color).unwrap().resolve();
                        if let Some(alpha) = samples["alpha"][index].as_f64() {
                            expected.alpha = crate::color::d65::alpha_byte(alpha);
                        }
                        assert_eq!(glyph.color, expected, "{case}");
                    }
                    if let Some(line) = samples["linetype"][index].as_str() {
                        assert_eq!(glyph.line, Some(LineType::parse(line).unwrap()), "{case}");
                    }
                    if let Some(shape) = samples["shape"][index].as_u64() {
                        assert_eq!(
                            glyph.shape,
                            crate::shape::SymbolKind::Ggplot(shape as u8),
                            "{case}"
                        );
                    }
                }
            }
        }
    }
}
