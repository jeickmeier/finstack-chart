//! Checked standalone path authoring with d3-path 3.1.0 finite-input semantics.
//!
//! Numeric commands retain full precision; SVG digits never change geometry or state.
//! Authoring accepts serializer-only sequences. A destination must separately validate
//! its initial move and paintable geometry. See ADR-015 for compatibility boundaries.

mod dash;
mod flatten;
pub use flatten::{FlatSubpath, FlattenedPath};
mod geometry;
mod lower;
mod svg;
pub use lower::Affine;

use crate::{ChartResult, Diagnostic, DiagnosticCode};

/// Output precision, independent of all path geometry and authoring state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Precision {
    /// Shortest unrounded finite numbers.
    #[default]
    Unrounded,
    /// At most this many fractional digits, validated to be at most 15.
    Digits(u8),
}
impl Precision {
    /// Interpret finite, nonnegative digits using floor and the >15 unrounded rule.
    pub fn from_digits(digits: f64) -> ChartResult<Self> {
        if !digits.is_finite() || digits < 0.0 {
            return Err(domain("Path digits must be finite and nonnegative"));
        }
        Ok(if digits.floor() > 15.0 {
            Self::Unrounded
        } else {
            Self::Digits(digits.floor() as u8)
        })
    }
}

/// Explicit per-builder and per-result resource bounds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathLimits {
    /// Maximum successful authoring operations, including no-op operations.
    pub max_operations: usize,
    /// Maximum retained numeric commands.
    pub max_commands: usize,
    /// Maximum UTF-8 bytes of one SVG result.
    pub max_svg_bytes: usize,
    /// Maximum commands delivered to an external sink in one replay.
    pub max_replay_commands: usize,
    /// Maximum destination segments emitted when lowering retained curves.
    pub max_subdivisions: usize,
}
impl Default for PathLimits {
    fn default() -> Self {
        Self {
            max_operations: 1_000_000,
            max_commands: 1_000_000,
            max_svg_bytes: 64 * 1024 * 1024,
            max_replay_commands: 1_000_000,
            max_subdivisions: 1_000_000,
        }
    }
}

/// One finite authoring operation. Validation happens before any mutation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "op",
    content = "args",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum PathOp {
    /// Set the current and subpath-start point.
    MoveTo([f64; 2]),
    /// Append a line and advance the current point.
    LineTo([f64; 2]),
    /// Control point and endpoint of a quadratic Bézier.
    QuadraticCurveTo([f64; 4]),
    /// Two controls and endpoint of a cubic Bézier.
    BezierCurveTo([f64; 6]),
    /// Corner, following point and nonnegative tangent radius.
    ArcTo([f64; 5]),
    /// Center/radius/angles circular arc.
    Arc {
        /// Center x.
        x: f64,
        /// Center y.
        y: f64,
        /// Nonnegative radius.
        r: f64,
        /// Initial angle in radians.
        a0: f64,
        /// Final angle in radians.
        a1: f64,
        /// Reverse the default clockwise y-down sweep.
        #[serde(default)]
        anticlockwise: bool,
    },
    /// Origin, signed width and signed height of a closed subpath.
    Rect([f64; 4]),
    /// Close according to the retained authoring state; empty is a no-op.
    ClosePath,
}

/// A full-precision retained path command; no source-row identity is implied.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Command {
    /// Absolute subpath start.
    MoveTo([f64; 2]),
    /// Absolute line endpoint.
    LineTo([f64; 2]),
    /// Quadratic control and endpoint.
    QuadraticTo([f64; 4]),
    /// Cubic controls and endpoint.
    CubicTo([f64; 6]),
    /// Circular SVG endpoint arc, retained analytically until destination lowering.
    Arc {
        /// Nonnegative radius in both dimensions.
        radius: f64,
        /// Choose the greater-than-half-circle arc.
        large: bool,
        /// Clockwise sweep in y-down coordinates.
        clockwise: bool,
        /// Absolute endpoint.
        to: [f64; 2],
    },
    /// Signed relative horizontal edge, used by rectangle serialization.
    Horizontal(f64),
    /// Signed relative vertical edge, used by rectangle serialization.
    Vertical(f64),
    /// Close to the geometric subpath start, including repeated closes.
    Close,
}

/// An external numeric consumer. A rejected command may leave its accepted prefix.
pub trait PathSink {
    /// Accept one checked command without manufacturing source targets.
    fn command(&mut self, command: &Command) -> ChartResult<()>;
}

