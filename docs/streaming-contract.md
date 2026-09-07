# Streaming retention and incremental computation

WP-18 implements DAT-03/04/06, STM-01/02/03 and the streaming portion of GRA-08.
The [transaction contract](adr/004-immutable-data-and-transactions.md) and
[action contract](adr/007-actions-gestures-and-controlled-state.md) remain authoritative.

## Acceptance and retention

`IngestionQueue` is an explicitly drained, bounded FIFO. `Queued` means accepted into
memory, not validated or committed. `AlreadyQueued` consumes no additional capacity;
reusing a queued ID with different content fails. Positive transaction, row-operation
and conservatively charged byte capacities apply simultaneously. `Backpressure` leaves
the producer responsible for retry. Opt-in `DropNewest` counts lost transactions and
row operations; it never discards previously accepted work. Drain reports the existing
atomic `CommitOutcome`; conflicts and validation failures are accounted separately.
This is an in-process queue, with no durable log, worker or implicit retry.

Count retention keeps the most recently inserted observations by stable insertion
ordinal. Reordering presentation does not change age; correcting a key preserves its
ordinal. Event-time retention declares a timestamp field, positive width, nonnegative
lateness and an application-supplied watermark. The inclusive lower boundary is
`watermark - width - allowed_lateness`, computed in i128 source ticks. Future rows never
advance the watermark. A watermark cannot regress within the same active policy.
All retained timestamps must be valid; null timestamps are rejected. Incoming rows
before the boundary reject the transaction by default. Explicit late-drop mode reports
the discarded count. Dataset row limits still bound future-dated populations.

Retention, corrections and policy changes commit atomically. Receipts distinguish input,
retained, evicted and late-dropped observations. Failed transactions preserve prior
snapshots and revisions. Immutable chunks remain valid for old presented/export/pinned
snapshots; evicting a live key does not rewrite those historical snapshots.

## Exact updates and declared fallbacks

Explicit-edge bins over an unfiltered complete source dataset cache exact per-chunk
classification and reversible counts/membership. Append, correction, removal and
retention reclassify changed chunks; retained chunks avoid rereading statistic values.
Operation identity, dataset/schema, parameters and compile limits fence reuse. Cache
contributions across operations are bounded by the prepared-row budget. Errors clear
partial compiler cache state. Unused entries are pruned after preparation.

Auto bins, filtered/subset/facet inputs, transform inputs and other built-in statistics
use their declared full-batch paths. There are no approximate quantiles or aggregates.
Operation metadata exposes this capability distinction. A fresh compiler produces the
same full tables, target membership, revisions and domains as a warm compiler.

Immutable membership publication, source lookup and geometry/domain work can still
scan the retained population. `StatUpdateMetrics` measures bin classification work;
it does not establish O(delta) overall preparation or resident-memory bounds.
The [focused benchmark](../crates/chart-core/examples/streaming_benchmark.rs) compares
warm and fresh complete preparations including membership publication, excluding
transaction ingestion, native paint and process RSS. Full PERF gates remain WP-22.

## Follow, reconciliation and historical pins

After a committed source change, `ActionReducer::reconcile_prepared` removes absent
active targets, cancels gestures whose source targets disappeared and updates aggregate
input identities. It retains compatible selections, annotations and hidden layers.
Removed selections are observable in `Reconciliation`.

Follow-latest aligns each explicit horizontal window's high endpoint to the latest
retained prepared extent while preserving width and direction. Typed timestamp windows
use integer arithmetic. Automatic windows remain automatic; vertical windows are
preserved. Manual navigation enters inspect-history, which preserves the supplied
viewport as new data arrive. Freeze retains the coherent presented scene while source
commits continue. Explicit resume releases freeze and allows latest presentation.

Pins retain at most one original presented scene. Descriptions explicitly mark that
scene historical when it differs from current presentation and preserve exact old
values. Unpin/disposal releases this ownership; other independently retained snapshots
may still own those chunks. The native adapter emits reconciliation events and uses
the same reducer and compiler as portable callers.

The versioned portable `stream` request supports `ConfigureQueue`, `Enqueue`,
`CommitNext`, `Status` and `Pinned`. Queue reconfiguration requires an empty queue.
Rust, Python and WASM execute the same core protocol; integer identities, counters and
event timestamps use canonical decimal strings. See the
[replay generator](../fixtures/streaming/generate.py) and
[completion evidence](evidence/wp-18-completion-2026-09-07.md).
