# Portable contracts and proof adapters

The implemented Rust, Python and WASM proof surfaces share the normalized core compiler,
state reducer, inspection, streaming and publication engine. Envelope version **1** is
independent of the evolving specification-document version. This document describes the
versioned compatibility scope. New applications use the host-native
[primary authoring API](authoring-guide.md#python-and-javascript): ordinary data and
components forward to the same Rust Chart/Output owners. Expanded D3/ggplot2 semantics
and primary-API qualification retain their own gates. [ADR-006](adr/006-portable-specification-and-binding-proofs.md) records
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

Legacy definitions require numeric `version: 1`. Retained path composition uses version
two in the primary/normalized definition and composition envelopes, and scene results
containing `VectorPath` report version two. A version-one definition cannot carry
version-two composition; mismatched capabilities reject. Data, state, transaction and
publication-profile envelopes retain independent version contracts.
Standalone `PathRequest`/path results also start at version one; they do not require a
chart, font or publication profile. [ADR-015](adr/015-path-authoring-and-replay.md)
records this migration. Unknown fields, duplicate fields,
unknown variants/operations and missing required fields reject. Optional fields may be
omitted only where the DTO declares a default; object field order does not change meaning.
Inputs are size-bounded before parsing and checked again before preparation/allocation.
Operation IDs and positive versions resolve exactly, for example
`{"id":"chart.bin","version":"1"}`. Changing wire spellings requires compatibility
review even when the Rust type retains its name.

Definitions with compatibility semantics, staged expressions, inferred interaction
groups, horizontal orientation or geometry-theme context require version **3** in both
primary and normalized definition envelopes. Version 1/2 cannot carry those capabilities.
Theme objects with geometry tokens use theme version 2; scene version continues to depend
on its own primitives. `ExecutionSemantics` retains the exact profile, policy version,
source SHA-256, grouping rule and scale stage; unsupported or contradictory policy fields
reject. The primary envelope's profile must agree with the canonical definition.
LibraryV1 omits the optional semantics object and retains existing legacy defaults.

Definitions with floating authored paint require at least version **4**. Every paint input
accepts either legacy `{red,green,blue,alpha}` bytes or a version-one color descriptor
with retained space/channels. CSS strings are parsed once at ingestion. Numeric
channels explicitly tag NaN, positive/negative infinity and negative zero; raw JSON
null is invalid. Primary owned host colors use the same descriptor. A v1–3 definition
cannot conceal a floating input in a palette, theme, rich text, path or expression.
Authored `AxisScale::Numeric` knots/parameters and `D3Band`/`D3Point` spacing require
definition version **5**, as do `ColorScale::Mapped`, a layer's `numeric_scales`, and
an axis's `numeric_format` descriptor. Numeric formatting retains its full specifier
and explicit locale values; it cannot coexist with legacy `number_format` or custom
tick labels and rejects nonnumeric guides. Formatting changes labels without rounding
mapped coordinates. Missing precision is inferred from the visible data-space tick step.
Mapped descriptors preserve typed inputs/outputs, interpolation, unknown policy and
quantile training source. Guide metadata includes real intervals, midpoint and the
prepared mapping contract. Floating paint survives style opacity until final scene lowering.
Old envelopes retain their prior meanings and cannot carry this variant. Standalone
numeric scales use a strict version-one `NumericScaleSpec` envelope with explicit
Legacy/D3 policy; unknown result values use the shared interpolation value codec.
The current primary host wire readers recognize definition versions 1–6 and require
the version appropriate for the contained capabilities. Standalone scale host
constructors and method qualification are recorded under SP-07.
Scene versions still describe resolved primitive capabilities; final colors are bytes.

Publication-profile envelope version **2** is required when its host/output theme has
floating paint or its optional `background` field is present. Version one retains the
old fields and byte tokens. Live-export Begin uses version two for a floating output
theme; Cancel/Status and byte-only Begin retain version one. Profile/capture metadata
preserves the authored descriptors independently of the resolved publication scene.
See [ADR-016](adr/016-color-values-and-paint-boundary.md) for exact lowering and the
legacy/floating palette distinction.

Layer/transform grammar retains source aesthetics and explicit statistic grouping before
generated fields replace visible mappings. Expressions contain typed read/constant,
unary/binary, selection and reduction nodes with a checked result index and operation
budgets. Cross-stage host composition rejects before serialization. Post-scale reads use
the pre-modifier snapshot, avoiding output-order dependence. Profile changes participate
in definition revisions, workers/cache equality and captured export provenance. See the
[stage authoring contract](authoring-guide.md#compatibility-stages) for population rules
and the supported aesthetic boundary.

Color-guide `title: null` (or an absent defaulted title) retains the generic fallback,
explicitly available through primary `legend().generic_title()`.
`title: ""` explicitly omits its title row, as emitted by primary `legend().untitled()`.
A guide with zero entries contributes no furniture; its prepared metadata remains
available. These fixes keep the existing version-1 representation and nonempty-title
behavior. [FIX-GG01](../crates/chart-export/tests/ggplot_legends.rs) covers primary
interchange, edits, visibility, compatible/incompatible guides and tight layout.

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

## Compatibility methods and ownership

The low-level proof adapters retain the constructor below. The primary packages expose
Data/Plot/Chart/Output, structured results and typed component methods; they do not
require the caller to assemble these JSON envelopes. Session forwards decoding into
the typed runtime, and wire version 1 remains unchanged.

Both compatibility adapters construct `Chart(definition_json, data_json, profile_json, font_bytes)`.
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

### Calendar resources and time operations (definition v5)

A `Calendar` positional scale retains `TimeScaleSpec` and an optional
`CalendarInterval`. Domains use canonical integer strings with an explicit source
unit. `CalendarZone::Utc` requires no external resource; `Local` carries version-one
`TimeZoneRules` with zone, revision, tzdata identity, inclusive covered UTC millisecond
bounds, initial offset seconds and ordered transitions. All calendar arithmetic and
formatting uses those immutable rules. Uncovered operations report `MissingResource`.

Axis `time_format` stores an optional custom pattern and explicit `TimeLocale`; absent
patterns use conditional formatting. It cannot coexist with numeric formatting or
custom tick labels. Both calendar scales and time-format-only axes require v5.
Standalone time envelopes independently use `{version: 1, spec: ...}`. Actual Python
and WASM handle proofs preserve the same resource revisions and exact integer values;
no host timezone arithmetic is part of the contract. Invalid reference Dates are
reported as numerical-domain diagnostics at integer timestamp result boundaries.

### Named chromatic identities (definition v6)

A mapped scale containing a `catalog` selection or an interpolator with the
`Chromatic` variant requires definition version **6**. The strict catalog selection
contains version one, scheme ID, exact optional size and reversal; its retained range
must equal that named array. The strict ramp payload retains interpolator ID and
reversal within the version-one interpolation descriptor. Catalog revision one is
pinned to d3-scale-chromatic 3.1.0. Existing v1–v5 definitions preserve their meanings;
an envelope cannot claim an older version while carrying named chromatic capabilities.
Guide metadata retains the complete mapping and participates in equality and caching.
The public catalog, scheme and owned interpolator APIs use this same core owner.
