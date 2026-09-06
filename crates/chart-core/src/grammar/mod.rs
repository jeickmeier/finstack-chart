//! Typed authoring and one staged, dependency-free grammar preparation engine.
//!
//! Output geometry is in declared calculation/data space, not destination pixels.
//! Scale mapping, axes and text-aware layout are the next preparation boundary (WP-06).

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
//! # Ok(())
//! # }
//! ```

mod compiler;
mod definition;
mod prepared;
mod stats;
mod typed;

pub use compiler::Compiler;
pub use definition::*;
pub use prepared::*;
pub use typed::*;

use crate::{Diagnostic, DiagnosticCode};
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Correct the declared mapping, operation, scope or budget before preparing again.",
    )
}
