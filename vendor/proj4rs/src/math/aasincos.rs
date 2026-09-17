//! arc sin, cosine, tan2 and sqrt that will NOT fail
//!
#![allow(dead_code)]
use crate::errors::{Error, Result};
use crate::math::consts::{FRAC_PI_2, PI};

const ONE_TOL: f64 = 1.000_000_000_000_01;
const ATOL: f64 = 1.0e-50;

pub(crate) fn aasin(v: f64) -> Result<f64> {
    let av = v.abs();
    if av >= 1. {
        if av > ONE_TOL {
            Err(Error::ArgumentTooLarge)
        } else {
            Ok(FRAC_PI_2 * v.signum())
        }
    } else {
        Ok(v.portable_asin())
    }
}

pub(crate) fn aacos(v: f64) -> Result<f64> {
    let av = v.abs();
    if av >= 1. {
        if av > ONE_TOL {
            Err(Error::ArgumentTooLarge)
        } else if v < 0. {
            Ok(PI)
        } else {
            Ok(0.)
        }
    } else {
        Ok(v.portable_acos())
    }
}

pub(crate) fn asqrt(v: f64) -> f64 {
    if v <= 0. { 0. } else { v.portable_sqrt() }
}

pub(crate) fn aatan2(n: f64, d: f64) -> f64 {
    if n.abs() < ATOL && d.abs() < ATOL {
        0.
    } else {
        n.portable_atan2(d)
    }
}

#[allow(unused_imports)]
use crate::portable_math::PortableFloat as _;
