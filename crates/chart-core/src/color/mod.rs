//! Floating color values with the d3-color 3.1.0 compatibility contract.
//!
//! Conversion and manipulation preserve out-of-gamut and undefined channels.
//! Only [`ColorValue::to_paint`] quantizes to the existing sRGB8 scene boundary.
//! Algorithms adapted from d3-color; see the retained ISC notice in `LICENSE`.
mod authoring;
mod paint;
pub use paint::Paint;
mod convert;
pub(crate) mod d65;
mod format;
mod parse;
mod r_parse;
pub use r_parse::{R_DEFAULT_PALETTE, parse_r, parse_r_with_palette};
mod scalar;
pub(crate) mod trig;

use crate::{ChartResult, Diagnostic, DiagnosticCode};
pub use parse::{MAX_CSS_BYTES, parse, parse_with_limit};
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use the documented color grammar, channels and descriptor version.",
    )
}
fn equal(a: f64, b: f64) -> bool {
    a == b || (a.is_nan() && b.is_nan())
}
macro_rules! channels {
    ($name:ident, $doc:literal, $($field:ident : $field_doc:literal),+) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $(#[doc = $field_doc] #[serde(with="scalar")] pub $field: f64,)+
            /// Unclamped linear alpha coverage.
            #[serde(with="scalar")]
            pub opacity: f64,
        }
        impl PartialEq for $name {
            fn eq(&self, rhs: &Self) -> bool { $(equal(self.$field,rhs.$field) &&)+ equal(self.opacity,rhs.opacity) }
        }
        impl Eq for $name {}
        impl $name {
            /// Construct an opaque floating value without clamping channels.
            pub fn new($($field:f64),+) -> Self { Self { $($field,)+ opacity:1. } }
            /// Copy with explicitly authored coverage, including exceptional values.
            pub fn opacity(mut self, opacity:f64) -> Self { self.opacity=opacity; self }
            /// Return an independent same-space value.
            pub fn copy(self) -> Self { self }
            /// Scale/add the space's lightness using the reference default when omitted.
            pub fn brighter(self, k:Option<f64>) -> Self {
                let ColorValue::$name(v) = ColorValue::$name(self).brightness(k, false) else { unreachable!() }; v
            }
            /// Inverse brightness operation, preserving coverage.
            pub fn darker(self, k:Option<f64>) -> Self {
                let ColorValue::$name(v) = ColorValue::$name(self).brightness(k, true) else { unreachable!() }; v
            }
            /// Convert to unclamped sRGB channels.
            pub fn rgb(self) -> Rgb { ColorValue::$name(self).rgb() }
            /// Reference displayability predicate; this does not clamp the value.
            pub fn displayable(self) -> bool { ColorValue::$name(self).displayable() }
            /// Lowercase clamped hexadecimal sRGB output.
            pub fn format_hex(self) -> String { ColorValue::$name(self).format_hex() }
            /// Lowercase hexadecimal sRGB and alpha output.
            pub fn format_hex8(self) -> String { ColorValue::$name(self).format_hex8() }
            /// Clamped integer RGB/RGBA CSS output.
            pub fn format_rgb(self) -> String { ColorValue::$name(self).format_rgb() }
            /// Clamped HSL/HSLA CSS output.
            pub fn format_hsl(self) -> String { ColorValue::$name(self).format_hsl() }
            /// Compatibility alias for format_hex.
            pub fn hex(self) -> String { self.format_hex() }
        }
        impl From<$name> for ColorValue { fn from(v:$name)->Self { Self::$name(v) } }
        impl std::fmt::Display for $name {
            fn fmt(&self, f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.write_str(&self.format_rgb()) }
        }
    }
}
channels!(Rgb, "Unclamped nonlinear sRGB channels on a nominal 0–255 scale.", r:"Red channel.",g:"Green channel.",b:"Blue channel.");
channels!(Hsl, "Cylindrical HSL; hue is degrees and saturation/lightness nominally 0–1.", h:"Hue in degrees; undefined hue is NaN.",s:"Saturation.",l:"Lightness.");
channels!(Lab, "CIELAB with the d3-color D50 white point and nominal lightness 0–100.", l:"Lightness.",a:"Green/red opponent channel.",b:"Blue/yellow opponent channel.");
channels!(Hcl, "Cylindrical D50 Lab; LCh differs only in constructor argument order.", h:"Hue in degrees.",c:"Chroma.",l:"Lightness.");
channels!(Cubehelix, "Cubehelix hue, saturation and lightness using the reference coefficients.", h:"Hue in degrees.",s:"Saturation.",l:"Lightness.");

