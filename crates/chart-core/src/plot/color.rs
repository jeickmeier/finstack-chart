use super::{error, fresh_id};
use crate::{
    ChartResult, DiagnosticCode, ScaleId,
    color::Paint,
    scales::{Bounds, ColorScale},
};
/// Named color scale retaining exact palette/domain/null/outside policies.
#[derive(Clone, Debug)]
pub struct ColorScaleBuilder {
    pub(super) name: String,
    pub(super) id: ChartResult<ScaleId>,
    pub(super) scale: ChartResult<ColorScale<Paint>>,
}
/// Discrete color mapping; the name selects this scale independently of the mapped field.
pub fn color_discrete(name: impl Into<String>) -> ColorScaleBuilder {
    ColorScaleBuilder {
        name: name.into(),
        id: fresh_id().map(ScaleId::new),
        scale: Ok(super::default_color_scale()),
    }
}
/// Continuous color mapping with an explicit numeric domain and two endpoint colors.
pub fn color_continuous(name: impl Into<String>, start: f64, end: f64) -> ColorScaleBuilder {
    ColorScaleBuilder {
        name: name.into(),
        id: fresh_id().map(ScaleId::new),
        scale: Bounds::new(start, end).map(|domain| ColorScale::Continuous {
            domain,
            palette: vec![
                crate::theme::rgb(245, 248, 255).into(),
                crate::theme::rgb(31, 119, 180).into(),
            ],
            clamp: true,
            missing: crate::theme::rgb(128, 128, 128).into(),
        }),
    }
}
impl ColorScaleBuilder {
    /// Replace the exact palette in order.
    pub fn palette<P: Into<Paint>>(mut self, palette: Vec<P>) -> Self {
        let palette = palette.into_iter().map(Into::into).collect();
        self.scale = self.scale.and_then(|mut s| {
            match &mut s {
                ColorScale::Mapped { .. } => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Set typed range outputs in the mapped scale descriptor.",
                    ));
                }
                ColorScale::Discrete { palette: p, .. }
                | ColorScale::Continuous { palette: p, .. } => *p = palette,
            };
            Ok(s)
        });
        self
    }
    /// Retain an exact named palette on a typed mapped color scale.
    pub fn palette_scheme(mut self, palette: crate::scales::chromatic::SchemeSpec) -> Self {
        self.scale = self.scale.and_then(|s| {
            let ColorScale::Mapped { scale, missing } = s else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Named palette ranges require a typed mapped scale.",
                ));
            };
            Ok(ColorScale::Mapped {
                scale: scale.with_palette(palette)?,
                missing,
            })
        });
        self
    }
    /// Set a fixed discrete category domain; unknown values use the explicit missing style.
    pub fn domain(mut self, domain: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let domain = domain.into_iter().map(Into::into).collect();
        self.scale = self.scale.and_then(|mut s| {
            let ColorScale::Discrete { domain: d, .. } = &mut s else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Category domains require a discrete color scale.",
                ));
            };
            *d = Some(domain);
            Ok(s)
        });
        self
    }
    /// Set null/unknown/outside color.
    pub fn missing(mut self, missing: impl Into<Paint>) -> Self {
        let missing = missing.into();
        self.scale = self.scale.map(|mut s| {
            if let ColorScale::Mapped { scale, .. } = &mut s {
                scale.missing_paint_is_na = false;
            }
            match &mut s {
                ColorScale::Discrete { missing: m, .. }
                | ColorScale::Continuous { missing: m, .. }
                | ColorScale::Mapped { missing: m, .. } => *m = missing,
            };
            s
        });
        self
    }
    /// Clamp continuous out-of-domain values; false uses the missing style.
    pub fn clamp(mut self, clamp: bool) -> Self {
        self.scale = self.scale.and_then(|mut s| {
            let ColorScale::Continuous { clamp: c, .. } = &mut s else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Clamp requires a continuous color scale.",
                ));
            };
            *c = clamp;
            Ok(s)
        });
        self
    }
}
/// Explicit color-guide override, separate from mappings and annotation text.
#[derive(Clone, Debug, Default)]
pub struct LegendBuilder {
    custom: Option<crate::grammar::CustomLegend>,
    pub(super) aesthetic: Option<crate::grammar::LegendAesthetic>,
    pub(super) options: Option<crate::grammar::LegendOptions>,
    pub(super) scale: Option<String>,
    pub(super) title: Option<String>,
    pub(super) generic: bool,
}
/// Configure an existing named color guide. Compatible guides remain automatic by default.
pub fn legend() -> LegendBuilder {
    LegendBuilder::default()
}
impl LegendBuilder {
    /// Draw a trained guide with an explicitly registered portable implementation.
    pub fn registered(
        mut self,
        operation: impl Into<String>,
        version: crate::Revision,
        parameters: serde_json::Value,
    ) -> Self {
        self.options.get_or_insert_with(Default::default).registered =
            Some(crate::grammar::GuideDrawingSelection {
                operation: crate::grammar::OperationRef {
                    id: operation.into(),
                    version,
                },
                parameters,
            });
        self
    }

