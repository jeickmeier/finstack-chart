//! A bounded, immutable destination scene with an intentionally small primitive subset.
//!
//! Submission order is paint order. No source rows, host objects or callbacks are stored.
//! Valid references describe resources; hosts must still resolve/validate their bytes
//! before rendering. Groups, transforms, fills/gradients, shaping and targets follow in
//! later packages; this is not a complete SCN-01 or serializable scene DTO.

use std::collections::BTreeMap;

use crate::geometry::positive;
use crate::limits::require_within;
use crate::services::{ResourceDescriptor, TextRequest, Units, validate_text};
use crate::{
    ChartResult, Diagnostic, DiagnosticCode, LayerId, Limits, Point, Rect, ResourceId, SceneStamp,
};

/// Unpremultiplied sRGB bytes, with linear alpha coverage in the range 0–255.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Color {
    /// Red channel.
    pub red: u8,
    /// Green channel.
    pub green: u8,
    /// Blue channel.
    pub blue: u8,
    /// Alpha, where zero is transparent and 255 is opaque.
    pub alpha: u8,
}

/// A solid stroke; native cap/join/dash choices await the capability spike.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Stroke<P = Color> {
    /// Unpremultiplied color.
    pub color: P,
    /// Positive width in scene units; validated by scene construction.
    pub width: f64,
}

/// Numeric path segments. After `Close`, a subsequent segment requires a new `MoveTo`.
#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq)]
pub enum PathCommand {
    /// Begin a subpath.
    MoveTo(Point),
    /// Straight segment to a point.
    LineTo(Point),
    /// Quadratic Bézier segment, control point followed by endpoint.
    QuadraticTo(Point, Point),
    /// Cubic Bézier segment, two controls followed by endpoint.
    CubicTo(Point, Point, Point),
    /// Close the current subpath.
    Close,
}

/// How uniformly sampled colors fill a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SampledGradientMode {
    /// Color i lies at (i + 0.5) / n, padded to the rectangle edges.
    #[default]
    CellCenters,
    /// Color i lies at i / (n - 1), spanning the rectangle edges.
    Endpoints,
    /// Each color fills one equal-width cell, with hard transitions at cell edges.
    Steps,
}
impl SampledGradientMode {
    /// Shared sRGB stops for a validated multi-color gradient. Steps duplicate
    /// each internal boundary so exporters paint one rectangle without cell seams.
    pub fn stops(self, colors: &[Color]) -> impl Iterator<Item = (f64, Color)> + '_ {
        colors
            .iter()
            .copied()
            .enumerate()
            .flat_map(move |(i, color)| {
                let n = colors.len() as f64;
                let start = match self {
                    Self::CellCenters => (i as f64 + 0.5) / n,
                    Self::Endpoints => i as f64 / (n - 1.),
                    Self::Steps => i as f64 / n,
                };
                std::iter::once((start, color))
                    .chain((self == Self::Steps).then_some(((i + 1) as f64 / n, color)))
            })
    }
    fn is_cell_centers(&self) -> bool {
        *self == Self::CellCenters
    }
}