/// Opaque RGB constructor; use the returned value's opacity builder for explicit alpha.
pub fn rgb(r: f64, g: f64, b: f64) -> Rgb {
    Rgb::new(r, g, b)
}
/// Opaque HSL constructor.
pub fn hsl(h: f64, s: f64, l: f64) -> Hsl {
    Hsl::new(h, s, l)
}
/// Opaque D50 Lab constructor.
pub fn lab(l: f64, a: f64, b: f64) -> Lab {
    Lab::new(l, a, b)
}
/// Neutral Lab constructor, distinct from presentation-theme grayscale conversion.
pub fn gray(l: f64) -> Lab {
    Lab::new(l, 0., 0.)
}
/// Opaque cylindrical Lab constructor in hue/chroma/lightness order.
pub fn hcl(h: f64, c: f64, l: f64) -> Hcl {
    Hcl::new(h, c, l)
}
/// Cylindrical Lab alias in lightness/chroma/hue order.
pub fn lch(l: f64, c: f64, h: f64) -> Hcl {
    Hcl::new(h, c, l)
}
/// Opaque Cubehelix constructor.
pub fn cubehelix(h: f64, s: f64, l: f64) -> Cubehelix {
    Cubehelix::new(h, s, l)
}
/// Parse the pinned CSS grammar; malformed CSS yields None, oversized CSS an error.
pub fn color(css: &str) -> ChartResult<Option<ColorValue>> {
    parse(css)
}

