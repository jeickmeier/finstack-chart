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
/// Portable publication composition and annotation coordinates.
pub mod composition;
pub mod data;
pub mod dense;
pub mod diagnostic;
/// Snapped and constrained annotation producers over a pinned presented basis.
pub mod editing;
pub mod geometry;
pub mod grammar;
pub mod identity;
pub mod ingestion;
pub mod inspection;
pub mod layout;
pub mod limits;
/// Semantic linked-view messages and explicit matching policies.
pub mod linking;
/// Pure navigation producers using pinned presented axes and semantic windows.
pub mod navigation;
/// Concise primary data, aesthetic and component builders.
pub mod plot;
pub mod provenance;
/// Retained typed chart runtime shared by authoring and portable adapters.
pub mod runtime;
pub mod scales;
pub mod scene;
pub mod scheduling;
/// Common authoring imports; specialist APIs remain in their family modules.
pub mod prelude {
    pub use crate::composition::{Collision, ConnectorOrigin, ScaleValue};
    pub use crate::data::TimeUnit;
    pub use crate::grammar::{
        BinField, ClipPolicy, EmptyPanels, FacetTarget, GroupValue, JitterUnits, LineOrder,
        OutlierPolicy, PanelKey, StatField, StatScope,
    };
    pub use crate::plot::{
        AesBuilder, AnnotationEditBuilder, AxisBuilder, AxisHandle, BinAesBuilder, CalloutBuilder,
        CaptionBuilder, ColorScaleBuilder, ColumnData, ColumnsBuilder, Data, DatasetHandle,
        DatasetRef, FacetBuilder, FieldHandle, FilterBuilder, FootnoteBuilder, InsetBuilder,
        LabelsBuilder, LayerBuilder, LayerHandle, LayoutOptions, LegendBuilder, LinkBuilder,
        Mapping, NumberFormatBuilder, PanelLetterBuilder, Plot, PlotBuilder, PlotEditBuilder,
        PlotLayer, PositionBuilder, RenderOptions, RichTextBuilder, RowsBuilder, ScaleBuilder,
        SourceNoteBuilder, StatAesBuilder, StatBuilder, StreamOptions, StyleBuilder,
        SubtitleBuilder, TextRunBuilder, TextStyle, ThemeBuilder, TitleBuilder, TransactionBuilder,
        TransformBuilder, TransformHandle, TransformRef, aes, annotation_edit, area, bars, bin,
        bin_aes, callout, caption, categorical, cells, color_continuous, color_discrete, column,
        count, custom_stat, dodge, facet_grid, facet_wrap, filter, fit, footnote, histogram,
        identity_stat, inset, jitter, labels, layout_options, legend, line, link,
        nullable_timestamps, number_format, ohlc, panel_letter, plot, points, rectangle,
        render_options, ribbon, rich_text, rule, scale_band, scale_linear, scale_log, scale_point,
        scale_session, scale_symlog, scale_utc, source_note, stack, stat_aes, stream_options,
        style, subtitle, summary, text_run, text_style, theme, time_value, timestamps, title,
        transform, volume, x_axis, y_axis,
    };
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
    AggregateId, DatasetId, DerivedId, FieldId, LayerId, ResourceId, Revision, RowKey, ScaleId,
    SceneStamp, SchemaVersion, SourceEpoch, TransformId,
};
pub use limits::Limits;

/// Versioned portable envelopes, validation and owned runtime boundary.
pub mod portable;
