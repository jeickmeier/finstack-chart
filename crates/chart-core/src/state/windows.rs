use super::*;
use crate::ScaleId;
/// Named-axis presentation window. Categories and timestamps retain their semantic identities.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub enum AxisWindow {
    /// Numeric endpoints in declared calculation units (before the scale transform).
    Numeric(f64, f64),
    /// Exact source timestamp ticks; unit belongs to the axis value space.
    Timestamp(
        #[serde(with = "crate::portable::signed")] i64,
        #[serde(with = "crate::portable::signed")] i64,
    ),
    /// Inclusive stable labels in domain order; a single category is a valid window.
    Category {
        /// First included category.
        first: String,
        /// Last included category.
        last: String,
    },
}
/// Complete overrides by named scale; omitted scales retain legacy or authored view policies.
pub type AxisWindows = BTreeMap<ScaleId, AxisWindow>;
pub(crate) fn validate_windows(windows: &AxisWindows) -> ChartResult<()> {
    if windows.len() > 64 {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Navigation supports at most 64 named-axis windows.",
        ));
    }
    for window in windows.values() {
        match window {
            AxisWindow::Numeric(a, b) => Viewport {
                x: Some((*a, *b)),
                y: None,
            }
            .validate()?,
            AxisWindow::Timestamp(a, b) if a == b => {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "Time window endpoints must be distinct.",
                ));
            }
            AxisWindow::Category { first, last } if first.len() > 4096 || last.len() > 4096 => {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Category window labels exceed the text budget.",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
