# WP-S05 complete symbol encoding — implementation and qualification

Status: **COMPLETE**. Final macOS qualification below supersedes the historical pending entries. Revision: working tree over `fab2505951061eaafe9adb52c86b248ee0dfa6bf`.
SHP-06/09 and FIX-S06/09 apply. WP-S01 is accepted; G-SHAPE remains open.

The core implementation supplies all thirteen symbol types, both ordered palettes,
`X`/`Times` identity, checked native `SymbolDraw`, standalone immutable Python/WASM
owners and primary `shape_symbol`/`shapeSymbol` layers. Explicit type catalogs,
source/prepared-group readers, numeric area scales and actual resolved guide glyphs
share the existing grammar and path destinations. Legacy point radius is preserved.
[ADR-020](../adr/020-shape-generators-and-curve-protocols.md) records paint, zero-size,
unit, ownership and guide policies.

## Evidence completed

- Independent pinned d3-shape fixtures contain 156 cases across all types and sizes.
  Rust tests compare exact SVG precision 0/3 and raw context coordinates under the
  predeclared `2e-12 * max(1, abs(expected))` tolerance. Independent filled-area,
  stroke-topology, aliases, negative/nonfinite/zero, budgets and native protocol
  assertions pass. Fixed reference star rotation constants resolved a reproduced
  Linux-libm difference at extreme sizes; fixtures and tolerances were preserved.
- The final focused Linux run passes 37 core tests, including five symbol
  integration tests that cover logarithmic center projection, legacy radius,
  exact path/target identity, filled versus open hit geometry, zero-size omission,
  missing-type policy, explicit catalog validation, wire v7 and guide samples.
  Input values 0/1/2 through a 0..2 to 16..256 mapping resolve to areas 16/136/256.
- Actual rebuilt Linux Python and Node WASM each pass all 156 fixtures and 104 append/upsert/remove/retention
  comparisons across thirteen types with and without facets, exact keys above 2^53,
  type/area corrections and exports retained after disposal. Separate presented
  interactions cover field/expression inputs, log centers, open-stroke interior
  misses, clipping/focus, zero size, generated count identities and exact mapped
  square widths in both marks and size guides.
- Independent Rust/Python authors export all thirteen types at sizes 16/64/256,
  mapped type/size legends, Editorial/Terminal/Grayscale, SVG/PDF/PNG at 300/600 DPI.
  All three PNG galleries, all three externally rendered SVGs and all three 600-DPI
  PDF inputs rendered at 96 DPI were visually inspected: expected closed/open
  topology, growing sizes, complete labels and guides, no overlapping furniture.
  Supplied Noto Sans is embedded. All twelve cross-host/theme/resolution comparisons
  have exact complete scenes (zero coordinate differences) and byte-identical
  PNG/SVG/PDF output. These observations are not native acceptance.
- Arc regressions after symbol integration pass all 64 Python updates and presented
  interaction checks. Complete scenes and PNG/SVG/PDF bytes match the preceding
  retained arc output exactly at both resolutions.
- Linux Rust 1.97.1 Clippy for core/export/Python/WASM all targets and rustdoc with
  warnings denied pass after borrowing shared legend metadata and deriving the
  equivalent pie default. Strict TypeScript and Python positive/negative symbol
  consumers pass (five intended Python negative errors).

## Pending qualification and limits

The final Linux Python and WASM source builds and symbol proofs pass after lint
cleanup. macOS native and fresh Python qualification remain pending: existing
builds are stalled in the operating-system loader/code-signature path, and the
normal repository check is waiting on the editor's build lock. No OS security
setting was changed, no unrelated process was terminated, and no native or complete
repository gate is inferred from Linux results. The full primary runner includes
symbol commands but has not yet completed on this combined source.

[Source identities](phase-2-shape-symbol/source-sha256.json),
[runtime identities](phase-2-shape-symbol/runtime-sha256.json),
[artifact identities](phase-2-shape-symbol/artifact-sha256.json) and
[exact comparison results](phase-2-shape-symbol/comparison.json) retain this snapshot.
Close WP-S05 only after native inspection, fresh macOS Python and required repository
qualification. Custom portable registration and
full cross-family styling/performance acceptance remain WP-S07/08.


Native follow-up: the task-owned ShapeSymbolProof application painted three
frames with one layout. Its window was visually inspected: the complete symbol
gallery, facets, labels and legends are readable with the expected shape semantics.
The screenshot, paint log and executable identity are retained under
`phase-2-shape-symbol/native/`. macOS Python and final repository qualification
remain open.


Final macOS qualification supersedes the pending platform statements above. The
fresh current-source Python extension passed the standalone and interaction corpus,
all family update/retention cases, and both publication resolutions. Publication was
compared with the independently authored Rust and WASM artifacts using the existing
family comparison contract. Native windows were inspected and retained as described
above. `CARGO_TARGET_DIR=target/shape-native-target mise run check` passed repository,
format, dependency/license, native/Kit build, workspace all-target checks/Clippy,
rustdoc with denied warnings, and WASM core compilation. `mise run test` with the
same private target passed 393 tests/doctests in 86 result blocks, zero failures and
zero ignored tests; empty test blocks are not counted as tests.

The accepted Python runtime, source hashes, publication, comparisons, build/check
and full-test logs are retained in this family's `macos/` evidence directory. Every
recorded source hash was checked again after the full suite and remained unchanged.
Native executables retain their separately recorded pre-radial standalone-addition
identities; the arc/symbol/stack implementation and native renderer were unchanged,
and the fresh host and workspace checks cover the subsequent shared-reader change.
The three transient proof applications were closed after inspection. This completes
the family package; cumulative G-SHAPE, remaining Phase 2 and WP-21/22/23 release
qualification are separate and remain open.
