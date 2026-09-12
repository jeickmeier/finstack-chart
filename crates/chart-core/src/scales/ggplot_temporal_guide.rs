//! Timestamp aesthetic guides reuse the calendar selectors and shared label contracts.
use super::*;
use crate::{ChartResult, DiagnosticCode, data::TimeUnit, interpolate::Number};

/// Reference temporal break selection, before guide visibility is applied.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum GgplotTemporalBreaks {
    /// Select reference pretty Date/datetime candidates.
    #[default]
    Automatic,
    /// R's null breaks, including on constant or nonfinite domains.
    None,
    /// Origin-relative candidates in the source timestamp unit; NaN is missing.
    Explicit(Vec<Number>),
    /// Reference date_breaks character width, using the common calendar owner.
    Width(String),
}
/// Reference Date/datetime arguments, separate from the timestamp representation.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GgplotTemporalGuideArguments {
    /// Date selection and formatting; otherwise use datetime semantics.
    pub date: bool,
    /// Candidate selection; an explicit empty vector differs from null breaks.
    pub breaks: GgplotTemporalBreaks,
    /// Approximate pretty count; explicit candidates and widths take precedence.
    pub count: Option<f64>,
    /// Vector labels before candidate visibility, shared with continuous guides.
    pub labels: GgplotGuideLabels,
    /// Explicit date_labels pattern, overriding the label argument as in R.
    pub format: Option<Box<GgplotTimeFormat>>,
}
impl GgplotTemporalGuideArguments {
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}
/// A temporal guide over numeric offsets from an exact timestamp origin.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GgplotTemporalGuide {
    /// Exact origin, in the input timestamp unit.
    #[serde(with = "crate::portable::signed")]
    pub origin: i64,
    /// Unit shared by the timestamp input and numeric offsets.
    pub unit: TimeUnit,
    /// Explicit calendar rules, independent of machine timezone.
    pub zone: CalendarZone,
    /// Optional reference selection and label controls.
    #[serde(
        default,
        skip_serializing_if = "GgplotTemporalGuideArguments::is_default"
    )]
    pub arguments: GgplotTemporalGuideArguments,
}
impl GgplotTemporalGuide {
    pub(super) fn validate(&self) -> ChartResult<()> {
        let calendar = Calendar::new(self.zone.clone())?;
        if self.arguments.date && self.zone != CalendarZone::Utc {
            return Err(error(
                DiagnosticCode::Validation,
                "Date guides require UTC calendar rules.",
            ));
        }
        if let Some(format) = &self.arguments.format {
            format.prepare(calendar)?;
        } else {
            self.arguments
                .labels
                .validate(match &self.arguments.breaks {
                    GgplotTemporalBreaks::Explicit(values) => Some(values.len()),
                    _ => None,
                })?;
        }
        if self.arguments.count.is_some_and(|n| !n.is_finite()) {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Temporal guide count must be finite.",
            ));
        }
        Ok(())
    }
    pub(super) fn resolve(
        &self,
        bounds: [f64; 2],
        budget: usize,
        label_budget: usize,
    ) -> ChartResult<Vec<GgplotContinuousGuideEntry>> {
        self.validate()?;
        if matches!(self.arguments.breaks, GgplotTemporalBreaks::None) {
            return Ok(vec![]);
        }
        let calendar = Calendar::new(self.zone.clone())?;
        let normalization = GgplotTimestampNormalization {
            origin: self.origin,
            unit: self.unit,
            date: self.arguments.date,
        };
        let constant = bounds[0].is_finite()
            && super::ggplot::zero_range(
                normalization.absolute(bounds[0]),
                normalization.absolute(bounds[1]),
            );
        let mut automatic_labels = None;
        let values = if constant {
            vec![bounds[0]]
        } else if let GgplotTemporalBreaks::Explicit(values) = &self.arguments.breaks {
            values.iter().map(|v| v.0).collect()
        } else {
            if bounds.iter().any(|v| !v.is_finite()) {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Temporal guide limits must be finite.",
                ));
            }
            let view = Bounds::new(bounds[0], bounds[1])?;
            let selected = match &self.arguments.breaks {
                GgplotTemporalBreaks::Width(text) => GgplotTimeBreaks {
                    values: super::ggplot_time::aesthetic_width(
                        TimeBounds {
                            start: utc::absolute(view.minimum(), self.origin)?,
                            end: utc::absolute(view.maximum(), self.origin)?,
                        },
                        self.unit,
                        &calendar,
                        self.arguments.date,
                        text,
                        budget,
                    )?,
                    labels: vec![],
                },
                GgplotTemporalBreaks::Automatic if self.arguments.date => {
                    super::ggplot_time_pretty::pretty_date(
                        self.origin,
                        view,
                        self.unit,
                        self.arguments.count.unwrap_or(5.),
                        budget,
                        false,
                    )?
                }
                GgplotTemporalBreaks::Automatic => super::ggplot_time_pretty::pretty_time(
                    self.origin,
                    view,
                    self.unit,
                    &calendar,
                    self.arguments.count.unwrap_or(5.),
                    budget,
                    false,
                )?,
                _ => unreachable!("explicit and null temporal breaks returned above"),
            };
            if !matches!(self.arguments.breaks, GgplotTemporalBreaks::Width(_)) {
                automatic_labels = Some(selected.labels.into_iter().map(Some).collect());
            }
            selected
                .values
                .iter()
                .map(|v| (i128::from(*v) - i128::from(self.origin)) as f64)
                .collect()
        };
        crate::limits::require_within(values.len() <= budget, "temporal guide candidate")?;
        let labels = if let Some(format) = &self.arguments.format {
            GgplotGuideLabels::Explicit(self.format_values(&values, format, &calendar)?)
        } else if matches!(self.arguments.labels, GgplotGuideLabels::Automatic) {
            let labels = match automatic_labels {
                Some(labels) => labels,
                None => {
                    let mut midnight = true;
                    for value in values.iter().filter(|v| v.is_finite()) {
                        let date =
                            calendar.components(utc::absolute(*value, self.origin)?, self.unit)?;
                        midnight &= date.hour == 0
                            && date.minute == 0
                            && date.second == 0
                            && date.nanosecond == 0;
                    }
                    self.format_values(
                        &values,
                        &GgplotTimeFormat {
                            pattern: if self.arguments.date || midnight {
                                "%Y-%m-%d"
                            } else {
                                "%Y-%m-%d %H:%M:%S"
                            }
                            .into(),
                            locale: None,
                        },
                        &calendar,
                    )?
                }
            };
            GgplotGuideLabels::Explicit(labels)
        } else {
            self.arguments.labels.clone()
        };
        super::ggplot_continuous_guide::numeric_guide_entries(
            values,
            bounds,
            NumericFamily::Linear,
            false,
            &labels,
            label_budget,
        )
    }
    fn format_values(
        &self,
        values: &[f64],
        format: &GgplotTimeFormat,
        calendar: &Calendar,
    ) -> ChartResult<Vec<Option<String>>> {
        let formatter = format.prepare(calendar.clone())?;
        values
            .iter()
            .map(|value| {
                if value.is_finite() {
                    formatter
                        .format(utc::absolute(*value, self.origin)?, self.unit)
                        .map(Some)
                } else {
                    Ok(None)
                }
            })
            .collect()
    }
}
