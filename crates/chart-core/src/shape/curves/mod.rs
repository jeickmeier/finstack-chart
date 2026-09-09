//! Streaming curve kernels adapted from d3-shape 3.2.0 (ISC; ../LICENSE).
mod basic;
mod basis;
mod monotone;
mod natural;
mod spline;

use super::{CurveProtocol, CurveSpec};
use crate::{ChartResult, path::Path};
type Point = [f64; 2];
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Standard,
    Open,
    Closed,
}
fn begin(path: &mut Path, line: Option<bool>, p: Point) -> ChartResult<()> {
    if line == Some(true) {
        path.line_to(p[0], p[1])
    } else {
        path.move_to(p[0], p[1])
    }
}
fn close(path: &mut Path, line: &mut Option<bool>, singleton: bool) -> ChartResult<()> {
    if *line == Some(true) || (line.is_none() && singleton) {
        path.close_path()?;
    }
    *line = line.map(|value| !value);
    Ok(())
}
fn cubic(path: &mut Path, a: Point, b: Point, c: Point) -> ChartResult<()> {
    path.bezier_curve_to(a[0], a[1], b[0], b[1], c[0], c[1])
}
pub(super) fn create(spec: CurveSpec, path: &mut Path) -> Box<dyn CurveProtocol + '_> {
    use CurveSpec::*;
    match spec {
        Linear | LinearClosed | Step | StepBefore | StepAfter | BumpX | BumpY => {
            Box::new(basic::Basic::new(path, spec))
        }
        Basis => Box::new(basis::Basis::new(path, Mode::Standard)),
        BasisOpen => Box::new(basis::Basis::new(path, Mode::Open)),
        BasisClosed => Box::new(basis::Basis::new(path, Mode::Closed)),
        Bundle { beta: 1. } => Box::new(basis::Basis::new(path, Mode::Standard)),
        Bundle { beta } => Box::new(basis::Bundle::new(path, beta)),
        Cardinal { tension } => Box::new(spline::Spline::cardinal(path, Mode::Standard, tension)),
        CardinalOpen { tension } => Box::new(spline::Spline::cardinal(path, Mode::Open, tension)),
        CardinalClosed { tension } => {
            Box::new(spline::Spline::cardinal(path, Mode::Closed, tension))
        }
        CatmullRom { alpha } => Box::new(spline::Spline::catmull_rom(path, Mode::Standard, alpha)),
        CatmullRomOpen { alpha } => Box::new(spline::Spline::catmull_rom(path, Mode::Open, alpha)),
        CatmullRomClosed { alpha } => {
            Box::new(spline::Spline::catmull_rom(path, Mode::Closed, alpha))
        }
        MonotoneX => Box::new(monotone::Monotone::new(path, false)),
        MonotoneY => Box::new(monotone::Monotone::new(path, true)),
        Natural => Box::new(natural::Natural::new(path)),
    }
}
