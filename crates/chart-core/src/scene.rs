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
#[derive(serde::Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    /// Unpremultiplied color.
    pub color: Color,
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

/// Authored minimal primitive, validated and copied into an immutable scene.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub enum Primitive {
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

/// Primitive metadata, without assigning fake source targets to decoration.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct SceneItem {
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
        let result = match &item.primitive {
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
        Primitive::Rectangle { .. } | Primitive::GradientRectangle { .. } => Ok(()),
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
pub struct LinearGradient {
    /// Direction across the full rectangle, with padded endpoints.
    pub direction: GradientDirection,
    /// Color at fraction zero.
    pub start: Color,
    /// Color at fraction one.
    pub end: Color,
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
    for command in commands {
        let to = match command {
            PathCommand::MoveTo(p) => {
                previous = Some(*p);
                start = Some(*p);
                index = 0;
                left = dashes[0];
                pen_down = false;
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
