# Portable contract v1 and proof adapters

The implemented Rust, Python and WASM proof surfaces share the normalized core compiler,
state reducer, inspection, streaming and publication engine. Envelope version **1** is
independent of the evolving specification-document version. This document describes the
original implemented scope; expanded D3/ggplot2 parity and the primary-authoring API retain
their own open gates. [ADR-006](adr/006-portable-specification-and-binding-proofs.md) records
dependency identity and wire ownership; [binding instructions](../fixtures/bindings/README.md)
run the actual adapters and independent comparisons.

## Executable schema and encoding

Rust Serde DTOs and their checked constructors are authoritative. Canonical examples live
in the shared [fixture catalog](../fixtures/README.md); there is no second handwritten
Python/JavaScript chart schema or evaluator.

| Input/result | Executable contract |
| --- | --- |
| Chart, initial data, schema-bearing batches, transactions and saved state | [Core wire DTOs](../crates/chart-core/src/portable/wire.rs) |
| Layer/statistic/position/scale/facet/theme definitions | [Grammar](../crates/chart-core/src/grammar/definition.rs), [complete original API matrix](alpha-api.md) |
| Actions, origins, controlled-state and presented-scene fences | [State/action contract](state-action-contract.md) |
| Queries, navigation, selection and editing | [Interaction contract](interaction-contract.md), [host tools](host-tools-contract.md) |
| Queue admission, commit, retention and follow behavior | [Streaming contract](streaming-contract.md) |
| Point dimensions, font resources and publication policy | [Profile DTO](../crates/chart-export/src/portable.rs), [typography/composition](theme-typography-composition-contract.md) |
| Deferred coherent export jobs | [Live-export contract](live-export-contract.md), [checked operations](../crates/chart-export/src/portable/live_export.rs) |

Versioned envelopes require numeric `version: 1`. Unknown fields, duplicate fields,
unknown variants/operations and missing required fields reject. Optional fields may be
omitted only where the DTO declares a default; object field order does not change meaning.
Inputs are size-bounded before parsing and checked again before preparation/allocation.
Operation IDs and positive versions resolve exactly, for example
`{"id":"chart.bin","version":"1"}`. Changing wire spellings requires compatibility
review even when the Rust type retains its name.

Batches contain `schema_version`, ordered `fields`, durable `keys` and matching `columns`.
Each column has tagged `values`, independent boolean `validity`, and optional exact
`formatted` cells. Supported kinds are Float64, Int64, UInt64, Boolean, Utf8, Categorical
and Timestamp. Categories carry codes/dictionaries; timestamps declare unit and timezone.
A false validity bit retains the payload but excludes it from numeric evaluation. It is
not a zero value. Exact display/decimal strings remain separate from floating coordinates.

All 64-bit identities, revisions, integer data, timestamps, ordinals and u64 counters use
canonical decimal strings, including `"9007199254740993"` and
`"18446744073709551615"`. JavaScript Number cannot preserve these identities. Numeric
versions/DPI/bounded capacities use the specified ordinary numeric types. Finite floats
use JSON numbers; nonfinite source payloads use explicit IEEE-bit strings such as
`"bits:7ff8000000000055"`. Geometry requires finite values. Leading zeros, plus signs,
out-of-range decimal strings and wrong scalar kinds reject where canonical integers
are required.

Transactions support append, whole-row upsert, remove, replace, retention, watermark and
category-order operations through the same atomic store. Expected source/dataset/schema
fences and transaction replay are mandatory contracts. A remembered identical transaction
is `AlreadyApplied`; reusing its ID for different input rejects. Queue admission is not a
commit, and coalescing numeric preparation does not discard accepted data operations.

Chart envelopes include the original delivered statistics, positions, geometry, scales,
facets, themes, text and composition. Registered portable extensions require an explicitly
supplied compiled registry. Callback code, native pointers, GPUI widgets and filesystem
paths are not portable resources. Native-only definitions and unknown registrations return
structured errors; serialization never installs executable plugins.

## Methods and ownership