/// Authored minimal primitive, validated and copied into an immutable scene.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum Primitive {
    /// Generated path with source anchors independent of control/tessellation vertices.
    ShapePath {
        /// Shared checked full-precision path geometry.
        geometry: crate::path::PathGeometry,
        /// Optional nonzero fill, including implicit subpath closure.
        fill: Option<Color>,
        /// Optional explicit stroke.
        stroke: Option<Stroke>,
        /// Optional even positive dash pattern in destination units; phase resets per subpath.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        dashes: Vec<f64>,
        /// One destination source anchor for each corresponding external semantic target.
        anchors: Vec<Point>,
    },
    /// Retained version-two path geometry; empty and move-only paths paint nothing.
    VectorPath {
        /// Immutable full-precision geometry with analytic circular arcs.
        geometry: crate::path::PathGeometry,
        /// Optional nonzero fill; open subpaths are implicitly closed for filling.
        fill: Option<Color>,
        /// Optional stroke, preserving explicit close joins and continuation.
        stroke: Option<Stroke>,
        /// Optional even positive dash pattern in destination units; phase resets per subpath.
        #[serde(skip_serializing_if = "Vec::is_empty")]
        dashes: Vec<f64>,
    },
    /// Explicit native callback invocation. Headless renderers reject this capability.
    NativePaint {
        /// Finite destination rectangle; the host enforces the scene clip.
        bounds: Rect,
        /// Exact host-registered painter identity/version.
        painter: crate::grammar::OperationRef,
        /// Bounded declarative painter data, never code.
        parameters: serde_json::Value,
        /// Resolved style token supplied to the painter.
        fill: Color,
    },
    /// Noncircular point symbol with an explicit semantic center.
    Symbol {
        /// Finite point center.
        center: Point,
        /// Positive half-extent in scene units.
        radius: f64,
        /// Square, diamond or triangle; circles use Primitive::Point.
        kind: crate::theme::Symbol,
        /// Solid fill.
        fill: Color,
    },
    /// Two-stop axis-aligned sRGB gradient with explicit alpha, retained as vector paint.
    GradientRectangle {
        /// Finite rectangle bounds.
        bounds: Rect,
        /// Full rectangle-local linear gradient.
        gradient: LinearGradient,
    },
    /// One sRGB gradient sampled uniformly, with explicit sample mode.
    /// Colors follow increasing x for horizontal or increasing y for vertical paint.
    SampledGradientRectangle {
        /// Finite destination rectangle.
        bounds: Rect,
        /// Direction in rectangle-local coordinates.
        direction: GradientDirection,
        /// At least two colors.
        colors: Vec<Color>,
        /// Defaults to cell centers for existing scene wire version 17.
        #[serde(default, skip_serializing_if = "SampledGradientMode::is_cell_centers")]
        mode: SampledGradientMode,
    },
    /// Dashed straight path; phase resets for each explicitly opened subpath.
    DashedPath {
        /// Numeric MoveTo/LineTo/Close commands.
        commands: Vec<PathCommand>,
        /// Solid stroke width and color.
        stroke: Stroke,
        /// Positive alternating on/off lengths, with an even count no larger than 16.
        dashes: Vec<f64>,
    },
    /// Explicitly shaped rich text. Outlines and PDF glyphs share the same logical clusters.
    GlyphRun {
        /// Destination baseline origin.
        origin: Point,
        /// Clockwise degrees about the origin.
        rotation: f64,
        /// Logical text, exact resource, advances and numeric glyph outlines.
        run: crate::typography::ShapedRun,
        /// Homogeneous run color.
        color: Color,
    },
    /// Straight rule between finite endpoints.
    Rule {
        /// Start point.
        from: Point,
        /// End point.
        to: Point,
        /// Solid stroke.
        stroke: Stroke,
    },
    /// Filled axis-aligned rectangle.
    Rectangle {
        /// Finite bounds.
        bounds: Rect,
        /// Solid fill.
        fill: Color,
    },
    /// Filled circular point symbol.
    Point {
        /// Finite center.
        center: Point,
        /// Positive radius in scene units.
        radius: f64,
        /// Solid fill.
        fill: Color,
    },
    /// Stroked numeric path, without SVG parsing or implied filling.
    Path {
        /// At least one drawing segment and valid subpath ordering.
        commands: Vec<PathCommand>,
        /// Solid stroke.
        stroke: Stroke,
    },
    /// Closed path filled using the nonzero winding rule.
    FilledPath {
        /// Explicit closed subpaths with at least one drawing segment.
        commands: Vec<PathCommand>,
        /// Solid interior color.
        fill: Color,
    },
    /// One plain text run; measurement/painting must use this explicit font descriptor.
    Text {
        /// Baseline origin in scene units.
        origin: Point,
        /// Preserved logical UTF-8 text.
        text: String,
        /// Font descriptor declared in this scene's resources.
        font: ResourceId,
        /// Positive size in scene units.
        font_size: f64,
        /// Text color.
        color: Color,
    },
}

