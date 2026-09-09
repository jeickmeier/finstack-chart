//! Integer-origin time mapping composed with shared continuous interpolation and calendars.
use super::{
    Calendar, CalendarDateTime, CalendarTicks, CalendarZone, ContinuousScale, ContinuousScaleSpec,
    NumericFamily, NumericScale, NumericScaleSpec, TimeBounds, TimeFormat, TimeFormatter, error,
    utc,
};
use crate::{
    ChartResult, DiagnosticCode,
    data::TimeUnit,
    interpolate::{InterpolationFactory, Number, Value},
};
use serde::{Deserialize, Serialize};

/// Portable time scale with exact timestamps and an explicit calendar resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimeScaleSpec {
    /// Monotone timestamps in the declared source unit.
    #[serde(with = "crate::portable::signed_vec")]
    pub domain: Vec<i64>,
    /// Timestamp resolution; no implicit millisecond conversion.
    pub unit: TimeUnit,
    /// Shared geometry and formatting calendar.
    pub zone: CalendarZone,
    /// Typed output knots.
    pub range: Vec<Value>,
    /// Shared range interpolation factory.
    pub factory: InterpolationFactory,
    /// Saturate mapping and inverse at the effective domain ends.
    pub clamp: bool,
    /// Missing input output.
    pub unknown: Value,
}
impl Default for TimeScaleSpec {
    fn default() -> Self {
        let continuous = ContinuousScaleSpec::d3(NumericFamily::Linear);
        Self {
            domain: vec![946_684_800_000, 946_771_200_000],
            unit: TimeUnit::Milliseconds,
            zone: CalendarZone::Utc,
            range: continuous.range,
            factory: continuous.factory,
            clamp: false,
            unknown: Value::Missing,
        }
    }
}
impl TimeScaleSpec {
    /// Reference local default: midnight January 1–2, 2000 in supplied rules.
    pub fn local(zone: CalendarZone) -> ChartResult<Self> {
        let calendar = Calendar::new(zone.clone())?;
        let domain = (1..=2)
            .map(|day| {
                calendar.from_components(
                    CalendarDateTime {
                        year: 2000,
                        month: 1,
                        day,
                        hour: 0,
                        minute: 0,
                        second: 0,
                        nanosecond: 0,
                        offset_seconds: 0,
                    },
                    TimeUnit::Milliseconds,
                )
            })
            .collect::<ChartResult<_>>()?;
        Ok(Self {
            domain,
            zone,
            ..Self::default()
        })
    }
}
/// Prepared time mapping. Calendar operations are independently checked against coverage.
#[derive(Clone, Debug)]
pub struct TimeScale {
    spec: TimeScaleSpec,
    origin: i64,
    mapping: ContinuousScale,
    inverse: Option<NumericScale>,
    date_inverse: Option<NumericScale>,
    calendar: Calendar,
}
impl TimeScale {
    /// Prepare interpolation once, subtracting an integer origin before float conversion.
    pub fn new(spec: TimeScaleSpec) -> ChartResult<Self> {
        let calendar = Calendar::new(spec.zone.clone())?;
        let origin = spec.domain.first().copied().unwrap_or(0);
        let domain = spec
            .domain
            .iter()
            .map(|v| utc::relative(*v, origin).map(Number))
            .collect::<ChartResult<Vec<_>>>()?;
        let mapping = ContinuousScale::new(ContinuousScaleSpec {
            family: NumericFamily::Linear,
            domain: domain.clone(),
            range: spec.range.clone(),
            factory: spec.factory,
            clamp: spec.clamp,
            unknown: spec.unknown.clone(),
        })?;
        let range = spec
            .range
            .iter()
            .map(|v| {
                if let Value::Number(n) = v {
                    Some(*n)
                } else {
                    None
                }
            })
            .collect::<Option<Vec<_>>>();
        let inverse = range
            .map(|range| {
                NumericScale::new(NumericScaleSpec {
                    domain,
                    range,
                    clamp: spec.clamp,
                    ..NumericScaleSpec::d3(NumericFamily::Linear)
                })
            })
            .transpose()?;
        // Date's inverse rounds the weighted absolute epoch in binary64 before TimeClip.
        // This compatibility path is confined to exactly representable millisecond Dates;
        // forward geometry and all native finer/larger timestamps retain integer origins.
        let date_inverse = if spec.unit == TimeUnit::Milliseconds
            && spec
                .domain
                .iter()
                .all(|v| v.unsigned_abs() <= 8_640_000_000_000_000)
        {
            inverse
                .as_ref()
                .map(|v| {
                    let mut n = v.spec().clone();
                    n.domain = spec.domain.iter().map(|v| Number(*v as f64)).collect();
                    NumericScale::new(n)
                })
                .transpose()?
        } else {
            None
        };
        Ok(Self {
            spec,
            origin,
            mapping,
            inverse,
            date_inverse,
            calendar,
        })
    }
    /// Authored descriptor; cloned descriptors and scales are independent.
    pub fn spec(&self) -> &TimeScaleSpec {
        &self.spec
    }
    /// Calendar used by ticks, nice and formatting.
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }
    /// Exact integer projection origin.
    pub fn origin(&self) -> i64 {
        self.origin
    }
    /// Map an exact timestamp or the explicit missing input.
    pub fn map(&self, value: Option<i64>) -> ChartResult<Value> {
        let value = value.map(|v| {
            let n = self.spec.domain.len().min(self.spec.range.len());
            if self.spec.clamp && n > 0 {
                let a = self.spec.domain[0];
                let b = self.spec.domain[n - 1];
                v.clamp(a.min(b), a.max(b))
            } else {
                v
            }
        });
        self.mapping
            .map(value.map(|v| utc::relative(v, self.origin)).transpose()?)
    }
    /// Numeric range inverse, truncating the absolute timestamp toward zero as Date does.
    /// Native finer units retain that unit's integer quantum and never cast absolute epochs.
    pub fn invert(&self, position: f64) -> ChartResult<i64> {
        if let Some(inverse) = &self.date_inverse {
            return date_absolute(inverse.invert(position)?);
        }
        let inverse = self.inverse.as_ref().ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "A typed time range has no numeric inverse.",
            )
        })?;
        let relative = inverse.invert(position)?;
        truncate_absolute(relative, self.origin)
    }
    fn bounds(&self) -> Option<TimeBounds> {
        Some(TimeBounds {
            start: *self.spec.domain.first()?,
            end: *self.spec.domain.last()?,
        })
    }
    /// Complete calendar candidates before any label measurement or thinning.
    pub fn ticks(&self, selection: CalendarTicks, budget: usize) -> ChartResult<Vec<i64>> {
        self.bounds().map_or_else(
            || Ok(vec![]),
            |domain| {
                self.calendar
                    .ticks(domain, self.spec.unit, selection, budget)
            },
        )
    }
    /// Copy with calendar-niced outer endpoints, preserving all interior knots and outputs.
    pub fn nice(&self, selection: CalendarTicks) -> ChartResult<Self> {
        let mut spec = self.spec.clone();
        if let Some(bounds) = self.bounds() {
            let nice = self.calendar.nice(bounds, spec.unit, selection)?;
            spec.domain[0] = nice.start;
            *spec.domain.last_mut().expect("nonempty domain") = nice.end;
        }
        Self::new(spec)
    }
    /// Compile custom or conditional labels with this scale's exact calendar revision.
    pub fn tick_format(&self, format: TimeFormat) -> ChartResult<TimeFormatter> {
        format.prepare(self.calendar.clone())
    }
    /// Immutable configuration replacement, with normal preparation checks.
    pub fn reconfigure(&self, spec: TimeScaleSpec) -> ChartResult<Self> {
        Self::new(spec)
    }
    /// Versioned portable standalone descriptor with canonical integer strings.
    pub fn to_json(&self) -> ChartResult<String> {
        crate::portable::encode(&TimeWire {
            version: 1,
            spec: self.spec.clone(),
        })
    }
    /// Decode bounded resources, reject unknown versions, then prepare the common engine.
    pub fn from_json(input: &str) -> ChartResult<Self> {
        let wire: TimeWire = crate::portable::decode(input)?;
        if wire.version != 1 {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported standalone time scale version.",
            ));
        }
        Self::new(wire.spec)
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeWire {
    version: u32,
    spec: TimeScaleSpec,
}
fn date_absolute(value: f64) -> ChartResult<i64> {
    if !value.is_finite() || value.abs() > 8_640_000_000_000_000. {
        return Err(error(
            DiagnosticCode::NumericalDomain,
            "The inverse is an invalid ECMAScript Date.",
        ));
    }
    truncate_absolute(value, 0)
}
fn truncate_absolute(relative: f64, origin: i64) -> ChartResult<i64> {
    let whole = utc::absolute(relative.trunc(), origin)?;
    let correction = if whole > 0 && relative.fract() < 0. {
        -1
    } else if whole < 0 && relative.fract() > 0. {
        1
    } else {
        0
    };
    whole.checked_add(correction).ok_or_else(|| {
        error(
            DiagnosticCode::PrecisionLoss,
            "Time inverse exceeds its integer representation.",
        )
    })
}

