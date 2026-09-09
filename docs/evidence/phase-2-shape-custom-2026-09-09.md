# WP-S07 custom shape portability — 9 September 2026

WP-S07 is complete for SHP-01/08/09 and FIX-S08/09. Base revision:
`fab2505951061eaafe9adb52c86b248ee0dfa6bf`, uncommitted Phase 2 continuation.
[Source](phase-2-shape-custom/integrated/source-sha256.json),
[runtime](phase-2-shape-custom/integrated/runtime-sha256.json) and artifact hashes
identify the qualification snapshot. G-SHAPE remains open for WP-S08.

`ExtensionRegistry` captures exact identity, version, family and portability for a
bounded shape catalog. `ShapeOperation` carries bounded JSON parameters; resolution
checks the factory result's protocol family. Native protocols remain directly usable.
Portable serialization rejects native-only registrations explicitly, and JSON never
installs code. Prepared layers own resolved native implementations independently of
the mutable registry or host handles. [ADR-020](../adr/020-shape-generators-and-curve-protocols.md)
records the wire-v9 capability, command/work limits and chart comparator datum shape.

The external example crate implements a shifted curve lifecycle, area-sized rectangle,
exact unsigned datum-field pie comparator, first-value stack ordering and shifted
cumulative offset. Standalone Cartesian/radial lines and areas, links, symbols, pies
and stacks reuse their existing kernels. Chart layers use the same implementations;
custom symbol size guides retain the actual resolved glyph. Built-in setters clear
corresponding custom selections, avoiding stale descriptor combinations. Host
registries, typed selections, registered generator/layout methods, primary builders,
portable restoration, positive/negative type consumers and generated Rust API docs
cover the public contract. The consolidated primary proof runner includes all new
custom tests, updates, publications and type checks.

Ten external Rust tests pass on macOS. They check independent projected coordinates,
gapped area lifecycle reuse, symbol area, pie ranks/angles and exact IDs, stack endpoints
and domains, registry disposal, wire restoration, native-only errors, invalid protocol
outputs, family/version/catalog limits and command/work exhaustion. A declared curve
bound fails before text callbacks when the figure budget is too small; an understated
bound fails at the checked path sink. Tests retain the existing typed-input and
resource contracts.

Actual macOS/Linux Python and Node WASM execute all five registered protocols,
including copied/disposed owners, invalid families, callback rejection, exact metadata,
missing registrations and malformed parameters. Both macOS Python and WASM pass 72
append/upsert/remove/retention comparisons over nine chart families with and without
facets. Current PNGs equal fresh batches exactly; old publication snapshots remain
unchanged after all mutations and chart disposal. The final Linux update rerun and
broader full-suite regression results are recorded separately below.

The ten-panel common gallery retains 53 source targets, 17 ShapePaths and three
custom size-guide paths. Rust-authored version-nine definitions are restored through
the registered Python/WASM engines. Across Editorial, Terminal and Grayscale at
300/600 DPI, all complete scenes and all SVG/PDF/PNG bytes match exactly in
[12 comparisons](phase-2-shape-custom/integrated/comparison.json). These gallery
restoration proofs complement separately authored core and host update tests; they
are not independent gallery authors. All 18 PNGs have equal RGBA pixels within each
configuration. Noto Sans is supplied and embedded in every PDF. Original SVGs were
externally rasterized, PDFs rendered with Poppler, and nine representative images
visually inspected alongside the native application: correct separate area gaps,
radial annular holes, rectangle guide sizes, 1/6–2/6–3/6 pie weights, shifted stacks,
and legible panel titles and guides. Both resolutions retain the same layout.

Visual inspection caught a gallery input mistake: inherited A/B/C groups produced
three full pies in one panel. The gallery and update fixture now explicitly select
one pie population. Regenerated artifacts were compared and inspected after the
correction; production arithmetic, reference fixtures and tolerances were unchanged.
An early comparison also ran before the final WASM PNG write completed; only the
successful post-completion comparison is retained as acceptance.

`mise run check` passes repository/dependency checks, formatting, native/Kit builds,
workspace all-target builds/Clippy, denied-warning rustdoc and WASM compilation.
Strict Python positives and its three intended negative diagnostics pass; the
TypeScript positive/negative consumer passes against the generated module. Command
logs, runtime hashes, actual native screenshot, external renders and visual inspection
record are in [integrated evidence](phase-2-shape-custom/integrated/). The earlier
standalone evidence remains historical. Full cross-family curved dash styling,
inspection acceptance inventory and cumulative G-SHAPE are WP-S08; final expanded
platform/performance/release qualification remains WP-21/22/23.

Final regression follow-up: the captured S07 source passes **414 macOS workspace
tests/doctests** and **413 Linux core/export/extension tests/doctests**, with zero
failed or ignored tests. Final Linux Python also passes all 72 corrected update
cases; its complete JSON records equal macOS Python and WASM exactly. These suites
ran on the recorded S07 snapshot before S08 dash changes. Logs are retained as
`macos-workspace-tests.log`, `linux-core-final.log` and `linux-updates-final.log`.

WP-S08 follow-up found a built-in generator regression beyond the registered-only
host cases: the temporary curve command cap remained on returned editable Paths.
The [integrated acceptance work](phase-2-shape-acceptance-2026-09-09.md) restores the
original caller edit budget only after successful bounded generation, with unchanged
host ownership fixtures and a new core regression. The S07 evidence above describes
its historical tested snapshot; the final shared-engine qualification belongs to S08.