mod guide;
pub use guide::{GuideAnimation, GuideComponent, GuideRole, GuideTickIdentity};

/// Primitive metadata, without assigning fake source targets to decoration.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct SceneItem {
    /// Optional portable guide roles; decorative primitives have no data targets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guide: Option<GuideComponent>,
    /// Optional originating layer for diagnostic context.
    pub layer: Option<LayerId>,
    /// Explicit clip override; `None` means the scene bounds, not unbounded overflow.
    pub clip: Option<Rect>,
    /// Primitive submitted at this position in paint order.
    pub primitive: Primitive,
}

/// Validated, owned scene snapshot; failed construction cannot modify an existing scene.
#[derive(Clone, Debug)]
pub struct Scene {
    stamp: SceneStamp,
    units: Units,
    bounds: Rect,
    items: Vec<SceneItem>,
    resources: Vec<ResourceDescriptor>,
}

impl Scene {
    /// Check counts, byte budgets, structure and references before copying input payloads.
    /// The caller owns input allocation; raising limits explicitly increases permitted work.
    pub fn new(
        stamp: SceneStamp,
        units: Units,
        bounds: Rect,
        items: &[SceneItem],
        resources: &[ResourceDescriptor],
        limits: Limits,
    ) -> ChartResult<Self> {
        validate(items, resources, units, limits).map_err(|mut error| {
            error.context.stamp = Some(stamp);
            error
        })?;
        Ok(Self {
            stamp,
            units,
            bounds,
            items: items.to_vec(),
            resources: resources.to_vec(),
        })
    }

    /// Fail before encoding if this scene needs any native-only painter; no implicit raster fallback.
    pub fn require_portable_paint(&self) -> ChartResult<()> {
        for item in &self.items {
            if let Primitive::NativePaint { painter, .. } = &item.primitive {
                let mut d = Diagnostic::error(
                    DiagnosticCode::UnsupportedCapability,
                    format!(
                        "Native painter {} version {} has no SVG/PDF/PNG representation.",
                        painter.id,
                        painter.version.get()
                    ),
                    "Replace this layer with portable numeric geometry before export.",
                );
                d.context.layer = item.layer;
                return Err(d);
            }
        }
        Ok(())
    }
    /// Input revision stamps retained exactly.
    pub const fn stamp(&self) -> SceneStamp {
        self.stamp
    }
    /// Destination units used by every coordinate, stroke and font size.
    pub const fn units(&self) -> Units {
        self.units
    }
    /// Destination bounds and default clip.
    pub const fn bounds(&self) -> Rect {
        self.bounds
    }
    /// Validated primitives in paint order; there is no mutable access to the snapshot.
    pub fn items(&self) -> &[SceneItem] {
        &self.items
    }
    /// Minimum scene wire version required by the retained primitive capabilities.
    pub fn wire_version(&self) -> u32 {
        if self.items.iter().any(|i| {
            i.guide.as_ref().is_some_and(|g| {
                matches!(
                    g.role,
                    GuideRole::LegendTitle
                        | GuideRole::LegendKey
                        | GuideRole::LegendLabel
                        | GuideRole::LegendBar
                        | GuideRole::LegendTick
                )
            })
        }) {
            return 19;
        }
        if self.items.iter().any(|item| {
            matches!(
                item.primitive,
                Primitive::SampledGradientRectangle {
                    mode: SampledGradientMode::Endpoints | SampledGradientMode::Steps,
                    ..
                }
            )
        }) {
            return 18;
        }
        if self
            .items
            .iter()
            .any(|item| matches!(item.primitive, Primitive::SampledGradientRectangle { .. }))
        {
            return 17;
        }
        if self.items.iter().any(|i| {
            i.guide.as_ref().and_then(|g| g.tick.as_ref()).is_some_and(
                |t| matches!(t.value, crate::composition::ScaleValue::Number(n) if !n.is_finite()),
            )
        }) {
            return 16;
        }
        if self
            .items
            .iter()
            .any(|i| i.guide.as_ref().is_some_and(|g| g.animation.is_some()))
        {
            return 15;
        }
        if self.items.iter().any(|i| i.guide.is_some()) {
            return 14;
        }
        if self.items.iter().any(|i| matches!(&i.primitive, Primitive::ShapePath { dashes, .. } | Primitive::VectorPath { dashes, .. } if !dashes.is_empty())) {
            return 4;
        }
        if self
            .items
            .iter()
            .any(|i| matches!(i.primitive, Primitive::ShapePath { .. }))
        {
            return 3;
        }
        if self
            .items
            .iter()
            .any(|i| matches!(i.primitive, Primitive::VectorPath { .. }))
        {
            2
        } else {
            1
        }
    }
    /// Declared immutable resource identities, still requiring host byte resolution.
    pub fn resources(&self) -> &[ResourceDescriptor] {
        &self.resources
    }
}

