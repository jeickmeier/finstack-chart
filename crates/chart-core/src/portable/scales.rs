//! Typed operation transport for owned scales; all behavior is delegated to scale kernels.
use super::encode;
use crate::{
    ChartResult,
    interpolate::{Number, Value},
    scales::*,
    typography::NumericLocale,
};
use serde::{Deserialize, Serialize};

/// Immutable changes at the host boundary.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScaleChange {
    /// Apply checked family-specific options atomically.
    Configure(Box<ScaleOptions>),
    /// Replace the complete configuration atomically.
    Reconfigure(Box<StandaloneScaleSpec>),
    /// Nice numeric endpoints with a count hint.
    Nice(Number),
    /// Nice calendar endpoints using the retained timezone rules.
    NiceTime(CalendarTicks),
    /// Explicit ordinal catalog growth, leaving the original scale unchanged.
    Train(Vec<ScaleKey>),
}
impl ScaleChange {
    /// Prepare an independent scale; errors do not change the input.
    pub fn apply(self, scale: &StandaloneScale) -> ChartResult<StandaloneScale> {
        match self {
            Self::Configure(options) => options.apply(scale),
            Self::Reconfigure(spec) => scale.reconfigure(*spec),
            Self::Nice(n) => scale.nice(n.0),
            Self::NiceTime(selection) => scale.reconfigure(StandaloneScaleSpec::Time(
                scale.time()?.nice(selection)?.spec().clone(),
            )),
            Self::Train(keys) => scale.train(keys),
        }
    }
}
/// Read-only standalone operations with typed, bounded argument decoding.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ScaleQuery {
    /// Immutable normalized configuration.
    Spec,
    /// Prepared observable domain, retaining typed identities.
    Domain,
    /// Authored range or effective interpolator samples.
    Range,
    /// Shared typed descriptor for chart color and numeric aesthetics.
    Mapped(ScaleTraining),
    /// Pure map with explicit input identity.
    Map(ScaleInput),
    /// Numeric or timestamp inverse.
    Invert(Number),
    /// Classification interval with explicit membership.
    InvertExtent(Value),
    /// Raw numeric candidates.
    Ticks {
        /// Count hint.
        count: Number,
        /// Hard output limit.
        budget: usize,
    },
    /// Raw calendar candidates.
    TimeTicks {
        /// Count or interval.
        selection: CalendarTicks,
        /// Hard output limit.
        budget: usize,
    },
    /// Exact numeric label.
    Format {
        /// Input number.
        value: Number,
        /// Count hint for inferred precision.
        count: Number,
        /// Optional d3-format specifier.
        specifier: Option<String>,
        /// Explicit locale; no host lookup.
        locale: Box<NumericLocale>,
    },
    /// Exact calendar label with retained timezone rules.
    TimeFormat {
        /// Exact source timestamp.
        #[serde(with = "super::signed")]
        value: i64,
        /// Calendar formatter.
        format: Box<TimeFormat>,
    },
    /// Prepared classifier cuts.
    Thresholds,
    /// Sequential rank sample boundaries.
    Quantiles(Number),
    /// Effective sequential/diverging output range.
    SampledRange,
    /// Distance between categorical starts.
    Step,
    /// Categorical band width.
    Bandwidth,
    /// Oriented categorical extent.
    Extent(ScaleKey),
    /// Categorical center.
    Center(ScaleKey),
    /// Calendar floor.
    Floor {
        /// Exact source timestamp.
        #[serde(with = "super::signed")]
        value: i64,
        /// Calendar interval.
        interval: CalendarInterval,
    },
    /// Calendar ceil.
    Ceil {
        /// Exact source timestamp.
        #[serde(with = "super::signed")]
        value: i64,
        /// Calendar interval.
        interval: CalendarInterval,
    },
    /// Calendar nearest boundary; ties select ceil.
    RoundTime {
        /// Exact source timestamp.
        #[serde(with = "super::signed")]
        value: i64,
        /// Calendar interval.
        interval: CalendarInterval,
    },
    /// Calendar offset with reference step truncation.
    Offset {
        /// Exact source timestamp.
        #[serde(with = "super::signed")]
        value: i64,
        /// Calendar interval.
        interval: CalendarInterval,
        /// Signed number of interval steps.
        steps: Number,
    },
}
impl ScaleQuery {
    /// Execute and encode its typed result without host arithmetic or coercion.
    pub fn execute(self, scale: &StandaloneScale) -> ChartResult<String> {
        match self {
            Self::Spec => encode(scale.spec()),
            Self::Domain => encode(&scale.domain()),
            Self::Range => encode(&scale.range()?),
            Self::Mapped(training) => encode(&scale.mapped(training)?),
            Self::Map(input) => scale.map(input)?.to_json(),
            Self::Invert(n) => encode(&scale.invert(n.0)?),
            Self::InvertExtent(v) => encode(&scale.invert_extent(&v)?),
            Self::Ticks { count, budget } => encode(&scale.ticks(count.0, budget)?),
            Self::TimeTicks { selection, budget } => encode(
                &scale
                    .time()?
                    .ticks(selection, budget)?
                    .into_iter()
                    .map(ScaleInput::Time)
                    .collect::<Vec<_>>(),
            ),
            Self::Format {
                value,
                count,
                specifier,
                locale,
            } => encode(
                &scale
                    .tick_format(count.0, specifier.as_deref(), *locale)?
                    .format(value.0),
            ),
            Self::TimeFormat { value, format } => {
                let s = scale.time()?;
                encode(&s.tick_format(*format)?.format(value, s.spec().unit)?)
            }
            Self::Thresholds => encode(&scale.thresholds()?),
            Self::Quantiles(count) => encode(&scale.quantiles(count.0)?),
            Self::SampledRange => encode(&scale.sampled_range()?),
            Self::Step => encode(&Number(scale.category()?.step())),
            Self::Bandwidth => encode(&Number(scale.category()?.bandwidth())),
            Self::Extent(key) => encode(&scale.category()?.extent(&key)?),
            Self::Center(key) => encode(&scale.category()?.center(&key)?.map(Number)),
            Self::Floor { value, interval } => {
                let s = scale.time()?;
                encode(&ScaleInput::Time(s.calendar().floor(
                    value,
                    s.spec().unit,
                    interval,
                )?))
            }
            Self::Ceil { value, interval } => {
                let s = scale.time()?;
                encode(&ScaleInput::Time(s.calendar().ceil(
                    value,
                    s.spec().unit,
                    interval,
                )?))
            }
            Self::RoundTime { value, interval } => {
                let s = scale.time()?;
                encode(&ScaleInput::Time(s.calendar().round(
                    value,
                    s.spec().unit,
                    interval,
                )?))
            }
            Self::Offset {
                value,
                interval,
                steps,
            } => {
                let s = scale.time()?;
                encode(&ScaleInput::Time(s.calendar().offset(
                    value,
                    s.spec().unit,
                    interval,
                    steps.0,
                )?))
            }
        }
    }
}
