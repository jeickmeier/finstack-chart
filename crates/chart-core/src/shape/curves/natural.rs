use super::*;
pub(super) struct Natural<'a> {
    path: &'a mut Path,
    line: Option<bool>,
    points: Vec<Point>,
}
impl<'a> Natural<'a> {
    pub(super) fn new(path: &'a mut Path) -> Self {
        Self {
            path,
            line: None,
            points: Vec::new(),
        }
    }
}
fn controls(points: &[Point]) -> (Vec<Point>, Vec<Point>) {
    let n = points.len() - 1;
    let mut a = vec![[0.; 2]; n];
    let mut b = vec![0.; n];
    let mut r = vec![[0.; 2]; n];
    b[0] = 2.;
    for j in 0..2 {
        r[0][j] = points[0][j] + 2. * points[1][j];
    }
    for i in 1..n - 1 {
        b[i] = 4.;
        for j in 0..2 {
            r[i][j] = 4. * points[i][j] + 2. * points[i + 1][j];
        }
    }
    b[n - 1] = 7.;
    for j in 0..2 {
        r[n - 1][j] = 8. * points[n - 1][j] + points[n][j];
    }
    for i in 1..n {
        let m = if i == n - 1 { 2. } else { 1. } / b[i - 1];
        b[i] -= m;
        let previous = r[i - 1];
        for (value, previous) in r[i].iter_mut().zip(previous) {
            *value -= m * previous;
        }
    }
    for j in 0..2 {
        a[n - 1][j] = r[n - 1][j] / b[n - 1];
    }
    for i in (0..n - 1).rev() {
        for j in 0..2 {
            a[i][j] = (r[i][j] - a[i + 1][j]) / b[i];
        }
    }
    let mut second = vec![[0.; 2]; n];
    for j in 0..2 {
        second[n - 1][j] = (points[n][j] + a[n - 1][j]) / 2.;
    }
    for i in 0..n - 1 {
        for j in 0..2 {
            second[i][j] = 2. * points[i + 1][j] - a[i + 1][j];
        }
    }
    (a, second)
}
impl CurveProtocol for Natural<'_> {
    fn area_start(&mut self) -> ChartResult<()> {
        self.line = Some(false);
        Ok(())
    }
    fn area_end(&mut self) -> ChartResult<()> {
        self.line = None;
        Ok(())
    }
    fn line_start(&mut self) -> ChartResult<()> {
        self.points.clear();
        Ok(())
    }
    fn line_end(&mut self) -> ChartResult<()> {
        let n = self.points.len();
        if n > 0 {
            begin(self.path, self.line, self.points[0])?;
            if n == 2 {
                self.path.line_to(self.points[1][0], self.points[1][1])?;
            } else if n > 2 {
                let (a, b) = controls(&self.points);
                for i in 0..n - 1 {
                    cubic(self.path, a[i], b[i], self.points[i + 1])?;
                }
            }
        }
        close(self.path, &mut self.line, n == 1)?;
        self.points.clear();
        Ok(())
    }
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()> {
        self.points.push([x, y]);
        Ok(())
    }
}