fn validate(
    items: &[SceneItem],
    resources: &[ResourceDescriptor],
    units: Units,
    limits: Limits,
) -> ChartResult<()> {
    require_within(items.len() <= limits.max_items, "scene item")?;
    require_within(resources.len() <= limits.max_resources, "resource count")?;
    // Preflight variable-sized payloads before building maps, scanning paths or cloning.
    let mut text_remaining = limits.max_text_bytes;
    let mut path_remaining = limits.max_path_commands;
    for item in items {
        if let Some(guide) = &item.guide {
            guide.validate(&mut text_remaining)?;
        }
        let result = match &item.primitive {
            Primitive::NativePaint {
                painter,
                parameters,
                ..
            } => {
                crate::grammar::validate_native_paint(painter, parameters)?;
                let bytes = crate::grammar::native_parameter_size(parameters)?
                    .saturating_add(painter.id.len());
                require_within(bytes <= text_remaining, "total native descriptor byte")?;
                text_remaining -= bytes;
                Ok(())
            }

            Primitive::SampledGradientRectangle { colors, mode, .. } => {
                let work = colors
                    .len()
                    .saturating_mul(if *mode == SampledGradientMode::Steps {
                        2
                    } else {
                        1
                    });
                require_within(work <= path_remaining, "total gradient sample")?;
                path_remaining -= work;
                if colors.len() < 2 {
                    return Err(crate::scales::error(
                        DiagnosticCode::Validation,
                        "A sampled gradient requires at least two colors.",
                    ));
                }
                Ok(())
            }
            Primitive::GlyphRun { run, .. } => {
                require_within(
                    run.text.len().saturating_add(run.language.len()) <= text_remaining,
                    "total UTF-8 text byte",
                )?;
                require_within(
                    run.outlines.len().saturating_add(run.glyphs.len()) <= path_remaining,
                    "total rich outline commands",
                )?;
                text_remaining -= run.text.len() + run.language.len();
                path_remaining -= run.outlines.len() + run.glyphs.len();
                Ok(())
            }
            Primitive::Text { text, .. } => {
                let result = require_within(text.len() <= text_remaining, "total UTF-8 text byte");
                if result.is_ok() {
                    text_remaining -= text.len();
                }
                result
            }
            Primitive::DashedPath {
                commands, dashes, ..
            } => {
                require_within(commands.len() <= path_remaining, "total path command")?;
                path_remaining -= commands.len();
                let lowered = dash_polyline(commands, dashes, path_remaining)?;
                path_remaining -= lowered.len();
                Ok(())
            }
            Primitive::ShapePath {
                geometry,
                anchors,
                dashes,
                ..
            } => {
                let count = geometry.commands().len().saturating_add(anchors.len());
                require_within(
                    count <= path_remaining,
                    "total path commands and source anchors",
                )?;
                path_remaining -= count;
                if !dashes.is_empty() {
                    path_remaining -= geometry.dashed(dashes, 0.25, path_remaining)?.len();
                }
                Ok(())
            }
            Primitive::VectorPath {
                geometry, dashes, ..
            } => {
                require_within(
                    geometry.commands().len() <= path_remaining,
                    "total path command",
                )?;
                path_remaining -= geometry.commands().len();
                if !dashes.is_empty() {
                    path_remaining -= geometry.dashed(dashes, 0.25, path_remaining)?.len();
                }
                Ok(())
            }
            Primitive::Path { commands, .. } | Primitive::FilledPath { commands, .. } => {
                let result = require_within(commands.len() <= path_remaining, "total path command");
                if result.is_ok() {
                    path_remaining -= commands.len();
                }
                result
            }
            _ => Ok(()),
        };
        result.map_err(|mut error| {
            error.context.layer = item.layer;
            error
        })?;
    }
    let mut bytes_remaining = limits.max_total_resource_bytes;
    for resource in resources {
        resource.validate(limits)?;
        require_within(resource.byte_len <= bytes_remaining, "total resource byte").map_err(
            |mut error| {
                error.context.resource = Some(resource.id);
                error
            },
        )?;
        bytes_remaining -= resource.byte_len;
    }
    let mut by_id = BTreeMap::new();
    for resource in resources {
        if by_id.insert(resource.id, resource).is_some() {
            let mut error = Diagnostic::error(
                DiagnosticCode::SchemaConflict,
                "A scene declares the same resource identity more than once.",
                "Declare one revision per resource identity in the coherent scene.",
            );
            error.context.resource = Some(resource.id);
            return Err(error);
        }
    }
    for item in items {
        validate_primitive(&item.primitive, &by_id, units, limits).map_err(|mut error| {
            error.context.layer = item.layer;
            error
        })?;
    }
    Ok(())
}

