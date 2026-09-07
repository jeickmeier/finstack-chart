//! Annotation edits operate on authored anchors and never mutate source observations.
use crate::composition::{Anchor, Annotation, ScaleValue};
use crate::layout::{LaidOutChart, ResolvedAxis, ResolvedScale};
use crate::{ChartResult, Diagnostic, DiagnosticCode, Point, SceneStamp};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Explicit editable part. Callout endpoints provide threshold/range handles without new data rows.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnnotationPart {
    /// Move the label's anchor; its callout endpoint stays fixed.
    Anchor,
    /// Move the callout endpoint; reject annotations without one.
    Callout,
    /// Move both endpoints by the same destination displacement (use Y-only for a threshold).
    Translate,
}
/// Constraints use the anchor's declared coordinate units. No implicit time-unit conversion.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ValueConstraint {
    /// Numeric calculation units, or relative/output units for non-data anchors.
    Number {
        /// Inclusive finite ordered interval, if constrained.
        bounds: Option<[f64; 2]>,
        /// Positive finite snap spacing; ties round toward the greater grid coordinate.
        step: Option<f64>,
        /// Grid origin in the same units.
        origin: f64,
    },
    /// Exact source timestamps. Snap arithmetic uses wide integers.
    Timestamp {
        /// Inclusive source-tick interval, if constrained.
        #[serde(with = "time_bounds")]
        bounds: Option<[i64; 2]>,
        /// Positive source-tick spacing.
        #[serde(with = "time_step")]
        step: Option<i64>,
        /// Exact grid origin.
        #[serde(with = "crate::portable::signed")]
        origin: i64,
    },
    /// Allowed decoded categories. Missing/forbidden categories reject; no ordinal substitution.
    Category(Vec<String>),
}
/// Movement and axis-value constraints, validated before gesture ownership begins.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EditConstraints {
    /// Allow horizontal changes.
    pub horizontal: bool,
    /// Allow vertical changes.
    pub vertical: bool,
    /// Horizontal value constraint.
    pub x: Option<ValueConstraint>,
    /// Vertical value constraint.
    pub y: Option<ValueConstraint>,
    /// Preserve the current anchor/callout ordering in the named dimension.
    /// None permits crossing; Some(true) is horizontal, Some(false) is vertical.
    pub preserve_order: Option<bool>,
}
impl Default for EditConstraints {
    fn default() -> Self {
        Self {
            horizontal: true,
            vertical: true,
            x: None,
            y: None,
            preserve_order: None,
        }
    }
}
fn apply(value: ScaleValue, constraint: Option<&ValueConstraint>) -> ChartResult<ScaleValue> {
    Ok(match (value, constraint) {
        (v, None) => v,
        (
            ScaleValue::Number(mut v),
            Some(ValueConstraint::Number {
                bounds,
                step,
                origin,
            }),
        ) => {
            if !origin.is_finite()
                || step.is_some_and(|s| !s.is_finite() || s <= 0.)
                || bounds.is_some_and(|b| !b[0].is_finite() || !b[1].is_finite() || b[0] > b[1])
            {
                return Err(invalid("Numeric edit constraint is invalid."));
            }
            if let Some(step) = step {
                v = (((v - origin) / step) + 0.5)
                    .floor()
                    .mul_add(*step, *origin);
            }
            if let Some([lo, hi]) = bounds {
                v = v.clamp(*lo, *hi);
            }
            if !v.is_finite() {
                return Err(invalid("Snapped coordinate is not representable."));
            }
            ScaleValue::Number(v)
        }
        (
            ScaleValue::Timestamp { mut value, unit },
            Some(ValueConstraint::Timestamp {
                bounds,
                step,
                origin,
            }),
        ) => {
            if step.is_some_and(|s| s <= 0) || bounds.is_some_and(|b| b[0] > b[1]) {
                return Err(invalid("Timestamp edit constraint is invalid."));
            }
            if let Some(step) = step {
                let step = i128::from(*step);
                let d = i128::from(value) - i128::from(*origin);
                let q = d.div_euclid(step) + i128::from(d.rem_euclid(step) * 2 >= step);
                value = i64::try_from(i128::from(*origin) + q * step)
                    .map_err(|_| invalid("Snapped timestamp overflows source ticks."))?;
            }
            if let Some([lo, hi]) = bounds {
                value = value.clamp(*lo, *hi);
            }
            ScaleValue::Timestamp { value, unit }
        }
        (ScaleValue::Category(v), Some(ValueConstraint::Category(allowed))) => {
            if allowed.is_empty()
                || allowed.len() > 4096
                || allowed.iter().any(|s| s.len() > 4096)
                || !allowed.contains(&v)
            {
                return Err(invalid(
                    "Category is absent from the explicit edit constraint.",
                ));
            }
            ScaleValue::Category(v)
        }
        _ => {
            return Err(invalid(
                "Edit constraint type does not match its anchor coordinate.",
            ));
        }
    })
}
fn lookup(axis: &ResolvedAxis, p: f64) -> ChartResult<ScaleValue> {
    let category = match &axis.scale {
        ResolvedScale::Band(s) => Some(s.category_at(p)?),
        ResolvedScale::Point(s) => Some(s.category_at(p)?),
        _ => None,
    };
    match category {
        Some(Some(s)) => Ok(ScaleValue::Category(s.into())),
        Some(None) => Err(invalid(
            "Pointer has no category in the presented scale window.",
        )),
        None => axis.invert_value(p),
    }
}
fn parent<'a>(
    chart: &'a LaidOutChart,
    panel: &Option<crate::grammar::PanelKey>,
) -> ChartResult<&'a LaidOutChart> {
    match panel {
        Some(k) => chart
            .panels()
            .iter()
            .find(|p| &p.key == k)
            .map(|p| p.chart.as_ref())
            .ok_or_else(|| invalid("Edited annotation panel is absent.")),
        None if chart.panels().is_empty() => Ok(chart),
        None => Err(invalid("A faceted anchor requires its panel identity.")),
    }
}
fn move_anchor(
    chart: &LaidOutChart,
    anchor: &Anchor,
    dx: f64,
    dy: f64,
    c: &EditConstraints,
) -> ChartResult<Anchor> {
    let (start, clip) = chart
        .project_anchor(anchor)?
        .ok_or_else(|| invalid("Edited annotation has no visible coordinate basis."))?;
    let p = Point::new(
        start.x() + if c.horizontal { dx } else { 0. },
        start.y() + if c.vertical { dy } else { 0. },
    )?;
    let (x, y) = match anchor {
        Anchor::Data {
            panel,
            scales,
            x,
            y,
        } => {
            let chart = parent(chart, panel)?;
            let axis = |id| {
                chart
                    .axes()
                    .get(&id)
                    .ok_or_else(|| invalid("Edited annotation axis is absent."))
            };
            (
                if c.horizontal {
                    lookup(axis(scales.x)?, p.x())?
                } else {
                    x.clone()
                },
                if c.vertical {
                    lookup(axis(scales.y)?, p.y())?
                } else {
                    y.clone()
                },
            )
        }
        Anchor::Panel { .. } | Anchor::Figure { .. } => (
            ScaleValue::Number(((p.x() - clip.origin().x()) / clip.width()).clamp(0., 1.)),
            ScaleValue::Number(((p.y() - clip.origin().y()) / clip.height()).clamp(0., 1.)),
        ),
        Anchor::Output { .. } => (
            ScaleValue::Number(p.x() - clip.origin().x()),
            ScaleValue::Number(p.y() - clip.origin().y()),
        ),
    };
    let x = if c.horizontal {
        apply(x, c.x.as_ref())?
    } else {
        x
    };
    let y = if c.vertical {
        apply(y, c.y.as_ref())?
    } else {
        y
    };
    let result = match anchor {
        Anchor::Data { panel, scales, .. } => Anchor::Data {
            panel: panel.clone(),
            scales: *scales,
            x,
            y,
        },
        _ => {
            let (ScaleValue::Number(x), ScaleValue::Number(y)) = (x, y) else {
                return Err(invalid(
                    "Relative/output annotation coordinates must remain numeric.",
                ));
            };
            match anchor {
                Anchor::Panel { panel, .. } => Anchor::Panel {
                    panel: panel.clone(),
                    x,
                    y,
                },
                Anchor::Figure { .. } => Anchor::Figure { x, y },
                _ => Anchor::Output { x, y },
            }
        }
    };
    chart
        .project_anchor(&result)?
        .ok_or_else(|| invalid("Constrained anchor is omitted by its scale policy."))?;
    Ok(result)
}
/// One cheap pinned edit producer. The reducer separately owns begin/preview/commit/cancel/history.
#[derive(Clone, Debug)]
pub struct AnnotationEditor {
    chart: Arc<LaidOutChart>,
    original: Annotation,
    part: AnnotationPart,
    constraints: EditConstraints,
}
impl AnnotationEditor {
    /// Capture the effective displayed annotation; no source values or reducers are mutated.
    pub fn new(
        chart: Arc<LaidOutChart>,
        id: &str,
        part: AnnotationPart,
        constraints: EditConstraints,
    ) -> ChartResult<Self> {
        if !constraints.horizontal && !constraints.vertical {
            return Err(invalid(
                "An edit must enable at least one movement dimension.",
            ));
        }
        let original = chart
            .prepared()
            .state()
            .annotations(chart.prepared().definition())
            .into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| {
                invalid("Edited annotation is absent from the presented definition/state.")
            })?;
        if part != AnnotationPart::Anchor && original.callout.is_none() {
            return Err(invalid(
                "Callout/translate editing requires a second endpoint.",
            ));
        }
        let result = Self {
            chart,
            original,
            part,
            constraints,
        };
        result.preview(result.chart.scene().stamp(), 0., 0.)?;
        Ok(result)
    }
    /// Captured annotation identity and values.
    pub fn original(&self) -> &Annotation {
        &self.original
    }
    /// Exact retained scene and resources, released when the editor is dropped.
    pub fn presented(&self) -> &Arc<LaidOutChart> {
        &self.chart
    }
    /// Keyboard-equivalent movement: one snap spacing/category, or one destination unit when unsnapped.
    /// Forward means right/down in the presented frame; reversed axes keep their declared direction.
    pub fn nudge(
        &self,
        stamp: SceneStamp,
        horizontal: bool,
        forward: bool,
        steps: u32,
    ) -> ChartResult<Annotation> {
        if steps == 0 || steps > 100 {
            return Err(invalid("Keyboard edits require 1..100 steps."));
        }
        let anchor = if self.part == AnnotationPart::Callout {
            self.original.callout.as_ref().expect("validated endpoint")
        } else {
            &self.original.anchor
        };
        let (p, _) = self
            .chart
            .project_anchor(anchor)?
            .ok_or_else(|| invalid("Keyboard handle has no coordinate basis."))?;
        let constraint = if horizontal {
            self.constraints.x.as_ref()
        } else {
            self.constraints.y.as_ref()
        };
        let mut probe = anchor.clone();
        let value = match &mut probe {
            Anchor::Data { x, y, .. } => {
                if horizontal {
                    x
                } else {
                    y
                }
            }
            _ => {
                let spacing = if let Some(ValueConstraint::Number {
                    step: Some(step), ..
                }) = constraint
                {
                    let (_, clip) = self.chart.project_anchor(anchor)?.expect("resolved anchor");
                    match anchor {
                        Anchor::Figure { .. } | Anchor::Panel { .. } => {
                            step * if horizontal {
                                clip.width()
                            } else {
                                clip.height()
                            }
                        }
                        _ => *step,
                    }
                } else {
                    1.
                };
                let delta = spacing * f64::from(steps) * if forward { 1. } else { -1. };
                return self.preview(
                    stamp,
                    if horizontal { delta } else { 0. },
                    if horizontal { 0. } else { delta },
                );
            }
        };
        let changed = match (&mut *value, constraint) {
            (
                ScaleValue::Number(v),
                Some(ValueConstraint::Number {
                    step: Some(step), ..
                }),
            ) => {
                *v += step;
                true
            }
            (
                ScaleValue::Timestamp { value, .. },
                Some(ValueConstraint::Timestamp {
                    step: Some(step), ..
                }),
            ) => {
                *value = value
                    .checked_add(*step)
                    .ok_or_else(|| invalid("Keyboard timestamp step overflows."))?;
                true
            }
            (ScaleValue::Category(label), _) => {
                let Anchor::Data { panel, scales, .. } = anchor else {
                    unreachable!()
                };
                let axis = parent(&self.chart, panel)?
                    .axes()
                    .get(&if horizontal { scales.x } else { scales.y })
                    .ok_or_else(|| invalid("Keyboard category axis is absent."))?;
                let labels = match &axis.scale {
                    ResolvedScale::Band(s) => s.visible_domain(),
                    ResolvedScale::Point(s) => s.visible_domain(),
                    _ => return Err(invalid("Keyboard category needs a category axis.")),
                };
                let i = labels
                    .iter()
                    .position(|s| s == label)
                    .ok_or_else(|| invalid("Keyboard category is absent."))?;
                if labels.len() < 2 {
                    return Ok(self.original.clone());
                }
                *label = labels[if i + 1 < labels.len() { i + 1 } else { i - 1 }].clone();
                true
            }
            _ => false,
        };
        let spacing = if changed {
            let q = self
                .chart
                .project_anchor(&probe)?
                .ok_or_else(|| invalid("Keyboard snap spacing cannot be projected."))?
                .0;
            if horizontal {
                (q.x() - p.x()).abs()
            } else {
                (q.y() - p.y()).abs()
            }
        } else {
            1.
        };
        let delta = spacing * f64::from(steps) * if forward { 1. } else { -1. };
        self.preview(
            stamp,
            if horizontal { delta } else { 0. },
            if horizontal { 0. } else { delta },
        )
    }
    /// Proposed annotation from total displacement since begin, never accumulated preview deltas.
    pub fn preview(&self, stamp: SceneStamp, dx: f64, dy: f64) -> ChartResult<Annotation> {
        if stamp != self.chart.scene().stamp() {
            return Err(Diagnostic::error(
                DiagnosticCode::Superseded,
                "Annotation edit has another scene basis.",
                "Keep the original presented stamp through commit/cancel.",
            ));
        }
        if !dx.is_finite() || !dy.is_finite() {
            return Err(invalid("Edit displacement must be finite."));
        }
        let mut next = self.original.clone();
        if self.part != AnnotationPart::Callout {
            next.anchor = move_anchor(&self.chart, &next.anchor, dx, dy, &self.constraints)?;
        }
        if self.part != AnnotationPart::Anchor {
            next.callout = Some(move_anchor(
                &self.chart,
                next.callout.as_ref().expect("validated endpoint"),
                dx,
                dy,
                &self.constraints,
            )?);
        }
        if let Some(horizontal) = self.constraints.preserve_order {
            let endpoints =
                |a: &Annotation| -> ChartResult<f64> {
                    let first = self
                        .chart
                        .project_anchor(&a.anchor)?
                        .ok_or_else(|| invalid("Range anchor is not visible."))?
                        .0;
                    let last =
                        self.chart
                            .project_anchor(a.callout.as_ref().ok_or_else(|| {
                                invalid("Range ordering requires two endpoints.")
                            })?)?
                            .ok_or_else(|| invalid("Range endpoint is not visible."))?
                            .0;
                    Ok(if horizontal {
                        last.x() - first.x()
                    } else {
                        last.y() - first.y()
                    })
                };
            let before = endpoints(&self.original)?;
            let after = endpoints(&next)?;
            if before == 0. || after == 0. || before.is_sign_positive() != after.is_sign_positive()
            {
                return Err(invalid(
                    "Range handles cannot cross or collapse under this constraint.",
                ));
            }
        }
        Ok(next)
    }
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Use a visible authored anchor, typed bounded constraints and a pinned presented basis.",
    )
}

mod time_step {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(v: &Option<i64>, s: S) -> Result<S::Ok, S::Error> {
        v.map(|v| v.to_string()).serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
        Option::<String>::deserialize(d)?
            .map(|v| v.parse().map_err(serde::de::Error::custom))
            .transpose()
    }
}
mod time_bounds {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S: Serializer>(v: &Option<[i64; 2]>, s: S) -> Result<S::Ok, S::Error> {
        v.map(|v| v.map(|v| v.to_string())).serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<[i64; 2]>, D::Error> {
        Option::<[String; 2]>::deserialize(d)?
            .map(|[a, b]| {
                Ok([
                    a.parse().map_err(serde::de::Error::custom)?,
                    b.parse().map_err(serde::de::Error::custom)?,
                ])
            })
            .transpose()
    }
}
