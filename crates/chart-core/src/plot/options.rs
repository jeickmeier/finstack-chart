//! Configuration for owning destinations and runtime services, separate from plot grammar.
use super::{AxisHandle, FieldHandle, LayerHandle, StyleBuilder, error};
use crate::{ChartResult, DiagnosticCode, Limits, Rect, grammar::PanelKey, layout::LayoutRequest};

macro_rules! option_fields {
    ($name:ident { $($field:ident: $ty:ty => $doc:literal),* $(,)? }) => {
        /// Optional destination layout settings; omitted values retain destination defaults.
        #[derive(Clone, Debug, Default)]
        pub struct $name { $( $field: Option<$ty>, )* }
        impl $name {
            $(#[doc = $doc] pub fn $field(mut self, value: $ty) -> Self { self.$field = Some(value); self })*
            /// Apply to a destination's supplied font/bounds/units without replacing its identity.
            pub fn apply(&self, request: &mut LayoutRequest) { $(if let Some(value) = &self.$field { request.$field.clone_from(value); })* }
        }
    };
}
option_fields!(LayoutOptions {
    device_scale: Option<f64> => "Set explicit device scale for automatic guide offsets; None selects headless policy.",
    font_size: f64 => "Set plain-label size in destination units.",
    padding: f64 => "Set the nonnegative inset in destination units.",
    minimum_plot: (f64, f64) => "Set minimum useful plot width and height.",
    tick_length: f64 => "Set guide tick length in destination units.",
    label_gap: f64 => "Set label/tick separation in destination units.",
    target_ticks: usize => "Set desired numeric/time tick count (2..128).",
    max_ticks: usize => "Set bounded generated/measured ticks per axis (2..4096).",
    max_categories: usize => "Bound categorical training/layout catalogs.",
    max_vertices: usize => "Bound projected geometry before destination callbacks.",
    limits: Limits => "Supply scene/resource/text/path budgets.",
    figure_bounds: Option<Rect> => "Set explicit enclosing figure clipping for specialist composition.",
    host_theme: crate::theme::ThemePatch<crate::color::Paint> => "Supply typed host theme tokens before authored overrides.",
    output_theme: crate::theme::ThemePatch<crate::color::Paint> => "Supply typed output theme overrides after authoring.",
    interaction_theme: std::collections::BTreeMap<crate::LayerId, crate::theme::ThemePatch<crate::color::Paint>> => "Supply interaction styling for exact layer identities."
});
/// Begin destination layout options without choosing a font, output units or physical size.
pub fn layout_options() -> LayoutOptions {
    LayoutOptions::default()
}
impl LayoutOptions {
    /// Configure host tokens through the shared style builder.
    pub fn host_style(self, style: StyleBuilder) -> Self {
        self.host_theme(style.patch)
    }
    /// Configure final output tokens through the shared style builder.
    pub fn output_style(self, style: StyleBuilder) -> Self {
        self.output_theme(style.patch)
    }
}

/// Explicit screen-density configuration; statistical density estimation is a separate feature.
#[derive(Clone, Debug, Default)]
pub struct RenderOptions {
    options: crate::dense::DensityOptions,
    volume: Vec<(LayerHandle, FieldHandle)>,
}
/// Begin the existing line/candle density defaults for native rendering.
pub fn render_options() -> RenderOptions {
    RenderOptions::default()
}
impl RenderOptions {
    /// Select horizontal line bucket width; None retains exact line paths.
    pub fn line_bucket_width(mut self, width: Option<f64>) -> Self {
        self.options.line_bucket_width = width;
        self
    }
    /// Select horizontal candle bucket width; None retains exact supplied candles.
    pub fn candle_bucket_width(mut self, width: Option<f64>) -> Self {
        self.options.candle_bucket_width = width;
        self
    }
    /// Bind optional supplied candle volume without fabricating absent volume.
    pub fn candle_volume(mut self, layer: LayerHandle, field: FieldHandle) -> Self {
        self.volume.push((layer, field));
        self
    }
    /// Bound horizontal buckets per panel (1..65536).
    pub fn max_columns(mut self, columns: usize) -> Self {
        self.options.max_columns = columns;
        self
    }
    /// Resolve typed handles against the current chart and validate the shared density policy.
    pub fn build(
        &self,
        chart: &crate::runtime::Chart,
    ) -> ChartResult<crate::dense::DensityOptions> {
        let mut options = self.options.clone();
        for (handle, field) in &self.volume {
            let layer = chart
                .definition()
                .layers
                .iter()
                .find(|l| l.id == handle.id())
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        "Density volume names an absent layer.",
                    )
                })?;
            let mut input = layer.data;
            while let crate::grammar::DataRef::Transform(id) = input {
                input = chart
                    .definition()
                    .transforms
                    .iter()
                    .find(|t| t.id == id)
                    .ok_or_else(|| {
                        error(
                            DiagnosticCode::MissingResource,
                            "Density volume transform is absent.",
                        )
                    })?
                    .input;
            }
            if input != crate::grammar::DataRef::Dataset(field.dataset) {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Volume field belongs to another dataset.",
                ));
            }
            options.candle_volume.insert(handle.id(), field.id());
        }
        options.validate()?;
        Ok(options)
    }
}
/// Explicit bounded ingestion configuration; accepted, committed and painted remain distinct.
#[derive(Clone, Copy, Debug, Default)]
pub struct StreamOptions {
    limits: crate::ingestion::QueueLimits,
}
/// Begin existing lossless backpressure and queue limits.
pub fn stream_options() -> StreamOptions {
    StreamOptions::default()
}
impl StreamOptions {
    /// Bound waiting transactions, including metadata-only operations.
    pub fn transactions(mut self, value: usize) -> Self {
        self.limits.transactions = value;
        self
    }
    /// Bound queued incoming/replacement/removal row charges.
    pub fn rows(mut self, value: usize) -> Self {
        self.limits.rows = value;
        self
    }
    /// Bound conservative queued payload bytes.
    pub fn bytes(mut self, value: usize) -> Self {
        self.limits.bytes = value;
        self
    }
    /// Explicitly choose lossless backpressure or arriving-transaction drop accounting.
    pub fn overload(mut self, value: crate::ingestion::OverloadPolicy) -> Self {
        self.limits.overload = value;
        self
    }
    /// Validate through the existing queue constructor, without accepting any work.
    pub fn build(self) -> ChartResult<crate::ingestion::QueueLimits> {
        Ok(crate::ingestion::IngestionQueue::new(self.limits)?.limits())
    }
}
/// Annotation-edit policy using its declared coordinate units and the shared editor.
#[derive(Clone, Debug)]
pub struct AnnotationEditBuilder {
    id: String,
    part: crate::editing::AnnotationPart,
    constraints: crate::editing::EditConstraints,
}
/// Select a stable authored annotation; building pins the current gesture/presented scene.
pub fn annotation_edit(id: impl Into<String>) -> AnnotationEditBuilder {
    AnnotationEditBuilder {
        id: id.into(),
        part: crate::editing::AnnotationPart::Anchor,
        constraints: Default::default(),
    }
}
impl AnnotationEditBuilder {
    /// Select anchor, leader endpoint or both endpoints.
    pub fn part(mut self, part: crate::editing::AnnotationPart) -> Self {
        self.part = part;
        self
    }
    /// Allow or prohibit horizontal movement.
    pub fn horizontal(mut self, enabled: bool) -> Self {
        self.constraints.horizontal = enabled;
        self
    }
    /// Allow or prohibit vertical movement.
    pub fn vertical(mut self, enabled: bool) -> Self {
        self.constraints.vertical = enabled;
        self
    }
    /// Set numeric, exact timestamp or category constraints in horizontal source units.
    pub fn x(mut self, value: crate::editing::ValueConstraint) -> Self {
        self.constraints.x = Some(value);
        self
    }
    /// Set numeric, exact timestamp or category constraints in vertical source units.
    pub fn y(mut self, value: crate::editing::ValueConstraint) -> Self {
        self.constraints.y = Some(value);
        self
    }
    /// Preserve endpoint ordering horizontally (true) or vertically (false), or permit crossing.
    pub fn preserve_order(mut self, horizontal: Option<bool>) -> Self {
        self.constraints.preserve_order = horizontal;
        self
    }
    /// Validate and pin the same acknowledged coordinate basis used by native editing.
    pub fn build(
        self,
        chart: &crate::runtime::Chart,
    ) -> ChartResult<crate::editing::AnnotationEditor> {
        crate::editing::AnnotationEditor::new(
            chart.navigation()?.presented().clone(),
            &self.id,
            self.part,
            self.constraints,
        )
    }
}
/// Linked-view configuration; messages retain exact values/provenance and root-origin fences.
#[derive(Clone, Debug)]
pub struct LinkBuilder {
    origin: String,
    axes: Vec<crate::linking::AxisLink>,
    selection: bool,
    source_panel: Option<PanelKey>,
    destination_panel: Option<PanelKey>,
    missing: crate::linking::MissingMatch,
}
/// Configure one sender's durable messages and explicit sender/receiver axis pairings.
pub fn link(origin: impl Into<String>) -> LinkBuilder {
    LinkBuilder {
        origin: origin.into(),
        axes: vec![],
        selection: false,
        source_panel: None,
        destination_panel: None,
        missing: crate::linking::MissingMatch::Reject,
    }
}
impl LinkBuilder {
    /// Pair axis handles; this explicitly asserts the application units agree.
    pub fn axis(mut self, source: AxisHandle, destination: AxisHandle) -> Self {
        self.axes.push(crate::linking::AxisLink {
            source: source.id(),
            destination: destination.id(),
        });
        self
    }
    /// Include explicit selection/clear messages.
    pub fn selection(mut self, enabled: bool) -> Self {
        self.selection = enabled;
        self
    }
    /// Select the sender facet's coordinate basis.
    pub fn source_panel(mut self, panel: PanelKey) -> Self {
        self.source_panel = Some(panel);
        self
    }
    /// Select the receiver facet's coordinate basis.
    pub fn destination_panel(mut self, panel: PanelKey) -> Self {
        self.destination_panel = Some(panel);
        self
    }
    /// Reject missing identities or report and omit them without source-row substitution.
    pub fn missing(mut self, policy: crate::linking::MissingMatch) -> Self {
        self.missing = policy;
        self
    }
    /// Capture from an effective durable event; linked echoes remain None.
    pub fn capture(
        &self,
        chart: &mut crate::runtime::Chart,
        event: &crate::state::StateEvent,
    ) -> ChartResult<Option<crate::linking::LinkMessage>> {
        let inspector = link_inspector(chart)?;
        crate::linking::LinkMessage::from_event(
            &self.origin,
            event,
            &inspector,
            chart.state(),
            &self.axes.iter().map(|a| a.source).collect::<Vec<_>>(),
            self.source_panel.as_ref(),
            self.selection,
        )
    }
    /// Resolve to a typed action and unmatched report; dispatch remains an explicit runtime command.
    pub fn resolve(
        &self,
        chart: &mut crate::runtime::Chart,
        message: &crate::linking::LinkMessage,
    ) -> ChartResult<crate::linking::LinkedUpdate> {
        let inspector = link_inspector(chart)?;
        message.resolve(
            &inspector,
            &self.axes,
            self.destination_panel.as_ref(),
            self.missing,
        )
    }
}

fn link_inspector(chart: &mut crate::runtime::Chart) -> ChartResult<crate::inspection::Inspector> {
    let stamp = chart
        .reducer()
        .presented()
        .ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "Linking requires an acknowledged scene.",
            )
        })?
        .scene()
        .stamp();
    chart.inspector(stamp, false)
}
