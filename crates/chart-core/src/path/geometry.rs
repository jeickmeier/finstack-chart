// Finite-input formulas/state adapted from d3-path 3.1.0, Copyright Mike Bostock,
// ISC license retained at fixtures/parity/d3-path/LICENSE.
use super::{ChartResult, Command, PathOp, State, domain, finite};
use std::f64::consts::{PI, TAU};
const EPSILON: f64 = 1e-6;

pub(super) fn evaluate(mut state: State, op: &PathOp) -> ChartResult<(State, Vec<Command>)> {
    let mut commands = Vec::with_capacity(5);
    match *op {
        PathOp::MoveTo(p) => {
            finite(&p)?;
            state.start = Some(p);
            state.current = Some(p);
            commands.push(Command::MoveTo(p));
        }
        PathOp::LineTo(p) => {
            finite(&p)?;
            state.current = Some(p);
            commands.push(Command::LineTo(p));
        }
        PathOp::QuadraticCurveTo(p) => {
            finite(&p)?;
            state.current = Some([p[2], p[3]]);
            commands.push(Command::QuadraticTo(p));
        }
        PathOp::BezierCurveTo(p) => {
            finite(&p)?;
            state.current = Some([p[4], p[5]]);
            commands.push(Command::CubicTo(p));
        }
        PathOp::ClosePath => {
            if state.current.is_some() {
                state.current = state.start;
                commands.push(Command::Close);
            }
        }
        PathOp::Rect([x, y, w, h]) => {
            finite(&[x, y, w, h, x + w, y + h])?;
            state.current = Some([x, y]);
            state.start = Some([x, y]);
            commands.extend([
                Command::MoveTo([x, y]),
                Command::Horizontal(w),
                Command::Vertical(h),
                Command::Horizontal(-w),
                Command::Close,
            ]);
        }
        PathOp::Arc {
            x,
            y,
            r,
            a0,
            a1,
            anticlockwise,
        } => {
            finite(&[x, y, r, a0, a1])?;
            if r < 0.0 {
                return Err(domain("Negative arc radius"));
            }
            let dx = r * a0.cos();
            let dy = r * a0.sin();
            let first = [x + dx, y + dy];
            let mut da = if anticlockwise { a0 - a1 } else { a1 - a0 };
            finite(&[first[0], first[1], da])?;
            if let Some(p) = state.current {
                finite(&[p[0] - first[0], p[1] - first[1]])?;
            }
            match state.current {
                None => commands.push(Command::MoveTo(first)),
                Some(p)
                    if (p[0] - first[0]).abs() > EPSILON || (p[1] - first[1]).abs() > EPSILON =>
                {
                    commands.push(Command::LineTo(first))
                }
                _ => (),
            }
            // Intentionally do not update authoring current/start for the implicit connection.
            if r != 0.0 {
                if da < 0.0 {
                    da = da % TAU + TAU;
                }
                if da > TAU - EPSILON {
                    let opposite = [x - dx, y - dy];
                    finite(&opposite)?;
                    commands.push(Command::Arc {
                        radius: r,
                        large: true,
                        clockwise: !anticlockwise,
                        to: opposite,
                    });
                    commands.push(Command::Arc {
                        radius: r,
                        large: true,
                        clockwise: !anticlockwise,
                        to: first,
                    });
                    state.current = Some(first);
                } else if da > EPSILON {
                    let last = [x + r * a1.cos(), y + r * a1.sin()];
                    finite(&last)?;
                    commands.push(Command::Arc {
                        radius: r,
                        large: da >= PI,
                        clockwise: !anticlockwise,
                        to: last,
                    });
                    state.current = Some(last);
                }
            }
        }
        PathOp::ArcTo([x1, y1, x2, y2, r]) => {
            finite(&[x1, y1, x2, y2, r])?;
            if r < 0.0 {
                return Err(domain("Negative tangent arc radius"));
            }
            let Some([x0, y0]) = state.current else {
                state.current = Some([x1, y1]);
                commands.push(Command::MoveTo([x1, y1]));
                return Ok((state, commands));
            };
            let x21 = x2 - x1;
            let y21 = y2 - y1;
            let x01 = x0 - x1;
            let y01 = y0 - y1;
            let l01_2 = x01 * x01 + y01 * y01;
            let cross = y01 * x21 - y21 * x01;
            finite(&[x21, y21, x01, y01, l01_2, cross])?;
            if l01_2 <= EPSILON {
                return Ok((state, commands));
            }
            if cross.abs() <= EPSILON || r == 0.0 {
                state.current = Some([x1, y1]);
                commands.push(Command::LineTo([x1, y1]));
            } else {
                let x20 = x2 - x0;
                let y20 = y2 - y0;
                let l21_2 = x21 * x21 + y21 * y21;
                let l20_2 = x20 * x20 + y20 * y20;
                let l21 = l21_2.sqrt();
                let l01 = l01_2.sqrt();
                let cosine = (l21_2 + l01_2 - l20_2) / (2.0 * l21 * l01);
                let l = r * ((PI - cosine.acos()) / 2.0).tan();
                let t01 = l / l01;
                let t21 = l / l21;
                let first = [x1 + t01 * x01, y1 + t01 * y01];
                let last = [x1 + t21 * x21, y1 + t21 * y21];
                finite(&[
                    l21_2,
                    l20_2,
                    cosine,
                    l,
                    t01,
                    t21,
                    first[0],
                    first[1],
                    last[0],
                    last[1],
                    y01 * x20,
                    x01 * y20,
                ])?;
                if (t01 - 1.0).abs() > EPSILON {
                    commands.push(Command::LineTo(first));
                }
                commands.push(Command::Arc {
                    radius: r,
                    large: false,
                    clockwise: y01 * x20 > x01 * y20,
                    to: last,
                });
                state.current = Some(last);
            }
        }
    }
    Ok((state, commands))
}