/// Retained space identity; aliases lower to the same mathematical space.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ColorSpace {
    /// Nonlinear sRGB channels.
    Rgb,
    /// Cylindrical HSL.
    Hsl,
    /// D50 Lab.
    Lab,
    /// Cylindrical D50 Lab.
    Hcl,
    /// Cubehelix.
    Cubehelix,
}
/// Tagged floating value, retaining authored channels rather than a byte roundtrip.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "space", content = "channels", deny_unknown_fields)]
pub enum ColorValue {
    /// RGB value.
    Rgb(Rgb),
    /// HSL value.
    Hsl(Hsl),
    /// D50 Lab value.
    Lab(Lab),
    /// Cylindrical D50 Lab value.
    Hcl(Hcl),
    /// Cubehelix value.
    Cubehelix(Cubehelix),
}
impl ColorValue {
    /// Exact current space.
    pub fn space(self) -> ColorSpace {
        match self {
            Self::Rgb(_) => ColorSpace::Rgb,
            Self::Hsl(_) => ColorSpace::Hsl,
            Self::Lab(_) => ColorSpace::Lab,
            Self::Hcl(_) => ColorSpace::Hcl,
            Self::Cubehelix(_) => ColorSpace::Cubehelix,
        }
    }
    /// Current coverage without clamping.
    pub fn opacity(self) -> f64 {
        match self {
            Self::Rgb(v) => v.opacity,
            Self::Hsl(v) => v.opacity,
            Self::Lab(v) => v.opacity,
            Self::Hcl(v) => v.opacity,
            Self::Cubehelix(v) => v.opacity,
        }
    }
    /// Same-space independent copy.
    pub fn copy(self) -> Self {
        self
    }
    /// Copy with a new coverage value.
    pub fn with_opacity(self, opacity: f64) -> Self {
        match self {
            Self::Rgb(v) => Self::Rgb(v.opacity(opacity)),
            Self::Hsl(v) => Self::Hsl(v.opacity(opacity)),
            Self::Lab(v) => Self::Lab(v.opacity(opacity)),
            Self::Hcl(v) => Self::Hcl(v.opacity(opacity)),
            Self::Cubehelix(v) => Self::Cubehelix(v.opacity(opacity)),
        }
    }
    /// Convert to a requested space; same-space copies preserve all channels.
    pub fn convert(self, space: ColorSpace) -> Self {
        match space {
            ColorSpace::Rgb => self.rgb().into(),
            ColorSpace::Hsl => self.hsl().into(),
            ColorSpace::Lab => self.lab().into(),
            ColorSpace::Hcl => self.hcl().into(),
            ColorSpace::Cubehelix => self.cubehelix().into(),
        }
    }
    /// Constructor conversion from CSS, matching the reference's undefined invalid value.
    pub fn from_css(css: &str, space: ColorSpace) -> ChartResult<Self> {
        Ok(parse(css)?.map_or_else(|| Self::undefined(space), |v| v.convert(space)))
    }
    /// Undefined constructor/conversion value, distinct from parsing failure.
    pub fn undefined(space: ColorSpace) -> Self {
        let n = f64::NAN;
        match space {
            ColorSpace::Rgb => rgb(n, n, n).opacity(n).into(),
            ColorSpace::Hsl => hsl(n, n, n).opacity(n).into(),
            ColorSpace::Lab => lab(n, n, n).opacity(n).into(),
            ColorSpace::Hcl => hcl(n, n, n).opacity(n).into(),
            ColorSpace::Cubehelix => cubehelix(n, n, n).opacity(n).into(),
        }
    }
    /// Apply the space's reference brightness operation without clamping.
    pub fn brighter(self, k: Option<f64>) -> Self {
        self.brightness(k, false)
    }
    /// Apply the inverse brightness operation without clamping.
    pub fn darker(self, k: Option<f64>) -> Self {
        self.brightness(k, true)
    }
    fn brightness(self, k: Option<f64>, dark: bool) -> Self {
        let k = k.unwrap_or(1.);
        match self {
            Self::Lab(v) => lab(v.l + if dark { -18. * k } else { 18. * k }, v.a, v.b)
                .opacity(v.opacity)
                .into(),
            Self::Hcl(v) => hcl(v.h, v.c, v.l + if dark { -18. * k } else { 18. * k })
                .opacity(v.opacity)
                .into(),
            _ => {
                let factor = if dark { 0.7_f64 } else { 1. / 0.7 }.powf(k);
                match self {
                    Self::Rgb(v) => rgb(v.r * factor, v.g * factor, v.b * factor)
                        .opacity(v.opacity)
                        .into(),
                    Self::Hsl(v) => hsl(v.h, v.s, v.l * factor).opacity(v.opacity).into(),
                    Self::Cubehelix(v) => {
                        cubehelix(v.h, v.s, v.l * factor).opacity(v.opacity).into()
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    /// Reference displayability without mutating or normalizing channels.
    pub fn displayable(self) -> bool {
        if let Self::Hsl(v) = self {
            ((0. <= v.s && v.s <= 1.) || v.s.is_nan())
                && 0. <= v.l
                && v.l <= 1.
                && 0. <= v.opacity
                && v.opacity <= 1.
        } else {
            let v = self.rgb();
            [
                -0.5 <= v.r && v.r < 255.5,
                -0.5 <= v.g && v.g < 255.5,
                -0.5 <= v.b && v.b < 255.5,
                0. <= v.opacity && v.opacity <= 1.,
            ]
            .into_iter()
            .all(|b| b)
        }
    }
    /// Normalize RGB/HSL; other spaces have no reference clamp method.
    pub fn clamp(self) -> Option<Self> {
        match self {
            Self::Rgb(v) => Some(v.clamp().into()),
            Self::Hsl(v) => Some(v.clamp().into()),
            _ => None,
        }
    }
    /// Copy with one checked channel update; unrelated channel names reject.
    pub fn with_channel(self, name: &str, value: f64) -> ChartResult<Self> {
        if name == "opacity" {
            return Ok(self.with_opacity(value));
        }
        macro_rules! update { ($v:ident,$($field:ident),+) => {
            match name { $(stringify!($field)=>$v.$field=value,)+ _=>return Err(error(DiagnosticCode::SchemaConflict,"Channel does not belong to this color space.")) }
        }; }
        Ok(match self {
            Self::Rgb(mut v) => {
                update!(v, r, g, b);
                v.into()
            }
            Self::Hsl(mut v) => {
                update!(v, h, s, l);
                v.into()
            }
            Self::Lab(mut v) => {
                update!(v, l, a, b);
                v.into()
            }
            Self::Hcl(mut v) => {
                update!(v, h, c, l);
                v.into()
            }
            Self::Cubehelix(mut v) => {
                update!(v, h, s, l);
                v.into()
            }
        })
    }
    /// Quantize once to unpremultiplied sRGB8 using reference display fallbacks.
    pub fn to_paint(self) -> crate::scene::Color {
        let v = self.rgb();
        crate::scene::Color {
            red: format::byte(v.r),
            green: format::byte(v.g),
            blue: format::byte(v.b),
            alpha: format::byte(format::alpha(v.opacity) * 255.),
        }
    }
}

/// Versioned standalone color descriptor; independent of chart-envelope versions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "DescriptorWire", into = "DescriptorWire")]
pub struct ColorDescriptor {
    value: ColorValue,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DescriptorWire {
    version: u32,
    value: ColorValue,
}
impl TryFrom<DescriptorWire> for ColorDescriptor {
    type Error = String;
    fn try_from(w: DescriptorWire) -> Result<Self, String> {
        if w.version != 1 {
            return Err("Unsupported standalone color descriptor version.".into());
        }
        Ok(Self::new(w.value))
    }
}
impl From<ColorDescriptor> for DescriptorWire {
    fn from(d: ColorDescriptor) -> Self {
        Self {
            version: 1,
            value: d.value,
        }
    }
}
impl ColorDescriptor {
    /// Retain a value in version one.
    pub fn new(value: impl Into<ColorValue>) -> Self {
        Self {
            value: value.into(),
        }
    }
    /// Owned value independent of the descriptor lifetime.
    pub fn value(self) -> ColorValue {
        self.value
    }
    /// Serialize exceptional channels explicitly.
    pub fn to_json(self) -> ChartResult<String> {
        serde_json::to_string(&self).map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
    }
    /// Parse a bounded, strictly versioned descriptor.
    pub fn from_json(json: &str) -> ChartResult<Self> {
        if json.len() > 16_384 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Color descriptor exceeds its byte budget.",
            ));
        }
        serde_json::from_str(json).map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
    }
}
