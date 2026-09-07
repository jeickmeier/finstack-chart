//! Standalone retained GPUI chart view over the shared core compiler/layout/inspection engine.
//! Native window/text work stays on GPUI's UI thread; no entity is allocated per mark.
mod native;
mod view;
pub use native::NativeFont;
pub use view::{ChartInput, ChartView, NativeMetrics, TooltipBuilder};
