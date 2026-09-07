//! Typed authoring and one staged, dependency-free grammar preparation engine.
//!
//! Output geometry is in declared calculation/data space, not destination pixels.
//! [`crate::layout`] maps this output into destination geometry with axes and text-aware margins.

//! Typed native inputs lower to the same explicit bin/rectangle recipe as field-based layers:
//!
//! ```
//! use chart_core::{ChartResult, DatasetId, FieldId, LayerId, Revision, RowKey,
//!                  SchemaVersion, SourceEpoch};
//! use chart_core::data::{DataLimits, TypedRows};
//! use chart_core::grammar::{BinSpec, ChartDefinition, CompileLimits, Compiler, Layer,
//!                           PreparedRows, TypedDataBuilder};
//! use chart_core::state::ChartState;
//! use chart_core::transaction::DataStore;
//!
//! # fn main() -> ChartResult<()> {
//! let data = DatasetId::new(1);
//! let x = FieldId::new(1);
//! let rows = TypedRows::snapshot(data, Revision::INITIAL,
//!     (1..=4).map(RowKey::new).collect(), vec![0.0, 0.5, 1.0, 2.0], 4)?;
//! let batch = TypedDataBuilder::new(rows.get()?, SchemaVersion::new(1))
//!     .float(x, "value", |value| Some(*value)).finish(DataLimits::default())?;
//! let store = DataStore::new(SourceEpoch::new(1), vec![(data, batch)], DataLimits::default())?;
//! let definition = ChartDefinition::new(Revision::new(1)).layer(
//!     Layer::histogram(LayerId::new(1), data, BinSpec::new(x, vec![0.0, 1.0, 2.0])));
//! let prepared = Compiler::new().prepare(&definition, &store.snapshot(),
//!     &ChartState::default(), CompileLimits::default())?;
//! let PreparedRows::Binned(bins) = prepared.layers()[0].table().rows() else {
//!     panic!("histogram has a generated bin schema");
//! };
//! assert_eq!(bins.iter().map(|bin| bin.count).collect::<Vec<_>>(), vec![2, 2]);
//! assert_eq!(prepared.layers()[0].marks().len(), 2);
//!
//! // Resolve using the destination's exact font and measurement service.
//! use chart_core::layout::{layout, LayoutRequest, LayoutStatus};
//! use chart_core::services::{ResourceDescriptor, ResourceKind, TextMeasurer,
//!                            TextMetrics, TextRequest, Units};
//! struct FixtureMetrics;
//! impl TextMeasurer for FixtureMetrics {
//!     fn measure(&self, text: TextRequest<'_>) -> ChartResult<TextMetrics> {
//!         // Demonstration only: real hosts must shape/paint this exact font and units.
//!         TextMetrics::new(text.text.chars().count() as f64 * 6., 9., 3.)
//!     }
//! }
//! let font = ResourceDescriptor { id: chart_core::ResourceId::new(1),
//!     revision: Revision::new(1), kind: ResourceKind::Font, byte_len: 123 };
//! let request = LayoutRequest::new(chart_core::Rect::new(0., 0., 400., 240.)?,
//!                                  Units::Points, font);
//! let result = layout(std::sync::Arc::new(prepared), &request, &FixtureMetrics)?;
//! assert_eq!(result.status(), LayoutStatus::Ready);
//! assert_eq!(result.targets().len(), result.scene().items().len());
//! # Ok(())
//! # }
//! ```

mod colors;
mod compiler;
mod definition;
mod extensions;
mod facets;
mod geometry_extensions;
pub(crate) mod positions;
mod prepared;
mod statistical_types;
mod statistics;
mod stats;
mod typed;

pub use colors::*;
pub use compiler::Compiler;
pub use definition::*;
pub use extensions::*;
pub use facets::*;
pub use geometry_extensions::*;
pub use prepared::*;
pub use statistical_types::*;
pub use typed::*;

use crate::{Diagnostic, DiagnosticCode};
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Correct the declared mapping, operation, scope or budget before preparing again.",
    )
}
