//! Standalone retained GPUI chart view over the shared core compiler/layout/inspection engine.
//! Native window/text work stays on GPUI's UI thread; no entity is allocated per mark.
mod custom;
mod native;
mod view;
pub use custom::{NativePainter, NativePainterRegistry, PreparedNativePaint};
pub use native::NativeFont;
pub use view::{
    ChartHostEvent, ChartInput, ChartView, ControlBuilder, ControlSlot, HostCommand, HostContext,
    NativeAnnotationTool, NativeDragTool, NativeMetrics, TooltipBuilder,
};
