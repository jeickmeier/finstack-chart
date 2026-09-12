//! R point-shape topology expressed through the shared path destination.
//! Size is the equivalent circle area; this preserves the reference's relative glyph extents.
use crate::{ChartResult, path::Path};
use std::f64::consts::{PI, SQRT_2, TAU};
fn segment(path: &mut Path, a: [f64; 2], b: [f64; 2]) -> ChartResult<()> {
    path.move_to(a[0], a[1])?;
    path.line_to(b[0], b[1])
}
fn polygon(path: &mut Path, points: &[[f64; 2]]) -> ChartResult<()> {
    path.move_to(points[0][0], points[0][1])?;
    for point in &points[1..] {
        path.line_to(point[0], point[1])?;
    }
    path.close_path()
}
fn circle(path: &mut Path, radius: f64) -> ChartResult<()> {
    path.move_to(radius, 0.)?;
    path.arc([0., 0.], radius, 0., TAU, false)
}
fn cross(path: &mut Path, radius: f64, diagonal: bool) -> ChartResult<()> {
    if diagonal {
        segment(path, [-radius, -radius], [radius, radius])?;
        segment(path, [-radius, radius], [radius, -radius])
    } else {
        segment(path, [-radius, 0.], [radius, 0.])?;
        segment(path, [0., -radius], [0., radius])
    }
}
fn diamond(path: &mut Path, radius: f64) -> ChartResult<()> {
    polygon(
        path,
        &[[-radius, 0.], [0., -radius], [radius, 0.], [0., radius]],
    )
}
fn triangle(path: &mut Path, radius: f64, down: bool, overlap: bool) -> ChartResult<()> {
    let altitude = (4. * PI / (3. * 3_f64.sqrt())).sqrt() * radius;
    let half_width = altitude * 3_f64.sqrt() / 2.;
    let baseline = altitude * if overlap { 0.75 } else { 0.5 };
    let direction = if down { -1. } else { 1. };
    polygon(
        path,
        &[
            [0., -direction * altitude],
            [half_width, direction * baseline],
            [-half_width, direction * baseline],
        ],
    )
}
pub(super) fn draw(shape: u8, path: &mut Path, size: f64) -> ChartResult<()> {
    let radius = (size / PI).sqrt();
    match shape {
        0 | 15 => path.rect(-radius, -radius, 2. * radius, 2. * radius),
        1 | 16 | 19 | 21 => circle(path, radius),
        2 | 17 | 24 => triangle(path, radius, false, false),
        3 => cross(path, SQRT_2 * radius, false),
        4 => cross(path, radius, true),
        5 => diamond(path, SQRT_2 * radius),
        6 | 25 => triangle(path, radius, true, false),
        7 => {
            path.rect(-radius, -radius, 2. * radius, 2. * radius)?;
            cross(path, radius, true)
        }
        8 => {
            cross(path, radius, true)?;
            cross(path, SQRT_2 * radius, false)
        }
        9 => {
            cross(path, SQRT_2 * radius, false)?;
            diamond(path, SQRT_2 * radius)
        }
        10 => {
            circle(path, radius)?;
            cross(path, radius, false)
        }
        11 => {
            triangle(path, radius, true, true)?;
            triangle(path, radius, false, true)
        }
        12 => {
            cross(path, radius, false)?;
            path.rect(-radius, -radius, 2. * radius, 2. * radius)
        }
        13 => {
            circle(path, radius)?;
            cross(path, radius, true)
        }
        14 => {
            polygon(path, &[[0., -radius], [radius, radius], [-radius, radius]])?;
            path.rect(-radius, -radius, 2. * radius, 2. * radius)
        }
        18 => diamond(path, radius),
        20 => circle(path, 2. * radius / 3.),
        22 => {
            let half = radius * (PI / 4.).sqrt();
            path.rect(-half, -half, 2. * half, 2. * half)
        }
        23 => diamond(path, radius * (PI / 2.).sqrt()),
        _ => Err(super::invalid(
            "Ggplot point shape must be an integer from zero through 25.",
        )),
    }
}
