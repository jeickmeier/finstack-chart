use super::{
    error, fresh_id,
    text::{TextStyle, plain},
};
use crate::{
    ChartResult, DiagnosticCode, ScaleId,
    composition::ScaleValue,
    layout::{AxisScale, AxisSide, AxisSpec, CustomGuideTick},
    scales::*,
    typography::{NumberFormat, NumberLocale, NumberNotation, RichText},
};
use std::collections::BTreeMap;

/// Typed positional scale configuration; incompatible options reject at Plot build.
#[derive(Clone, Debug)]
pub struct ScaleBuilder {
    value: ChartResult<AxisScale>,
}
/// Linear numeric training with baseline domain defaults.
pub fn scale_linear() -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Linear(ContinuousDomain::default())),
    }
}
/// Logarithmic numeric scale with explicit base.
pub fn scale_log(base: f64) -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Nonlinear {
            transform: ScaleTransform::Log { base },
            domain: ContinuousDomain::default(),
        }),
    }
}
/// Symmetric logarithmic scale with an explicit positive linear threshold.
pub fn scale_symlog(threshold: f64) -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Nonlinear {
            transform: ScaleTransform::Symlog { threshold },
            domain: ContinuousDomain::default(),
        }),
    }
}
/// Categorical bands using stable first-seen labels by default.
pub fn scale_band() -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Band(BandOptions::default())),
    }
}
/// Categorical point centers using stable labels.
pub fn scale_point() -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Point(PointOptions::default())),
    }
}
/// Integer-origin UTC scale with automatic calendar ticks.
pub fn scale_utc() -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Utc {
            domain: None,
            interval: None,
        }),
    }
}
/// Supplied session calendar; no exchange calendar or clock is inferred.
pub fn scale_session(calendar: SessionCalendar) -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Session(calendar)),
    }
}
impl ScaleBuilder {
    fn numeric(mut self, change: impl FnOnce(&mut ContinuousDomain) -> ChartResult<()>) -> Self {
        self.value = self.value.and_then(|mut scale| {
            match &mut scale {
                AxisScale::Linear(domain) | AxisScale::Nonlinear { domain, .. } => change(domain)?,
                _ => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "This option requires a numeric scale.",
                    ));
                }
            };
            Ok(scale)
        });
        self
    }
    /// Explicit numeric domain, preserving descending direction.
    pub fn domain(self, start: f64, end: f64) -> Self {
        self.numeric(|d| {
            d.explicit = Some(Bounds::new(start, end)?);
            Ok(())
        })
    }
    /// Set an extra automatic-domain baseline contribution.
    pub fn baseline(self, baseline: Baseline) -> Self {
        self.numeric(|d| {
            d.baseline = baseline;
            Ok(())
        })
    }
    /// Set automatic numeric-domain padding, or point-scale outer padding.
    pub fn padding(mut self, padding: f64) -> Self {
        self.value = self.value.and_then(|mut scale| {
            match &mut scale {
                AxisScale::Linear(d) | AxisScale::Nonlinear { domain: d, .. } => {
                    d.padding = padding
                }
                AxisScale::Point(d) => d.padding = padding,
                _ => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Use band inner/outer padding or a numeric/point scale.",
                    ));
                }
            };
            Ok(scale)
        });
        self
    }
    /// Expand automatic numeric domains to the existing nice tick grid.
    pub fn nice(self, nice: bool) -> Self {
        self.numeric(|d| {
            d.nice = nice;
            Ok(())
        })
    }
    /// Set the fixed numeric nice-training tick target.
    pub fn nice_ticks(self, ticks: usize) -> Self {
        self.numeric(|d| {
            d.nice_ticks = ticks;
            Ok(())
        })
    }
    /// Explicit stable category domain/order for band or point scales.
    pub fn categories(mut self, values: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let values = values.into_iter().map(Into::into).collect();
        self.value = self.value.and_then(|mut scale| {
            match &mut scale {
                AxisScale::Band(d) => d.domain = Some(values),
                AxisScale::Point(d) => d.domain = Some(values),
                _ => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Categories require a band or point scale.",
                    ));
                }
            };
            Ok(scale)
        });
        self
    }
    /// Set band inner and outer step padding.
    pub fn band_padding(mut self, inner: f64, outer: f64) -> Self {
        self.value = self.value.and_then(|mut scale| {
            let AxisScale::Band(d) = &mut scale else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Band padding requires a band scale.",
                ));
            };
            d.inner_padding = inner;
            d.outer_padding = outer;
            Ok(scale)
        });
        self
    }
    /// Set outer step padding for a point scale.
    pub fn point_padding(mut self, padding: f64) -> Self {
        self.value = self.value.and_then(|mut scale| {
            let AxisScale::Point(options) = &mut scale else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Point padding requires a point scale.",
                ));
            };
            options.padding = padding;
            Ok(scale)
        });
        self
    }
    /// Exact UTC endpoints in the mapped field's original integer unit.
    pub fn time_domain(mut self, start: i64, end: i64) -> Self {
        self.value = self.value.and_then(|mut scale| {
            let AxisScale::Utc { domain, .. } = &mut scale else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Exact time domains require a UTC scale.",
                ));
            };
            *domain = Some(TimeBounds { start, end });
            Ok(scale)
        });
        self
    }
    /// Explicit UTC calendar interval.
    pub fn interval(mut self, value: UtcInterval) -> Self {
        self.value = self.value.and_then(|mut scale| {
            let AxisScale::Utc { interval, .. } = &mut scale else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Calendar intervals require a UTC scale.",
                ));
            };
            *interval = Some(value);
            Ok(scale)
        });
        self
    }
}
/// Stable named-axis identity for layer binding, annotation anchors and runtime navigation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AxisHandle(pub(super) ScaleId);
impl AxisHandle {
    /// Exact scale identity.
    pub fn id(self) -> ScaleId {
        self.0
    }
}
/// Axis label, scale and guide options using the existing shared axis implementation.
#[derive(Clone, Debug)]
pub struct AxisBuilder {
    pub(super) name: String,
    pub(super) spec: AxisSpec,
    pub(super) failure: Option<crate::Diagnostic>,
    secondary: Option<(String, f64, f64)>,
}
/// Primary bottom x axis.
pub fn x_axis() -> AxisBuilder {
    AxisBuilder {
        name: "x".into(),
        spec: AxisSpec::new(ScaleId::new(0), AxisSide::Bottom),
        failure: None,
        secondary: None,
    }
}
/// Primary left y axis.
pub fn y_axis() -> AxisBuilder {
    AxisBuilder {
        name: "y".into(),
        spec: AxisSpec::new(ScaleId::new(1), AxisSide::Left),
        failure: None,
        secondary: None,
    }
}
impl AxisBuilder {
    /// Name an independent axis with a fresh stable identity, retained by clones.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        if name != self.name {
            match fresh_id() {
                Ok(id) => self.spec.id = ScaleId::new(id),
                Err(e) => self.failure = Some(e),
            };
            self.name = name;
        }
        self
    }
    /// Resolve this builder's stable identity.
    pub fn handle(&self) -> ChartResult<AxisHandle> {
        self.failure
            .clone()
            .map_or(Ok(AxisHandle(self.spec.id)), Err)
    }
    /// Position the guide on one side without changing its scale identity.
    pub fn side(mut self, side: AxisSide) -> Self {
        self.spec.side = side;
        self
    }
    /// Set the axis title, separately from x/y annotation labels.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.spec.title = Some(plain(label));
        self
    }
    /// Set a rich axis title.
    pub fn rich_label(mut self, text: impl Into<RichText>) -> Self {
        self.spec.title = Some(text.into());
        self
    }
    /// Set tick-label typography.
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.spec.typography = Some(style.run);
        self
    }
    /// Set clockwise tick-label rotation.
    pub fn rotation(mut self, degrees: f64) -> Self {
        self.spec.label_rotation = degrees;
        self
    }
    /// Configure the positional scale family and training policy.
    pub fn scale(mut self, scale: ScaleBuilder) -> Self {
        match scale.value {
            Ok(scale) => self.spec.scale = scale,
            Err(e) => self.failure = Some(e),
        };
        self
    }
    /// Show or hide this guide without removing the scale.
    pub fn visible(mut self, visible: bool) -> Self {
        self.spec.visible = visible;
        self
    }
    /// Set an explicit calculation-space viewport, distinct from source filtering.
    pub fn viewport(mut self, start: f64, end: f64) -> Self {
        match Bounds::new(start, end) {
            Ok(v) => self.spec.viewport = Some(v),
            Err(e) => self.failure = Some(e),
        };
        self
    }
    /// Set an explicit destination range.
    pub fn range(mut self, start: f64, end: f64) -> Self {
        match Bounds::new(start, end) {
            Ok(v) => self.spec.range = Some(v),
            Err(e) => self.failure = Some(e),
        };
        self
    }
    /// Set outside-domain projection behavior.
    pub fn outside(mut self, outside: OutsidePolicy) -> Self {
        self.spec.outside = outside;
        self
    }
    /// Supply semantic tick values and labels; duplicates/empty text retain their meaning.
    pub fn ticks(mut self, ticks: impl IntoIterator<Item = (ScaleValue, String)>) -> Self {
        self.spec.guide_ticks = Some(
            ticks
                .into_iter()
                .map(|(value, label)| CustomGuideTick { value, label })
                .collect(),
        );
        self
    }
    /// Set an explicit portable numeric tick formatter.
    pub fn format(mut self, format: NumberFormatBuilder) -> Self {
        self.spec.number_format = Some(format.value);
        self
    }
    /// Show an affine alternate-unit guide over a named primary numeric scale.
    pub fn secondary(mut self, source: impl Into<String>, factor: f64, offset: f64) -> Self {
        self.secondary = Some((source.into(), factor, offset));
        self
    }
    pub(super) fn lower(mut self, axes: &BTreeMap<String, ScaleId>) -> ChartResult<AxisSpec> {
        if let Some(e) = self.failure {
            return Err(e);
        }
        if let Some((source, factor, offset)) = self.secondary {
            self.spec.scale = AxisScale::Secondary {
                source: *axes.get(&source).ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        format!("No primary axis named '{source}'."),
                    )
                })?,
                factor,
                offset,
            };
        }
        Ok(self.spec)
    }
}
/// Portable numeric tick formatting options.
#[derive(Clone, Debug)]
pub struct NumberFormatBuilder {
    value: NumberFormat,
}
/// Fixed two-decimal formatting with explicit en-US punctuation.
pub fn number_format() -> NumberFormatBuilder {
    NumberFormatBuilder {
        value: NumberFormat {
            notation: NumberNotation::Fixed,
            precision: 2,
            locale: NumberLocale::EnUs,
            grouping: false,
            prefix: String::new(),
            suffix: String::new(),
        },
    }
}
impl NumberFormatBuilder {
    /// Set fixed/scientific/percent notation.
    pub fn notation(mut self, notation: NumberNotation) -> Self {
        self.value.notation = notation;
        self
    }
    /// Set decimal precision (0..12).
    pub fn precision(mut self, precision: u8) -> Self {
        self.value.precision = precision;
        self
    }
    /// Set explicit punctuation locale.
    pub fn locale(mut self, locale: NumberLocale) -> Self {
        self.value.locale = locale;
        self
    }
    /// Enable integer digit grouping.
    pub fn grouping(mut self, enabled: bool) -> Self {
        self.value.grouping = enabled;
        self
    }
    /// Set a bounded fixed prefix.
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.value.prefix = prefix.into();
        self
    }
    /// Set a bounded fixed suffix.
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.value.suffix = suffix.into();
        self
    }
}
