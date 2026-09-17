//! Parameterization of the pinned preset constructors; inheritance stays in the shared graph.
use super::elements::invalid;
use super::{ElementTheme, ThemeEntry, ThemePreset, ThemeValue};
use crate::{ChartResult, color::Paint};
use serde::{Deserialize, Serialize};
/// Custom inputs shared by all nine reference presets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ThemePresetOptions {
    /// Base text size in points.
    pub base_size: f64,
    /// Logical family name, resolved only through explicitly supplied fonts.
    pub base_family: String,
    /// Optional title family; absent inherits the base family.
    pub header_family: Option<String>,
    /// Base line width; absent derives base_size/22.
    pub base_line_size: Option<f64>,
    /// Base rectangle width; absent derives base_size/22.
    pub base_rect_size: Option<f64>,
    /// Foreground reference colour.
    pub ink: String,
    /// Background colour; absent uses white, or transparent ink in Void.
    pub paper: Option<String>,
    /// Accent reference colour.
    pub accent: String,
}
impl Default for ThemePresetOptions {
    fn default() -> Self {
        Self {
            base_size: 11.,
            base_family: String::new(),
            header_family: None,
            base_line_size: None,
            base_rect_size: None,
            ink: "black".into(),
            paper: None,
            accent: "#3366FF".into(),
        }
    }
}
impl ElementTheme {
    /// Construct a complete preset with explicit physical and colour defaults.
    pub fn preset_with(preset: ThemePreset, options: ThemePresetOptions) -> ChartResult<Self> {
        if !options.base_size.is_finite()
            || options.base_size <= 0.
            || [options.base_line_size, options.base_rect_size]
                .into_iter()
                .flatten()
                .any(|v| !v.is_finite() || v < 0.)
            || options.base_family.len() > 4096
            || options
                .header_family
                .as_ref()
                .is_some_and(|v| v.len() > 4096)
        {
            return Err(invalid("Invalid complete theme base controls."));
        }
        let ink = Paint::from_css(&options.ink)?.resolve();
        Paint::from_css(&options.accent)?;
        let paper = options.paper.clone().unwrap_or_else(|| {
            if preset == ThemePreset::Void {
                format!("#{:02X}{:02X}{:02X}00", ink.red, ink.green, ink.blue)
            } else {
                "white".into()
            }
        });
        let paper_color = Paint::from_css(&paper)?.resolve();
        let mut theme = Self::preset(preset)?;
        for (name, entry) in &mut theme.elements {
            let values = match entry {
                ThemeEntry::Element(e) => Some(&mut e.properties),
                _ => None,
            };
            if let Some(values) = values {
                for (property, value) in values {
                    if let ThemeValue::Text(v) = value {
                        if matches!(
                            property.as_str(),
                            "colour" | "arrow.fill" | "fill" | "ink" | "paper" | "accent"
                        ) {
                            if let Some(ratio) = mix_ratio(preset, name, property) {
                                let channel = |a: u8, b: u8| {
                                    (f64::from(a) * (1. - ratio) + f64::from(b) * ratio)
                                        .round_ties_even() as u8
                                };
                                *v = format!(
                                    "#{:02X}{:02X}{:02X}{:02X}",
                                    channel(ink.red, paper_color.red),
                                    channel(ink.green, paper_color.green),
                                    channel(ink.blue, paper_color.blue),
                                    channel(ink.alpha, paper_color.alpha)
                                );
                            } else {
                                *v = match v.as_str() {
                                    "black" => options.ink.clone(),
                                    "white" | "#00000000" => paper.clone(),
                                    "#3366FF" => options.accent.clone(),
                                    _ => v.clone(),
                                };
                            }
                        } else if property == "family" {
                            *v = options.base_family.clone();
                        }
                    }
                    if let ThemeValue::Number(v) = value {
                        match (name.as_str(), property.as_str()) {
                            ("text", "size") => *v = options.base_size,
                            ("geom", "fontsize") => *v = options.base_size / (72.27 / 25.4),
                            ("point", "size") | ("geom", "pointsize") => {
                                *v = options.base_size / 11. * 1.5
                            }
                            ("line", "linewidth")
                            | ("point", "stroke")
                            | ("geom", "linewidth" | "borderwidth") => {
                                *v = options.base_line_size.unwrap_or(options.base_size / 22.)
                            }
                            ("rect" | "polygon", "linewidth") if preset != ThemePreset::Void => {
                                *v = options.base_rect_size.unwrap_or(options.base_size / 22.)
                            }
                            _ => {}
                        }
                    }
                    scale_points(value, options.base_size / 11.);
                }
            } else if let ThemeEntry::Value(v) = entry {
                scale_points(v, options.base_size / 11.);
            }
        }
        if let Some(family) = options.header_family
            && let Some(ThemeEntry::Element(title)) = theme.elements.get_mut("title")
        {
            title
                .properties
                .insert("family".into(), ThemeValue::Text(family));
        }
        theme.validate()?;
        Ok(theme)
    }
}
fn scale_points(value: &mut ThemeValue, factor: f64) {
    match value {
        ThemeValue::Unit(v) | ThemeValue::Margin(v) => {
            for n in v {
                if n.unit == "points" {
                    n.value = n.value.map(|v| v * factor);
                }
            }
        }
        _ => {}
    }
}
fn mix_ratio(p: ThemePreset, node: &str, property: &str) -> Option<f64> {
    use ThemePreset::*;
    // These constants are the pinned col_mix calls, not ratios inferred from rounded colours.
    let property = if property == "arrow.fill" {
        "colour"
    } else {
        property
    };
    match (node, property) {
        ("axis.text", "colour") if !matches!(p, Linedraw | Classic | Void) => Some(0.302),
        ("axis.ticks", "colour") => match p {
            Linedraw | Void => None,
            Light => Some(0.702),
            _ => Some(0.2),
        },
        ("panel.background", "fill") => match p {
            Grey => Some(0.92),
            Dark => Some(0.499),
            _ => None,
        },
        ("panel.border", "colour") => match p {
            Bw | Test => Some(0.2),
            Light => Some(0.702),
            _ => None,
        },
        ("panel.grid", "colour") => match p {
            Bw | Minimal => Some(0.92),
            Light => Some(0.871),
            Dark => Some(0.42),
            _ => None,
        },
        ("strip.background", "fill") => match p {
            Grey | Test => Some(0.85),
            Bw | Classic => Some(0.851),
            Light => Some(0.702),
            Dark => Some(0.15),
            _ => None,
        },
        ("strip.background", "colour") if matches!(p, Bw | Classic | Test) => Some(0.2),
        ("strip.text", "colour") => match p {
            Linedraw | Light | Void => None,
            Dark => Some(0.899),
            _ => Some(0.1),
        },
        _ => None,
    }
}
