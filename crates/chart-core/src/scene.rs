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
            Primitive::Text { text, .. } => {
                let result = require_within(text.len() <= text_remaining, "total UTF-8 text byte");
                if result.is_ok() {
                    text_remaining -= text.len();
                }
                result
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
        Primitive::Rule { stroke, .. } => {
            positive(stroke.width, "Stroke width must be finite and positive.")
        }
        Primitive::Rectangle { .. } => Ok(()),
        Primitive::Point { center, radius, .. } => {
            positive(*radius, "Point radius must be finite and positive.")?;
            Rect::new(
                center.x() - radius,
                center.y() - radius,
                2.0 * radius,
                2.0 * radius,
            )?;
            Ok(())
        }
        Primitive::Path { commands, .. } | Primitive::FilledPath { commands, .. } => {
            if let Primitive::Path { stroke, .. } = primitive {
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
