//! Fixed fdlibm sine/cosine evaluation and fused operations matching the pinned Node arm64 oracle.
//! Adapted from V8 13.6.233 src/base/ieee754.cc and Sun fdlibm.
//! Copyright (C) 1993 Sun Microsystems, Inc. All rights reserved.
//! Developed at SunSoft, a Sun Microsystems, Inc. business.
//! Permission to use, copy, modify, and distribute this software is freely granted,
//! provided that this notice is preserved. V8 modifications copyright 2016 the V8
//! project authors. See LICENSE for the additional V8 redistribution terms.
#![allow(clippy::excessive_precision)] // Reference decimal literals preserve binary64 coefficients.
mod reduce_large;
fn high(x: f64) -> u32 {
    ((x.to_bits() >> 32) as u32) & 0x7fff_ffff
}
fn kernel_sin(x: f64, y: f64, tail: bool) -> f64 {
    const S1: f64 = -1.66666666666666324348e-01;
    const S2: f64 = 8.33333333332248946124e-03;
    const S3: f64 = -1.98412698298579493134e-04;
    const S4: f64 = 2.75573137070700676789e-06;
    const S5: f64 = -2.50507602534068634195e-08;
    const S6: f64 = 1.58969099521155010221e-10;
    if high(x) < 0x3e40_0000 {
        return x;
    }
    let z = x * x;
    let v = z * x;
    let r = z.mul_add(z.mul_add(z.mul_add(z.mul_add(S6, S5), S4), S3), S2);
    if tail {
        x - v.mul_add(-S1, z.mul_add(y.mul_add(0.5, -(v * r)), -y))
    } else {
        v.mul_add(z.mul_add(r, S1), x)
    }
}
fn kernel_cos(x: f64, y: f64) -> f64 {
    const C1: f64 = 4.16666666666666019037e-02;
    const C2: f64 = -1.38888888888741095749e-03;
    const C3: f64 = 2.48015872894767294178e-05;
    const C4: f64 = -2.75573143513906633035e-07;
    const C5: f64 = 2.08757232129817482790e-09;
    const C6: f64 = -1.13596475577881948265e-11;
    let ix = high(x);
    if ix < 0x3e40_0000 {
        return 1.;
    }
    let z = x * x;
    let r = z * z.mul_add(
        z.mul_add(z.mul_add(z.mul_add(z.mul_add(C6, C5), C4), C3), C2),
        C1,
    );
    if ix < 0x3fd3_3333 {
        1. + z.mul_add(-0.5, z.mul_add(r, -(x * y)))
    } else {
        let qx = if ix > 0x3fe9_0000 {
            0.28125
        } else {
            f64::from_bits(u64::from(ix - 0x0020_0000) << 32)
        };
        let iz = 0.5 * z - qx;
        let a = 1. - qx;
        a + (z.mul_add(r, -(x * y)) - iz)
    }
}
fn reduce(x: f64) -> (i32, f64, f64) {
    const INV: f64 = f64::from_bits(0x3fe4_5f30_6dc9_c883);
    const P1: f64 = 1.57079632673412561417e+00;
    const P1T: f64 = 6.07710050650619224932e-11;
    const P2: f64 = 6.07710050630396597660e-11;
    const P2T: f64 = 2.02226624879595063154e-21;
    const P3: f64 = 2.02226624871116645580e-21;
    const P3T: f64 = 8.47842766036889956997e-32;
    const NPIO2: [u32; 32] = [
        0x3FF921FB, 0x400921FB, 0x4012D97C, 0x401921FB, 0x401F6A7A, 0x4022D97C, 0x4025FDBB,
        0x402921FB, 0x402C463A, 0x402F6A7A, 0x4031475C, 0x4032D97C, 0x40346B9C, 0x4035FDBB,
        0x40378FDB, 0x403921FB, 0x403AB41B, 0x403C463A, 0x403DD85A, 0x403F6A7A, 0x40407E4C,
        0x4041475C, 0x4042106C, 0x4042D97C, 0x4043A28C, 0x40446B9C, 0x404534AC, 0x4045FDBB,
        0x4046C6CB, 0x40478FDB, 0x404858EB, 0x404921FB,
    ];
    let ix = high(x);
    if ix <= 0x3fe9_21fb {
        return (0, x, 0.);
    }
    if ix < 0x4002_d97c {
        if x > 0. {
            let mut z = x - P1;
            let (y0, y1) = if ix != 0x3ff9_21fb {
                let y = z - P1T;
                (y, (z - y) - P1T)
            } else {
                z -= P2;
                let y = z - P2T;
                (y, (z - y) - P2T)
            };
            return (1, y0, y1);
        } else {
            let mut z = x + P1;
            let (y0, y1) = if ix != 0x3ff9_21fb {
                let y = z + P1T;
                (y, (z - y) + P1T)
            } else {
                z += P2;
                let y = z + P2T;
                (y, (z - y) + P2T)
            };
            return (-1, y0, y1);
        }
    }
    if ix <= 0x4139_21fb {
        let t = x.abs();
        let n = t.mul_add(INV, 0.5) as i32;
        let f = f64::from(n);
        let mut r = (-f).mul_add(P1, t);
        let mut w = f * P1T;
        let mut y0 = r - w;
        if !(n < 32 && ix != NPIO2[n as usize - 1]) {
            let j = (ix >> 20) as i32;
            if j - ((high(y0) >> 20) & 0x7ff) as i32 > 16 {
                let t = r;
                r = (-f).mul_add(P2, t);
                w = f.mul_add(P2T, f.mul_add(P2, -(t - r)));
                y0 = r - w;
                if j - ((high(y0) >> 20) & 0x7ff) as i32 > 49 {
                    let t = r;
                    r = (-f).mul_add(P3, t);
                    w = f.mul_add(P3T, f.mul_add(P3, -(t - r)));
                    y0 = r - w;
                }
            }
        }
        let y1 = (r - y0) - w;
        return if x < 0. { (-n, -y0, -y1) } else { (n, y0, y1) };
    }
    let e0 = (ix >> 20) as i32 - 1046;
    let mut z = f64::from_bits(
        (u64::from((ix as i32).wrapping_sub(e0.wrapping_shl(20)) as u32) << 32)
            | (x.to_bits() & 0xffff_ffff),
    );
    let mut tx = [0.; 3];
    for v in &mut tx[..2] {
        *v = z.trunc();
        z = (z - *v) * 16_777_216.;
    }
    tx[2] = z;
    let mut nx = 3;
    while tx[nx - 1] == 0. {
        nx -= 1;
    }
    let mut y = [0.; 2];
    let n = reduce_large::rem_pio2_large(&tx[..nx], &mut y, e0, 2);
    if x < 0. {
        (-n, -y[0], -y[1])
    } else {
        (n, y[0], y[1])
    }
}
pub(crate) fn sin(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if high(x) <= 0x3fe9_21fb {
        return kernel_sin(x, 0., false);
    }
    let (n, a, b) = reduce(x);
    match n & 3 {
        0 => kernel_sin(a, b, true),
        1 => kernel_cos(a, b),
        2 => -kernel_sin(a, b, true),
        _ => -kernel_cos(a, b),
    }
}
pub(crate) fn cos(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    if high(x) <= 0x3fe9_21fb {
        return kernel_cos(x, 0.);
    }
    let (n, a, b) = reduce(x);
    match n & 3 {
        0 => kernel_cos(a, b),
        1 => -kernel_sin(a, b, true),
        2 => -kernel_cos(a, b),
        _ => kernel_sin(a, b, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_trigonometry_matches_pinned_v8_anchors() {
        let corpus: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/parity/d3-scale-chromatic/trig.json"
        )))
        .unwrap();
        let mut failures = vec![];
        for row in corpus["cases"].as_array().unwrap() {
            let values: Vec<crate::interpolate::Number> =
                serde_json::from_value(row.clone()).unwrap();
            for (name, a, b) in [
                ("sin", sin(values[0].0), values[1].0),
                ("cos", cos(values[0].0), values[2].0),
            ] {
                if !(a.is_nan() && b.is_nan()) && a.to_bits() != b.to_bits() && failures.len() < 20
                {
                    failures.push(format!(
                        "{name}({:.17e}): {a:.17e} != {b:.17e}",
                        values[0].0
                    ));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
}
