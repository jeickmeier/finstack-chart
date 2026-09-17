use super::*;
use crate::data::TimeUnit;

/// Policy for values in a supplied calendar's closed intervals.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClosedSessionPolicy {
    /// Omit the value; line geometry splits at the gap.
    Omit,
    /// Snap to the nearest session boundary, earlier boundary wins a tie.
    Nearest,
    /// Reject the input rather than choosing a timestamp.
    Error,
}
/// Supplied ordered non-overlapping active intervals, in explicit source timestamp units.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SessionCalendar {
    /// Stable caller-defined calendar identity (no exchange correctness implied).
    pub id: String,
    /// Calendar revision.
    pub revision: crate::Revision,
    /// Source unit.
    pub unit: TimeUnit,
    /// Active intervals; interior intervals are half-open, the final end is included.
    pub sessions: Vec<TimeBounds>,
    /// Closed-session behavior.
    pub closed: ClosedSessionPolicy,
}
/// Compressed active-time mapping; original integer timestamps are always retained in source.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionScale {
    calendar: SessionCalendar,
    offsets: Vec<i64>,
    inner: LinearScale,
}
impl SessionScale {
    /// Validate calendar and map active duration, with an optional exact timestamp viewport.
    pub fn new(
        calendar: SessionCalendar,
        range: Bounds,
        viewport: Option<TimeBounds>,
        outside: OutsidePolicy,
    ) -> ChartResult<Self> {
        if calendar.id.is_empty()
            || calendar.sessions.is_empty()
            || calendar.sessions.len() > 100_000
        {
            return Err(error(
                DiagnosticCode::Validation,
                "Session calendar needs an identity and 1..100000 intervals.",
            ));
        }
        let mut offsets = vec![];
        let mut total = 0_i64;
        let mut previous = None;
        for s in &calendar.sessions {
            if s.start >= s.end || previous.is_some_and(|v| s.start < v) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Sessions must be ordered, non-overlapping and have positive duration.",
                ));
            }
            offsets.push(total);
            total = total
                .checked_add(s.end.checked_sub(s.start).ok_or_else(|| {
                    error(DiagnosticCode::PrecisionLoss, "Session duration overflow.")
                })?)
                .ok_or_else(|| {
                    error(DiagnosticCode::PrecisionLoss, "Calendar duration overflow.")
                })?;
            previous = Some(s.end);
        }
        if total as u64 > 1_u64 << 53 {
            return Err(error(
                DiagnosticCode::PrecisionLoss,
                "Active session duration exceeds exact binary64 ticks; use coarser timestamp units.",
            ));
        }
        let domain = Bounds::new(0., total as f64)?;
        let mut result = Self {
            calendar,
            offsets,
            inner: LinearScale::from_domains(domain, domain, range, outside)?,
        };
        if let Some(v) = viewport {
            let a = result.offset(v.start)?.ok_or_else(|| {
                error(
                    DiagnosticCode::NumericalDomain,
                    "Session viewport starts in a closed interval.",
                )
            })?;
            let b = result.offset(v.end)?.ok_or_else(|| {
                error(
                    DiagnosticCode::NumericalDomain,
                    "Session viewport ends in a closed interval.",
                )
            })?;
            result.inner = LinearScale::from_domains(
                domain,
                Bounds::new(a as f64, b as f64)?,
                range,
                outside,
            )?;
        }
        Ok(result)
    }
    fn offset(&self, t: i64) -> ChartResult<Option<i64>> {
        let i = self.calendar.sessions.partition_point(|s| s.start <= t);
        if i > 0 {
            let j = i - 1;
            let s = self.calendar.sessions[j];
            if t < s.end || (j + 1 == self.calendar.sessions.len() && t == s.end) {
                return Ok(Some(self.offsets[j] + (t - s.start)));
            }
        }
        match self.calendar.closed {
            ClosedSessionPolicy::Omit => Ok(None),
            ClosedSessionPolicy::Error => Err(error(
                DiagnosticCode::NumericalDomain,
                "Timestamp lies outside supplied active sessions.",
            )),
            ClosedSessionPolicy::Nearest => {
                let before = i.checked_sub(1).map(|j| {
                    (
                        self.calendar.sessions[j].end,
                        self.offsets[j]
                            + (self.calendar.sessions[j].end - self.calendar.sessions[j].start),
                    )
                });
                let after = self
                    .calendar
                    .sessions
                    .get(i)
                    .map(|s| (s.start, self.offsets[i]));
                let best = match (before, after) {
                    (Some(a), Some(b)) => {
                        if (i128::from(t) - i128::from(a.0)).abs()
                            <= (i128::from(t) - i128::from(b.0)).abs()
                        {
                            a
                        } else {
                            b
                        }
                    }
                    (Some(a), None) => a,
                    (None, Some(b)) => b,
                    _ => unreachable!(),
                };
                Ok(Some(best.1))
            }
        }
    }
    /// Retain active-time coordinates independently of viewport censoring.
    pub(crate) fn coordinate_value(&self, value: i64) -> ChartResult<Option<f64>> {
        self.offset(value).map(|v| v.map(|v| v as f64))
    }
    /// Map an original timestamp through the explicit calendar policy.
    pub fn map(&self, t: i64) -> ChartResult<Option<f64>> {
        self.offset(t)?
            .map(|t| self.inner.map(t as f64))
            .transpose()
            .map(Option::flatten)
    }
    /// Invert to an active timestamp. A compressed shared boundary chooses the next session start.
    pub fn invert(&self, p: f64) -> ChartResult<i64> {
        self.timestamp_at_offset(self.inner.invert(p)?)
    }
    /// Declared active-time domain and effective view, in compressed timestamp ticks.
    pub fn navigation_bounds(&self) -> (Bounds, Bounds) {
        (self.inner.domain(), self.inner.viewport())
    }
    /// Resolve compressed ticks using the same boundary convention as pointer inversion.
    pub fn timestamp_at_offset(&self, offset: f64) -> ChartResult<i64> {
        if !offset.is_finite() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Session offset must be finite.",
            ));
        }
        let offset = offset.round();
        if offset < 0. || offset > self.inner.domain().end() {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Session inverse lies outside the supplied calendar.",
            ));
        }
        let t = offset as i64;
        let i = self.offsets.partition_point(|v| *v <= t).saturating_sub(1);
        self.calendar.sessions[i]
            .start
            .checked_add(t - self.offsets[i])
            .ok_or_else(|| {
                error(
                    DiagnosticCode::PrecisionLoss,
                    "Session inverse overflows timestamp.",
                )
            })
    }
    /// Exact supplied calendar identity, revision, units and policy.
    pub fn calendar(&self) -> &SessionCalendar {
        &self.calendar
    }
    pub(crate) fn coordinate_domain(&self) -> Bounds {
        self.inner.domain()
    }
    pub(crate) fn coordinate_viewport(&self) -> Bounds {
        self.inner.viewport()
    }
    /// Destination range.
    pub fn range(&self) -> Bounds {
        self.inner.range()
    }
    /// Bounded active-time ticks, formatted with UTC calendar logic and exact source units.
    pub fn ticks(&self, target: usize, max_ticks: usize) -> ChartResult<Vec<UtcTick>> {
        let mut ticks: Vec<UtcTick> = self
            .inner
            .ticks(target, max_ticks)?
            .into_iter()
            .map(|t| {
                let p = self.inner.map(t.value)?.expect("tick in view");
                let value = self.invert(p)?;
                Ok(UtcTick {
                    value,
                    label: format_utc(value, self.calendar.unit, UtcInterval::Ticks(1))?,
                })
            })
            .collect::<ChartResult<_>>()?;
        ticks.dedup_by_key(|t| t.value);
        Ok(ticks)
    }
}
