//! Explicit saving policy; no ambient current device, figure, directory or display size.
use crate::{ExportOptions, FigureSnapshot, PageSize, error};
use chart_core::{ChartResult, DiagnosticCode};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

/// Units accepted by the reference saving contract.
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum SaveUnits {
    /// Inches.
    #[default]
    #[serde(rename = "in")]
    Inches,
    /// Centimeters.
    #[serde(rename = "cm")]
    Centimeters,
    /// Millimeters.
    #[serde(rename = "mm")]
    Millimeters,
    /// Pixels at the explicitly selected DPI.
    #[serde(rename = "px")]
    Pixels,
}
/// Numeric DPI or a named reference density.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum SaveDpi {
    /// Positive integral raster density.
    Number(u32),
    /// screen=72, print=300, retina=320.
    Named(String),
}
impl Default for SaveDpi {
    fn default() -> Self {
        Self::Number(300)
    }
}
impl SaveDpi {
    /// Resolve a named or numeric density without a display query.
    pub fn resolve(&self) -> ChartResult<u32> {
        match self {
            Self::Number(n) if *n > 0 => Ok(*n),
            Self::Named(n) => match n.as_str() {
                "screen" => Ok(72),
                "print" => Ok(300),
                "retina" => Ok(320),
                _ => Err(invalid("Unknown DPI name.")),
            },
            _ => Err(invalid("DPI must be positive.")),
        }
    }
}
/// Portable size/device/path policy. Missing dimensions require an explicit current page.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SaveOptions {
    /// Width in selected units; absence uses the explicitly supplied current page.
    pub width: Option<f64>,
    /// Height in selected units; absence uses the explicitly supplied current page.
    pub height: Option<f64>,
    /// Physical or pixel units.
    pub units: SaveUnits,
    /// Positive multiplier applied to both physical dimensions.
    pub scale: f64,
    /// Density used for pixels and raster metadata.
    pub dpi: SaveDpi,
    /// Reject either scaled dimension at or above fifty inches.
    pub limitsize: bool,
    /// Explicitly allow parent creation at the final host write.
    pub create_dir: bool,
    /// Device name overrides filename inference; custom devices use exact registered names.
    pub device: Option<String>,
}
impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            units: SaveUnits::Inches,
            scale: 1.,
            dpi: SaveDpi::default(),
            limitsize: true,
            create_dir: false,
            device: None,
        }
    }
}
fn invalid(message: &str) -> chart_core::Diagnostic {
    error(DiagnosticCode::Validation, message)
}
/// Resolved immutable policy. Resolution performs no filesystem I/O.
#[derive(Clone, Debug, serde::Serialize)]
pub struct SavePlan {
    /// Explicit host path after page-number substitution.
    pub path: PathBuf,
    /// Canonical selected device name or exact custom name.
    pub device: String,
    /// Resolved physical dimensions.
    pub page: PageSize,
    /// Resolved density.
    pub dpi: u32,
    /// Explicit parent creation policy.
    pub create_dir: bool,
}
impl SaveOptions {
    /// Resolve one output filename and dimensions. Numbered names support %d and %0Nd;
    /// %% is a literal percent. Page numbering starts at one.
    pub fn resolve(
        &self,
        path: impl AsRef<Path>,
        current: Option<PageSize>,
        page_number: u32,
    ) -> ChartResult<SavePlan> {
        if !self.scale.is_finite() || self.scale <= 0. {
            return Err(invalid("Save scale must be finite and positive."));
        }
        let dpi = self.dpi.resolve()?;
        let factor = match self.units {
            SaveUnits::Inches => 72.,
            SaveUnits::Centimeters => 72. / 2.54,
            SaveUnits::Millimeters => 72. / 25.4,
            SaveUnits::Pixels => 72. / f64::from(dpi),
        };
        let dimension = |value: Option<f64>, fallback: Option<f64>| -> ChartResult<f64> {
            let value = match value {
                Some(v) => v * factor,
                None => fallback.ok_or_else(|| {
                    invalid("Missing dimensions require an explicit current page.")
                })?,
            };
            if !value.is_finite() || value <= 0. {
                return Err(invalid("Save dimensions must be finite and positive."));
            }
            Ok(value * self.scale)
        };
        let page = PageSize::points(
            dimension(self.width, current.map(PageSize::width))?,
            dimension(self.height, current.map(PageSize::height))?,
        )?;
        if self.limitsize && (page.width() >= 3600. || page.height() >= 3600.) {
            return Err(invalid(
                "Save dimensions must be less than fifty inches when limitsize is enabled.",
            ));
        }
        let path = numbered_path(path.as_ref(), page_number)?;
        let selected = self
            .device
            .clone()
            .or_else(|| {
                path.extension()
                    .and_then(|s| s.to_str())
                    .map(str::to_lowercase)
            })
            .ok_or_else(|| invalid("Supply a device or a filename extension."))?;
        if selected.is_empty() {
            return Err(invalid("Device name must not be empty."));
        }
        let device = crate::host::format(&selected).map_or(selected, |f| f.name().to_owned());
        Ok(SavePlan {
            path,
            device,
            page,
            dpi,
            create_dir: self.create_dir,
        })
    }
}
fn numbered_path(path: &Path, page: u32) -> ChartResult<PathBuf> {
    if page == 0 {
        return Err(invalid("Output page numbers start at one."));
    }
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return Err(invalid("A UTF-8 output filename is required."));
    };
    let mut output = String::new();
    let mut chars = name.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            output.push(c);
            continue;
        }
        if chars.peek() == Some(&'%') {
            chars.next();
            output.push('%');
            continue;
        }
        let mut digits = String::new();
        while chars.peek().is_some_and(char::is_ascii_digit) {
            digits.push(chars.next().expect("peeked"));
        }
        if chars.next() != Some('d') {
            return Err(invalid("Filename placeholders must be %d, %0Nd or %% ."));
        }
        let width = if digits.is_empty() {
            0
        } else {
            if !digits.starts_with('0') {
                return Err(invalid("Padded page placeholders require a leading zero."));
            }
            digits
                .parse::<usize>()
                .map_err(|_| invalid("Invalid page padding."))?
        };
        if width > 32 {
            return Err(invalid("Page padding exceeds thirty-two digits."));
        }
        output.push_str(&format!("{page:0width$}"));
    }
    Ok(path.with_file_name(output))
}
/// Custom device hook consumes the same immutable captured figure as built-in devices.
pub trait CustomDevice: Send + Sync + std::fmt::Debug {
    /// Unique exact device name, independent of filesystem suffixes.
    fn name(&self) -> &str;
    /// Encode synchronously with an explicit output byte budget. Errors are preserved.
    fn encode(&self, figure: &FigureSnapshot, max_bytes: usize) -> ChartResult<Vec<u8>>;
}
/// Bounded explicit registry; cloning pins installed callbacks for independent owners.
#[derive(Clone, Debug, Default)]
pub struct DeviceRegistry {
    devices: BTreeMap<String, Arc<dyn CustomDevice>>,
}
impl DeviceRegistry {
    /// Register a unique device; built-in names cannot be replaced.
    pub fn register(&mut self, device: Arc<dyn CustomDevice>) -> ChartResult<()> {
        let name = device.name();
        if name.is_empty()
            || name.len() > 128
            || crate::host::format(name).is_ok()
            || self.devices.contains_key(name)
        {
            return Err(invalid(
                "Custom device name is empty, reserved or already registered.",
            ));
        }
        if self.devices.len() >= 64 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Custom device registry is full.",
            ));
        }
        self.devices.insert(name.to_owned(), device);
        Ok(())
    }
}
impl SavePlan {
    /// Apply resolved dimensions/density while preserving caller background, text and capture policy.
    pub fn options(&self, options: ExportOptions) -> ExportOptions {
        options.page(self.page).dpi(self.dpi)
    }
    /// Encode one already captured figure. No path is touched until write is called.
    pub fn encode(
        &self,
        figure: &FigureSnapshot,
        registry: &DeviceRegistry,
        max_bytes: usize,
    ) -> ChartResult<Vec<u8>> {
        let profile = &figure.metadata().profile;
        if profile.page != self.page || profile.dpi != self.dpi {
            return Err(invalid(
                "Captured dimensions and DPI differ from the resolved save plan.",
            ));
        }
        let max_bytes = max_bytes.min(profile.max_output_bytes);
        let bytes = if let Some(device) = registry.devices.get(&self.device) {
            device.encode(figure, max_bytes)?
        } else {
            figure.export(crate::host::format(&self.device)?)?.bytes
        };
        if bytes.len() > max_bytes {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Saved output exceeds its byte budget.",
            ));
        }
        Ok(bytes)
    }
    /// Perform the explicitly requested host write, preserving the original I/O error.
    pub fn write(&self, bytes: &[u8]) -> std::io::Result<()> {
        if self.create_dir
            && let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, bytes)
    }
}