/// Owned immutable numeric geometry, independent of the originating builder.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "GeometryWire")]
pub struct PathGeometry {
    commands: Vec<Command>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct GeometryWire {
    commands: Vec<Command>,
}
impl TryFrom<GeometryWire> for PathGeometry {
    type Error = Diagnostic;
    fn try_from(wire: GeometryWire) -> ChartResult<Self> {
        Self::from_commands(wire.commands, PathLimits::default().max_commands)
    }
}
impl PathGeometry {
    /// Own checked numeric commands, including serializer-only sequences.
    /// Scene submission performs the additional initial-move validation.
    pub fn from_commands(commands: Vec<Command>, max_commands: usize) -> ChartResult<Self> {
        require(commands.len() <= max_commands, "Path command limit")?;
        for command in &commands {
            match command {
                Command::MoveTo(p) | Command::LineTo(p) => finite(p)?,
                Command::QuadraticTo(p) => finite(p)?,
                Command::CubicTo(p) => finite(p)?,
                Command::Horizontal(v) | Command::Vertical(v) => finite(&[*v])?,
                Command::Arc { radius, to, .. } => {
                    finite(&[*radius, to[0], to[1]])?;
                    if *radius <= 0. {
                        return Err(domain("Retained arc radius must be positive"));
                    }
                }
                Command::Close => (),
            }
        }
        Ok(Self { commands })
    }
    /// Check geometric submission separately from serializer-only authoring state.
    /// Repeated closes and post-close continuation retain the current geometric point.
    pub fn validate_for_scene(&self) -> ChartResult<()> {
        let mut current: Option<[f64; 2]> = None;
        let mut start: Option<[f64; 2]> = None;
        for command in &self.commands {
            if !matches!(command, Command::MoveTo(_)) && current.is_none() {
                return Err(Diagnostic::error(
                    DiagnosticCode::Validation,
                    "A submitted path must start with moveTo",
                    "Add an explicit move before the initial segment; standalone serialization preserves bare segments",
                ));
            }
            current = Some(match *command {
                Command::MoveTo(p) => {
                    start = Some(p);
                    p
                }
                Command::LineTo(p) => p,
                Command::QuadraticTo(p) => [p[2], p[3]],
                Command::CubicTo(p) => [p[4], p[5]],
                Command::Arc { to, .. } => to,
                Command::Horizontal(dx) => {
                    let p = current.expect("checked geometric start");
                    [p[0] + dx, p[1]]
                }
                Command::Vertical(dy) => {
                    let p = current.expect("checked geometric start");
                    [p[0], p[1] + dy]
                }
                Command::Close => start.expect("checked geometric subpath"),
            });
            finite(&current.expect("assigned geometric point"))?;
        }
        Ok(())
    }
    /// Borrow full-precision commands in operation order.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
    /// Replay with an explicit work bound. A sink failure stops at that command.
    pub fn replay(&self, sink: &mut impl PathSink, max_commands: usize) -> ChartResult<()> {
        require(
            self.commands.len() <= max_commands,
            "Path replay command limit",
        )?;
        for command in &self.commands {
            sink.command(command)?;
        }
        Ok(())
    }
    /// Serialize without mutating the numeric geometry.
    pub fn to_svg(&self, precision: Precision, max_bytes: usize) -> ChartResult<String> {
        svg::serialize(&self.commands, precision, max_bytes)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct State {
    start: Option<[f64; 2]>,
    current: Option<[f64; 2]>,
}

/// Independent bounded mutable builder; rejected operations leave all state unchanged.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    geometry: PathGeometry,
    state: State,
    precision: Precision,
    limits: PathLimits,
    operations: usize,
}
impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}
/// Construct an independent unrounded path with default limits.
pub fn path() -> Path {
    Path::new()
}
/// Construct an independent path with the reference's default three output digits.
pub fn path_round() -> Path {
    Path {
        precision: Precision::Digits(3),
        ..Path::new()
    }
}
impl Path {
    /// End a generator-specific command cap while preserving the caller's owned path budget.
    pub(crate) fn restore_command_limit(mut self, limit: usize) -> Self {
        debug_assert!(limit >= self.limits.max_commands);
        self.limits.max_commands = limit;
        self
    }
    /// Decode a standalone version-one operation request and build atomically.
    pub fn from_json(input: &str) -> ChartResult<Self> {
        let request: PathRequest = crate::portable::decode(input)?;
        request.build()
    }
    /// Owned standalone result, containing numeric geometry, state and SVG output.
    pub fn result_json(&self) -> ChartResult<String> {
        crate::portable::encode(&serde_json::json!({
            "version": 1, "geometry": self.geometry, "current": self.current_point(),
            "start": self.start_point(), "operations": self.operations, "svg": self.to_svg()?,
        }))
    }
    /// Owned command copy for a host sink, enforcing replay work independently of SVG bytes.
    pub fn replay_json(&self) -> ChartResult<String> {
        require(
            self.geometry.commands.len() <= self.limits.max_replay_commands,
            "Path replay command limit",
        )?;
        crate::portable::encode(&self.geometry.commands)
    }
    /// Apply a checked named finite operation from a typed host adapter.
    pub fn draw(&mut self, method: &str, values: &[f64], anticlockwise: bool) -> ChartResult<()> {
        let op = match (method, values) {
            ("moveTo", &[x, y]) => PathOp::MoveTo([x, y]),
            ("lineTo", &[x, y]) => PathOp::LineTo([x, y]),
            ("quadraticCurveTo", &[a, b, x, y]) => PathOp::QuadraticCurveTo([a, b, x, y]),
            ("bezierCurveTo", &[a, b, c, d, x, y]) => PathOp::BezierCurveTo([a, b, c, d, x, y]),
            ("arcTo", &[a, b, c, d, r]) => PathOp::ArcTo([a, b, c, d, r]),
            ("arc", &[x, y, r, a0, a1]) => PathOp::Arc {
                x,
                y,
                r,
                a0,
                a1,
                anticlockwise,
            },
            ("rect", &[x, y, w, h]) => PathOp::Rect([x, y, w, h]),
            ("closePath", &[]) => PathOp::ClosePath,
            _ => {
                return Err(Diagnostic::error(
                    DiagnosticCode::Validation,
                    "Unknown path method or incorrect argument count",
                    "Use one of the eight path methods with its declared numeric arguments",
                ));
            }
        };
        self.apply(&op)
    }
    /// Construct an independent unrounded path with default limits.
    pub fn new() -> Self {
        Self {
            geometry: PathGeometry { commands: vec![] },
            state: State::default(),
            precision: Precision::Unrounded,
            limits: PathLimits::default(),
            operations: 0,
        }
    }
    /// Construct with optional explicit digits; `None` selects unrounded output.
    pub fn with_digits(digits: Option<f64>) -> ChartResult<Self> {
        Self::with_options(
            digits
                .map(Precision::from_digits)
                .transpose()?
                .unwrap_or_default(),
            PathLimits::default(),
        )
    }
    /// Construct with validated precision and explicit resource bounds.
    pub fn with_options(precision: Precision, limits: PathLimits) -> ChartResult<Self> {
        if matches!(precision, Precision::Digits(16..)) {
            return Err(domain("Path precision exceeds 15 digits"));
        }
        Ok(Self {
            precision,
            limits,
            ..Self::new()
        })
    }
    /// Reference authoring current point; may differ from the geometric SVG point.
    pub fn current_point(&self) -> Option<[f64; 2]> {
        self.state.current
    }
    /// Reference remembered subpath start, including implicit-start behavior.
    pub fn start_point(&self) -> Option<[f64; 2]> {
        self.state.start
    }
    /// Successful operations, including no-ops; rejected operations are not counted.
    pub fn operation_count(&self) -> usize {
        self.operations
    }
    /// Own a numeric snapshot unaffected by subsequent builder changes.
    pub fn geometry(&self) -> PathGeometry {
        self.geometry.clone()
    }
    /// Non-destructive standalone SVG output under this builder's precision policy.
    pub fn to_svg(&self) -> ChartResult<String> {
        self.geometry
            .to_svg(self.precision, self.limits.max_svg_bytes)
    }
    /// Replay using this builder's explicit replay bound.
    pub fn replay(&self, sink: &mut impl PathSink) -> ChartResult<()> {
        self.geometry.replay(sink, self.limits.max_replay_commands)
    }
    /// Lower a numeric snapshot using this builder's destination work budget.
    pub fn lower(&self, max_error: f64) -> ChartResult<Vec<crate::scene::PathCommand>> {
        self.geometry.lower(max_error, self.limits.max_subdivisions)
    }
    /// Apply one checked operation atomically.
    pub fn apply(&mut self, operation: &PathOp) -> ChartResult<()> {
        require(
            self.operations < self.limits.max_operations,
            "Path operation limit",
        )?;
        let (state, commands) = geometry::evaluate(self.state, operation)?;
        require(
            commands.len()
                <= self
                    .limits
                    .max_commands
                    .saturating_sub(self.geometry.commands.len()),
            "Path command limit",
        )?;
        self.geometry.commands.extend(commands);
        self.state = state;
        self.operations += 1;
        Ok(())
    }
    /// Apply a batch atomically, retaining the old builder if any operation fails.
    pub fn apply_batch(&mut self, operations: &[PathOp]) -> ChartResult<()> {
        require(
            operations.len() <= self.limits.max_operations.saturating_sub(self.operations),
            "Path operation limit",
        )?;
        let mut candidate = self.clone();
        for operation in operations {
            candidate.apply(operation)?;
        }
        *self = candidate;
        Ok(())
    }
    /// Start a subpath.
    pub fn move_to(&mut self, x: f64, y: f64) -> ChartResult<()> {
        self.apply(&PathOp::MoveTo([x, y]))
    }
    /// Append a line.
    pub fn line_to(&mut self, x: f64, y: f64) -> ChartResult<()> {
        self.apply(&PathOp::LineTo([x, y]))
    }
    /// Append a quadratic Bézier.
    pub fn quadratic_curve_to(&mut self, cx: f64, cy: f64, x: f64, y: f64) -> ChartResult<()> {
        self.apply(&PathOp::QuadraticCurveTo([cx, cy, x, y]))
    }
    /// Append a cubic Bézier.
    pub fn bezier_curve_to(
        &mut self,
        cx1: f64,
        cy1: f64,
        cx2: f64,
        cy2: f64,
        x: f64,
        y: f64,
    ) -> ChartResult<()> {
        self.apply(&PathOp::BezierCurveTo([cx1, cy1, cx2, cy2, x, y]))
    }
    /// Append a tangent circular arc.
    pub fn arc_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, r: f64) -> ChartResult<()> {
        self.apply(&PathOp::ArcTo([x1, y1, x2, y2, r]))
    }
    /// Append a center/radius/angle arc with an explicit sweep direction.
    pub fn arc(
        &mut self,
        center: [f64; 2],
        r: f64,
        a0: f64,
        a1: f64,
        anticlockwise: bool,
    ) -> ChartResult<()> {
        self.apply(&PathOp::Arc {
            x: center[0],
            y: center[1],
            r,
            a0,
            a1,
            anticlockwise,
        })
    }
    /// Append a rectangle with signed or zero extents.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64) -> ChartResult<()> {
        self.apply(&PathOp::Rect([x, y, width, height]))
    }
    /// Close according to the reference's remembered current/start state.
    pub fn close_path(&mut self) -> ChartResult<()> {
        self.apply(&PathOp::ClosePath)
    }
}

