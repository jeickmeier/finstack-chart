# Portable contract v1 and binding proof surface

This is the WP-09 / G1 foundation with WP-10 statistics/positions and WP-11 scale/geometry and WP-12 facet/layout coverage, executable
Rust validation and actual Python/WASM fixtures. It is not the complete future G4 schema. See [ADR-006](adr/006-portable-specification-and-binding-proofs.md)
for ownership, dependency identity, versioning and bounds; see the
[fixture](../fixtures/bindings/README.md) for commands and independent expectations.

## Envelopes

Each envelope has required numeric `version: 1`. Unknown required constructs/variants,
unknown fields, duplicate fields and missing required fields reject. Optional fields may
be absent or null. The public decoder caps input before parsing. Rust names/enum case in
the current DTO are intentional wire spellings; changing them requires schema compatibility
work. Field/object order does not affect meaning.

| Envelope | Required content | Executable definition |
| --- | --- | --- |
| Chart | `definition`: revision, default mappings, transforms and ordered layers | [ChartEnvelope](../crates/chart-core/src/portable/wire.rs), [normalized grammar](../crates/chart-core/src/grammar/definition.rs) |
| Initial data | `epoch`, `datasets`; each dataset has `id` and schema-bearing `batch` | [DataEnvelope / BatchWire](../crates/chart-core/src/portable/wire.rs) |
| Transaction | `id`, `epoch`, `expected` dataset/schema versions, ordered `operations` | [TransactionEnvelope / MutationWire](../crates/chart-core/src/portable/wire.rs) |
| Action | `definition_revision`, `expected_state`, `action` | [ActionEnvelope](../crates/chart-core/src/portable/wire.rs), [common reducer](../crates/chart-core/src/state.rs) |
| State | `definition_revision`, `state_revision`, `viewport_revision`, `viewport`, `hidden_layers` | [StateEnvelope](../crates/chart-core/src/portable/wire.rs) |
| Profile | `width_pt`, `height_pt`, `font_size`, `padding`, `dpi`, `resource`, `outline`, `full_domain` | [ProfileEnvelope](../crates/chart-export/src/portable.rs) |

The chart DTO includes identity/count/explicit and automatic bins/summary/intercept OLS,
named transforms, affine stat spaces, source filters, generated/source mappings, grouping,
point/line/area/ribbon/bar/OHLC/rule/rectangle geoms, identity/stack/normalize/dodge/seeded jitter positions,
explicit styling, gap/order/invalid policies and named scale bindings.
[Statistics and positions](statistics-contract.md) specifies the operation versions, typed
output fields, defaults, failure behavior and exact recomputation capabilities.
Operation descriptors are `{ "id": "chart.bin", "version": "1" }`; unknown registrations
and mismatched versions/parameters reject. The complete definitions are round-trippable
native core structs, not a parallel Python/JS grammar implementation. Optional `definition.axes`
now authors linear/log/symlog/band/point/UTC/supplied-session and secondary-unit policies.
Optional layer `color` and source `low`/`high` mappings are specified in the
[scale/geometry contract](scale-geometry-contract.md). Facet/theme fields remain future work.
Semantic layer results additionally expose `color_legend` and `invalid_geometry`.

Batch fields are `schema_version`, `fields`, `keys`, `columns`. A field has `id`, `name`,
`kind`, `nullable`, optional `unit`/`label`. Column objects have `values`, `validity`, optional
`formatted`. Values are tagged `Float64`, `Int64`, `UInt64`, `Boolean`, `Utf8`, `Categorical`
or `Timestamp`; category payloads contain `codes`/`dictionary`, while other payloads are
arrays. Timestamp fields carry explicit `unit` and `timezone`. Validity is a boolean array
independent of the value payload. No null-to-zero coercion occurs. Formatted cells retain
exact decimal/display strings separately from numeric coordinates.

All 64-bit identities, revisions, integer data, timestamps, ordinals and u64 counts use
canonical decimal strings: `"9007199254740993"`, `"-9223372036854775808"`,
`"18446744073709551615"`. Numbers, leading zeros, plus signs and out-of-range strings
reject for these types. Finite float columns use JSON numbers; nonfinite source payloads
use e.g. `"bits:7ff8000000000055"` to preserve exact IEEE bits. Geometry rejects nonfinite
values through existing core rules. Envelope versions/DPI/category codes and bounded
capacities use ordinary numbers; retention capacity is explicitly u32 on every host.

Transaction variants are `AppendBatch`, `UpsertByKey`, `RemoveKeys`, `ReplaceSnapshot`,
`SetRetention` and `ResetCategoryOrder`, with existing core semantics. Data operation semantics
are fixed by envelope version 1. Actions are `SetViewport`, `SetLayerVisible`, `Reset`.
Native-only closures must be materialized before data ingestion; `TypedDataBuilder::to_portable_spec`
returns an actionable unsupported-operation error. The profile resource variant is
`{"Font":{"id":"...","revision":"..."}}`, with static font bytes passed separately.
Native pointers/widgets/filesystem paths are not resource representations.

## Runtime API and ownership

