# ADR-006: Portable envelopes and executable binding proofs

Status: ACCEPTED for WP-09 / minimal G1. Date: 6 September 2026.
Requirements: BND-01–04, ARC-02, DAT-01, QLT-01; FIX-15/16 subsets.

## Ownership and dependency decision

Core owns versioned wire contracts and the synchronous `portable::Session`; it calls the
existing data store, compiler and state reducer. The normalized grammar types themselves
carry strict Serde representations, avoiding a second grammar/algorithm model. Validated
private data structures are reconstructed through `Schema::new`/`NormalizedBatch::new`;
finite scene geometry only implements serialization, not unchecked deserialization.
`chart-export::portable::PortableChart` adds explicit fonts/profile and the existing
`FigureSnapshot` pipeline. Both proof adapters use that same session, including SVG output
in WASM. The WASM-to-export workspace edge is deliberate and graph-checked.

Promote already locked Serde **1.0.229** (derive/rc) and serde_json **1.0.151** into core's
normal graph. Core is no longer dependency-free, but remains synchronous and independent
of GPUI, Python, browser objects, I/O, fonts and mandatory threading. JSON parsing/serialization
is a portable boundary; no statistical or layout computation moves into a host adapter.

Adopt [PyO3 0.29.2](https://docs.rs/crate/pyo3/0.29.2), with matching build-config, for the
actual Python module. Its [detachment API](https://pyo3.rs/v0.29.2/parallelism.html) is used
around Rust-only construction, preparation, updates and exports. Default features support
ordinary Rust workspace tests; the explicit `extension-module` feature selects host Python
symbol resolution. The build script uses PyO3's own extension linker helper and, for
Python-linked binaries, its libpython rpath helper. No hardcoded interpreter path is stored
in repository source. This fixed the mise Python macOS test loader failure without excluding
the adapter from workspace tests.

Use already locked [wasm-bindgen **0.2.128**](https://github.com/wasm-bindgen/wasm-bindgen/releases/tag/0.2.128)
and its exact matching CLI; the 0.2.122 CLI initially available on the machine cannot process
this schema. The inspected generated glue copies both font input and `Vec<u8>` output, and
runtime tests prove those copies survive mutation, memory growth and payload disposal.
The proof uses single-threaded Node WebAssembly, not a browser renderer or WASI process.
Tested Node is **24.14.0**, Python is **3.14.6**, Rust is **1.97.1**, macOS **26.5.2 arm64**.
Pinned crate identities/checksums are in Cargo.lock. New PyO3 packages use MIT OR Apache-2.0;
license/source checks pass. The same six existing unmaintained workspace advisories remain
open, without ignores. No prior locked dependency version is upgraded.

## Version and validation policy

Envelope version **1** is the only supported range (1..=1). There is no earlier schema to
migrate; zero/future versions reject explicitly. Normalized built-in operation names and
versions (`chart.identity`, `chart.bin`, `chart.affine`, version `"1"`) must match their
parameters before evaluation. Unknown/native-only operations are not executable plugins.
Changes to a wire field/variant or statistical interpretation require a new version or an
explicit migration; native Rust refactors must preserve version 1 semantic round trips.
The current executable DTO definitions and [wire reference](../portable-contract.md)
are the schema authority. This package does not claim the future G4 grammar surface.

Definitions, initial data, ordered transactions, revision-fenced actions, state snapshots
and basic publication profiles use separate envelopes. All DTOs reject unknown fields and
variants; required non-optional fields cannot disappear. Optional fields may be absent or
null. Duplicate object fields reject. The public decoder checks a **4 MiB** UTF-8 input cap,
**32** structural nesting levels and **200,000** structural/string tokens before Serde
allocation. Core then checks its existing schema/data/operation/geometry budgets before
preparation. These are bounded work/input contracts, not a hard process RSS quota; host
FFI copies exist before the Rust boundary and large retained output can require allocations.

Every identity/revision, signed/unsigned source integer, timestamp, insertion ordinal and
u64 count is a **canonical decimal string**. No arbitrary 64-bit value is represented by a
JavaScript Number. Envelope version, colors, category codes, DPI and bounded counts/capacities
are ordinary integers; portable retention uses u32 independently of host pointer width.
Float64 columns use JSON numbers for finite values, or `bits:` followed by 16 hex digits for
non-finite IEEE bits, preserving NaN payloads. Null validity remains independent of payload;
original formatted cells, category dictionaries and timestamp unit/timezone metadata survive.

The constructor validates a chart against its initial data. Transactions keep existing
atomic/replay/conflict behavior. Actions check definition and expected state revisions before
calling the common reducer. State restoration checks the current expected revision,
nonregression and content/revision consistency: different state cannot reuse a state revision,
and a changed viewport cannot reuse its viewport revision. Hidden IDs must be distinct and
belong to the definition. These are minimal portable fences, not the later gesture/controlled
presentation scheduler of ADR-007 / WP-15–19.

## Ownership and errors

Owned immutable ingestion is the only binding data mode. JSON fields/columns and supplied
font bytes become Rust-owned data; output JSON strings, Python bytes and JS Uint8Arrays are
independent copies. No borrowed array pointer/view or zero-copy claim is exposed. The WASM
proof observes module memory only in its test harness to force invalidation; this does not
add a production memory accessor. Dispose drops the Rust payload idempotently; later calls
return `CHART_DISPOSED_HANDLE`. JS callers call generated `free()` only after final use to
release the wrapper; it is not a substitute for the recoverable payload-disposal protocol.
Python destruction also drops its payload. Retained `FigureSnapshot`s and returned bytes
remain valid independently. Concurrent calls on one mutable Python object remain subject
to PyO3 borrowing rules; no hidden queue or worker is introduced.

Python raises `ChartError`; `args[0]` contains the structured diagnostic JSON. WASM throws
an Error whose `message` contains the same diagnostic shape. Codes use the existing stable
`CHART_*` spellings with correction/context identities. Transaction-level rejected/conflict
outcomes are returned as typed JSON, not confused with applied receipts. Host-language type
errors before Rust entry remain normal host errors. Typed native accessors have an explicit
`to_portable_spec` rejection; callers may materialize them into an owned batch and then use
portable field mappings. Native widgets/handles/painters are not resource DTO variants.

## Evidence and remaining support

[Completion evidence](../evidence/wp-09-completion-2026-09-06.md) records actual Rust/Python/
WASM execution, independent counts/domains/targets, exact large values, error/disposal/copy
cases and inspected output. The fixed outline SVG is byte-identical in all three runtimes;
that is a measured fixture property, not a general floating-point/platform identity promise.
The [proof matrix](../portable-contract.md#proof-support-matrix) distinguishes exercised,
exposed and future behavior. Full themes/facets/statistics, rich typography, sustained load,
Linux execution, browser interaction, wheels/notebooks, free-threaded Python, public package
publication and complete G4 parity remain later work.
