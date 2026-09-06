//! Portable, synchronous chart foundations.
//!
//! WP-02 provides identities, structured diagnostics, finite geometry, bounded immutable
//! scenes and explicit host services. It does not yet compile chart definitions, render,
//! export or decode a wire specification. See `docs/adr/002-minimal-core-contracts.md`.
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

pub mod diagnostic;
pub mod geometry;
pub mod identity;
pub mod limits;
pub mod scene;
pub mod services;

pub use diagnostic::{ChartResult, Diagnostic, DiagnosticCode, DiagnosticContext, Severity};
pub use geometry::{Point, Rect};
pub use identity::{DatasetId, FieldId, LayerId, ResourceId, Revision, RowKey, SceneStamp};
pub use limits::Limits;
