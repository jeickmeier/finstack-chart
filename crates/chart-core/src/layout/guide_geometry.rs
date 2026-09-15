//! Destination-only guide geometry. Mapping and domain training remain with scales.
use super::{
    AxisSide, GuideProfile, GuideSpec, GuideStyle, LayoutRequest, ResolvedAxis, ResolvedScale,
};
use crate::{ChartResult, DiagnosticCode, Point, Rect, scales::Bounds, scene::PathCommand};

/// Selection is immutable; adaptive presentation can hide labels or complete ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GuideLabelPolicy {
    /// Preserve requested ticks and labels, including collisions.
    Preserve,
    /// Hide colliding/outside labels while preserving tick rules and values.
    HideLabels,
    /// Explicitly omit colliding/outside ticks, retaining legacy adaptive behavior.
    ThinTicks,
}
/// Axis decorations have an explicit figure overflow policy independent of data clipping.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum GuideOverflow {
    /// Permit decoration geometry outside the layout cell.
    #[default]
    Visible,
    /// Clip decorations to the current figure/panel cell.
    Clip,
}
/// Optional controls inherit the guide profile; lengths and padding may be signed.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuideGeometry {
    /// Inner tick length; D3 default 6.
    pub inner: Option<f64>,
    /// Domain end-cap length; D3 default 6.
    pub outer: Option<f64>,
    /// Label spacing after max(inner, 0); D3 default 3.
    pub padding: Option<f64>,
    /// Explicit destination-unit pixel offset; automatic uses the supplied device scale.
    pub offset: Option<f64>,
    /// Profile default is Preserve for D3 and ThinTicks for LibraryV1.
    pub labels: Option<GuideLabelPolicy>,
    /// Decoration clipping; distinct from data clipping and label selection.
    #[serde(default)]
    pub overflow: GuideOverflow,
    /// Clip inner tick rules to the plot; labels/domain use overflow above.
    #[serde(default)]
    pub clip_ticks: bool,
}
impl GuideGeometry {
    pub(super) fn validate(&self) -> ChartResult<()> {
        if [self.inner, self.outer, self.padding, self.offset]
            .into_iter()
            .flatten()
            .any(|x| !x.is_finite())
        {
            return Err(crate::scales::error(
                DiagnosticCode::Validation,
                "Guide geometry must be finite; signed lengths and padding are supported.",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
pub(super) struct Geometry {
    pub inner: f64,
    pub outer: f64,
    pub padding: f64,
    pub offset: f64,
    pub labels: GuideLabelPolicy,
    pub overflow: GuideOverflow,
    pub clip_ticks: bool,
}
impl Geometry {
    pub fn resolve(style: &GuideStyle, request: &LayoutRequest) -> Self {
        let d3 = style.profile == GuideProfile::D3_3_0_0;
        let options = style.geometry.clone().unwrap_or_default();
        Self {
            inner: options
                .inner
                .unwrap_or(if d3 { 6. } else { request.tick_length }),
            outer: options.outer.unwrap_or(if d3 { 6. } else { 0. }),
            padding: options
                .padding
                .unwrap_or(if d3 { 3. } else { request.label_gap }),
            offset: options.offset.unwrap_or(
                if d3 && request.device_scale.is_none_or(|scale| scale <= 1.) {
                    0.5
                } else {
                    0.
                },
            ),
            labels: options
                .labels
                .unwrap_or(if d3 || style.ggplot_axis.is_some() {
                    GuideLabelPolicy::Preserve
                } else {
                    GuideLabelPolicy::ThinTicks
                }),
            overflow: options.overflow,
            clip_ticks: options.clip_ticks,
        }
    }
    pub fn spacing(self) -> f64 {
        self.inner.max(0.) + self.padding
    }
}
pub(super) fn range(
    axis: &ResolvedAxis,
    axes: &std::collections::BTreeMap<crate::ScaleId, ResolvedAxis>,
) -> Bounds {
    match &axis.scale {
        ResolvedScale::Unbounded(s) => s.range(),
        ResolvedScale::Linear(s) => s.range(),
        ResolvedScale::Numeric(s) => s.range(),
        ResolvedScale::Nonlinear(s) => s.range(),
        ResolvedScale::Band(s) => s.range(),
        ResolvedScale::Point(s) => s.range(),
        ResolvedScale::Provider(s) => s.range(),
        ResolvedScale::Utc(s) => s.range(),
        ResolvedScale::Calendar(s) => s.range(),
        ResolvedScale::Session(s) => s.range(),
        ResolvedScale::Secondary { source, .. }
        | ResolvedScale::SecondaryTime { source, .. }
        | ResolvedScale::SecondaryDiscrete { source, .. } => range(&axes[source], axes),
    }
}
pub(super) fn point(side: AxisSide, along: f64, outward: f64, plot: Rect) -> ChartResult<Point> {
    match side {
        AxisSide::Bottom => Point::new(along, plot.max_y() + outward),
        AxisSide::Top => Point::new(along, plot.origin().y() - outward),
        AxisSide::Left => Point::new(plot.origin().x() - outward, along),
        AxisSide::Right => Point::new(plot.max_x() + outward, along),
    }
}
pub(super) fn domain(
    spec: &GuideSpec,
    range: Bounds,
    plot: Rect,
    geometry: Geometry,
) -> ChartResult<Vec<PathCommand>> {
    let offset = geometry.offset;
    let translation = spec.translation[usize::from(!spec.side.horizontal())];
    let start = range.start() + translation + offset;
    let end = range.end() + translation + offset;
    let sign = if matches!(spec.side, AxisSide::Top | AxisSide::Left) {
        -1.
    } else {
        1.
    };
    let baseline = sign * offset;
    let mut commands = vec![PathCommand::MoveTo(point(
        spec.side,
        start,
        if geometry.outer == 0. {
            baseline
        } else {
            geometry.outer
        },
        plot,
    )?)];
    if geometry.outer != 0. {
        commands.push(PathCommand::LineTo(point(
            spec.side, start, baseline, plot,
        )?));
    }
    commands.push(PathCommand::LineTo(point(spec.side, end, baseline, plot)?));
    if geometry.outer != 0. {
        commands.push(PathCommand::LineTo(point(
            spec.side,
            end,
            geometry.outer,
            plot,
        )?));
    }
    Ok(commands)
}
