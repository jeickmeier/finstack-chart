use super::{error, version};
use crate::{data::*, grammar::ChartDefinition, state::*, transaction::*, *};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Versioned normalized built-in definition, separate from data and transient state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChartEnvelope {
    /// Minimum retained capability version: legacy 1, paths 2, stages 3, floating paint 4, scales 5, chromatic 6, shapes 7.
    pub version: u32,
    /// Existing normalized built-ins; operation IDs and versions are validated by Compiler.
    pub definition: ChartDefinition,
}
impl ChartEnvelope {
    /// Check envelope version; preparation additionally validates all operation/data contracts.
    pub fn validate(&self) -> ChartResult<()> {
        if !matches!(self.version, 1..=56) || self.version != self.definition.wire_version() {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Definition envelope version does not match its capabilities.",
            ));
        }
        Ok(())
    }
}
/// Owned column input; validity is independent of the retained payload.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColumnWire {
    /// Seven explicit source kinds. Int64/UInt64/Timestamp payloads use decimal strings.
    pub values: ColumnValues,
    /// One boolean per row; false retains but does not evaluate the payload.
    pub validity: Vec<bool>,
    /// Optional exact display values, independent of binary floating-point coordinates.
    pub formatted: Option<Vec<Option<String>>>,
}
/// A schema-bearing immutable ingestion batch. Construction always goes through core validation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchWire {
    /// Exact schema version.
    pub schema_version: SchemaVersion,
    /// Named fields, ordered to match the columns.
    pub fields: Vec<Field>,
    /// Exact durable row identities as decimal strings.
    pub keys: Vec<RowKey>,
    /// Owned columns in schema order.
    pub columns: Vec<ColumnWire>,
}
impl BatchWire {
    /// Validate budgets, schema, kinds, lengths, dictionaries and identities, then own data.
    pub fn into_batch(self) -> ChartResult<NormalizedBatch> {
        self.into_batch_with_limits(DataLimits::default())
    }
    /// Materialize an already decoded batch under explicit owner-selected data budgets.
    pub fn into_batch_with_limits(self, limits: DataLimits) -> ChartResult<NormalizedBatch> {
        if self.fields.len() > limits.max_fields || self.keys.len() > limits.max_batch_rows {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Portable field/row budget exceeded",
            ));
        }
        let schema = Arc::new(Schema::new(self.schema_version, self.fields)?);
        NormalizedBatch::new(
            schema,
            self.keys,
            self.columns
                .into_iter()
                .map(|c| Column::new(c.values, c.validity, c.formatted))
                .collect(),
            limits,
        )
    }
    /// Copy immutable batch payloads into an owned wire value without losing null/display data.
    pub fn from_batch(batch: &NormalizedBatch) -> Self {
        Self {
            schema_version: batch.schema().version(),
            fields: batch.schema().fields().to_vec(),
            keys: batch.keys().to_vec(),
            columns: batch
                .columns()
                .iter()
                .map(|c| ColumnWire {
                    values: c.values().clone(),
                    validity: c.validity().to_vec(),
                    formatted: c.formatted_values().map(<[_]>::to_vec),
                })
                .collect(),
        }
    }
}
/// Named dataset registration at revision zero.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetWire {
    /// Stable identity, independent of array order.
    pub id: DatasetId,
    /// Initial immutable batch.
    pub batch: BatchWire,
}
/// Initial data envelope; later edits use TransactionEnvelope.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataEnvelope {
    /// Supported schema version.
    pub version: u32,
    /// Exact source epoch fence.
    pub epoch: SourceEpoch,
    /// Fixed registered dataset set.
    pub datasets: Vec<DatasetWire>,
}
impl DataEnvelope {
    /// Validate an interchange value and adopt its data in a typed single-writer store.
    pub fn into_store(self) -> ChartResult<DataStore> {
        version(self.version)?;
        if self.datasets.len() > DataLimits::default().max_datasets {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Dataset budget exceeded",
            ));
        }
        DataStore::new(
            self.epoch,
            self.datasets
                .into_iter()
                .map(|d| Ok((d.id, d.batch.into_batch()?)))
                .collect::<ChartResult<_>>()?,
            DataLimits::default(),
        )
    }
}
/// Versioned ordered mutation, sharing the existing transaction semantics.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MutationWire {
    /// Append only absent keys.
    AppendBatch(BatchWire),
    /// Replace whole rows or insert absent keys.
    UpsertByKey(BatchWire),
    /// Remove keys, reporting absent keys as no-ops.
    RemoveKeys(Vec<RowKey>),
    /// Replace snapshot with schema/reused-key rules unchanged.
    ReplaceSnapshot(BatchWire),
    /// Update explicit count retention.
    SetRetention(RetentionWire),
    /// Nondecreasing supplied event-time watermark, encoded as an exact decimal string.
    AdvanceWatermark(#[serde(with = "super::signed")] i64),
    /// Reset one first-seen category catalog.
    ResetCategoryOrder(FieldId),
}
impl MutationWire {
    fn into_mutation(self) -> ChartResult<Mutation> {
        Ok(match self {
            Self::AppendBatch(b) => Mutation::AppendBatch(b.into_batch()?),
            Self::UpsertByKey(b) => Mutation::UpsertByKey(b.into_batch()?),
            Self::ReplaceSnapshot(b) => Mutation::ReplaceSnapshot(b.into_batch()?),
            Self::RemoveKeys(keys) => Mutation::RemoveKeys(keys),
            Self::SetRetention(r) => Mutation::SetRetention(r.into()),
            Self::AdvanceWatermark(w) => Mutation::AdvanceWatermark(w),
            Self::ResetCategoryOrder(f) => Mutation::ResetCategoryOrder(f),
        })
    }
}
/// One dataset mutation, in transaction order.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationWire {
    /// Registered dataset identity.
    pub dataset: DatasetId,
    /// Explicit operation; unknown variants reject.
    pub mutation: MutationWire,
}
/// Versioned source transaction, separate from chart and action envelopes.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransactionEnvelope {
    /// Supported envelope version.
    pub version: u32,
    /// Opaque replay ID, bounded by the input envelope.
    pub id: String,
    /// Exact source epoch.
    pub epoch: SourceEpoch,
    /// Expected dataset and schema revisions.
    pub expected: Vec<DatasetVersion>,
    /// Ordered atomic mutations.
    pub operations: Vec<OperationWire>,
}
impl TransactionEnvelope {
    /// Validate an interchange value and produce the same typed atomic transaction.
    pub fn into_transaction(self) -> ChartResult<Transaction> {
        version(self.version)?;
        if self.operations.len() > DataLimits::default().max_operations {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Transaction operation budget exceeded",
            ));
        }
        Ok(Transaction {
            id: TransactionId::new(self.id)?,
            epoch: self.epoch,
            expected: self.expected,
            operations: self
                .operations
                .into_iter()
                .map(|o| {
                    Ok(Operation {
                        dataset: o.dataset,
                        mutation: o.mutation.into_mutation()?,
                    })
                })
                .collect::<ChartResult<_>>()?,
        })
    }
}
/// Programmatic action envelope with exact definition/state fences.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionEnvelope {
    /// Supported envelope version.
    pub version: u32,
    /// Definition fence.
    pub definition_revision: Revision,
    /// Expected state revision; stale host replies cannot overwrite newer state.
    pub expected_state: Revision,
    /// Shared action; scene-dependent variants additionally require an acknowledged scene.
    pub action: ChartAction,
}
/// Exact portable durable state snapshot, separate from the definition and transient previews.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateEnvelope {
    /// Optional committed interaction state; ephemeral pointer/gesture values are excluded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interaction: Option<crate::state::InteractionSnapshot>,
    /// Supported envelope version.
    pub version: u32,
    /// Definition fence.
    pub definition_revision: Revision,
    /// Effective state revision.
    pub state_revision: Revision,
    /// Effective viewport revision, no greater than state_revision.
    pub viewport_revision: Revision,
    /// Captured numeric/relative-timestamp visible intervals.
    pub viewport: Viewport,
    /// Hidden layer identities, distinct and belonging to this definition.
    pub hidden_layers: Vec<LayerId>,
}
impl StateEnvelope {
    pub(crate) fn capture(d: &ChartDefinition, s: &ChartState) -> Self {
        Self {
            version: super::VERSION,
            definition_revision: d.revision,
            interaction: Some(s.interaction_snapshot()),
            state_revision: s.revision(),
            viewport_revision: s.viewport_revision(),
            viewport: s.committed_viewport(),
            hidden_layers: d
                .layers
                .iter()
                .filter(|l| !s.is_visible(l.id))
                .map(|l| l.id)
                .collect(),
        }
    }
    pub(crate) fn into_state(self, d: &ChartDefinition) -> ChartResult<ChartState> {
        version(self.version)?;
        if self.definition_revision != d.revision {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "State definition revision disagrees",
            ));
        }
        let mut state = ChartState::from_portable(
            d,
            self.state_revision,
            self.viewport_revision,
            self.viewport,
            self.hidden_layers,
        )?;
        if let Some(interaction) = self.interaction {
            state.restore_interaction(d, interaction)?;
        }
        Ok(state)
    }
}

/// Portable count capacity uses a platform-independent unsigned 32-bit number.
/// Runtime dataset/byte budgets still bound retained data; zero capacity is valid.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RetentionWire {
    /// No automatic eviction within the configured data budgets.
    Unbounded,
    /// Evict oldest insertion ordinals beyond this count.
    Count(u32),
    /// Inclusive event-time horizon; all integer ticks are canonical decimal strings.
    EventTime(crate::data::EventTimeWindow),
}
impl From<RetentionWire> for RetentionPolicy {
    fn from(value: RetentionWire) -> Self {
        match value {
            RetentionWire::Unbounded => Self::Unbounded,
            RetentionWire::Count(rows) => Self::Count(rows as usize),
            RetentionWire::EventTime(window) => Self::EventTime(window),
        }
    }
}
