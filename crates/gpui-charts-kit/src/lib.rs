//! Optional Kit theme/control adapter. Core sees only serializable chart tokens/actions.
use chart_core::scene::Color;
use chart_core::theme::ThemePatch;
use chart_core::{ChartResult, Diagnostic, DiagnosticCode};
use gpui_charts::{ChartInput, ChartView, NativeFont};
use gpui_kit::component::{Theme, button::Button};
use gpui_kit::{Context, Entity};

fn color(c: gpui_kit::Hsla) -> ChartResult<Color> {
    let c: gpui_kit::Rgba = c.into();
    if [c.r, c.g, c.b, c.a]
        .iter()
        .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
    {
        return Err(Diagnostic::error(
            DiagnosticCode::Validation,
            "Kit color is not a finite normalized sRGB value.",
            "Provide valid semantic theme colors.",
        ));
    }
    let byte = |v: f32| (v * 255.).round() as u8;
    Ok(Color {
        red: byte(c.r),
        green: byte(c.g),
        blue: byte(c.b),
        alpha: byte(c.a),
    })
}
/// Copy Kit's current semantic token snapshot into the generic chart host cascade.
/// Font-family strings never trigger system font lookup: the caller keeps its explicit font bytes.
pub fn theme_patch(theme: &Theme) -> ChartResult<ThemePatch> {
    let t = theme.semantic_tokens();
    let patch = ThemePatch {
        background: Some(color(t.colors.background)?),
        panel: Some(color(t.colors.surface)?),
        foreground: Some(color(t.colors.foreground)?),
        grid: Some(color(t.colors.border)?),
        annotation: Some(color(t.colors.muted_foreground)?),
        focus: Some(color(t.colors.ring)?),
        selection: Some(color(t.colors.selection)?),
        font_size: Some(f64::from(f32::from(t.typography.sm.size))),
        padding: Some(f64::from(f32::from(t.spacing.sm))),
        gap: Some(f64::from(f32::from(t.spacing.xs))),
        ..ThemePatch::default()
    };
    patch.validate()?;
    Ok(patch)
}
/// Primary native mount with a captured Kit theme and explicitly supplied font resource.
/// The returned input keeps the regular native tooltip/control/accessibility builders.
pub fn chart_input(
    plot: &chart_core::plot::Plot,
    font: NativeFont,
    theme: &Theme,
) -> ChartResult<ChartInput> {
    Ok(ChartInput::from_plot(plot, font)?
        .layout(chart_core::plot::layout_options().host_theme(theme_patch(theme)?)))
}
/// Apply a Kit snapshot at the host level; authored named/plot/layer tokens keep their precedence.
pub fn apply_theme(
    view: &mut ChartView,
    theme: &Theme,
    cx: &mut Context<ChartView>,
) -> ChartResult<()> {
    let mut r = view.layout_request().clone();
    r.host_theme = theme_patch(theme)?;
    view.set_layout(r, cx)
}
/// A real Kit control dispatching through the shared chart reducer. Recoverable errors stay on
/// ChartView's existing diagnostic surface; the control creates no parallel interaction state.
pub fn reset_button(chart: Entity<ChartView>) -> Button {
    Button::new("chart-reset")
        .label("Reset view")
        .on_click(move |_, _, cx| {
            let _ = chart.update(cx, |view, cx| {
                view.dispatch_chart(chart_core::state::ChartAction::Reset, cx)
            });
        })
}