fn validate_primitive(
    primitive: &Primitive,
    resources: &BTreeMap<ResourceId, &ResourceDescriptor>,
    units: Units,
    limits: Limits,
) -> ChartResult<()> {
    match primitive {
        Primitive::VectorPath {
            geometry, stroke, ..
        }
        | Primitive::ShapePath {
            geometry, stroke, ..
        } => {
            if let Some(stroke) = stroke {
                positive(stroke.width, "Stroke width must be finite and positive.")?;
            }
            geometry.validate_for_scene()
        }
        Primitive::NativePaint {
            painter,
            parameters,
            ..
        } => crate::grammar::validate_native_paint(painter, parameters),
        Primitive::GlyphRun { rotation, run, .. } => {
            if !rotation.is_finite() || rotation.abs() > 360. {
                return Err(path_error());
            }
            if resources.get(&run.font.id).is_none_or(|r| **r != run.font) {
                let mut e = path_error();
                e.code = DiagnosticCode::MissingResource;
                e.context.resource = Some(run.font.id);
                return Err(e);
            }
            validate_text(
                TextRequest {
                    text: &run.text,
                    font: &run.font,
                    font_size: run.font_size,
                    units,
                },
                limits,
            )?;
            require_within(
                run.glyphs.len() <= limits.max_path_commands,
                "rich glyph count",
            )?;
            if run.glyphs.iter().any(|g| {
                g.id == 0
                    || g.start > g.end
                    || !run.text.is_char_boundary(g.start)
                    || !run.text.is_char_boundary(g.end)
            }) {
                return Err(path_error());
            }
            if !run.outlines.is_empty() {
                validate_primitive(
                    &Primitive::FilledPath {
                        commands: run.outlines.clone(),
                        fill: Color {
                            red: 0,
                            green: 0,
                            blue: 0,
                            alpha: 255,
                        },
                    },
                    resources,
                    units,
                    limits,
                )?;
            }
            Ok(())
        }
        Primitive::Rule { stroke, .. } => {
            positive(stroke.width, "Stroke width must be finite and positive.")
        }
        Primitive::Rectangle { .. }
        | Primitive::GradientRectangle { .. }
        | Primitive::SampledGradientRectangle { .. } => Ok(()),
        Primitive::Point { center, radius, .. } | Primitive::Symbol { center, radius, .. } => {
            if matches!(
                primitive,
                Primitive::Symbol {
                    kind: crate::theme::Symbol::Circle,
                    ..
                }
            ) {
                return Err(path_error());
            }
            positive(*radius, "Point radius must be finite and positive.")?;
            Rect::new(
                center.x() - radius,
                center.y() - radius,
                2.0 * radius,
                2.0 * radius,
            )?;
            Ok(())
        }
        Primitive::Path { commands, .. }
        | Primitive::FilledPath { commands, .. }
        | Primitive::DashedPath { commands, .. } => {
            if let Primitive::Path { stroke, .. } | Primitive::DashedPath { stroke, .. } = primitive
            {
                positive(stroke.width, "Stroke width must be finite and positive.")?;
            }
            let mut open = false;
            let mut drawn = false;
            for command in commands {
                match command {
                    PathCommand::MoveTo(_) => {
                        if open && matches!(primitive, Primitive::FilledPath { .. }) {
                            return Err(path_error());
                        }
                        open = true;
                    }
                    PathCommand::Close if open => open = false,
                    PathCommand::LineTo(_)
                    | PathCommand::QuadraticTo(_, _)
                    | PathCommand::CubicTo(_, _, _)
                        if open =>
                    {
                        drawn = true
                    }
                    _ => return Err(path_error()),
                }
            }
            if !drawn || (matches!(primitive, Primitive::FilledPath { .. }) && open) {
                return Err(path_error());
            }
            Ok(())
        }
        Primitive::Text {
            text,
            font,
            font_size,
            ..
        } => {
            let resource = resources.get(font).ok_or_else(|| {
                let mut error = Diagnostic::error(DiagnosticCode::MissingResource,
                    "The text font is absent from the scene resource descriptors.",
                    "Declare the referenced font and its immutable revision before constructing the scene.");
                error.context.resource = Some(*font);
                error
            })?;
            validate_text(
                TextRequest {
                    text,
                    font: resource,
                    font_size: *font_size,
                    units,
                },
                limits,
            )
        }
    }
}

