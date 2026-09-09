//! Binary64 fdlibm argument reduction, adapted from the pinned libm 0.2.16 source.
/* origin: FreeBSD /usr/src/lib/msun/src/k_rem_pio2.c */
/*
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunSoft, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */
// Keep the audited fdlibm index ordering, including convolution and carry propagation.
#![allow(clippy::needless_range_loop, clippy::explicit_counter_loop)]
use libm::scalbn;
const INIT_JK: [usize; 4] = [3, 4, 4, 6];
const IPIO2: [i32; 66] = [
    0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62, 0x95993C, 0x439041, 0xFE5163,
    0xABDEBB, 0xC561B7, 0x246E3A, 0x424DD2, 0xE00649, 0x2EEA09, 0xD1921C, 0xFE1DEB, 0x1CB129,
    0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C, 0x7026B4, 0x5F7E41, 0x3991D6, 0x398353, 0x39F49C,
    0x845F8B, 0xBDF928, 0x3B1FF8, 0x97FFDE, 0x05980F, 0xEF2F11, 0x8B5A0A, 0x6D1F6D, 0x367ECF,
    0x27CB09, 0xB74F46, 0x3F669E, 0x5FEA2D, 0x7527BA, 0xC7EBE5, 0xF17B3D, 0x0739F7, 0x8A5292,
    0xEA6BFB, 0x5FB11F, 0x8D5D08, 0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20, 0x9AF436, 0x1DA9E3,
    0x91615E, 0xE61B08, 0x659985, 0x5F14A0, 0x68408D, 0xFFD880, 0x4D7327, 0x310606, 0x1556CA,
    0x73A8C9, 0x60E27B, 0xC08C6B,
];
macro_rules! i {
    ($a:expr,$i:expr) => {
        $a[$i]
    };
    ($a:expr,$i:expr,=,$v:expr) => {
        $a[$i] = $v
    };
    ($a:expr,$i:expr,+=,$v:expr) => {
        $a[$i] += $v
    };
    ($a:expr,$i:expr,-=,$v:expr) => {
        $a[$i] -= $v
    };
    ($a:expr,$i:expr,&=,$v:expr) => {
        $a[$i] &= $v
    };
    ($a:expr,$i:expr,==,$v:expr) => {
        $a[$i] == $v
    };
}
const PIO2: [f64; 8] = [
    1.57079625129699707031e+00,
    7.54978941586159635335e-08,
    5.39030252995776476554e-15,
    3.28200341580791294123e-22,
    1.27065575308067607349e-29,
    1.22933308981111328932e-36,
    2.73370053816464559624e-44,
    2.16741683877804819444e-51,
];
pub(crate) fn rem_pio2_large(x: &[f64], y: &mut [f64], e0: i32, prec: usize) -> i32 {
    extern "C" fn floor(x: f64) -> f64 {
        x.floor()
    }
    let x1p24 = f64::from_bits(0x4170000000000000);
    let x1p_24 = f64::from_bits(0x3e70000000000000);
    if cfg!(target_pointer_width = "64") {
        debug_assert!(e0 <= 16360);
    }
    let nx = x.len();
    let mut fw: f64;
    let mut n: i32;
    let mut ih: i32;
    let mut z: f64;
    let mut f: [f64; 20] = [0.; 20];
    let mut fq: [f64; 20] = [0.; 20];
    let mut q: [f64; 20] = [0.; 20];
    let mut iq: [i32; 20] = [0; 20];
    let jk = i!(INIT_JK, prec);
    let jp = jk;
    let jx = nx - 1;
    let mut jv = (e0 - 3) / 24;
    if jv < 0 {
        jv = 0;
    }
    let mut q0 = e0 - 24 * (jv + 1);
    let jv = jv as usize;
    let mut j = (jv as i32) - (jx as i32);
    let m = jx + jk;
    for i in 0..=m {
        i!(f, i, =, if j < 0 {
            0.
        } else {
            i!(IPIO2, j as usize) as f64
        });
        j += 1;
    }
    for i in 0..=jk {
        fw = 0f64;
        for j in 0..=jx {
            fw = i!(x, j).mul_add(i!(f, jx + i - j), fw);
        }
        i!(q, i, =, fw);
    }
    let mut jz = jk;
    'recompute: loop {
        let mut i = 0i32;
        z = i!(q, jz);
        for j in (1..=jz).rev() {
            fw = (x1p_24 * z) as i32 as f64;
            i!(iq, i as usize, =, (z - x1p24 * fw) as i32);
            z = i!(q, j - 1) + fw;
            i += 1;
        }
        z = scalbn(z, q0);
        z -= 8.0 * floor(z * 0.125);
        n = z as i32;
        z -= n as f64;
        ih = 0;
        if q0 > 0 {
            i = i!(iq, jz - 1) >> (24 - q0);
            n += i;
            i!(iq, jz - 1, -=, i << (24 - q0));
            ih = i!(iq, jz - 1) >> (23 - q0);
        } else if q0 == 0 {
            ih = i!(iq, jz - 1) >> 23;
        } else if z >= 0.5 {
            ih = 2;
        }
        if ih > 0 {
            n += 1;
            let mut carry = 0i32;
            for i in 0..jz {
                let j = i!(iq, i);
                if carry == 0 {
                    if j != 0 {
                        carry = 1;
                        i!(iq, i, =, 0x1000000 - j);
                    }
                } else {
                    i!(iq, i, =, 0xffffff - j);
                }
            }
            if q0 > 0 {
                match q0 {
                    1 => {
                        i!(iq, jz - 1, &=, 0x7fffff);
                    }
                    2 => {
                        i!(iq, jz - 1, &=, 0x3fffff);
                    }
                    _ => {}
                }
            }
            if ih == 2 {
                z = 1. - z;
                if carry != 0 {
                    z -= scalbn(1., q0);
                }
            }
        }
        if z == 0. {
            let mut j = 0;
            for i in (jk..=jz - 1).rev() {
                j |= i!(iq, i);
            }
            if j == 0 {
                let mut k = 1;
                while i!(iq, jk - k, ==, 0) {
                    k += 1;
                }
                for i in (jz + 1)..=(jz + k) {
                    i!(f, jx + i, =, i!(IPIO2, jv + i) as f64);
                    fw = 0f64;
                    for j in 0..=jx {
                        fw = i!(x, j).mul_add(i!(f, jx + i - j), fw);
                    }
                    i!(q, i, =, fw);
                }
                jz += k;
                continue 'recompute;
            }
        }
        break;
    }
    if z == 0. {
        jz -= 1;
        q0 -= 24;
        while i!(iq, jz) == 0 {
            jz -= 1;
            q0 -= 24;
        }
    } else {
        z = scalbn(z, -q0);
        if z >= x1p24 {
            fw = (x1p_24 * z) as i32 as f64;
            i!(iq, jz, =, (z - x1p24 * fw) as i32);
            jz += 1;
            q0 += 24;
            i!(iq, jz, =, fw as i32);
        } else {
            i!(iq, jz, =, z as i32);
        }
    }
    fw = scalbn(1., q0);
    for i in (0..=jz).rev() {
        i!(q, i, =, fw * (i!(iq, i) as f64));
        fw *= x1p_24;
    }
    for i in (0..=jz).rev() {
        fw = 0f64;
        let mut k = 0;
        while (k <= jp) && (k <= jz - i) {
            fw = i!(PIO2, k).mul_add(i!(q, i + k), fw);
            k += 1;
        }
        i!(fq, jz - i, =, fw);
    }
    match prec {
        0 => {
            fw = 0f64;
            for i in (0..=jz).rev() {
                fw += i!(fq, i);
            }
            i!(y, 0, =, if ih == 0 { fw } else { -fw });
        }
        1 | 2 => {
            fw = 0f64;
            for i in (0..=jz).rev() {
                fw += i!(fq, i);
            }
            i!(y, 0, =, if ih == 0 { fw } else { -fw });
            fw = i!(fq, 0) - fw;
            for i in 1..=jz {
                fw += i!(fq, i);
            }
            i!(y, 1, =, if ih == 0 { fw } else { -fw });
        }
        3 => {
            for i in (1..=jz).rev() {
                fw = i!(fq, i - 1) + i!(fq, i);
                i!(fq, i, +=, i!(fq, i - 1) - fw);
                i!(fq, i - 1, =, fw);
            }
            for i in (2..=jz).rev() {
                fw = i!(fq, i - 1) + i!(fq, i);
                i!(fq, i, +=, i!(fq, i - 1) - fw);
                i!(fq, i - 1, =, fw);
            }
            fw = 0f64;
            for i in (2..=jz).rev() {
                fw += i!(fq, i);
            }
            if ih == 0 {
                i!(y, 0, =, i!(fq, 0));
                i!(y, 1, =, i!(fq, 1));
                i!(y, 2, =, fw);
            } else {
                i!(y, 0, =, -i!(fq, 0));
                i!(y, 1, =, -i!(fq, 1));
                i!(y, 2, =, -fw);
            }
        }
        _ => unreachable!(),
    }
    n & 7
}
