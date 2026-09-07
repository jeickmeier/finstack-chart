# WP-21 — Original correctness, fidelity and supported-platform hardening

Status: DONE for the owner's original WP-11–23 assignment. Parent: `9d86ef7`.
Completed 7 September 2026; original WP-14/17/20 prerequisites satisfied. Expanded
D3/ggplot2/primary-authoring acceptance remains separate and open. No feature fixtures
or numerical tolerances were weakened. Concurrent planning documents are preserved.

## Changes and reproduced regressions

Frozen native charts now reproject the exact retained prepared object, font and painter
resources into new bounds. The core acknowledges a new frozen layout only after painting,
requires the same prepared object and an advancing layout revision, and preserves gesture
and pin bases. Later data/theme changes become visible only on resume. The new focused
core regression checks source/state identity, changed bounds, monotone layout revisions,
foreign-object rejection and old-projection release.

The previous intermittent redraw observation was reproduced in the four-chart scheduling
example with `--isolated-redraw`: explicit parent observations and periodic parent
notifications are disabled. In the [before trace](wp21/native-redraw-before.jsonl), all
workers finish source revision 160 with zero pending/active jobs while all painted scenes
remain at revision 148 through ten drain observations. Native painting needed an explicit
platform frame demand. Accepted or failed background completion now coalesces one frame
request using the mounted window; its callback retains only a weak chart entity. Window
activation also refreshes the chart. No repeating animation or polling loop was added.

Two final runs reach painted revision 160 on every chart with zero lag, advancing scenes,
compatible stale rejection, bounded slots and disposal. The [first trace](wp21/native-redraw-after.jsonl),
[first oracle](wp21/native-redraw-check.json), [repeat trace](wp21/native-redraw-repeat.jsonl)
and [repeat oracle](wp21/native-redraw-repeat-check.json) retain evidence. This establishes
the tested idle-frame regression boundary; it is not an upstream GPUI concurrency proof.

## Fixture acceptance index

All rows below pass for the original feature scope. Prior inspected artifacts remain
valid; the final full runtime replay rechecks numerical/scene/output contracts. Native
surfaces are macOS arm64; portable semantics additionally execute in Python/Node WASM,
and this package adds Linux headless Rust execution and cross-host comparison.

| Fixture | Passing evidence |
| --- | --- |
| FIX-01/02 | [WP-10 independent explicit/automatic histograms](wp-10-completion-2026-09-06.md); current 36-case three-host replay |
| FIX-03/04/05 | [WP-10 summary/quantile/OLS population and zoom/filter tests](wp-10-completion-2026-09-06.md); independent expected values in current comparison |
| FIX-06 | [WP-12 facet/broadcast/shared-domain artifacts](wp-12-completion-2026-09-07.md); current fixture runner rejects missing facet policy/unknown target |
| FIX-07 | [WP-06 finite/fallback/time tests](wp-06-completion-2026-09-06.md), [WP-11 nonlinear/session/family tests and images](wp-11-completion-2026-09-07.md); final core/host replay |
| FIX-08 | [WP-18 atomic transaction and retention replay](wp-18-completion-2026-09-07.md); 70 stream steps per host and final core suite |
| FIX-09 | [WP-15 stable selection/pin contracts](wp-15-completion-2026-09-07.md), [WP-18 reorder/correction/eviction reconciliation](wp-18-completion-2026-09-07.md); current actions/stream replay |
| FIX-10 | [WP-16 gesture/input evidence](wp-16-completion-2026-09-07.md), [WP-17 links/editing/keyboard/capture-loss evidence](wp-17-completion-2026-09-07.md); 47 input steps per host |
| FIX-11 | [WP-19 deterministic slow/stale scheduling](wp-19-completion-2026-09-07.md); reproduced native stall and two passing corrected runs above |
| FIX-12 | [WP-13 editorial/terminal/grayscale native/export inspection](wp-13-completion-2026-09-07.md); current composition/theme replay |
| FIX-13 | [WP-13 physical figure, font/rich text/clip/gradient/vector/raster evidence](wp-13-completion-2026-09-07.md); current exact dimensions and PDF image/font checks |
| FIX-14 | [WP-20 coherent live exports and cancellation/release](wp-20-completion-2026-09-07.md); 40 current three-host steps, independent geometry and immutable bytes |
| FIX-15/16 | [WP-09 exact identity/time/null/memory/disposal contract](wp-09-completion-2026-09-06.md); final full three-host runner and Linux-to-host comparison |
| FIX-17 | [WP-14 registered portable extension and unsupported native-only export](wp-14-completion-2026-09-07.md); final five external-extension tests and actual registered host fixtures |
| FIX-18 | New core frozen-resize regression; [native trace](wp21/native-hardening.jsonl), [independent checker result](wp21/native-hardening-check.json), inspected stages below; 20 old entities released |

