//! Reference display fallbacks and CSS strings; conversion stays separate from quantization.
use super::*;
pub(super) fn alpha(v: f64) -> f64 {
    if v.is_nan() {
        1.
    } else if v <= 0. {
        0.
    } else {
        v.min(1.)
    }
}
pub(super) fn byte(v: f64) -> u8 {
    if v.is_nan() {
        0
    } else {
        v.round().clamp(0., 255.) as u8
    }
}
fn unit(v: f64) -> f64 {
    if v.is_nan() || v <= 0. { 0. } else { v.min(1.) }
}
fn hue(v: f64) -> f64 {
    let v = if v.is_nan() || v == 0. { 0. } else { v } % 360.;
    if v < 0. { v + 360. } else { v }
}
use crate::number::ecmascript as number;
impl Rgb {
    /// Return rounded byte-range RGB channels and normalized coverage as a floating value.
    pub fn clamp(self) -> Self {
        rgb(
            byte(self.r).into(),
            byte(self.g).into(),
            byte(self.b).into(),
        )
        .opacity(alpha(self.opacity))
    }
}
impl Hsl {
    /// Wrap hue and bound saturation/lightness/coverage with reference undefined fallbacks.
    pub fn clamp(self) -> Self {
        hsl(hue(self.h), unit(self.s), unit(self.l)).opacity(alpha(self.opacity))
    }
}
impl ColorValue {
    /// Clamped lowercase hexadecimal sRGB.
    pub fn format_hex(self) -> String {
        let c = self.rgb();
        format!("#{:02x}{:02x}{:02x}", byte(c.r), byte(c.g), byte(c.b))
    }
    /// Clamped lowercase hexadecimal sRGB and alpha.
    pub fn format_hex8(self) -> String {
        let c = self.rgb();
        format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            byte(c.r),
            byte(c.g),
            byte(c.b),
            byte(if c.opacity.is_nan() {
                255.
            } else {
                c.opacity * 255.
            })
        )
    }
    /// Clamped comma-form RGB/RGBA, matching the reference string punctuation.
    pub fn format_rgb(self) -> String {
        let c = self.rgb();
        let a = alpha(c.opacity);
        if a == 1. {
            format!("rgb({}, {}, {})", byte(c.r), byte(c.g), byte(c.b))
        } else {
            format!(
                "rgba({}, {}, {}, {})",
                byte(c.r),
                byte(c.g),
                byte(c.b),
                number(a)
            )
        }
    }
    /// Clamped comma-form HSL/HSLA with full floating channel precision.
    pub fn format_hsl(self) -> String {
        let c = self.hsl().clamp();
        let h = number(c.h);
        let s = number(c.s * 100.);
        let l = number(c.l * 100.);
        if c.opacity == 1. {
            format!("hsl({h}, {s}%, {l}%)")
        } else {
            format!("hsla({h}, {s}%, {l}%, {})", number(c.opacity))
        }
    }
    /// Deprecated upstream alias retained for method parity.
    pub fn hex(self) -> String {
        self.format_hex()
    }
}
impl std::fmt::Display for ColorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format_rgb())
    }
}