/// Shared host adapter for save policy; dimensions of `current` are explicit points.
#[doc(hidden)]
pub fn resolve_save_json(
    filename: &str,
    options: &str,
    current: Option<Vec<f64>>,
    page_number: u32,
) -> ChartResult<String> {
    let current = match current {
        None => None,
        Some(v) => {
            let [width, height] = v.as_slice() else {
                return Err(invalid("Current page requires exactly two dimensions."));
            };
            Some(PageSize::points(*width, *height)?)
        }
    };
    let options: SaveOptions = chart_core::portable::decode(options)?;
    chart_core::portable::encode(&options.resolve(filename, current, page_number)?)
}

/// Bounded immutable-snapshot collection for explicitly supported multi-page devices.
#[derive(Clone)]
pub struct FigurePages {
    pages: Vec<FigureSnapshot>,
}
impl FigurePages {
    /// Start with one captured page; the source figure may subsequently be disposed.
    pub fn new(first: FigureSnapshot) -> Self {
        Self { pages: vec![first] }
    }
    /// Retain one more immutable page, rejecting before exceeding the1024-page budget.
    pub fn push(&mut self, page: FigureSnapshot) -> ChartResult<()> {
        if self.pages.len() >= 1024 {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Figure page budget exceeded.",
            ));
        }
        self.pages.push(page);
        Ok(())
    }
    /// Encode using the canonical multi-page device owner.
    pub fn export(&self, format: crate::Format) -> ChartResult<Vec<u8>> {
        FigureSnapshot::export_pages(&self.pages, format)
    }
}
