# Pinned shape oracle

Reference: d3-shape 3.2.0, d3-path 3.1.0, Node 24.14.0. `manifest.json` retains
exact source/lock/generator/data hashes and `LICENSE` retains the ISC grant.
Generate with `mise exec -- node tools/reference/node/shape.mjs`; pass a separate
output directory to verify byte-identical regeneration without changing fixtures.

`inventory.json` lists all 63 exports, aliases, methods, defaults, factory controls,
fill/stroke palettes and boundary helper inheritance. `cases.json` retains 333
full-precision path-context streams, unrounded/digits 0/3/12 strings, 42 pie/stack
layout records and observed custom curve lifecycle calls. These are independent
reference expectations, not Rust-generated baselines or feature-pass declarations.
Entry status is OPEN for every geometric family until its owner supplies actual
implementation and acceptance evidence. New family cases expand this seed corpus.

WP-S01 proves context replay through the existing path engine. Its exact rounded
string tests do not prove built-in shape generators: those begin at WP-S02.
[ADR-020](../../docs/adr/020-shape-generators-and-curve-protocols.md) records typed
adaptations, numeric comparison policy, resource ownership and implementation scope.
