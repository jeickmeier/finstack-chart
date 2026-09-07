//! Destination layout of the common prepared grammar into a finite immutable scene.
//!
//! At most four text-measurement passes solve one Cartesian panel. Plain, unrotated
//! axes use an explicit destination font; rich shaping, legends and shared panels
//! follow in later packages. No source filtering or statistics run during layout.

mod axes;
mod engine;
mod project;
mod types;
pub use engine::layout;
pub use types::*;
