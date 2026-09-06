//! Opaque identities and revisions. Values retain all 64 bits, including zero.

use crate::{ChartResult, Diagnostic, DiagnosticCode};

macro_rules! identity {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            /// Wrap a caller-assigned identity without narrowing or reinterpretation.
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Return the exact original identity.
            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

identity!(
    DatasetId,
    "Identity of a dataset, independent of its position or revision."
);
identity!(
    LayerId,
    "Identity of a definition layer, independent of paint order."
);
identity!(FieldId, "Identity of a field within its dataset schema.");
identity!(
    ResourceId,
    "Identity of a host-supplied resource, independent of its revision."
);
identity!(
    RowKey,
    "Stable source-row identity within a dataset; never a positional index."
);

/// A revision within one explicitly identified owner/epoch, not a global clock.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Revision(u64);

impl Revision {
    /// Initial revision before any effective change.
    pub const INITIAL: Self = Self(0);

    /// Wrap an existing revision without narrowing it.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Return the exact revision counter.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Advance without wrapping; the owner remains responsible for effective-change rules.
    pub fn checked_next(self) -> ChartResult<Self> {
        self.0.checked_add(1).map(Self).ok_or_else(|| {
            Diagnostic::error(
                DiagnosticCode::RevisionOverflow,
                "The revision counter is exhausted.",
                "Start a new owner/epoch and resynchronize dependent snapshots.",
            )
        })
    }
}

/// Compilation inputs identifying a scene. Comparing stamps does not schedule or present it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SceneStamp {
    /// Definition revision.
    pub definition: Revision,
    /// Coherent store commit revision; dataset-specific revisions belong to data snapshots.
    pub store: Revision,
    /// Layout/resource/profile revision supplied by the owner.
    pub layout: Revision,
    /// Viewport revision.
    pub viewport: Revision,
}
