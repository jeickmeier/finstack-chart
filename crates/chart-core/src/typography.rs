//! Explicit rich-text requests and destination-shaped immutable glyphs.
use crate::scene::PathCommand;
use crate::services::{ResourceDescriptor, TextMetrics, Units};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Limits, Point};
use serde::{Deserialize, Serialize};

/// Direction of one authored directional run. Mixed paragraphs use separate ordered runs.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextDirection {
    /// Left-to-right run.
    #[default]
    LeftToRight,
    /// Right-to-left run.
    RightToLeft,
}
/// One homogeneous run. Exact face resources, including bold faces, are caller-owned.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RichRun {
    /// Preserved logical UTF-8 text, without line breaks; lines are explicit in RichText.
    pub text: String,
    /// Parsed plotmath notation; logical text must match its source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub math: Option<MathExpression>,
    /// Primary face; absent uses the destination's supplied face.
    #[serde(default)]
    pub font: Option<ResourceDescriptor>,
    /// Ordered explicit fallback faces; fallback replaces this complete run and emits a warning.
    #[serde(default)]
    pub fallback: Vec<ResourceDescriptor>,
    /// Requested weight (100..900) must match the chosen face's OS/2 weight.
    #[serde(default = "normal_weight")]
    pub weight: u16,
    /// Size multiplier on destination label size.
    #[serde(default = "one")]
    pub size: f64,
    /// Optional run color; absent inherits the furniture color.
    #[serde(default)]
    pub color: Option<crate::color::Paint>,
    /// Explicit shaping language tag; default is undetermined.
    #[serde(default = "und")]
    pub language: String,
    /// One authored directional run.
    #[serde(default)]
    pub direction: TextDirection,
    /// Enable OpenType tabular numeral substitution when available.
    #[serde(default)]
    pub tabular: bool,
}
fn normal_weight() -> u16 {
    400
}
fn one() -> f64 {
    1.
}
fn und() -> String {
    "und".into()
}
impl RichRun {
    /// Plain regular text in the destination font, using an explicit undetermined language.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            math: None,
            font: None,
            fallback: vec![],
            weight: 400,
            size: 1.,
            color: None,
            language: und(),
            direction: TextDirection::LeftToRight,
            tabular: false,
        }
    }
    /// Replace a template's label, reparsing mathematical notation when present.
    pub fn replace_text(&mut self, text: impl Into<String>, limits: Limits) -> ChartResult<()> {
        let text = text.into();
        if let Some(math) = &self.math {
            self.math = Some(MathExpression::parse(
                text.clone(),
                math.fonts.clone(),
                limits,
            )?);
        }
        self.text = text;
        Ok(())
    }
    /// Validate work bounds and portable language/typography parameters before shaping.
    pub fn validate(&self, limits: Limits) -> ChartResult<()> {
        if let Some(math) = &self.math {
            if math.source != self.text {
                return Err(crate::scales::error(
                    DiagnosticCode::Validation,
                    "Math run source differs from its logical text.",
                ));
            }
            math.validate(limits)?;
        }
        if self.text.len() > limits.max_text_bytes || self.fallback.len() > 16 {
            return Err(Diagnostic::error(
                DiagnosticCode::ResourceLimit,
                "Rich text exceeds its text/fallback budget.",
                "Reduce run bytes or fallback faces.",
            ));
        }
        if self.text.chars().any(char::is_control)
            || !(100..=900).contains(&self.weight)
            || !self.size.is_finite()
            || self.size <= 0.
            || self.size > 64.
            || self.language.is_empty()
            || self.language.len() > 63
            || !self
                .language
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "Invalid rich run, weight, size or language.",
                "Use explicit lines, a finite size multiplier, a real face weight and an ASCII language tag.",
            ));
        }
        for font in self.font.iter().chain(&self.fallback) {
            font.validate(limits)?;
        }
        Ok(())
    }
}
/// One multiline block; baseline spacing is a positive multiple of each line's tallest run.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RichText {
    /// Lines, each with ordered independently shaped directional runs.
    pub lines: Vec<Vec<RichRun>>,
    /// Positive multiplier on maximum line ascent plus descent.
    pub line_spacing: f64,
    /// Clockwise rotation about the block's first baseline origin, in degrees.
    pub rotation: f64,
}
impl RichText {
    /// Single run with 1.2 line spacing and no rotation.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            lines: vec![vec![RichRun::new(text)]],
            line_spacing: 1.2,
            rotation: 0.,
        }
    }
    /// Parse a mathematical block through the shared nonexecuting plotmath owner.
    pub fn math(source: impl Into<String>, fonts: MathFonts) -> ChartResult<Self> {
        let expression = MathExpression::parse(source, fonts, Limits::default())?;
        let mut run = RichRun::new(expression.source.clone());
        run.math = Some(expression);
        Ok(Self {
            lines: vec![vec![run]],
            line_spacing: 1.2,
            rotation: 0.,
        })
    }
    /// Check bounded lines/runs and total logical text before allocating shaped geometry.
    pub fn validate(&self, limits: Limits) -> ChartResult<()> {
        if self.lines.len() > 128
            || self.lines.iter().map(Vec::len).sum::<usize>() > 1024
            || self
                .lines
                .iter()
                .flatten()
                .map(|r| r.text.len())
                .sum::<usize>()
                > limits.max_text_bytes
        {
            return Err(Diagnostic::error(
                DiagnosticCode::ResourceLimit,
                "Rich text block exceeds its line/run/text budget.",
                "Reduce block size.",
            ));
        }
        if !self.line_spacing.is_finite()
            || self.line_spacing <= 0.
            || self.line_spacing > 8.
            || !self.rotation.is_finite()
            || self.rotation.abs() > 360.
        {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "Invalid rich text spacing or rotation.",
                "Use positive line spacing up to8 and rotation -360..360 degrees.",
            ));
        }
        for r in self.lines.iter().flatten() {
            r.validate(limits)?;
        }
        Ok(())
    }
}
/// Destination request used by the same shaper for measurement and paint geometry.
pub struct ShapeRequest<'a> {
    /// Authored homogeneous run.
    pub run: &'a RichRun,
    /// Explicit destination default face.
    pub default_font: &'a ResourceDescriptor,
    /// Base size before the run multiplier.
    pub font_size: f64,
    /// Native logical pixels or physical points.
    pub units: Units,
    /// Bounded geometry/text budget.
    pub limits: Limits,
}
/// One shaped glyph with exact logical cluster and baseline placement.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ShapedGlyph {
    /// Face glyph index.
    pub id: u16,
    /// UTF-8 cluster start.
    pub start: usize,
    /// UTF-8 cluster end, exclusive.
    pub end: usize,
    /// Baseline displacement in destination units.
    pub position: Point,
    /// Pen advance after this glyph, in destination coordinates.
    pub advance: Point,
}
/// Immutable result of an explicit font shaping service, also used for text measurement.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ShapedRun {
    /// Preserved logical text.
    pub text: String,
    /// Actually selected exact face descriptor.
    pub font: ResourceDescriptor,
    /// Actual destination font size.
    pub font_size: f64,
    /// Authored shaping language.
    pub language: String,
    /// Authored run direction.
    pub direction: TextDirection,
    /// OpenType tabular feature request.
    pub tabular: bool,
    /// Destination advances and baseline metrics.
    pub metrics: TextMetrics,
    /// Positioned face glyphs with source clusters for searchable PDF.
    pub glyphs: Vec<ShapedGlyph>,
    /// Numeric glyph outlines in baseline-local coordinates, using nonzero winding.
    pub outlines: Vec<PathCommand>,
    /// True only when an explicitly listed fallback replaced the primary face.
    pub used_fallback: bool,
}

