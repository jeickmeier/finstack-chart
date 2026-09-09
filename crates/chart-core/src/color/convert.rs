//! Source-adapted d3-color 3.1.0 conversion formulae; ISC notice in LICENSE.
use super::*;
const XN: f64 = 0.96422;
const ZN: f64 = 0.82521;
const T0: f64 = 4. / 29.;
const T1: f64 = 6. / 29.;
const T2: f64 = 3. * T1 * T1;
const T3: f64 = T1 * T1 * T1;
const A: f64 = -0.14861;
const B: f64 = 1.78277;
const C: f64 = -0.29227;
const D: f64 = -0.90649;
const E: f64 = 1.97294;
const ED: f64 = E * D;
const EB: f64 = E * B;
const BC_DA: f64 = B * C - D * A;
fn truthy(v: f64) -> bool {
    v != 0. && !v.is_nan()
}
fn hsl2rgb(h: f64, m1: f64, m2: f64) -> f64 {
    (if h < 60. {
        m1 + (m2 - m1) * h / 60.
    } else if h < 180. {
        m2
    } else if h < 240. {
        m1 + (m2 - m1) * (240. - h) / 60.
    } else {
        m1
    }) * 255.
}
fn rgb2linear(v: f64) -> f64 {
    let v = v / 255.;
    if v <= 0.04045 {
        v / 12.92
    } else {
        pxfm::f_pow((v + 0.055) / 1.055, 2.4)
    }
}
fn linear2rgb(v: f64) -> f64 {
    255. * if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * pxfm::f_pow(v, 1. / 2.4) - 0.055
    }
}
fn xyz2lab(t: f64) -> f64 {
    if t > T3 {
        pxfm::f_pow(t, 1. / 3.)
    } else {
        t / T2 + T0
    }
}
fn lab2xyz(t: f64) -> f64 {
    if t > T1 { t * t * t } else { T2 * (t - T0) }
}
impl ColorValue {
    /// Convert to unclamped RGB; same-space values preserve their exact channels.
    pub fn rgb(self) -> Rgb {
        match self {
            Self::Rgb(v) => v,
            Self::Hsl(v) => {
                let h = v.h % 360. + if v.h < 0. { 360. } else { 0. };
                let s = if h.is_nan() || v.s.is_nan() { 0. } else { v.s };
                let m2 = v.l + if v.l < 0.5 { v.l } else { 1. - v.l } * s;
                let m1 = 2. * v.l - m2;
                rgb(
                    hsl2rgb(if h >= 240. { h - 240. } else { h + 120. }, m1, m2),
                    hsl2rgb(h, m1, m2),
                    hsl2rgb(if h < 120. { h + 240. } else { h - 120. }, m1, m2),
                )
                .opacity(v.opacity)
            }
            Self::Lab(v) => {
                let y = (v.l + 16.) / 116.;
                let x = if v.a.is_nan() { y } else { y + v.a / 500. };
                let z = if v.b.is_nan() { y } else { y - v.b / 200. };
                let x = XN * lab2xyz(x);
                let y = lab2xyz(y);
                let z = ZN * lab2xyz(z);
                rgb(
                    linear2rgb(3.1338561 * x - 1.6168667 * y - 0.4906146 * z),
                    linear2rgb(-0.9787684 * x + 1.9161415 * y + 0.0334540 * z),
                    linear2rgb(0.0719453 * x - 0.2289914 * y + 1.4052427 * z),
                )
                .opacity(v.opacity)
            }
            Self::Hcl(_) => ColorValue::Lab(self.lab()).rgb(),
            Self::Cubehelix(v) => {
                let h = if v.h.is_nan() {
                    0.
                } else {
                    (v.h + 120.) * (std::f64::consts::PI / 180.)
                };
                let a = if v.s.is_nan() {
                    0.
                } else {
                    v.s * v.l * (1. - v.l)
                };
                let cos = super::trig::cos(h);
                let sin = super::trig::sin(h);
                rgb(
                    255. * (v.l + a * (A * cos + B * sin)),
                    255. * (v.l + a * (C * cos + D * sin)),
                    255. * (v.l + a * (E * cos)),
                )
                .opacity(v.opacity)
            }
        }
    }
    /// Convert to HSL, preserving undefined achromatic hue/saturation semantics.
    pub fn hsl(self) -> Hsl {
        if let Self::Hsl(v) = self {
            return v;
        }
        let v = self.rgb();
        let r = v.r / 255.;
        let g = v.g / 255.;
        let b = v.b / 255.;
        let (min, max) = if r.is_nan() || g.is_nan() || b.is_nan() {
            (f64::NAN, f64::NAN)
        } else {
            (r.min(g).min(b), r.max(g).max(b))
        };
        let mut h = f64::NAN;
        let mut s = max - min;
        let l = (max + min) / 2.;
        if truthy(s) {
            h = if r == max {
                (g - b) / s + if g < b { 6. } else { 0. }
            } else if g == max {
                (b - r) / s + 2.
            } else {
                (r - g) / s + 4.
            };
            s /= if l < 0.5 { max + min } else { 2. - max - min };
            h *= 60.;
        } else {
            s = if l > 0. && l < 1. { 0. } else { h };
        }
        hsl(h, s, l).opacity(v.opacity)
    }
    /// Convert to D50 Lab; the direct HCL path never quantizes through RGB.
    pub fn lab(self) -> Lab {
        match self {
            Self::Lab(v) => v,
            Self::Hcl(v) => {
                if v.h.is_nan() {
                    return lab(v.l, 0., 0.).opacity(v.opacity);
                }
                let h = v.h * (std::f64::consts::PI / 180.);
                lab(v.l, super::trig::cos(h) * v.c, super::trig::sin(h) * v.c).opacity(v.opacity)
            }
            _ => {
                let v = self.rgb();
                let r = rgb2linear(v.r);
                let g = rgb2linear(v.g);
                let b = rgb2linear(v.b);
                let y = xyz2lab(0.2225045 * r + 0.7168786 * g + 0.0606169 * b);
                let (x, z) = if r == g && g == b {
                    (y, y)
                } else {
                    (
                        xyz2lab((0.4360747 * r + 0.3850649 * g + 0.1430804 * b) / XN),
                        xyz2lab((0.0139322 * r + 0.0971045 * g + 0.7141733 * b) / ZN),
                    )
                };
                lab(116. * y - 16., 500. * (x - y), 200. * (y - z)).opacity(v.opacity)
            }
        }
    }
    /// Convert to cylindrical Lab; neutral endpoint hue/chroma remain undefined.
    pub fn hcl(self) -> Hcl {
        if let Self::Hcl(v) = self {
            return v;
        }
        let v = self.lab();
        if v.a == 0. && v.b == 0. {
            return hcl(
                f64::NAN,
                if 0. < v.l && v.l < 100. { 0. } else { f64::NAN },
                v.l,
            )
            .opacity(v.opacity);
        }
        let h = libm::atan2(v.b, v.a) * (180. / std::f64::consts::PI);
        hcl(
            if h < 0. { h + 360. } else { h },
            (v.a * v.a + v.b * v.b).sqrt(),
            v.l,
        )
        .opacity(v.opacity)
    }
    /// Convert using the pinned Cubehelix coefficients and singular endpoint behavior.
    pub fn cubehelix(self) -> Cubehelix {
        if let Self::Cubehelix(v) = self {
            return v;
        }
        let v = self.rgb();
        let r = v.r / 255.;
        let g = v.g / 255.;
        let b = v.b / 255.;
        let l = (BC_DA * b + ED * r - EB * g) / (BC_DA + ED - EB);
        let bl = b - l;
        let k = (E * (g - l) - C * bl) / D;
        let s = (k * k + bl * bl).sqrt() / (E * l * (1. - l));
        let h = if truthy(s) {
            libm::atan2(k, bl) * (180. / std::f64::consts::PI) - 120.
        } else {
            f64::NAN
        };
        cubehelix(if h < 0. { h + 360. } else { h }, s, l).opacity(v.opacity)
    }
}
