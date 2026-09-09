use super::*;
pub(super) struct Basis<'a> {
    path: &'a mut Path,
    mode: Mode,
    line: Option<bool>,
    count: u8,
    previous: [Point; 2],
    seed: [Point; 3],
}
impl<'a> Basis<'a> {
    pub(super) fn new(path: &'a mut Path, mode: Mode) -> Self {
        Self {
            path,
            mode,
            line: None,
            count: 0,
            previous: [[f64::NAN; 2]; 2],
            seed: [[f64::NAN; 2]; 3],
        }
    }
    fn emit(&mut self, p: Point) -> ChartResult<()> {
        let [a, b] = self.previous;
        cubic(
            self.path,
            [(2. * a[0] + b[0]) / 3., (2. * a[1] + b[1]) / 3.],
            [(a[0] + 2. * b[0]) / 3., (a[1] + 2. * b[1]) / 3.],
            [
                (a[0] + 4. * b[0] + p[0]) / 6.,
                (a[1] + 4. * b[1] + p[1]) / 6.,
            ],
        )
    }
}
impl CurveProtocol for Basis<'_> {
    fn area_start(&mut self) -> ChartResult<()> {
        if self.mode != Mode::Closed {
            self.line = Some(false);
        }
        Ok(())
    }
    fn area_end(&mut self) -> ChartResult<()> {
        self.line = None;
        Ok(())
    }
    fn line_start(&mut self) -> ChartResult<()> {
        self.count = 0;
        self.previous = [[f64::NAN; 2]; 2];
        self.seed = [[f64::NAN; 2]; 3];
        Ok(())
    }
    fn line_end(&mut self) -> ChartResult<()> {
        if self.mode == Mode::Closed {
            match self.count {
                1 => {
                    self.path.move_to(self.seed[0][0], self.seed[0][1])?;
                    self.path.close_path()?;
                }
                2 => {
                    let [a, b, _] = self.seed;
                    self.path
                        .move_to((a[0] + 2. * b[0]) / 3., (a[1] + 2. * b[1]) / 3.)?;
                    self.path
                        .line_to((b[0] + 2. * a[0]) / 3., (b[1] + 2. * a[1]) / 3.)?;
                    self.path.close_path()?;
                }
                3 => {
                    for p in self.seed {
                        self.point(p[0], p[1])?;
                    }
                }
                _ => {}
            }
            return Ok(());
        }
        if self.mode == Mode::Standard {
            if self.count == 3 {
                self.emit(self.previous[1])?;
            }
            if self.count >= 2 {
                self.path
                    .line_to(self.previous[1][0], self.previous[1][1])?;
            }
        }
        close(
            self.path,
            &mut self.line,
            self.count == if self.mode == Mode::Open { 3 } else { 1 },
        )
    }
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()> {
        let p = [x, y];
        let [a, b] = self.previous;
        match self.mode {
            Mode::Standard => match self.count {
                0 => {
                    self.count = 1;
                    begin(self.path, self.line, p)?;
                }
                1 => self.count = 2,
                _ => {
                    if self.count == 2 {
                        self.count = 3;
                        self.path
                            .line_to((5. * a[0] + b[0]) / 6., (5. * a[1] + b[1]) / 6.)?;
                    }
                    self.emit(p)?;
                }
            },
            Mode::Open => match self.count {
                0 | 1 => self.count += 1,
                2 => {
                    self.count = 3;
                    begin(
                        self.path,
                        self.line,
                        [(a[0] + 4. * b[0] + x) / 6., (a[1] + 4. * b[1] + y) / 6.],
                    )?;
                }
                _ => {
                    self.count = 4;
                    self.emit(p)?;
                }
            },
            Mode::Closed => match self.count {
                0 | 1 => {
                    self.seed[self.count as usize] = p;
                    self.count += 1;
                }
                2 => {
                    self.seed[2] = p;
                    self.count = 3;
                    self.path
                        .move_to((a[0] + 4. * b[0] + x) / 6., (a[1] + 4. * b[1] + y) / 6.)?;
                }
                _ => self.emit(p)?,
            },
        }
        self.previous = [b, p];
        Ok(())
    }
}

pub(super) struct Bundle<'a> {
    basis: Basis<'a>,
    beta: f64,
    points: Vec<Point>,
}
impl<'a> Bundle<'a> {
    pub(super) fn new(path: &'a mut Path, beta: f64) -> Self {
        Self {
            basis: Basis::new(path, Mode::Standard),
            beta,
            points: Vec::new(),
        }
    }
}
impl CurveProtocol for Bundle<'_> {
    fn line_start(&mut self) -> ChartResult<()> {
        self.points.clear();
        self.basis.line_start()
    }
    fn line_end(&mut self) -> ChartResult<()> {
        if self.points.len() > 1 {
            let first = self.points[0];
            let last = self.points[self.points.len() - 1];
            let delta = [last[0] - first[0], last[1] - first[1]];
            for (i, p) in self.points.iter().enumerate() {
                let t = i as f64 / (self.points.len() - 1) as f64;
                self.basis.point(
                    self.beta * p[0] + (1. - self.beta) * (first[0] + t * delta[0]),
                    self.beta * p[1] + (1. - self.beta) * (first[1] + t * delta[1]),
                )?;
            }
        }
        self.points.clear();
        self.basis.line_end()
    }
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()> {
        self.points.push([x, y]);
        Ok(())
    }
}
