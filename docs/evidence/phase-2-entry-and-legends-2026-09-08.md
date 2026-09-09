# Phase 2 entry and shared legend acceptance

Date: 8 September 2026. Baseline: `fab2505951061eaafe9adb52c86b248ee0dfa6bf` plus
the previously authored Phase 2 plan/review edits. Result: uncommitted P2-00/GG-01 slice.
Requirements: GG2-01/04/12, ARC-03/04, BND-01, GRA-07, SCL-05, LAY-03, THM-03,
QLT-02/05. Fixtures: FIX-GG01 and existing authoring/facet regressions.

## Verified handoff and entry contract

The predecessor task Review API simplicity has completed turns without turn errors.
Git HEAD `fab2505`, dated 23:11:16 UTC, contains the primary Rust/Python/WASM API and
[completion report](primary-authoring-completion-2026-09-08.md). An ancestry check
passed. Recent task-reader turns contained no final message; verification therefore
used the completed state and committed implementation/report, not an assumed commit
from an idle status. The waiting heartbeat was paused after verification and deleted
after the authorized handoff was fulfilled.

AP-00–08 are delivered; AP-09 performance qualification remains open. P2-00 completes
its entry contract with [ADR-014](../adr/014-phase-2-integration-contract.md) and the
[coverage register](../phase-2-coverage.md). It links eight D3 inventories and all
twenty GG packages to existing AP routes, requirements, fixtures, surfaces and gaps.
New profile semantics/provenance belong to the canonical definition; the first
delivering semantic package implements compatible migration. Reference workspace
layout and ownership are fixed, but concrete locks/oracles remain with lane entries
and GG-00. No runtime profile or reference dependency was added here.

## Reproduced defects and fixes

1. A primary discrete scale with an explicitly empty domain produced no entries but
   still painted `Series` and reserved guide width. Before the fix, the independent
   empty-guide assertion observed one title instead of zero. Shared guide collection
   now ignores zero-entry metadata for layout; prepared metadata remains available.
2. `legend().untitled()` stored `None`, which the painter interprets as the generic
   `Color` fallback. After correcting the empty-guide case, the untitled case failed
   with one `Color` title instead of zero. Primary builds and edits now store an
   explicit empty title; shared painting skips that title row. Legacy absent titles
   keep their fallback, and nonempty titles keep their existing behavior.

The existing single/faceted painter is retained. The change adds no alternate guide
engine, wire field, envelope version or dependency. `generic_title()` explicitly
selects the legacy Color/Value fallback, distinct from an inferred field title and
from omission. Shared host dispatch and Python/TypeScript declarations expose it.
The full suite found baseline primary fixture authors relying on the old `untitled()`
behavior; those authors now use `generic_title()`. Independent expected fixtures and
comparators remain unchanged. The portable
contract and changelog document empty versus absent titles. The primary interchange
round trip retains the explicit omission.

## FIX-GG01 matrix and independent expectations

[Rust acceptance](../../crates/chart-export/tests/ggplot_legends.rs) runs eight cases
in ordinary, one-panel collected-facet and one-panel local-guide layouts: 24 cases.
The single-panel scatter and one-panel facet control therefore share all edge cases.

| Case | Expected behavior | Result |
| --- | --- | --- |
| Two-entry scatter/control | One Series title, Alpha/Beta once, exact blue/red swatches | PASS |
| Empty guide | No title, keys or reserved furniture; source marks remain | PASS |
| Hidden guide | No guide furniture after the typed visibility action; marks remain | PASS |
| Tight 130 × 65 point layout | Bounded swatches/clips and explicit Legend pressure for clipping/omission | PASS |
| Shared compatible guide | Two layers retain four source marks but one title/key set | PASS |
| Incompatible guides | Separate Series/Other titles and opposite ordered palettes | PASS |
| Untitled build | Keys remain; neither Series nor fallback Color is painted | PASS |
| Untitled immutable edit | Same omission survives primary serialization/loading | PASS |

Tests check exact label counts, swatch colors/order, clip containment, source targets,
empty guide targets and one target list per core scene item. PNGs are fully decoded.
The full existing facet suite retains its stronger same-ID/incompatible-palette case.
The export scene adds a background before core items; target assertions use the core
layout indices, and host comparisons use the properly aligned portable scene.

