//! Local portability patch: same formula evaluation with pinned libm transcendentals.
//! This removes native-libc versus WebAssembly math differences, without rounding outputs.
#[allow(dead_code)]
pub(crate) trait PortableFloat {
    fn portable_sin(self) -> f64;
    fn portable_cos(self) -> f64;
    fn portable_tan(self) -> f64;
    fn portable_asin(self) -> f64;
    fn portable_acos(self) -> f64;
    fn portable_atan(self) -> f64;
    fn portable_sinh(self) -> f64;
    fn portable_cosh(self) -> f64;
    fn portable_atanh(self) -> f64;
    fn portable_sqrt(self) -> f64;
    fn portable_ln(self) -> f64;
    fn portable_ln_1p(self) -> f64;
    fn portable_exp(self) -> f64;
    fn portable_atan2(self, other: f64) -> f64;
    fn portable_hypot(self, other: f64) -> f64;
    fn portable_powf(self, other: f64) -> f64;
    fn portable_sin_cos(self) -> (f64, f64);
}
impl PortableFloat for f64 {
    fn portable_sin(self) -> f64 {
        libm::sin(self)
    }
    fn portable_cos(self) -> f64 {
        libm::cos(self)
    }
    fn portable_tan(self) -> f64 {
        libm::tan(self)
    }
    fn portable_asin(self) -> f64 {
        libm::asin(self)
    }
    fn portable_acos(self) -> f64 {
        libm::acos(self)
    }
    fn portable_atan(self) -> f64 {
        libm::atan(self)
    }
    fn portable_sinh(self) -> f64 {
        libm::sinh(self)
    }
    fn portable_cosh(self) -> f64 {
        libm::cosh(self)
    }
    fn portable_atanh(self) -> f64 {
        libm::atanh(self)
    }
    fn portable_sqrt(self) -> f64 {
        libm::sqrt(self)
    }
    fn portable_ln(self) -> f64 {
        libm::log(self)
    }
    fn portable_ln_1p(self) -> f64 {
        libm::log1p(self)
    }
    fn portable_exp(self) -> f64 {
        libm::exp(self)
    }
    fn portable_atan2(self, other: f64) -> f64 {
        libm::atan2(self, other)
    }
    fn portable_hypot(self, other: f64) -> f64 {
        libm::hypot(self, other)
    }
    fn portable_powf(self, other: f64) -> f64 {
        libm::pow(self, other)
    }
    fn portable_sin_cos(self) -> (f64, f64) {
        (libm::sin(self), libm::cos(self))
    }
}