Both adapters construct `Chart(definition_json, data_json, profile_json, font_bytes)`.
They expose `semantics()`, `scene()`, `definition()`, `state()`, `transaction(json)`,
`action(json)`, `restore_state(json, expected_revision_string)`, and `dispose()`.
Python additionally exposes `export("svg" | "pdf" | "png") -> bytes`; WASM exposes
`svg() -> Uint8Array`. All JSON results are owned strings; parse them in the host when needed.
The native equivalent is `chart_export::portable::PortableChart`.

`semantics` returns versioned domains, typed prepared rows and their generated schemas,
operation records (parameters, exclusions and incremental capabilities), target provenance, exact source
chunks/schema and resulting state/revisions. `scene` returns the versioned immutable point
scene with stamp/bounds, primitives, resources, targets, fonts/hashes and diagnostics.
The target array has one entry per scene item, including empty entries for page background
and other decorations. The
scene DTO is an output snapshot, not an arbitrary scene-import or executable-painter API.
The proof profile has one font and basic two-axis point layout; output uses WP-08's encoder.

Ingestion and returned arrays are copies. Mutating the original bytearray/Uint8Array cannot
change captured fonts; mutating an output Uint8Array cannot alter a later export. No borrowed
WASM view survives a growth operation; none is exposed by the chart API. Returned copies do
survive growth and disposal. `dispose()` is idempotent, releases the payload and yields
controlled errors on later calls. JS callers should then call generated `free()` once to
release the wrapper, with no subsequent calls; Python also releases the payload on destruction.
Rust `FigureSnapshot`s retained independently remain valid and release after their last owner.

Recoverable JSON/core failures are Python `ChartError` or JS Error with a structured JSON
payload (`args[0]` / `message`). Parse it for `code`, `message`, `correction`, `severity`,
`context`. Errors do not abort either runtime. Transactions return `Applied`, `AlreadyApplied`,
`Rejected` or `Conflict` JSON with receipts/diagnostics; inspect the variant before assuming a
commit. Malformed host-language argument types remain normal Python/JS type errors.
Python Rust-only work detaches the interpreter; callbacks per point/frame are not supported.

## Proof-support matrix

| Surface | Actual WP-09/10 evidence | Boundaries |
| --- | --- | --- |
| Definition round trip | Same normalized histogram, line and point layers in native Rust/Python/WASM | Existing other normalized built-ins are exposed; full cross-runtime family coverage follows later packages. Unknown/future/native operations reject. |
| Data | All seven kinds ingested; correction/replay; null payload/display metadata; exact UInt64/Int64 extremes, >2^53 IDs and nanosecond timestamps compared | Owned JSON batches; no Arrow/NumPy/buffer borrowing, decimal arithmetic or zero-copy promise. |
| Statistics/targets/positions | Original correction fixture plus 12 actual WP-10 cases: count, summary, OLS, automatic/overflow bins, transformed values, stack/normalize, dodge and both jitter units; generated schemas, exact model/aggregate membership and final scenes agree | Built-in scope documented in the statistics contract; themes/extensions and full platform parity remain WP-13/14/21. |
| Scale/geometry families | 12 additional WP-11 cases through actual Rust/Python/WASM; log/symlog/point/color/sessions, area/ribbon/bars/heatmap/OHLC, independent and secondary axes; exact SVG and vector PDF checks | [Family contract](scale-geometry-contract.md); full platform/fidelity gate remains WP-21. |
| Actions/state | Viewport action, unchanged bin population, exact resulting revisions; restore and stale action/conflict handling | Minimal viewport/visibility/reset API, no full gesture/selection/follow scheduler. |
| Scene/publication | Point-scene JSON compared; exact fixture SVG bytes in all runtimes; Python/Rust PDF/PNG verified and inspected | Basic static-font profile; full typography/composition remains WP-13. WASM PDF/PNG not exposed or advertised by the proof. |
| Facets/guides/layout | Seven additional WP-12 cases through Rust/Python/WASM; exact typed panels, shared/free scales, broadcast/target policies, grouped/facet/chart statistics, aligned scenes and exact SVG; actual negative FIX-06 cases reject | [Facet/layout contract](facet-layout-contract.md); full themes/composition remain WP-13. |
| Python lifetime | Actual CPython 3.14.6 extension, input mutation, immutable returned bytes, interpreter-detached progress, disposal | Local module build; no wheels/notebooks/viewer, interpreter/free-threaded matrix or Python per-point callbacks. |
| WASM lifetime | Actual single-thread Node 24.14.0 WebAssembly, input/output mutation, forced memory growth invalidation, owned-copy survival, 20 repeated actions and disposal | Generated nodejs glue/TypeScript declarations; no browser DOM/renderer, workers/shared memory, manual pointer/view API or sustained RSS claim. |
| Malformed inputs | 13 constructor cases in each actual host plus core transaction/state/budget/native-accessor negatives | Version 1 only; no silent migrations or arbitrary dynamic plugins. |
| Platforms | macOS arm64 native/Python/Node execution; core/export browser-target compilation | Linux/hosted CI execution remains unverified; a Node WASM proof does not certify a browser product. |

All required built-in portability must expand with WP-11–14 and be re-executed at WP-21/G4.
This table must change when runtime fixtures change; compilation alone cannot extend it.
