# Reproduction commands

Run from the repository under the versions in `environment.json`. Commands use the
committed lockfile. `target/hierarchy-proof/python-module` contains the freshly built
PyO3 extension; `target/hierarchy-proof/wasm-module` contains the matching generated
WASM module and current package facades.

```sh
mise exec -- python scripts/hierarchy/requests.py target/hierarchy-proof/requests.json
mise exec -- python scripts/hierarchy/padding.py requests target/hierarchy-proof/padding-requests.json
mise exec -- python scripts/hierarchy/controls.py requests target/hierarchy-proof/control-requests.json

# Python build uses PYO3_PYTHON set to the recorded mise Python executable.
mise exec -- cargo build -p chart-python --features extension-module,extension-proof --locked
cp target/debug/libchart_python.dylib target/hierarchy-proof/python-module/chart_python.so
mise exec -- cargo build -p chart-wasm --features extension-proof --target wasm32-unknown-unknown --locked
target/axis-ticks/tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen --target nodejs --out-dir target/hierarchy-proof/wasm-module target/wasm32-unknown-unknown/debug/chart_wasm.wasm
cp packages/wasm/*.cjs packages/wasm/*.cts target/hierarchy-proof/wasm-module/

CARGO_TARGET_DIR=target/hierarchy-proof mise exec -- cargo run -p chart-extension-example --example hierarchy_proof --locked -- target/hierarchy-proof/requests.json target/hierarchy-proof/rust.json
mise exec -- python scripts/bindings/hierarchy.py target/hierarchy-proof/requests.json target/hierarchy-proof/python.json target/hierarchy-proof/python-module
node scripts/bindings/hierarchy.cjs target/hierarchy-proof/wasm-module target/hierarchy-proof/requests.json target/hierarchy-proof/wasm.json
mise exec -- python scripts/hierarchy/compare.py target/hierarchy-proof/rust.json target/hierarchy-proof/python.json target/hierarchy-proof/wasm.json
```

Run the same three hosts with `padding-requests.json`/`padding-HOST.json` and
`control-requests.json`/`control-HOST.json`; compare using:

```sh
mise exec -- python scripts/hierarchy/padding.py compare target/hierarchy-proof/padding-rust.json target/hierarchy-proof/padding-python.json target/hierarchy-proof/padding-wasm.json
mise exec -- python scripts/hierarchy/controls.py compare target/hierarchy-proof/control-rust.json target/hierarchy-proof/control-python.json target/hierarchy-proof/control-wasm.json

CARGO_TARGET_DIR=target/hierarchy-proof mise exec -- cargo run -p chart-export --example hierarchy_chart_proof --locked -- target/hierarchy-chart/rust
mise exec -- python scripts/bindings/hierarchy_chart.py target/hierarchy-proof/python-module target/hierarchy-chart/python
node scripts/bindings/hierarchy_chart.cjs target/hierarchy-proof/wasm-module target/hierarchy-chart/wasm
mise exec -- python scripts/bindings/axis_components_compare.py target/hierarchy-chart hierarchy
mise exec -- python scripts/bindings/hierarchy_updates.py target/hierarchy-proof/python-module docs/evidence/phase-2-hierarchy-integration/python-updates.json
node scripts/bindings/hierarchy_updates.cjs target/hierarchy-proof/wasm-module docs/evidence/phase-2-hierarchy-integration/wasm-updates.json
mise exec -- python scripts/hierarchy/updates_compare.py docs/evidence/phase-2-hierarchy-integration/python-updates.json docs/evidence/phase-2-hierarchy-integration/wasm-updates.json
mise exec -- python scripts/bindings/hierarchy_ownership.py target/hierarchy-proof/python-module docs/evidence/phase-2-hierarchy-integration/python-ownership.json
node --expose-gc scripts/bindings/hierarchy_ownership.cjs target/hierarchy-proof/wasm-module docs/evidence/phase-2-hierarchy-integration/wasm-ownership.json

CARGO_TARGET_DIR=target/hierarchy-proof mise exec -- cargo run -p chart-export --example hierarchy_benchmark --release --locked
CARGO_TARGET_DIR=target/axis-rust-proof mise run test
CARGO_TARGET_DIR=target/axis-rust-proof mise run check
mise exec -- python scripts/hierarchy/catalog.py
```

Typing uses mypy `--strict` with `MYPYPATH=packages/python` on
`scripts/bindings/authoring/hierarchy_typing.py`. The installed mypy module is supplied
through the task's `PYTHONPATH`; no package download is part of qualification. TypeScript
uses `--noEmit --strict --target ES2022 --lib ESNext,DOM --module NodeNext
--moduleResolution NodeNext`. Copy `hierarchy_types.cts` next to generated WASM types and
replace its package-relative import with `./authoring.cjs` before compiling.

Linux executes the headless packages in a cached image, with networking disabled:

```sh
docker run --rm --network none -e CARGO_NET_OFFLINE=true -e CARGO_TARGET_DIR=/workspace/target \
  -v /Users/jeickmeier/Projects/finstack-chart:/workspace:ro \
  -v /Users/jeickmeier/.cargo/registry:/usr/local/cargo/registry:ro \
  -v /Users/jeickmeier/.cargo/git:/usr/local/cargo/git:ro \
  -v /Users/jeickmeier/Projects/finstack-chart/target/path-proof/linux-target:/workspace/target \
  -w /workspace rust:1.97.1-bookworm cargo test -p chart-core -p chart-export -p chart-text --locked
```

Native inspection builds `chart-gallery --example hierarchy` using
`CARGO_TARGET_DIR=target/shape-native-target`, bundles the executable as HierarchyProof,
ad-hoc signs the local proof app, opens it, captures the nine-panel screenshot and closes
that exact process. The prior app bundle needed re-signing after replacing its executable;
no system security setting was changed. Independent SVG inspection uses the existing
resvg 0.45.1 inspection binary with the committed Noto Sans font. PDF inspection uses
`pdftoppm -singlefile -scale-to 700 -png`. Contact sheets preserve both text and outline
results; images were inspected rather than accepted merely because files were generated.

The final native/host builds precede only the extra constructor regression test and
evidence documentation; no production behavior changed after those builds (the final facade edit removes an extra blank line). Source
qualification includes the additional test. `source-snapshot.json` fingerprints sources,
fixtures and proof scripts; `chart-artifact-hashes.json` identifies actual outputs.
