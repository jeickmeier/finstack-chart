//! D65 conversions for the pinned ggplot2 palette policies. D3's D50 values
//! retain their existing contract. Matrices use the sRGB/XYZ D65 coefficients;
//! Lab's decimal breakpoint follows farver 2.1.2 for reference byte agreement.
use super::{Rgb, rgb};

fn decode(v: f64) -> f64 {
    let v = v / 255.;
    if v <= 0.04045 {
        v / 12.92
    } else {
        pxfm::f_pow((v + 0.055) / 1.055, 2.4)
    }
}
fn encode(v: f64) -> f64 {
    255. * if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * pxfm::f_pow(v, 1. / 2.4) - 0.055
    }
}
fn from_xyz([x, y, z]: [f64; 3]) -> Rgb {
    rgb(
        encode(3.2404542 * x - 1.5371385 * y - 0.4985314 * z),
        encode(-0.9692660 * x + 1.8760108 * y + 0.0415560 * z),
        encode(0.0556434 * x - 0.2040259 * y + 1.0572252 * z),
    )
}
pub(crate) fn lab(value: Rgb) -> [f64; 3] {
    let [r, g, b] = [value.r, value.g, value.b].map(decode);
    let f = |x: f64| {
        if x > 0.008856 {
            x.cbrt()
        } else {
            7.787 * x + 16. / 116.
        }
    };
    let x = f((0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047);
    let y = f(0.2126729 * r + 0.7151522 * g + 0.0721750 * b);
    let z = f((0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883);
    [116. * y - 16., 500. * (x - y), 200. * (y - z)]
}
pub(crate) fn from_lab([l, a, b]: [f64; 3]) -> Rgb {
    let y = (l + 16.) / 116.;
    let inverse = |v: f64| {
        if v * v * v > 0.008856 {
            v * v * v
        } else {
            (v - 16. / 116.) / 7.787
        }
    };
    from_xyz([
        0.95047 * inverse(y + a / 500.),
        inverse(y),
        1.08883 * inverse(y - b / 200.),
    ])
}
pub(crate) fn from_hcl(h: f64, c: f64, l: f64) -> Rgb {
    if l == 0. {
        return rgb(0., 0., 0.);
    }
    let angle = h.to_radians();
    let u = c * super::trig::cos(angle);
    let v = c * super::trig::sin(angle);
    let white_sum = 0.95047 + 15. + 3. * 1.08883;
    let up = u / (13. * l) + 4. * 0.95047 / white_sum;
    let vp = v / (13. * l) + 9. / white_sum;
    let y = if l > 8. {
        ((l + 16.) / 116.).powi(3)
    } else {
        l / (24389. / 27.)
    };
    let x = 9. * y * up / (4. * vp);
    let z = y * (12. - 3. * up - 20. * vp) / (4. * vp);
    from_xyz([x, y, z])
}

// R grDevices derives the sRGB matrix from its chromaticities and white point
// x=.3137, y=.3291. These independently captured matrix values are numerical
// reference data, distinct from farver's standardized rounded matrix above.
pub(crate) fn device_lab(value: Rgb) -> [f64; 3] {
    let [r, g, b] = [value.r, value.g, value.b].map(decode);
    let f = |x: f64| {
        if x > 216. / 24389. {
            pxfm::f_pow(x, 1. / 3.)
        } else {
            (24389. / 27. * x + 16.) / 116.
        }
    };
    let x = f(
        (0.4168213418853169 * r + 0.3565767170779746 * g + 0.1798076535860854 * b)
            / (0.3137 / 0.3291),
    );
    let y = f(0.21492350440961652 * r + 0.7131534341559492 * g + 0.07192306143443415 * b);
    let z = f(
        (0.019538500400874244 * r + 0.11885890569265832 * g + 0.9469869755533831 * b)
            / ((1. - 0.3137 - 0.3291) / 0.3291),
    );
    [116. * y - 16., 500. * (x - y), 200. * (y - z)]
}
pub(crate) fn from_device_lab([l, a, b]: [f64; 3]) -> Rgb {
    let y = (l + 16.) / 116.;
    let inverse = |v: f64| {
        if v * v * v > 216. / 24389. {
            v * v * v
        } else {
            (116. * v - 16.) / (24389. / 27.)
        }
    };
    let x = (0.3137 / 0.3291) * inverse(y + a / 500.);
    let z = ((1. - 0.3137 - 0.3291) / 0.3291) * inverse(y - b / 200.);
    let y = inverse(y);
    let device_encode =
        |v| ((encode(v) / 255. * 100_000.).round_ties_even() / 100_000.).clamp(0., 1.) * 255.;
    rgb(
        device_encode(3.2065205171444644 * x - 1.521041783773656 * y - 0.493310848791456 * z),
        device_encode(-0.971982546201232 * x + 1.8812686516084873 * y + 0.04167248459958932 * z),
        device_encode(0.05583833859309792 * x - 0.204740574841359 * y + 1.060928433268859 * z),
    )
}