/// Destination time projection retaining authored knots and its calendar resource.
#[derive(Clone, Debug, PartialEq)]
pub struct TimeAxisScale {
    spec: TimeScaleSpec,
    calendar: Calendar,
    origin: i64,
    mapping: super::NumericAxisScale,
    date_inverse: Option<NumericScale>,
    outside: super::OutsidePolicy,
}
impl TimeAxisScale {
    /// Numeric time outputs share the positional numeric engine and outside policies.
    pub fn resolve(
        spec: TimeScaleSpec,
        range: super::Bounds,
        viewport: Option<TimeBounds>,
        outside: super::OutsidePolicy,
    ) -> ChartResult<Self> {
        use crate::interpolate::FactoryKind;
        let prepared = TimeScale::new(spec.clone())?;
        if !matches!(
            spec.factory.kind,
            FactoryKind::Value | FactoryKind::Number | FactoryKind::Round
        ) || spec.factory.gamma.is_some()
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Time axes require linear or rounded numeric range interpolation.",
            ));
        }
        let origin = prepared.origin;
        let mut numeric = prepared
            .inverse
            .ok_or_else(|| {
                error(
                    DiagnosticCode::SchemaConflict,
                    "Time axes require a numeric range.",
                )
            })?
            .spec()
            .clone();
        numeric.round = spec.factory.kind == FactoryKind::Round;
        let viewport = viewport
            .map(|v| {
                super::Bounds::new(
                    utc::relative(v.start, origin)?,
                    utc::relative(v.end, origin)?,
                )
            })
            .transpose()?;
        let mapping = super::NumericAxisScale::resolve(numeric, range, viewport, outside)?;
        Ok(Self {
            spec,
            calendar: prepared.calendar,
            origin,
            mapping,
            date_inverse: prepared.date_inverse,
            outside,
        })
    }
    /// Original authored configuration.
    pub fn spec(&self) -> &TimeScaleSpec {
        &self.spec
    }
    /// Common calendar for geometry and labels.
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }
    /// Exact original outer timestamps.
    pub fn domain(&self) -> TimeBounds {
        TimeBounds {
            start: self.spec.domain[0],
            end: *self.spec.domain.last().expect("resolved domain"),
        }
    }
    /// Visible timestamp endpoints.
    pub fn viewport(&self) -> TimeBounds {
        let v = self.mapping.viewport();
        TimeBounds {
            start: utc::absolute(v.start(), self.origin).expect("checked view"),
            end: utc::absolute(v.end(), self.origin).expect("checked view"),
        }
    }
    /// Exact timestamp origin.
    pub fn origin(&self) -> i64 {
        self.origin
    }
    /// Timestamp source resolution.
    pub fn unit(&self) -> TimeUnit {
        self.spec.unit
    }
    /// Destination units.
    pub fn range(&self) -> super::Bounds {
        self.mapping.range()
    }
    /// Project a timestamp using the retained numeric mapping.
    pub fn map(&self, mut value: i64) -> ChartResult<Option<f64>> {
        let view = self.viewport();
        let (low, high) = (view.start.min(view.end), view.start.max(view.end));
        if self.outside == super::OutsidePolicy::Omit && !(low..=high).contains(&value) {
            return Ok(None);
        }
        if self.outside == super::OutsidePolicy::Clamp {
            value = value.clamp(low, high);
        }
        if self.spec.clamp {
            let domain = self.domain();
            value = value.clamp(domain.start.min(domain.end), domain.start.max(domain.end));
        }
        self.mapping.map(utc::relative(value, self.origin)?)
    }
    /// Invert to an exact timestamp, truncating toward epoch zero.
    pub fn invert(&self, position: f64) -> ChartResult<i64> {
        if let Some(inverse) = &self.date_inverse {
            date_absolute(inverse.invert(self.mapping.inverse_coordinate(position)?)?)
        } else {
            truncate_absolute(self.mapping.invert(position)?, self.origin)
        }
    }
    /// Full domain in the numeric output coordinate used by navigation.
    pub fn coordinate_domain(&self) -> ChartResult<super::Bounds> {
        self.mapping.coordinate_domain()
    }
    /// Visible output coordinates used by navigation.
    pub fn coordinate_viewport(&self) -> super::Bounds {
        self.mapping.coordinate_viewport()
    }
    /// Reverse navigation coordinates through the same retained mapping.
    pub fn coordinate_inverse(&self, value: f64) -> ChartResult<i64> {
        if let Some(inverse) = &self.date_inverse {
            date_absolute(inverse.invert_unbounded(value)?)
        } else {
            truncate_absolute(self.mapping.coordinate_inverse(value)?, self.origin)
        }
    }
    /// Unthinned source-time candidates for the visible domain.
    pub fn ticks(&self, selection: CalendarTicks, budget: usize) -> ChartResult<Vec<i64>> {
        self.calendar
            .ticks(self.viewport(), self.unit(), selection, budget)
    }
}