[Python](../../scripts/bindings/ggplot_legends.py) and
[WASM](../../scripts/bindings/ggplot_legends.cjs) execute all 24 saved primary inputs
through freshly built adapters. Each host's complete portable scene is exactly equal
to Rust, including identities, targets, fonts, panels and diagnostics; no numeric
tolerance or identity normalization was needed for these fixtures. Each also exercises
six host-dispatched untitled/generic edits. Each runtime encodes 24 SVG/PDF/PNG triplets:
216 artifacts across Rust/Python/WASM. Explicit disposal/free follows the existing API.

## Commands and artifacts

Environment: Darwin arm64, mise Rust 1.97.1 and Python 3.14.6, Node 24.14.0,
wasm-bindgen CLI 0.2.128; the repository's supplied Noto Sans font and existing locked
encoders. No dependency installation was required. Source identities and retained
artifact digests are in [the evidence directory](phase-2-legends/README.md).

Commands from repository root (the test artifact directory must be absolute):

```sh
mise exec -- python3 scripts/check_repository.py
git merge-base --is-ancestor fab2505951061eaafe9adb52c86b248ee0dfa6bf HEAD
mise run fmt
GG_LEGEND_ARTIFACTS="$PWD/target/ggplot-legends/native" mise exec -- cargo test -p chart-export --test ggplot_legends --test authoring --locked
WASM_BINDGEN=/private/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen TSC_JS=/Users/jeickmeier/.npm/_npx/e04ecd76da0b5726/node_modules/typescript/lib/tsc.js PYTHONPATH=/Users/jeickmeier/.cache/uv/archive-v0/PKSDgIDzoTfeXd75NY8Rf/lib/python3.14/site-packages mise exec -- python3 scripts/run_primary_authoring_proofs.py target/ggplot-legends/primary
mise exec -- python3 scripts/bindings/ggplot_legends.py target/ggplot-legends/primary/python-module target/ggplot-legends/native target/ggplot-legends/python
mise exec -- node scripts/bindings/ggplot_legends.cjs target/ggplot-legends/primary/wasm-module target/ggplot-legends/native target/ggplot-legends/wasm
mise run check
mise run test
```

The full primary runner builds fresh extension-proof modules before both host runs.
TypeScript 6.0.2 and mypy 2.3.0 came from existing local tool installations identified
above; these absolute paths describe this execution environment, not new repository
dependencies. Positive declarations pass; the five expected Python errors and the
TypeScript expected-error assertions pass their negative contract checks. All 34
component families, 23 actions, 47 input transitions and 70 streaming steps pass
the independent three-host comparison. Focused Rust acceptance passes six tests,
including the new test's complete 24-case matrix.

Final `mise run check` passes repository/dependency/build/lint/docs/WASM checks.
Final `mise run test` passes **253 tests, zero failed and zero ignored** on Darwin
arm64. The existing dependency `block 0.1.6` emits a future-incompatibility notice;
neither command reports a failing gate. Initial check/test logs are retained with the
final results; the initial full test failure was the fixture-author migration described
above, resolved without changing its independent expectations.

All 24 Rust PDFs were independently rendered using `pdftoppm -r 96 -singlefile -png`
and their text extracted with `pdftotext`. Six labeled sheets show all 24 direct PNG
and 24 PDF-rendered results; every sheet was visually inspected. Titles, ordering,
missing furniture and swatch colors agree. Tight cases intentionally clip Alpha and
omit trailing content, matching the tested pressure diagnostic. SVG scene/encoding
checks and its PNG rendering retain the same guide geometry and logical text.
The final rerun's 48 input/scene files and all 72 publication artifacts are byte-equal
to the retained inspected set; no visual baseline was replaced.

The first inspection helper found no Pillow in mise Python. The bundled artifact
Python supplied Pillow to assemble the inspection sheets; no runtime dependency was
added. Initial test-authoring compile errors and the export-background index offset
were corrected before the two behavioral failures above were recorded.

## Acceptance boundary

P2-00 passes its contract/register scope. GG-01 passes its specified shared legend
scene and inspected SVG/PDF/PNG acceptance, plus actual Python/WASM execution.
This is not full ggplot2 guide parity: colorbars, multi-aesthetic keys, advanced
collection and placement remain GG-05 and its D3 prerequisites. No ggplot2 R oracle,
fresh native-window interaction/visual run, Linux runtime or sustained performance
measurement ran in this slice. Existing native ownership remains unchanged; native
and expanded platform/performance qualification is not inferred from headless scenes.
AP-09, all D3 gates, G-GGPLOT, G-PARITY and expanded G4 remain open.

Next reviewable package: GG-00's reference inventory/oracle or the earliest independent
D3 entry under the combined plan. WP-S01 still waits for WP-P04; shared kernel work is
counted once and reference locks use ADR-014's common workspace.
