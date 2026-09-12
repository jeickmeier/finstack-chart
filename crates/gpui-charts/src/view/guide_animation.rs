//! One host clock and one retained plan per chart; no tick entities or worker callbacks.
use super::*;
use std::time::{Duration, Instant};
#[derive(Default)]
pub(super) struct AnimationState {
    pub duration: Duration,
    pub running: Option<Running>,
}
pub(super) struct Running {
    plan: chart_core::layout::LayoutGuideTransition,
    target: Rc<NativeFrame>,
    start: Instant,
}
impl ChartInput {
    /// Opt into shared axis updates with a linear duration. Zero means immediate updates.
    /// GPUI's reduced-motion setting always selects the final frame immediately.
    pub fn guide_transition(mut self, duration: Duration) -> Self {
        self.guide_duration = duration;
        self
    }
}
impl ChartView {
    /// Replace the host duration; zero also completes any in-flight guide update.
    pub fn set_guide_transition(&mut self, duration: Duration, cx: &mut Context<Self>) {
        self.guide_animation.duration = duration;
        cx.notify();
    }
    pub(super) fn begin_guide_animation(
        &mut self,
        target: Rc<NativeFrame>,
        window: &Window,
        reduced_motion: bool,
    ) -> ChartResult<Rc<NativeFrame>> {
        let prior = self.frame.clone();
        self.guide_animation.running = None;
        self.cached = Some(target.clone());
        if self.guide_animation.duration.is_zero() || self.scheduling_metrics().disposed {
            return Ok(target);
        }
        let Some(prior) = prior else {
            return Ok(target);
        };
        if target.chart.guide_presentation().is_empty() {
            return Ok(target);
        }
        let plan = match chart_core::layout::LayoutGuideTransition::new(
            prior.chart.clone(),
            target.chart.clone(),
            target.request.limits,
        ) {
            Ok(plan) => plan,
            Err(e) if e.code == chart_core::DiagnosticCode::UnsupportedCapability => {
                return Ok(target);
            }
            Err(e) => return Err(e),
        };
        self.guide_animation.running = Some(Running {
            plan,
            target,
            start: Instant::now(),
        });
        self.sample_guide_animation(window, reduced_motion)
    }
    pub(super) fn sample_guide_animation(
        &mut self,
        window: &Window,
        reduced_motion: bool,
    ) -> ChartResult<Rc<NativeFrame>> {
        let Some(running) = &self.guide_animation.running else {
            return self.cached.clone().ok_or_else(|| {
                crate::native::error(
                    chart_core::DiagnosticCode::Validation,
                    "No guide presentation target.",
                )
            });
        };
        let fraction = if reduced_motion
            || self.guide_animation.duration.is_zero()
            || self.scheduling_metrics().disposed
        {
            1.
        } else {
            (running.start.elapsed().as_secs_f64() / self.guide_animation.duration.as_secs_f64())
                .min(1.)
        };
        let chart = running.plan.sample(fraction)?;
        let mut frame = NativeFrame::from_layout(
            chart,
            running.target.request.clone(),
            &self.font,
            &self.painters,
            &self.density,
            running.target.bounds,
            window,
        )?;
        frame.job = running.target.job;
        let frame = Rc::new(frame);
        if fraction == 1. {
            self.guide_animation.running = None;
            self.cached = Some(frame.clone());
        }
        Ok(frame)
    }
}
