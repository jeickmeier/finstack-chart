use super::*;

macro_rules! scalar {
    ($a:ident, $b:ident, $method:ident) => {
        $b.clone().$method($a.one()?)
    };
}
macro_rules! string {
    ($a:ident, $b:ident, $method:ident) => {
        $b.clone().$method($a.string()?)
    };
}
macro_rules! pair {
    ($a:ident, $b:ident, $method:ident) => {{
        let (x, y) = $a.pair()?;
        $b.clone().$method(x, y)
    }};
}
macro_rules! empty {
    ($a:ident, $b:ident, $method:ident) => {{
        $a.count(0)?;
        $b.clone().$method()
    }};
}
macro_rules! xy {
    ($a:ident, $b:ident, $method:ident) => {{
        $a.count(2)?;
        $b.clone()
            .$method(scale_value(&$a.0[0])?, scale_value(&$a.0[1])?)
    }};
}
/// Resolve host panel labels through the same exact group-value conversion used by facets.
pub fn panel_key(value: Value) -> ChartResult<crate::grammar::PanelKey> {
    if let Value::Array(values) = value {
        return Ok(crate::grammar::PanelKey {
            values: groups(values)?,
        });
    }
    serde_json::from_value(value).map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
}
fn exact_i64(value: &Value) -> ChartResult<i64> {
    if let Some(s) = value.as_str() {
        return s.parse().map_err(|_| {
            error(
                DiagnosticCode::Validation,
                "Expected an exact signed 64-bit integer.",
            )
        });
    }
    value
        .as_i64()
        .filter(|v| v.unsigned_abs() <= 9_007_199_254_740_991)
        .ok_or_else(|| {
            error(
                DiagnosticCode::PrecisionLoss,
                "Large integers require an exact integer adapter.",
            )
        })
}
impl Component {
    /// Apply a scalar option through its owning core builder, preserving this reusable component.
    pub fn set(&self, method: &str, arguments: &str) -> ChartResult<Self> {
        let a = Args::parse(arguments)?;
        Ok(Self(match &self.0 {
            Kind::Expression(expr) => Kind::Expression(expr.set(method, &a)?),
            Kind::ScaleAes(_) => return Err(unsupported(method)),
            Kind::Aes(b) => Kind::Aes(match method {
                "x" => b.clone().x(a.mapping()?),
                "y" => b.clone().y(a.mapping()?),
                "x2" => b.clone().x2(a.mapping()?),
                "y2" => b.clone().y2(a.mapping()?),
                "low" => b.clone().low(a.mapping()?),
                "high" => b.clone().high(a.mapping()?),
                "size" => b.clone().size(a.mapping()?),
                "group" => b.clone().group(a.mapping()?),
                "color" => b.clone().color(a.mapping()?),
                "fill" => b.clone().fill(a.mapping()?),
                "stroke" => b.clone().stroke(a.mapping()?),
                "shape" => b.clone().shape(a.mapping()?),
                "linetype" => b.clone().linetype(a.mapping()?),
                "alpha" => b.clone().alpha(a.mapping()?),
                "linewidth" => b.clone().linewidth(a.mapping()?),
                "fill_scale" => string!(a, b, fill_scale),
                "stroke_scale" => string!(a, b, stroke_scale),
                "group_all" => empty!(a, b, group_all),
                "color_scale" => string!(a, b, color_scale),
                _ => return Err(unsupported(method)),
            }),
            Kind::Layer(b) => Kind::Layer(match method {
                "hierarchy_value" => b.clone().hierarchy_value(a.mapping()?),
                "hierarchy_label" => b.clone().hierarchy_label(a.mapping()?),
                "hierarchy_layout" => scalar!(a, b, hierarchy_layout),
                "hierarchy_projection" => scalar!(a, b, hierarchy_projection),
                "hierarchy_order" => scalar!(a, b, hierarchy_order),
                "hierarchy_limits" => scalar!(a, b, hierarchy_limits),
                "hierarchy_aggregation" => {
                    use crate::grammar::HierarchyAggregation as A;
                    b.clone()
                        .hierarchy_aggregation(match a.one::<A<String>>()? {
                            A::Count => A::Count,
                            A::Sum(name) => A::Sum(name.into()),
                            A::Registered(op) => A::Registered(op),
                        })
                }
                "hierarchy_source" => {
                    use crate::grammar::HierarchySource as S;
                    b.clone().hierarchy_source(match a.one::<S<String>>()? {
                        S::Paths(name) => S::Paths(name.into()),
                        S::Table { id, parent } => S::Table {
                            id: id.map(Into::into),
                            parent: parent.map(Into::into),
                        },
                    })
                }

                "symbol_kind" => scalar!(a, b, symbol_kind),
                "symbol_size" => scalar!(a, b, symbol_size),
                "symbol_paint" => scalar!(a, b, symbol_paint),
                "symbol_missing" => scalar!(a, b, symbol_missing),
                "symbol_title" => string!(a, b, symbol_title),
                "symbol_types" => {
                    a.count(3)?;
                    b.clone()
                        .symbol_types(a.at::<String>(0)?, a.at(1)?, a.at(2)?)
                }
                "symbol_groups" => {
                    a.count(2)?;
                    b.clone().symbol_groups(a.at(0)?, a.at(1)?)
                }
                "symbol_size_guide" => {
                    a.count(2)?;
                    b.clone().symbol_size_guide(a.at::<String>(0)?, a.at(1)?)
                }
                "shape_protocol" => {
                    a.count(2)?;
                    b.clone().shape_protocol(a.at(0)?, a.at(1)?)
                }
                "curve" => scalar!(a, b, curve),
                "arc_parameters" => scalar!(a, b, arc_parameters),
                "radial_parameters" => scalar!(a, b, radial_parameters),
                "pie_angles" => scalar!(a, b, pie_angles),
                "pie_order" => scalar!(a, b, pie_order),
                "pie_grouped" => scalar!(a, b, pie_grouped),
                "shape_value" => {
                    a.count(2)?;
                    let input = numeric_input(&a.0[1])?;
                    b.clone().shape_value(a.at(0)?, input)
                }
                "numeric_scale" => {
                    a.count(3)?;
                    let input = numeric_input(&a.0[1])?;
                    b.clone().numeric_scale(a.at(0)?, input, a.at(2)?)
                }
                "value_scale" => {
                    a.count(3)?;
                    b.clone()
                        .value_scale(a.at(0)?, numeric_input(&a.0[1])?, a.at(2)?)
                }
                "aesthetic_value" => {
                    a.count(2)?;
                    b.clone().aesthetic_value(a.at(0)?, a.at(1)?)
                }
                "aesthetic_units" => scalar!(a, b, aesthetic_units),
                "line_type" => scalar!(a, b, line_type),
                "radius" => scalar!(a, b, radius),
                "linewidth" => scalar!(a, b, linewidth),
                "alpha" => scalar!(a, b, alpha),
                "fill" => b.clone().fill(a.color()?),
                "stroke" => b.clone().stroke(a.color()?),
                "orientation" => scalar!(a, b, orientation),
                "name" => string!(a, b, name),
                "from_transform" => string!(a, b, from_transform),
                "independent" => empty!(a, b, independent),
                "size" => scalar!(a, b, size),
                "color" => b.clone().color(a.color()?),
                "color_group" => string!(a, b, color_group),
                "axes" => {
                    let (x, y) = a.pair::<String, String>()?;
                    b.clone().axes(x, y)
                }
                "scope" => scalar!(a, b, scope),
                "facet_target" => scalar!(a, b, facet_target),
                "clip" => scalar!(a, b, clip),
                "invalid" => scalar!(a, b, invalid),
                "candle_colors" => b
                    .clone()
                    .candle_colors(a.one::<crate::grammar::CandleColors<crate::color::Paint>>()?),
                "width" => scalar!(a, b, width),
                "order" => scalar!(a, b, order),
                "connect_gaps" => scalar!(a, b, connect_gaps),
                "bins" => scalar!(a, b, bins),
                "breaks" => scalar!(a, b, breaks),
                "baseline" => scalar!(a, b, baseline),
                "geometry" => {
                    a.count(3)?;
                    b.clone().geometry(
                        a.at::<String>(0)?,
                        Revision::new(exact_u64(&a.0[1])?),
                        a.at(2)?,
                    )
                }
                _ => return Err(unsupported(method)),
            }),
            Kind::Stat(b) => Kind::Stat(match method {
                "field_parameter" => {
                    a.count(2)?;
                    b.clone()
                        .field_parameter(a.at::<String>(0)?, mapping(&a.0[1])?)
                }
                "x" => b.clone().x(a.mapping()?),
                "y" => b.clone().y(a.mapping()?),
                "group" => b.clone().group(a.mapping()?),
                "group_all" => empty!(a, b, group_all),
                "bins" => scalar!(a, b, bins),
                "breaks" => scalar!(a, b, breaks),
                "outliers" => scalar!(a, b, outliers),
                "required" => b.clone().required(
                    a.one::<Vec<Value>>()?
                        .iter()
                        .map(mapping)
                        .collect::<ChartResult<Vec<_>>>()?,
                ),
                "quantiles" => scalar!(a, b, quantiles),
                "empty_sum_zero" => scalar!(a, b, empty_sum_zero),
                "transform" => pair!(a, b, transform),
                "y_transform" => pair!(a, b, y_transform),
                _ => return Err(unsupported(method)),
            }),
            Kind::StatAes(b) => Kind::StatAes(match method {
                "color" => scalar!(a, b, color),
                "color_scale" => string!(a, b, color_scale),
                "color_group" => string!(a, b, color_group),
                "x" | "y" | "x2" | "y2" | "size" => {
                    a.count(1)?;
                    let v = stat_numeric(&a.0[0])?;
                    match method {
                        "x" => b.clone().x(v),
                        "y" => b.clone().y(v),
                        "x2" => b.clone().x2(v),
                        "y2" => b.clone().y2(v),
                        _ => b.clone().size(v),
                    }
                }
                _ => return Err(unsupported(method)),
            }),
            Kind::BinAes(b) => Kind::BinAes(match method {
                "color_group" => string!(a, b, color_group),
                "x" | "y" | "x2" | "y2" | "size" => {
                    a.count(1)?;
                    let v = bin_numeric(&a.0[0])?;
                    match method {
                        "x" => b.clone().x(v),
                        "y" => b.clone().y(v),
                        "x2" => b.clone().x2(v),
                        "y2" => b.clone().y2(v),
                        _ => b.clone().size(v),
                    }
                }
                _ => return Err(unsupported(method)),
            }),
            Kind::Position(b) => Kind::Position(match method {
                "normalize" => scalar!(a, b, normalize),
                "stack_order" => scalar!(a, b, stack_order),
                "stack_offset" => scalar!(a, b, stack_offset),
                "stack_missing" => scalar!(a, b, stack_missing),
                "width" => scalar!(a, b, width),
                "displacement" => pair!(a, b, displacement),
                "units" => scalar!(a, b, units),
                _ => return Err(unsupported(method)),
            }),
            Kind::Filter(b) => Kind::Filter(match method {
                "minimum" => scalar!(a, b, minimum),
                "maximum" => scalar!(a, b, maximum),
                _ => return Err(unsupported(method)),
            }),
            Kind::Transform(b) => Kind::Transform(match method {
                "from_transform" => string!(a, b, from_transform),
                "scope" => scalar!(a, b, scope),
                "facet_target" => scalar!(a, b, facet_target),
                "invalid" => scalar!(a, b, invalid),
                _ => return Err(unsupported(method)),
            }),
            Kind::Scale(b) => Kind::Scale(match method {
                "domain" => pair!(a, b, domain),
                "baseline" => scalar!(a, b, baseline),
                "padding" => scalar!(a, b, padding),
                "nice" => scalar!(a, b, nice),
                "nice_ticks" => scalar!(a, b, nice_ticks),
                "categories" => b.clone().categories(a.one::<Vec<String>>()?),
                "band_padding" => pair!(a, b, band_padding),
                "point_padding" => scalar!(a, b, point_padding),
                "time_domain" => {
                    a.count(2)?;
                    b.clone()
                        .time_domain(exact_i64(&a.0[0])?, exact_i64(&a.0[1])?)
                }
                "interval" => scalar!(a, b, interval),
                "calendar_interval" => scalar!(a, b, calendar_interval),
                _ => return Err(unsupported(method)),
            }),
            Kind::Axis(b) => Kind::Axis(match method {
                "expansion" => scalar!(a, b, expansion),
                "discrete_policy" => scalar!(a, b, discrete_policy),
                "continuous_limits" => {
                    let values = a
                        .one::<Option<Vec<Value>>>()?
                        .map(|values| {
                            values
                                .into_iter()
                                .map(|value| match value {
                                    Value::Bool(value) => {
                                        Ok(crate::interpolate::Number(if value { 1. } else { 0. }))
                                    }
                                    value => serde_json::from_value(value).map_err(|e| {
                                        error(DiagnosticCode::Validation, e.to_string())
                                    }),
                                })
                                .collect::<ChartResult<Vec<_>>>()
                        })
                        .transpose()?;
                    b.clone().continuous_limits(values)
                }
                "guide_profile" => scalar!(a, b, guide_profile),
                "guide_components" => scalar!(a, b, guide_components),
                "guide_geometry" => scalar!(a, b, guide_geometry),
                "tick_size" => scalar!(a, b, tick_size),
                "tick_size_inner" => scalar!(a, b, tick_size_inner),
                "tick_size_outer" => scalar!(a, b, tick_size_outer),
                "tick_padding" => scalar!(a, b, tick_padding),
                "tick_offset" => scalar!(a, b, tick_offset),
                "tick_arguments" => scalar!(a, b, tick_arguments),
                "minor_breaks" => scalar!(a, b, minor_breaks),
                "tick_format" => b.clone().tick_format(guide_formatter(a.one()?)?),
                "tick_values" => b.clone().tick_values(
                    a.one::<Option<Vec<Value>>>()?
                        .map(|values| {
                            values
                                .iter()
                                .map(scale_value)
                                .collect::<ChartResult<Vec<_>>>()
                        })
                        .transpose()?,
                ),

                "numeric_format" => scalar!(a, b, numeric_format),
                "time_format" => scalar!(a, b, time_format),
                "oob" => scalar!(a, b, oob),
                "name" => string!(a, b, name),
                "side" => scalar!(a, b, side),
                "label" => string!(a, b, label),
                "rotation" => scalar!(a, b, rotation),
                "visible" => scalar!(a, b, visible),
                "viewport" => pair!(a, b, viewport),
                "range" => pair!(a, b, range),
                "outside" => scalar!(a, b, outside),
                "ticks" => b.clone().ticks(
                    a.one::<Vec<(Value, String)>>()?
                        .into_iter()
                        .map(|(v, s)| scale_value(&v).map(|v| (v, s)))
                        .collect::<ChartResult<Vec<_>>>()?,
                ),
                "secondary" => {
                    a.count(3)?;
                    b.clone().secondary(a.at::<String>(0)?, a.at(1)?, a.at(2)?)
                }
                "secondary_transform" => {
                    a.count(2)?;
                    b.clone().secondary_transform(a.at::<String>(0)?, a.at(1)?)
                }
                _ => return Err(unsupported(method)),
            }),
            Kind::Guide(b) => Kind::Guide(match method {
                "guide_profile" => scalar!(a, b, guide_profile),
                "guide_components" => scalar!(a, b, guide_components),
                "guide_geometry" => scalar!(a, b, guide_geometry),
                "tick_size" => scalar!(a, b, tick_size),
                "tick_size_inner" => scalar!(a, b, tick_size_inner),
                "tick_size_outer" => scalar!(a, b, tick_size_outer),
                "tick_padding" => scalar!(a, b, tick_padding),
                "tick_offset" => scalar!(a, b, tick_offset),
                "tick_arguments" => scalar!(a, b, tick_arguments),
                "minor_breaks" => scalar!(a, b, minor_breaks),
                "tick_format" => b.clone().tick_format(guide_formatter(a.one()?)?),
                "tick_values" => b.clone().tick_values(
                    a.one::<Option<Vec<Value>>>()?
                        .map(|values| {
                            values
                                .iter()
                                .map(scale_value)
                                .collect::<ChartResult<Vec<_>>>()
                        })
                        .transpose()?,
                ),

                "scale" => string!(a, b, scale),
                "side" => scalar!(a, b, side),
                "translate" => pair!(a, b, translate),
                "label" => string!(a, b, label),
                "rotation" => scalar!(a, b, rotation),
                "visible" => scalar!(a, b, visible),
                "numeric_format" => scalar!(a, b, numeric_format),
                "time_format" => scalar!(a, b, time_format),
                "ticks" => b.clone().ticks(
                    a.one::<Vec<(Value, String)>>()?
                        .into_iter()
                        .map(|(v, s)| scale_value(&v).map(|v| (v, s)))
                        .collect::<ChartResult<Vec<_>>>()?,
                ),
                _ => return Err(unsupported(method)),
            }),
            Kind::Color(b) => Kind::Color(match method {
                "palette_scheme" => b
                    .clone()
                    .palette_scheme(a.one::<crate::scales::chromatic::SchemeSpec>()?),
                "palette" => b.clone().palette(
                    a.one::<Vec<Value>>()?
                        .iter()
                        .map(color)
                        .collect::<ChartResult<_>>()?,
                ),
                "domain" => b.clone().domain(a.one::<Vec<String>>()?),
                "missing" => b.clone().missing(a.color()?),
                "clamp" => scalar!(a, b, clamp),
                _ => return Err(unsupported(method)),
            }),
            Kind::Legend(b) => Kind::Legend(match method {
                "untitled" => empty!(a, b, untitled),
                "generic_title" => empty!(a, b, generic_title),
                "scale" => string!(a, b, scale),
                "title" => string!(a, b, title),
                _ => return Err(unsupported(method)),
            }),
            Kind::Facet(b) => Kind::Facet(match method {
                "columns" => scalar!(a, b, columns),
                "empty" => scalar!(a, b, empty),
                "free_x" => scalar!(a, b, free_x),
                "free_y" => scalar!(a, b, free_y),
                "gap" => scalar!(a, b, gap),
                "collect_guides" => scalar!(a, b, collect_guides),
                "order" => b.clone().order(
                    a.one::<Vec<Value>>()?
                        .into_iter()
                        .map(panel_key)
                        .collect::<ChartResult<_>>()?,
                ),
                _ => return Err(unsupported(method)),
            }),
            Kind::Style(b) => Kind::Style(match method {
                "color_mode" => scalar!(a, b, color_mode),
                "gradient" => b
                    .clone()
                    .gradient(a.one::<crate::scene::LinearGradient<crate::color::Paint>>()?),
                "background" => b.clone().background(a.color()?),
                "panel" => b.clone().panel(a.color()?),
                "foreground" => b.clone().foreground(a.color()?),
                "grid" => b.clone().grid(a.color()?),
                "mark" => b.clone().mark(a.color()?),
                "annotation" => b.clone().annotation(a.color()?),
                "focus" => b.clone().focus(a.color()?),
                "selection" => b.clone().selection(a.color()?),
                "font_size" => scalar!(a, b, font_size),
                "padding" => scalar!(a, b, padding),
                "gap" => scalar!(a, b, gap),
                "tick_length" => scalar!(a, b, tick_length),
                "stroke_width" => scalar!(a, b, stroke_width),
                "dashes" => scalar!(a, b, dashes),
                "symbol" => scalar!(a, b, symbol),
                _ => return Err(unsupported(method)),
            }),
            Kind::Theme(b) => Kind::Theme(match method {
                "geometry" => {
                    let mut value: Value = a.one()?;
                    if let Some(object) = value.as_object_mut() {
                        for name in ["ink", "paper", "accent"] {
                            if let Some(v) = object.get_mut(name) {
                                *v = serde_json::to_value(color(v)?).map_err(|e| {
                                    error(DiagnosticCode::Validation, e.to_string())
                                })?;
                            }
                        }
                    }
                    b.clone()
                        .geometry(options::<crate::theme::GeometryTheme<crate::color::Paint>>(
                            value,
                        )?)
                }
                "preset" => scalar!(a, b, preset),
                _ => return Err(unsupported(method)),
            }),
            Kind::TextStyle(b) => Kind::TextStyle(match method {
                "size" => scalar!(a, b, size),
                "weight" => scalar!(a, b, weight),
                "font" => scalar!(a, b, font),
                "fallback" => scalar!(a, b, fallback),
                "color" => b.clone().color(a.color()?),
                "language" => string!(a, b, language),
                "direction" => scalar!(a, b, direction),
                "tabular" => scalar!(a, b, tabular),
                _ => return Err(unsupported(method)),
            }),
            Kind::Rich(b) => Kind::Rich(match method {
                "line_spacing" => scalar!(a, b, line_spacing),
                "rotation" => scalar!(a, b, rotation),
                _ => return Err(unsupported(method)),
            }),
            Kind::Title(b) => Kind::Title(match method {
                "line_spacing" => scalar!(a, b, line_spacing),
                "rotation" => scalar!(a, b, rotation),
                _ => return Err(unsupported(method)),
            }),
            Kind::Subtitle(b) => Kind::Subtitle(match method {
                "line_spacing" => scalar!(a, b, line_spacing),
                "rotation" => scalar!(a, b, rotation),
                _ => return Err(unsupported(method)),
            }),
            Kind::Caption(b) => Kind::Caption(match method {
                "line_spacing" => scalar!(a, b, line_spacing),
                "rotation" => scalar!(a, b, rotation),
                _ => return Err(unsupported(method)),
            }),
            Kind::Note(b) => Kind::Note(match method {
                "line_spacing" => scalar!(a, b, line_spacing),
                "rotation" => scalar!(a, b, rotation),
                _ => return Err(unsupported(method)),
            }),
            Kind::Footnote(b) => Kind::Footnote(match method {
                "line_spacing" => scalar!(a, b, line_spacing),
                "rotation" => scalar!(a, b, rotation),
                _ => return Err(unsupported(method)),
            }),
            Kind::VectorPath(b) => Kind::VectorPath(match method {
                "transform" => {
                    a.count(3)?;
                    b.clone()
                        .transform(crate::path::Affine::new(a.at(0)?)?, a.at(1)?, a.at(2)?)?
                }
                "anchor" => b.clone().anchor(a.one()?),
                "fill" => b.clone().fill(a.one::<Option<crate::color::Paint>>()?),
                "stroke" => b
                    .clone()
                    .stroke(a.one::<Option<crate::scene::Stroke<crate::color::Paint>>>()?),
                "overflow" => b.clone().overflow(a.one()?),
                _ => return Err(unsupported(method)),
            }),
            Kind::Labels(b) => Kind::Labels(match method {
                "id" => string!(a, b, id),
                "at" => xy!(a, b, at),
                "panel" => b.clone().panel(panel_key(a.one()?)?),
                "figure_at" => pair!(a, b, figure_at),
                "output_at" => pair!(a, b, output_at),
                "panel_at" => {
                    a.count(3)?;
                    let p = if a.0[0].is_null() {
                        None
                    } else {
                        Some(panel_key(a.at(0)?)?)
                    };
                    b.clone().panel_at(p, a.at(1)?, a.at(2)?)
                }
                "axes" => {
                    let (x, y) = a.pair::<String, String>()?;
                    b.clone().axes(x, y)
                }
                "text" => string!(a, b, text),
                "offset" => pair!(a, b, offset),
                "priority" => scalar!(a, b, priority),
                "collision" => scalar!(a, b, collision),
                "overflow" => scalar!(a, b, overflow),
                _ => return Err(unsupported(method)),
            }),
            Kind::Callout(b) => Kind::Callout(match method {
                "at" => xy!(a, b, at),
                "to_data" => xy!(a, b, to_data),
                "to" => scalar!(a, b, to),
                "text" => string!(a, b, text),
                "connector_origin" => scalar!(a, b, connector_origin),
                "offset" => pair!(a, b, offset),
                _ => return Err(unsupported(method)),
            }),
            Kind::Panel(b) => Kind::Panel(match method {
                "panel" => b.clone().panel(panel_key(a.one()?)?),
                _ => return Err(unsupported(method)),
            }),
            Kind::Inset(b) => Kind::Inset(match method {
                "id" => string!(a, b, id),
                "panel" => b.clone().panel(panel_key(a.one()?)?),
                "rectangle" => {
                    a.count(4)?;
                    b.clone().rectangle(a.at(0)?, a.at(1)?, a.at(2)?, a.at(3)?)
                }
                "x_view" => pair!(a, b, x_view),
                "y_view" => pair!(a, b, y_view),
                "guides" => scalar!(a, b, guides),
                _ => return Err(unsupported(method)),
            }),
            Kind::Format(b) => Kind::Format(match method {
                "notation" => scalar!(a, b, notation),
                "precision" => scalar!(a, b, precision),
                "locale" => scalar!(a, b, locale),
                "grouping" => scalar!(a, b, grouping),
                "prefix" => string!(a, b, prefix),
                "suffix" => string!(a, b, suffix),
                _ => return Err(unsupported(method)),
            }),
            Kind::Layout(b) => Kind::Layout(match method {
                "device_scale" => scalar!(a, b, device_scale),
                "font_size" => scalar!(a, b, font_size),
                "padding" => scalar!(a, b, padding),
                "minimum_plot" => scalar!(a, b, minimum_plot),
                "tick_length" => scalar!(a, b, tick_length),
                "label_gap" => scalar!(a, b, label_gap),
                "target_ticks" => scalar!(a, b, target_ticks),
                "max_ticks" => scalar!(a, b, max_ticks),
                "max_categories" => scalar!(a, b, max_categories),
                "max_vertices" => scalar!(a, b, max_vertices),
                "limits" => b.clone().limits(options(a.one()?)?),
                "figure_bounds" => {
                    let value = a.one::<Option<[f64; 4]>>()?;
                    b.clone().figure_bounds(
                        value
                            .map(|[x, y, w, h]| crate::Rect::new(x, y, w, h))
                            .transpose()?,
                    )
                }
                "host_theme" => scalar!(a, b, host_theme),
                "output_theme" => scalar!(a, b, output_theme),
                "interaction_theme" => scalar!(a, b, interaction_theme),
                _ => return Err(unsupported(method)),
            }),
            Kind::Render(b) => Kind::Render(match method {
                "line_bucket_width" => scalar!(a, b, line_bucket_width),
                "candle_bucket_width" => scalar!(a, b, candle_bucket_width),
                "max_columns" => scalar!(a, b, max_columns),
                _ => return Err(unsupported(method)),
            }),
            Kind::Stream(b) => Kind::Stream(match method {
                "transactions" => scalar!(a, b, transactions),
                "rows" => scalar!(a, b, rows),
                "bytes" => scalar!(a, b, bytes),
                "overload" => scalar!(a, b, overload),
                _ => return Err(unsupported(method)),
            }),
            Kind::Edit(b) => Kind::Edit(match method {
                "part" => scalar!(a, b, part),
                "horizontal" => scalar!(a, b, horizontal),
                "vertical" => scalar!(a, b, vertical),
                "x" => scalar!(a, b, x),
                "y" => scalar!(a, b, y),
                "preserve_order" => scalar!(a, b, preserve_order),
                _ => return Err(unsupported(method)),
            }),
            Kind::Link(b) => Kind::Link(match method {
                "selection" => scalar!(a, b, selection),
                "source_panel" => b.clone().source_panel(panel_key(a.one()?)?),
                "destination_panel" => b.clone().destination_panel(panel_key(a.one()?)?),
                "missing" => scalar!(a, b, missing),
                _ => return Err(unsupported(method)),
            }),
            Kind::Run(_) => return Err(unsupported(method)),
        }))
    }
}

// Registered formatter descriptors use the same exact-version host conversion as scales.
fn guide_formatter(mut value: Value) -> ChartResult<Option<crate::layout::GuideFormatter>> {
    if let Some(version) = value
        .get_mut("Registered")
        .and_then(|registered| registered.get_mut("operation"))
        .and_then(|operation| operation.get_mut("version"))
    {
        *version = Value::String(exact_u64(version)?.to_string());
    }
    serde_json::from_value(value).map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
}
