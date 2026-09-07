# FIX-17 extension fixtures

The [public example crate](../../examples/custom-extension/src/lib.rs) implements a
custom density histogram and chamfered bars with exact aggregate targets, explicit
semantic hit values, reverse keyboard order and shared custom guides. A builtin point
layer consumes the same generated table. `portable-cases.json` also exercises independent
up/down/doji candle colors on the unchanged supplied OHLC dataset.

From the repository root:

```sh
mise exec -- cargo test -p chart-extension-example -p chart-export --test contracts --test extensions --locked
mise exec -- cargo run -p chart-export --example extension_proof --locked
mise exec -- cargo run -p chart-gallery --example extension_gallery --locked
WASM_BINDGEN=/path/to/wasm-bindgen mise run bindings-proof artifacts/wp-14/bindings
```

Use the exact wasm-bindgen CLI 0.2.128. The proof runner builds explicit extension-proof
features and uses registered constructors only for the custom case. Default constructors,
wrong versions, native-only geometry and wrong generated fields must fail. The runner
compares all 36 alpha cases across actual Rust/Python/WASM with unchanged tolerances.
`--write-fixture` on the export example deliberately regenerates this new declarative
fixture from public constructors; it is not a baseline-acceptance command.

The native gallery has two buttons. Inspect both renderings, click the plot, Escape then
Right: reverse custom order first focuses interval 1–2 with count 3 and density 0.5.
Right again focuses interval 0–1. The native painter keeps its explicit export rejection;
the portable geometry emits vector SVG/PDF and deterministic PNG through normal export.
[Contract](../../docs/extension-contract.md) and [evidence](../../docs/evidence/wp-14-completion-2026-09-07.md).
