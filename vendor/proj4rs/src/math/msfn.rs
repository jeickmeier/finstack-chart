#[inline]
pub(crate) fn msfn(sinphi: f64, cosphi: f64, es: f64) -> f64 {
    cosphi / (1. - es * sinphi * sinphi).portable_sqrt()
}

#[allow(unused_imports)]
use crate::portable_math::PortableFloat as _;