Both adapters construct `Chart(definition_json, data_json, profile_json, font_bytes)`.
The matching Rust object is `chart_export::portable::PortableChart`. JSON inputs/results
are owned strings. Python byte results and WASM Uint8Array results are owned copies.

| Methods | Meaning |
| --- | --- |
| `definition()`, `semantics()`, `state()`, `scene()` | Canonical definition, computed typed/provenance results, durable state and immutable publication-scene JSON |
| `transaction(json)`, `stream(json)` | Atomic operation outcomes or explicit configure/enqueue/commit/status/pin operations |
| `present()` | Explicit acknowledgement of a scene for subsequent scene-dependent input |
| `query(json)` | Presented-scene lookup/navigation/selection without recompiling statistics per pointer event |
| `action(json)` | Programmatic action with definition/state fences through the common reducer |
| `dispatch(json)` | Full origin/state/scene-fenced action request and effective events |
| `restore_state(json, expected_revision_string)` | Checked durable-state restore; ephemeral gesture/hover preview is not restored |
| `dense_preview(json)` | Explicit line/candle destination reduction with retained exact source semantics and work counts |
| `export_control(json)`, `export_job(job_id_string)` | Bounded immutable capture/cancel/status, then later synchronous execution on the caller's executor |
| `dispose()` | Idempotent payload release; later operations return a disposed-handle diagnostic |

Python also provides `export("svg" | "pdf" | "png") -> bytes`. The WASM convenience
method is `svg() -> Uint8Array`; deferred job operations accept SVG/PDF/PNG. Actual shared
live-export fixtures cover SVG and PNG. A browser PDF workflow is not certified merely
because the deferred format is accepted by the Rust-backed method.

`scene()` returns an output snapshot with stamp/bounds, numeric primitives, targets,
resource/font identity and diagnostics. It is not an arbitrary scene-import or native
painter protocol. `semantics()` retains source metadata, generated schemas, operation
records, exclusions and source/aggregate/derived provenance. A generated or reduced mark
must not impersonate an original source row.

Input buffers are copied. Mutating a Python bytearray or JavaScript Uint8Array after
construction cannot modify a font/snapshot. Returned copies survive later memory growth
and chart disposal. No borrowed WASM memory view is exposed by this API. After `dispose`,
JavaScript may call generated `free()` once to release the wrapper, with no later calls.
Python destruction also releases ownership. Rust-only Python work detaches the interpreter;
there are no per-point Python callbacks or implicit browser worker/thread requirements.

Recoverable failures are Python `ChartError` or JavaScript `Error` with a structured JSON
payload in `args[0]` / `message`. Parse `code`, `message`, `correction`, `severity` and
`context`. Ordinary wrong host argument types remain host type errors. Transactions return
`Applied`, `AlreadyApplied`, `Rejected` or `Conflict` JSON; inspect that outcome before
claiming a commit. Failed updates preserve the prior valid state where the contract permits.

## Executed scope and compatibility

[WP-21](evidence/wp-21-completion-2026-09-07.md) records actual Rust/Python/Node WASM
execution for 36 original cases, 23 action transitions, 47 input steps, 70 streaming steps,
three density cases and 40 live-export steps. Exact IDs/timestamps, null/invalid values,
scene stamps, aggregate memberships, malformed input, memory growth and disposal are
compared. Linux aarch64 also runs the headless Rust fixtures against the same host outputs.
The [support matrix](support-matrix.md) identifies current test environments and limits.

The Python/WASM packages remain proof adapters: no wheel/npm/browser viewer/notebook or
free-threaded interpreter release is asserted. Node WASM execution does not certify browser
DOM accessibility, browser font policy or a GPUI binding. The `extension-proof` build
feature adds only a known compiled example registry through
`with_example_extensions` / `withExampleExtensions`; default constructors remain explicit.

Crate versions and envelope/operation versions are separate. Public Rust enum additions
and struct-field changes can break exhaustive consumers before 1.0 and belong in the
[change log](../CHANGELOG.md). Optional v1 fields must preserve their documented defaults;
incompatible required fields or semantics need a version/migration decision. The planned
primary-authoring API is not an implicit migration of these interchange contracts.
