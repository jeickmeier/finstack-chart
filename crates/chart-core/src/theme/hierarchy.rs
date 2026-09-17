//! One bounded inheritance graph and explicit context for reference themes.
use super::elements::*;
use crate::ChartResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Deserialize)]
struct Node {
    classes: Vec<String>,
    parents: Vec<String>,
}
#[derive(Deserialize)]
struct Reference {
    tree: BTreeMap<String, Node>,
    presets: BTreeMap<String, BTreeMap<String, ThemeEntry>>,
}
fn reference() -> ChartResult<Reference> {
    serde_json::from_str(include_str!("reference_data.json"))
        .map_err(|_| invalid("Invalid bundled theme reference data."))
}
/// Complete reference preset. Gray is an authoring alias of Grey.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    /// Grey panel.
    Grey,
    /// White panel with borders.
    Bw,
    /// Fine monochrome lines.
    Linedraw,
    /// Light furniture.
    Light,
    /// Dark panels.
    Dark,
    /// Minimal furniture.
    Minimal,
    /// Classic axes.
    Classic,
    /// No axes or panels.
    Void,
    /// Reference test theme.
    Test,
}
impl ThemePreset {
    fn name(self) -> &'static str {
        match self {
            Self::Grey => "grey",
            Self::Bw => "bw",
            Self::Linedraw => "linedraw",
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Minimal => "minimal",
            Self::Classic => "classic",
            Self::Void => "void",
            Self::Test => "test",
        }
    }
}
impl ElementTheme {
    /// Select a complete reference preset.
    pub fn preset(preset: ThemePreset) -> ChartResult<Self> {
        let data = reference()?;
        Ok(Self {
            complete: true,
            elements: data.presets[preset.name()].clone(),
        })
    }
    /// Add one node, validating its element/property schema.
    pub fn element(mut self, name: impl Into<String>, entry: ThemeEntry) -> ChartResult<Self> {
        self.elements.insert(name.into(), entry);
        self.validate()?;
        Ok(self)
    }
    /// Check node/class/property and collection budgets without rendering.
    pub fn validate(&self) -> ChartResult<()> {
        let data = reference()?;
        self.validate_with(&data)
    }
    fn validate_with(&self, data: &Reference) -> ChartResult<()> {
        if self.elements.len() > 256 {
            return Err(invalid("Theme element budget exceeds256."));
        }
        for (name, entry) in &self.elements {
            let node = data
                .tree
                .get(name)
                .ok_or_else(|| invalid(format!("Unknown theme element {name}.")))?;
            let valid = match entry {
                ThemeEntry::Blank => node.classes.iter().any(|c| c.contains("element_")),
                ThemeEntry::Element(e) => {
                    e.validate()?;
                    let class = format!("element_{}", format!("{:?}", e.kind).to_lowercase());
                    node.classes.contains(&class)
                }
                ThemeEntry::Value(v) => {
                    v.validate(0)?;
                    match v {
                        ThemeValue::Arrow(_) => false,
                        ThemeValue::Missing => true,
                        ThemeValue::Number(_) => node.classes.iter().any(|s| s == "numeric"),
                        ThemeValue::Relative(_) => {
                            node.classes.iter().any(|s| s == "numeric" || s == "rel")
                        }
                        ThemeValue::Text(_) => node.classes.iter().any(|s| s == "character"),
                        ThemeValue::Bool(_) => node.classes.iter().any(|s| s == "logical"),
                        ThemeValue::Unit(_) | ThemeValue::Margin(_) => {
                            node.classes.iter().any(|s| s == "unit" || s == "margin")
                        }
                        ThemeValue::Vector(_) => node
                            .classes
                            .iter()
                            .any(|s| s == "numeric" || s == "character" || s == "unit"),
                    }
                }
            };
            if !valid {
                return Err(invalid(format!("Wrong class for theme element {name}.")));
            }
        }
        Ok(())
    }
    /// Resolve one node, applying every ordered parent and explicit blank policy.
    pub fn resolve_element(&self, name: &str, skip_blank: bool) -> ChartResult<Option<ThemeEntry>> {
        let data = reference()?;
        self.validate_with(&data)?;
        resolve(self, &data, name, skip_blank, 0)
    }
    /// Resolve the entire graph once for consumers; no process-global cache or context.
    pub fn resolve_all(&self) -> ChartResult<BTreeMap<String, Option<ThemeEntry>>> {
        let data = reference()?;
        self.validate_with(&data)?;
        data.tree
            .keys()
            .map(|n| Ok((n.clone(), resolve(self, &data, n, false, 0)?)))
            .collect()
    }
    /// Merge authored properties within each element; a complete right theme replaces all nodes.
    pub fn update(&self, next: &Self) -> ChartResult<Self> {
        self.compose(next, false)
    }
    /// Replace each authored element as a whole; omitted properties resume inheritance.
    pub fn replace(&self, next: &Self) -> ChartResult<Self> {
        self.compose(next, true)
    }
    fn compose(&self, next: &Self, replace: bool) -> ChartResult<Self> {
        next.validate()?;
        if next.complete {
            return Ok(next.clone());
        }
        let mut out = self.clone();
        for (name, value) in &next.elements {
            let merged = if !replace {
                match (out.elements.get(name), value) {
                    (Some(ThemeEntry::Element(old)), ThemeEntry::Element(new)) => {
                        if old.kind != new.kind {
                            return Err(invalid("Only matching element classes can merge."));
                        }
                        let mut e = old.clone();
                        e.properties.extend(new.properties.clone());
                        ThemeEntry::Element(e)
                    }
                    _ => value.clone(),
                }
            } else {
                value.clone()
            };
            out.elements.insert(name.clone(), merged);
        }
        out.validate()?;
        Ok(out)
    }
    /// Expand a reference subtheme suffix map through the same node validator.
    pub fn subtheme(prefix: &str, entries: BTreeMap<String, ThemeEntry>) -> ChartResult<Self> {
        let (prefix, suffix) = match prefix {
            "axis" => ("axis.", ""),
            "axis_x" => ("axis.", ".x"),
            "axis_y" => ("axis.", ".y"),
            "axis_top" => ("axis.", ".x.top"),
            "axis_bottom" => ("axis.", ".x.bottom"),
            "axis_left" => ("axis.", ".y.left"),
            "axis_right" => ("axis.", ".y.right"),
            "legend" => ("legend.", ""),
            "strip" => ("strip.", ""),
            "plot" => ("plot.", ""),
            "panel" => ("panel.", ""),
            _ => return Err(invalid("Unknown theme subtheme.")),
        };
        let out = Self {
            complete: false,
            elements: entries
                .into_iter()
                .map(|(name, value)| (format!("{prefix}{name}{suffix}"), value))
                .collect(),
        };
        out.validate()?;
        Ok(out)
    }
}
fn resolve(
    theme: &ElementTheme,
    data: &Reference,
    name: &str,
    skip_blank: bool,
    depth: usize,
) -> ChartResult<Option<ThemeEntry>> {
    if depth > 32 {
        return Err(invalid("Theme inheritance depth exceeds32."));
    }
    let Some(node) = data.tree.get(name) else {
        return Ok(None);
    };
    let mut child = theme.elements.get(name).cloned();
    if matches!(child, Some(ThemeEntry::Blank)) {
        if skip_blank {
            child = None;
        } else {
            return Ok(child);
        }
    }
    if node.parents.is_empty() {
        if let Some(ThemeEntry::Element(_)) = &child {
            child = combine(child, data.presets["grey"].get(name).cloned())?;
        }
        return Ok(child);
    }
    let skip=skip_blank||child.as_ref().is_some_and(|e|!matches!(e,ThemeEntry::Element(e) if e.properties.get("inherit.blank")==Some(&ThemeValue::Bool(true))));
    for parent in &node.parents {
        child = combine(child, resolve(theme, data, parent, skip, depth + 1)?)?;
    }
    Ok(child)
}
fn value(child: ThemeValue, parent: Option<&ThemeValue>) -> ThemeValue {
    match (child, parent) {
        (ThemeValue::Relative(a), Some(ThemeValue::Relative(b))) => ThemeValue::Relative(a * b),
        (ThemeValue::Relative(a), Some(ThemeValue::Number(b))) => ThemeValue::Number(a * b),
        (ThemeValue::Relative(a), Some(ThemeValue::Unit(b))) => ThemeValue::Unit(
            b.iter()
                .map(|n| ThemeLength {
                    value: n.value.map(|n| n * a),
                    unit: n.unit.clone(),
                })
                .collect(),
        ),
        (ThemeValue::Margin(mut a), Some(ThemeValue::Margin(b))) => {
            for (n, p) in a.iter_mut().zip(b) {
                if n.value.is_none() {
                    *n = if p.value.is_none() {
                        ThemeLength {
                            value: Some(0.),
                            unit: "points".into(),
                        }
                    } else {
                        p.clone()
                    };
                }
            }
            ThemeValue::Margin(a)
        }
        (a, _) => a,
    }
}
fn combine(
    child: Option<ThemeEntry>,
    parent: Option<ThemeEntry>,
) -> ChartResult<Option<ThemeEntry>> {
    let Some(parent) = parent else {
        return Ok(child);
    };
    let Some(child) = child else {
        return Ok(Some(parent));
    };
    Ok(Some(match (child, parent) {
        (ThemeEntry::Blank, _) => ThemeEntry::Blank,
        (ThemeEntry::Value(a), ThemeEntry::Value(b)) => ThemeEntry::Value(value(a, Some(&b))),
        (ThemeEntry::Element(e), ThemeEntry::Blank) => {
            if e.properties.get("inherit.blank") == Some(&ThemeValue::Bool(true)) {
                ThemeEntry::Blank
            } else {
                ThemeEntry::Element(e)
            }
        }
        (ThemeEntry::Element(mut a), ThemeEntry::Element(b)) => {
            if a.kind != b.kind {
                return Err(invalid("Incompatible inherited theme element classes."));
            }
            for name in ["size", "linewidth", "margin"] {
                if let Some(v) = a.properties.get(name).cloned() {
                    a.properties
                        .insert(name.into(), value(v, b.properties.get(name)));
                }
            }
            for (n, v) in b.properties {
                a.properties.entry(n).or_insert(v);
            }
            ThemeEntry::Element(a)
        }
        (a, _) => a,
    }))
}
/// Isolated authoring context; published themes are immutable snapshots.
#[derive(Clone, Debug)]
pub struct ThemeContext {
    theme: ElementTheme,
}
impl ThemeContext {
    /// Start an explicit context with a validated theme.
    pub fn new(theme: ElementTheme) -> ChartResult<Self> {
        theme.validate()?;
        Ok(Self { theme })
    }
    /// Get an independent snapshot.
    pub fn get(&self) -> ElementTheme {
        self.theme.clone()
    }
    /// Replace the whole context and return its previous theme.
    pub fn set(&mut self, theme: ElementTheme) -> ChartResult<ElementTheme> {
        theme.validate()?;
        Ok(std::mem::replace(&mut self.theme, theme))
    }
    /// Merge authored element properties into this context.
    pub fn update(&mut self, theme: &ElementTheme) -> ChartResult<()> {
        self.theme = self.theme.update(theme)?;
        Ok(())
    }
    /// Replace authored elements as a whole in this context.
    pub fn replace(&mut self, theme: &ElementTheme) -> ChartResult<()> {
        self.theme = self.theme.replace(theme)?;
        Ok(())
    }
}
