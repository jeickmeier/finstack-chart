//! Owned host syntax dispatch. Each call immediately updates a canonical typed builder.
//! Only small scalar options cross JSON here; owned data and components remain typed handles.
mod compose;
mod expression;
use expression::Expr;
mod configure;
pub use configure::panel_key;
mod draft;
use super::*;
use crate::grammar::{BinNumeric, GroupValue, StatNumeric};
pub use draft::Draft;
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Clone)]
#[expect(
    clippy::large_enum_variant,
    reason = "Authoring-only values; inline builders avoid a separate allocation on every fluent option call."
)]
enum Kind {
    Expression(Expr),
    ScaleAes(AfterScaleAesBuilder),
    Aes(AesBuilder),
    Layer(LayerBuilder),
    Stat(StatBuilder),
    StatAes(StatAesBuilder),
    BinAes(BinAesBuilder),
    Position(PositionBuilder),
    Filter(FilterBuilder),
    Transform(TransformBuilder),
    Scale(ScaleBuilder),
    Axis(AxisBuilder),
    Guide(GuideBuilder),
    Color(ColorScaleBuilder),
    Legend(LegendBuilder),
    Facet(FacetBuilder),
    Style(StyleBuilder),
    Theme(ThemeBuilder),
    TextStyle(TextStyle),
    Run(TextRunBuilder),
    Rich(RichTextBuilder),
    Title(TitleBuilder),
    Subtitle(SubtitleBuilder),
    Caption(CaptionBuilder),
    Note(SourceNoteBuilder),
    Footnote(FootnoteBuilder),
    Labels(LabelsBuilder),
    VectorPath(VectorPathBuilder),
    Callout(CalloutBuilder),
    Panel(PanelLetterBuilder),
    Inset(InsetBuilder),
    Format(NumberFormatBuilder),
    Layout(LayoutOptions),
    Render(RenderOptions),
    Stream(StreamOptions),
    Edit(AnnotationEditBuilder),
    Link(LinkBuilder),
}
/// One owned primary builder, shared by Python and WASM syntax adapters.
#[derive(Clone)]
pub struct Component(Kind);
impl Component {
    /// Start a source expression from an owner-scoped field handle.
    pub fn source_expression(field: FieldHandle) -> Self {
        Self(Kind::Expression(Expr::Source(source_expr(field))))
    }
    /// Snapshot a standalone path into the canonical fixed-annotation builder.
    pub fn vector_path(id: &str, path: &crate::path::Path) -> Self {
        Self(Kind::VectorPath(super::vector_path(id, path.geometry())))
    }
    /// Attach a numeric aesthetic while preserving an owner-scoped field identity.
    /// Attach a typed text or line-type scale through the same owner-checked core path.
    pub fn value_scale_field(
        &self,
        target: crate::grammar::ValueAesthetic,
        field: FieldHandle,
        scale: crate::scales::MappedScaleSpec,
    ) -> ChartResult<Self> {
        let Kind::Layer(layer) = &self.0 else {
            return Err(unsupported("value_scale"));
        };
        Ok(Self(Kind::Layer(layer.clone().value_scale(
            target,
            Mapping::Handle(field),
            scale,
        ))))
    }
    pub fn numeric_scale_field(
        &self,
        target: crate::grammar::NumericAesthetic,
        field: FieldHandle,
        scale: crate::scales::MappedScaleSpec,
    ) -> ChartResult<Self> {
        let Kind::Layer(layer) = &self.0 else {
            return Err(unsupported("numeric_scale"));
        };
        Ok(Self(Kind::Layer(layer.clone().numeric_scale(
            target,
            Mapping::Handle(field),
            scale,
        ))))
    }
    /// Resolve a symbol catalog through an owner-scoped field handle.
    pub fn symbol_types_field(
        &self,
        field: FieldHandle,
        domain: Vec<String>,
        palette: Vec<crate::shape::SymbolKind>,
    ) -> ChartResult<Self> {
        let Kind::Layer(layer) = &self.0 else {
            return Err(unsupported("symbol_types"));
        };
        Ok(Self(Kind::Layer(layer.clone().symbol_types(
            Mapping::Handle(field),
            domain,
            palette,
        ))))
    }
    /// Attach an unnormalized shape parameter through an owned source field.
    pub fn shape_value_field(
        &self,
        target: crate::grammar::NumericAesthetic,
        field: FieldHandle,
    ) -> ChartResult<Self> {
        let Kind::Layer(layer) = &self.0 else {
            return Err(unsupported("shape_value"));
        };
        Ok(Self(Kind::Layer(
            layer.clone().shape_value(target, Mapping::Handle(field)),
        )))
    }
    /// Attach an unnormalized shape parameter through a core source expression.
    pub fn shape_value_expression(
        &self,
        target: crate::grammar::NumericAesthetic,
        expression: &Self,
    ) -> ChartResult<Self> {
        let (Kind::Layer(layer), Kind::Expression(Expr::Source(expr))) = (&self.0, &expression.0)
        else {
            return Err(unsupported("shape_value source expression"));
        };
        Ok(Self(Kind::Layer(
            layer
                .clone()
                .shape_value(target, Mapping::Expression(expr.clone())),
        )))
    }
    /// Attach a numeric aesthetic through a source expression without host evaluation.
    /// Attach a typed text or line-type scale through the same owner-checked core path.
    pub fn value_scale_expression(
        &self,
        target: crate::grammar::ValueAesthetic,
        expression: &Self,
        scale: crate::scales::MappedScaleSpec,
    ) -> ChartResult<Self> {
        let (Kind::Layer(layer), Kind::Expression(Expr::Source(expr))) = (&self.0, &expression.0)
        else {
            return Err(unsupported("value_scale source expression"));
        };
        Ok(Self(Kind::Layer(layer.clone().value_scale(
            target,
            Mapping::Expression(expr.clone()),
            scale,
        ))))
    }
    pub fn numeric_scale_expression(
        &self,
        target: crate::grammar::NumericAesthetic,
        expression: &Self,
        scale: crate::scales::MappedScaleSpec,
    ) -> ChartResult<Self> {
        let (Kind::Layer(layer), Kind::Expression(Expr::Source(expr))) = (&self.0, &expression.0)
        else {
            return Err(unsupported("numeric_scale source expression"));
        };
        Ok(Self(Kind::Layer(layer.clone().numeric_scale(
            target,
            Mapping::Expression(expr.clone()),
            scale,
        ))))
    }
}
struct Args(Vec<Value>);
impl Args {
    fn parse(input: &str) -> ChartResult<Self> {
        crate::portable::decode(input).map(Self)
    }
    fn count(&self, count: usize) -> ChartResult<()> {
        if self.0.len() == count {
            Ok(())
        } else {
            Err(error(
                DiagnosticCode::Validation,
                format!(
                    "Expected {count} builder arguments, received {}.",
                    self.0.len()
                ),
            ))
        }
    }
    fn at<T: DeserializeOwned>(&self, index: usize) -> ChartResult<T> {
        serde_json::from_value(
            self.0
                .get(index)
                .cloned()
                .ok_or_else(|| error(DiagnosticCode::Validation, "Missing builder argument."))?,
        )
        .map_err(|e| {
            error(
                DiagnosticCode::Validation,
                format!("Builder argument {index}: {e}"),
            )
        })
    }
    fn one<T: DeserializeOwned>(&self) -> ChartResult<T> {
        self.count(1)?;
        self.at(0)
    }
    fn string(&self) -> ChartResult<String> {
        self.one()
    }
    fn pair<T: DeserializeOwned, U: DeserializeOwned>(&self) -> ChartResult<(T, U)> {
        self.count(2)?;
        Ok((self.at(0)?, self.at(1)?))
    }
    fn mapping(&self) -> ChartResult<Mapping> {
        self.count(1)?;
        mapping(&self.0[0])
    }
    fn color(&self) -> ChartResult<crate::color::Paint> {
        self.count(1)?;
        color(&self.0[0])
    }
}
fn unsupported(name: &str) -> Diagnostic {
    error(
        DiagnosticCode::UnsupportedCapability,
        format!("This builder does not support '{name}'."),
    )
}
pub(super) fn options<T: Default + serde::Serialize + DeserializeOwned>(
    value: Value,
) -> ChartResult<T> {
    let Value::Object(patch) = value else {
        return Err(error(
            DiagnosticCode::Validation,
            "Options require named fields.",
        ));
    };
    let mut base = serde_json::to_value(T::default())
        .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?;
    let target = base
        .as_object_mut()
        .ok_or_else(|| error(DiagnosticCode::Validation, "Options have no named fields."))?;
    for (key, mut value) in patch {
        if target
            .get(&key)
            .is_some_and(|v| v.as_str().is_some_and(|s| s.parse::<u64>().is_ok()))
            && value.is_number()
        {
            value = Value::String(exact_u64(&value)?.to_string());
        }
        target.insert(key, value);
    }
    serde_json::from_value(base).map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
}
fn exact_u64(value: &Value) -> ChartResult<u64> {
    if let Some(s) = value.as_str() {
        return s.parse().map_err(|_| {
            error(
                DiagnosticCode::Validation,
                "Expected an exact unsigned 64-bit integer.",
            )
        });
    }
    value
        .as_u64()
        .filter(|v| *v <= 9_007_199_254_740_991)
        .ok_or_else(|| {
            error(
                DiagnosticCode::PrecisionLoss,
                "Large integers require an exact integer adapter.",
            )
        })
}
fn numeric_input(value: &Value) -> ChartResult<NumericScaleInput> {
    if value.get("Statistical").is_some() {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Statistical {
            #[serde(rename = "Statistical")]
            field: crate::grammar::StatField,
        }
        let input: Statistical = serde_json::from_value(value.clone())
            .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?;
        Ok(NumericScaleInput::Statistical(input.field))
    } else {
        Ok(NumericScaleInput::Source(mapping(value)?))
    }
}
fn mapping(value: &Value) -> ChartResult<Mapping> {
    match value {
        Value::String(name) => Ok(name.clone().into()),
        Value::Number(value) => value
            .as_f64()
            .map(Mapping::Literal)
            .ok_or_else(|| error(DiagnosticCode::Validation, "Invalid numeric literal.")),
        _ => {
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct Timestamp {
                field: String,
                #[serde(with = "crate::portable::signed")]
                origin: i64,
            }
            let value: Timestamp = serde_json::from_value(value.clone()).map_err(|_| error(DiagnosticCode::SchemaConflict, "Source mappings require a field name, numeric literal or exact timestamp mapping."))?;
            Ok(Mapping::Timestamp {
                field: value.field,
                origin: value.origin,
            })
        }
    }
}
fn stat_numeric(value: &Value) -> ChartResult<StatNumeric> {
    if let Some(v) = value.as_f64() {
        return Ok(StatNumeric::Literal(v));
    }
    if value.is_string() {
        return serde_json::from_value(value.clone())
            .map(StatNumeric::Field)
            .map_err(|e| error(DiagnosticCode::SchemaConflict, e.to_string()));
    }
    if let Ok(field) = serde_json::from_value::<crate::grammar::StatField>(value.clone()) {
        return Ok(StatNumeric::Field(field));
    }
    serde_json::from_value(value.clone())
        .map_err(|e| error(DiagnosticCode::SchemaConflict, e.to_string()))
}
fn bin_numeric(value: &Value) -> ChartResult<BinNumeric> {
    if let Some(v) = value.as_f64() {
        return Ok(BinNumeric::Literal(v));
    }
    if value.is_string() {
        return serde_json::from_value(value.clone())
            .map(BinNumeric::Field)
            .map_err(|e| error(DiagnosticCode::SchemaConflict, e.to_string()));
    }
    serde_json::from_value(value.clone())
        .map_err(|e| error(DiagnosticCode::SchemaConflict, e.to_string()))
}
fn group(value: Value) -> ChartResult<GroupValue> {
    match value {
        Value::String(v) => Ok(GroupValue::Text(v)),
        Value::Bool(v) => Ok(GroupValue::Boolean(v)),
        Value::Number(v) => {
            if let Some(v) = v
                .as_i64()
                .filter(|v| v.unsigned_abs() <= 9_007_199_254_740_991)
            {
                Ok(GroupValue::Int(v))
            } else {
                Err(error(
                    DiagnosticCode::PrecisionLoss,
                    "Group integers require an exact integer adapter.",
                ))
            }
        }
        value => serde_json::from_value(value)
            .map_err(|e| error(DiagnosticCode::Validation, e.to_string())),
    }
}
fn groups(values: Vec<Value>) -> ChartResult<Vec<GroupValue>> {
    values.into_iter().map(group).collect()
}
/// Retain a parsed host color using the same descriptor contract as every authored input.
pub fn color(value: &Value) -> ChartResult<crate::color::Paint> {
    serde_json::from_value(value.clone())
        .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
}

