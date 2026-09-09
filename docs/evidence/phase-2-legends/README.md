# FIX-GG01 retained evidence

See [the report](../phase-2-entry-and-legends-2026-09-08.md) for scope, commands,
environment and limits. `native/` holds all 24 tested Rust primary inputs and scenes
(deterministic gzip), plus SVG/PDF/PNG. `inspection/` holds all 24 PNGs and all 24
independently rendered PDFs arranged in six labeled sheets, each visually inspected.

Python/WASM use those same input identities and compare full portable scenes exactly.
Their outputs are retained in `target/ggplot-legends/{python,wasm}`; their digests
are recorded here without duplicating identical scene/input content. The host scripts
and report describe reproduction using freshly built actual modules.

Initial and final check/test logs preserve failed evidence as well as the corrected
result. The initial full-suite failure exposed primary fixture authors relying on the
old `untitled()` behavior. They now request `generic_title()` explicitly; the independent
legacy fixtures and comparator remain unchanged. Full primary runtime/type proofs
validate that migration in Rust/Python/WASM.

`sources.sha256` identifies tested implementation/test/script inputs.
`artifacts.sha256` identifies retained files and the runtime output artifacts.
These are run digests, not deterministic PDF-byte or cross-platform guarantees.
`primary-proof.log` records the full primary runtime/type validation; the expanded
runtime files remain in `target/ggplot-legends/primary`.
