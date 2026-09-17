//! Immutable conformal libmap kernels, Copyright2021 Plan9 Foundation (MIT).
//! See ADR031 and docs/licenses/plan9-mit.txt. No global initializer state is retained.
use super::MapprojMethod;
use super::geography_mapproj::{Coord, Place, complex_div, complex_sqrt, normalize};
use crate::{ChartResult, DiagnosticCode};
use libm::{atan2, cos, hypot, log, sin, sqrt};
use std::f64::consts::PI;
fn mul(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] * b[0] - a[1] * b[1], a[0] * b[1] + a[1] * b[0]]
}
fn square(a: [f64; 2]) -> [f64; 2] {
    [a[0] * a[0] - a[1] * a[1], a[0] * a[1] * 2.]
}
fn div2(mut c: [f64; 2], mut d: [f64; 2]) -> [f64; 2] {
    if d[1].abs() > d[0].abs() {
        d.swap(0, 1);
        c.swap(0, 1);
    }
    let den = if d[0].abs() > 1e19 {
        1e38
    } else {
        d[0] * d[0] + d[1] * d[1]
    };
    let t = d[1] / d[0];
    [(c[0] + t * c[1]) / (d[0] + t * d[1]), den]
}
fn sqrt_abs(z: [f64; 2]) -> [f64; 2] {
    let r = z[0] * z[0] + z[1] * z[1];
    if r <= 0. {
        return [0., 0.];
    }
    let x = sqrt((sqrt(r) + z[0].abs()) / 2.);
    [x, z[1] / (x * 2.)]
}
// Bulirsch's complex first-kind elliptic integral (the source a=b=1 case).
// The source's correction-array coefficient a-b is identically zero, so no
// correction arrays need allocation. All iterative state remains local.
fn elliptic(mut x: f64, mut y: f64, mut kc: f64) -> Option<[f64; 2]> {
    if kc == 0. || x < 0. {
        return None;
    }
    let sy = if y > 0. {
        1.
    } else if y == 0. {
        0.
    } else {
        -1.
    };
    y = y.abs();
    let z = square([x, y]);
    let d = kc * kc;
    let mut k = 1. - d;
    let e1 = 1. + z[0];
    let f = div2([1. + d * z[0], d * z[1]], [e1, z[1]]);
    let mut dn = sqrt_abs([f[0], -k * x * y * 2. / f[1]]);
    if f[0] < 0. {
        dn = [-dn[1], -dn[0]];
    }
    if k < 0. {
        dn = [dn[0].abs(), dn[1].abs()];
    }
    let mut c = 1. + dn[0];
    let (mut f, mut m, mut a, mut b, mut e, mut l) = (1., 1., 2., 1., 1., 4.);
    kc = kc.abs();
    let mut m1 = 1.;
    let mut converged = false;
    for _ in 1..32 {
        m1 = (kc + m) / 2.;
        let m2 = m1 * m1;
        k *= f / (m2 * 4.);
        b += e * kc;
        e = a;
        let v = div2([kc + m * dn[0], m * dn[1]], [c, dn[1]]);
        dn = sqrt_abs([v[0] / m1, k * dn[1] * 2. / v[1]]);
        let xy = mul(dn, [x, y]);
        x = xy[0].abs();
        y = xy[1].abs();
        a += b / m1;
        l *= 2.;
        c = 1. + dn[0];
        k *= k;
        if k <= 1e-6 {
            converged = true;
            break;
        }
        kc = sqrt(m * kc);
        f = m2;
        m = m1;
    }
    if !converged {
        return None;
    }
    x *= m1;
    y *= m1;
    let v = div2([1. - y, x], [1. + y, -x]);
    let e2 = x * 2. / v[1];
    let d = a / (m1 * l);
    let mut u = atan2(e2, v[0]);
    if u < 0. {
        u += PI;
    }
    let a = d * sy / 2.;
    Some([d * u, (-1. - log(v[0] * v[0] + e2 * e2)) * a + a])
}
fn place(lat: f64, lon: f64) -> Place {
    Place {
        lat: Coord::degrees(lat),
        lon: Coord::degrees(lon),
    }
}
fn stereo(p: Place) -> [f64; 2] {
    super::geography_mapproj::stereographic_raw(p)
}
struct Square {
    kc: f64,
    constant: f64,
    side: f64,
}
impl Square {
    fn new() -> Self {
        let kc = 1. / (3. + 2. * sqrt(2.));
        let constant = -(1. + sqrt(2.));
        let side = 2. * elliptic(-constant, 0., kc).unwrap()[0];
        Self { kc, constant, side }
    }
    fn apply(&self, z: [f64; 2]) -> [f64; 2] {
        let w = z[0] - 1.;
        if (w * w + z[1] * z[1]).abs() <= 1e-6 {
            return [self.side, 0.];
        }
        let mut v = complex_div([z[0] + 1., z[1]], [w, z[1]]);
        v[0] = (v[0] * self.constant).max(0.);
        v[1] *= self.constant;
        elliptic(v[0], v[1], self.kc).unwrap_or([f64::NAN; 2])
    }
    fn guyou(&self, p: Place) -> [f64; 2] {
        let east = p.lon.l < 0.;
        let q = normalize(
            p,
            place(0., if east { -90. } else { 90. }),
            Coord::degrees(0.),
        );
        let z = stereo(q);
        let mut v = self.apply([z[0] / 2., z[1] / 2.]);
        if !east {
            v[0] -= self.side;
        }
        v
    }
    fn square(&self, p: Place) -> [f64; 2] {
        let q = Place {
            lat: Coord {
                l: p.lat.l.abs(),
                s: p.lat.s.abs(),
                c: p.lat.c,
            },
            ..p
        };
        if q.lat.l < 0.0001 && q.lon.l.abs() > PI - 0.0001 {
            return [if q.lon.l > 0. { 0. } else { self.side }, -self.side / 2.];
        }
        let z = stereo(q);
        let r = sqrt(sqrt(hypot(z[0], z[1]) / 2.));
        let theta = atan2(z[0], -z[1]) / 4.;
        let mut v = self.apply([r * sin(theta), -r * cos(theta)]);
        if p.lat.l < 0. {
            v[1] = -self.side - v[1];
        }
        v
    }
}
pub(super) fn project(method: MapprojMethod, p: Place) -> ChartResult<Option<[f64; 2]>> {
    let v = match method {
        MapprojMethod::Guyou => Square::new().guyou(p),
        MapprojMethod::Square => Square::new().square(p),
        MapprojMethod::Tetra => return Ok(tetra(p)),
        MapprojMethod::Hex => Hex::new().apply(p),
        MapprojMethod::Eisenlohr => eisenlohr(p),
        _ => {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "Conformal projection kernel qualification is incomplete.",
            ));
        }
    };
    Ok(Some(v))
}
const TETRA: [[[f64; 4]; 4]; 4] = [
    [
        [0.; 4],
        [90., 0., 90., -90.],
        [0., 45., -45., 150.],
        [0., -45., -135., 30.],
    ],
    [
        [90., 0., -90., 90.],
        [0.; 4],
        [0., 135., -135., -150.],
        [0., -135., -45., -30.],
    ],
    [
        [0., 45., 135., -30.],
        [0., 135., 45., -150.],
        [0.; 4],
        [-90., 0., 180., 90.],
    ],
    [
        [0., -45., 45., -150.],
        [0., -135., 135., -30.],
        [-90., 0., 0., 90.],
        [0.; 4],
    ],
];
fn tetra(p: Place) -> Option<[f64; 2]> {
    let root3 = sqrt(3.);
    let tkc = sqrt(0.5 - 0.25 * root3);
    let tk = sqrt(0.5 + 0.25 * root3);
    let tcon = 2. * sqrt(root3);
    let f0 = elliptic(tcon / (root3 - 1.), 0., tkc)?;
    let fpi = elliptic(1e15, 0., tk)?;
    let mut distances = [0.; 4];
    for (i, (sign, lon)) in [(1., 0.), (1., 180.), (-1., 90.), (-1., -90.)]
        .into_iter()
        .enumerate()
    {
        let s = sign / root3;
        let c = sqrt(1. - s * s);
        let lon = Coord::degrees(lon);
        distances[i] = p.lat.s * s + p.lat.c * c * (p.lon.s * lon.s + p.lon.c * lon.c);
    }
    let mut i = 0;
    for k in 1..4 {
        if distances[k] > distances[i] {
            i = k;
        }
    }
    let mut j = if i == 0 { 1 } else { 0 };
    for k in 0..4 {
        if k != i && distances[k] > distances[j] {
            j = k;
        }
    }
    let a = TETRA[i][j];
    let q = normalize(p, place(a[0], a[1]), Coord::degrees(a[2]));
    let z = stereo(q);
    let z = [(z[0] / 2.).max(0.00001), z[1] / 2.];
    let mut z2 = square(z);
    let z4 = square(z2);
    z2[0] *= 2. * root3;
    z2[1] *= 2. * root3;
    let s = complex_div(
        [z4[0] + z2[0] - 1., z4[1] + z2[1]],
        [z4[0] - z2[0] - 1., z4[1] - z2[1]],
    );
    let t = complex_sqrt([s[0] - 1., s[1]]);
    let b = complex_div([tcon * t[0], tcon * t[1]], [root3 + 1. - s[0], -s[1]]);
    let v = if b[0] < 0. {
        let v = elliptic(-b[0], -b[1], tk)?;
        [2. * fpi[0] - v[0], 2. * fpi[1] - v[1]]
    } else {
        elliptic(b[0], b[1], tk)?
    };
    let t = if s[1] >= 0. {
        [f0[0] - v[1], f0[1] + v[0]]
    } else {
        [f0[0] + v[1], f0[1] - v[0]]
    };
    let rot = Coord::degrees(a[3]);
    Some([
        t[0] * rot.c + t[1] * rot.s + [0., 0., -1., 1.][i] * f0[0] * root3,
        t[1] * rot.c - t[0] * rot.s + [0., 2., -1., -1.][i] * f0[0],
    ])
}
fn cubrt(mut a: f64) -> f64 {
    if a == 0. || !a.is_finite() {
        return a;
    }
    let mut y = if a < 0. {
        a = -a;
        -1.
    } else {
        1.
    };
    for _ in 0..360 {
        if a >= 1. {
            break;
        }
        a *= 8.;
        y /= 2.;
    }
    for _ in 0..360 {
        if a <= 1. {
            break;
        }
        a /= 8.;
        y *= 2.;
    }
    let mut x = 1.;
    for _ in 0..32 {
        let old = x;
        x = (2. * old + a / (old * old)) / 3.;
        if (x - old).abs() <= 1e-14 {
            break;
        }
    }
    x * y
}
struct Hex {
    kc: f64,
    w2: f64,
    rootroot3: f64,
    rootk: f64,
    cr: [f64; 3],
    ci: [f64; 3],
}
impl Hex {
    fn new() -> Self {
        let r = sqrt(3.);
        let t = 15. - 8. * r;
        let kc = t * (1. - sqrt(1. - 1. / (t * t)));
        let mut h = Self {
            kc,
            w2: 2. * elliptic(1e15, 0., kc).unwrap()[0],
            rootroot3: sqrt(r),
            rootk: sqrt(kc),
            cr: [0.; 3],
            ci: [0.; 3],
        };
        let c = h.apply(place(90., 0.))[0];
        let d = h.apply(place(0., 0.))[0];
        for i in 0..3 {
            h.cr[i] = c + (c - d) * [0.5, -1., 0.5][i];
            h.ci[i] = (c - d) * [-1., 0., 1.][i] * r / 2.;
        }
        h
    }
    fn apply(&self, p: Place) -> [f64; 2] {
        let north = p.lat.l >= 0.;
        let mut q = Place {
            lat: Coord {
                l: p.lat.l.abs(),
                s: p.lat.s.abs(),
                c: p.lat.c,
            },
            ..p
        };
        if q.lat.l < 0.0001 {
            for (i, cut) in [-PI / 3., PI / 3., PI].into_iter().enumerate() {
                let delta =
                    super::geography_mapproj::cirmod((q.lon.l - cut) * 180. / PI) * PI / 180.;
                if delta.abs() < 0.0001 {
                    return if i == 2 {
                        [2. * self.cr[0] - self.cr[1], 0.]
                    } else {
                        [self.cr[1], 2. * self.ci[2 * i]]
                    };
                }
            }
            q.lat = Coord::radians(0.0001);
        }
        q = normalize(q, place(90., 90.), Coord::degrees(0.));
        let z = stereo(q);
        let z = [z[0] / 2., z[1] / 2.];
        let s = complex_div([1. - z[0], -z[1]], [1. + z[0], z[1]]);
        let t = square(s);
        let arg = [1. + 3. * t[0], 3. * t[1]];
        let theta = atan2(arg[1], arg[0]) / 3.;
        let r = cubrt(hypot(arg[0], arg[1]));
        let u = [r * cos(theta), r * sin(theta)];
        let v = complex_sqrt([u[0] - 1., u[1]]);
        let y = complex_div(
            [self.rootroot3 + v[0], v[1]],
            [self.rootroot3 - v[0], -v[1]],
        );
        let y = [y[0] / self.rootk, y[1] / self.rootk];
        let mut out = elliptic(y[0].abs(), y[1], self.kc).unwrap_or([f64::NAN; 2]);
        if y[0] < 0. {
            out[0] = self.w2 - out[0];
        }
        if !north {
            let i = if -PI / 3. > p.lon.l {
                0
            } else if PI / 3. >= p.lon.l {
                1
            } else {
                2
            };
            let kr = [0.5, -1., 0.5][i];
            let ki = [-1., 0., 1.][i] * sqrt(3.) / 2.;
            let l = 2. * (kr * (self.cr[i] - out[0]) + ki * (self.ci[i] - out[1]));
            out[0] += l * kr;
            out[1] += l * ki;
        }
        out
    }
}
// Eisenlohr forward formula from d3-geo-projection4.0.0 (ISC/BSD notices
// retained in docs/licenses/d3-geo-projection-4.0.0.txt), normalized to libmap
// units by omitting d3's (3+2sqrt(2)) factor. Longitude is east-positive here.
fn eisenlohr(p: Place) -> [f64; 2] {
    let longitude = -p.lon.l / 2.;
    let latitude = p.lat.l / 2.;
    let sl = sin(longitude);
    let cl = cos(longitude);
    let root = sqrt(p.lat.c);
    let cp = cos(latitude);
    let t = sin(latitude) / (cp + sqrt(2.) * cl * root);
    let c = sqrt(2. / (1. + t * t));
    let v = sqrt((sqrt(2.) * cp + (cl + sl) * root) / (sqrt(2.) * cp + (cl - sl) * root));
    [
        c * (v - 1. / v) - 2. * log(v),
        c * t * (v + 1. / v) - 2. * libm::atan(t),
    ]
}
