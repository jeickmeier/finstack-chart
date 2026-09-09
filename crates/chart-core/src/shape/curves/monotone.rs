use super::*;
pub(super) struct Monotone<'a> {
    path: &'a mut Path,
    reflect: bool,
    line: Option<bool>,
    count: u8,
    previous: [Point; 2],
    t: f64,
}
impl<'a> Monotone<'a> {
    pub(super) fn new(path: &'a mut Path, reflect: bool) -> Self {
        Self {
            path,
            reflect,
            line: None,
            count: 0,
            previous: [[f64::NAN; 2]; 2],
            t: f64::NAN,
        }
    }
    fn reflect(&self, p: Point) -> Point {
        if self.reflect { [p[1], p[0]] } else { p }
    }
    fn slope3(&self, p: Point) -> f64 {
        let [a, b] = self.previous;
        let h0 = b[0] - a[0];
        let h1 = p[0] - b[0];
        let s0 = (b[1] - a[1])
            / if h0 != 0. {
                h0
            } else if h1 < 0. {
                -0.
            } else {
                0.
            };
        let s1 = (p[1] - b[1])
            / if h1 != 0. {
                h1
            } else if h0 < 0. {
                -0.
            } else {
                0.
            };
        let slope = (s0 * h1 + s1 * h0) / (h0 + h1);
        let sign = |x: f64| if x < 0. { -1. } else { 1. };
        if s0.is_nan() || s1.is_nan() || slope.is_nan() {
            return 0.;
        }
        let result = (sign(s0) + sign(s1)) * s0.abs().min(s1.abs()).min(0.5 * slope.abs());
        if result.is_nan() || result == 0. {
            0.
        } else {
            result
        }
    }
    fn slope2(&self, t: f64) -> f64 {
        let [a, b] = self.previous;
        let h = b[0] - a[0];
        if h != 0. {
            (3. * (b[1] - a[1]) / h - t) / 2.
        } else {
            t
        }
    }
    fn emit(&mut self, t0: f64, t1: f64) -> ChartResult<()> {
        let [a, b] = self.previous;
        let dx = (b[0] - a[0]) / 3.;
        let c0 = self.reflect([a[0] + dx, a[1] + dx * t0]);
        let c1 = self.reflect([b[0] - dx, b[1] - dx * t1]);
        let end = self.reflect(b);
        cubic(self.path, c0, c1, end)
    }
}
impl CurveProtocol for Monotone<'_> {
    fn area_start(&mut self) -> ChartResult<()> {
        self.line = Some(false);
        Ok(())
    }
    fn area_end(&mut self) -> ChartResult<()> {
        self.line = None;
        Ok(())
    }
    fn line_start(&mut self) -> ChartResult<()> {
        self.count = 0;
        self.previous = [[f64::NAN; 2]; 2];
        self.t = f64::NAN;
        Ok(())
    }
    fn line_end(&mut self) -> ChartResult<()> {
        match self.count {
            2 => {
                let p = self.reflect(self.previous[1]);
                self.path.line_to(p[0], p[1])?;
            }
            3 => self.emit(self.t, self.slope2(self.t))?,
            _ => {}
        }
        close(self.path, &mut self.line, self.count == 1)
    }
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()> {
        let p = if self.reflect { [y, x] } else { [x, y] };
        if p == self.previous[1] {
            return Ok(());
        }
        let mut t = f64::NAN;
        match self.count {
            0 => {
                self.count = 1;
                begin(self.path, self.line, [x, y])?;
            }
            1 => self.count = 2,
            2 => {
                self.count = 3;
                t = self.slope3(p);
                self.emit(self.slope2(t), t)?;
            }
            _ => {
                t = self.slope3(p);
                self.emit(self.t, t)?;
            }
        }
        self.previous = [self.previous[1], p];
        self.t = t;
        Ok(())
    }
}
