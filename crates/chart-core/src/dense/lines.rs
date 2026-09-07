use super::{column, error};
use crate::scene::PathCommand;
use crate::{ChartResult, Point, Rect};

/// Select first/minimum-y/maximum-y/last in each consecutive horizontal bucket,
/// preserving original index order. Endpoints and every bucket's extrema survive.
/// Returns None for nonmonotonic x; callers must retain that exact run. One supplied
/// slice is one gap-free run; the function never joins separate runs.
pub fn line_envelope(
    points: &[Point],
    view: Rect,
    width: f64,
    max_columns: usize,
) -> ChartResult<Option<Vec<usize>>> {
    if !width.is_finite() || width <= 0. || !(1..=65_536).contains(&max_columns) {
        return Err(error("Invalid line-envelope bucket policy."));
    }
    column(view.max_x(), view, width, max_columns)?;
    let forward = points.windows(2).all(|p| p[0].x() <= p[1].x());
    let reverse = points.windows(2).all(|p| p[0].x() >= p[1].x());
    if !forward && !reverse {
        return Ok(None);
    }
    let mut out = vec![];
    let mut start = 0;
    while start < points.len() {
        let bucket = column(points[start].x(), view, width, max_columns)?;
        let mut end = start + 1;
        let (mut low, mut high) = (start, start);
        while end < points.len() && column(points[end].x(), view, width, max_columns)? == bucket {
            if points[end].y() < points[low].y() {
                low = end;
            }
            if points[end].y() > points[high].y() {
                high = end;
            }
            end += 1;
        }
        let mut chosen = [start, low, high, end - 1];
        chosen.sort_unstable();
        for i in chosen {
            if out.last() != Some(&i) {
                out.push(i);
            }
        }
        start = end;
    }
    Ok(Some(out))
}
pub(super) fn reduce_path(
    commands: &[PathCommand],
    view: Rect,
    width: f64,
    max: usize,
) -> ChartResult<Option<(Vec<PathCommand>, usize, usize)>> {
    let mut out = vec![];
    let mut run = vec![];
    let (mut samples, mut runs) = (0, 0);
    let mut flush = |run: &mut Vec<Point>| -> ChartResult<bool> {
        if run.is_empty() {
            return Ok(true);
        }
        let Some(indices) = line_envelope(run, view, width, max)? else {
            return Ok(false);
        };
        samples += run.len();
        runs += 1;
        for (j, i) in indices.into_iter().enumerate() {
            out.push(if j == 0 {
                PathCommand::MoveTo(run[i])
            } else {
                PathCommand::LineTo(run[i])
            });
        }
        run.clear();
        Ok(true)
    };
    for c in commands {
        match c {
            PathCommand::MoveTo(p) => {
                if !flush(&mut run)? {
                    return Ok(None);
                }
                run.push(*p);
            }
            PathCommand::LineTo(p) => run.push(*p),
            _ => return Ok(None),
        }
    }
    if !flush(&mut run)? {
        return Ok(None);
    }
    Ok(Some((out, samples, runs)))
}
