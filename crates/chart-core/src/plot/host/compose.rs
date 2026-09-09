use super::*;

impl Component {
    /// Compose typed components directly; misplaced figure/stage/style components reject.
    pub fn with(&self, method: &str, other: &Self) -> ChartResult<Self> {
        Ok(Self(match (&self.0, method, &other.0) {
            (Kind::Expression(a), method, Kind::Expression(b)) => {
                Kind::Expression(a.with(method, b)?)
            }
            (Kind::Aes(a), method, Kind::Expression(Expr::Source(expr))) => {
                Kind::Aes(match method {
                    "x" => a.clone().x(expr.clone()),
                    "y" => a.clone().y(expr.clone()),
                    "x2" => a.clone().x2(expr.clone()),
                    "y2" => a.clone().y2(expr.clone()),
                    "low" => a.clone().low(expr.clone()),
                    "high" => a.clone().high(expr.clone()),
                    "size" => a.clone().size(expr.clone()),
                    _ => return Err(unsupported(method)),
                })
            }
            (Kind::Filter(a), "expression", Kind::Expression(Expr::Source(expr))) => {
                Kind::Filter(a.clone().expression(expr.clone()))
            }
            (Kind::Stat(a), method, Kind::Expression(Expr::Source(expr))) => {
                Kind::Stat(match method {
                    "x" => a.clone().x(expr.clone()),
                    "y" => a.clone().y(expr.clone()),
                    _ => return Err(unsupported(method)),
                })
            }
            (Kind::StatAes(a), method, Kind::Expression(Expr::Stat(expr))) => {
                Kind::StatAes(match method {
                    "x" => a.clone().x(expr.clone()),
                    "y" => a.clone().y(expr.clone()),
                    "x2" => a.clone().x2(expr.clone()),
                    "y2" => a.clone().y2(expr.clone()),
                    "size" => a.clone().size(expr.clone()),
                    _ => return Err(unsupported(method)),
                })
            }
            (Kind::BinAes(a), method, Kind::Expression(Expr::Bin(expr))) => {
                Kind::BinAes(match method {
                    "x" => a.clone().x(expr.clone()),
                    "y" => a.clone().y(expr.clone()),
                    "x2" => a.clone().x2(expr.clone()),
                    "y2" => a.clone().y2(expr.clone()),
                    "size" => a.clone().size(expr.clone()),
                    _ => return Err(unsupported(method)),
                })
            }
            (Kind::ScaleAes(a), method, Kind::Expression(Expr::Scale(expr))) => {
                Kind::ScaleAes(match method {
                    "size" => a.clone().size(expr.clone()),
                    "color" => a.clone().color(expr.clone()),
                    _ => return Err(unsupported(method)),
                })
            }
            (Kind::Layer(a), "after_scale", Kind::ScaleAes(b)) => {
                Kind::Layer(a.clone().after_scale(b.clone()))
            }
            (Kind::Layer(b), "aes", Kind::Aes(v)) => Kind::Layer(b.clone().aes(v.clone())),
            (Kind::Layer(b), "stat", Kind::Stat(v)) => Kind::Layer(b.clone().stat(v.clone())),
            (Kind::Layer(b), "after_stat", Kind::StatAes(v)) => {
                Kind::Layer(b.clone().after_stat(v.clone()))
            }
            (Kind::Layer(b), "after_bin", Kind::BinAes(v)) => {
                Kind::Layer(b.clone().after_bin(v.clone()))
            }
            (Kind::Layer(b), "filter", Kind::Filter(v)) => Kind::Layer(b.clone().filter(v.clone())),
            (Kind::Layer(b), "position", Kind::Position(v)) => {
                Kind::Layer(b.clone().position(v.clone()))
            }
            (Kind::Layer(b), "style", Kind::Style(v)) => Kind::Layer(b.clone().style(v.clone())),
            (Kind::Layer(b), "from_transform", Kind::Transform(v)) => {
                Kind::Layer(b.clone().from_transform(v.handle()?))
            }
            (Kind::Transform(b), "aes", Kind::Aes(v)) => Kind::Transform(b.clone().aes(v.clone())),
            (Kind::Transform(b), "filter", Kind::Filter(v)) => {
                Kind::Transform(b.clone().filter(v.clone()))
            }
            (Kind::Transform(b), "from_transform", Kind::Transform(v)) => {
                Kind::Transform(b.clone().from_transform(v.handle()?))
            }
            (Kind::Guide(b), "text_style", Kind::TextStyle(v)) => {
                Kind::Guide(b.clone().text_style(v.clone()))
            }
            (Kind::Guide(b), "rich_label", Kind::Rich(v)) => {
                Kind::Guide(b.clone().rich_label(v.clone()))
            }
            (Kind::Guide(b), "format", Kind::Format(v)) => Kind::Guide(b.clone().format(v.clone())),
            (Kind::Axis(b), "scale", Kind::Scale(v)) => Kind::Axis(b.clone().scale(v.clone())),
            (Kind::Axis(b), "coordinate_scale", Kind::Scale(v)) => {
                Kind::Axis(b.clone().coordinate_scale(v.clone()))
            }
            (Kind::Axis(b), "text_style", Kind::TextStyle(v)) => {
                Kind::Axis(b.clone().text_style(v.clone()))
            }
            (Kind::Axis(b), "rich_label", Kind::Rich(v)) => {
                Kind::Axis(b.clone().rich_label(v.clone()))
            }
            (Kind::Axis(b), "format", Kind::Format(v)) => Kind::Axis(b.clone().format(v.clone())),
            (Kind::Theme(b), "style", Kind::Style(v)) => Kind::Theme(b.clone().style(v.clone())),
            (Kind::Run(b), "style", Kind::TextStyle(v)) => Kind::Run(b.clone().style(v.clone())),
            (Kind::Rich(b), "style", Kind::TextStyle(v)) => Kind::Rich(b.clone().style(v.clone())),
            (Kind::Rich(b), "run", Kind::Run(v)) => Kind::Rich(b.clone().run(v.clone())),
            (Kind::Rich(b), "line", Kind::Run(v)) => Kind::Rich(b.clone().line([v.clone()])),
            (Kind::Title(b), "style", Kind::TextStyle(v)) => {
                Kind::Title(b.clone().style(v.clone()))
            }
            (Kind::Subtitle(b), "style", Kind::TextStyle(v)) => {
                Kind::Subtitle(b.clone().style(v.clone()))
            }
            (Kind::Caption(b), "style", Kind::TextStyle(v)) => {
                Kind::Caption(b.clone().style(v.clone()))
            }
            (Kind::Note(b), "style", Kind::TextStyle(v)) => Kind::Note(b.clone().style(v.clone())),
            (Kind::Footnote(b), "style", Kind::TextStyle(v)) => {
                Kind::Footnote(b.clone().style(v.clone()))
            }
            (Kind::Title(b), "rich", Kind::Rich(v)) => Kind::Title(b.clone().rich(v.clone())),
            (Kind::Subtitle(b), "rich", Kind::Rich(v)) => Kind::Subtitle(b.clone().rich(v.clone())),
            (Kind::Caption(b), "rich", Kind::Rich(v)) => Kind::Caption(b.clone().rich(v.clone())),
            (Kind::Note(b), "rich", Kind::Rich(v)) => Kind::Note(b.clone().rich(v.clone())),
            (Kind::Footnote(b), "rich", Kind::Rich(v)) => Kind::Footnote(b.clone().rich(v.clone())),
            (Kind::Labels(b), "style", Kind::TextStyle(v)) => {
                Kind::Labels(b.clone().style(v.clone()))
            }
            (Kind::Labels(b), "rich", Kind::Rich(v)) => Kind::Labels(b.clone().rich(v.clone())),
            (Kind::Callout(b), "label", Kind::Labels(v)) => {
                Kind::Callout(b.clone().label(v.clone()))
            }
            (Kind::Callout(b), "style", Kind::TextStyle(v)) => {
                Kind::Callout(b.clone().style(v.clone()))
            }
            (Kind::Panel(b), "style", Kind::TextStyle(v)) => {
                Kind::Panel(b.clone().style(v.clone()))
            }
            (Kind::Panel(b), "rich", Kind::Rich(v)) => Kind::Panel(b.clone().rich(v.clone())),
            (Kind::Inset(b), "layer", Kind::Layer(v)) => Kind::Inset(b.clone().layer(v.handle()?)),
            (Kind::Layout(b), "host_style", Kind::Style(v)) => {
                Kind::Layout(b.clone().host_style(v.clone()))
            }
            (Kind::Layout(b), "output_style", Kind::Style(v)) => {
                Kind::Layout(b.clone().output_style(v.clone()))
            }
            _ => return Err(unsupported(method)),
        }))
    }
    /// Add a per-layer theme token patch without exposing or manufacturing layer IDs.
    pub fn theme_layer(&self, layer: &Self, style: &Self) -> ChartResult<Self> {
        let (Kind::Theme(b), Kind::Style(style)) = (&self.0, &style.0) else {
            return Err(unsupported("layer style"));
        };
        Ok(Self(Kind::Theme(
            b.clone().layer(layer.layer_handle()?, style.clone()),
        )))
    }
    /// Bind a candle's supplied volume using exact layer/dataset field handles.
    pub fn candle_volume(&self, layer: &Self, field: FieldHandle) -> ChartResult<Self> {
        let Kind::Render(b) = &self.0 else {
            return Err(unsupported("candle volume"));
        };
        Ok(Self(Kind::Render(
            b.clone().candle_volume(layer.layer_handle()?, field),
        )))
    }
    /// Configure an explicit source/destination link between named axis components.
    pub fn link_axis(&self, source: &Self, destination: &Self) -> ChartResult<Self> {
        let (Kind::Link(b), Kind::Axis(source), Kind::Axis(destination)) =
            (&self.0, &source.0, &destination.0)
        else {
            return Err(unsupported("link axis"));
        };
        Ok(Self(Kind::Link(
            b.clone().axis(source.handle()?, destination.handle()?),
        )))
    }
    /// Attach one component through the same typed PlotBuilder method used by native Rust.
    pub fn attach(&self, slot: &str, plot: PlotBuilder) -> ChartResult<PlotBuilder> {
        Ok(match (slot, &self.0) {
            ("aes", Kind::Aes(v)) => plot.aes(v.clone()),
            ("layer", Kind::Layer(v)) => plot.layer(v.clone()),
            ("layer", Kind::Labels(v)) => plot.layer(v.clone()),
            ("layer", Kind::VectorPath(v)) => plot.layer(v.clone()),
            ("layer", Kind::Callout(v)) => plot.layer(v.clone()),
            ("title", Kind::Title(v)) => plot.title(v.clone()),
            ("subtitle", Kind::Subtitle(v)) => plot.subtitle(v.clone()),
            ("caption", Kind::Caption(v)) => plot.caption(v.clone()),
            ("source_note", Kind::Note(v)) => plot.source_note(v.clone()),
            ("footnote", Kind::Footnote(v)) => plot.footnote(v.clone()),
            ("theme", Kind::Theme(v)) => plot.theme(v.clone()),
            ("guide", Kind::Guide(v)) => plot.guide(v.clone()),
            ("axis", Kind::Axis(v)) => plot.axis(v.clone()),
            ("x_axis", Kind::Axis(v)) => plot.x_axis(v.clone()),
            ("y_axis", Kind::Axis(v)) => plot.y_axis(v.clone()),
            ("scale", Kind::Color(v)) => plot.scale(v.clone()),
            ("legend", Kind::Legend(v)) => plot.legend(v.clone()),
            ("facet", Kind::Facet(v)) => plot.facet(v.clone()),
            ("panel_letter", Kind::Panel(v)) => plot.panel_letter(v.clone()),
            ("inset", Kind::Inset(v)) => plot.inset(v.clone()),
            ("transform", Kind::Transform(v)) => plot.transform(v.clone()),
            _ => return Err(unsupported(slot)),
        })
    }
    /// Apply one component to an immutable definition edit, preserving its original source owner.
    pub fn attach_edit(&self, slot: &str, edit: PlotEditBuilder) -> ChartResult<PlotEditBuilder> {
        Ok(match (slot, &self.0) {
            ("title", Kind::Title(v)) => edit.title(v.clone()),
            ("subtitle", Kind::Subtitle(v)) => edit.subtitle(v.clone()),
            ("caption", Kind::Caption(v)) => edit.caption(v.clone()),
            ("source_note", Kind::Note(v)) => edit.source_note(v.clone()),
            ("footnote", Kind::Footnote(v)) => edit.footnote(v.clone()),
            ("theme", Kind::Theme(v)) => edit.theme(v.clone()),
            ("guide", Kind::Guide(v)) => edit.guide(v.clone()),
            ("axis" | "x_axis" | "y_axis", Kind::Axis(v)) => edit.axis(v.clone()),
            ("scale", Kind::Color(v)) => edit.scale(v.clone()),
            ("legend", Kind::Legend(v)) => edit.legend(v.clone()),
            ("facet", Kind::Facet(v)) => edit.facet(v.clone()),
            ("panel_letter", Kind::Panel(v)) => edit.panel_letter(v.clone()),
            ("inset", Kind::Inset(v)) => edit.inset(v.clone()),
            ("transform", Kind::Transform(v)) => edit.transform(v.clone()),
            ("annotation", Kind::Labels(v)) => edit.annotation(v.clone()),
            ("annotation", Kind::VectorPath(v)) => edit.annotation(v.clone()),
            ("annotation", Kind::Callout(v)) => edit.annotation(v.clone()),
            _ => return Err(unsupported(slot)),
        })
    }
    /// Replace a named complete layer while keeping its existing stable identity.
    pub fn edit_layer(&self, name: &str, edit: PlotEditBuilder) -> ChartResult<PlotEditBuilder> {
        let Kind::Layer(layer) = &self.0 else {
            return Err(unsupported("layer edit"));
        };
        Ok(edit.layer(name, layer.clone()))
    }
    /// Pin and validate an annotation-edit producer against the runtime's presented/gesture basis.
    pub fn annotation_editor(
        &self,
        chart: &crate::runtime::Chart,
    ) -> ChartResult<crate::editing::AnnotationEditor> {
        let Kind::Edit(edit) = &self.0 else {
            return Err(unsupported("annotation editor"));
        };
        edit.clone().build(chart)
    }
    /// Obtain the shared link producer; capture and dispatch remain explicit runtime commands.
    pub fn link_builder(&self) -> ChartResult<LinkBuilder> {
        let Kind::Link(link) = &self.0 else {
            return Err(unsupported("link builder"));
        };
        Ok(link.clone())
    }
}
