//! Shared reference approximation knot ordering and duplicate reduction.
/// Inputs have already passed the caller's population and resource validation.
/// Interpolation remains owned by the existing numeric/palette engines.
pub(crate) fn knots(mut pairs: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    pairs.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut result = Vec::with_capacity(pairs.len());
    let mut begin = 0;
    while begin < pairs.len() {
        let end = begin + 1 + pairs[begin + 1..].partition_point(|p| p[0] == pairs[begin][0]);
        let count = (end - begin) as f64;
        let sum = pairs[begin..end].iter().map(|p| p[1]).sum::<f64>();
        let mean = if sum.is_finite() {
            sum / count
        } else {
            pairs[begin..end].iter().map(|p| p[1] / count).sum::<f64>()
        };
        result.push([pairs[begin][0], mean]);
        begin = end;
    }
    result
}
