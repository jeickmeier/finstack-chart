use super::{
    LayerHandle, error, fresh_id,
    text::{TextStyle, plain},
};
use crate::{
    ChartResult, DiagnosticCode,
    composition::{Anchor, Annotation, Collision, ConnectorOrigin, Inset, PanelLetter, ScaleValue},
    grammar::{PanelKey, ScaleBindings},
    scales::Bounds,
    typography::RichText,
};

/// Fixed path annotation builder; geometry owns its snapshot independently of a mutable path.
#[derive(Clone, Debug)]
pub struct VectorPathBuilder(pub(super) crate::composition::VectorAnnotation);
/// Compose one path with a stable annotation identity and a black one-unit stroke.
pub fn vector_path(
    id: impl Into<String>,
    geometry: crate::path::PathGeometry,
) -> VectorPathBuilder {
    VectorPathBuilder(crate::composition::VectorAnnotation {
        id: id.into(),
        geometry,
        anchor: Anchor::Output { x: 0., y: 0. },
        fill: None,
        stroke: Some(crate::scene::Stroke {
            color: crate::theme::rgb(0, 0, 0).into(),
            width: 1.,
        }),
        overflow: false,
    })
}
impl VectorPathBuilder {
    /// Transform local geometry before anchor placement with an explicit output error/work bound.
    pub fn transform(
        mut self,
        map: crate::path::Affine,
        max_error: f64,
        max_commands: usize,
    ) -> ChartResult<Self> {
        self.0.geometry = self.0.geometry.transformed(map, max_error, max_commands)?;
        Ok(self)
    }
    /// Position the local path origin through an explicit coordinate-space anchor.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.0.anchor = anchor;
        self
    }
    /// Set optional nonzero fill.
    pub fn fill<P: Into<crate::color::Paint>>(mut self, fill: Option<P>) -> Self {
        self.0.fill = fill.map(Into::into);
        self
    }
    /// Set optional solid stroke.
    pub fn stroke<P: Into<crate::color::Paint>>(
        mut self,
        stroke: Option<crate::scene::Stroke<P>>,
    ) -> Self {
        self.0.stroke = stroke.map(|s| s.map_color(Into::into));
        self
    }
    /// Use the full figure clip instead of the addressed panel clip.
    pub fn overflow(mut self, overflow: bool) -> Self {
        self.0.overflow = overflow;
        self
    }
}
impl From<f64> for ScaleValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}
impl From<&str> for ScaleValue {
    fn from(value: &str) -> Self {
        Self::Category(value.into())
    }
}
impl From<String> for ScaleValue {
    fn from(value: String) -> Self {
        Self::Category(value)
    }
}
/// Exact timestamp annotation/tick value in explicit source units.
pub fn time_value(value: i64, unit: crate::data::TimeUnit) -> ScaleValue {
    ScaleValue::Timestamp { value, unit }
}
/// One x/y annotation, never a per-observation replicated mark or a chart metadata bag.
#[derive(Clone, Debug)]
pub struct LabelsBuilder {
    pub(super) value: Annotation,
    pub(super) failure: Option<crate::Diagnostic>,
    pub(super) axes: Option<(String, String)>,
}
/// Start a fixed x/y annotation with stable identity.
///
/// ```compile_fail
/// use chart_core::prelude::*;
/// let invalid = labels().title("Not an annotation option");
/// ```
pub fn labels() -> LabelsBuilder {
    let id = fresh_id();
    LabelsBuilder {
        value: Annotation {
            id: id
                .as_ref()
                .map(|id| format!("annotation_{id}"))
                .unwrap_or_default(),
            anchor: Anchor::Data {
                panel: None,
                scales: ScaleBindings::default(),
                x: ScaleValue::Number(0.),
                y: ScaleValue::Number(0.),
            },
            text: plain(""),
            offset: [0., 0.],
            priority: 0,
            collision: Collision::Keep,
            callout: None,
            connector_origin: ConnectorOrigin::Label,
            overflow: false,
        },
        failure: id.err(),
        axes: None,
    }
}
impl LabelsBuilder {
    /// Use a caller-supplied stable annotation identity.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.value.id = id.into();
        self
    }
    /// Position by explicit numeric/category/timestamp data values.
    pub fn at(mut self, x: impl Into<ScaleValue>, y: impl Into<ScaleValue>) -> Self {
        self.value.anchor = Anchor::Data {
            panel: None,
            scales: ScaleBindings::default(),
            x: x.into(),
            y: y.into(),
        };
        self
    }
    /// Position by fractions of the selected panel's useful plot rectangle.
    pub fn panel_at(mut self, panel: Option<PanelKey>, x: f64, y: f64) -> Self {
        self.value.anchor = Anchor::Panel { panel, x, y };
        self
    }
    /// Position by fractions of the full figure.
    pub fn figure_at(mut self, x: f64, y: f64) -> Self {
        self.value.anchor = Anchor::Figure { x, y };
        self
    }
    /// Position in explicit destination units from the figure top-left.
    pub fn output_at(mut self, x: f64, y: f64) -> Self {
        self.value.anchor = Anchor::Output { x, y };
        self
    }
    /// Select the facet for a data/panel anchor.
    pub fn panel(mut self, panel: PanelKey) -> Self {
        match &mut self.value.anchor {
            Anchor::Data { panel: p, .. } | Anchor::Panel { panel: p, .. } => *p = Some(panel),
            _ => {
                self.failure = Some(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Figure/output anchors have no panel selector.",
                ))
            }
        };
        self
    }
    /// Resolve data coordinates through these named axes.
    pub fn axes(mut self, x: impl Into<String>, y: impl Into<String>) -> Self {
        self.axes = Some((x.into(), y.into()));
        self
    }
    /// Set logical annotation text; newlines create explicit lines.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.value.text = plain(text);
        self
    }
    /// Set independently styled rich content.
    pub fn rich(mut self, text: impl Into<RichText>) -> Self {
        self.value.text = text.into();
        self
    }
    /// Apply shared typography options.
    pub fn style(mut self, style: TextStyle) -> Self {
        style.apply(&mut self.value.text);
        self
    }
    /// Set destination-unit displacement without changing the semantic anchor.
    pub fn offset(mut self, x: f64, y: f64) -> Self {
        self.value.offset = [x, y];
        self
    }
    /// Set deterministic collision placement priority.
    pub fn priority(mut self, priority: i32) -> Self {
        self.value.priority = priority;
        self
    }
    /// Set keep/hide/shift-then-hide collision behavior.
    pub fn collision(mut self, collision: Collision) -> Self {
        self.value.collision = collision;
        self
    }
    /// Permit overflow beyond the addressed panel or figure clip.
    pub fn overflow(mut self, enabled: bool) -> Self {
        self.value.overflow = enabled;
        self
    }
    pub(super) fn lower(
        mut self,
        axes: &std::collections::BTreeMap<String, crate::ScaleId>,
    ) -> ChartResult<Annotation> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        if let Some((x, y)) = self.axes {
            let Anchor::Data { scales, .. } = &mut self.value.anchor else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Named axes require a data anchor.",
                ));
            };
            scales.x = *axes.get(&x).ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("Unknown annotation x axis '{x}'."),
                )
            })?;
            scales.y = *axes.get(&y).ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("Unknown annotation y axis '{y}'."),
                )
            })?;
        }
        Ok(self.value)
    }
}
/// Text annotation with an explicit leader endpoint using the same annotation engine.
#[derive(Clone, Debug)]
pub struct CalloutBuilder {
    pub(super) label: LabelsBuilder,
}
/// Start a callout; choose both its label anchor and leader endpoint explicitly.
pub fn callout() -> CalloutBuilder {
    CalloutBuilder { label: labels() }
}
impl CalloutBuilder {
    /// Position the label using data coordinates.
    pub fn at(mut self, x: impl Into<ScaleValue>, y: impl Into<ScaleValue>) -> Self {
        self.label = self.label.at(x, y);
        self
    }
    /// Set a leader endpoint in any explicit coordinate space.
    pub fn to(mut self, anchor: Anchor) -> Self {
        self.label.value.callout = Some(anchor);
        self
    }
    /// Set a data-space leader endpoint using primary scales.
    pub fn to_data(self, x: impl Into<ScaleValue>, y: impl Into<ScaleValue>) -> Self {
        self.to(Anchor::Data {
            panel: None,
            scales: ScaleBindings::default(),
            x: x.into(),
            y: y.into(),
        })
    }
    /// Supply the complete label component, retaining its identity/anchor/text options.
    pub fn label(mut self, label: LabelsBuilder) -> Self {
        let endpoint = self.label.value.callout.take();
        let origin = self.label.value.connector_origin;
        self.label = label;
        self.label.value.callout = endpoint;
        self.label.value.connector_origin = origin;
        self
    }
    /// Set the logical label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.label = self.label.text(text);
        self
    }
    /// Apply shared label typography.
    pub fn style(mut self, style: TextStyle) -> Self {
        self.label = self.label.style(style);
        self
    }
    /// Join the label box or exact authored anchor to the leader endpoint.
    pub fn connector_origin(mut self, origin: ConnectorOrigin) -> Self {
        self.label.value.connector_origin = origin;
        self
    }
    /// Offset the label in destination units.
    pub fn offset(mut self, x: f64, y: f64) -> Self {
        self.label = self.label.offset(x, y);
        self
    }
}
/// One figure panel letter, separate from observed data and x/y annotations.
#[derive(Clone, Debug)]
pub struct PanelLetterBuilder {
    pub(super) value: PanelLetter,
}
/// Begin a panel letter in the existing top-left furniture slot.
pub fn panel_letter(text: impl Into<String>) -> PanelLetterBuilder {
    PanelLetterBuilder {
        value: PanelLetter {
            panel: None,
            text: plain(text),
        },
    }
}
impl PanelLetterBuilder {
    /// Select an explicit facet identity.
    pub fn panel(mut self, panel: PanelKey) -> Self {
        self.value.panel = Some(panel);
        self
    }
    /// Apply shared typography.
    pub fn style(mut self, style: TextStyle) -> Self {
        style.apply(&mut self.value.text);
        self
    }
    /// Set rich panel-label content.
    pub fn rich(mut self, text: impl Into<RichText>) -> Self {
        self.value.text = text.into();
        self
    }
}
/// Alternate viewport into already prepared layer data, never a separate stat population.
#[derive(Clone, Debug)]
pub struct InsetBuilder {
    pub(super) value: Inset,
    pub(super) failure: Option<crate::Diagnostic>,
}
/// Start an inset with explicit prepared-layer selection required before build.
pub fn inset() -> InsetBuilder {
    let id = fresh_id();
    InsetBuilder {
        value: Inset {
            id: id
                .as_ref()
                .map(|id| format!("inset_{id}"))
                .unwrap_or_default(),
            panel: None,
            rectangle: [0.6, 0.05, 0.35, 0.35],
            layers: vec![],
            x_view: None,
            y_view: None,
            guides: true,
        },
        failure: id.err(),
    }
}
impl InsetBuilder {
    /// Override the stable inset identity.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.value.id = id.into();
        self
    }
    /// Select the parent facet panel.
    pub fn panel(mut self, panel: PanelKey) -> Self {
        self.value.panel = Some(panel);
        self
    }
    /// Set x/y/width/height fractions of the parent plot.
    pub fn rectangle(mut self, x: f64, y: f64, width: f64, height: f64) -> Self {
        self.value.rectangle = [x, y, width, height];
        self
    }
    /// Add an existing prepared layer to this view.
    pub fn layer(mut self, layer: LayerHandle) -> Self {
        self.value.layers.push(layer.id());
        self
    }
    /// Set an independent calculation-space x viewport without rerunning statistics.
    pub fn x_view(mut self, start: f64, end: f64) -> Self {
        match Bounds::new(start, end) {
            Ok(v) => self.value.x_view = Some(v),
            Err(e) => self.failure = Some(e),
        };
        self
    }
    /// Set an independent y viewport.
    pub fn y_view(mut self, start: f64, end: f64) -> Self {
        match Bounds::new(start, end) {
            Ok(v) => self.value.y_view = Some(v),
            Err(e) => self.failure = Some(e),
        };
        self
    }
    /// Show or hide inset axes/tick labels.
    pub fn guides(mut self, visible: bool) -> Self {
        self.value.guides = visible;
        self
    }
}
