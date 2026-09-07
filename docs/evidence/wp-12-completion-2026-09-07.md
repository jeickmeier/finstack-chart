# WP-12 completion evidence — 7 September 2026

WP-12 is **DONE** for the facet, guide and shared-layout slice. Starting
revision was `a6fb2ea` (completed WP-11). This report and its evidence belong to the
WP-12 completion commit. The owner assignment remains WP-11 through WP-23, with a
separate commit after each completed package. WP-13 follows this package; G2–G4 remain open.

## Contract evidence

| Contract | Result and scope |
| --- | --- |
| GRA-07, FIX-06 | Explicit typed wrap/grid catalogs, stable panel keys, keep/drop policy, and mandatory broadcast/target declarations for missing facet fields. Independent source counts B=2/A=3, a separate-schema y=50 annotation and six grid cells pass. |
| GRA-03/08 | Group, facet and chart statistical scopes use the common compiler. Independent means are B groups `[100,120]`, A groups `[2,9]`, facet means `110` and `13/3`, chart mean `233/5`. Cache tests verify reuse and scope invalidation; filtered keys and dropped panels follow the effective source population. |
| SCL-05 | Shared domains union compatible panel contributions; free domains differ intentionally. Log training includes all positive panel populations. Default plot clips and explicit figure overflow are retained through scene and inspection metadata. |
| LAY-01 | One synchronized four-pass solver aligns margins/plots; font metrics and multiple sizes are tested. Native resize produces a meaningful compact state and recovers after enlargement. |
| LAY-02/03, WP-12 subset | Exact destination font measurement, facet headers, shared/per-panel color legends and incompatible-guide separation pass. Rich/rotated typography, full themes and publication furniture remain WP-13. |
| BND-01/03/04, extended proof scope | Seven new cases execute through actual Rust, Python and Node WASM. Each host rejects a missing facet policy and an unknown panel target with the stable documented errors. Source keys remain exact above 2^53. |

[The contract](../facet-layout-contract.md) and [ADR-005](../adr/005-foundational-scales-and-layout.md)
record defaults, limits, cache scope, clipping and fallback choices. The implementation
adds no dependency and creates no separate host/recipe statistics engine. Public
definitions gain optional facets, targeting/scope policies and semantic guide titles;
Rust struct-literal users must supply the new fields or use existing constructors.

## Commands and results

Working directory: `/Users/jeickmeier/Projects/finstack-chart`. Environment remains
macOS 26.5.2 arm64, Rust 1.97.1, CPython 3.14.6/PyO3 0.29.2, Node 24.14.0 and
wasm-bindgen 0.2.128. [Environment](wp-12/environment.txt) and
[module hashes](wp-12/module-sha256.json) identify the actual runtime artifacts.

| Command | Result |
| --- | --- |
| `mise run fmt` | PASS; [log](wp-12/fmt.log). |
| `mise run check` | PASS repository/target isolation, workspace builds, optional Kit, Clippy, rustdoc and core WASM. [Log](wp-12/check.log). |
| `mise run test` | PASS **142 tests**: 126 core, 13 export, 1 native conversion and 2 Rustdoc examples. Nine new focused facet tests. [Log](wp-12/test.log). |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-12` | PASS **31 cases in each real runtime**, including seven new facets and two additional negative FIX-06 checks per host. [Runner](wp-12/bindings.log), [comparison](wp-12/compare.log). |
| `mise exec -- cargo build -p chart-gallery --example family_gallery --locked` | PASS; [build](wp-12/native-build.log). The gallery selects facet fixtures with `--facets`; the temporary native app used a copied executable named `facet_gallery`. |
| Native and publication inspection | Seven native views, compact/recovery, representative PNGs and an actual rendered vector PDF inspected; see captures below. |

The three runtimes agree within **1e-12** for semantic values and **1e-10 points** for
final scenes, with **identical SVG bytes**. Existing WP-09–11 fixtures, assertions and
tolerances are retained. The new host negative tests initially compared Rust variant
names instead of stable wire codes, and the Node check initially referenced a font
outside its scope; the harness was corrected and the complete proof rerun passed.

The original small publication profile exercised a valid compact grid. A separate
explicit 600 × 400 point facet profile provides useful multi-panel publication evidence;
this does not replace a failing visual baseline. The nine core tests include independent
tiny-size/font cases and a 1e-12 log-domain tolerance for ordinary transcendental roundoff.

## Retained artifacts and visual findings

Native captures are JPEGs emitted by the existing capture service, named with their
actual format. Seven views were inspected at 1050 × 733 captured pixels:
[shared/broadcast](wp-12/native-ui/shared-broadcast.jpg),
[free/targeted](wp-12/native-ui/free-target.jpg),
[grid/empty](wp-12/native-ui/grid-empty.jpg),
[group summaries](wp-12/native-ui/group-summary.jpg),
[facet summaries](wp-12/native-ui/panel-summary.jpg),
[chart summary](wp-12/native-ui/chart-summary.jpg),
[shared log](wp-12/native-ui/shared-log.jpg).

The native grid was narrowed to a 700 × 553 capture, producing the explicit
[compact state](wp-12/native-ui/grid-compact.jpg), then enlarged to restore
[aligned useful plots](wp-12/native-ui/grid-restored.jpg). No invalid geometry, cross-panel
painting or lost empty-panel identity appeared. Independent y scales include only their
own contributions; the annotation therefore extends A's free domain to 50 and is absent
from B. Shared log scales have matching guide positions. Exact-domain endpoint symbols
intentionally clip at plot boundaries; authors may request padding.

Inspected exports include the [grid PNG](wp-12/native/statistics-facet-grid-empty.png),
[facet summary PNG](wp-12/native/statistics-facet-panel-summary.png), and
[rendered grid PDF](wp-12/grid-pdf-render.png). All seven new SVG/PNG/PDF exports are
retained in the neighboring native artifact directory. PDF image-object checks are empty.
[Facet semantics](wp-12/facet-semantics.json) and [destination scenes](wp-12/facet-scenes.json)
retain the new case results from every runtime; complete 31-case working artifacts remain
at `artifacts/wp-12/` and are reproducible using the recorded command.

## Remaining gates

WP-13 owns full themes, rich typography and publication composition; WP-14 owns external
extension and alpha API closure. Complete selection/navigation, accessibility, streaming,
bounded scheduling and live-export integration remain WP-15–20. No Linux execution,
screen-reader certification, release packaging or performance certification is claimed.
Existing dependency advisories and the `block` future-compiler warning remain unresolved.
