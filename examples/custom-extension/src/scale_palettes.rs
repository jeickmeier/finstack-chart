//! Pure reference palettes installed explicitly by native and portable proof hosts.
use chart_core::{
    ChartResult, DiagnosticCode, Revision,
    grammar::{
        CustomScalePalette, ExtensionDescriptor, ExtensionRegistry, ScalePaletteDomain,
        ScalePaletteInput, ScalePaletteOutput,
    },
    interpolate::{Number, Value},
};
use std::sync::Arc;
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mode {
    Full,
    Endpoints,
    Short,
    Empty,
    Null,
    Named,
    Missing,
    Index,
    Length,
    First,
    Constant,
    Reject,
    RepeatCount,
    ReverseCount,
    SquareCount,
    ShortCount,
    NamedSquareCount,
    MissingCount,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum Channel {
    Colour,
    Size,
    Alpha,
    Linetype,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    #[serde(default)]
    count: bool,
    mode: Mode,
    channel: Channel,
}
fn parameters(value: &serde_json::Value) -> ChartResult<Parameters> {
    serde_json::from_value(value.clone())
        .map_err(|e| super::error(DiagnosticCode::Validation, e.to_string()))
}
/// Count/vector example with the same pure implementation in all installed hosts.
pub struct Palette {
    /// Whether this identity may execute in a portable consumer.
    pub portable: bool,
}
impl CustomScalePalette for Palette {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable {
                "example.scale_palette"
            } else {
                "example.native_scale_palette"
            },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, value: &serde_json::Value) -> ChartResult<()> {
        parameters(value).map(|_| ())
    }
    fn evaluate(&self, input: ScalePaletteInput<'_>) -> ChartResult<ScalePaletteOutput> {
        let p = parameters(input.parameters)?;
        if matches!(p.mode, Mode::Reject) {
            return Err(super::error(
                DiagnosticCode::Validation,
                "Requested palette rejects evaluation.",
            ));
        }
        if matches!(p.mode, Mode::RepeatCount) {
            let count = match input.domain {
                ScalePaletteDomain::Count(n) if n <= 4096 => n,
                ScalePaletteDomain::Normalized([Number(n)])
                    if n.is_finite() && *n >= 0. && *n <= 4096. =>
                {
                    *n as usize
                }
                _ => {
                    return Err(super::error(
                        DiagnosticCode::Validation,
                        "Count callback requires one bounded nonnegative repeat count.",
                    ));
                }
            };
            return Ok(ScalePaletteOutput {
                values: Some(vec![Value::Number(Number(3.)); count]),
                names: None,
            });
        }
        if matches!(
            p.mode,
            Mode::ReverseCount
                | Mode::SquareCount
                | Mode::ShortCount
                | Mode::NamedSquareCount
                | Mode::MissingCount
        ) {
            let ScalePaletteDomain::Count(n) = input.domain else {
                return Err(super::error(
                    DiagnosticCode::Validation,
                    "A positional example requires a category count.",
                ));
            };
            if n > 4096 {
                return Err(super::error(
                    DiagnosticCode::ResourceLimit,
                    "Positional example count exceeds its budget.",
                ));
            }
            let mut values = (1..=n)
                .map(|i| match p.mode {
                    Mode::ReverseCount => Value::Number(Number((n + 1 - i) as f64)),
                    Mode::MissingCount => Value::Missing,
                    Mode::ShortCount => Value::Number(Number(i as f64)),
                    _ => Value::Number(Number((i * i) as f64)),
                })
                .collect::<Vec<_>>();
            if matches!(p.mode, Mode::ShortCount) {
                values.truncate(n.saturating_sub(1));
            }
            let names = matches!(p.mode, Mode::NamedSquareCount).then(|| {
                (0..n)
                    .rev()
                    .map(|i| char::from(b'a' + (i % 26) as u8).to_string())
                    .collect()
            });
            return Ok(ScalePaletteOutput {
                values: Some(values),
                names,
            });
        }
        let samples = match input.domain {
            ScalePaletteDomain::Count(count) => (1..=count)
                .map(|i| Number(i as f64 / count.max(1) as f64))
                .collect::<Vec<_>>(),
            ScalePaletteDomain::Normalized(values) if p.count => (1..=values.len())
                .map(|i| Number(i as f64 / values.len().max(1) as f64))
                .collect(),
            ScalePaletteDomain::Normalized(values) => values.to_vec(),
        };
        let mut values = samples
            .iter()
            .enumerate()
            .map(|(i, Number(sample))| {
                let t = match p.mode {
                    Mode::Endpoints => i as f64 / samples.len().saturating_sub(1).max(1) as f64,
                    Mode::Index => (i + 1) as f64 / samples.len() as f64,
                    Mode::Length => samples.len() as f64 / 10.,
                    Mode::First => samples[0].0,
                    Mode::Constant => match p.channel {
                        Channel::Colour => 0.,
                        Channel::Size => 0.5,
                        Channel::Alpha => 0.3,
                        Channel::Linetype => 0.,
                    },
                    _ => *sample,
                };
                if t.is_nan() {
                    return Value::Missing;
                }
                match p.channel {
                    Channel::Colour => {
                        Value::Text(if t < 0.5 { "#ff0000" } else { "#0000ff" }.into())
                    }
                    Channel::Size => Value::Number(Number(1. + 4. * t)),
                    Channel::Alpha => Value::Number(Number(t)),
                    Channel::Linetype => Value::Text("22".into()),
                }
            })
            .collect::<Vec<_>>();
        match p.mode {
            Mode::Short => values.truncate(1),
            Mode::Empty => values.clear(),
            Mode::Missing if values.len() > 1 => values[1] = Value::Missing,
            _ => {}
        }
        let names = matches!(p.mode, Mode::Named).then(|| {
            (0..values.len())
                .map(|i| ["c", "b", "a", "d"][i % 4].to_owned())
                .collect()
        });
        Ok(ScalePaletteOutput {
            values: (!matches!(p.mode, Mode::Null)).then_some(values),
            names,
        })
    }
}
/// Install the portable and native-only qualified versions without replacing prior registrations.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_scale_palette(Arc::new(Palette { portable: true }))?;
    registry.register_scale_palette(Arc::new(Palette { portable: false }))
}
