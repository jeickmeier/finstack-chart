//! Pinned reference palette names, reusing established count and Lab owners.
use super::{GgplotDiscretePalette, chromatic};
use crate::{ChartResult, DiagnosticCode, color::Paint, interpolate::Value, scene::Color};
#[path = "ggplot_named_data.rs"]
mod data;
#[derive(Clone, Copy)]
enum NamedPalette {
    Hue,
    Grey,
    Brewer(chromatic::SchemeId),
    Viridis(chromatic::ggplot::ViridisOption),
    Gradient(&'static [u32]),
    Manual(&'static [u32]),
}
fn lookup(name: &str) -> ChartResult<NamedPalette> {
    let name = name.to_lowercase();
    data::PALETTES
        .binary_search_by_key(&name.as_str(), |(key, _)| *key)
        .map(|index| data::PALETTES[index].1)
        .map_err(|_| {
            super::error(
                DiagnosticCode::Validation,
                format!("Unknown reference palette: {name}"),
            )
        })
}
fn paint(value: u32) -> Paint {
    Paint::from(Color {
        red: (value >> 24) as u8,
        green: (value >> 16) as u8,
        blue: (value >> 8) as u8,
        alpha: value as u8,
    })
}
impl NamedPalette {
    fn count(self) -> Option<GgplotDiscretePalette> {
        Some(match self {
            Self::Hue => GgplotDiscretePalette::Hue(Default::default()),
            Self::Grey => GgplotDiscretePalette::Grey {
                start: 0.2,
                end: 0.8,
            },
            Self::Brewer(id) => GgplotDiscretePalette::Brewer { id, reverse: false },
            Self::Viridis(option) => GgplotDiscretePalette::Viridis {
                option,
                begin: 0.,
                end: 1.,
                reverse: false,
                alpha: 1.,
            },
            Self::Gradient(_) | Self::Manual(_) => return None,
        })
    }
    fn colors(self) -> ChartResult<Vec<Paint>> {
        match self {
            Self::Gradient(colors) | Self::Manual(colors) => {
                Ok(colors.iter().map(|c| paint(*c)).collect())
            }
            _ => {
                let n = if let Self::Brewer(id) = self {
                    *id.info().sizes.last().expect("catalog sizes")
                } else {
                    255
                };
                self.count()
                    .expect("count family")
                    .count_values(n)?
                    .into_iter()
                    .map(|value| match value {
                        Value::Color(color) => Ok(Paint::from(color)),
                        _ => Err(super::error(
                            DiagnosticCode::Validation,
                            "A reference named palette returned a missing anchor.",
                        )),
                    })
                    .collect()
            }
        }
    }
}
pub(crate) fn continuous(name: &str) -> ChartResult<Vec<Paint>> {
    lookup(name)?.colors()
}
pub(crate) fn discrete(name: &str, n: usize) -> ChartResult<Vec<Value>> {
    let palette = lookup(name)?;
    if let Some(count) = palette.count() {
        return count.count_values(n);
    }
    match palette {
        NamedPalette::Manual(colors) => Ok((0..n)
            .map(|i| {
                colors
                    .get(i)
                    .map_or(Value::Missing, |v| Value::Color(paint(*v).value()))
            })
            .collect()),
        NamedPalette::Gradient(_) => super::ggplot::gradient_count(&palette.colors()?, n),
        _ => unreachable!("count families returned above"),
    }
}

pub(crate) fn validate(name: &str) -> ChartResult<()> {
    lookup(name).map(|_| ())
}