fn path_error() -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        "A path must contain drawing segments within explicitly opened subpaths.",
        "Start with MoveTo, include a line or curve, and use a new MoveTo after Close.",
    )
}

/// Direction of a two-stop sRGB gradient in rectangle-local coordinates.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientDirection {
    /// Left to right.
    Horizontal,
    /// Top to bottom.
    Vertical,
}
/// Two explicit sRGB stops; interpolation and alpha are shared by native/SVG/PDF.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LinearGradient<P = Color> {
    /// Direction across the full rectangle, with padded endpoints.
    pub direction: GradientDirection,
    /// Color at fraction zero.
    pub start: P,
    /// Color at fraction one.
    pub end: P,
}
/// Lower an explicitly dashed polyline to independent numeric segments for native painting.
/// Phase resets at MoveTo. Curves require a solid path; budgets stop pathological tiny dashes.
pub fn dash_polyline(
    commands: &[PathCommand],
    dashes: &[f64],
    limit: usize,
) -> ChartResult<Vec<PathCommand>> {
    if dashes.is_empty()
        || dashes.len() > 16
        || !dashes.len().is_multiple_of(2)
        || dashes.iter().any(|v| !v.is_finite() || *v <= 0.)
    {
        return Err(path_error());
    }
    let mut out = vec![];
    let mut previous = None;
    let mut start = None;
    let mut index = 0;
    let mut left = dashes[0];
    let mut pen_down = false;
    let mut work = 0;
    let mut subpath_output = 0;
    for command in commands {
        let to = match command {
            PathCommand::MoveTo(p) => {
                previous = Some(*p);
                start = Some(*p);
                index = 0;
                left = dashes[0];
                pen_down = false;
                subpath_output = out.len();
                continue;
            }
            PathCommand::LineTo(p) => *p,
            PathCommand::Close => start.ok_or_else(path_error)?,
            _ => {
                return Err(Diagnostic::error(
                    DiagnosticCode::UnsupportedCapability,
                    "Dashed paths accept straight segments only.",
                    "Use a solid curve or explicitly flatten it to a bounded polyline.",
                ));
            }
        };
        let from = previous.ok_or_else(path_error)?;
        let dx = to.x() - from.x();
        let dy = to.y() - from.y();
        let length = dx.hypot(dy);
        if !length.is_finite() {
            return Err(path_error());
        }
        let mut at = 0.;
        while at < length {
            work += 1;
            require_within(work <= limit, "dash subdivision work")?;
            let available = length - at;
            let finishes_dash = left <= available;
            let end = if finishes_dash { at + left } else { length };
            if end <= at {
                return Err(Diagnostic::error(
                    DiagnosticCode::PrecisionLoss,
                    "Dash size cannot advance along this path at destination precision.",
                    "Use a representable dash length at this output size.",
                ));
            }
            if index % 2 == 0 {
                require_within(
                    out.len().saturating_add(2) <= limit,
                    "dashed numeric command",
                )?;
                let point = |distance: f64| {
                    Point::new(
                        from.x() + dx * (distance / length),
                        from.y() + dy * (distance / length),
                    )
                };
                if !pen_down {
                    out.push(PathCommand::MoveTo(point(at)?));
                }
                out.push(PathCommand::LineTo(point(end)?));
                pen_down = true;
            } else {
                pen_down = false;
            }
            at = end;
            if finishes_dash {
                index = (index + 1) % dashes.len();
                left = dashes[index];
            } else {
                left -= available;
            }
        }
        previous = if matches!(command, PathCommand::Close) {
            // A dash crossing the closing seam is one joined stroke, not two butt caps.
            if pen_down && out.len() > subpath_output {
                let last_start = out[subpath_output..]
                    .iter()
                    .rposition(|c| matches!(c, PathCommand::MoveTo(_)))
                    .expect("painted dash has a move")
                    + subpath_output;
                if last_start == subpath_output {
                    if let Some(last) = out.last_mut() {
                        *last = PathCommand::Close;
                    }
                } else {
                    let tail = out.len() - last_start;
                    out[subpath_output..].rotate_right(tail);
                    out.remove(subpath_output + tail);
                }
            }
            None
        } else {
            Some(to)
        };
    }
    Ok(out)
}

