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
    registrations:
        std::sync::Arc<crate::grammar::interpolation_extensions::InterpolationRegistrations>,
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
        Self::new_with_registrations(spec, Default::default())
    }
    /// Prepare a time range against explicitly installed interpolation factories.
    pub fn new_with_registry(
        spec: TimeScaleSpec,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        Self::new_with_registrations(spec, registry.interpolations.clone())
    }
    pub(crate) fn new_with_registrations(
        spec: TimeScaleSpec,
        registrations: std::sync::Arc<
            crate::grammar::interpolation_extensions::InterpolationRegistrations,
        >,
    ) -> ChartResult<Self> {
        let calendar = Calendar::new(spec.zone.clone())?;
        let origin = spec.domain.first().copied().unwrap_or(0);
        let domain = spec
            .domain
            .iter()
            .map(|v| utc::relative(*v, origin).map(Number))
            .collect::<ChartResult<Vec<_>>>()?;
        let mapping = ContinuousScale::new_with_registrations(
            ContinuousScaleSpec {
                family: NumericFamily::Linear,
                domain: domain.clone(),
                range: spec.range.clone(),
                factory: spec.factory.clone(),
                clamp: spec.clamp,
                unknown: spec.unknown.clone(),
            },
            registrations.clone(),
        )?;
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
            registrations,
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
        Self::new_with_registrations(spec, self.registrations.clone())
    }
    /// Compile custom or conditional labels with this scale's exact calendar revision.
    pub fn tick_format(&self, format: TimeFormat) -> ChartResult<TimeFormatter> {
        format.prepare(self.calendar.clone())
    }
    /// Immutable configuration replacement, with normal preparation checks.
    pub fn reconfigure(&self, spec: TimeScaleSpec) -> ChartResult<Self> {
        Self::new_with_registrations(spec, self.registrations.clone())
    }
    /// Versioned portable standalone descriptor with canonical integer strings.
    pub fn to_json(&self) -> ChartResult<String> {
        self.spec
            .factory
            .validate_registration(&self.registrations, true)?;
        crate::portable::encode(&TimeWire {
            version: if self.spec.factory.has_registration() {
                2
            } else {
                1
            },
            spec: self.spec.clone(),
        })
    }
    /// Decode bounded resources, reject unknown versions, then prepare the common engine.
    pub fn from_json(input: &str) -> ChartResult<Self> {
        Self::from_json_with_registry(input, &crate::grammar::ExtensionRegistry::new())
    }
    /// Decode explicit native factory references with supplied code registrations.
    pub fn from_json_with_registry(
        input: &str,
        registry: &crate::grammar::ExtensionRegistry,
    ) -> ChartResult<Self> {
        let wire: TimeWire = crate::portable::decode(input)?;
        let required = if wire.spec.factory.has_registration() {
            2
        } else {
            1
        };
        if wire.version != required {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported standalone time scale version.",
            ));
        }
        wire.spec
            .factory
            .validate_registration(&registry.interpolations, true)?;
        Self::new_with_registry(wire.spec, registry)
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
    view: TimeBounds,
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
        Self::resolve_reference(spec, range, viewport, outside, None)
    }
    /// Resolve optional reference expansion in seconds. Source timestamps remain exact;
    /// fractional view boundaries are retained relative to the integer origin.
    pub fn resolve_reference(
        spec: TimeScaleSpec,
        range: super::Bounds,
        viewport: Option<TimeBounds>,
        outside: super::OutsidePolicy,
        expansion: Option<super::GgplotExpansion>,
    ) -> ChartResult<Self> {
        Self::resolve_reference_unit(spec, range, viewport, outside, expansion, 1.)
    }
    /// Shared time projection with reference expansion measured in the supplied
    /// number of elapsed seconds (one for datetime, 86400 for Date).
    pub(crate) fn resolve_reference_unit(
        spec: TimeScaleSpec,
        range: super::Bounds,
        viewport: Option<TimeBounds>,
        outside: super::OutsidePolicy,
        expansion: Option<super::GgplotExpansion>,
        reference_seconds: f64,
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
        let reference = expansion.is_some();
        let factor = utc::ticks_per_second(spec.unit) as f64 * reference_seconds;
        let end = utc::relative(*spec.domain.last().expect("prepared domain"), origin)?;
        // Calibrate a constant domain in reference units, including when an
        // explicit viewport overrides expansion. Preserve the authored outer range.
        if reference && end == 0. {
            numeric.domain = vec![Number(0.), Number(factor)];
            numeric.range = vec![
                numeric.range[0],
                *numeric.range.last().expect("prepared range"),
            ];
        }
        let viewport = match (viewport, expansion) {
            (None, Some(expansion)) => {
                // The reference's near-zero decision uses absolute epoch magnitude,
                // while projection still subtracts the exact integer origin first.
                let expansion = if super::ggplot::zero_range(
                    origin as f64,
                    *spec.domain.last().expect("prepared domain") as f64,
                ) {
                    super::GgplotExpansion {
                        mult: [0.; 2],
                        add: [
                            expansion.add[0] + expansion.mult[0],
                            expansion.add[1] + expansion.mult[1],
                        ],
                    }
                } else {
                    expansion
                };
                let expanded =
                    expansion.continuous_viewport(super::Bounds::new(0., end / factor)?)?;
                Some(super::Bounds::new(
                    expanded.start() * factor,
                    expanded.end() * factor,
                )?)
            }
            (viewport, _) => viewport,
        };
        let mapping = super::NumericAxisScale::resolve(numeric, range, viewport, outside)?;
        let relative = mapping.viewport();
        let (first, last) = if relative.start() <= relative.end() {
            (relative.start().floor(), relative.end().ceil())
        } else {
            (relative.start().ceil(), relative.end().floor())
        };
        let view = TimeBounds {
            start: utc::absolute(first, origin)?,
            end: utc::absolute(last, origin)?,
        };
        Ok(Self {
            spec,
            calendar: prepared.calendar,
            origin,
            view,
            mapping,
            date_inverse: if reference {
                None
            } else {
                prepared.date_inverse
            },
            outside,
        })
    }
    pub(crate) fn shifted(&self, offset: i64) -> ChartResult<Self> {
        let mut shifted = self.clone();
        shifted.origin = utc::shift_timestamp(self.origin, offset)?;
        shifted.view = utc::shift_bounds(self.view, offset)?;
        shifted.spec.domain = self
            .spec
            .domain
            .iter()
            .map(|v| utc::shift_timestamp(*v, offset))
            .collect::<ChartResult<_>>()?;
        // Secondary guides use exact origin-relative projection, not the legacy
        // JavaScript-Date inverse rounding used by interactive numeric outputs.
        shifted.date_inverse = None;
        Ok(shifted)
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
    /// Integer enclosure of the visible interval. Reference expansion may place the
    /// actual boundaries between source quanta; use `relative_viewport` for those.
    pub fn viewport(&self) -> TimeBounds {
        self.view
    }
    /// Exact floating view endpoints after subtracting `origin`, in source units.
    pub fn relative_viewport(&self) -> super::Bounds {
        self.mapping.viewport()
    }
    /// Inclusive integer timestamps inside the visible interval, or none when the
    /// interval lies wholly between source quanta.
    pub fn tick_bounds(&self) -> ChartResult<Option<TimeBounds>> {
        let view = self.mapping.viewport();
        let low = view.minimum().ceil();
        let high = view.maximum().floor();
        if low > high {
            return Ok(None);
        }
        Ok(Some(TimeBounds {
            start: utc::absolute(low, self.origin)?,
            end: utc::absolute(high, self.origin)?,
        }))
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
        let low = self.view.start.min(self.view.end);
        let high = self.view.start.max(self.view.end);
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
    /// Preserve an exact source timestamp as an origin-relative guide coordinate.
    pub(crate) fn relative_guide(&self, value: i64) -> ChartResult<f64> {
        utc::relative(value, self.origin)
    }
    /// Date guide candidates use whole UTC days, including before the epoch.
    pub(crate) fn floor_date_guide(&self, value: i64) -> ChartResult<i64> {
        let day = utc::ticks_per_second(self.unit()) * 86400;
        i64::try_from(i128::from(value).div_euclid(day) * day).map_err(|_| {
            error(
                DiagnosticCode::PrecisionLoss,
                "Floored Date guide exceeds timestamp representation.",
            )
        })
    }
    /// Project an already selected origin-relative guide coordinate, including
    /// positions between representable source timestamps.
    pub(crate) fn map_relative_guide(&self, value: f64) -> ChartResult<Option<f64>> {
        self.mapping.map(value)
    }
    /// Preserve a fractional guide timestamp by promoting its source resolution.
    /// Positions finer than nanoseconds or outside i64 have no raw timestamp.
    pub(crate) fn guide_timestamp(&self, relative: f64) -> Option<(i64, TimeUnit)> {
        let units = [
            TimeUnit::Seconds,
            TimeUnit::Milliseconds,
            TimeUnit::Microseconds,
            TimeUnit::Nanoseconds,
        ];
        for unit in units {
            let numerator = utc::ticks_per_second(unit);
            let denominator = utc::ticks_per_second(self.unit());
            if numerator < denominator {
                continue;
            }
            let factor = numerator / denominator;
            let delta = relative * factor as f64;
            if !delta.is_finite() || delta.fract() != 0. || delta.abs() > (1_u64 << 53) as f64 {
                continue;
            }
            let absolute = i128::from(self.origin) * factor + delta as i128;
            if let Ok(value) = i64::try_from(absolute) {
                return Some((value, unit));
            }
        }
        None
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
        let domain = self.domain();
        super::Bounds::new(
            self.mapping
                .coordinate(utc::relative(domain.start, self.origin)?)?,
            self.mapping
                .coordinate(utc::relative(domain.end, self.origin)?)?,
        )
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
    pub(crate) fn ggplot_width_ticks(
        &self,
        width: super::CalendarInterval,
        budget: usize,
        date: bool,
    ) -> ChartResult<Vec<i64>> {
        match self.tick_bounds()? {
            Some(bounds) if date && width.unit == super::CalendarUnit::Day => {
                super::ggplot_time::date_day_width(bounds, self.unit(), width.step, budget)
            }
            Some(bounds) => super::ggplot_time::breaks_width_from(
                bounds,
                self.view.start.min(self.view.end),
                self.unit(),
                &self.calendar,
                width,
                budget,
            ),
            None => Ok(Vec::new()),
        }
    }
    /// Unthinned source-time candidates for the visible domain.
    pub fn ticks(&self, selection: CalendarTicks, budget: usize) -> ChartResult<Vec<i64>> {
        match self.tick_bounds()? {
            Some(bounds) => self.calendar.ticks(bounds, self.unit(), selection, budget),
            None => Ok(Vec::new()),
        }
    }
}
