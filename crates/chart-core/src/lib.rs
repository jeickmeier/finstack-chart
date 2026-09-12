//! Compose immutable plots from ordinary data, then retain a [`runtime::Chart`] for live use.
//!
//! The primary [`prelude`] contains data, aesthetic and component builders. Preparation,
//! inspection and updates use one portable synchronous engine; native windows and supplied
//! font resources belong to destination crates. Export requires no GPUI event loop.
//!
//! ```
//! use chart_core::prelude::*;
//! # fn main() -> ChartResult<()> {
//! let data = Data::columns()
//!     .column("time", [1.0, 2.0, 3.0])
//!     .column("value", [10.0, 12.0, 11.0])
//!     .build()?;
//! let plot = plot(data)
//!     .aes(aes().x("time").y("value"))
//!     .layer(line().name("prices"))
//!     .layer(points().size(3.0))
//!     .title(title("Prices"))
//!     .x_axis(x_axis().label("Time"))
//!     .y_axis(y_axis().label("USD"))
//!     .build()?;
//! let mut chart = plot.chart()?;
//! assert_eq!(chart.prepare()?.layers().len(), 2);
//! let edited = plot.edit().title(title("Daily prices")).build()?;
//! chart.apply_plot(&edited, plot.definition().revision)?;
//! # Ok(())
//! # }
//! ```
//!
//! `chart-export::Output` publishes a Plot or captured Chart with supplied fonts. The
//! native `gpui-charts::ChartInput::from_plot` mounts it once. Low-level grammar, scene,
//! scale and resource modules remain available for specialist and compatibility use.

/// Bounded semantic descriptions and paged accessible data alternatives.
pub mod accessibility;
pub mod color;
pub mod composition;
pub mod data;
pub mod dense;
pub mod diagnostic;
/// Snapped and constrained annotation producers over a pinned presented basis.
pub mod editing;
pub mod geometry;
pub mod grammar;
/// Immutable bounded topology and standalone hierarchy algorithms.
pub mod hierarchy;
pub mod identity;
pub mod ingestion;
pub mod inspection;
pub mod interpolate;
pub mod layout;
pub mod limits;
/// Semantic linked-view messages and explicit matching policies.
pub mod linking;
/// Pure navigation producers using pinned presented axes and semantic windows.
pub mod navigation;
/// Portable publication composition and annotation coordinates.
mod number;
/// Checked standalone numeric paths and precision-controlled SVG path data.
pub mod path;
/// Concise primary data, aesthetic and component builders.
pub mod plot;
pub mod provenance;
/// Retained typed chart runtime shared by authoring and portable adapters.
pub mod runtime;
pub mod scales;
pub mod scene;
pub mod scheduling;
/// Checked reusable geometric generators and curve lifecycles.
pub mod shape;
/// Common authoring imports; specialist APIs remain in their family modules.
pub mod prelude {
    pub use crate::color::{ColorDescriptor, ColorValue, Paint};
    pub use crate::composition::{Collision, ConnectorOrigin, ScaleValue};
    pub use crate::data::TimeUnit;
    pub use crate::grammar::{
        BinField, ClipPolicy, EmptyPanels, FacetTarget, GroupValue, JitterUnits, LineOrder,
        NumericAesthetic, Orientation, OutlierPolicy, PanelKey, StatField, StatScope,
    };
    pub use crate::path::{Path, PathGeometry, PathLimits, PathOp, Precision, path, path_round};
    pub use crate::plot::{
        AesBuilder, AnnotationEditBuilder, AxisBuilder, AxisHandle, BinAesBuilder, CalloutBuilder,
        CaptionBuilder, ColorScaleBuilder, ColumnData, ColumnsBuilder, Data, DatasetHandle,
        DatasetRef, FacetBuilder, FieldHandle, FilterBuilder, FootnoteBuilder, GuideBuilder,
        GuideHandle, InsetBuilder, LabelsBuilder, LayerBuilder, LayerHandle, LayoutOptions,
        LegendBuilder, LinkBuilder, Mapping, NumberFormatBuilder, NumericScaleInput,
        PanelLetterBuilder, Plot, PlotBuilder, PlotEditBuilder, PlotLayer, PositionBuilder,
        Profile, RenderOptions, RichTextBuilder, RowsBuilder, ScaleBuilder, SourceNoteBuilder,
        StatAesBuilder, StatBuilder, StreamOptions, StyleBuilder, SubtitleBuilder, TextRunBuilder,
        TextStyle, ThemeBuilder, TitleBuilder, TransactionBuilder, TransformBuilder,
        TransformHandle, TransformRef, aes, after_scale_expr, annotation_edit, area, axis_guide,
        bars, bin, bin_aes, callout, caption, categorical, cells, color_continuous, color_discrete,
        color_mapped, column, count, custom_stat, dodge, facet_grid, facet_wrap, filter, fit,
        footnote, from_theme, hierarchy, hierarchy_cluster, hierarchy_icicle, hierarchy_pack,
        hierarchy_sunburst, hierarchy_tree, hierarchy_treemap, histogram, identity_stat, inset,
        jitter, labels, layout_options, legend, line, link, nullable_timestamps, number_format,
        ohlc, panel_letter, plot, points, rectangle, render_options, ribbon, rich_text, rule,
        scale_aes, scale_band, scale_band_d3, scale_binned, scale_calendar, scale_date,
        scale_duration, scale_linear, scale_log, scale_numeric, scale_point, scale_point_d3,
        scale_registered, scale_reverse, scale_session, scale_sqrt, scale_symlog, scale_utc,
        shape_arc, shape_area, shape_area_radial, shape_line, shape_line_radial, shape_link,
        shape_link_horizontal, shape_link_radial, shape_link_vertical, shape_pie, shape_stack,
        shape_symbol, source_expr, source_note, stack, stat_aes, stream_options, style, subtitle,
        summary, text_run, text_style, theme, time_value, timestamps, title, transform, volume,
        x_axis, y_axis,
    };
    pub use crate::plot::{VectorPathBuilder, vector_path};
    pub use crate::runtime::Chart;
    pub use crate::scales::{
        Baseline, ClosedSessionPolicy, OutsidePolicy, SessionCalendar, TimeBounds, UtcInterval,
    };
    pub use crate::theme::{NamedTheme, Symbol, rgb};
    pub use crate::typography::{NumberLocale, NumberNotation, TextDirection};
    pub use crate::{ChartResult, Diagnostic, DiagnosticCode};
}
pub mod services;
pub mod state;
/// Serializable headless presentation tokens and cascade.
pub mod theme;
pub mod transaction;
/// Logical rich text and explicit shaping service contracts.
pub mod typography;

pub use diagnostic::{ChartResult, Diagnostic, DiagnosticCode, DiagnosticContext, Severity};
pub use geometry::{Point, Rect};
pub use identity::{
    AggregateId, DatasetId, DerivedId, FieldId, GuideId, HierarchyId, HierarchyNodeId, LayerId,
    ResourceId, Revision, RowKey, ScaleId, SceneStamp, SchemaVersion, SourceEpoch, TransformId,
};
pub use limits::Limits;

/// Versioned portable envelopes, validation and owned runtime boundary.
pub mod portable;