/// Transform exact baseline-local numeric outlines, preserving curve commands.
pub fn placed_outlines(
    run: &ShapedRun,
    origin: Point,
    rotation: f64,
) -> ChartResult<Vec<PathCommand>> {
    let (sin, cos) = rotation.to_radians().sin_cos();
    let point = |p: Point| {
        Point::new(
            origin.x() + cos * p.x() - sin * p.y(),
            origin.y() + sin * p.x() + cos * p.y(),
        )
    };
    run.outlines
        .iter()
        .map(|c| {
            Ok(match c {
                PathCommand::MoveTo(a) => PathCommand::MoveTo(point(*a)?),
                PathCommand::LineTo(a) => PathCommand::LineTo(point(*a)?),
                PathCommand::QuadraticTo(a, b) => PathCommand::QuadraticTo(point(*a)?, point(*b)?),
                PathCommand::CubicTo(a, b, c) => {
                    PathCommand::CubicTo(point(*a)?, point(*b)?, point(*c)?)
                }
                PathCommand::Close => PathCommand::Close,
            })
        })
        .collect()
}

/// Portable numeric guide notation; each variant formats the actual scale tick value.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumberNotation {
    /// Fixed decimal places.
    Fixed,
    /// Scientific mantissa and decimal exponent.
    Scientific,
    /// Multiply display labels by 100 and append a percent sign.
    Percent,
}
/// Explicit portable punctuation; never reads a machine locale.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumberLocale {
    /// Decimal point and optional comma groups.
    EnUs,
    /// Decimal comma and optional narrow nonbreaking-space groups.
    FrFr,
}
/// Validated numeric display descriptor. It never rounds the mapped scale coordinate.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NumberFormat {
    /// Fixed, scientific or percent display.
    pub notation: NumberNotation,
    /// Decimal places, bounded to 0..12.
    pub precision: u8,
    /// Explicit punctuation locale.
    pub locale: NumberLocale,
    /// Group integer digits in threes for fixed/percent notation.
    pub grouping: bool,
    /// Optional fixed prefix, at most 64 bytes.
    pub prefix: String,
    /// Optional fixed suffix, at most 64 bytes.
    pub suffix: String,
}
impl NumberFormat {
    /// Validate before tick generation, including empty-domain guides.
    pub fn validate(&self) -> ChartResult<()> {
        if self.precision > 12
            || self.prefix.len() > 64
            || self.suffix.len() > 64
            || self
                .prefix
                .chars()
                .chain(self.suffix.chars())
                .any(char::is_control)
        {
            return Err(Diagnostic::error(
                DiagnosticCode::Validation,
                "Invalid portable numeric formatter.",
                "Use 0..12 decimals and bounded printable affixes.",
            ));
        }
        Ok(())
    }
    /// Format an exact finite scale tick, retaining Unicode minus and explicit locale.
    pub fn format(&self, value: f64) -> ChartResult<String> {
        self.validate()?;
        let value = if self.notation == NumberNotation::Percent {
            value * 100.
        } else {
            value
        };
        if !value.is_finite() {
            return Err(Diagnostic::error(
                DiagnosticCode::NumericalDomain,
                "Numeric display conversion exceeds finite range.",
                "Choose a suitable notation or represented domain.",
            ));
        }
        let value = if value == 0. { 0. } else { value };
        let digits = usize::from(self.precision);
        let mut s = if self.notation == NumberNotation::Scientific {
            format!("{value:.digits$e}")
        } else {
            format!("{value:.digits$}")
        };
        if self.grouping && self.notation != NumberNotation::Scientific {
            let (integer, fraction) = s
                .split_once('.')
                .map_or((s.as_str(), None), |(a, b)| (a, Some(b)));
            let sign = integer.starts_with('-');
            let raw = integer.trim_start_matches('-');
            let mut grouped = String::new();
            if sign {
                grouped.push('-');
            }
            for (i, c) in raw.chars().enumerate() {
                if i > 0 && (raw.len() - i) % 3 == 0 {
                    grouped.push(if self.locale == NumberLocale::EnUs {
                        ','
                    } else {
                        '\u{202f}'
                    });
                }
                grouped.push(c);
            }
            if let Some(f) = fraction {
                grouped.push('.');
                grouped.push_str(f);
            }
            s = grouped;
        }
        if self.locale == NumberLocale::FrFr {
            s = s.replace('.', ",");
        }
        s = s.replace('-', "−");
        Ok(format!(
            "{}{}{}{}",
            self.prefix,
            s,
            if self.notation == NumberNotation::Percent {
                "%"
            } else {
                ""
            },
            self.suffix
        ))
    }
}

impl RichText {
    /// Whether any run requires the authored floating-color capability.
    pub fn has_floating_paint(&self) -> bool {
        self.lines
            .iter()
            .flatten()
            .any(|r| r.color.is_some_and(crate::color::Paint::is_floating))
    }
}

mod numeric_format;
pub use numeric_format::*;
mod ggplot_format;
pub use ggplot_format::ggplot_numeric_labels;
mod ggplot_duration;
pub use ggplot_duration::ggplot_duration_labels;

pub use crate::scales::{TimeFormat, TimeFormatter, TimeLocale};

mod math;
pub use math::{MathExpression, MathFonts, MathNode};

pub(crate) mod math_layout;
mod math_symbols;
