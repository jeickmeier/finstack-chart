# WP-14 completion evidence — 7 September 2026

WP-14 is **DONE** for extension contracts and the alpha API. Starting revision:
`f6c41c1` (WP-13). This evidence belongs to the WP-14 completion commit. The owner
assignment remains WP-11 through WP-23, committing each package before the next.
The cumulative [alpha feature matrix](../alpha-api.md) passes G2 at its stated scope.

## Contract and evidence

| Requirements | Result |
| --- | --- |
| SCP-01/02, ARC-03, GRA-01/08, SCN-02, QLT-05 | Separate public-API example crate implements a registered density histogram with its own generated schema and exact provenance. Its chamfered bars and a builtin point layer share the same named transform/scales. Registry/output/input validation and two compile-fail accessor cases pass. No host object or new runtime enters core. |
| GRA-08, DAT-06 supporting evidence | View changes reuse the actual named transform allocation. A keyed correction changes counts from [3,3] to [2,4] and density to [1/3,2/3], agreeing with fresh batch. Parameter changes give independently expected [1,5]. Unsupported specialized-update declarations reject; full recomputation is explicit. |
| INT-06, THM-03, FIX-17 | Custom polygon hits, finite/exact semantic values, atomic selection policy, reverse keyboard order and Lower/Boundary/Upper guides pass. Native keyboard inspection first shows interval 1–2, then 0–1, each count 3/density 0.5. Builtin points and custom bars remain separately inspectable. |
| ARC-03, SCN-03, FIX-17 | Explicit native painter registry renders rounded gradient bars under the common clip. Export rejects native-only painters by identity with UnsupportedCapability; no silent omission. Portable custom geometry emits inspected vector SVG/PDF/PNG. |
| BND-01, FIX-15/16/17 | Actual Rust, CPython and Node WASM execute all 36 alpha cases. Custom registration is explicitly selected; plain constructors, wrong versions, native-only descriptors and wrong generated fields reject. Schemas, membership, interaction metadata, labels and SVG are checked across hosts. |
| SCP-02 alpha refinement | Optional up/down candle colors assign green/red by supplied close versus open, including independent doji expectations. Coordinate capability/projection/inverse and wrong guide-type errors are exercised. All alpha builtin fields remain in the common portable definition. |

The [extension contract](../extension-contract.md) and
[ADR-012](../adr/012-registered-alpha-extensions.md) specify interfaces, defaults,
invalidation, portability, trusted-native execution, bounded output and diagnostics.
[Fixture commands](../../fixtures/extensions/README.md) reproduce the external example.

## Commands and outcomes

All commands ran in `/Users/jeickmeier/Projects/finstack-chart` on macOS 26.5.2 arm64,
Rust 1.97.1, CPython 3.14.6/PyO3 0.29.2 and Node 24.14.0/wasm-bindgen 0.2.128.
[Environment](wp-14/environment.txt), [compiled module hashes](wp-14/module-sha256.json).

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS [format log](wp-14/fmt.log). |
| `mise run check` | PASS repository/dependency isolation, native/Kit builds, Clippy, rustdoc and core WASM; [check log](wp-14/check.log). |
| `mise run test` | PASS **159 tests**: 126 core, 23 export, 5 external extension, 1 native conversion, 4 Rustdoc examples including 2 compile-fail; [test log](wp-14/test.log). |
| `mise exec -- cargo test -p chart-extension-example -p chart-export --test contracts --test extensions --locked` | PASS nine focused tests; [log](wp-14/extension-tests.log). Invalid schemas, excessive parameter nodes/depth, wrong targets/membership and finite-output failures are included. |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-14/bindings` | PASS 36 cases in each actual host; [runner](wp-14/bindings.log), [comparison](wp-14/compare.log). Semantics 1e-12, scene 1e-10 points, SVG bytes exact; existing fixture values/tolerances unchanged. |
| `mise exec -- cargo run -p chart-export --example extension_proof --locked` | PASS public registered stat/geom capture; retained [SVG](wp-14/extension.svg), [PDF](wp-14/extension.pdf), [PNG](wp-14/extension.png), [scene](wp-14/extension-scene.json), [semantics](wp-14/extension-semantics.json). |
| `mise exec -- cargo build -p chart-gallery --example extension_gallery --locked` | PASS [native build](wp-14/native-build.log). Actual app rendering/buttons/keyboard inspected. |
| `pdfinfo`, `pdffonts`, `pdfimages -list`, `pdftoppm` | PASS one 360 × 240 point PDF page, zero image objects and zero font objects in explicit outline mode; [PDF raster](wp-14/extension-pdf.png) inspected. PNG is 750 × 500 at 150 DPI. |

New-case native/Python/WASM output is retained under [wp-14](wp-14/); the other 34 cases
are covered by the complete comparison log and their earlier retained evidence. Native
captures [portable](wp-14/native-ui/portable.jpg), [native painter](wp-14/native-ui/native-painter.jpg),
[first keyboard target](wp-14/native-ui/keyboard-first.jpg) and
[second keyboard target](wp-14/native-ui/keyboard-second.jpg) were inspected at actual
window size. Shared guides, two chamfered bars, builtin points and gradient clipping agree.

Validation caught an unescaped rustdoc count literal and an ambiguous hit assertion after
adding the shared builtin point layer. The assertion now checks that the custom polygon
excludes the corner while the point legitimately includes it. It preserves the exact
custom hit contract. No visual baseline was regenerated to suppress a failure. Native
launch initially hit a stale temporary app executable; a fresh owned app bundle ran the
final build and supplied the recorded final captures.

## Limits and next action

Custom-stat execution is trusted native code with checked output and exact batch fallback;
no dynamic loading or unverified incremental performance is advertised. Generic scale/coord
registration and automatic curve subdivision are not claimed. Native callbacks explicitly
cannot export. The current inspection policy is the alpha subset, with complete gestures,
selection and host controls assigned to WP-15–17.

G2 passes the cumulative matrix; G3/G4 remain open. Linux runtime, full accessibility,
streaming/scheduling/performance and release packaging/provenance remain later work.
The six previously recorded unmaintained dependency advisories and `block` future-compiler
warning remain unresolved; this package does not reclassify them. **Next: WP-15 complete
action reducer and state ownership**, after committing this package.
