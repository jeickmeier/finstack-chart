//! Retained authored colors and the explicit sRGB8 preparation boundary.
use super::{ColorDescriptor, ColorValue};
use crate::scene::Color;

/// An authored byte color or versioned floating color descriptor.
/// Legacy byte objects keep their exact JSON representation and meaning.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(untagged)]
pub enum Paint {
    /// Legacy unpremultiplied sRGB8 channels.
    Bytes(Color),
    /// Retained color space and floating channels, including explicit exceptional tags.
    Value(ColorDescriptor),
}
impl Paint {
    /// Convert once at preparation; renderers consume the resulting compact bytes.
    pub fn resolve(self) -> Color {
        match self {
            Self::Bytes(value) => value,
            Self::Value(value) => value.value().to_paint(),
        }
    }
    /// Floating RGB for a legacy byte input, otherwise its original space/channels.
    pub fn value(self) -> ColorValue {
        match self {
            Self::Bytes(value) => ColorValue::from(super::rgb(
                f64::from(value.red),
                f64::from(value.green),
                f64::from(value.blue),
            ))
            .with_opacity(f64::from(value.alpha) / 255.),
            Self::Value(value) => value.value(),
        }
    }
    /// Whether this input requires the authored floating-color capability.
    pub fn is_floating(self) -> bool {
        matches!(self, Self::Value(_))
    }
}
impl From<Color> for Paint {
    fn from(value: Color) -> Self {
        Self::Bytes(value)
    }
}
impl From<ColorValue> for Paint {
    fn from(value: ColorValue) -> Self {
        Self::Value(ColorDescriptor::new(value))
    }
}
impl From<ColorDescriptor> for Paint {
    fn from(value: ColorDescriptor) -> Self {
        Self::Value(value)
    }
}
macro_rules! input {
    ($($ty:ty),*) => {$(impl From<$ty> for Paint {
        fn from(value: $ty) -> Self { Self::from(ColorValue::from(value)) }
    })*};
}
input!(
    super::Rgb,
    super::Hsl,
    super::Lab,
    super::Hcl,
    super::Cubehelix
);

impl Paint {
    /// Parse authored CSS once. Six/eight-digit legacy hex keeps exact byte semantics.
    pub fn from_css(css: &str) -> crate::ChartResult<Self> {
        if let Some(hex) = css
            .strip_prefix('#')
            .filter(|h| matches!(h.len(), 6 | 8) && h.is_ascii())
        {
            return hex_bytes(hex).map(Into::into);
        }
        super::color(css)?.map(Into::into).ok_or_else(|| {
            super::error(
                crate::DiagnosticCode::Validation,
                "Invalid authored CSS color.",
            )
        })
    }
}
impl<'de> serde::Deserialize<'de> for Paint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Bytes(Color),
            Value(ColorDescriptor),
            Css(String),
        }
        match Wire::deserialize(deserializer)? {
            Wire::Bytes(value) => Ok(Self::Bytes(value)),
            Wire::Value(value) => Ok(Self::Value(value)),
            Wire::Css(css) => Self::from_css(&css).map_err(|e| serde::de::Error::custom(e.message)),
        }
    }
}

pub(super) fn hex_bytes(hex: &str) -> crate::ChartResult<Color> {
    let digits = hex.as_bytes();
    let invalid = || {
        super::error(
            crate::DiagnosticCode::Validation,
            "Invalid hexadecimal color.",
        )
    };
    if !matches!(digits.len(), 3 | 4 | 6 | 8) || !digits.iter().all(u8::is_ascii_hexdigit) {
        return Err(invalid());
    }
    let nibble = |i: usize| {
        char::from(digits[i])
            .to_digit(16)
            .map(|v| v as u8)
            .ok_or_else(invalid)
    };
    let short = digits.len() <= 4;
    let channel = |i: usize| -> crate::ChartResult<u8> {
        if short {
            Ok(nibble(i)? * 17)
        } else {
            Ok(nibble(i * 2)? * 16 + nibble(i * 2 + 1)?)
        }
    };
    Ok(Color {
        red: channel(0)?,
        green: channel(1)?,
        blue: channel(2)?,
        alpha: if matches!(digits.len(), 4 | 8) {
            channel(3)?
        } else {
            255
        },
    })
}
