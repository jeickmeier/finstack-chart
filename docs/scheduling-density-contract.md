# Bounded preparation and destination density

WP-19 implements STM-04/05, SCN-04 and GPU-02. See the
[completion evidence](evidence/wp-19-completion-2026-09-07.md) and
[benchmark protocol](adr/008-benchmark-protocol.md).

## Preparation ownership and admission

`PreparationScheduler<T>` is synchronous core policy: one executor-owned active input
and one newest pending input. Submission replaces only pending preparation. The source
transaction is already committed and is never dropped by this scheduler. Hosts own
executors and threading; core introduces neither mandatory threads nor a global pool.

Job tokens capture the source epoch/store revision and definition, viewport, layout,
resource and presentation compatibility generations. An active compatible result may
be older than the newest committed data: admitting it prevents refresh starvation.
Within an epoch, data submission and presentation never regress. Incompatible, unknown,
duplicate and disposed completions cannot release another job or overwrite a newer
admitted scene. Completion admission and actual paint acknowledgment are separate.

Native `queue_data` captures immutable inputs and uses GPUI background execution for
numeric preparation, with at most one worker and one pending request per chart. The
compiler cache returns with its worker. Current selection/hover and source reconciliation
are applied before installation. Presentation-affecting host operations invalidate and
resubmit the latest source. Destination layout/path conversion/index construction still
run on the UI thread; resize uses the current actual bounds. `set_data` retains its
synchronous atomic API. An invalid background input leaves the last coherent scene
visible and exposes a diagnostic; acceptance does not promise successful preparation.

Disposal releases pending inputs and cached compiler ownership. A running calculation
is not forcibly interrupted; its result is rejected and its captured resources release
when it finishes. Weak native entity callbacks cannot resurrect an unmounted chart.
Metrics expose active/pending counts, submitted/coalesced/completed/stale/failed work,
committed and painted revisions, and compatible commit lag. Large counters serialize as
canonical decimal strings.

## Cache boundaries

A successful graph cache compares immutable dataset identity, source epoch, full
transform definitions, facet scope and limits. Data updates reuse unaffected dataset
transform/layer statistical outputs; unrelated stores with equal public IDs cannot hit
those entries. Statistical population signatures include input, filters, statistic,
facet scope and invalid policy. Cached output includes diagnostics as well as tables.
A failed preparation preserves the previous valid graph cache.

Style changes preserve numeric tables. State/viewport and presentation-only changes
rebind prepared metadata while retaining exact numeric populations and marks. Native
pointer overlays reuse the painted frame. Font/bounds changes rebuild layout. Geometry
can still scan unaffected source rows on mixed-dataset updates, and exact-bin contribution
caches may fall back after other cache reuse. This is not whole-pipeline O(delta).

Each immutable dataset owns a lazy shared row-key index, prepared before interactive
lookup. Mutation invalidates only the candidate dataset index. Historical snapshots
retain their own index and values. Exact row lookup then uses the ordered key index;
hover does not rescan the full dataset to describe a selected observation.

## Dense rendering and exact semantics

`DenseChart` pairs an exact `LaidOutChart` with a separate reduced paint scene. Native
painting uses the latter; inspection, source descriptions, statistics, domains and normal
publication use the exact source. Raw/prepared/rendered work counts remain separate.
Portable `dense_preview` explicitly requests the same core reduction in Rust, Python
and WASM. Normal SVG/PDF/PNG export remains exact and independent of that preview.

Monotonic straight line runs retain first, minimum, maximum and last samples in each
contiguous horizontal destination bucket, in original sequence order. Gaps and outer
endpoints survive; descending runs are supported. Curved/dashed/nonmonotonic unsupported
paths retain exact geometry. Bounds scale with destination columns plus series/gap runs;
this is not a universal vertex cap for arbitrary input shapes.

Candle buckets use stable chronological order and supplied OHLC: first open, maximum
high, minimum low and last close. Optional supplied numeric volumes use compensated
finite sums, counting only valid inputs and preserving absence. No finance model is
introduced. Each bucket retains all exact represented targets and panel/group context.
Original exact rows remain inspectable. Invalid prices, unsafe integer conversion and
sum overflow report errors. Mapped colors/sizes, custom geometry and insets use exact
candle fallback. Resolved theme color mode, mark overrides, stroke width and dashes are
applied to reduced candles. Sparse candles stay exact when aggregation would not reduce
item count. Export resolution uses a fresh exact layout by default.
