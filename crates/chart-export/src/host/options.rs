use super::*;
/// Actual reusable publication options behind Python/WASM fluent syntax.
#[derive(Clone)]
pub struct Options(pub(crate) crate::ExportOptions);
fn page(width: f64, height: f64, unit: &str) -> ChartResult<crate::PageSize> {
    match unit {
        "pt" => crate::PageSize::points(width, height),
        "mm" => crate::PageSize::millimeters(width, height),
        _ => Err(error(
            DiagnosticCode::Validation,
            "Page units must be pt or mm.",
        )),
    }
}
impl Options {
    /// Acquire immutable static inputs using these canonical options.
    pub fn request(&self, output: &Output, plot: &Plot) -> ChartResult<FigureRequest> {
        output.request(plot, self.0.clone())
    }
    /// Start the canonical publication defaults with explicit physical dimensions.
    pub fn new(width: f64, height: f64, unit: &str) -> ChartResult<Self> {
        Ok(Self(crate::export_options(page(width, height, unit)?)))
    }
    /// Apply a scalar option through ExportOptions; invalid fields/families reject.
    pub fn set(&self, name: &str, input: &str) -> ChartResult<Self> {
        let a: Vec<Value> = portable::decode(input)?;
        if name == "page" {
            if a.len() != 3 {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Page requires width, height and unit.",
                ));
            }
            return Ok(Self(self.0.clone().page(page(
                value(&a[0])?,
                value(&a[1])?,
                &value::<String>(&a[2])?,
            )?)));
        }
        let [v] = a.as_slice() else {
            return Err(error(
                DiagnosticCode::Validation,
                "Publication option requires one value.",
            ));
        };
        let b = self.0.clone();
        Ok(Self(match name {
            "dpi" => b.dpi(value(v)?),
            "raster_device" => b.raster_device(value(v)?),
            "vector_device" => b.vector_device(value(v)?),
            "precision" => b.precision(value(v)?),
            "max_raster_pixels" => b.max_raster_pixels(exact_u64(v)?),
            "max_output_bytes" => b.max_output_bytes(value(v)?),
            "background" => b.background(chart_core::plot::host::color(v)?),
            "interaction" => b.interaction(value(v)?),
            "basis" => b.basis(value(v)?),
            "view" => b.view(value(v)?),
            "text" => b.text(value(v)?),
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    format!("Unknown publication option '{name}'."),
                ));
            }
        }))
    }
    /// Compose the same destination layout builder used by native Rust.
    pub fn layout(&self, component: &chart_core::plot::host::Component) -> ChartResult<Self> {
        Ok(Self(self.0.clone().layout(component.layout_options()?)))
    }
}