/// Standalone operation interchange; independent of chart, data and scene versions.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathRequest {
    /// Supported version is one.
    pub version: u32,
    /// Omitted/null selects unrounded output; finite nonnegative values are floored.
    #[serde(default)]
    pub digits: Option<f64>,
    /// Explicit work and output budgets.
    #[serde(default)]
    pub limits: PathLimits,
    /// Ordered authoring operations, applied atomically.
    pub operations: Vec<PathOp>,
}
impl PathRequest {
    /// Validate this version and execute one atomic batch in a fresh owned builder.
    pub fn build(self) -> ChartResult<Path> {
        if self.version != 1 {
            return Err(Diagnostic::error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported standalone path request version",
                "Use path request version one",
            ));
        }
        let mut path = Path::with_options(
            self.digits
                .map(Precision::from_digits)
                .transpose()?
                .unwrap_or_default(),
            self.limits,
        )?;
        path.apply_batch(&self.operations)?;
        Ok(path)
    }
}

fn domain(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::NumericalDomain,
        message,
        "Supply finite representable path coordinates, angles and nonnegative radii/digits",
    )
}
fn require(ok: bool, message: &str) -> ChartResult<()> {
    if ok {
        Ok(())
    } else {
        Err(Diagnostic::error(
            DiagnosticCode::ResourceLimit,
            message,
            "Reduce the path or explicitly increase its resource budget",
        ))
    }
}
fn finite(values: &[f64]) -> ChartResult<()> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(domain("Non-finite path input or intermediate geometry"))
    }
}
