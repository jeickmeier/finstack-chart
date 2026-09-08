use super::{error, fresh_id};
use crate::{
    ChartResult, DiagnosticCode, ScaleId,
    scales::{Bounds, ColorScale},
    scene::Color,
};
/// Named color scale retaining exact palette/domain/null/outside policies.
#[derive(Clone, Debug)]
pub struct ColorScaleBuilder {
    pub(super) name: String,
    pub(super) id: ChartResult<ScaleId>,
    pub(super) scale: ChartResult<ColorScale>,
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
                crate::theme::rgb(245, 248, 255),
                crate::theme::rgb(31, 119, 180),
            ],
            clamp: true,
            missing: crate::theme::rgb(128, 128, 128),
        }),
    }
}
impl ColorScaleBuilder {
    /// Replace the exact palette in order.
    pub fn palette(mut self, palette: Vec<Color>) -> Self {
        self.scale = self.scale.map(|mut s| {
            match &mut s {
                ColorScale::Discrete { palette: p, .. }
                | ColorScale::Continuous { palette: p, .. } => *p = palette,
            };
            s
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
    pub fn missing(mut self, missing: Color) -> Self {
        self.scale = self.scale.map(|mut s| {
            match &mut s {
                ColorScale::Discrete { missing: m, .. }
                | ColorScale::Continuous { missing: m, .. } => *m = missing,
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
    pub(super) scale: Option<String>,
    pub(super) title: Option<String>,
    pub(super) untitled: bool,
}
/// Configure an existing named color guide. Compatible guides remain automatic by default.
pub fn legend() -> LegendBuilder {
    LegendBuilder::default()
}
impl LegendBuilder {
    /// Select an existing authored scale; this does not create a data mapping.
    pub fn scale(mut self, scale: impl Into<String>) -> Self {
        self.scale = Some(scale.into());
        self
    }
    /// Override only this guide's title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self.untitled = false;
        self
    }
    /// Omit this guide's title while retaining its keys and labels.
    pub fn untitled(mut self) -> Self {
        self.title = None;
        self.untitled = true;
        self
    }
}
