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
/// Elapsed seconds with reference duration breaks and labels; no timezone is inferred.
pub fn scale_duration() -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Duration(ContinuousDomain::default())),
    }
}
/// Reference positional bins under the ggplot2 profile, before and after statistics.
pub fn scale_binned(spec: GgplotBinnedPosition) -> ScaleBuilder {
    ScaleBuilder {
        value: spec.validate().map(|_| AxisScale::Binned {
            spec: Box::new(spec),
            prepared: None,
        }),
    }
}
/// Authored D3-compatible numerical knots and outputs on a named positional axis.
pub fn scale_numeric(spec: NumericScaleSpec) -> ScaleBuilder {
    ScaleBuilder {
        value: NumericScale::new(spec.clone()).map(|_| AxisScale::Numeric(spec)),
    }
}
/// Select a versioned positional provider installed in the plot's extension registry.
pub fn scale_registered(
    id: impl Into<String>,
    version: crate::Revision,
    parameters: serde_json::Value,
) -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Registered {
            operation: crate::grammar::OperationRef::new(id, version),
            parameters,
        }),
    }
}
/// Nonnegative square-root positional transformation, before statistics under ggplot2.
pub fn scale_sqrt() -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Nonlinear {
            transform: ScaleTransform::Sqrt,
            domain: ContinuousDomain::default(),
        }),
    }
}
/// Reverse numeric coordinates using a shared pre-stat transform under the ggplot2 profile.
pub fn scale_reverse() -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Nonlinear {
            transform: ScaleTransform::Reverse,
            domain: ContinuousDomain::default(),
        }),
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
/// D3-compatible categorical bands, using the stable trained chart catalog by default.
pub fn scale_band_d3(spec: BandSpec) -> ScaleBuilder {
    ScaleBuilder {
        value: BandScale::resolve_d3(&[], &spec, Bounds::new(0., 1.).expect("finite"))
            .map(|_| AxisScale::D3Band(spec)),
    }
}
/// D3-compatible points with zero default padding, alignment and integer rounding.
pub fn scale_point_d3(spec: PointSpec) -> ScaleBuilder {
    ScaleBuilder {
        value: PointScale::resolve_d3(&[], &spec, Bounds::new(0., 1.).expect("finite"))
            .map(|_| AxisScale::D3Point(spec)),
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
/// Reference Date axes over UTC timestamps in the source's declared integer unit.
/// Expansion amounts are days; data and inspection retain exact timestamps.
pub fn scale_date() -> ScaleBuilder {
    ScaleBuilder {
        value: Ok(AxisScale::Date { domain: None }),
    }
}
/// D3-compatible exact timestamp knots with an explicit UTC or local calendar.
pub fn scale_calendar(spec: TimeScaleSpec) -> ScaleBuilder {
    ScaleBuilder {
        value: TimeAxisScale::resolve(
            spec.clone(),
            Bounds::new(0., 1.).expect("finite"),
            None,
            OutsidePolicy::Extend,
        )
        .map(|_| AxisScale::Calendar {
            spec,
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
    /// Explicit calendar interval for an authored calendar time scale.
    pub fn calendar_interval(mut self, interval: CalendarInterval) -> Self {
        self.value = self.value.and_then(|mut scale| {
            interval.validate()?;
            let AxisScale::Calendar {
                interval: target, ..
            } = &mut scale
            else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Calendar intervals require a calendar scale.",
                ));
            };
            *target = Some(interval);
            Ok(scale)
        });
        self
    }

    fn numeric(mut self, change: impl FnOnce(&mut ContinuousDomain) -> ChartResult<()>) -> Self {
        self.value = self.value.and_then(|mut scale| {
            match &mut scale {
                AxisScale::Linear(domain)
                | AxisScale::Duration(domain)
                | AxisScale::Nonlinear { domain, .. } => change(domain)?,
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
                AxisScale::Linear(d)
                | AxisScale::Duration(d)
                | AxisScale::Nonlinear { domain: d, .. } => d.padding = padding,
                AxisScale::Point(d) => d.padding = padding,
                AxisScale::D3Point(d) => d.padding = padding,
                AxisScale::D3Band(d) => {
                    d.padding_inner = padding;
                    d.padding_outer = padding;
                }
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
                AxisScale::D3Point(d) => d.domain = Some(values),
                AxisScale::D3Band(d) => d.domain = Some(values),
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
            match &mut scale {
                AxisScale::Band(d) => {
                    d.inner_padding = inner;
                    d.outer_padding = outer;
                }
                AxisScale::D3Band(d) => {
                    d.padding_inner = inner;
                    d.padding_outer = outer;
                }
                _ => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Band padding requires a band scale.",
                    ));
                }
            }
            Ok(scale)
        });
        self
    }
    /// Set outer step padding for a point scale.
    pub fn point_padding(mut self, padding: f64) -> Self {
        self.value = self.value.and_then(|mut scale| {
            match &mut scale {
                AxisScale::Point(d) => d.padding = padding,
                AxisScale::D3Point(d) => d.padding = padding,
                _ => {
                    return Err(error(
                        DiagnosticCode::UnsupportedCapability,
                        "Point padding requires a point scale.",
                    ));
                }
            }
            Ok(scale)
        });
        self
    }
    /// Exact UTC endpoints in the mapped field's original integer unit.
    pub fn time_domain(mut self, start: i64, end: i64) -> Self {
        self.value = self.value.and_then(|mut scale| {
            let (AxisScale::Utc { domain, .. } | AxisScale::Date { domain }) = &mut scale else {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Exact time domains require a UTC or Date scale.",
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
    secondary_transform: Option<Box<NumericScaleSpec>>,
}
/// Primary bottom x axis.
pub fn x_axis() -> AxisBuilder {
    AxisBuilder {
        name: "x".into(),
        spec: AxisSpec::new(ScaleId::new(0), AxisSide::Bottom),
        failure: None,
        secondary: None,
        secondary_transform: None,
    }
}
/// Primary left y axis.
pub fn y_axis() -> AxisBuilder {
    AxisBuilder {
        name: "y".into(),
        spec: AxisSpec::new(ScaleId::new(1), AxisSide::Left),
        failure: None,
        secondary: None,
        secondary_transform: None,
    }
}
impl AxisBuilder {
    /// Set factor levels, nullable category limits and reference guide selection.
    pub fn discrete_policy(
        mut self,
        policy: Option<crate::scales::GgplotDiscretePosition>,
    ) -> Self {
        self.spec.discrete = policy.map(Box::new);
        self
    }
    /// Set reference continuous limits independently from category identity.
    pub fn continuous_limits(mut self, limits: Option<Vec<crate::interpolate::Number>>) -> Self {
        self.spec.continuous_limits = limits;
        self
    }
    /// Override reference expansion; None restores the selected profile default.
    /// Numeric amounts use transformed units, categories use index units, and datetime
    /// amounts use elapsed seconds regardless of the timestamp source unit.
    pub fn expansion(mut self, expansion: Option<GgplotExpansion>) -> Self {
        self.spec.expansion = expansion;
        self
    }

    /// Apply this transform only at coordinate projection, after statistics.
    pub fn coordinate_scale(mut self, scale: ScaleBuilder) -> Self {
        self.spec.scale_stage = Some(crate::grammar::ScaleStage::AfterStatistics);
        self.scale(scale)
    }
    /// Explicit population handling outside scale limits; viewport clipping is separate.
    pub fn oob(mut self, policy: crate::grammar::ScaleOob) -> Self {
        self.spec.population_oob = Some(policy);
        self
    }
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
    /// Select guide policy without changing its shared scale or data-domain training.
    pub fn guide_profile(mut self, profile: crate::layout::GuideProfile) -> Self {
        self.spec.profile = profile;
        self
    }
    /// Replace independent component styles; None restores profile/theme inheritance.
    pub fn guide_components(mut self, components: Option<crate::layout::GuideComponents>) -> Self {
        self.spec.components = components;
        self
    }
    /// Replace independent geometry; None resets all controls to the guide profile.
    pub fn guide_geometry(mut self, geometry: Option<crate::layout::GuideGeometry>) -> Self {
        self.spec.geometry = geometry;
        self
    }
    /// Set inner ticks and domain caps together in destination units.
    pub fn tick_size(mut self, size: f64) -> Self {
        let geometry = self.spec.geometry.get_or_insert_with(Default::default);
        geometry.inner = Some(size);
        geometry.outer = Some(size);
        self
    }
    /// Set signed inner tick length independently of domain caps.
    pub fn tick_size_inner(mut self, size: f64) -> Self {
        self.spec
            .geometry
            .get_or_insert_with(Default::default)
            .inner = Some(size);
        self
    }
    /// Set signed domain cap length independently of inner ticks.
    pub fn tick_size_outer(mut self, size: f64) -> Self {
        self.spec
            .geometry
            .get_or_insert_with(Default::default)
            .outer = Some(size);
        self
    }
    /// Set signed padding after max(inner tick length, zero).
    pub fn tick_padding(mut self, padding: f64) -> Self {
        self.spec
            .geometry
            .get_or_insert_with(Default::default)
            .padding = Some(padding);
        self
    }
    /// Set an explicit destination-unit offset; None restores the host/profile policy.
    pub fn tick_offset(mut self, offset: Option<f64>) -> Self {
        self.spec
            .geometry
            .get_or_insert_with(Default::default)
            .offset = offset;
        self
    }
    /// Replace per-guide density/interval/format hints; None restores profile defaults.
    pub fn tick_arguments(mut self, arguments: Option<GuideTickArguments>) -> Self {
        self.spec.tick_arguments = arguments;
        self
    }
    /// Select reference minor candidates independently of major values and labels.
    pub fn minor_breaks(mut self, policy: Option<crate::layout::MinorBreaks>) -> Self {
        self.spec.minor_breaks = policy;
        self
    }
    /// Replace typed tick values independently of formatting. None restores automatic values.
    pub fn tick_values(mut self, values: Option<Vec<ScaleValue>>) -> Self {
        self.spec.tick_values = values;
        self.spec.guide_ticks = None;
        self
    }
    /// Replace the formatter independently of values. None restores scale-default formatting.
    pub fn tick_format(mut self, format: Option<crate::layout::GuideFormatter>) -> Self {
        self.spec.tick_format = format;
        self.spec.number_format = None;
        self.spec.numeric_format = None;
        self.spec.time_format = None;
        self
    }
    /// Supply coupled legacy tick values and labels; the D3 profile preserves empty labels.
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
    /// Conditional or custom time labels using the axis calendar resource.
    pub fn time_format(mut self, format: TimeFormat) -> Self {
        self.spec.time_format = Some(format);
        self
    }
    /// Apply a D3 numeric specifier; missing precision is inferred from the visible tick step.
    pub fn numeric_format(mut self, format: crate::typography::NumericFormat) -> Self {
        self.spec.numeric_format = Some(format);
        self
    }
    /// Show an alternate-unit guide over a named primary scale. Numeric axes support
    /// affine conversion; time axes require factor one and an additive offset in
    /// seconds (datetime) or days (Date), exactly representable in source units.
    pub fn secondary(mut self, source: impl Into<String>, factor: f64, offset: f64) -> Self {
        self.secondary = Some((source.into(), factor, offset));
        self.secondary_transform = None;
        self
    }
    /// Independent alternate units using a strictly monotone shared numeric mapping.
    pub fn secondary_transform(
        mut self,
        source: impl Into<String>,
        transform: NumericScaleSpec,
    ) -> Self {
        self.secondary = Some((source.into(), 1., 0.));
        self.secondary_transform = Some(Box::new(transform));
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
                transform: self.secondary_transform,
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

/// Independent guide identity; layer bindings and navigation retain positional `AxisHandle`s.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuideHandle(pub(super) crate::GuideId);
impl GuideHandle {
    /// Exact retained guide identity.
    pub fn id(self) -> crate::GuideId {
        self.0
    }
}
impl AxisHandle {
    /// Stable identity of this positional scale's default guide.
    pub fn guide(self) -> GuideHandle {
        GuideHandle(crate::GuideId::new(self.0.get()))
    }
}
/// An independently placed guide over an existing named positional scale.
#[derive(Clone, Debug)]
pub struct GuideBuilder {
    pub(super) name: String,
    pub(super) spec: crate::layout::GuideSpec,
    pub(super) failure: Option<crate::Diagnostic>,
    source: String,
}
/// Add a guide by name over an existing scale; its default placement is the bottom edge.
pub fn axis_guide(name: impl Into<String>, source: impl Into<String>) -> GuideBuilder {
    let id = fresh_id();
    GuideBuilder {
        name: name.into(),
        source: source.into(),
        spec: crate::layout::GuideSpec::new(
            crate::GuideId::new(id.clone().unwrap_or(0)),
            ScaleId::new(0),
            AxisSide::Bottom,
        ),
        failure: id.err(),
    }
}
impl GuideBuilder {
    /// Resolve this builder's independent identity without resolving its named scale.
    pub fn handle(&self) -> ChartResult<GuideHandle> {
        self.failure
            .clone()
            .map_or(Ok(GuideHandle(self.spec.id)), Err)
    }
    /// Replace the referenced scale without changing this guide's identity or presentation.
    pub fn scale(mut self, name: impl Into<String>) -> Self {
        self.source = name.into();
        self
    }
    /// Set the edge; its orientation must agree with the referenced positional scale.
    pub fn side(mut self, side: AxisSide) -> Self {
        self.spec.side = side;
        self
    }
    /// Translate only this guide in explicit destination units.
    pub fn translate(mut self, x: f64, y: f64) -> Self {
        self.spec.translation = [x, y];
        self
    }
    /// Set a plain guide title.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.spec.title = Some(plain(label));
        self
    }
    /// Set a rich guide title.
    pub fn rich_label(mut self, text: impl Into<RichText>) -> Self {
        self.spec.title = Some(text.into());
        self
    }
    /// Set tick-label typography.
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.spec.typography = Some(style.run);
        self
    }
    /// Set clockwise tick-label rotation in degrees.
    pub fn rotation(mut self, degrees: f64) -> Self {
        self.spec.label_rotation = degrees;
        self
    }
    /// Show or hide this guide independently of its scale and the other guides.
    pub fn visible(mut self, visible: bool) -> Self {
        self.spec.visible = visible;
        self
    }
    /// Select guide policy without changing its shared scale or data-domain training.
    pub fn guide_profile(mut self, profile: crate::layout::GuideProfile) -> Self {
        self.spec.profile = profile;
        self
    }
    /// Replace independent component styles; None restores profile/theme inheritance.
    pub fn guide_components(mut self, components: Option<crate::layout::GuideComponents>) -> Self {
        self.spec.components = components;
        self
    }
    /// Replace independent geometry; None resets all controls to the guide profile.
    pub fn guide_geometry(mut self, geometry: Option<crate::layout::GuideGeometry>) -> Self {
        self.spec.geometry = geometry;
        self
    }
    /// Set inner ticks and domain caps together in destination units.
    pub fn tick_size(mut self, size: f64) -> Self {
        let geometry = self.spec.geometry.get_or_insert_with(Default::default);
        geometry.inner = Some(size);
        geometry.outer = Some(size);
        self
    }
    /// Set signed inner tick length independently of domain caps.
    pub fn tick_size_inner(mut self, size: f64) -> Self {
        self.spec
            .geometry
            .get_or_insert_with(Default::default)
            .inner = Some(size);
        self
    }
    /// Set signed domain cap length independently of inner ticks.
    pub fn tick_size_outer(mut self, size: f64) -> Self {
        self.spec
            .geometry
            .get_or_insert_with(Default::default)
            .outer = Some(size);
        self
    }
    /// Set signed padding after max(inner tick length, zero).
    pub fn tick_padding(mut self, padding: f64) -> Self {
        self.spec
            .geometry
            .get_or_insert_with(Default::default)
            .padding = Some(padding);
        self
    }
    /// Set an explicit destination-unit offset; None restores the host/profile policy.
    pub fn tick_offset(mut self, offset: Option<f64>) -> Self {
        self.spec
            .geometry
            .get_or_insert_with(Default::default)
            .offset = offset;
        self
    }
    /// Replace per-guide density/interval/format hints; None restores profile defaults.
    pub fn tick_arguments(mut self, arguments: Option<GuideTickArguments>) -> Self {
        self.spec.tick_arguments = arguments;
        self
    }
    /// Select reference minor candidates independently of major values and labels.
    pub fn minor_breaks(mut self, policy: Option<crate::layout::MinorBreaks>) -> Self {
        self.spec.minor_breaks = policy;
        self
    }
    /// Replace typed tick values independently of formatting. None restores automatic values.
    pub fn tick_values(mut self, values: Option<Vec<ScaleValue>>) -> Self {
        self.spec.tick_values = values;
        self.spec.guide_ticks = None;
        self
    }
    /// Replace the formatter independently of values. None restores scale-default formatting.
    pub fn tick_format(mut self, format: Option<crate::layout::GuideFormatter>) -> Self {
        self.spec.tick_format = format;
        self.spec.number_format = None;
        self.spec.numeric_format = None;
        self.spec.time_format = None;
        self
    }
    /// Supply coupled legacy tick values and labels; the D3 profile preserves empty labels.
    pub fn ticks(mut self, ticks: impl IntoIterator<Item = (ScaleValue, String)>) -> Self {
        self.spec.guide_ticks = Some(
            ticks
                .into_iter()
                .map(|(value, label)| CustomGuideTick { value, label })
                .collect(),
        );
        self
    }
    /// Set a portable numeric formatter.
    pub fn format(mut self, format: NumberFormatBuilder) -> Self {
        self.spec.number_format = Some(format.value);
        self
    }
    /// Use a shared numeric specifier with inferred precision.
    pub fn numeric_format(mut self, format: crate::typography::NumericFormat) -> Self {
        self.spec.numeric_format = Some(format);
        self
    }
    /// Format exact timestamp values using the referenced scale's explicit calendar.
    pub fn time_format(mut self, format: TimeFormat) -> Self {
        self.spec.time_format = Some(format);
        self
    }
    pub(super) fn lower(
        mut self,
        axes: &BTreeMap<String, ScaleId>,
    ) -> ChartResult<crate::layout::GuideSpec> {
        if let Some(e) = self.failure {
            return Err(e);
        }
        self.spec.scale = *axes.get(&self.source).ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("No positional scale named '{}'.", self.source),
            )
        })?;
        Ok(self.spec)
    }
}
