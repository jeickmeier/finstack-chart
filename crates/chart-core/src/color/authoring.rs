//! Checked overload resolution shared by actual host color values.
use super::*;
impl ColorSpace {
    /// Stable descriptor space name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rgb => "Rgb",
            Self::Hsl => "Hsl",
            Self::Lab => "Lab",
            Self::Hcl => "Hcl",
            Self::Cubehelix => "Cubehelix",
        }
    }
    /// Resolve a supported constructor/space name; LCh aliases cylindrical Lab.
    pub fn from_name(name: &str) -> ChartResult<Self> {
        match name {
            "rgb" | "Rgb" => Ok(Self::Rgb),
            "hsl" | "Hsl" => Ok(Self::Hsl),
            "lab" | "Lab" => Ok(Self::Lab),
            "hcl" | "Hcl" | "lch" => Ok(Self::Hcl),
            "cubehelix" | "Cubehelix" => Ok(Self::Cubehelix),
            _ => Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unknown color space.",
            )),
        }
    }
}
impl ColorValue {
    /// Resolve numeric constructor overloads without host coercion or host color math.
    pub fn construct(name: &str, args: &[f64]) -> ChartResult<Self> {
        let base = if name == "gray" { 1 } else { 3 };
        if args.len() != base && args.len() != base + 1 {
            return Err(error(
                DiagnosticCode::Validation,
                "Color constructor requires its numeric channels and optional opacity.",
            ));
        }
        let value: Self = match name {
            "rgb" => rgb(args[0], args[1], args[2]).into(),
            "hsl" => hsl(args[0], args[1], args[2]).into(),
            "lab" => lab(args[0], args[1], args[2]).into(),
            "hcl" => hcl(args[0], args[1], args[2]).into(),
            "lch" => lch(args[0], args[1], args[2]).into(),
            "cubehelix" => cubehelix(args[0], args[1], args[2]).into(),
            "gray" => gray(args[0]).into(),
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Unknown color constructor.",
                ));
            }
        };
        Ok(value.with_opacity(args.get(base).copied().unwrap_or(1.)))
    }
    /// Read an authored floating channel; unknown channels reject.
    pub fn channel(self, name: &str) -> ChartResult<f64> {
        if name == "opacity" {
            return Ok(self.opacity());
        }
        macro_rules! read { ($v:ident,$($f:ident),+) => { match name { $(stringify!($f)=>Ok($v.$f),)+ _=>Err(error(DiagnosticCode::SchemaConflict,"Channel does not belong to this color space.")) } }; }
        match self {
            Self::Rgb(v) => read!(v, r, g, b),
            Self::Hsl(v) => read!(v, h, s, l),
            Self::Lab(v) => read!(v, l, a, b),
            Self::Hcl(v) => read!(v, h, c, l),
            Self::Cubehelix(v) => read!(v, h, s, l),
        }
    }
    /// Exact canonical tagged value with explicit exceptional channel tags.
    pub fn value_json(self) -> ChartResult<String> {
        serde_json::to_string(&self).map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
    }
    /// Resolve a supported CSS method name; aliases retain exactly the same formatting.
    pub fn format(self, method: &str) -> ChartResult<String> {
        Ok(match method {
            "formatHex" | "hex" => self.format_hex(),
            "formatHex8" => self.format_hex8(),
            "formatRgb" | "toString" => self.format_rgb(),
            "formatHsl" => self.format_hsl(),
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Unknown color formatting operation.",
                ));
            }
        })
    }
    /// Checked clamp for callers that need a diagnostic for non-RGB/HSL spaces.
    pub fn checked_clamp(self) -> ChartResult<Self> {
        self.clamp().ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "Clamp is defined only for RGB and HSL color values.",
            )
        })
    }
}
