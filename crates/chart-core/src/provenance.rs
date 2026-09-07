//! Source, aggregate and derived targets resolved only against explicit immutable snapshots.

use crate::data::{DatasetSnapshot, DatasetVersion, RowView, StoreSnapshot};
use crate::{
    AggregateId, ChartResult, DatasetId, DerivedId, Diagnostic, DiagnosticCode, Revision, RowKey,
};
use std::{collections::BTreeSet, sync::Arc};

/// A durable source identity, intentionally independent of position or data revision.
#[derive(serde::Serialize, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceRef {
    /// Dataset identity; replacing another dataset never preserves this association.
    pub dataset: DatasetId,
    /// Caller-supplied source key.
    pub key: RowKey,
}

/// Provenance distinguishes original rows from aggregate memberships and model scopes.
#[derive(serde::Serialize, Clone, Debug, Eq, PartialEq)]
pub enum Target {
    /// Source row resolved in the supplied current or explicitly historical snapshot.
    Source(SourceRef),
    /// Aggregate with explicit group/bin identity, input version and compact member keys.
    Aggregate {
        /// Aggregate identity within this input scope.
        id: AggregateId,
        /// Nonempty group/bin identity, not a claimed source row.
        group: String,
        /// Exact data/schema version used to compute the aggregate.
        input: DatasetVersion,
        /// Distinct source members; an empty aggregate is allowed.
        members: Arc<[RowKey]>,
    },
    /// Derived/model output that must never masquerade as a source row.
    Derived {
        /// Derived identity.
        id: DerivedId,
        /// Nonempty registered model/operation name.
        model: String,
        /// Model/operation definition version.
        model_version: Revision,
        /// Exact input dataset scopes, each dataset at most once.
        inputs: Vec<DatasetVersion>,
    },
}

/// Resolution result, borrowing rows from the explicitly supplied snapshot.
#[derive(Debug)]
pub enum ResolvedTarget<'a> {
    /// Existing source row.
    Source(RowView<'a>),
    /// Source absent from this snapshot; active selection can drop it without inventing a row.
    MissingSource(SourceRef),
    /// Aggregate members at the requested immutable input revision.
    Aggregate {
        /// Aggregate identity.
        id: AggregateId,
        /// Group/bin identity.
        group: &'a str,
        /// Exact source membership, not one representative source key.
        members: Vec<RowView<'a>>,
    },
    /// Derived output's declared model/input scopes.
    Derived {
        /// Derived identity.
        id: DerivedId,
        /// Model/operation name.
        model: &'a str,
        /// Model definition revision.
        model_version: Revision,
        /// Exact captured input datasets.
        inputs: Vec<&'a DatasetSnapshot>,
    },
}

impl Target {
    /// Resolve without falling back to a different revision or inventing membership.
    pub fn resolve<'a>(&'a self, snapshot: &'a StoreSnapshot) -> ChartResult<ResolvedTarget<'a>> {
        match self {
            Self::Source(source) => Ok(snapshot.dataset(source.dataset)?.row(source.key).map_or(
                ResolvedTarget::MissingSource(*source),
                ResolvedTarget::Source,
            )),
            Self::Aggregate {
                id,
                group,
                input,
                members,
            } => {
                let data = resolve_version(snapshot, *input)?;
                if group.is_empty()
                    || members.len() > data.len()
                    || members.iter().collect::<BTreeSet<_>>().len() != members.len()
                {
                    return Err(invalid(
                        "Aggregate group and distinct member scope are invalid.",
                    ));
                }
                let rows = members
                    .iter()
                    .map(|key| {
                        data.row(*key).ok_or_else(|| {
                            invalid("Aggregate member is absent at its declared input revision.")
                        })
                    })
                    .collect::<ChartResult<_>>()?;
                Ok(ResolvedTarget::Aggregate {
                    id: *id,
                    group,
                    members: rows,
                })
            }
            Self::Derived {
                id,
                model,
                model_version,
                inputs,
            } => {
                if model.is_empty()
                    || inputs.is_empty()
                    || inputs
                        .iter()
                        .map(|v| v.dataset)
                        .collect::<BTreeSet<_>>()
                        .len()
                        != inputs.len()
                {
                    return Err(invalid(
                        "A derived model requires a name and distinct nonempty input scopes.",
                    ));
                }
                let scopes = inputs
                    .iter()
                    .map(|input| resolve_version(snapshot, *input))
                    .collect::<ChartResult<_>>()?;
                Ok(ResolvedTarget::Derived {
                    id: *id,
                    model,
                    model_version: *model_version,
                    inputs: scopes,
                })
            }
        }
    }
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::Validation,
        message,
        "Supply accurate aggregate membership/model scope or recompute the derived target.",
    )
}
fn resolve_version(
    snapshot: &StoreSnapshot,
    expected: DatasetVersion,
) -> ChartResult<&DatasetSnapshot> {
    let data = snapshot.dataset(expected.dataset)?;
    if data.version() != expected {
        let mut e = Diagnostic::error(
            DiagnosticCode::RevisionConflict,
            "Target input revision differs from this snapshot.",
            "Use the explicitly retained historical snapshot or recompute this target's provenance.",
        );
        e.context.dataset = Some(expected.dataset);
        e.context.dataset_revision = Some(data.version().revision);
        e.context.schema_version = Some(data.version().schema_version);
        return Err(e);
    }
    Ok(data)
}
