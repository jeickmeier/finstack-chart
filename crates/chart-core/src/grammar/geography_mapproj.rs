//! Safe immutable translation of Plan 9 libmap kernels, Copyright 2021 Plan 9 Foundation.
//! Distributed under MIT; see docs/licenses/plan9-mit.txt and ADR031 for provenance.
use super::{GeoPosition, MapprojMethod, MapprojProjection};
use crate::ChartResult;
use libm::{acos, asin, atan2, cos, exp, log, sin, sqrt};
use std::f64::consts::{FRAC_PI_2, PI};
const RAD: f64 = PI / 180.;
const ECC: f64 = 0.08227185422;
fn tan(v: f64) -> f64 {
    sin(v) / cos(v)
}
#[derive(Clone, Copy)]
pub(super) struct Coord {
    pub(super) l: f64,
    pub(super) s: f64,
    pub(super) c: f64,
}
impl Coord {
    pub(super) fn radians(l: f64) -> Self {
        Self {
            l,
            s: sin(l),
            c: cos(l),
        }
    }
    pub(super) fn degrees(v: f64) -> Self {
        let v = cirmod(v);
        if v == 90. {
            Self {
                l: v * RAD,
                s: 1.,
                c: 0.,
            }
        } else if v == -90. {
            Self {
                l: v * RAD,
                s: -1.,
                c: 0.,
            }
        } else {
            Self::radians(v * RAD)
        }
    }
}
pub(super) fn cirmod(v: f64) -> f64 {
    let v = libm::fmod(v, 360.);
    if v >= 180. {
        v - 360.
    } else if v < -180. {
        v + 360.
    } else {
        v
    }
}
#[derive(Clone, Copy)]
pub(super) struct Place {
    pub(super) lat: Coord,
    pub(super) lon: Coord,
}
pub(super) fn normalize(mut p: Place, pole: Place, twist: Coord) -> Place {
    if pole.lat.s == 1. {
        if pole.lon.l + twist.l == 0. {
            return p;
        }
        p.lon.l -= pole.lon.l + twist.l;
    } else {
        if pole.lon.l != 0. {
            p.lon = Coord::radians(p.lon.l - pole.lon.l);
        }
        let s = pole.lat.s * p.lat.s + pole.lat.c * p.lat.c * p.lon.c;
        let c = sqrt(1. - s * s);
        let lon = atan2(
            p.lat.c * p.lon.s,
            -(pole.lat.c * p.lat.s - pole.lat.s * p.lat.c * p.lon.c),
        ) - twist.l;
        p = Place {
            lat: Coord {
                l: atan2(s, c),
                s,
                c,
            },
            lon: Coord::radians(lon),
        };
    }
    p.lon = Coord::radians(p.lon.l);
    if p.lon.l > PI {
        p.lon.l -= 2. * PI;
    } else if p.lon.l < -PI {
        p.lon.l += 2. * PI;
    }
    p
}
fn oriented(input: GeoPosition, o: [f64; 3]) -> Place {
    let mut lat = cirmod(o[0]);
    let mut lon = -o[1];
    let mut twist = -o[2];
    if lat > 90. {
        lat = 180. - lat;
        lon -= 180.;
        twist -= 180.;
    } else if lat < -90. {
        lat = -180. - lat;
        lon -= 180.;
        twist -= 180.;
    }
    normalize(
        Place {
            lat: Coord::radians(input[1] * PI / 180.),
            lon: Coord::radians(-(input[0] * PI / 180.)),
        },
        Place {
            lat: Coord::degrees(lat),
            lon: Coord::degrees(lon),
        },
        Coord::degrees(twist),
    )
}
fn polar(r: f64, p: Place) -> GeoPosition {
    [-r * p.lon.s, -r * p.lon.c]
}
pub(super) fn perspective(p: Place, v: f64) -> Option<GeoPosition> {
    if v >= 1000. {
        return (p.lat.l >= 0.).then(|| polar(p.lat.c, p));
    }
    if v <= 1.0001 && p.lat.s <= v + 0.01 {
        return None;
    }
    let r = p.lat.c * (v - 1.) / (v - p.lat.s);
    if r > 4. || (v.abs() > 1. && p.lat.s < 1. / v) || (v.abs() <= 1. && p.lat.s < v) {
        return None;
    }
    Some(polar(r, p))
}
pub(super) fn stereographic_raw(p: Place) -> GeoPosition {
    polar(2. * p.lat.c / (1. + p.lat.s), p)
}
fn rectangular(p: Place, a: f64) -> Option<GeoPosition> {
    let c = cos(a * RAD);
    (c >= 0.1).then_some([-c * p.lon.l, p.lat.l])
}
fn kernel(method: MapprojMethod, a: f64, b: f64, p: Place) -> ChartResult<Option<GeoPosition>> {
    use MapprojMethod::*;
    let lat = p.lat;
    let lon = p.lon;
    let value = match method {
        Mercator | SpMercator => {
            if lat.l.abs() > 80. * RAD {
                return Ok(None);
            }
            let mut y = 0.5 * log((1. + lat.s) / (1. - lat.s));
            if method == SpMercator {
                y += 0.5 * ECC * log((1. - ECC * lat.s) / (1. + ECC * lat.s));
            }
            [-lon.l, y]
        }
        Sinusoidal => [-lon.l * lat.c, lat.l],
        Cylindrical => {
            if lat.l.abs() > 80. * RAD {
                return Ok(None);
            }
            [-lon.l, lat.s / lat.c]
        }
        Cylequalarea => [-lon.l * Coord::degrees(a).c.powi(2), lat.s],
        Rectangular => return Ok(rectangular(p, a)),
        Gall => [
            -cos(a * RAD) / (2. * cos(a * RAD / 2.).powi(2)) * lon.l,
            if lat.s.abs() < 0.1 {
                sin(lat.l / 2.) / cos(lat.l / 2.)
            } else {
                (1. - lat.c) / lat.s
            },
        ],
        Mollweide => {
            let mut z = lat.l;
            if z.abs() < 89.9 * RAD {
                for _ in 0..64 {
                    let w = (2. * z + sin(2. * z) - PI * lat.s) / (2. + 2. * cos(2. * z));
                    z -= w;
                    if w.abs() < 0.00001 {
                        break;
                    }
                }
            }
            [-(2. / PI) * cos(z) * lon.l, sin(z)]
        }
        Gilbert => {
            let s = tan(lat.l / 2.).clamp(-1., 1.);
            [-sin(lon.l / 2.) * sqrt(1. - s * s), s]
        }
        Azequidistant => polar(FRAC_PI_2 - lat.l, p),
        Azequalarea => polar(sqrt(1. - lat.s), p),
        Perspective => return Ok(perspective(p, a)),
        Gnomonic => return Ok(perspective(p, 0.)),
        Stereographic => return Ok(perspective(p, -1.)),
        Orthographic => return Ok((lat.l >= 0.).then(|| polar(lat.c, p))),
        Laue => {
            if lat.l < PI / 4. + 0.0001 {
                return Ok(None);
            }
            let r = tan(PI - 2. * lat.l);
            if r > 3. {
                return Ok(None);
            }
            polar(r, p)
        }
        Fisheye => {
            let u = sin(PI / 4. - lat.l / 2.) / a;
            if u.abs() > 0.97 {
                return Ok(None);
            }
            polar(tan(asin(u)), p)
        }
        Newyorker => {
            let r = FRAC_PI_2 - lat.l;
            let s = if r < 0.001 {
                0.
            } else if r < a * RAD {
                return Ok(None);
            } else {
                log(r / (a * RAD))
            };
            polar(s, p)
        }
        Conic => {
            if a.abs() < 0.1 {
                return kernel(Cylindrical, 0., 0., p);
            }
            let c = Coord::degrees(a);
            if (lat.l - c.l).abs() > 80. * RAD {
                return Ok(None);
            }
            let r = c.c / c.s - tan(lat.l - c.l);
            if r > 3. {
                return Ok(None);
            }
            [-r * sin(lon.l * c.s), -r * cos(lon.l * c.s)]
        }
        Simpleconic => {
            let c = Coord::degrees(a);
            let d = Coord::degrees(b);
            if (c.l + d.l).abs() < 0.01 {
                return Ok(rectangular(p, a));
            }
            let (k, r0) = if (c.l - d.l).abs() < 0.01 {
                (c.s / c.l, c.c / c.s + c.l)
            } else {
                let k = (d.c - c.c) / (c.l - d.l);
                (k, ((c.c + d.c) / k + d.l + c.l) / 2.)
            };
            let r = r0 - lat.l;
            [-r * sin(k * lon.l), -r * cos(k * lon.l)]
        }
        Lambert => {
            let (a, b) = if a.abs() > b.abs() { (b, a) } else { (a, b) };
            if (a + b).abs() < 0.1 {
                return kernel(Mercator, 0., 0., p);
            }
            if a > 89.5 && b > 89.5 {
                return Ok(perspective(p, -1.));
            }
            let c = Coord::degrees(a);
            let d = Coord::degrees(b);
            if lat.l < -80. * RAD {
                return Ok(None);
            }
            let k = if (b - a).abs() < 0.1 {
                c.s + 0.5 * (d.s - c.s)
            } else {
                2. * log(d.c / c.c) / log((1. + c.s) * (1. - d.s) / ((1. - c.s) * (1. + d.s)))
            };
            let mut r = if lat.l > 89. * RAD {
                0.
            } else {
                c.c * exp(0.5 * k * log((1. + c.s) * (1. - lat.s) / ((1. - c.s) * (1. + lat.s))))
            };
            if d.l < 0. {
                r = -r;
            }
            [-r * sin(k * lon.l), -r * cos(k * lon.l)]
        }
        Bonne => {
            if (a * RAD).abs() < 0.01 {
                return kernel(Sinusoidal, 0., 0., p);
            }
            let c = Coord::degrees(a);
            let r = c.c / c.s + c.l - lat.l;
            let alpha = if r < 0.001 {
                if c.c.abs() < 1e-10 {
                    lon.l
                } else if lat.c.abs() == 0. {
                    0.
                } else {
                    lon.l / (1. + c.c * c.c * c.c / lat.c / 3.)
                }
            } else {
                lon.l * lat.c / r
            };
            [-r * sin(alpha), -r * cos(alpha)]
        }
        Polyconic => {
            if lat.l.abs() > 0.01 {
                let r = lat.c / lat.s;
                let alpha = lon.l * lat.s;
                [-r * sin(alpha), lat.l + r * (1. - cos(alpha))]
            } else {
                let l2 = lon.l * lon.l;
                let n2 = lat.l * lat.l;
                [
                    -lon.l * (1. - n2 * (3. + l2) / 6.),
                    lat.l * (1. + (l2 / 2.) * (1. - (8. + l2) * n2 / 12.)),
                ]
            }
        }
        Aitoff => {
            let q = normalize(
                Place {
                    lon: Coord::radians(lon.l / 2.),
                    ..p
                },
                Place {
                    lat: Coord::degrees(0.),
                    lon: Coord::degrees(0.),
                },
                Coord::degrees(0.),
            );
            let [x, y] = polar(sqrt(1. - q.lat.s), q);
            [2. * x, y]
        }
        Bicentric => {
            if lon.c <= 0.01 || lat.c <= 0.01 {
                return Ok(None);
            }
            let x = -Coord::degrees(a.abs()).c * lon.s / lon.c;
            let y = lat.s / (lat.c * lon.c);
            if x * x + y * y > 9. {
                return Ok(None);
            }
            [x, y]
        }
        Elliptic => {
            if a.abs() < 1. {
                return kernel(Azequidistant, 0., 0., p);
            }
            let c = Coord::degrees(a.abs());
            let r1 = acos(lat.c * (lon.c * c.c - lon.s * c.s));
            let r2 = acos(lat.c * (lon.c * c.c + lon.s * c.s));
            let x = -(r1 * r1 - r2 * r2) / (4. * c.l);
            let mut y = sqrt(((r1 * r1 + r2 * r2) / 2. - (c.l * c.l + x * x)).max(0.));
            if lat.l < 0. {
                y = -y;
            }
            [x, y]
        }
        Trapezoidal => {
            if (a.abs() - b.abs()).abs() < 0.1 {
                return Ok(rectangular(p, a));
            }
            let c = Coord::degrees(a);
            let d = Coord::degrees(b);
            let k = if (b - a).abs() < 0.1 {
                d.s
            } else {
                (d.c - c.c) / (c.l - d.l)
            };
            let y = -d.l - d.c / k + lat.l;
            [y * k * lon.l, y]
        }
        Albers | SpAlbers => {
            return albers(
                p,
                a,
                b,
                if method == SpAlbers {
                    0.006768657997
                } else {
                    0.
                },
            );
        }
        Lagrange => {
            let q = Place {
                lat: Coord {
                    l: lat.l.abs(),
                    s: lat.s.abs(),
                    c: lat.c,
                },
                ..p
            };
            let [z1, z2] = perspective(q, -1.).unwrap_or([f64::NAN; 2]);
            let w = complex_sqrt([-z2 / 2., z1 / 2.]);
            let t = complex_div([w[0] - 1., w[1]], [w[0] + 1., w[1]]);
            [t[1], if lat.l < 0. { t[0] } else { -t[0] }]
        }
        Globular => two_circles(-2. * lon.l / PI, 2. * lat.l / PI, lat.c, lat.s),
        Vandergrinten => {
            let t = 2. * lat.l / PI;
            let v = if t.abs() >= 1. {
                1.
            } else {
                t.abs() / (1. + sqrt(1. - t * t))
            };
            let p2 = 2. * v / (1. + v);
            let mut out = two_circles(-lon.l / PI, v, sqrt(1. - p2 * p2), p2);
            if t < 0. {
                out[1] = -out[1];
            }
            out
        }
        Harrison => {
            let u2 = cos(b * RAD);
            let u3 = sin(b * RAD);
            let p1 = -lat.c * lon.s;
            let p2 = -lat.c * lon.c;
            let d = a * u2 + u3 * p2 - u2 * lat.s;
            if d < 0.01 || a * lat.s < 1. {
                return Ok(None);
            }
            let t = (1. + a * u2) / d;
            let x = t * p1;
            let y = t * p2 * u2 + (a - t * (a - lat.s)) * u3;
            if t < 0. || x * x + y * y > 16. {
                return Ok(None);
            }
            [x, y]
        }
        Lune => {
            let c = Coord::degrees(a);
            if lat.l < c.l - 0.0001 {
                return Ok(None);
            }
            // The conformal source consumes stereographic coordinates even when
            // its ordinary visible-cap return flag is false.
            let east = stereographic_raw(Place {
                lat: c,
                lon: Coord::degrees(-90.),
            });
            let xy = stereographic_raw(p);
            let z = [xy[0] / east[0], xy[1] / east[0]];
            let w1 = complex_pow([1. + z[0], z[1]], b / 180.);
            let w2 = complex_pow([1. - z[0], -z[1]], b / 180.);
            complex_div(
                [w1[0] - w2[0], w1[1] - w2[1]],
                [w1[0] + w2[0], w1[1] + w2[1]],
            )
        }
        Mecca | Homing => {
            let c = Coord::degrees(a);
            let (az, rad) = azimuth(p, c);
            if method == Homing {
                if lon.c < 0. {
                    return Ok(None);
                }
                [-rad.l * az.s, -rad.l * az.c]
            } else {
                let x = -lon.l;
                let y = if az.s.abs() < 0.02 {
                    -az.c * rad.s / c.c
                } else {
                    x * az.c / az.s
                };
                if y.abs() > 2. || rad.c < 0. {
                    return Ok(None);
                }
                [x, y]
            }
        }
        _ => return super::geography_mapproj_complex::project(method, p),
    };
    Ok(Some(value))
}
pub(super) fn project(
    spec: &MapprojProjection,
    input: GeoPosition,
) -> ChartResult<Option<GeoPosition>> {
    if input.iter().any(|v| !v.is_finite()) {
        return Ok(None);
    }
    let a = spec.parameters.first().copied().unwrap_or(0.);
    let b = spec.parameters.get(1).copied().unwrap_or(0.);
    // Source setproj deliberately leaves a null initializer as geographic identity.
    use MapprojMethod::*;
    let identity = match spec.method {
        Cylequalarea => a > 89.,
        Rectangular => cos(a * RAD) < 0.1,
        Gall => a.abs() > 80.,
        Perspective => (a - 1.).abs() < 0.0001,
        Fisheye => a < 0.1,
        Bicentric | Elliptic => a.abs() > 89.,
        Mecca | Homing => a.abs() > 80.,
        Lambert => (a + b).abs() >= 0.1 && a.abs().max(b.abs()) > 89.5 && a.min(b) <= 89.5,
        Simpleconic => ((a + b) * RAD).abs() < 0.01 && cos(a * RAD) < 0.1,
        Trapezoidal => (a.abs() - b.abs()).abs() < 0.1 && cos(a * RAD) < 0.1,
        Albers | SpAlbers => {
            let (mut lo, mut hi) = (a, b);
            for _ in 0..32 {
                if lo < -90. {
                    lo = -180. - lo;
                }
                if hi > 90. {
                    hi = 180. - hi;
                }
                if lo <= hi {
                    break;
                }
                std::mem::swap(&mut lo, &mut hi);
            }
            (hi - lo < 1. && lo <= 89.) || ((hi + lo).abs() < 1. && lo > 89.)
        }
        Harrison => a < 1.001 || 1. + a * cos(b * RAD) < sqrt(a * a - 1.),
        _ => false,
    };
    if identity {
        return Ok(Some(input));
    }
    kernel(spec.method, a, b, oriented(input, spec.orientation))
}
pub(super) fn complex_div(mut z: [f64; 2], mut w: [f64; 2]) -> [f64; 2] {
    if w[0].abs() < w[1].abs() {
        w = [w[1], -w[0]];
        z = [z[1], -z[0]];
    }
    let r = w[1] / w[0];
    let t = w[0] + r * w[1];
    [(z[0] + r * z[1]) / t, (z[1] - r * z[0]) / t]
}
pub(super) fn complex_sqrt(z: [f64; 2]) -> [f64; 2] {
    let x = z[0].abs();
    let y = z[1].abs();
    let (mut r, s) = if x >= y {
        if x == 0. {
            return [0., 0.];
        }
        (x, y / x)
    } else {
        (y, x / y)
    };
    r *= sqrt(1. + s * s);
    if z[0] > 0. {
        let u = sqrt((r + z[0]) / 2.);
        [u, z[1] / (2. * u)]
    } else {
        let mut v = sqrt((r - z[0]) / 2.);
        if z[1] < 0. {
            v = -v;
        }
        [z[1] / (2. * v), v]
    }
}
pub(super) fn complex_pow(z: [f64; 2], power: f64) -> [f64; 2] {
    let theta = power * atan2(z[1], z[0]);
    let r = libm::pow(libm::hypot(z[0], z[1]), power);
    [r * cos(theta), r * sin(theta)]
}
fn two_circles(m: f64, p: f64, p1: f64, p2: f64) -> [f64; 2] {
    if m > 0. {
        let [x, y] = two_circles(-m, p, p1, p2);
        return [-x, y];
    }
    if p < 0. {
        let [x, y] = two_circles(m, -p, p1, -p2);
        return [x, -y];
    }
    if p < 0.01 {
        return [m, p + (p2 - p) * (m / p1) * (m / p1)];
    }
    if m > -0.01 {
        return [m - m * p * p, p];
    }
    let b = if p >= 1. {
        1.
    } else if p > 0.99 {
        0.5 * (p + 1. + p1 * p1 / (1. - p))
    } else {
        0.5 * (p * p - p1 * p1 - p2 * p2) / (p - p2)
    };
    let a = 0.5 * (m - 1. / m);
    let t = m * m - p * p + 2. * (b * p - a * m);
    let bb = b * b;
    let qa = 1. + a * a / bb;
    let qb = -2. * a + a * t / bb;
    let qc = t * t / (4. * bb) - m * m + 2. * a * m;
    let disc = qb * qb - 4. * qa * qc;
    let x = if disc < 0. {
        0.
    } else {
        (-qb - sqrt(disc)) / (2. * qa)
    };
    [x, (x * a + t / 2.) / b]
}
fn albers(p: Place, mut a: f64, mut b: f64, e2: f64) -> ChartResult<Option<GeoPosition>> {
    for _ in 0..32 {
        if a < -90. {
            a = -180. - a;
        }
        if b > 90. {
            b = 180. - b;
        }
        if a <= b {
            break;
        }
        std::mem::swap(&mut a, &mut b);
    }
    if b - a < 1. {
        if a > 89. {
            return kernel(MapprojMethod::Azequalarea, 0., 0., p);
        }
        return Ok(None);
    }
    if (b + a).abs() < 1. {
        return kernel(MapprojMethod::Cylequalarea, a, 0., p);
    }
    let num = |s: f64| {
        if e2 == 0. {
            1.
        } else {
            let s = e2 * s * s;
            1. + s * (2. / 3. + s * (3. / 5. + s * (4. / 7. + s * 5. / 9.)))
        }
    };
    let den = num(1.);
    let c = Coord::degrees(a);
    let d = Coord::degrees(b);
    let sb1 = c.s * num(c.s) / den;
    let sb2 = d.s * num(d.s) / den;
    let n = (c.c * c.c / (1. - e2 * c.s * c.s) - d.c * d.c / (1. - e2 * d.s * d.s))
        / (2. * (1. - e2) * den * (sb2 - sb1));
    let r1 = c.c / (n * sqrt(1. - e2 * c.s * c.s));
    let r0sq = r1 * r1 + 2. * (1. - e2) * den * sb1 / n;
    let r = sqrt(r0sq - 2. * (1. - e2) * p.lat.s * num(p.lat.s) / n);
    let t = n * p.lon.l;
    let (mut x, mut y) = (-r * sin(t), r * cos(t));
    if a < 0. && d.c > c.c {
        x = -x;
    } else {
        y = -y;
    }
    Ok(Some([x, y]))
}
fn azimuth(p: Place, c: Coord) -> (Coord, Coord) {
    if p.lat.c < 0.0001 {
        let az = Coord::radians(FRAC_PI_2 + p.lat.l - p.lon.l);
        let mut r = (p.lat.l - c.l).abs();
        if r > PI {
            r = 2. * PI - r;
        }
        return (az, Coord::radians(r));
    }
    let rc = (c.s * p.lat.s + c.c * p.lat.c * p.lon.c).clamp(-1., 1.);
    let rs = sqrt(1. - rc * rc);
    let az = if rs.abs() < 0.001 {
        Coord {
            l: 0.,
            s: 0.,
            c: 1.,
        }
    } else {
        Coord {
            l: 0.,
            s: (c.c * p.lon.s / rs).clamp(-1., 1.),
            c: ((c.s - rc * p.lat.s) / (rs * p.lat.c)).clamp(-1., 1.),
        }
    };
    (
        az,
        Coord {
            l: atan2(rs, rc),
            s: rs,
            c: rc,
        },
    )
}
