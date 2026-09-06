# WP-03 completion and G0 capability report

Date: 6 September 2026. Working directory: `/Users/jeickmeier/Projects/finstack-chart`.
Starting revision: `435e127` (WP-02 committed). Result: that revision plus the uncommitted
WP-03 slice. Assignment: native, font and export capability spike only. No chart compiler,
data transaction implementation, public renderer/exporter API or binding runtime was added.

**WP-03 is DONE; G0 architecture/capability evidence is established for the initial macOS
host.** This is a feasibility gate. Full requirement/FIX coverage and G1–G4 remain open.
The dependency advisory scan still fails; this is not release approval.

## Changes and decision basis

- A [shared retained fixture](../../fixtures/capability/scene.svg) and explicit font
  resources feed the actual [GPUI proof](../../examples/chart-gallery/examples/capability_spike.rs)
  and [headless export proof](../../crates/chart-export/examples/capability_export.rs).
  The resource loader exercises WP-02 identities/revisions/budget/provider contracts.
  Extended styles use a temporary usvg tree; integration with the normalized core Scene
  and real chart layout remains WP-07/08. No second chart semantic engine was introduced.
- [ADR-003](../adr/003-font-and-renderer-capability-route.md) selects the font/vector routes,
  records exact candidate identities and limitations, and keeps candidates in dev dependencies.
  Core has no dependencies; normal export still depends only on core.
- [ADR-008](../adr/008-benchmark-protocol.md) records the reference environment, separate
  timing/accounting boundaries, all five unchanged PERF workloads and a measured starting profile.
- The macOS `check` task now builds and lints the Kit-gated capability example in addition
  to the standalone/Kit dependency examples. The new artifact checker has an actual
  runner and independent dimensional/font/vector expectations.

## G0 evidence

| G0 component | Evidence / disposition |
| --- | --- |
| Crate/dependency boundaries | WP-01/02 accepted reports; refreshed `mise run check`, three-target isolation checks, one pinned GPUI identity and optional Kit remain intact. |
| Authoring/normalization and defaults | ADR-002 retains typed authoring lowered to shared portable contracts; specification defaults remain normative and unchanged. Ergonomic signatures/compiler and algorithm execution remain their planned WP-05/06/10 work. |
| Native primitive/font feasibility | Actual release GPUI window, vector paths/glyph outlines, clips, gradient and Kit controls inspected; native resource bytes explicitly loaded. |
| Publication/font feasibility | Actual SVG/PDF text and outline modes, plus 300/600 DPI PNG inspected; physical dimensions and fonts independently checked. |
| Decisions, target matrix and starting profile | ADR-001/002/003/008/010 and updated support matrix. Initial platform is macOS Apple Silicon; other runtime/platform gates remain unverified. |

## Native and visual verification

[Corrected native release screenshot](wp-03/native-release.png),
[smaller viewport](wp-03/native-resized.png),
[initial defect screenshot](wp-03/native-before-fixes.png).

The first screenshot exposed two adapter defects: text group transforms were missing,
leaving the rotated label horizontal, and even-odd filling left holes at stroke joins.
The fixes accumulate local group transforms and request nonzero fills. The corrected
release screenshot was inspected against the publication artifacts: rotation, all three
cap/join probes, curve/dash clip boundaries, gradient direction, point shapes, accent/minus
glyphs and rich runs are present. No visual baseline was replaced to conceal a failure;
the initial screenshot is retained with this explanation.

The shared figure uses a white page, explicit font resources and a small static input.
It does not establish tight-bound diagnostics, complete themes/multi-panel layout,
arbitrary clip/gradient support, all Unicode scripts or production high-DPI tolerances.
Resize preserves the fixture aspect ratio; it is not a chart label-reflow test.

Actual input procedure, using native accessibility/keyboard automation:

1. Focus the input, paste `café Ω 12.5%`, press Tab then Return. Both the editable value
   and applied status contain the complete Unicode text. The same Apply button also
   worked by pointer in the debug run. Synthetic character typing omitted `é` in one
   automation attempt; Unicode paste succeeded, so no IME/keyboard-layout claim follows.
2. Tab to Remount, activate with Return, repeat to generation 5. The input resets while
   the applied status remains. Two frames after each remount the weak old input handle
   reports `old_input_released=true` in the [release trace](wp-03/native-release.txt).
   An immediate debug check had reported false because a frame still retained the entity.
3. Resize the window; inspect the smaller scene. Close with the native window control.
   The trace records window closure and `Proof` view drop at generation 5; process exits 0.
   A second launch ran the bounded native profile and also closed/dropped successfully.

[Actual macOS accessibility tree](wp-03/native-accessibility.txt) exposes a settable text
field, two named buttons, applied-status container and descriptive image node. The
[profile run tree](wp-03/native-profile-accessibility.txt) also exposes these hooks after a
full refresh. The inspector reported the window as its focused element, so per-control
focus announcements have **not** been certified. No VoiceOver session, meaningful chart
point navigation or accessible data table was implemented. GPU-03's full acceptance
remains WP-17/21; keyboard success alone is not screen-reader support.

## Publication artifacts