## Executed checks and environments

| Command/run | Result |
| --- | --- |
| `mise run fmt`, `mise run check` | PASS; [final macOS check log](wp21/macos-check.log), includes native/Kit checks, Clippy, rustdoc, dependency isolation and WASM core target |
| `mise run test` | PASS **218 tests**: 175 core, 33 export, five external-extension, one native conversion, four Rustdoc; [log](wp21/macos-test.log) |
| `WASM_BINDGEN=/tmp/wp09-tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen mise run bindings-proof artifacts/wp-21` | PASS actual 36 cases, 23 actions, 47 input steps, 70 streaming steps, three density fixtures, 40 live-export steps in Rust/CPython/Node WASM; [log](wp21/macos-bindings.log) |
| Linux `cargo test -p chart-core -p chart-export -p chart-text --locked --offline` | PASS **212 tests**, same 175 core/33 export/four Rustdoc; [log](wp21/linux-test.log) |
| Linux `cargo check ... --all-targets --locked --offline`, warnings-denied rustdoc, `cargo run -p chart-export --example binding_proof --locked --offline` | PASS; [environment/check/full fixture log](wp21/linux-check.log) |
| `python scripts/bindings/compare.py /tmp/wp21-linux/compare` | PASS Linux Rust versus macOS Python/WASM using the unchanged independent oracle; semantic 1e-12, scene 1e-10 pt, SVG byte equality, dimensions/vector/font checks; [log](wp21/linux-cross-host-compare.log) |
| `python scripts/check_native_hardening_trace.py docs/evidence/wp21/native-hardening.jsonl` | PASS original data/theme through 698→478→10→798 logical-pixel widths, later-source resume, malformed definition/font diagnostics and 20 releases |
| `python scripts/check_scheduling_trace.py ...` | PASS both final native redraw runs, 160 commits per chart, zero final lag/active/pending and disposed workers |

macOS: 26.5.2 arm64, Rust 1.97.1, CPython 3.14.6/PyO3 0.29.2,
Node 24.14.0/wasm-bindgen 0.2.128; supplied fixture fonts. Linux: local Docker
`rust:1.97.1-bookworm`, aarch64, offline read-only source/registry mounts, output in
`/tmp/wp21-linux`. This is not Linux GPUI or remote CI execution. CI now includes actual
portable runtime jobs in addition to build/test jobs; those remote jobs were not run here.

Final changes after the portable replay affect native frame demand, native examples,
CI and documentation only. Final native build/check/test and actual regression runs
cover them. [Linux artifact hashes](wp21/linux-artifacts-sha256.json) and
[macOS artifact hashes](wp21/macos-artifacts-sha256.json) identify locally retained outputs
under `/tmp/wp21-linux/bindings` and `artifacts/wp-21`; source fixtures/runners reproduce them.

## Inspected native evidence

The actual [hardening gallery](../../examples/chart-gallery/examples/hardening_gallery.rs)
exercised the frozen figure at [480×240](wp21/native-ui/frozen-resized.png),
[tiny bounds](wp21/native-ui/tiny-frozen.png) and
[restored 800×400](wp21/native-ui/frozen-restored.png). Axes/marks fit the new destination;
tiny bounds produce the declared limited scene. [Resume](wp21/native-ui/resumed.png)
shows the latest ±999 observations and updated pink background. The
[malformed replacement](wp21/native-ui/malformed-retained.png) retains the valid picture
and displays an actionable diagnostic; [recovery](wp21/native-ui/recovered.png) removes it.
[Lifecycle completion](wp21/native-ui/lifecycle-complete.png) shows 20 released old entities.
Manual captures have their own [stepped trace](wp21/native-hardening-stepped.jsonl).
The [final redraw image](wp21/native-ui/redraw-complete.png) shows all four charts at 160.
All linked images were visually inspected, rather than inferred from file creation.

## Remaining acceptance boundaries

Original FIX-01–18 hardening and G3 pass. [ADR-009](../adr/009-supported-platform-and-accessibility.md)
and the refreshed [support matrix](../support-matrix.md) state exact platform/accessibility
limits, including unverified VoiceOver and OS focus exposure. Weak entity release is a
lifetime check, not an RSS plateau. Sustained PERF-01–05 remains WP-22. Expanded parity,
primary-authoring and G4 release acceptance remain open, as do the previously recorded
unmaintained-dependency and upstream `block` future-compiler risks. Next: original WP-22.
