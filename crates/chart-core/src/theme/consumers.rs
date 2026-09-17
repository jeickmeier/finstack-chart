//! Shared resolved-element access for grammar and destination consumers.
use super::{ElementTheme, GeometryTheme, ThemeEntry, ThemeSpec, ThemeValue};
use crate::{ChartResult, color::Paint, services::Units};
use std::collections::BTreeMap;
/// Fully inherited reference elements, shared by all destination consumers.
#[derive(Clone, Debug, Default)]
pub struct ResolvedElements {
    pub(crate) overrides: super::ThemePatch<Paint>,
    /// Destination-local lengths populated once before layout.
    pub(crate) lengths: BTreeMap<String, Vec<Option<f64>>>,
    /// Resolved named element values.
    pub elements: BTreeMap<String, Option<ThemeEntry>>,
    /// Exact caller-supplied font faces.
    pub fonts: Vec<crate::grammar::TextFont>,
}
impl ResolvedElements {
    /// Resolve one bounded hierarchy once for a preparation pass.
    pub fn new(theme: &ElementTheme) -> ChartResult<Self> {
        Ok(Self {
            elements: theme.resolve_all()?,
            overrides: Default::default(),
            fonts: Vec::new(),
            lengths: BTreeMap::new(),
        })
    }
    /// Read a length already resolved for this destination's font and bounds.
    pub fn destination_length(&self, name: &str, property: &str, index: usize) -> Option<f64> {
        self.lengths
            .get(&format!("{name}:{property}"))?
            .get(index)
            .copied()
            .flatten()
    }
    pub(crate) fn prepare_lengths(
        &mut self,
        units: Units,
        font_size: f64,
        width: f64,
        height: f64,
    ) -> ChartResult<()> {
        let mut lengths = BTreeMap::new();
        for (name, entry) in &self.elements {
            let properties: Vec<(&str, &ThemeValue)> = match entry {
                Some(ThemeEntry::Element(e)) => {
                    e.properties.iter().map(|(k, v)| (k.as_str(), v)).collect()
                }
                Some(ThemeEntry::Value(v)) => vec![("", v)],
                _ => Vec::new(),
            };
            for (property, value) in properties {
                if let ThemeValue::Unit(v) | ThemeValue::Margin(v) = value {
                    let resolved = (0..v.len())
                        .map(|i| {
                            if v[i].unit == "null"
                                && matches!(name.as_str(), "panel.widths" | "panel.heights")
                            {
                                return Ok(None);
                            }
                            let extent = if v[i].unit == "snpc" {
                                width.min(height)
                            } else if property == "margin" {
                                if i % 2 == 0 { height } else { width }
                            } else if name.ends_with("width") || name == "panel.widths" {
                                width
                            } else {
                                height
                            };
                            self.length(name, property, i, units, font_size, extent)
                        })
                        .collect::<ChartResult<Vec<_>>>()?;
                    lengths.insert(format!("{name}:{property}"), resolved);
                }
            }
        }
        self.lengths = lengths;
        Ok(())
    }
    /// Allocate panel extents from recycled absolute lengths and null weights.
    pub fn panel_sizes(
        &self,
        name: &str,
        count: usize,
        available: f64,
    ) -> ChartResult<Option<Vec<f64>>> {
        let Some(ThemeValue::Unit(values)) = self.value(name, "") else {
            return Ok(None);
        };
        if values.is_empty() || count == 0 {
            return Ok(None);
        }
        let mut fixed = 0.;
        let mut weight = 0.;
        for i in 0..count {
            let value = &values[i % values.len()];
            let n = value.value.unwrap_or(0.);
            if n < 0. {
                return Err(super::elements::invalid(
                    "Panel lengths must be nonnegative.",
                ));
            }
            if value.unit == "null" {
                weight += n;
            } else {
                fixed += self
                    .destination_length(name, "", i % values.len())
                    .unwrap_or(0.);
            }
        }
        let remainder = (available - fixed).max(0.);
        Ok(Some(
            (0..count)
                .map(|i| {
                    let value = &values[i % values.len()];
                    if value.unit == "null" {
                        if weight > 0. {
                            remainder * value.value.unwrap_or(0.) / weight
                        } else {
                            0.
                        }
                    } else {
                        self.destination_length(name, "", i % values.len())
                            .unwrap_or(0.)
                    }
                })
                .collect(),
        ))
    }
    /// Whether an element resolves to an explicit blank.
    pub fn blank(&self, name: &str) -> bool {
        matches!(self.elements.get(name), Some(Some(ThemeEntry::Blank)))
    }
    /// Read an inherited property, or a scalar node when property is empty.
    pub fn value(&self, name: &str, property: &str) -> Option<&ThemeValue> {
        match self.elements.get(name)?.as_ref()? {
            ThemeEntry::Element(e) => e.properties.get(property),
            ThemeEntry::Value(v) if property.is_empty() => Some(v),
            _ => None,
        }
    }
    /// Read a finite resolved numeric property.
    pub fn number(&self, name: &str, property: &str) -> Option<f64> {
        match self.value(name, property)? {
            ThemeValue::Number(v) => Some(*v),
            _ => None,
        }
    }
    /// Resolve a CSS paint; explicit missing paint means transparent.
    pub fn paint(&self, name: &str, property: &str) -> ChartResult<Option<Paint>> {
        match self.value(name, property) {
            Some(ThemeValue::Text(v)) => Ok(Some(Paint::from_css(v)?)),
            Some(ThemeValue::Missing) => Ok(Some(
                crate::scene::Color {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                }
                .into(),
            )),
            _ => Ok(None),
        }
    }
    /// Resolve an explicit length using destination units and contextual text/panel extent.
    pub fn length(
        &self,
        name: &str,
        property: &str,
        index: usize,
        units: Units,
        font_size: f64,
        extent: f64,
    ) -> ChartResult<Option<f64>> {
        let Some(ThemeValue::Unit(v) | ThemeValue::Margin(v)) = self.value(name, property) else {
            return Ok(None);
        };
        let Some(length) = v.get(index) else {
            return Ok(None);
        };
        let Some(value) = length.value else {
            return Ok(None);
        };
        let per_inch = match units {
            Units::Points => 72.,
            Units::LogicalPixels => 96.,
        };
        let factor = match length.unit.as_str() {
            "points" | "pt" => per_inch / 72.27,
            "bigpts" => per_inch / 72.,
            "picas" => per_inch * 12. / 72.27,
            "dida" => per_inch * (1238. / 1157.) / 72.27,
            "cicero" => per_inch * 12. * (1238. / 1157.) / 72.27,
            "scaledpts" => per_inch / 65536. / 72.27,
            "mm" => per_inch / 25.4,
            "cm" => per_inch / 2.54,
            "inches" | "in" => per_inch,
            "lines" => font_size * self.number("text", "lineheight").unwrap_or(1.2),
            "char" => font_size,
            "npc" | "snpc" => extent,
            "native" | "null" => {
                return Err(super::elements::invalid(
                    "Theme native/null lengths require an explicit coordinate or allocation context.",
                ));
            }
            _ => return Err(super::elements::invalid("Unsupported theme length unit.")),
        };
        Ok(Some(value * factor))
    }
}
impl ThemeSpec {
    /// Resolve a hierarchy once; absence preserves the existing presentation cascade.
    pub fn resolved_elements(&self) -> ChartResult<Option<ResolvedElements>> {
        let mut resolved = self
            .hierarchy
            .as_ref()
            .map(|theme| {
                if theme.complete {
                    ResolvedElements::new(theme)
                } else {
                    ResolvedElements::new(
                        &ElementTheme::preset(super::ThemePreset::Grey)?.update(theme)?,
                    )
                }
            })
            .transpose()?;
        if let Some(elements) = &mut resolved {
            elements.fonts.clone_from(&self.fonts);
            elements.overrides = self.plot.clone();
        }
        Ok(resolved)
    }
    /// Resolve reference geometry defaults, followed by explicit geometry overrides.
    pub fn geometry_defaults(&self) -> ChartResult<GeometryTheme<Paint>> {
        if let Some(geometry) = &self.geometry {
            return Ok(geometry.clone());
        }
        let mut result = GeometryTheme::default();
        if let Some(elements) = self.resolved_elements()? {
            if let Some(v) = elements.paint("geom", "ink")? {
                result.ink = v;
            }
            if let Some(v) = elements.paint("geom", "paper")? {
                result.paper = v;
            }
            if let Some(v) = elements.paint("geom", "accent")? {
                result.accent = v;
            }
            if let Some(v) = elements.number("geom", "pointsize") {
                result.point_size = v;
            }
            if let Some(v) = elements.number("geom", "linewidth") {
                result.line_width = v;
            }
        }
        result.validate()?;
        Ok(result)
    }
}