| Artifact | Actual inspection |
| --- | --- |
| [Text SVG](wp-03/capability-text.svg) | Browser rendering inspected; text nodes preserved; CSS embeds exact supplied TTF bytes. [Browser screenshot](wp-03/svg-text-browser.png). |
| [Outline SVG](wp-03/capability-outline.svg) | Browser rendering inspected; 29 path nodes / 8,550 outline commands including clip; no text/image nodes. [Browser screenshot](wp-03/svg-outline-browser.png). |
| [Text PDF](wp-03/capability-text.pdf) | Poppler rendering inspected; two subset-embedded CID TrueType fonts with Unicode maps; extracted title/rotation/Greek/numeric text present. [Preview](wp-03/pdf-text-preview.png), [extracted text](wp-03/pdf-extracted-text.txt). |
| [Outline PDF](wp-03/capability-outline.pdf) | Poppler rendering inspected; no font objects or searchable text, no image objects. [Preview](wp-03/pdf-outline-preview.png). |
| [300 DPI PNG](wp-03/capability-300.png) | Inspected: 2126 × 1417 pixels; pHYs 11811 pixels/metre. |
| [600 DPI PNG](wp-03/capability-600.png) | Inspected: 4252 × 2835 pixels; pHYs 23622 pixels/metre. |

The final PNGs explicitly fill the white background through the fractional border caused
by integer pixel rounding, avoiding a premultiplied-alpha edge in the encoded output.
Both final resolutions were reinspected after that correction.

Both SVGs explicitly declare 180 × 120 mm. Both PDFs measure 510.236 × 340.157 points
in Poppler (within 0.002 points of independent expectations) and contain **zero raster
images**. PNG uses uniform physical scaling plus integer size rounding. The comparison
is visual/structural, not a pixel-identical renderer certification. Text extraction can
normalize accent sequences. These are ordinary RGB PDF files, not certified PDF/A or
tagged/press-ready PDFs.

The [artifact checker](../../scripts/check_capability_artifacts.py) verifies these sizes,
SVG structures and embedded font hashes, PDF image/font/text policy and PNG density.
[Result](wp-03/artifact-check.txt): all four artifact groups passed. Its initial authoring
mistake expected seven source paths; the fixture has exactly six authored path elements
(two curves, three joins, one rule). That count was corrected before acceptance; output
or visual expectations were not changed to mask a rendering failure.

## Commands, environment and results

Environment: Apple M5 Max / 18 cores / 128 GiB; macOS 26.5.2 (25F84), arm64;
Rust 1.97.1; Python 3.14.6; mise 2026.8.3; cargo-deny 0.19.8; Xcode 26.6 / SDK 26.5.
Native scale factor 2; actual canvas 1168 × 697 logical pixels, fit figure width 1045.5.
Full reference workload dimensions and measurement limitations are in ADR-008.

Commands ran from the working directory above:

```sh
mise run fmt
mise run check
mise run test
mise exec -- cargo run -p chart-export --example capability_export --release --locked -- docs/evidence/wp-03
mise exec -- python scripts/check_capability_artifacts.py
mise exec -- cargo build -p chart-gallery --example capability_spike --features kit --release --locked
mise exec -- cargo deny --locked check advisories
pdftoppm -scale-to 1500 -png -singlefile docs/evidence/wp-03/capability-text.pdf docs/evidence/wp-03/pdf-text-preview
pdftoppm -scale-to 1500 -png -singlefile docs/evidence/wp-03/capability-outline.pdf docs/evidence/wp-03/pdf-outline-preview
pdftotext -layout docs/evidence/wp-03/capability-text.pdf docs/evidence/wp-03/pdf-extracted-text.txt
```

The built executable was launched from a temporary inspection `.app` wrapper with stdout/
stderr retained; normal invocation and wrapper details are in the fixture README. It ran
once interactively and once with `--profile`. A loopback-only Python HTTP server served
the SVG files for browser inspection; server and inspection windows/tabs were closed.

| Check | Result |
| --- | --- |
| [Repository/build/lint/docs/WASM check](wp-03/check.txt) | PASS, including explicit Kit capability-example build and strict lint. The first run's nested-if lint was corrected. |
| [Workspace tests](wp-03/test.txt) | PASS: 21 core integration tests and 1 core doctest; shell packages still have zero semantic tests. This is not exporter/binding feature certification. |
| [Release native build](wp-03/native-build.txt) | PASS; actual runtime/visual/input evidence above is additional to compilation. |
| [Publication structure check](wp-03/artifact-check.txt) | PASS: SVG/PDF text + outline and both PNG resolutions. |
| Missing-glyph negative probe | PASS: resource 0 / revision 0 rejects U+10FFFF before silently accepting a glyph substitute; recorded in export trace. General fallback diagnostics remain unimplemented. |
| [Starting native profile](wp-03/native-profile.txt) | Samples 11–40: p50 3.587 ms, p95 3.637 ms for conversion/tessellation/submission only. No PERF gate claimed. |
| [Headless profile](wp-03/export-profile.txt) | Prepared-tree text-PDF conversion after 10 warm-up calls, n=30: p50 0.367 ms, p95 0.491 ms. Cold and raster timings retained separately. |
| [Advisory scan](wp-03/advisories.txt) | FAIL: same six unmaintained packages; no ignore entries added. No safe upgrade reported. `block` 0.1.6 future-compiler warning also remains. |

## Requirement and next-work disposition

ARC-04's capability/dependency selection is supported for this spike. LAY-02/04,
SCN-03, GPU-01/03, EXP-01/02 and QLT-03/04 have the concrete WP-03 proof subset above;
none is declared fully implemented by a static fixture. No full canonical FIX or PERF
case passes here. The [ADR capability matrix](../adr/003-font-and-renderer-capability-route.md)
records unsupported capabilities and implementation paths without changing the specification.

**Next ready package: WP-04.** WP-07 and WP-08 now have their WP-03 prerequisite, but
still require WP-06. Native/core Scene integration, destination measurement bridges,
complete resource diagnostics, actual chart semantics, snapshot export and Python/WASM
runtime proofs remain planned work. Linux execution/hosted CI and release accessibility
are unverified. WP-08 must revisit font-dependency maintenance before production promotion;
WP-23 owns final release disposition of all tracked advisory/compiler risks.
