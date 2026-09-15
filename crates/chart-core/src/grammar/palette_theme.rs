//! Resolve reference palette selection before scale population training.
use super::{ChartDefinition, interpolation_extensions};
use crate::{
    ChartResult, DiagnosticCode,
    interpolate::{InterpolationSpec, Value},
    scales::{GgplotDiscretePalette, GgplotScalePolicy, ScaleFunctionSpec, ScaleRangeFunction},
    theme::ThemeScalePalette,
};
use std::borrow::Cow;

pub(super) fn resolve(definition: &ChartDefinition) -> ChartResult<Cow<'_, ChartDefinition>> {
    let Some(theme) = &definition.theme else {
        return Ok(Cow::Borrowed(definition));
    };
    if theme.scale_palettes.is_empty() {
        return Ok(Cow::Borrowed(definition));
    }
    let selected = interpolation_extensions::mapped_scales(definition)
        .map(|scale| {
            let family = if matches!(
                scale.ggplot.as_deref(),
                Some(GgplotScalePolicy::Discrete { .. })
            ) {
                "discrete"
            } else {
                "continuous"
            };
            scale.palette_theme_aesthetics.iter().find_map(|aesthetic| {
                // Reference theme validation makes the color alias override colour.
                let aesthetic = if aesthetic == "color" {
                    "colour"
                } else {
                    aesthetic
                };
                let alias = (aesthetic == "colour").then(|| format!("palette.color.{family}"));
                alias
                    .as_ref()
                    .and_then(|key| theme.scale_palettes.get(key))
                    .or_else(|| {
                        theme
                            .scale_palettes
                            .get(&format!("palette.{aesthetic}.{family}"))
                    })
                    .cloned()
            })
        })
        .collect::<Vec<_>>();
    if selected.iter().all(Option::is_none) {
        return Ok(Cow::Borrowed(definition));
    }
    let mut resolved = definition.clone();
    for (scale, call) in interpolation_extensions::mapped_scales_mut(&mut resolved).zip(selected) {
        if let Some(call) = call {
            select(scale, call)?;
        }
    }
    Ok(Cow::Owned(resolved))
}
fn select(
    scale: &mut crate::scales::MappedScaleSpec,
    palette: ThemeScalePalette,
) -> ChartResult<()> {
    scale.catalog = None;
    match palette {
        ThemeScalePalette::Registered(call) => scale.palette_function = Some(Box::new(call)),
        ThemeScalePalette::Named(name) => {
            crate::scales::ggplot_named::validate(&name)?;
            scale.palette_function = None;
            if let Some(GgplotScalePolicy::Discrete { palette, .. }) = scale.ggplot.as_deref_mut() {
                *palette = GgplotDiscretePalette::Named(name);
            } else {
                gradient(scale, crate::scales::ggplot_named::continuous(&name)?)?;
            }
        }
        ThemeScalePalette::Colors(colors) => {
            if colors.len() == 1 {
                return select(
                    scale,
                    ThemeScalePalette::Named(colors[0].clone().unwrap_or_default()),
                );
            }
            if colors.is_empty() {
                return Err(super::error(
                    DiagnosticCode::Validation,
                    "Theme color vectors require colors or a known palette name.",
                ));
            }
            scale.palette_function = None;
            if let Some(GgplotScalePolicy::Discrete { palette, .. }) = scale.ggplot.as_deref_mut() {
                *palette = GgplotDiscretePalette::Values(
                    colors
                        .into_iter()
                        .map(|v| v.map_or(Value::Missing, Value::Text))
                        .collect(),
                );
            } else {
                gradient(
                    scale,
                    colors
                        .iter()
                        .map(|v| crate::color::parse_r(v.as_deref().unwrap_or("transparent")))
                        .collect::<ChartResult<Vec<_>>>()?,
                )?;
            }
        }
    }
    Ok(())
}
fn gradient(
    scale: &mut crate::scales::MappedScaleSpec,
    colors: Vec<crate::color::Paint>,
) -> ChartResult<()> {
    let ScaleFunctionSpec::Interpolated(interpolated) = &mut scale.function else {
        return Err(super::error(
            DiagnosticCode::UnsupportedCapability,
            "Theme gradients require an interpolated reference scale.",
        ));
    };
    interpolated.output = ScaleRangeFunction::Interpolate(InterpolationSpec::GgplotPalette {
        spec: crate::scales::chromatic::ggplot::PaletteSpec::Gradient {
            colors,
            values: None,
        },
    });
    if let Some(GgplotScalePolicy::Binned(policy)) = scale.ggplot.as_deref_mut() {
        policy.palette = None;
    }
    Ok(())
}
