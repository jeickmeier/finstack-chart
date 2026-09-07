# WP-09 portable Rust / Python / WASM fixture

The five positive JSON files are authored version-1 inputs for the same core session.
They are not outputs generated from expected values. [negative.json](negative.json) describes
13 field mutations applied identically in the two host runtimes. The schema and public
surface are documented in [the portable contract](../../docs/portable-contract.md).

## Run

Under mise, provide Node (tested 24.14.0), Poppler (`pdfinfo`, `pdffonts`, `pdfimages`) and
**wasm-bindgen-cli 0.2.128** matching Cargo.lock. The runner does not download tools or install
into Python. A different CLI version fails with an explicit instruction; set `WASM_BINDGEN`
to a task-local executable when the PATH version differs.

```sh
mise run bindings-proof
# With a locally downloaded/installed matching CLI:
WASM_BINDGEN=/path/to/wasm-bindgen-0.2.128 mise run bindings-proof
# Optional explicit artifact directory:
WASM_BINDGEN=/path/to/wasm-bindgen-0.2.128 mise exec -- python scripts/run_binding_proofs.py artifacts/my-binding-proof
```

[The runner](../../scripts/run_binding_proofs.py) builds the native example, the real PyO3
module with the selected Python, and the actual `wasm32-unknown-unknown` module. It generates
nodejs glue and declarations with the matching CLI, runs Python/Node independently and
compares actual outputs. Results/logs are under `artifacts/bindings` by default. It copies
the Python extension to the output directory to isolate it from later Cargo feature builds.
Full Python packaging is not implied. The command was actually executed on macOS; Linux
runtime support is not claimed by the runner's portability alone.

## Independent expectations

- Dataset ID `9007199254741001`; layer IDs `9007199254742001`–`...2003`;
  row keys `9007199254743001`–`...3004`. All stay decimal strings.
- Source x values [0.25, 0.75, 1.25, 1.75], with the third x invalid. Initial second y is
  invalid. Explicit bins [0,1,2] count **[2,1]**. Correcting the second row to x=1.5, y=4
  yields **[1,2]**, with aggregate members [row1] / [row2,row4]. The invalid third row's
  payload remains 1.25 but does not join a bin or paint a point.
- Viewport [0.5,1.6] clips geometry; it does not change those counts or training x=[0,2].
  Line/point trained y extends from [1,2] to [1,4]. One correction advances store/dataset
  revision to `"1"`; replay returns AlreadyApplied. One viewport action advances state
  and viewport revision to `"1"`.
- Four source nanosecond timestamps are 1712345678901234567 through 1712345678901234570.
  Adjacent ticks stay distinct without passing through JS Number. UInt64 max and Int64 min/
  max, boolean/UTF-8/category data and original `0.2500` formatting remain intact.
- Page 360 × 240 pt; at 150 DPI, PNG is exactly 750 × 500 pixels. The fixture uses outlined
  Noto Sans from [the unchanged OFL capability font](../capability/README.md), with no
  system-font lookup. SVG/PDF preserve vectors; outline PDF has no fonts/images.

[compare.py](../../scripts/bindings/compare.py) checks semantic numbers at **1e-10 absolute**,
scene numbers at **0.01 pt**, and IDs/types/members/counts/revisions exactly. It also checks
these independent expectations; agreement alone is insufficient. The observed outline SVG
bytes match exactly across all three runtimes. Float64 nonfinite bit preservation, malformed
JSON bounds, atomic failures, revision restoration and native accessor rejection have
[core tests](../../crates/chart-core/tests/portable.rs); retained publication ownership has
[export tests](../../crates/chart-export/tests/portable.rs).

Python tests progress of another Python thread during detached Rust work with bytecode
switching suppressed around the call. WASM tests capture the instantiated module's memory
inside the test harness only, grow it, verify the old view is detached, and verify returned
SVG copies and future calls stay valid. No production memory-view API or zero-copy claim
is introduced. [Completion evidence](../../docs/evidence/wp-09-completion-2026-09-06.md)
records actual results, inspected images and remaining limits.
