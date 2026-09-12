//! Portable figure furniture. Text positions declare data, plot-relative, figure or output units.
use crate::grammar::{PanelKey, ScaleBindings};
use crate::scales::Bounds;
use crate::typography::RichText;
use crate::{ChartResult, Diagnostic, DiagnosticCode, LayerId, Limits};
use serde::{Deserialize, Serialize};

/// An annotation coordinate in a resolved named scale's calculation units.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ScaleValue {
    /// Numeric calculation units, including explicitly transformed statistical units.
    Number(f64),
    /// Category label identity, never a transient ordinal.
    Category(String),
    /// Missing category identity, distinct from any display label.
    MissingCategory,
    /// Exact source timestamp and explicit tick unit.
    Timestamp {
        /// Source ticks, preserved as a decimal wire string.
        #[serde(with = "crate::portable::signed")]
        value: i64,
        /// Tick unit.
        unit: crate::data::TimeUnit,
    },
}
/// Explicit anchor semantics, preserved when output dimensions change.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Anchor {
    /// Named data scales in a single plot or explicit facet.
    Data {
        /// None addresses the single-panel chart.
        panel: Option<PanelKey>,
        /// Positional scale identities.
        scales: ScaleBindings,
        /// Horizontal source/calculation value.
        x: ScaleValue,
        /// Vertical source/calculation value.
        y: ScaleValue,
    },
    /// Fractions of the useful plot rectangle; (0,0) is top-left.
    Panel {
        /// None addresses the single-panel plot.
        panel: Option<PanelKey>,
        /// Horizontal fraction.
        x: f64,
        /// Vertical fraction.
        y: f64,
    },
    /// Fractions of the full figure; (0,0) is top-left.
    Figure {
        /// Horizontal fraction.
        x: f64,
        /// Vertical fraction.
        y: f64,
    },
    /// Absolute destination units from the full figure's top-left.
    Output {
        /// Horizontal displacement.
        x: f64,
        /// Vertical displacement.
        y: f64,
    },
}
/// Deterministic label collision fallback after testing finite candidate offsets.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Collision {
    /// Preserve the authored position, even when text overlaps.
    #[default]
    Keep,
    /// Hide a colliding label and report pressure; logical text remains in the definition.
    Hide,
    /// Test up/right/down/left at one then two line heights; hide with pressure if all fail.
    ShiftThenHide,
}
/// Whether a callout joins a laid-out label or two explicit authored endpoints.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConnectorOrigin {
    /// Nearest point on the laid-out label box, preserving ordinary callout behavior.
    #[default]
    Label,
    /// Exact authored anchor to callout endpoint, for threshold/range annotation geometry.
    Anchor,
}
impl ConnectorOrigin {
    fn is_default(&self) -> bool {
        *self == Self::Label
    }
}
/// Direct label or callout. Higher priority is placed first, ties retain declaration order.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    /// Stable authored identity.
    pub id: String,
    /// Top-left of the rotated text bounding box before output-unit offset.
    pub anchor: Anchor,
    /// Explicit line/run/font/rotation specification.
    pub text: RichText,
    /// Destination-unit displacement from the anchor.
    #[serde(default)]
    pub offset: [f64; 2],
    /// Higher values claim collision space first; all labels follow data in paint order.
    #[serde(default)]
    pub priority: i32,
    /// Explicit automatic collision policy.
    #[serde(default)]
    pub collision: Collision,
    /// Optional leader endpoint in its independently declared coordinate space.
    #[serde(default)]
    pub callout: Option<Anchor>,
    /// Connector origin; labels may be offset without moving threshold/range line endpoints.
    #[serde(default, skip_serializing_if = "ConnectorOrigin::is_default")]
    pub connector_origin: ConnectorOrigin,
    /// False clips to the addressed plot (or figure for a figure/output anchor).
    #[serde(default)]
    pub overflow: bool,
}
/// Stable panel letter; not a source-data target.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PanelLetter {
    /// None addresses the single-panel plot.
    pub panel: Option<PanelKey>,
    /// Rich letter/label placed at the plot top-left with a destination gap inset.
    pub text: RichText,
}
/// An inset is another view of already-prepared layers, never a separate statistical population.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Inset {
    /// Stable view identity (unique within this figure).
    pub id: String,
    /// Parent single plot or explicit facet.
    pub panel: Option<PanelKey>,
    /// x, y, width, height fractions of the parent useful plot, each bounded to `[0,1]`.
    pub rectangle: [f64; 4],
    /// Explicit prepared layer identities to show, nonempty and unique.
    pub layers: Vec<LayerId>,
    /// Independent horizontal viewport in calculation units; statistics are reused.
    pub x_view: Option<Bounds>,
    /// Independent vertical viewport in calculation units.
    pub y_view: Option<Bounds>,
    /// Whether the inset has axes/tick labels.
    pub guides: bool,
}
/// Versioned bounded publication composition, shared by every destination.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FigureComposition {
    /// Version one for legacy furniture; version two enables retained paths.
    pub version: u32,
    /// Figure title reserved above all panels.
    #[serde(default)]
    pub title: Option<RichText>,
    /// Subtitle reserved below title.
    #[serde(default)]
    pub subtitle: Option<RichText>,
    /// Caption reserved below panels.
    #[serde(default)]
    pub caption: Option<RichText>,
    /// Source notes below caption, in declaration order.
    #[serde(default)]
    pub source_notes: Vec<RichText>,
    /// Footnotes after source notes, in declaration order.
    #[serde(default)]
    pub footnotes: Vec<RichText>,
    /// Explicit panel letters.
    #[serde(default)]
    pub panel_letters: Vec<PanelLetter>,
    /// Data/relative/output direct labels and callouts.
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    /// Fixed retained paths in destination units, translated to an explicit anchor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<VectorAnnotation>,
    /// Bounded alternate views into prepared panels/layers.
    #[serde(default)]
    pub insets: Vec<Inset>,
}
impl Default for FigureComposition {
    fn default() -> Self {
        Self {
            version: 1,
            title: None,
            subtitle: None,
            caption: None,
            source_notes: vec![],
            footnotes: vec![],
            panel_letters: vec![],
            annotations: vec![],
            paths: vec![],
            insets: vec![],
        }
    }
}
impl FigureComposition {
    /// Validate text/identity/count/placement bounds before shaping or cloning inset geometry.
    pub fn validate(&self, limits: Limits) -> ChartResult<()> {
        if !matches!(self.version, 1 | 2) || (self.version == 1 && !self.paths.is_empty()) {
            return Err(Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported figure composition version.",
                "Use composition version two for retained paths, or version one for legacy furniture.",
            ));
        }
        if self.annotations.len().saturating_add(self.paths.len()) > 256
            || self.insets.len() > 4
            || self.panel_letters.len() > 256
            || self.source_notes.len() + self.footnotes.len() > 64
        {
            return Err(Diagnostic::error(
                DiagnosticCode::ResourceLimit,
                "Figure furniture exceeds its count budget.",
                "Use at most 256 annotations/letters, 64 notes and four insets.",
            ));
        }
        let mut remaining = limits.max_text_bytes;
        for text in self
            .title
            .iter()
            .chain(&self.subtitle)
            .chain(&self.caption)
            .chain(&self.source_notes)
            .chain(&self.footnotes)
            .chain(self.panel_letters.iter().map(|p| &p.text))
            .chain(self.annotations.iter().map(|a| &a.text))
        {
            text.validate(limits)?;
            let bytes = text
                .lines
                .iter()
                .flatten()
                .map(|r| r.text.len())
                .sum::<usize>();
            crate::limits::require_within(bytes <= remaining, "figure furniture text byte")?;
            remaining -= bytes;
        }
        let valid_id =
            |s: &str| !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control);
        let mut ids = std::collections::BTreeSet::new();
        for a in &self.annotations {
            if !valid_id(&a.id) || !ids.insert(&a.id) || a.offset.iter().any(|v| !v.is_finite()) {
                return Err(invalid());
            }
        }
        for a in &self.annotations {
            for anchor in std::iter::once(&a.anchor).chain(&a.callout) {
                anchor.validate()?;
            }
        }
        let mut path_remaining = limits.max_path_commands;
        for a in &self.paths {
            if !valid_id(&a.id) || !ids.insert(&a.id) {
                return Err(invalid());
            }
            a.anchor.validate()?;
            a.geometry.validate_for_scene()?;
            crate::limits::require_within(
                a.geometry.commands().len() <= path_remaining,
                "figure path command",
            )?;
            path_remaining -= a.geometry.commands().len();
            if let Some(stroke) = a.stroke {
                crate::geometry::positive(
                    stroke.width,
                    "Path stroke must be finite and positive.",
                )?;
            }
        }
        ids.clear();
        for i in &self.insets {
            if !valid_id(&i.id)
                || !ids.insert(&i.id)
                || i.layers.is_empty()
                || i.layers.len() > 256
                || i.layers
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != i.layers.len()
                || i.rectangle
                    .iter()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
                || i.rectangle[2] <= 0.
                || i.rectangle[3] <= 0.
                || i.rectangle[0] + i.rectangle[2] > 1.
                || i.rectangle[1] + i.rectangle[3] > 1.
            {
                return Err(invalid());
            }
            for view in [i.x_view, i.y_view].into_iter().flatten() {
                view.distinct()?;
            }
        }
        Ok(())
    }
}
/// One fixed path annotation with stable identity and no source-row replication.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VectorAnnotation {
    /// Stable authored identity, unique among all annotations.
    pub id: String,
    /// Position of the local path origin; geometry itself uses destination units.
    pub anchor: Anchor,
    /// Owned full-precision path snapshot.
    pub geometry: crate::path::PathGeometry,
    /// Optional nonzero fill.
    pub fill: Option<crate::color::Paint>,
    /// Optional solid stroke.
    pub stroke: Option<crate::scene::Stroke<crate::color::Paint>>,
    /// Allow the path to extend from the addressed panel to the full figure clip.
    pub overflow: bool,
}
impl Anchor {
    pub(crate) fn validate(&self) -> ChartResult<()> {
        match self {
            Self::Panel { x, y, .. } | Self::Figure { x, y } => {
                if [x, y]
                    .into_iter()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
                {
                    return Err(invalid());
                }
            }
            Self::Output { x, y } => {
                if !x.is_finite() || !y.is_finite() {
                    return Err(invalid());
                }
            }
            Self::Data { x, y, .. } => {
                for value in [x, y] {
                    if matches!(value, ScaleValue::Number(v) if !v.is_finite()) {
                        return Err(invalid());
                    }
                }
            }
        }
        Ok(())
    }
}
impl FigureComposition {
    pub(crate) fn validate_references(
        &self,
        definition: &crate::grammar::ChartDefinition,
    ) -> ChartResult<()> {
        let panel = |key: &Option<PanelKey>| -> ChartResult<()> {
            match (&definition.facets, key) {
                (None, None) => Ok(()),
                (Some(spec), Some(key)) if spec.order.contains(key) => Ok(()),
                _ => Err(Diagnostic::error(
                    DiagnosticCode::Validation,
                    "Data/panel furniture must name an existing facet, or no facet for an ordinary chart.",
                    "Select a panel from the authored facet catalog.",
                )),
            }
        };
        let axis = |id, horizontal| {
            if definition.axes.is_empty() {
                id == crate::ScaleId::new(if horizontal { 0 } else { 1 })
            } else {
                definition
                    .axes
                    .iter()
                    .any(|a| a.id == id && a.side.horizontal() == horizontal)
            }
        };
        for anchor in self
            .annotations
            .iter()
            .flat_map(|a| std::iter::once(&a.anchor).chain(&a.callout))
            .chain(self.paths.iter().map(|a| &a.anchor))
        {
            match anchor {
                Anchor::Data {
                    panel: p, scales, ..
                } => {
                    panel(p)?;
                    if !axis(scales.x, true) || !axis(scales.y, false) {
                        return Err(Diagnostic::error(
                            DiagnosticCode::MissingResource,
                            "Annotation names an absent or incorrectly oriented axis.",
                            "Use existing x/y axis handles.",
                        ));
                    }
                }
                Anchor::Panel { panel: p, .. } => panel(p)?,
                _ => {}
            }
        }
        for letter in &self.panel_letters {
            panel(&letter.panel)?;
        }
        for inset in &self.insets {
            panel(&inset.panel)?;
            for id in &inset.layers {
                if !definition.layers.iter().any(|l| {
                    l.id == *id
                        && match (&l.facet, &inset.panel) {
                            (crate::grammar::FacetTarget::Panels(keys), Some(key)) => {
                                keys.contains(key)
                            }
                            _ => true,
                        }
                }) {
                    return Err(Diagnostic::error(
                        DiagnosticCode::MissingResource,
                        "Inset names a layer absent from its parent panel.",
                        "Use layer handles present in the selected panel.",
                    ));
                }
            }
        }
        Ok(())
    }
}
fn invalid() -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        "Invalid figure furniture identity, offset or inset rectangle.",
        "Use unique bounded IDs, finite offsets and nonempty inset rectangles inside the parent plot.",
    )
}

impl FigureComposition {
    /// Whether retained furniture contains floating paint inputs.
    pub fn has_floating_paint(&self) -> bool {
        self.title
            .iter()
            .chain(&self.subtitle)
            .chain(&self.caption)
            .chain(&self.source_notes)
            .chain(&self.footnotes)
            .chain(self.panel_letters.iter().map(|p| &p.text))
            .chain(self.annotations.iter().map(|a| &a.text))
            .any(RichText::has_floating_paint)
            || self.paths.iter().any(|p| {
                p.fill.is_some_and(crate::color::Paint::is_floating)
                    || p.stroke.is_some_and(|s| s.color.is_floating())
            })
    }
}
