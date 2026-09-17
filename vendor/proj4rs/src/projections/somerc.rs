//!
//! Swiss Oblique Mercator
//!
//! ref: <https://proj.org/operations/projections/somerc.html>
//!
//! somerc: "Swiss. Obl. Mercator" "\n\tCyl, Ell\n\tFor CH1903";
//!
use crate::errors::{Error, Result};
use crate::math::{
    aasin,
    consts::{EPS_10, FRAC_PI_2, FRAC_PI_4},
};
use crate::parameters::ParamList;
use crate::proj::ProjData;

// Projection stub
super::projection! { somerc }

#[derive(Debug, Clone)]
pub(crate) struct Projection {
    e: f64,
    rone_es: f64,
    k: f64,
    c: f64,
    hlf_e: f64,
    k_r: f64,
    cosp0: f64,
    sinp0: f64,
}

#[allow(non_snake_case)]
impl Projection {
    pub fn somerc(p: &mut ProjData, _: &ParamList) -> Result<Self> {
        let el = &p.ellps;
        let hlf_e = 0.5 * el.e;

        let (sinphi, cosphi) = p.phi0.portable_sin_cos();

        let cp = cosphi * cosphi;
        let c = (1. + el.es * cp * cp * el.rone_es).portable_sqrt();
        let sinp0 = sinphi / c;
        let phip0 = aasin(sinp0)?;
        let cosp0 = phip0.portable_cos();
        let sp = sinphi * el.e;
        let k = (FRAC_PI_4 + 0.5 * phip0).portable_tan().portable_ln()
            - c * ((FRAC_PI_4 + 0.5 * p.phi0).portable_tan().portable_ln()
                - hlf_e * ((1. + sp) / (1. - sp)).portable_ln());
        let k_r = p.k0 * el.one_es.portable_sqrt() / (1. - sp * sp);
        Ok(Self {
            e: el.e,
            rone_es: el.rone_es,
            k,
            c,
            hlf_e,
            k_r,
            cosp0,
            sinp0,
        })
    }

    #[inline(always)]
    pub fn forward(&self, lam: f64, phi: f64, z: f64) -> Result<(f64, f64, f64)> {
        let sp = self.e * phi.portable_sin();
        let phip = 2.
            * ((self.c
                * ((FRAC_PI_4 + 0.5 * phi).portable_tan().portable_ln()
                    - self.hlf_e * ((1. + sp) / (1. - sp)).portable_ln())
                + self.k)
                .portable_exp())
            .portable_atan()
            - FRAC_PI_2;

        let lamp = self.c * lam;
        let cp = phip.portable_cos();
        let phipp =
            aasin(self.cosp0 * phip.portable_sin() - self.sinp0 * cp * lamp.portable_cos())?;
        let lampp = aasin(cp * lamp.portable_sin() / phipp.portable_cos())?;

        Ok((
            self.k_r * lampp,
            self.k_r * (FRAC_PI_4 + 0.5 * phipp).portable_tan().portable_ln(),
            z,
        ))
    }

    #[inline(always)]
    pub fn inverse(&self, x: f64, y: f64, z: f64) -> Result<(f64, f64, f64)> {
        const NITER: isize = 6;

        let phipp = 2. * (((y / self.k_r).portable_exp()).portable_atan() - FRAC_PI_4);
        let lampp = x / self.k_r;
        let cp = phipp.portable_cos();
        let mut phip =
            aasin(self.cosp0 * phipp.portable_sin() + self.sinp0 * cp * lampp.portable_cos())?;
        let lamp = aasin(cp * lampp.portable_sin() / phip.portable_cos())?;
        let con = (self.k - (FRAC_PI_4 + 0.5 * phip).portable_tan().portable_ln()) / self.c;

        let mut i = NITER;
        while i > 0 {
            let esp = self.e * phip.portable_sin();
            let delp = (con + (FRAC_PI_4 + 0.5 * phip).portable_tan().portable_ln()
                - self.hlf_e * ((1. + esp) / (1. - esp)).portable_ln())
                * (1. - esp * esp)
                * phip.portable_cos()
                * self.rone_es;
            phip -= delp;
            if delp.abs() < EPS_10 {
                break;
            }
            i -= 1;
        }
        if i <= 0 {
            Err(Error::ToleranceConditionError)
        } else {
            Ok((lamp / self.c, phip, z))
        }
    }

    pub const fn has_inverse() -> bool {
        true
    }

    pub const fn has_forward() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::math::consts::EPS_10;
    use crate::proj::Proj;
    use crate::tests::utils::{test_proj_forward, test_proj_inverse};

    #[test]
    fn proj_somerc_el() {
        let p = Proj::from_proj_string("+proj=somerc +ellps=GRS80").unwrap();

        println!("{:#?}", p.projection());

        let inputs = [
            ((2., 1., 0.), (222638.98158654713, 110579.96521824898, 0.)),
            ((2., -1., 0.), (222638.98158654713, -110579.96521825089, 0.)),
            ((-2., 1., 0.), (-222638.98158654713, 110579.96521824898, 0.)),
            (
                (-2., -1., 0.),
                (-222638.98158654713, -110579.96521825089, 0.),
            ),
        ];

        test_proj_forward(&p, &inputs, EPS_10);
        test_proj_inverse(&p, &inputs, EPS_10);
    }

    #[test]
    fn proj_somerc_sp() {
        let p = Proj::from_proj_string("+proj=somerc +a=6400000").unwrap();

        println!("{:#?}", p.projection());

        let inputs = [
            ((2., 1., 0.), (223402.14425527418, 111706.74357494408, 0.)),
            ((2., -1., 0.), (223402.14425527418, -111706.74357494518, 0.)),
            ((-2., 1., 0.), (-223402.14425527418, 111706.74357494408, 0.)),
            (
                (-2., -1., 0.),
                (-223402.14425527418, -111706.74357494518, 0.),
            ),
        ];

        test_proj_forward(&p, &inputs, EPS_10);
        test_proj_inverse(&p, &inputs, EPS_10);
    }
}

#[allow(unused_imports)]
use crate::portable_math::PortableFloat as _;
