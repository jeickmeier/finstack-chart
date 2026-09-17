//! Reference theme consumers: resolve before measurement and retain semantic content.
use super::*;
use crate::{
    ChartResult,
    grammar::PreparedChart,
    theme::{ResolvedElements, ThemeValue},
    typography::RichText,
};

pub(super) fn resolve(chart: &PreparedChart) -> ChartResult<Option<ResolvedElements>> {
    chart
        .definition()
        .theme
        .as_ref()
        .map(|t| t.resolved_elements())
        .transpose()
        .map(Option::flatten)
}
fn font_scale(elements: &ResolvedElements, units: crate::services::Units) -> f64 {
    elements.overrides.font_size.map_or(1., |v| {
        v / physical(elements.number("text", "size").unwrap_or(11.), units, 72.)
    })
}
fn physical(value: f64, units: crate::services::Units, per_inch: f64) -> f64 {
    value
        * match units {
            crate::services::Units::Points => 72.,
            crate::services::Units::LogicalPixels => 96.,
        }
        / per_inch
}
pub(super) fn text_style(
    elements: &ResolvedElements,
    name: &str,
    text: &RichText,
    r: &LayoutRequest,
) -> ChartResult<Option<RichText>> {
    if elements.blank(name) {
        return Ok(None);
    }
    let mut result = text.clone();
    let size = elements
        .number(name, "size")
        .map(|v| physical(v, r.units, 72.) * font_scale(elements, r.units) / r.font_size);
    let paint = elements
        .overrides
        .foreground
        .or(elements.paint(name, "colour")?);
    let family = match elements.value(name, "family") {
        Some(ThemeValue::Text(v)) => v.as_str(),
        _ => "",
    };
    let face = match elements.value(name, "face") {
        Some(ThemeValue::Text(v)) => v.as_str(),
        Some(ThemeValue::Number(v)) => match *v as i32 {
            2 => "bold",
            3 => "italic",
            4 => "bold.italic",
            _ => "plain",
        },
        _ => "plain",
    };
    let selected = elements
        .fonts
        .iter()
        .find(|f| f.family == family && f.face == face);
    if (!family.is_empty() || face != "plain") && selected.is_none() {
        return Err(crate::scales::error(
            crate::DiagnosticCode::MissingResource,
            "Theme family and face require an explicitly supplied TextFont resource.",
        ));
    }

    for run in result.lines.iter_mut().flatten() {
        if let Some(size) = size {
            run.size *= size;
        }
        if let Some(font) = selected {
            run.font = Some(font.font);
            run.weight = font.weight;
        }
        if let Some(paint) = paint {
            run.color = Some(paint);
        }
    }
    if let Some(value) = elements.number(name, "angle") {
        result.rotation = value;
    }
    if let Some(value) = elements.number(name, "lineheight") {
        result.line_spacing = value;
    }
    Ok(Some(result))
}
fn line(
    elements: &ResolvedElements,
    name: &str,
    local: &GuideLineStyle,
    units: crate::services::Units,
) -> ChartResult<GuideLineStyle> {
    let mut result = GuideLineStyle {
        arrow: match elements.value(name, "arrow") {
            Some(ThemeValue::Arrow(a)) => Some(a.clone()),
            _ => None,
        },
        arrow_fill: elements.paint(name, "arrow.fill")?,
        line_end: match elements.value(name, "lineend") {
            Some(ThemeValue::Text(v)) => Some(match v.as_str() {
                "round" => crate::grammar::LineEnd::Round,
                "square" => crate::grammar::LineEnd::Square,
                _ => crate::grammar::LineEnd::Butt,
            }),
            _ => None,
        },
        line_join: match elements.value(name, "linejoin") {
            Some(ThemeValue::Text(v)) => Some(match v.as_str() {
                "round" => crate::grammar::LineJoin::Round,
                "bevel" => crate::grammar::LineJoin::Bevel,
                _ => crate::grammar::LineJoin::Miter,
            }),
            _ => None,
        },
        visible: Some(!elements.blank(name)),
        color: if name.starts_with("panel.grid") {
            elements.overrides.grid.or(elements.paint(name, "colour")?)
        } else {
            elements
                .overrides
                .foreground
                .or(elements.paint(name, "colour")?)
        },
        width: elements.overrides.stroke_width.or(elements
            .number(name, "linewidth")
            .map(|v| crate::grammar::reference_linewidth(v, units))),
        dashes: None,
    };
    if result.width == Some(0.) {
        result.width = None;
        result.visible = Some(false);
    }
    if let Some(width) = result.width {
        result.dashes = Some(dashes(elements, name, width)?);
    }
    if matches!(
        elements.value(name, "linetype"),
        Some(ThemeValue::Number(0.))
    ) || matches!(elements.value(name,"linetype"),Some(ThemeValue::Text(v)) if v=="blank")
    {
        result.visible = Some(false);
    }
    // Explicit guide components remain the final override.
    result = result.overlay(Some(local));
    Ok(result)
}
pub(super) fn minor_line(
    r: &LayoutRequest,
    side: AxisSide,
    local: &GuideLineStyle,
    default_length: f64,
) -> ChartResult<(GuideLineStyle, f64)> {
    let Some(elements) = &r.resolved_theme else {
        return Ok((local.clone(), default_length));
    };
    let suffix = match side {
        AxisSide::Bottom => "x.bottom",
        AxisSide::Top => "x.top",
        AxisSide::Left => "y.left",
        AxisSide::Right => "y.right",
    };
    let style = line(
        elements,
        &format!("axis.minor.ticks.{suffix}"),
        &GuideLineStyle::default(),
        r.units,
    )?;
    let length = elements
        .destination_length(&format!("axis.minor.ticks.length.{suffix}"), "", 0)
        .unwrap_or(default_length);
    Ok((style, length))
}
fn guide(
    elements: &ResolvedElements,
    side: AxisSide,
    style: &mut GuideStyle,
    r: &LayoutRequest,
    radial: Option<bool>,
) -> ChartResult<()> {
    let cartesian_suffix = match side {
        AxisSide::Bottom => "x.bottom",
        AxisSide::Top => "x.top",
        AxisSide::Left => "y.left",
        AxisSide::Right => "y.right",
    };
    let suffix = radial.map_or(cartesian_suffix, |theta| if theta { "theta" } else { "r" });
    let mut components = style.components.clone().unwrap_or_default();
    components.domain = line(
        elements,
        &format!("axis.line.{suffix}"),
        &components.domain,
        r.units,
    )?;
    components.ticks = line(
        elements,
        &format!("axis.ticks.{suffix}"),
        &components.ticks,
        r.units,
    )?;
    let name = format!("axis.text.{suffix}");
    components.labels.visible = components.labels.visible.or(Some(!elements.blank(&name)));
    components.labels.color = components
        .labels
        .color
        .or(elements.overrides.foreground)
        .or(elements.paint(&name, "colour")?);
    components.labels.font_size = components.labels.font_size.or(elements
        .number(&name, "size")
        .map(|v| physical(v, r.units, 72.) * font_scale(elements, r.units)));
    components.labels.hjust = components.labels.hjust.or(elements.number(&name, "hjust"));
    components.labels.vjust = components.labels.vjust.or(elements.number(&name, "vjust"));
    components.labels.rotation = components
        .labels
        .rotation
        .or(elements.number(&name, "angle"));
    let ticks_hidden = components.ticks.visible == Some(false);
    style.components = Some(components);
    let geometry = style.geometry.get_or_insert_with(Default::default);
    geometry.outer.get_or_insert(0.);
    if ticks_hidden {
        geometry.inner.get_or_insert(0.);
    }
    geometry.inner = geometry.inner.or(elements.length(
        &format!("axis.ticks.length.{suffix}"),
        "",
        0,
        r.units,
        r.font_size,
        r.bounds.width(),
    )?);
    let margin_index = match side {
        AxisSide::Bottom => 0,
        AxisSide::Top => 2,
        AxisSide::Left => 1,
        AxisSide::Right => 3,
    };
    geometry.padding = geometry.padding.or(elements.length(
        &name,
        "margin",
        margin_index,
        r.units,
        r.font_size,
        r.bounds.width(),
    )?);
    if let Some(title) = &style.title {
        style.title = text_style(
            elements,
            &format!("axis.title.{cartesian_suffix}"),
            title,
            r,
        )?;
    }
    Ok(())
}
pub(super) fn configure(chart: &PreparedChart, r: &mut LayoutRequest) -> ChartResult<()> {
    let mut elements = resolve(chart)?;
    if let Some(elements) = &mut elements {
        elements.overrides.overlay(&r.output_theme);
        if let Some(size) = elements.number("text", "size") {
            r.font_size = physical(size, r.units, 72.);
        }
        if let Some(size) = elements.overrides.font_size {
            r.font_size = size;
        }
        elements.prepare_lengths(r.units, r.font_size, r.bounds.width(), r.bounds.height())?;
    }
    r.resolved_theme = elements.map(std::sync::Arc::new);
    let Some(elements) = r.resolved_theme.clone() else {
        return Ok(());
    };
    let base = r.clone();
    let theta = match &chart.definition().coordinate {
        Some(crate::grammar::CoordinateSpec::Radial(spec)) => {
            Some(spec.theta == crate::grammar::ThetaAxis::X)
        }
        _ => None,
    };
    for axis in &mut r.axes {
        guide(
            &elements,
            axis.side,
            &mut axis.guide,
            &base,
            theta.map(|x| x == axis.side.horizontal()),
        )?;
    }
    for item in &mut r.guides {
        guide(
            &elements,
            item.side,
            &mut item.style,
            &base,
            theta.map(|x| x == item.side.horizontal()),
        )?;
    }
    Ok(())
}
pub(super) fn rectangle(
    elements: &ResolvedElements,
    name: &str,
    bounds: crate::Rect,
    r: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Vec<crate::scene::Primitive>> {
    if elements.blank(name) {
        if name == "plot.background"
            && let Some(fill) = elements.overrides.background
        {
            return Ok(vec![crate::scene::Primitive::Rectangle {
                bounds,
                fill: fill.resolve(),
            }]);
        }
        return Ok(Vec::new());
    }
    let mut path = crate::path::Path::new();
    path.rect(
        bounds.origin().x(),
        bounds.origin().y(),
        bounds.width(),
        bounds.height(),
    )?;
    let width = elements
        .number(name, "linewidth")
        .map(|v| crate::grammar::reference_linewidth(v, r.units))
        .unwrap_or(0.);
    let stroke = if width > 0. {
        elements
            .paint(name, "colour")?
            .map(|p| crate::scene::Stroke {
                color: p.resolve(),
                width,
            })
    } else {
        None
    };
    let dashes = if width > 0. {
        dashes(elements, name, width)?
    } else {
        Vec::new()
    };
    let join = match elements.value(name, "linejoin") {
        Some(ThemeValue::Text(v)) => Some(match v.as_str() {
            "round" => crate::grammar::LineJoin::Round,
            "bevel" => crate::grammar::LineJoin::Bevel,
            _ => crate::grammar::LineJoin::Miter,
        }),
        _ => None,
    };
    if stroke.is_none() && elements.paint(name, "fill")?.is_none() {
        return Ok(Vec::new());
    }
    let primitive = crate::scene::Primitive::ShapePath {
        geometry: path.geometry(),
        fill_rule: crate::scene::FillRule::NonZero,
        fill: if name == "panel.border" {
            None
        } else if name == "plot.background" {
            elements
                .overrides
                .background
                .or(elements.paint(name, "fill")?)
                .map(crate::color::Paint::resolve)
        } else {
            elements
                .paint(name, "fill")?
                .map(crate::color::Paint::resolve)
        },
        stroke,
        dashes,
        anchors: Vec::new(),
    };
    super::stroke_outline::expand(primitive, 0, None, join, remaining)
}
pub(super) fn line_primitives(
    elements: &ResolvedElements,
    name: &str,
    commands: Vec<crate::scene::PathCommand>,
    r: &LayoutRequest,
    remaining: &mut usize,
) -> ChartResult<Vec<crate::scene::Primitive>> {
    let style = line(elements, name, &GuideLineStyle::default(), r.units)?;
    if style.visible == Some(false) {
        return Ok(Vec::new());
    }
    let stroke = crate::scene::Stroke {
        color: style
            .color
            .map(crate::color::Paint::resolve)
            .unwrap_or(crate::theme::rgb(0, 0, 0)),
        width: style.width.unwrap_or(1.),
    };
    let primitive = if let Some(dashes) = style.dashes.as_ref().filter(|v| !v.is_empty()) {
        crate::scene::Primitive::DashedPath {
            commands,
            stroke,
            dashes: dashes.clone(),
        }
    } else {
        crate::scene::Primitive::Path { commands, stroke }
    };
    let paint = crate::grammar::Style {
        color: stroke.color,
        stroke_width: stroke.width,
        units: Some(crate::grammar::AestheticUnits::Destination),
        ..Default::default()
    };
    let mut primitives =
        super::recipe_intervals::with_arrows(vec![primitive], style.arrow.as_ref(), &paint, r)?;
    if let Some(fill) = style.arrow_fill {
        for p in primitives.iter_mut().skip(1) {
            if let crate::scene::Primitive::ShapePath { fill: paint, .. } = p {
                *paint = Some(fill.resolve());
            }
        }
    }
    let mut result = Vec::new();
    for primitive in primitives {
        result.extend(super::stroke_outline::expand(
            primitive,
            0,
            style.line_end,
            style.line_join,
            remaining,
        )?);
    }
    Ok(result)
}
fn dashes(elements: &ResolvedElements, name: &str, width: f64) -> ChartResult<Vec<f64>> {
    use crate::grammar::LineType;
    let kind = match elements.value(name, "linetype") {
        Some(ThemeValue::Text(v)) => LineType::parse(v)?,
        Some(ThemeValue::Number(v)) => match *v as u8 {
            0 => LineType::Blank,
            2 => LineType::Dashed,
            3 => LineType::Dotted,
            4 => LineType::DotDash,
            5 => LineType::LongDash,
            6 => LineType::TwoDash,
            _ => LineType::Solid,
        },
        _ => LineType::Solid,
    };
    kind.pattern(width)
}
pub(super) fn margins(
    elements: &ResolvedElements,
    name: &str,
    block: &mut super::text::Block,
    r: &LayoutRequest,
) -> ChartResult<()> {
    let size = elements
        .number(name, "size")
        .map(|v| physical(v, r.units, 72.))
        .unwrap_or(r.font_size);
    let mut margins = [0.; 4];
    for (i, value) in margins.iter_mut().enumerate() {
        *value = elements
            .length(
                name,
                "margin",
                i,
                r.units,
                size,
                if i % 2 == 0 {
                    r.bounds.height()
                } else {
                    r.bounds.width()
                },
            )?
            .unwrap_or(0.);
    }
    block.bounds = crate::Rect::new(
        block.bounds.origin().x() - margins[3],
        block.bounds.origin().y() - margins[0],
        (block.bounds.width() + margins[1] + margins[3]).max(0.),
        (block.bounds.height() + margins[0] + margins[2]).max(0.),
    )?;
    Ok(())
}
pub(super) fn inset(r: &LayoutRequest, side: usize) -> ChartResult<f64> {
    let Some(elements) = &r.resolved_theme else {
        return Ok(r.padding);
    };
    let index = [3, 1, 0, 2][side];
    Ok(elements
        .length(
            "plot.margin",
            "",
            index,
            r.units,
            r.font_size,
            if side < 2 {
                r.bounds.width()
            } else {
                r.bounds.height()
            },
        )?
        .unwrap_or(r.padding))
}
pub(super) fn justification(elements: &ResolvedElements, name: &str) -> [f64; 2] {
    let coordinate = |value: &ThemeValue, vertical: bool| match value {
        ThemeValue::Number(v) => *v,
        ThemeValue::Text(v) => match v.as_str() {
            "left" if !vertical => 0.,
            "right" if !vertical => 1.,
            "bottom" if vertical => 0.,
            "top" if vertical => 1.,
            _ => 0.5,
        },
        _ => 0.5,
    };
    match elements.value(name, "") {
        Some(ThemeValue::Vector(v)) if v.len() == 2 => {
            [coordinate(&v[0], false), coordinate(&v[1], true)]
        }
        Some(v) => [coordinate(v, false), coordinate(v, true)],
        None => [0.5, 0.5],
    }
}
pub(super) fn legend_position(
    elements: &ResolvedElements,
) -> Option<crate::grammar::LegendPosition> {
    use crate::grammar::LegendPosition as P;
    match elements.value("legend.position", "") {
        Some(ThemeValue::Text(v)) => match v.as_str() {
            "left" => Some(P::Left),
            "right" => Some(P::Right),
            "top" => Some(P::Top),
            "bottom" => Some(P::Bottom),
            "inside" => {
                let [x, y] = justification(elements, "legend.position.inside");
                Some(P::Inside { x, y: 1. - y })
            }
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(units: crate::services::Units) -> LayoutRequest {
        LayoutRequest::new(
            crate::Rect::new(0., 0., 600., 400.).unwrap(),
            units,
            crate::services::ResourceDescriptor {
                id: crate::ResourceId::new(99),
                revision: crate::Revision::new(0),
                kind: crate::services::ResourceKind::Font,
                byte_len: 1,
            },
        )
    }
    #[test]
    fn source_default_axis_components_and_signed_physical_units() {
        use crate::theme::*;
        let theme = ElementTheme::preset(ThemePreset::Grey)
            .unwrap()
            .update(
                &ElementTheme::subtheme(
                    "axis_x",
                    std::collections::BTreeMap::from([(
                        "ticks.length".into(),
                        ThemeEntry::Value(ThemeValue::Unit(vec![ThemeLength {
                            value: Some(-2.),
                            unit: "mm".into(),
                        }])),
                    )]),
                )
                .unwrap(),
            )
            .unwrap();
        let elements = ResolvedElements::new(&theme).unwrap();
        for units in [
            crate::services::Units::Points,
            crate::services::Units::LogicalPixels,
        ] {
            let r = request(units);
            let mut style = GuideStyle::default();
            guide(&elements, AxisSide::Bottom, &mut style, &r, None).unwrap();
            let c = style.components.unwrap();
            assert_eq!(c.domain.visible, Some(false));
            assert_eq!(c.ticks.visible, Some(true));
            assert_eq!(c.ticks.line_end, Some(crate::grammar::LineEnd::Butt));
            assert_eq!(c.ticks.line_join, Some(crate::grammar::LineJoin::Round));
            assert_eq!(c.labels.hjust, Some(0.5));
            assert_eq!(c.labels.vjust, Some(1.));
            let dpi = if units == crate::services::Units::Points {
                72.
            } else {
                96.
            };
            assert!((c.labels.font_size.unwrap() - 8.8 * dpi / 72.).abs() < 1e-13);
            assert!((style.geometry.unwrap().inner.unwrap() + 2. * dpi / 25.4).abs() < 1e-13);
        }
    }
    #[test]
    fn source_element_line_arrow_and_stroke_contract() {
        use crate::theme::*;
        let oracle: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/theme-line-controls.json"
        ))
        .unwrap();
        let arrow = crate::grammar::ArrowSpec {
            angle: 25.,
            length_mm: 4.,
            ends: crate::grammar::ArrowEnds::Both,
            closed: true,
        };
        let element = ThemeElement::new(ElementKind::Line)
            .property("colour", ThemeValue::Text("#123456".into()))
            .unwrap()
            .property("linewidth", ThemeValue::Number(0.7))
            .unwrap()
            .property("linetype", ThemeValue::Text("dashed".into()))
            .unwrap()
            .property("lineend", ThemeValue::Text("square".into()))
            .unwrap()
            .property("linejoin", ThemeValue::Text("bevel".into()))
            .unwrap()
            .property("arrow", ThemeValue::Arrow(arrow.clone()))
            .unwrap()
            .property("arrow.fill", ThemeValue::Text("red".into()))
            .unwrap();
        let theme = ElementTheme::preset(ThemePreset::Grey)
            .unwrap()
            .update(
                &ElementTheme::default()
                    .element("axis.line", ThemeEntry::Element(element))
                    .unwrap(),
            )
            .unwrap();
        let e = ResolvedElements::new(&theme).unwrap();
        let style = line(
            &e,
            "axis.line.x.bottom",
            &GuideLineStyle::default(),
            crate::services::Units::Points,
        )
        .unwrap();
        assert!(
            (style.width.unwrap() - oracle["gp"]["lwd"].as_f64().unwrap() * 72. / 96.).abs()
                < 1e-13
        );
        assert_eq!(style.arrow, Some(arrow));
        assert_eq!(style.line_end, Some(crate::grammar::LineEnd::Square));
        assert_eq!(style.line_join, Some(crate::grammar::LineJoin::Bevel));
        assert_eq!(style.dashes.unwrap(), vec![style.width.unwrap() * 4.; 2]);
    }
    #[test]
    fn source_text_lineheight_margin_and_explicit_font_resource() {
        use crate::theme::*;
        let theme = ElementTheme::preset(ThemePreset::Grey).unwrap();
        let mut elements = ResolvedElements::new(&theme).unwrap();
        let r = request(crate::services::Units::Points);
        let mut source = RichText::plain("a");
        source
            .lines
            .push(vec![crate::typography::RichRun::new("b")]);
        let text = text_style(&elements, "plot.title", &source, &r)
            .unwrap()
            .unwrap();
        assert_eq!(text.line_spacing, 0.9);
        text.validate(r.limits).unwrap();
        let mut block = super::super::text::Block {
            bounds: crate::Rect::new(0., 0., 10., 20.).unwrap(),
            items: vec![],
            diagnostics: vec![],
        };
        margins(&elements, "plot.title", &mut block, &r).unwrap();
        assert!((block.bounds.height() - (20. + 5.5 * 72. / 72.27)).abs() < 1e-13);
        let themed = theme
            .update(
                &ElementTheme::default()
                    .element(
                        "text",
                        ThemeEntry::Element(
                            ThemeElement::new(ElementKind::Text)
                                .property("family", ThemeValue::Text("Supplied".into()))
                                .unwrap(),
                        ),
                    )
                    .unwrap(),
            )
            .unwrap();
        elements = ResolvedElements::new(&themed).unwrap();
        assert!(text_style(&elements, "axis.text", &RichText::plain("1"), &r).is_err());
        elements.fonts.push(crate::grammar::TextFont {
            family: "Supplied".into(),
            face: "plain".into(),
            font: r.font,
            weight: 400,
        });
        let text = text_style(&elements, "axis.text", &RichText::plain("1"), &r)
            .unwrap()
            .unwrap();
        assert_eq!(text.lines[0][0].font, Some(r.font));
    }
}
