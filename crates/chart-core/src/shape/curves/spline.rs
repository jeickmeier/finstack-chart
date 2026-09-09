use super::*;
#[derive(Clone, Copy)]
enum Kernel {
    Cardinal(f64),
    CatmullRom(f64),
}
pub(super) struct Spline<'a> {
    path: &'a mut Path,
    mode: Mode,
    kernel: Kernel,
    line: Option<bool>,
    count: u8,
    previous: [Point; 3],
    seed: [Point; 3],
    lengths: [[f64; 2]; 3],
}
impl<'a> Spline<'a> {
    pub(super) fn cardinal(path: &'a mut Path, mode: Mode, tension: f64) -> Self {
        Self::new(path, mode, Kernel::Cardinal((1. - tension) / 6.))
    }
    pub(super) fn catmull_rom(path: &'a mut Path, mode: Mode, alpha: f64) -> Self {
        if alpha == 0. {
            Self::cardinal(path, mode, 0.)
        } else {
            Self::new(path, mode, Kernel::CatmullRom(alpha))
        }
    }
    fn new(path: &'a mut Path, mode: Mode, kernel: Kernel) -> Self {
        Self {
            path,
            mode,
            kernel,
            line: None,
            count: 0,
            previous: [[f64::NAN; 2]; 3],
            seed: [[f64::NAN; 2]; 3],
            lengths: [[0.; 2]; 3],
        }
    }
    fn emit(&mut self, p: Point) -> ChartResult<()> {
        let [p0, p1, p2] = self.previous;
        let (a, b) = match self.kernel {
            Kernel::Cardinal(k) => (
                [p1[0] + k * (p2[0] - p0[0]), p1[1] + k * (p2[1] - p0[1])],
                [p2[0] + k * (p1[0] - p[0]), p2[1] + k * (p1[1] - p[1])],
            ),
            Kernel::CatmullRom(_) => {
                let [[l01, l01_2], [l12, l12_2], [l23, l23_2]] = self.lengths;
                let mut a = p1;
                let mut b = p2;
                if l01 > 1e-12 {
                    let numerator = 2. * l01_2 + 3. * l01 * l12 + l12_2;
                    let denominator = 3. * l01 * (l01 + l12);
                    for j in 0..2 {
                        a[j] = (p1[j] * numerator - p0[j] * l12_2 + p2[j] * l01_2) / denominator;
                    }
                }
                if l23 > 1e-12 {
                    let numerator = 2. * l23_2 + 3. * l23 * l12 + l12_2;
                    let denominator = 3. * l23 * (l23 + l12);
                    for j in 0..2 {
                        b[j] = (p2[j] * numerator + p1[j] * l23_2 - p[j] * l12_2) / denominator;
                    }
                }
                (a, b)
            }
        };
        cubic(self.path, a, b, p2)
    }
}
impl CurveProtocol for Spline<'_> {
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
        self.previous = [[f64::NAN; 2]; 3];
        self.seed = [[f64::NAN; 2]; 3];
        self.lengths = [[0.; 2]; 3];
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
                    self.path.line_to(self.seed[0][0], self.seed[0][1])?;
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
            match self.count {
                2 => self
                    .path
                    .line_to(self.previous[2][0], self.previous[2][1])?,
                3 => match self.kernel {
                    Kernel::Cardinal(_) => self.emit(self.previous[1])?,
                    Kernel::CatmullRom(_) => {
                        let p = self.previous[2];
                        self.point(p[0], p[1])?;
                    }
                },
                _ => {}
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
        if let Kernel::CatmullRom(alpha) = self.kernel
            && self.count > 0
        {
            let dx = self.previous[2][0] - x;
            let dy = self.previous[2][1] - y;
            let square = (dx * dx + dy * dy).powf(alpha);
            self.lengths[2] = [square.sqrt(), square];
        }
        match self.mode {
            Mode::Standard => match self.count {
                0 => {
                    self.count = 1;
                    begin(self.path, self.line, p)?;
                }
                1 => {
                    self.count = 2;
                    if matches!(self.kernel, Kernel::Cardinal(_)) {
                        self.previous[1] = p;
                    }
                }
                _ => {
                    self.count = 3;
                    self.emit(p)?;
                }
            },
            Mode::Open => match self.count {
                0 | 1 => self.count += 1,
                2 => {
                    self.count = 3;
                    begin(self.path, self.line, self.previous[2])?;
                }
                _ => {
                    self.count = 4;
                    self.emit(p)?;
                }
            },
            Mode::Closed => match self.count {
                0 => {
                    self.count = 1;
                    self.seed[0] = p;
                }
                1 => {
                    self.count = 2;
                    self.seed[1] = p;
                    self.path.move_to(x, y)?;
                }
                2 => {
                    self.count = 3;
                    self.seed[2] = p;
                }
                _ => self.emit(p)?,
            },
        }
        self.lengths = [self.lengths[1], self.lengths[2], self.lengths[2]];
        self.previous = [self.previous[1], self.previous[2], p];
        Ok(())
    }
}
