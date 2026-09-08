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
                "group_all" => empty!(a, b, group_all),
                "color_scale" => string!(a, b, color_scale),
                _ => return Err(unsupported(method)),
            }),
            Kind::Layer(b) => Kind::Layer(match method {
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
                "candle_colors" => scalar!(a, b, candle_colors),
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
                _ => return Err(unsupported(method)),
            }),
            Kind::Axis(b) => Kind::Axis(match method {
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
                _ => return Err(unsupported(method)),
            }),
            Kind::Color(b) => Kind::Color(match method {
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
                "gradient" => scalar!(a, b, gradient),
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
