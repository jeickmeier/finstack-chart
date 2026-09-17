# GG16 registered facet labeller qualification — 16 September2026

FacetLabeller.registered already selected the shared installed guide formatter registry. This slice preserves that selection and its logical ScaleValue values/names, adding optional FacetLabelContext to vector and per-value formatter inputs. It provides the exact GroupValues (including missing and margin values), full stable PanelKey, row/column, final strip side and wrap/grid context. No second registry, painter or serialized capability is introduced. Shared strip preparation passes the remaining text budget; guide output validation enforces exact cardinality and byte budgets before painting.

The external example `examples/custom-extension/src/facet_labels.rs` registers portable and native-only implementations through CustomGuideFormatter. `examples/custom-extension/tests/facet_labels.rs` passes two focused tests (`/private/tmp/gg16-facet-label-tests3.log`): missing/margin/int/text context, top/right grid sides, serialization/replay scene equality, installed native-only JSON/Points rejection and LogicalPixels execution, malformed result cardinality and text-budget rejection, and missing labels.

Mode9 in the independent Rust/Python/WASM extension authors selects the labeller. The full author scope is now ten authors/30publications, including the separate registered-model mode8. Host scripts also exercise malformed and native-only labellers. Root's final2 actual Rust/Python/WASM execution passed all30publications per host with exact equality. Aggregate acceptance remains recorded by the root ledger.

Scoped Clippy passed with warnings denied in `/private/tmp/gg16-facet-label-clippy.log` (core and external test target,22seconds).

Native modes8/9 were rebuilt and inspected with readable720px gallery cells. Model interval and all typed facet labels are visible in `/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-16_20-37-34.png`; manifest and zero-diagnostic paint trace are under `/private/tmp/gg16-model-labeller-native`. All owned windows were closed.
