//! Bounded typed reference theme elements, shared by every presentation consumer.
use crate::{ChartResult, Diagnostic, DiagnosticCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
/// Physical or contextual theme length. Contextual units resolve against the consumer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeLength {
    /// None retains an unspecified margin side for inheritance.
    pub value: Option<f64>,
    /// Reference unit name: points, mm, cm, inches, lines, npc or null.
    pub unit: String,
}
/// Typed scalar and compound element property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ThemeValue {
    /// Bounded reference arrow control using the shared physical arrow owner.
    Arrow(crate::grammar::ArrowSpec),
    /// Explicit NA; unlike absence, it does not inherit a property.
    Missing,
    /// Logical control.
    Bool(bool),
    /// Absolute scalar.
    Number(f64),
    /// Text, colour spelling or an enumerated reference control.
    Text(String),
    /// Multiplier applied to the inherited numeric/unit value.
    Relative(f64),
    /// One or more physical/contextual lengths.
    Unit(Vec<ThemeLength>),
    /// Top/right/bottom/left lengths, with independently unspecified sides.
    Margin(Vec<ThemeLength>),
    /// Bounded scalar-vector controls, such as inside justification.
    Vector(Vec<ThemeValue>),
}
/// Element property schema; blank is represented separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementKind {
    /// Stroke controls.
    Line,
    /// Rectangle background/border.
    Rect,
    /// Measured text.
    Text,
    /// Point glyph.
    Point,
    /// Polygon glyph.
    Polygon,
    /// Geometry defaults and expression tokens.
    Geom,
}
/// A component element. Absent properties inherit, explicit Missing values do not.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeElement {
    /// Schema selecting accepted properties.
    pub kind: ElementKind,
    /// Validated reference property names, with typed values.
    pub properties: BTreeMap<String, ThemeValue>,
}
/// One node in the reference hierarchy.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ThemeEntry {
    /// No paint and no reserved component space.
    Blank,
    /// Styled component.
    Element(ThemeElement),
    /// Scalar/unit/control node.
    Value(ThemeValue),
}
/// Explicit reference theme, independent of process globals.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ElementTheme {
    /// Complete themes form a replacement boundary when composed.
    #[serde(default)]
    pub complete: bool,
    /// Registered reference node names; omission means inheritance.
    #[serde(default)]
    pub elements: BTreeMap<String, ThemeEntry>,
}
pub(super) fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use bounded reference theme elements and values.",
    )
}
impl ThemeValue {
    pub(super) fn validate(&self, depth: usize) -> ChartResult<()> {
        if depth > 8 {
            return Err(invalid("Theme value nesting exceeds eight."));
        }
        match self {
            Self::Arrow(a) => a.validate()?,
            Self::Number(v) | Self::Relative(v) if !v.is_finite() => {
                return Err(invalid("Theme values must be finite."));
            }
            Self::Text(v) if v.len() > 4096 => {
                return Err(invalid("Theme text exceeds 4096 bytes."));
            }
            Self::Unit(v) | Self::Margin(v) => {
                if v.len() > 64 || matches!(self, Self::Margin(_)) && v.len() != 4 {
                    return Err(invalid("Theme unit/margin size is invalid."));
                }
                for n in v {
                    if n.value.is_some_and(|v| !v.is_finite())
                        || !matches!(
                            n.unit.as_str(),
                            "points"
                                | "bigpts"
                                | "picas"
                                | "dida"
                                | "cicero"
                                | "scaledpts"
                                | "snpc"
                                | "pt"
                                | "mm"
                                | "cm"
                                | "inches"
                                | "in"
                                | "lines"
                                | "npc"
                                | "null"
                                | "char"
                                | "native"
                        )
                    {
                        return Err(invalid("Unsupported or nonfinite theme unit."));
                    }
                }
            }
            Self::Vector(v) => {
                if v.len() > 64 {
                    return Err(invalid("Theme vector exceeds64 values."));
                }
                for x in v {
                    x.validate(depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
impl ThemeElement {
    /// Construct a component with no authored properties.
    pub fn new(kind: ElementKind) -> Self {
        Self {
            kind,
            properties: BTreeMap::new(),
        }
    }
    /// Add or replace one validated property.
    pub fn property(mut self, name: impl Into<String>, value: ThemeValue) -> ChartResult<Self> {
        self.properties.insert(name.into(), value);
        self.validate()?;
        Ok(self)
    }
    pub(super) fn validate(&self) -> ChartResult<()> {
        let allowed = match self.kind {
            ElementKind::Line => {
                "arrow arrow.fill colour inherit.blank lineend linejoin linetype linewidth"
            }
            ElementKind::Rect | ElementKind::Polygon => {
                "colour fill inherit.blank linejoin linetype linewidth"
            }
            ElementKind::Text => {
                "angle colour debug face family fontweight fontwidth hjust inherit.blank italic lineheight margin size vjust"
            }
            ElementKind::Point => "colour fill inherit.blank shape size stroke",
            ElementKind::Geom => {
                "accent bordertype borderwidth colour family fill fontsize ink linetype linewidth paper pointshape pointsize"
            }
        };
        for (name, value) in &self.properties {
            if !allowed.split_whitespace().any(|v| v == name) {
                return Err(invalid(format!(
                    "Unsupported {name} property for {:?}.",
                    self.kind
                )));
            }
            value.validate(0)?;
            if matches!(
                name.as_str(),
                "colour" | "fill" | "ink" | "paper" | "accent" | "arrow.fill"
            ) && let ThemeValue::Text(colour) = value
            {
                crate::color::Paint::from_css(colour)?;
            }
            let valid = match name.as_str() {
                "colour" | "fill" | "ink" | "paper" | "accent" | "arrow.fill" | "family" => {
                    matches!(value, ThemeValue::Text(_) | ThemeValue::Missing)
                }
                "lineend" => {
                    matches!(value,ThemeValue::Text(v) if matches!(v.as_str(),"butt"|"round"|"square"))
                }
                "linejoin" => {
                    matches!(value,ThemeValue::Text(v) if matches!(v.as_str(),"mitre"|"miter"|"round"|"bevel"))
                }

                "arrow" => matches!(
                    value,
                    ThemeValue::Arrow(_) | ThemeValue::Bool(false) | ThemeValue::Missing
                ),
                "inherit.blank" | "debug" => {
                    matches!(value, ThemeValue::Bool(_) | ThemeValue::Missing)
                }
                "margin" => matches!(value, ThemeValue::Margin(_) | ThemeValue::Missing),
                "size" | "linewidth" => matches!(
                    value,
                    ThemeValue::Number(_) | ThemeValue::Relative(_) | ThemeValue::Missing
                ),
                "angle" | "hjust" | "vjust" | "lineheight" | "fontweight" | "fontwidth"
                | "stroke" | "fontsize" | "pointsize" | "borderwidth" => {
                    matches!(value, ThemeValue::Number(_) | ThemeValue::Missing)
                }
                _ => true,
            };
            if !valid {
                return Err(invalid(format!("Invalid type for theme property {name}.")));
            }
        }
        Ok(())
    }
}
