# ADR-004: Immutable data and ordered atomic transactions

Status: ACCEPTED for WP-04. Date: 6 September 2026.
Requirements: DAT-01–06, ARC-03, QLT-01 (data foundation portions).

## Decision and ownership

Keep the implementation synchronous and dependency-free in `chart-core::data`,
`transaction` and `provenance`. `DataStore` is a single writer over an immutable
`StoreSnapshot`; a `SnapshotHandle` owns an `Arc` and can be cloned or explicitly
disposed. Disposal is local, idempotent and checked on access. Other owners remain
valid. Neither a renderer nor an interpreter is involved.

`TypedRows<T>::snapshot` owns rows and unique caller keys without `Clone`, `Send` or
`Sync` requirements. Typed callers must keep logical values immutable, including through
interior mutability. `NormalizedBatch` is the fully immutable portable boundary: private
owned vectors, immutable schema and shared storage. Normalized snapshots satisfy ordinary
Rust thread-transfer bounds; no worker or mandatory threading is introduced. Typed
channel lowering and closure serialization diagnostics remain WP-05/WP-09.

Schemas have stable field IDs and explicit versions. The seven specified column kinds
preserve exact source integers/timestamps, independent validity, units/labels, and optional
original cell strings. Timestamp timezone is required metadata; core performs no timezone
database lookup. Null payloads remain stored but are not valid source values. Category
labels define identity; batch-local dictionary codes do not. Duplicate keys, fields,
names, dictionary labels, malformed lengths/types and invalid valid-row codes are errors.

## Storage and ordering

Dataset registration order and authored row order are explicit. Caller keys are `u64`
identities, independent of physical position. Each inserted key gets a checked monotonic
ordinal. Upsert preserves existing order/ordinals and replaces complete rows. Replacement
uses its supplied order, preserves ordinals for reused keys, and gives new keys fresh
ordinals. A removed then reinserted key gets a fresh ordinal. These ordinals are available
for later equal-x tie breaking; WP-04 does not sort chart lines.

Snapshots share immutable chunks (default maximum 4,096 rows). Append copies incoming
sub-batches as needed and clones existing chunk handles without copying their row payloads.
Upsert/removal copies only affected chunks. Replacement can rebuild the dataset. Category
first-seen catalogs persist across removal and schema-version changes while the categorical
field ID survives; an explicit reset rebuilds from retained authored rows. An explicit
category domain overrides display order without filtering the data.

This is a storage-sharing contract, not an incremental-time or RSS guarantee. Key lookup,
validation and comparison currently scan rows/chunks; category catalog growth copies its
bounded label metadata. Repeated small batches can produce small chunks. Compaction,
indexing, scheduling and sustained-load/memory plateau evidence belong to WP-18/19/22.

## Transaction and revision rules

Transactions carry a bounded opaque ID, source epoch, one exact dataset/schema base for
each touched dataset, and ordered operations. Bases are checked once against the initial
snapshot. Operations stage in order on private candidate state; any failure discards the
entire candidate. The store publishes once after validation, revision advancement and
replay-budget checks. Public outcomes are applied, already applied, rejected and conflict.
There is no queue: applied means committed. WP-18 will own queued ingestion acknowledgements.

Append rejects existing or within-batch duplicate keys. Upsert accepts complete rows only.
Repeated removal keys collapse to one request; absent keys are counted no-ops. Changed
replacement schemas require a strictly newer version; append/upsert require the exact
current schema, including metadata. A replacement and subsequent operations can use the
new schema in the same transaction while the transaction header still fences the old base.

Each effectively changed dataset advances once, and the coherent store revision advances
once. Logical no-ops preserve revisions, including A-to-B-to-A row edits, identical upserts
and dictionary recoding. Effective state includes schema, order, ordinal counter, category
history and retention policy: append-then-remove can therefore change revision even if no
row survives. Masked null payload differences have no logical effect; original formatting
and exact float bits do. Counter exhaustion rejects atomically, without wrapping.

Receipts report each operation's executed inserted/updated/removed/absent/evicted counts;
these are not net counts. Final versions and previously retained source keys absent at
commit are separate. Dataset receipt order is ID order; store registration order is unchanged.
Count retention is enforced after each operation and evicts the oldest insertion ordinals,
including after reorder/replacement. Retained row/byte limits apply after retention at each
operation boundary. Explicit event-time watermarks/lateness/window retention remain WP-18.

## Replay horizon and resource limits

The cache is FIFO and bounded by entries and charged payload/receipt bytes (defaults 128
and 64 MiB). Retries do not refresh it. It retains the complete transaction representation:
identical ID, epoch, bases and payload return the original receipt before stale-base checks;
reusing a remembered ID with different content is an error. Float payload equality uses
bits, including NaNs. Rejected transactions consume no ID or cache entry. A transaction
that cannot fit alone is rejected before publication or eviction of previous entries.

After eviction, stale bases reject a replay of an effective commit. A previously pure no-op
may be evaluated again if its bases are still current; outside the bounded horizon no ID
recognition is promised, and it remains a no-op. Sources must resynchronize on conflicts.
`dedup_horizon` exposes count/byte capacities and oldest/newest IDs. `reset_epoch` requires
a strictly greater epoch and clears history while preserving data/revisions. Old epoch
transactions always conflict, even if their IDs were previously remembered.

`DataLimits` also bounds dataset/field/batch counts, retained rows, batch/transaction/dataset
payload charges and remembered categories. Charges are conservative logical accounting,
not measured allocator capacity/RSS. Caller input allocation and separately held historical
handles are outside the store budget. Owners must release handles; the core does not retain
an unbounded history. Process-wide allocation failure is not a recoverable quota mechanism.

## Numeric validity and provenance

Numeric projection preserves source values and returns finite `f64` values or explicit
gaps. It counts null, non-finite and precision exclusions in one diagnostic with at most
32 sampled keys; strict mode errors. Integer magnitudes above 2^53 are conservatively
excluded. Timestamps subtract a checked integer origin before conversion and retain source
units. Later scales/geometry share the declared transformation; line gap behavior and
native pixel conversion are not implemented here.

Targets distinguish source keys, aggregates with group/revision/member keys, and derived
models with versioned input scopes. Resolution uses the supplied immutable snapshot;
stale aggregate/derived scopes fail rather than silently resolving a different revision.
Missing source keys return an explicit missing-source result. Pinned historical handles
continue to resolve old rows. Receipts expose final removed sources for the future action
reducer; active selection events and historical tooltip labels remain WP-15/16/18.

## Evidence and next consumers

[WP-04 evidence](../evidence/wp-04-completion-2026-09-06.md) covers independent results,
atomic failure, ownership release, exact IDs/time, replay limits and chunk sharing.
WP-05 consumes normalized data and provenance for grammar preparation. Python/WASM wire
encoding and runtime lifetimes remain WP-09; WASM compilation is not binding execution.
No full canonical fixture, PERF case or G1–G4 gate is closed by this package.
