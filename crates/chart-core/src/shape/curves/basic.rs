use super::*;
pub(super) struct Basic<'a> {
    path: &'a mut Path,
    spec: CurveSpec,
    line: Option<bool>,
    count: u8,
    previous: Point,
    t: f64,
}
impl<'a> Basic<'a> {
    pub(super) fn new(path: &'a mut Path, spec: CurveSpec) -> Self {
        Self {
            path,
            spec,
            line: None,
            count: 0,
            previous: [f64::NAN; 2],
            t: match spec {
                CurveSpec::StepBefore => 0.,
                CurveSpec::StepAfter => 1.,
                _ => 0.5,
            },
        }
    }
    fn step(&self) -> bool {
        matches!(
            self.spec,
            CurveSpec::Step | CurveSpec::StepBefore | CurveSpec::StepAfter
        )
    }
}
impl CurveProtocol for Basic<'_> {
    fn area_start(&mut self) -> ChartResult<()> {
        if self.spec != CurveSpec::LinearClosed {
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
        self.previous = [f64::NAN; 2];
        Ok(())
    }
    fn line_end(&mut self) -> ChartResult<()> {
        if self.spec == CurveSpec::LinearClosed {
            return if self.count > 0 {
                self.path.close_path()
            } else {
                Ok(())
            };
        }
        if self.step() {
            if self.t > 0. && self.t < 1. && self.count == 2 {
                self.path.line_to(self.previous[0], self.previous[1])?;
            }
            if self.line.is_some() {
                self.t = 1. - self.t;
            }
        }
        close(self.path, &mut self.line, self.count == 1)
    }
    fn point(&mut self, x: f64, y: f64) -> ChartResult<()> {
        let p = [x, y];
        if self.count == 0 {
            self.count = 1;
            begin(self.path, self.line, p)?;
        } else {
            self.count = 2;
            match self.spec {
                CurveSpec::Linear | CurveSpec::LinearClosed => self.path.line_to(x, y)?,
                CurveSpec::BumpX => {
                    let mid = (self.previous[0] + x) / 2.;
                    cubic(self.path, [mid, self.previous[1]], [mid, y], p)?;
                }
                CurveSpec::BumpY => {
                    let mid = (self.previous[1] + y) / 2.;
                    cubic(self.path, [self.previous[0], mid], [x, mid], p)?;
                }
                _ if self.t <= 0. => {
                    self.path.line_to(self.previous[0], y)?;
                    self.path.line_to(x, y)?;
                }
                _ => {
                    let mid = self.previous[0] * (1. - self.t) + x * self.t;
                    self.path.line_to(mid, self.previous[1])?;
                    self.path.line_to(mid, y)?;
                }
            }
        }
        self.previous = p;
        Ok(())
    }
}
