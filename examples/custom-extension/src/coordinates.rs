//! Paired nonlinear coordinate example with a closed-form inverse and fixed panel edges.
use chart_core::{ChartResult, Revision, grammar::*};
use std::sync::Arc;
/// Portable identity.
pub const WAVE: &str = "example.wave_coordinate";
/// Native-only counterpart.
pub const NATIVE_WAVE: &str = "example.native_wave_coordinate";
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    amplitude: f64,
}
struct Wave {
    portable: bool,
}
#[derive(Debug)]
struct TrainedWave {
    amplitude: f64,
}
impl TrainedCoordinate for TrainedWave {
    fn forward(&self, p: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        Ok(Some([
            p[0],
            p[1] + self.amplitude * libm::sin(std::f64::consts::TAU * p[0]),
        ]))
    }
    fn bounds(&self, input: [[f64; 2]; 2]) -> ChartResult<Option<[[f64; 2]; 2]>> {
        let angle = input[0].map(|x| std::f64::consts::TAU * x);
        let endpoints = angle.map(libm::sin);
        let mut low = endpoints[0].min(endpoints[1]);
        let mut high = endpoints[0].max(endpoints[1]);
        let tau = std::f64::consts::TAU;
        for (phase, value) in [
            (std::f64::consts::FRAC_PI_2, 1.),
            (3. * std::f64::consts::FRAC_PI_2, -1.),
        ] {
            if libm::ceil((angle[0] - phase) / tau) <= libm::floor((angle[1] - phase) / tau) {
                low = low.min(value);
                high = high.max(value);
            }
        }
        let a = low * self.amplitude;
        let b = high * self.amplitude;
        Ok(Some([
            input[0],
            [input[1][0] + a.min(b), input[1][1] + a.max(b)],
        ]))
    }
    fn has_inverse(&self) -> bool {
        true
    }
    fn inverse(&self, p: [f64; 2]) -> ChartResult<Option<[f64; 2]>> {
        Ok(Some([
            p[0],
            p[1] - self.amplitude * libm::sin(std::f64::consts::TAU * p[0]),
        ]))
    }
}
impl CustomCoordinate for Wave {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.portable { WAVE } else { NATIVE_WAVE },
            Revision::new(1),
            self.portable,
        )
    }
    fn validate(&self, p: &serde_json::Value) -> ChartResult<()> {
        let p: Parameters = serde_json::from_value(p.clone()).map_err(|_| {
            chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Wave coordinate requires only amplitude.",
                "Supply a finite amplitude in -0.5..0.5.",
            )
        })?;
        if !p.amplitude.is_finite() || p.amplitude.abs() > 0.5 {
            return Err(chart_core::Diagnostic::error(
                chart_core::DiagnosticCode::Validation,
                "Wave amplitude is outside -0.5..0.5.",
                "Supply a finite bounded amplitude.",
            ));
        }
        Ok(())
    }
    fn train(&self, input: CoordinateTrainInput<'_>) -> ChartResult<Arc<dyn TrainedCoordinate>> {
        self.validate(input.parameters)?;
        let p: Parameters = serde_json::from_value(input.parameters.clone()).expect("validated");
        Ok(Arc::new(TrainedWave {
            amplitude: p.amplitude,
        }))
    }
}
/// Install both identities into the common registry.
pub fn register(registry: &mut ExtensionRegistry) -> ChartResult<()> {
    registry.register_coordinate(Arc::new(Wave { portable: true }))?;
    registry.register_coordinate(Arc::new(Wave { portable: false }))
}