    /// Place independent portable vector guide content.
    pub fn custom(mut self, guide: crate::grammar::CustomLegend) -> Self {
        self.custom = Some(guide);
        self
    }

    /// Select every scale bound to a nonpositional aesthetic.
    pub fn aesthetic(mut self, value: crate::grammar::LegendAesthetic) -> Self {
        self.aesthetic = Some(value);
        self
    }
    /// Set guide-only presentation and key overrides.
    pub fn options(mut self, value: crate::grammar::LegendOptions) -> Self {
        self.options = Some(value);
        self
    }

    /// Select an existing authored scale; this does not create a data mapping.
    pub fn scale(mut self, scale: impl Into<String>) -> Self {
        self.scale = Some(scale.into());
        self
    }
    /// Override only this guide's title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self.generic = false;
        self
    }
    /// Omit this guide's title while retaining its keys and labels.
    pub fn untitled(self) -> Self {
        self.title(String::new())
    }
    /// Use the scale's generic Color/Value title instead of the inferred field/group title.
    pub fn generic_title(mut self) -> Self {
        self.title = None;
        self.generic = true;
        self
    }
}

/// Named typed scale for color: classifiers, custom interpolation and asymmetric domains.
pub fn color_mapped(
    name: impl Into<String>,
    scale: crate::scales::MappedScaleSpec,
) -> ColorScaleBuilder {
    let shade = if scale.reference_guides() { 127 } else { 128 };
    let missing = if scale.missing_paint_is_na {
        crate::scene::Color {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        }
    } else {
        crate::theme::rgb(shade, shade, shade)
    };
    ColorScaleBuilder {
        name: name.into(),
        id: fresh_id().map(ScaleId::new),
        scale: Ok(ColorScale::Mapped {
            scale,
            missing: missing.into(),
        }),
    }
}

impl LegendBuilder {
    pub(super) fn apply(
        self,
        definition: &mut crate::grammar::ChartDefinition,
        names: &std::collections::BTreeMap<String, ScaleId>,
    ) -> ChartResult<()> {
        use crate::grammar::{
            LegendAesthetic as A, NumericAesthetic as N, PaintAesthetic as P, ValueAesthetic as V,
        };
        if let Some(mut guide) = self.custom {
            if self.scale.is_some() || self.aesthetic.is_some() {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Custom guides do not select mapped scales.",
                ));
            }
            if let Some(options) = self.options {
                guide.options = options;
            }
            if let Some(title) = self.title {
                guide.options.title = Some(title);
            }
            definition.custom_legends.push(guide);
            return Ok(());
        }
        let mut ids = std::collections::BTreeSet::new();
        if let Some(name) = &self.scale {
            ids.insert(*names.get(name).ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("Legend names absent scale '{name}'."),
                )
            })?);
        }
        if let Some(a) = self.aesthetic {
            for layer in &definition.layers {
                match a {
                    A::Color => ids.extend(layer.color.iter().map(|e| e.id)),
                    A::Fill | A::Stroke => ids.extend(
                        layer
                            .paint_scales
                            .get(&if a == A::Fill { P::Fill } else { P::Stroke })
                            .map(|e| e.id),
                    ),
                    A::Shape | A::LineType => ids.extend(
                        layer
                            .value_scales
                            .get(&if a == A::Shape { V::Shape } else { V::LineType })
                            .map(|e| e.id),
                    ),
                    _ => {
                        let channel = match a {
                            A::Size => N::Size,
                            A::AreaSize => N::AreaSize,
                            A::Alpha => N::Alpha,
                            A::Opacity => N::Opacity,
                            A::StrokeWidth => N::StrokeWidth,
                            _ => unreachable!(),
                        };
                        ids.extend(layer.numeric_scales.get(&channel).map(|e| e.id));
                    }
                }
            }
        }
        if ids.is_empty() {
            return Err(error(
                DiagnosticCode::MissingResource,
                "Legend override requires a mapped scale or aesthetic.",
            ));
        }
        for id in ids {
            let mut found = false;
            for color in definition
                .layers
                .iter_mut()
                .flat_map(|l| l.color.iter_mut().chain(l.paint_scales.values_mut()))
                .filter(|c| c.id == id)
            {
                found = true;
                if self.generic {
                    color.title = None;
                }
                if let Some(title) = &self.title {
                    color.title = Some(title.clone());
                }
            }
            found |= definition.layers.iter().any(|l| {
                l.numeric_scales
                    .values()
                    .chain(l.value_scales.values())
                    .any(|e| e.id == id)
            });
            if !found {
                return Err(error(
                    DiagnosticCode::MissingResource,
                    "Legend scale has no mapped layer.",
                ));
            }
            if self.options.is_some() || self.aesthetic.is_some() {
                let mut options = self.options.clone().unwrap_or_default();
                if self.title.is_some() {
                    options.title = self.title.clone();
                }
                options.validate()?;
                definition.legends.insert(id, options);
            }
        }
        Ok(())
    }
}