fn scale_value(value: &Value) -> ChartResult<crate::composition::ScaleValue> {
    match value {
        Value::String(v) => Ok(v.clone().into()),
        Value::Number(v) => Ok(v
            .as_f64()
            .ok_or_else(|| error(DiagnosticCode::NumericalDomain, "Invalid coordinate."))?
            .into()),
        value => serde_json::from_value(value.clone())
            .map_err(|e| error(DiagnosticCode::Validation, e.to_string())),
    }
}
impl Component {
    /// Construct one named core component with positional scalar arguments.
    pub fn new(name: &str, arguments: &str) -> ChartResult<Self> {
        let a = Args::parse(arguments)?;
        macro_rules! empty {
            ($kind:ident, $ctor:ident) => {{
                a.count(0)?;
                Kind::$kind($ctor())
            }};
        }
        Ok(Self(match name {
            "source_expr" => Kind::Expression(Expr::Source(source_expr(a.mapping()?))),
            "stat_expr" => Kind::Expression(Expr::Stat(crate::grammar::Expression::read(a.one()?))),
            "bin_expr" => Kind::Expression(Expr::Bin(crate::grammar::Expression::read(a.one()?))),
            "after_scale_expr" => Kind::Expression(Expr::Scale(after_scale_expr(a.one()?))),
            "from_theme" => Kind::Expression(Expr::Scale(from_theme(a.one()?))),
            "scale_aes" => empty!(ScaleAes, scale_aes),
            "aes" => empty!(Aes, aes),
            "points" => empty!(Layer, points),
            "line" => empty!(Layer, line),
            "hierarchy" => Kind::Layer(hierarchy(
                a.one::<crate::grammar::HierarchyRecipe<String>>()?
                    .try_map_fields(|name| Ok(Mapping::from(name)))?,
            )),
            "hierarchy_tree" => {
                let (id, parent) = a.pair::<String, String>()?;
                Kind::Layer(hierarchy_tree(id, parent))
            }
            "hierarchy_cluster" => {
                let (id, parent) = a.pair::<String, String>()?;
                Kind::Layer(hierarchy_cluster(id, parent))
            }
            "hierarchy_icicle" => {
                let (id, parent) = a.pair::<String, String>()?;
                Kind::Layer(hierarchy_icicle(id, parent))
            }
            "hierarchy_sunburst" => {
                let (id, parent) = a.pair::<String, String>()?;
                Kind::Layer(hierarchy_sunburst(id, parent))
            }
            "hierarchy_treemap" => {
                let (id, parent) = a.pair::<String, String>()?;
                Kind::Layer(hierarchy_treemap(id, parent))
            }
            "hierarchy_pack" => {
                let (id, parent) = a.pair::<String, String>()?;
                Kind::Layer(hierarchy_pack(id, parent))
            }

            "shape_line" => empty!(Layer, shape_line),
            "shape_line_radial" => empty!(Layer, shape_line_radial),
            "shape_area_radial" => empty!(Layer, shape_area_radial),
            "shape_link" => Kind::Layer(shape_link(a.one()?)),
            "shape_link_horizontal" => empty!(Layer, shape_link_horizontal),
            "shape_link_vertical" => empty!(Layer, shape_link_vertical),
            "shape_link_radial" => empty!(Layer, shape_link_radial),
            "shape_symbol" => empty!(Layer, shape_symbol),
            "shape_arc" => empty!(Layer, shape_arc),
            "shape_pie" => empty!(Layer, shape_pie),
            "shape_area" => empty!(Layer, shape_area),
            "area" => empty!(Layer, area),
            "ribbon" => empty!(Layer, ribbon),
            "bars" => empty!(Layer, bars),
            "volume" => empty!(Layer, volume),
            "ohlc" => empty!(Layer, ohlc),
            "rule" => empty!(Layer, rule),
            "rectangle" => empty!(Layer, rectangle),
            "cells" => empty!(Layer, cells),
            "histogram" => empty!(Layer, histogram),
            "identity_stat" => empty!(Stat, identity_stat),
            "bin" => empty!(Stat, bin),
            "count" => empty!(Stat, count),
            "summary" => empty!(Stat, summary),
            "fit" => empty!(Stat, fit),
            "custom_stat" => {
                a.count(3)?;
                Kind::Stat(custom_stat(
                    a.at::<String>(0)?,
                    Revision::new(exact_u64(&a.0[1])?),
                    a.at(2)?,
                ))
            }
            "stat_aes" => empty!(StatAes, stat_aes),
            "bin_aes" => empty!(BinAes, bin_aes),
            "stack" => Kind::Position(stack(groups(a.one()?)?)),
            "shape_stack" => Kind::Position(shape_stack(groups(a.one()?)?)),
            "dodge" => Kind::Position(dodge(groups(a.one()?)?)),
            "jitter" => {
                a.count(1)?;
                Kind::Position(jitter(exact_u64(&a.0[0])?))
            }
            "filter" => Kind::Filter(filter(a.mapping()?)),
            "scale_linear" => empty!(Scale, scale_linear),
            "scale_reverse" => empty!(Scale, scale_reverse),
            "scale_sqrt" => empty!(Scale, scale_sqrt),
            "scale_log" => Kind::Scale(scale_log(a.one()?)),
            "scale_symlog" => Kind::Scale(scale_symlog(a.one()?)),
            "scale_band" => empty!(Scale, scale_band),
            "scale_point" => empty!(Scale, scale_point),
            "scale_utc" => empty!(Scale, scale_utc),
            "scale_date" => empty!(Scale, scale_date),
            "scale_duration" => empty!(Scale, scale_duration),
            "scale_binned" => Kind::Scale(scale_binned(a.one()?)),
            "scale_session" => Kind::Scale(scale_session(a.one()?)),
            "scale_numeric" => Kind::Scale(scale_numeric(a.one()?)),
            "scale_registered" => {
                a.count(3)?;
                Kind::Scale(scale_registered(
                    a.at::<String>(0)?,
                    crate::Revision::new(exact_u64(&a.0[1])?),
                    a.0[2].clone(),
                ))
            }
            "scale_band_d3" => Kind::Scale(scale_band_d3(a.one()?)),
            "scale_point_d3" => Kind::Scale(scale_point_d3(a.one()?)),
            "scale_calendar" => Kind::Scale(scale_calendar(a.one()?)),
            "axis_guide" => {
                let (name, source) = a.pair::<String, String>()?;
                Kind::Guide(axis_guide(name, source))
            }
            "x_axis" => empty!(Axis, x_axis),
            "y_axis" => empty!(Axis, y_axis),
            "color_discrete" => Kind::Color(color_discrete(a.string()?)),
            "color_mapped" => {
                a.count(2)?;
                Kind::Color(color_mapped(a.at::<String>(0)?, a.at(1)?))
            }
            "color_continuous" => {
                a.count(3)?;
                Kind::Color(color_continuous(a.at::<String>(0)?, a.at(1)?, a.at(2)?))
            }
            "legend" => empty!(Legend, legend),
            "facet_wrap" => Kind::Facet(facet_wrap(a.string()?)),
            "facet_grid" => {
                let (row, col) = a.pair::<String, String>()?;
                Kind::Facet(facet_grid(row, col))
            }
            "style" => empty!(Style, style),
            "theme" => empty!(Theme, theme),
            "text_style" => empty!(TextStyle, text_style),
            "text_run" => Kind::Run(text_run(a.string()?)),
            "rich_text" => Kind::Rich(rich_text(a.string()?)),
            "title" => Kind::Title(title(a.string()?)),
            "subtitle" => Kind::Subtitle(subtitle(a.string()?)),
            "caption" => Kind::Caption(caption(a.string()?)),
            "source_note" => Kind::Note(source_note(a.string()?)),
            "footnote" => Kind::Footnote(footnote(a.string()?)),
            "labels" => empty!(Labels, labels),
            "callout" => empty!(Callout, callout),
            "panel_letter" => Kind::Panel(panel_letter(a.string()?)),
            "inset" => empty!(Inset, inset),
            "number_format" => empty!(Format, number_format),
            "layout_options" => empty!(Layout, layout_options),
            "render_options" => empty!(Render, render_options),
            "stream_options" => empty!(Stream, stream_options),
            "annotation_edit" => Kind::Edit(annotation_edit(a.string()?)),
            "link" => Kind::Link(link(a.string()?)),
            _ => return Err(unsupported(name)),
        }))
    }
    /// Construct a named transform around the actual shared statistic builder.
    pub fn transform(name: &str, stat: &Self) -> ChartResult<Self> {
        let Kind::Stat(stat) = &stat.0 else {
            return Err(unsupported("transform statistic"));
        };
        Ok(Self(Kind::Transform(transform(name, stat.clone()))))
    }
    /// Resolve an owner-scoped source field without serializing its identity.
    pub fn field(&self, method: &str, field: FieldHandle) -> ChartResult<Self> {
        let mapping = Mapping::Handle(field);
        Ok(Self(match &self.0 {
            Kind::Layer(b) => Kind::Layer(match method {
                "hierarchy_value" => b.clone().hierarchy_value(mapping),
                "hierarchy_label" => b.clone().hierarchy_label(mapping),
                _ => return Err(unsupported(method)),
            }),
            Kind::Aes(b) => Kind::Aes(match method {
                "x" => b.clone().x(mapping),
                "y" => b.clone().y(mapping),
                "x2" => b.clone().x2(mapping),
                "y2" => b.clone().y2(mapping),
                "low" => b.clone().low(mapping),
                "high" => b.clone().high(mapping),
                "size" => b.clone().size(mapping),
                "group" => b.clone().group(mapping),
                "color" => b.clone().color(mapping),
                "fill" => b.clone().fill(mapping),
                "stroke" => b.clone().stroke(mapping),
                "shape" => b.clone().shape(mapping),
                "linetype" => b.clone().linetype(mapping),
                "alpha" => b.clone().alpha(mapping),
                "linewidth" => b.clone().linewidth(mapping),
                _ => return Err(unsupported(method)),
            }),
            Kind::Stat(b) => Kind::Stat(match method {
                "x" => b.clone().x(mapping),
                "y" => b.clone().y(mapping),
                "group" => b.clone().group(mapping),
                _ => return Err(unsupported(method)),
            }),
            _ => return Err(unsupported(method)),
        }))
    }
    /// Bind an exact owner-scoped source field in a registered statistic's parameter object.
    pub fn field_parameter(&self, name: &str, field: FieldHandle) -> ChartResult<Self> {
        let Kind::Stat(stat) = &self.0 else {
            return Err(unsupported("field parameter"));
        };
        Ok(Self(Kind::Stat(stat.clone().field_parameter(name, field))))
    }
    /// Retain an owned source handle as the input of a layer or shared transform.
    pub fn data(&self, data: &Data) -> ChartResult<Self> {
        Ok(Self(match &self.0 {
            Kind::Layer(b) => Kind::Layer(b.clone().data(data.clone())),
            Kind::Transform(b) => Kind::Transform(b.clone().data(data.clone())),
            _ => return Err(unsupported("data")),
        }))
    }
    /// Stable opaque layer identity, also used by native host wrappers for named state commands.
    pub fn layer_handle(&self) -> ChartResult<LayerHandle> {
        let Kind::Layer(b) = &self.0 else {
            return Err(unsupported("layer handle"));
        };
        b.handle()
    }
    /// Resolve reusable destination layout options without copying source data.
    pub fn layout_options(&self) -> ChartResult<LayoutOptions> {
        let Kind::Layout(b) = &self.0 else {
            return Err(unsupported("layout options"));
        };
        Ok(b.clone())
    }
    /// Resolve an owned queue configuration through the existing core validation.
    pub fn stream_options(&self) -> ChartResult<StreamOptions> {
        let Kind::Stream(b) = &self.0 else {
            return Err(unsupported("stream options"));
        };
        Ok(*b)
    }
    /// Resolve screen-density options against the owning runtime.
    pub fn render_options(&self) -> ChartResult<RenderOptions> {
        let Kind::Render(b) = &self.0 else {
            return Err(unsupported("render options"));
        };
        Ok(b.clone())
    }
}
