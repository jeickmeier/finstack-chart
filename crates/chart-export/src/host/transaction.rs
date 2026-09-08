use super::*;
use chart_core::{
    plot::{Data, DatasetRef, TransactionBuilder},
    transaction::Transaction,
};
/// A host-owned transaction builder with bases captured once at creation.
#[derive(Clone)]
pub struct Updates(TransactionBuilder);
impl Runtime {
    /// Begin atomic ordered updates against the current core source fences.
    pub fn transaction(&self) -> ChartResult<Updates> {
        Ok(Updates(self.chart()?.transaction()?))
    }
    /// Commit one immutable transaction, preserving applied/replay/rejected/conflict outcomes.
    pub fn commit(&mut self, transaction: &Transaction) -> ChartResult<String> {
        portable::encode(
            &self
                .get_mut()?
                .runtime_mut()
                .apply_transaction(transaction.clone())?,
        )
    }
    /// Accept one immutable transaction for later explicit drain.
    pub fn enqueue(&mut self, transaction: &Transaction) -> ChartResult<String> {
        portable::encode(&self.get_mut()?.runtime_mut().enqueue(transaction.clone())?)
    }
}
impl Updates {
    /// Supply a stable retry key without changing expected source bases.
    pub fn id(&self, id: &str) -> Self {
        Self(self.0.clone().id(id))
    }
    /// Append, upsert or replace an owned normalized batch in one named/handled dataset.
    pub fn data(&self, operation: &str, target: DatasetRef, data: &Data) -> ChartResult<Self> {
        let b = self.0.clone();
        Ok(Self(match operation {
            "append" => b.append(target, data),
            "upsert" => b.upsert(target, data),
            "replace" => b.replace(target, data),
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Data mutation must be append, upsert or replace.",
                ));
            }
        }))
    }
    /// Remove exact durable keys, without numerical conversion or positional identity.
    pub fn remove(&self, target: DatasetRef, keys: Vec<u64>) -> Self {
        Self(self.0.clone().remove(target, keys))
    }
    /// Configure count retention, including explicit unbounded None.
    pub fn retain_count(&self, target: DatasetRef, count: Option<usize>) -> Self {
        Self(self.0.clone().retain_count(target, count))
    }
    /// Configure the shared typed event-time/count retention protocol.
    pub fn retention(&self, target: DatasetRef, input: &str) -> ChartResult<Self> {
        Ok(Self(self.0.clone().retention(
            target,
            portable::decode::<portable::RetentionWire>(input)?.into(),
        )))
    }
    /// Configure event time through the authored field name and exact integer options.
    pub fn retain_event_time(
        &self,
        target: DatasetRef,
        field: &str,
        input: &str,
    ) -> ChartResult<Self> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Event {
            width: String,
            watermark: String,
            allowed_lateness: String,
            late: chart_core::data::LateDataPolicy,
        }
        let e: Event = portable::decode(input)?;
        let ticks = |s: String| {
            s.parse::<i64>().map_err(|_| {
                error(
                    DiagnosticCode::Validation,
                    "Event time requires signed 64-bit ticks.",
                )
            })
        };
        Ok(Self(self.0.clone().retain_event_time(
            target,
            field,
            ticks(e.width)?,
            ticks(e.watermark)?,
            ticks(e.allowed_lateness)?,
            e.late,
        )))
    }
    /// Advance supplied event time in exact ticks.
    pub fn watermark(&self, target: DatasetRef, ticks: i64) -> Self {
        Self(self.0.clone().watermark(target, ticks))
    }
    /// Rebuild one exact first-seen category catalog from retained rows.
    pub fn reset_categories(&self, target: DatasetRef, field: &str) -> Self {
        Self(self.0.clone().reset_categories(target, field))
    }
    /// Materialize the canonical transaction without committing or retrying it.
    pub fn build(&self) -> ChartResult<Transaction> {
        self.0.clone().build()
    }
}
