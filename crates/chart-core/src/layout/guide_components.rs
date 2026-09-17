//! Resolved guide component styles shared by every destination.
use crate::{
    ChartResult, DiagnosticCode, Limits,
    color::Paint,
    scene::{Color, Stroke},
};

/// Optional line properties inherit the corresponding whole-guide component.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GuideLineStyle {
    /// Optional shared physical arrow heads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow: Option<crate::grammar::ArrowSpec>,
    /// Independent arrow-head fill, inheriting the line color when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_fill: Option<Paint>,
    /// Optional portable stroke endpoint policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<crate::grammar::LineEnd>,
    /// Optional portable stroke corner policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_join: Option<crate::grammar::LineJoin>,
    /// Omit this component when false; selection and scale mapping are unchanged.
    pub visible: Option<bool>,
    /// Authored paint, resolved by the common color owner.
    pub color: Option<Paint>,
    /// Positive finite width in destination units.
    pub width: Option<f64>,
    /// Even positive alternating on/off lengths; an empty list resets to solid.
    pub dashes: Option<Vec<f64>>,
}
/// Optional label properties; rich runs retain their explicit font resource and weight.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GuideTextStyle {
    /// Optional horizontal justification relative to the tick anchor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hjust: Option<f64>,
    /// Optional vertical justification in reference bottom-to-top coordinates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vjust: Option<f64>,
    /// Hide this label without removing its semantic tick or line.
    pub visible: Option<bool>,
    /// Override the label paint, including rich runs.
    pub color: Option<Paint>,
    /// Positive destination base size; rich-run size remains a multiplier.
    pub font_size: Option<f64>,
    /// Existing rich-run font/fallback/weight/shaping policy; its text is replaced by the tick label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typography: Option<crate::typography::RichRun>,
    /// Finite clockwise label rotation; absent inherits the guide rotation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f64>,
}
/// An override for one occurrence in the original selected tick list.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GuideTickStyle {
    /// Zero-based selection index, before adaptive presentation.
    pub index: usize,
    /// Hide line and label together without changing selection.
    pub visible: Option<bool>,
    /// Independent line overrides.
    pub line: GuideLineStyle,
    /// Independent label overrides.
    pub label: GuideTextStyle,
}
/// Whole-guide components and bounded occurrence-specific overrides (wire v14).
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GuideComponents {
    /// Domain path, including its end caps.
    pub domain: GuideLineStyle,
    /// Default tick-line properties.
    pub ticks: GuideLineStyle,
    /// Default label properties.
    pub labels: GuideTextStyle,
    /// At most one override per original selection index.
    pub per_tick: Vec<GuideTickStyle>,
}
impl GuideLineStyle {
    fn validate(&self) -> ChartResult<()> {
        if let Some(a) = &self.arrow {
            crate::theme::ThemeElement::new(crate::theme::ElementKind::Line)
                .property("arrow", crate::theme::ThemeValue::Arrow(a.clone()))?;
        }
        if let Some(width) = self.width {
            crate::geometry::positive(width, "Guide stroke width must be finite and positive.")?;
        }
        if let Some(dashes) = &self.dashes
            && !dashes.is_empty()
        {
            // Reuse the common checked dash grammar without allocating a path.
            crate::scene::dash_polyline(&[], dashes, 0)?;
        }
        Ok(())
    }
    pub(super) fn overlay(&self, local: Option<&Self>) -> Self {
        let Some(local) = local else {
            return self.clone();
        };
        Self {
            arrow: local.arrow.clone().or_else(|| self.arrow.clone()),
            arrow_fill: local.arrow_fill.or(self.arrow_fill),
            line_end: local.line_end.or(self.line_end),
            line_join: local.line_join.or(self.line_join),
            visible: local.visible.or(self.visible),
            color: local.color.or(self.color),
            width: local.width.or(self.width),
            dashes: local.dashes.clone().or_else(|| self.dashes.clone()),
        }
    }
    pub(super) fn stroke(&self, color: Color) -> Stroke {
        Stroke {
            color: self.color.map(Paint::resolve).unwrap_or(color),
            width: self.width.unwrap_or(1.),
        }
    }
}
impl GuideTextStyle {
    fn validate(&self, limits: Limits) -> ChartResult<()> {
        if [self.hjust, self.vjust]
            .into_iter()
            .flatten()
            .any(|v| !v.is_finite())
        {
            return Err(crate::scales::error(
                DiagnosticCode::Validation,
                "Guide justification must be finite.",
            ));
        }
        if let Some(run) = &self.typography {
            run.validate(limits)?;
        }
        if self.rotation.is_some_and(|angle| !angle.is_finite()) {
            return Err(crate::scales::error(
                DiagnosticCode::Validation,
                "Guide rotation must be finite.",
            ));
        }
        if let Some(size) = self.font_size {
            crate::geometry::positive(size, "Guide font size must be finite and positive.")?;
        }
        Ok(())
    }
    fn overlay(&self, local: Option<&Self>) -> Self {
        let Some(local) = local else {
            return self.clone();
        };
        Self {
            hjust: local.hjust.or(self.hjust),
            vjust: local.vjust.or(self.vjust),
            visible: local.visible.or(self.visible),
            color: local.color.or(self.color),
            font_size: local.font_size.or(self.font_size),
            typography: local.typography.clone().or_else(|| self.typography.clone()),
            rotation: local.rotation.or(self.rotation),
        }
    }
}
impl GuideComponents {
    pub(super) fn validate(&self, limits: Limits) -> ChartResult<()> {
        self.domain.validate()?;
        self.ticks.validate()?;
        self.labels.validate(limits)?;
        crate::limits::require_within(
            self.per_tick.len() <= limits.max_items,
            "guide style override",
        )?;
        let mut seen = std::collections::BTreeSet::new();
        for style in &self.per_tick {
            crate::limits::require_within(
                style.index < limits.max_items,
                "guide style selection index",
            )?;
            if !seen.insert(style.index) {
                return Err(crate::scales::error(
                    DiagnosticCode::Validation,
                    "A guide tick has duplicate style overrides.",
                ));
            }
            style.line.validate()?;
            style.label.validate(limits)?;
        }
        Ok(())
    }
    pub(super) fn overrides(&self) -> std::collections::BTreeMap<usize, &GuideTickStyle> {
        self.per_tick.iter().map(|s| (s.index, s)).collect()
    }
    pub(super) fn tick(
        &self,
        local: Option<&GuideTickStyle>,
    ) -> (bool, GuideLineStyle, GuideTextStyle) {
        (
            local.and_then(|s| s.visible).unwrap_or(true),
            self.ticks.overlay(local.map(|s| &s.line)),
            self.labels.overlay(local.map(|s| &s.label)),
        )
    }
}
