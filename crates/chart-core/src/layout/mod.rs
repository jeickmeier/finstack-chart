//! Destination layout of the common prepared grammar into a finite immutable scene.
//!
//! At most four synchronized text-measurement passes solve Cartesian panels with
//! aligned margins, shared/free domains and compatible color guides. All text uses an
//! explicit destination font. No source filtering or statistics run during layout.

mod axes;
mod composition;
mod coordinates;
mod engine;
mod facets;
mod guide_ticks;
mod project;
mod text;
mod theme;
mod types;
mod work;
pub use coordinates::*;
pub use engine::layout;
pub use types::*;

pub(crate) use axes::validate_definition_axes;
