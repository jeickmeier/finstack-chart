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
    pub(super) id: ChartResult<LayerId>,
    pub(super) name: Option<String>,
    pub(super) data: Option<Data>,
    pub(super) input: Option<super::TransformRef>,
    pub(super) mappings: AesBuilder,
    pub(super) inherit: bool,
    pub(super) geom: Geom,
    pub(super) style: Style,
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
    pub(super) candle_colors: Option<CandleColors>,
    pub(super) theme: super::StyleBuilder,
    pub(super) failure: Option<crate::Diagnostic>,
}
impl LayerBuilder {
    fn new(geom: Geom) -> Self {
        Self {
            id: fresh_id().map(LayerId::new),
            name: None,
            data: None,
            input: None,
            mappings: AesBuilder::default(),
            inherit: true,
            geom,
            style: Style::default(),
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
    /// Set a constant point radius or line stroke width in baseline destination units.
    pub fn size(mut self, value: f64) -> Self {
        self.style.radius = value;
        self.style.stroke_width = value;
        self
    }
    /// Set a constant fill/stroke color; mapped color remains authoritative.
    pub fn color(mut self, color: crate::scene::Color) -> Self {
        self.style.color = color;
        self
    }
    /// Map resolved group identities through an explicit named color scale.
    pub fn color_group(mut self, scale: impl Into<String>) -> Self {
        self.generated_color = Some(ColorInput::Group);
        self.generated_color_scale = Some(scale.into());
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
    pub fn candle_colors(mut self, colors: CandleColors) -> Self {
        self.candle_colors = Some(colors);
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
            Geom::Line { order, .. } | Geom::Area { order, .. } | Geom::Ribbon { order, .. } => {
                *order = value
            }
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
            Geom::Line { connect_gaps, .. }
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
            let grouping = mapped
                .group
                .as_ref()
                .map(|v| v.field(data))
                .transpose()?
                .map_or(Grouping::All, Grouping::Field);
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
        layer.style = self.style;
        if let Some(stat) = &self.stat {
            layer.statistic = stat.lower(data, &mapped)?;
            if self.generated.is_none()
                && let Some(mappings) = stat.default_mappings(layer.geom)?
            {
                layer.mappings = mappings;
            }
        }
        if let Some(mappings) = &self.generated {
            layer.mappings = mappings.clone();
        }
        layer.filters = self
            .filters
            .iter()
            .map(|f| f.lower(data))
            .collect::<ChartResult<_>>()?;
        if let Some(position) = &self.position {
            layer.position = position.lower()?;
        }
        layer.scope = self.scope;
        layer.facet = self.facet.clone();
        layer.clip = self.clip;
        layer.invalid = self.invalid;
        layer.geometry_extension = self.extension.clone();
        layer.candle_colors = self.candle_colors;
        layer.inherit = false;
        Ok((layer, mapped))
    }
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