/// Exact closed numeric polygon for square, diamond or triangle point symbols.
pub fn symbol_path(
    center: Point,
    r: f64,
    s: crate::theme::Symbol,
) -> ChartResult<Vec<PathCommand>> {
    use crate::theme::Symbol;
    positive(r, "Symbol radius must be finite and positive.")?;
    let offsets: &[(f64, f64)] = match s {
        Symbol::Square => &[(-1., -1.), (1., -1.), (1., 1.), (-1., 1.)],
        Symbol::Diamond => &[(0., -1.), (1., 0.), (0., 1.), (-1., 0.)],
        Symbol::Triangle => &[(0., -1.), (1., 1.), (-1., 1.)],
        Symbol::Circle => return Err(path_error()),
    };
    let mut p = vec![];
    for (i, (x, y)) in offsets.iter().enumerate() {
        let point = Point::new(center.x() + x * r, center.y() + y * r)?;
        p.push(if i == 0 {
            PathCommand::MoveTo(point)
        } else {
            PathCommand::LineTo(point)
        });
    }
    p.push(PathCommand::Close);
    Ok(p)
}

impl<P> LinearGradient<P> {
    /// Transform endpoints without altering the gradient direction.
    pub fn map_colors<Q>(self, mut map: impl FnMut(P) -> Q) -> LinearGradient<Q> {
        LinearGradient {
            direction: self.direction,
            start: map(self.start),
            end: map(self.end),
        }
    }
}

impl<P> Stroke<P> {
    /// Transform a stroke's color without changing its width.
    pub fn map_color<Q>(self, map: impl FnOnce(P) -> Q) -> Stroke<Q> {
        Stroke {
            color: map(self.color),
            width: self.width,
        }
    }
}
