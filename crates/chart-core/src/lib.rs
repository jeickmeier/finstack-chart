//! Portable, synchronous chart foundations.
//!
//! WP-02 provides identities, structured diagnostics, finite geometry, bounded immutable
//! scenes and explicit host services. WP-04 adds immutable typed/column snapshots, atomic
//! data transactions and provenance. WP-05 adds typed authoring and grammar preparation
//! into data-space geometry. Scale/layout resolution, rendering, export and wire decoding
//! remain later boundaries. See ADR-002 and ADR-004 in `docs/adr/`.
//!
//! A minimal destination scene can be constructed without any host runtime:
//!
//! ```
//! use chart_core::{ChartResult, Limits, Rect, SceneStamp};
//! use chart_core::scene::{Color, Primitive, Scene, SceneItem};
//! use chart_core::services::Units;
//!
//! # fn main() -> ChartResult<()> {
//! let bounds = Rect::new(0.0, 0.0, 100.0, 80.0)?;
//! let item = SceneItem {
//!     layer: None,
//!     clip: None, // Default clip is the scene bounds.
//!     primitive: Primitive::Rectangle {
//!         bounds: Rect::new(10.0, 20.0, 30.0, 40.0)?,
//!         fill: Color { red: 20, green: 40, blue: 60, alpha: 255 },
//!     },
//! };
//! let scene = Scene::new(
//!     SceneStamp::default(), Units::Points, bounds, &[item], &[], Limits::default(),
//! )?;
//! assert_eq!(scene.items().len(), 1);
//! assert!(Rect::new(0.0, 0.0, f64::INFINITY, 1.0).is_err());
//! # Ok(())
//! # }
//! ```

/// Bounded semantic descriptions and paged accessible data alternatives.
pub mod accessibility;
/// Portable publication composition and annotation coordinates.
pub mod composition;
pub mod data;
pub mod diagnostic;
/// Snapped and constrained annotation producers over a pinned presented basis.
pub mod editing;
pub mod geometry;
pub mod grammar;
pub mod identity;
pub mod inspection;
pub mod layout;
pub mod limits;
/// Semantic linked-view messages and explicit matching policies.
pub mod linking;
/// Pure navigation producers using pinned presented axes and semantic windows.
pub mod navigation;
pub mod provenance;
pub mod scales;
pub mod scene;
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
