//! One mathematical box layout over the caller's existing font shaping service.
use super::{MathExpression, MathNode, RichRun, ShapeRequest, ShapedRun};
use crate::{ChartResult, DiagnosticCode, Point, scene::PathCommand, services::TextMeasurer};

pub(crate) enum MathPaint {
    Glyph {
        run: ShapedRun,
        x: f64,
        y: f64,
    },
    Stroke {
        commands: Vec<PathCommand>,
        width: f64,
    },
}
pub(crate) struct MathBox {
    pub width: f64,
    pub ascent: f64,
    pub descent: f64,
    pub italic: f64,
    pub paint: Vec<MathPaint>,
}
impl MathBox {
    fn empty(width: f64) -> Self {
        Self {
            width,
            ascent: 0.,
            descent: 0.,
            italic: 0.,
            paint: vec![],
        }
    }
    fn moved(mut self, x: f64, y: f64) -> ChartResult<Self> {
        for paint in &mut self.paint {
            match paint {
                MathPaint::Glyph { x: gx, y: gy, .. } => {
                    *gx += x;
                    *gy += y;
                }
                MathPaint::Stroke { commands, .. } => {
                    for command in commands {
                        let shift = |p: &mut Point| -> ChartResult<()> {
                            *p = Point::new(p.x() + x, p.y() + y)?;
                            Ok(())
                        };
                        match command {
                            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => shift(p)?,
                            PathCommand::QuadraticTo(p, q) => {
                                shift(p)?;
                                shift(q)?;
                            }
                            PathCommand::CubicTo(p, q, r) => {
                                shift(p)?;
                                shift(q)?;
                                shift(r)?;
                            }
                            PathCommand::Close => {}
                        }
                    }
                }
            }
        }
        Ok(self)
    }
    fn append(&mut self, other: Self, x: f64, y: f64) -> ChartResult<()> {
        if x + other.width >= self.width {
            self.italic = other.italic;
        }
        self.width = self.width.max(x + other.width);
        self.ascent = self.ascent.max(other.ascent - y);
        self.descent = self.descent.max(other.descent + y);
        self.paint.extend(other.moved(x, y)?.paint);
        Ok(())
    }
}
#[derive(Clone, Copy)]
enum Face {
    Regular,
    Italic,
    Bold,
    BoldItalic,
    Symbol,
}
#[derive(Clone, Copy)]
struct Style {
    face: Option<Face>,
    level: u8,
    display: bool,
    cramped: bool,
    magnification: f64,
}
impl Style {
    fn smaller(self) -> Self {
        Self {
            level: (self.level + 1).min(2),
            display: false,
            magnification: 1.,
            ..self
        }
    }
    fn factor(self) -> f64 {
        self.magnification
            * match self.level {
                0 => 1.,
                1 => 0.7,
                _ => 0.5,
            }
    }
}
pub(crate) fn layout(
    expression: &MathExpression,
    request: ShapeRequest<'_>,
    measurer: &dyn TextMeasurer,
) -> ChartResult<MathBox> {
    expression.validate(request.limits)?;
    let remaining = request.limits.max_path_commands;
    let mut context = Context {
        expression,
        request,
        measurer,
        remaining,
        properties: std::collections::BTreeMap::new(),
    };
    context.node(
        &expression.ast,
        Style {
            face: None,
            level: 0,
            display: true,
            cramped: false,
            magnification: 1.,
        },
    )
}
struct Context<'a> {
    expression: &'a MathExpression,
    request: ShapeRequest<'a>,
    measurer: &'a dyn TextMeasurer,
    remaining: usize,
    properties: std::collections::BTreeMap<(u8, u8), Properties>,
}
#[derive(Clone, Copy)]
struct Properties {
    x: f64,
    cap: f64,
    axis: f64,
    quad: f64,
    digit: f64,
    descender: f64,
}
impl Context<'_> {
    fn properties(&mut self, style: Style) -> ChartResult<Properties> {
        let face = style.face.unwrap_or(Face::Regular);
        let key = (
            match face {
                Face::Regular => 0,
                Face::Italic => 1,
                Face::Bold => 2,
                Face::BoldItalic => 3,
                Face::Symbol => 4,
            },
            style.level,
        );
        if let Some(metrics) = self.properties.get(&key) {
            return Ok(*metrics);
        }
        let style = Style {
            magnification: 1.,
            ..style
        };
        let properties = Properties {
            x: self.glyph("x", face, style)?.ascent,
            cap: self.glyph("X", face, style)?.ascent,
            axis: self.glyph("+", face, style)?.ascent / 2.,
            quad: self.glyph("M", face, style)?.width,
            digit: self.glyph("0", face, style)?.ascent,
            descender: self.glyph("g", face, style)?.descent,
        };
        self.properties.insert(key, properties);
        Ok(properties)
    }
    fn rule_gap(&self) -> f64 {
        if self.request.units == crate::services::Units::Points {
            1.08
        } else {
            1.44
        }
    }
    fn rule_width(&self) -> f64 {
        if self.request.units == crate::services::Units::Points {
            0.75
        } else {
            1.
        }
    }

    fn glyph(&mut self, text: &str, face: Face, style: Style) -> ChartResult<MathBox> {
        if text.is_empty() {
            return Ok(MathBox::empty(0.));
        }
        let fonts = &self.expression.fonts;
        let resource = match face {
            Face::Regular => fonts.regular.or(self.request.run.font),
            Face::Symbol => fonts.symbol.or(fonts.regular).or(self.request.run.font),
            Face::Italic => Some(fonts.italic.ok_or_else(|| {
                crate::scales::error(
                    DiagnosticCode::MissingResource,
                    "Math italic identifiers require an explicit italic face.",
                )
            })?),
            Face::Bold => Some(fonts.bold.ok_or_else(|| {
                crate::scales::error(
                    DiagnosticCode::MissingResource,
                    "Math bold style requires an explicit bold face.",
                )
            })?),
            Face::BoldItalic => Some(fonts.bold_italic.ok_or_else(|| {
                crate::scales::error(
                    DiagnosticCode::MissingResource,
                    "Math bold italic style requires an explicit bold italic face.",
                )
            })?),
        };
        let mut run = RichRun::new(text);
        run.font = resource;
        run.fallback.clone_from(&self.request.run.fallback);
        run.weight = if matches!(face, Face::Bold | Face::BoldItalic) {
            700
        } else {
            400
        };
        run.size = self.request.run.size * style.factor();
        run.language.clone_from(&self.request.run.language);
        run.direction = self.request.run.direction;
        let mut limits = self.request.limits;
        limits.max_path_commands = self.remaining;
        let shaped = self.measurer.shape(ShapeRequest {
            run: &run,
            default_font: self.request.default_font,
            font_size: self.request.font_size,
            units: self.request.units,
            limits,
        })?;
        let count = shaped.outlines.len().saturating_add(shaped.glyphs.len());
        crate::limits::require_within(count <= self.remaining, "math glyph geometry")?;
        self.remaining -= count;
        let mut ink_top = f64::INFINITY;
        let mut ink_bottom = f64::NEG_INFINITY;
        let mut include = |point: Point| {
            ink_top = ink_top.min(point.y());
            ink_bottom = ink_bottom.max(point.y());
        };
        for command in &shaped.outlines {
            match command {
                PathCommand::MoveTo(p) | PathCommand::LineTo(p) => include(*p),
                PathCommand::QuadraticTo(p, q) => {
                    include(*p);
                    include(*q);
                }
                PathCommand::CubicTo(p, q, r) => {
                    include(*p);
                    include(*q);
                    include(*r);
                }
                PathCommand::Close => {}
            }
        }
        let (ascent, descent) = if shaped.outlines.is_empty() {
            if text.trim().is_empty() {
                (0., 0.)
            } else {
                (shaped.metrics.ascent(), shaped.metrics.descent())
            }
        } else {
            (-ink_top, ink_bottom)
        };
        Ok(MathBox {
            width: shaped.metrics.width(),
            ascent,
            descent,
            italic: if matches!(face, Face::Italic | Face::BoldItalic) {
                0.15 * ascent
            } else {
                0.
            },
            paint: vec![MathPaint::Glyph {
                run: shaped,
                x: 0.,
                y: 0.,
            }],
        })
    }
    fn text_box(&mut self, text: &str, face: Face, style: Style) -> ChartResult<MathBox> {
        let mut result = self.glyph(text, face, style)?;
        // A character string combines its glyph boxes with the empty baseline box.
        // Symbol atoms instead preserve signed depth for accents and operator shifts.
        result.ascent = result.ascent.max(0.);
        result.descent = result.descent.max(0.);
        Ok(result)
    }
    fn stroke(&mut self, b: &mut MathBox, points: &[(f64, f64)], width: f64) -> ChartResult<()> {
        crate::limits::require_within(points.len() <= self.remaining, "math rule geometry")?;
        self.remaining -= points.len();
        let commands = points
            .iter()
            .enumerate()
            .map(|(i, &(x, y))| {
                Ok(if i == 0 {
                    PathCommand::MoveTo(Point::new(x, y)?)
                } else {
                    PathCommand::LineTo(Point::new(x, y)?)
                })
            })
            .collect::<ChartResult<_>>()?;
        for &(x, y) in points {
            b.width = b.width.max(x);
            b.ascent = b.ascent.max(-y);
            b.descent = b.descent.max(y);
        }
        b.paint.push(MathPaint::Stroke { commands, width });
        Ok(())
    }
    fn row(
        &mut self,
        nodes: &[MathNode],
        style: Style,
        separator: Option<&str>,
    ) -> ChartResult<MathBox> {
        let mut b = MathBox::empty(0.);
        for (i, node) in nodes.iter().enumerate() {
            if i > 0
                && let Some(separator) = separator
            {
                let s = self.glyph(separator, Face::Symbol, style)?;
                let x = b.width + b.italic;
                b.italic = 0.;
                b.append(s, x, 0.)?;
            }
            let item = self.node(node, style)?;
            let x = b.width + b.italic;
            b.italic = 0.;
            b.append(item, x, 0.)?;
        }
        Ok(b)
    }
    fn script(
        &mut self,
        base: &MathNode,
        sub: Option<&MathNode>,
        sup: Option<&MathNode>,
        style: Style,
    ) -> ChartResult<MathBox> {
        let mut b = self.node(base, style)?;
        let x = b.width + b.italic;
        let italic = b.italic;
        b.width = x;
        b.italic = 0.;
        let metrics = self.properties(style)?;
        let small = self.properties(style.smaller())?;
        let simple = matches!(
            base,
            MathNode::Text(_) | MathNode::Symbol(_) | MathNode::Number(_)
        );
        let upper = if let Some(sup) = sup {
            Some(self.node(sup, style.smaller())?)
        } else {
            None
        };
        let lower = if let Some(sub) = sub {
            Some(self.node(
                sub,
                Style {
                    cramped: true,
                    ..style.smaller()
                },
            )?)
        } else {
            None
        };
        let mut rise = if simple {
            0.
        } else {
            b.ascent - 0.3861111 * metrics.cap
        };
        let mut drop = if simple {
            0.
        } else {
            b.descent + 0.05 * metrics.cap
        };
        if let Some(upper) = &upper {
            rise = rise
                .max(if style.cramped {
                    0.7 * metrics.x
                } else if style.display {
                    0.95 * metrics.x
                } else {
                    0.825 * metrics.x
                })
                .max(upper.descent + 0.25 * metrics.x);
        }
        if let Some(lower) = &lower {
            if let Some(upper) = &upper {
                drop = drop.max(0.45 * metrics.cap);
                if rise - upper.descent - lower.ascent + drop < 4. * self.rule_gap() {
                    let adjustment = (0.8 * metrics.x - rise + upper.descent).max(0.);
                    rise += adjustment;
                    drop -= adjustment;
                }
            } else {
                drop = drop.max(0.35 * metrics.x).max(lower.ascent - 0.8 * small.x);
            }
        }
        if let Some(upper) = upper {
            b.append(upper, x + if lower.is_some() { italic } else { 0. }, -rise)?;
        }
        if let Some(lower) = lower {
            b.append(lower, x, drop)?;
        }
        b.italic = 0.;

        Ok(b)
    }
    fn node(&mut self, node: &MathNode, style: Style) -> ChartResult<MathBox> {
        match node {
            MathNode::Text(s) => self.text_box(
                &if matches!(style.face, Some(Face::Symbol)) {
                    super::math_symbols::symbol_text(s)
                } else {
                    s.clone()
                },
                style.face.unwrap_or(Face::Regular),
                style,
            ),
            MathNode::Number(s) => self.text_box(s, Face::Regular, style),
            MathNode::Symbol(s) => {
                if matches!(s.as_str(), "cdots" | "...") {
                    let mut dots = self.glyph("…", Face::Symbol, style)?;
                    let shift = self.properties(style)?.axis - dots.ascent / 2.;
                    let ascent = dots.ascent + shift;
                    let descent = dots.descent - shift;
                    dots = dots.moved(0., -shift)?;
                    dots.ascent = ascent;
                    dots.descent = descent;
                    Ok(dots)
                } else if let Some(symbol) = super::math_symbols::symbol(s) {
                    self.glyph(symbol, Face::Symbol, style)
                } else {
                    let text = if matches!(style.face, Some(Face::Symbol)) {
                        super::math_symbols::symbol_text(s)
                    } else {
                        s.clone()
                    };
                    let mut b = MathBox::empty(0.);
                    for c in text.chars() {
                        let face = if c.is_ascii_digit() {
                            Face::Regular
                        } else {
                            style.face.unwrap_or(Face::Regular)
                        };
                        let atom = self.glyph(&c.to_string(), face, style)?;
                        let x = b.width + b.italic;
                        b.italic = 0.;
                        b.append(atom, x, 0.)?;
                    }
                    Ok(b)
                }
            }
            MathNode::Unary { operator, value } => {
                let mut b = if operator == "~" {
                    self.glyph(" ", style.face.unwrap_or(Face::Regular), style)?
                } else {
                    self.glyph(super::math_symbols::operator(operator), Face::Symbol, style)?
                };
                let v = self.node(value, style)?;
                let x = b.width
                    + if operator != "~" && style.level == 0 {
                        self.properties(style)?.quad / 6.
                    } else {
                        0.
                    };
                b.append(v, x, 0.)?;
                Ok(b)
            }
            MathNode::Binary {
                operator,
                left,
                right,
            } => {
                let mut b = self.node(left, style)?;
                let metrics = self.properties(style)?;
                let spacing = match operator.as_str() {
                    "*" | "/" => 0.,
                    "~" => self.glyph(" ", Face::Symbol, style)?.width,
                    "~~" => 2. * self.glyph(" ", Face::Symbol, style)?.width,
                    _ if style.level > 0 => 0.,
                    "+" | "-" | ":" | "%+-%" | "%*%" | "%/%" | "%.%" => metrics.quad * 2. / 9.,
                    _ => metrics.quad * 5. / 18.,
                };
                let mut x = b.width + b.italic + spacing;
                b.italic = 0.;
                if !matches!(operator.as_str(), "*" | "~" | "~~") {
                    let op = if operator == "/" {
                        let mut slash = MathBox::empty(metrics.x);
                        let depth = metrics.axis / 2.;
                        let height = metrics.cap + depth;
                        self.stroke(
                            &mut slash,
                            &[(metrics.x / 4., depth), (metrics.x * 0.75, -height)],
                            self.rule_width(),
                        )?;
                        slash
                    } else {
                        self.glyph(super::math_symbols::operator(operator), Face::Symbol, style)?
                    };
                    let width = op.width;
                    b.append(op, x, 0.)?;
                    x += width + spacing;
                }
                let right = self.node(right, style)?;
                b.append(right, x, 0.)?;
                Ok(b)
            }
            MathNode::Superscript { base, script } => {
                if let MathNode::Subscript { base, script: sub } = base.as_ref() {
                    self.script(base, Some(sub), Some(script), style)
                } else {
                    self.script(base, None, Some(script), style)
                }
            }
            MathNode::Subscript { base, script } => {
                if let MathNode::Superscript { base, script: sup } = base.as_ref() {
                    self.script(base, Some(script), Some(sup), style)
                } else {
                    self.script(base, Some(script), None, style)
                }
            }
            MathNode::Group { visible, value } => {
                let b = self.node(value, style)?;
                if *visible {
                    self.delimit("(", b, ")", false, style)
                } else {
                    Ok(b)
                }
            }
            MathNode::Call { head, arguments } => self.call(head, arguments, style),
        }
    }
    fn delimit(
        &mut self,
        left: &str,
        body: MathBox,
        right: &str,
        scalable: bool,
        style: Style,
    ) -> ChartResult<MathBox> {
        let mut result = MathBox::empty(0.);
        let metrics = self.properties(style)?;
        let distance =
            (body.ascent - metrics.axis).max(body.descent + metrics.axis) + 0.2 * metrics.x;
        let mut body = Some(body);
        for (delimiter, before) in [(left, true), (right, false)] {
            if !before {
                let x = result.width;
                result.append(body.take().expect("body inserted once"), x, 0.)?;
            }
            if delimiter.is_empty() || delimiter == "." {
                continue;
            }
            let delimiter = match delimiter {
                "lfloor" => "⎣",
                "rfloor" => "⎦",
                "lceil" => "⎡",
                "rceil" => "⎤",
                "langle" => "〈",
                "rangle" => "〉",
                "||" => "|",
                other => other,
            };
            let glyph = if scalable {
                if !matches!(delimiter, "(" | ")" | "[" | "]" | "{" | "}" | "|" | "||") {
                    return Err(crate::scales::error(
                        DiagnosticCode::Validation,
                        "Reference scalable math delimiters support only parentheses, brackets, braces and bars.",
                    ));
                }
                let (top, extension, bottom, middle) = match delimiter {
                    "(" => ("⎛", "⎜", "⎝", None),
                    ")" => ("⎞", "⎟", "⎠", None),
                    "[" => ("⎡", "⎢", "⎣", None),
                    "]" => ("⎤", "⎥", "⎦", None),
                    "{" => ("⎧", "⎪", "⎩", Some("⎨")),
                    "}" => ("⎫", "⎪", "⎭", Some("⎬")),
                    _ => ("⎪", "⎪", "⎪", None),
                };
                let upper = self.glyph(top, Face::Symbol, style)?;
                let lower = self.glyph(bottom, Face::Symbol, style)?;
                let extender = self.glyph(extension, Face::Symbol, style)?;
                let upper_height = upper.ascent + upper.descent;
                let lower_height = lower.ascent + lower.descent;
                let distance =
                    distance.max(if middle.is_some() { 1.2 } else { 0.8 } * upper_height);
                let upper_y = upper.ascent - metrics.axis - distance;
                let lower_y = distance - lower.descent - metrics.axis;
                let mut b = MathBox::empty(upper.width.max(lower.width));
                b.append(upper, 0., upper_y)?;
                b.append(lower, 0., lower_y)?;
                if let Some(middle) = middle {
                    let center = self.glyph(middle, Face::Symbol, style)?;
                    let y = (center.ascent - center.descent) / 2. - metrics.axis;
                    b.append(center, 0., y)?;
                } else {
                    let start = -metrics.axis - distance + upper_height;
                    let end = distance - metrics.axis - lower_height;
                    let height = extender.ascent + extender.descent;
                    if end > start && height > 0. {
                        let n = libm::ceil((end - start) / (0.99 * height)) as usize;
                        crate::limits::require_within(
                            n <= self.remaining,
                            "math delimiter extenders",
                        )?;
                        for i in 0..n {
                            let part = self.glyph(extension, Face::Symbol, style)?;
                            let y = start
                                + (i as f64 + 0.5) * (end - start) / n as f64
                                + (part.ascent - part.descent) / 2.;
                            b.append(part, 0., y)?;
                        }
                    }
                }
                b
            } else {
                self.glyph(
                    delimiter,
                    Face::Symbol,
                    Style {
                        magnification: 1.25,
                        ..style
                    },
                )?
            };
            let x = result.width;
            result.append(glyph, x, 0.)?;
        }
        Ok(result)
    }
    fn call(&mut self, head: &MathNode, args: &[MathNode], style: Style) -> ChartResult<MathBox> {
        let name = if let MathNode::Symbol(s) = head {
            s.as_str()
        } else {
            ""
        };
        let arity = |min, max| -> ChartResult<()> {
            if args.len() < min || args.len() > max {
                Err(crate::scales::error(
                    DiagnosticCode::Validation,
                    format!("Invalid argument count for math {name}."),
                ))
            } else {
                Ok(())
            }
        };
        match name {
            "plain" | "bold" | "italic" | "bolditalic" | "symbol" | "displaystyle"
            | "textstyle" | "scriptstyle" | "scriptscriptstyle" => {
                arity(1, 1)?;
                let next = match name {
                    "plain" => Style {
                        face: Some(Face::Regular),
                        ..style
                    },
                    "bold" => Style {
                        face: Some(Face::Bold),
                        ..style
                    },
                    "italic" => Style {
                        face: Some(Face::Italic),
                        ..style
                    },
                    "bolditalic" => Style {
                        face: Some(Face::BoldItalic),
                        ..style
                    },
                    "symbol" => Style {
                        face: Some(Face::Symbol),
                        ..style
                    },
                    "displaystyle" => Style {
                        level: 0,
                        display: true,
                        ..style
                    },
                    "textstyle" => Style {
                        level: 0,
                        display: false,
                        ..style
                    },
                    "scriptstyle" => Style {
                        level: 1,
                        display: false,
                        ..style
                    },
                    _ => Style {
                        level: 2,
                        display: false,
                        ..style
                    },
                };
                self.node(&args[0], next)
            }
            "paste" | "list" => {
                self.row(args, style, if name == "list" { Some(", ") } else { None })
            }
            "phantom" => {
                arity(1, 1)?;
                let mut b = self.node(&args[0], style)?;
                b.paint.clear();
                Ok(b)
            }
            "frac" | "over" | "atop" => {
                arity(2, 2)?;
                let child = if style.display {
                    Style {
                        display: false,
                        ..style
                    }
                } else {
                    style.smaller()
                };
                let mut n = self.node(&args[0], child)?;
                let mut d = self.node(
                    &args[1],
                    Style {
                        cramped: true,
                        ..child
                    },
                )?;
                n.width += n.italic;
                n.italic = 0.;
                d.width += d.italic;
                d.italic = 0.;
                let metrics = self.properties(style)?;
                let rule = self.rule_gap();
                let clearance = if style.display { 3. * rule } else { rule };
                let numerator_target = metrics.axis
                    + if style.display {
                        3.51 * rule + 0.15 * metrics.cap + 0.7 * metrics.descender
                    } else {
                        1.51 * rule + 0.08333333 * metrics.cap
                    };
                let denominator_target = -metrics.axis
                    + if style.display {
                        3.51 * rule + 0.7 * metrics.digit + 0.344444 * metrics.cap
                    } else {
                        1.51 * rule + 0.7 * metrics.digit + 0.08333333 * metrics.cap
                    };
                let rise = numerator_target.max(n.descent + metrics.axis + rule / 2. + clearance);
                let drop = denominator_target.max(d.ascent - metrics.axis - rule / 2. + clearance);
                let width = n.width.max(d.width);
                let nx = (width - n.width) / 2.;
                let dx = (width - d.width) / 2.;
                let mut b = MathBox::empty(width);
                b.append(n, nx, -rise)?;
                b.append(d, dx, drop)?;
                if name != "atop" {
                    self.stroke(
                        &mut b,
                        &[(0., -metrics.axis), (width, -metrics.axis)],
                        self.rule_width(),
                    )?;
                }
                Ok(b)
            }
            "sqrt" => {
                arity(1, 2)?;
                let metrics = self.properties(style)?;
                let mut body = self.node(
                    &args[0],
                    Style {
                        cramped: true,
                        ..style
                    },
                )?;
                body.width += body.italic;
                let gap = 0.4 * metrics.x;
                let space = 0.2 * metrics.x;
                let trail = metrics.quad / 18.;
                let radical_width = 0.6 * metrics.cap;
                let top = body.ascent + gap;
                let middle = (body.ascent - body.descent) / 2.;
                let order = if args.len() == 2 {
                    Some(self.node(&args[1], style.smaller())?)
                } else {
                    None
                };
                let lead = order.as_ref().map_or(radical_width, |v| {
                    radical_width.max(v.width + 0.4 * radical_width)
                });
                let width = lead + space + body.width + 2. * trail;
                let mut b = MathBox::empty(width);
                let mut logical_ascent = body.ascent;
                let mut logical_descent = body.descent;
                if let Some(order) = order {
                    let x = lead - order.width - 0.4 * radical_width;
                    let rise = (top - order.ascent).max(middle + order.descent + gap);
                    logical_ascent = logical_ascent.max(order.ascent + rise);
                    logical_descent = logical_descent.max(order.descent);
                    b.append(order, x, -rise)?;
                }
                let x = lead - radical_width;
                let bottom = body.descent;
                b.append(body, lead + space, 0.)?;
                self.stroke(
                    &mut b,
                    &[
                        (x, -0.8 * middle),
                        (x + 0.3 * radical_width, -middle),
                        (x + 0.6 * radical_width, bottom),
                        (lead, -top),
                        (width - trail, -top),
                    ],
                    self.rule_width(),
                )?;
                b.ascent = logical_ascent + gap;
                b.descent = logical_descent;
                b.width = width;
                Ok(b)
            }
            "hat" | "widehat" | "tilde" | "widetilde" | "dot" | "ring" | "bar" | "underline" => {
                arity(1, 1)?;
                let mut b = self.node(&args[0], style)?;
                let metrics = self.properties(style)?;
                let top = -b.ascent - 0.2 * metrics.cap;
                let width = b.width;
                let italic = b.italic;
                let thickness = self.rule_width();
                match name {
                    "bar" => {
                        self.stroke(&mut b, &[(italic, top), (width + italic, top)], thickness)?;
                    }
                    "underline" => {
                        let y = b.descent + 0.1 * metrics.cap;
                        self.stroke(&mut b, &[(0., y), (width + italic, y)], thickness)?;
                    }
                    "widehat" => self.stroke(
                        &mut b,
                        &[
                            (0., top),
                            ((width + italic) / 2., top - 0.3 * metrics.cap),
                            (width + italic, top),
                        ],
                        thickness,
                    )?,
                    "widetilde" => {
                        let width = width + italic;
                        let height = 0.3 * metrics.cap;
                        let mut points = vec![(0., top)];
                        for i in 0..=8 {
                            let x = width * (0.05 + 0.9 * i as f64 / 8.);
                            let y = top
                                - height / 2.
                                    * (libm::sin(std::f64::consts::PI * (i as f64 / 4. - 0.5))
                                        + 1.);
                            points.push((x, y));
                        }
                        points.push((width, top - height));
                        self.stroke(&mut b, &points, thickness)?;
                    }
                    _ => {
                        let accent = self.glyph(
                            match name {
                                "hat" => "^",
                                "tilde" => "~",
                                "dot" => "⋅",
                                _ => "°",
                            },
                            if matches!(name, "dot" | "ring") {
                                Face::Symbol
                            } else {
                                style.face.unwrap_or(Face::Regular)
                            },
                            style,
                        )?;
                        let total = (width + italic).max(accent.width);
                        let x = (total - accent.width) / 2. + 0.9 * italic;
                        let y = -b.ascent - accent.descent - 0.1 * metrics.cap;
                        let body_x = (total - width) / 2.;
                        b = b.moved(body_x, 0.)?;
                        b.width = total;
                        b.append(accent, x, y)?;
                    }
                }
                Ok(b)
            }
            "group" | "bgroup" => {
                arity(3, 3)?;
                let delimiter = |v: &MathNode| -> ChartResult<String> {
                    match v {
                        MathNode::Text(s) | MathNode::Symbol(s) => Ok(s.clone()),
                        _ => Err(crate::scales::error(
                            DiagnosticCode::Validation,
                            "Math delimiter must be a string or delimiter name.",
                        )),
                    }
                };
                let left = delimiter(&args[0])?;
                let right = delimiter(&args[2])?;
                let body = self.node(&args[1], style)?;
                self.delimit(&left, body, &right, name == "bgroup", style)
            }
            "integral" => {
                arity(1, 3)?;
                let metrics = self.properties(style)?;
                let mut b = if style.display {
                    let upper = self.glyph("⌠", Face::Symbol, style)?;
                    let lower = self.glyph("⌡", Face::Symbol, style)?;
                    let up = metrics.axis + 0.99 * upper.descent;
                    let down = 0.99 * lower.ascent - metrics.axis;
                    let mut b = MathBox::empty(upper.width.max(lower.width));
                    b.append(upper, 0., -up)?;
                    b.append(lower, 0., down)?;
                    b
                } else {
                    self.glyph("∫", Face::Symbol, style)?
                };
                let width = b.width;
                if args.len() > 1 {
                    let sub = self.node(&args[1], style.smaller())?;
                    let drop = b.descent + (sub.ascent - sub.descent) / 2.;
                    b.append(sub, width / 2. + metrics.quad / 6., drop)?;
                }
                if args.len() > 2 {
                    let sup = self.node(&args[2], style.smaller())?;
                    let rise = b.ascent - (sup.ascent - sup.descent) / 2.;
                    b.append(sup, width + metrics.quad / 6., -rise)?;
                }
                let body = self.node(&args[0], style)?;
                let x = b.width;
                b.append(body, x, 0.)?;
                Ok(b)
            }
            "sum" | "prod" | "union" | "intersect" | "lim" | "min" | "max" | "inf" | "sup" => {
                arity(1, 3)?;
                let symbol = match name {
                    "sum" => "∑",
                    "prod" => "∏",
                    "union" => "∪",
                    "intersect" => "∩",
                    s => s,
                };
                let metrics = self.properties(style)?;
                let is_symbol = symbol != name;
                let mut op = self.glyph(
                    symbol,
                    if is_symbol {
                        Face::Symbol
                    } else {
                        Face::Regular
                    },
                    Style {
                        magnification: if is_symbol && style.display { 1.25 } else { 1. },
                        ..style
                    },
                )?;
                if is_symbol && style.display {
                    let display_axis = self
                        .glyph(
                            "+",
                            style.face.unwrap_or(Face::Regular),
                            Style {
                                magnification: 1.25,
                                ..style
                            },
                        )?
                        .ascent
                        / 2.;
                    let offset = (op.ascent - op.descent) / 2. - display_axis;
                    let ascent = op.ascent - offset;
                    let descent = op.descent + offset;
                    op = op.moved(0., offset)?;
                    op.ascent = ascent;
                    op.descent = descent;
                }
                let lower = if args.len() > 1 {
                    Some(self.node(&args[1], style.smaller())?)
                } else {
                    None
                };
                let upper = if args.len() > 2 {
                    Some(self.node(&args[2], style.smaller())?)
                } else {
                    None
                };
                let width = lower.as_ref().map_or(op.width, |v| op.width.max(v.width));
                let width = upper.as_ref().map_or(width, |v| width.max(v.width));
                let top = op.ascent;
                let bottom = op.descent;
                let mut b = MathBox::empty(width);
                let x = (width - op.width) / 2.;
                b.append(op, x, 0.)?;
                let gap = 0.15 * metrics.cap;
                if let Some(sub) = lower {
                    let x = (width - sub.width) / 2.;
                    let y = bottom + sub.ascent + gap;
                    b.append(sub, x, y)?;
                }
                if let Some(sup) = upper {
                    let x = (width - sup.width) / 2.;
                    let y = -top - sup.descent - gap;
                    b.append(sup, x, y)?;
                }
                b.ascent += gap;
                b.descent += gap;
                let body = self.node(&args[0], style)?;
                b.append(body, width + metrics.quad / 6., 0.)?;
                Ok(b)
            }
            _ => {
                let mut b = self.node(head, style)?;
                let row = self.row(args, style, Some(", "))?;
                let delimited = self.delimit("(", row, ")", false, style)?;
                let x = b.width;
                b.append(delimited, x, 0.)?;
                Ok(b)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Limits, ResourceId, Revision,
        services::{ResourceDescriptor, ResourceKind, TextMetrics, TextRequest, Units},
        typography::MathFonts,
    };
    struct Metrics;
    impl TextMeasurer for Metrics {
        fn measure(&self, r: TextRequest<'_>) -> ChartResult<TextMetrics> {
            TextMetrics::new(
                r.text.chars().count() as f64 * r.font_size / 2.,
                r.font_size * 0.75,
                r.font_size * 0.25,
            )
        }
        fn shape(&self, r: ShapeRequest<'_>) -> ChartResult<ShapedRun> {
            let size = r.font_size * r.run.size;
            Ok(ShapedRun {
                text: r.run.text.clone(),
                font: r.run.font.unwrap_or(*r.default_font),
                font_size: size,
                language: r.run.language.clone(),
                direction: r.run.direction,
                tabular: r.run.tabular,
                metrics: TextMetrics::new(
                    r.run.text.chars().count() as f64 * size / 2.,
                    size * 0.75,
                    size * 0.25,
                )?,
                glyphs: vec![],
                outlines: vec![PathCommand::MoveTo(Point::new(0., -size * 0.75)?)],
                used_fallback: false,
            })
        }
    }
    fn draw(source: &str) -> MathBox {
        let font = ResourceDescriptor {
            id: ResourceId::new(700),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: 1,
        };
        let fonts = MathFonts {
            regular: Some(font),
            italic: Some(font),
            bold: Some(font),
            bold_italic: Some(font),
            symbol: Some(font),
        };
        let expression = MathExpression::parse(source, fonts, Limits::default()).unwrap();
        let run = RichRun::new(source);
        layout(
            &expression,
            ShapeRequest {
                run: &run,
                default_font: &font,
                font_size: 12.,
                units: Units::Points,
                limits: Limits::default(),
            },
            &Metrics,
        )
        .unwrap()
    }
    #[test]
    fn every_reference_syntax_and_symbol_has_bounded_math_layout() {
        let v: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../fixtures/parity/ggplot2/plotmath-syntax-inventory.json"
        ))
        .unwrap();
        let mut cases = 0;
        for row in v["syntax"].as_array().unwrap() {
            if row["parseable"].as_bool() == Some(true) {
                let b = draw(row["syntax"].as_str().unwrap());
                assert!(b.width.is_finite() && b.ascent.is_finite() && b.descent.is_finite());
                cases += 1;
            }
        }
        for aliases in v["range_and_list_alias_expansion"]
            .as_object()
            .unwrap()
            .values()
        {
            for name in aliases.as_array().unwrap() {
                let name = name.as_str().unwrap();
                assert!(super::super::math_symbols::symbol(name).is_some());
                let b = draw(name);
                assert!(b.width > 0.);
                cases += 1;
            }
        }
        assert_eq!(cases, 149);
    }
    #[test]
    fn fraction_scripts_phantom_and_style_are_actual_box_topology() {
        let b = draw("frac(x,y)");
        let positions = b
            .paint
            .iter()
            .filter_map(|p| {
                if let MathPaint::Glyph { run, y, .. } = p {
                    Some((run.text.as_str(), *y, run.font_size))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert!(positions[0].1 < 0. && positions[1].1 > 0.);
        assert!((positions[0].2 - 12.).abs() < 1e-12);
        assert!(
            b.paint
                .iter()
                .any(|p| matches!(p, MathPaint::Stroke { .. }))
        );
        let b = draw("x[i]^2");
        let ys = b
            .paint
            .iter()
            .filter_map(|p| {
                if let MathPaint::Glyph { y, .. } = p {
                    Some(*y)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(ys[0], 0.);
        assert!(ys[1] < 0. && ys[2] > 0.);
        let plain = draw("x");
        let phantom = draw("phantom(x)");
        assert_eq!(
            (plain.width, plain.ascent, plain.descent),
            (phantom.width, phantom.ascent, phantom.descent)
        );
        assert!(phantom.paint.is_empty());
        assert!(draw("bgroup(\"(\",frac(x,y),\")\")").ascent > plain.ascent);
        assert!(
            draw("sqrt(x,3)")
                .paint
                .iter()
                .any(|p| matches!(p, MathPaint::Stroke { .. }))
        );
    }
}
