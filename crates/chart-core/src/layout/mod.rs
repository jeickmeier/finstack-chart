//! Destination layout of the common prepared grammar into a finite immutable scene.
//!
//! At most four synchronized text-measurement passes solve Cartesian panels with
//! aligned margins, shared/free domains and compatible color guides. All text uses an
//! explicit destination font. No source filtering or statistics run during layout.

mod axes;
mod composition;
mod coordinates;
mod engine;
mod facet_policy;
mod facets;
mod hierarchy;
mod legend_colorbar;
mod legend_keys;
pub use hierarchy::{HierarchyHistoryEntry, HierarchySnapshot, ResolvedHierarchy};
mod guide_components;
mod guide_config;
pub use guide_components::{GuideComponents, GuideLineStyle, GuideTextStyle, GuideTickStyle};
mod guide_geometry;
pub use guide_geometry::{GuideGeometry, GuideLabelPolicy, GuideOverflow};
mod guide_animation;
mod guide_breaks;
mod guide_discrete;
mod guide_selection;
mod guide_ticks;
mod guide_transition;
mod minor_breaks;
mod temporal_minor_breaks;
pub use guide_animation::{GuidePresentationSnapshot, LayoutGuideTransition};
pub use guide_config::*;
pub use guide_transition::{
    GuideDomain, GuideTransitionFrame, GuideTransitionPlan, GuideTransitionTick,
};
mod project;
mod recipe_intervals;
mod recipe_marks;
mod recipe_surfaces;
mod row_annotation;
mod text;
mod text_marks;
mod theme;
mod types;
mod work;
pub use coordinates::*;
pub use engine::layout;
pub use types::*;

pub(crate) use axes::validate_definition_axes;

mod secondary_discrete;

mod ggplot_axis;
pub use ggplot_axis::{AxisCap, GgplotAxisOptions, LogTickOptions};

mod stroke_outline;

mod recipe_distributions;
