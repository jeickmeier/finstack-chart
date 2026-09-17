# GG14–18 final integration evidence

Revision `295e7a0ec2e9145b140c838751b615ee40e9f64a` plus the shared working tree, macOS arm64, Rust 1.97.1, Python 3.14.6, Node 24.14.0 and wasm-bindgen 0.2.128. This report consolidates the last implementation wave in the authorized GG06–18 assignment. Earlier accepted packages retain their dated reports. GG06–16 are complete. Prescribed workspace and repository checks are complete through the recorded resumed runs. GG17 remains open on Windows EMF playback, and GG18 retains that prerequisite; GG19 certification is outside this assignment.

## Independent authors and retained publications

| Package | Cases and contract | Identical files per host |
| --- | --- | ---: |
| GG14 themes | 16 authors; nine presets, full hierarchy, measured controls, multilingual text; extra Preserve/Outline at 300/600 DPI | 60 |
| GG14 math | Nine supplied-font authors; every consumer, rotated Latin/Greek/Cyrillic formulas | 27 |
| GG14 furniture | Eight title/caption/tag cases, panel/plot/margin placement | 24 |
| GG15 geography | 19 authors, explicit CRS/projections, holes/dateline, axes/graticules/overlays; 300 DPI case | 57 |
| GG16 extensions | Ten external authors, including models and typed facet labeller context | 30 |
| GG17 devices | Raster/vector/EMF/PicTeX, four advanced TIFF codecs and retained PDF/PS/TIFF pages at two DPIs | 30 |
| GG18 integration | Four combined authors, including geographic custom models | 12 |
| Total | Independently authored Rust/Python/Node WASM output | 240 |

These are exact byte comparisons for publications and structural comparisons for scene JSONs, not screenshots used as numerical oracles. Evidence roots are `/private/tmp/finstack-chart-proof-20260916/gg14-16-18-final`, `gg15-16-18-final2` and `gg17-final`. Their comparison manifests and command logs identify each scope. SVG/PDF/PNG publications were independently rendered and inspected; native galleries were inspected with presentation stamps and diagnostics. Device inspection uses independent ImageIO, libtiff, Ghostscript, Poppler, Tectonic and official libwebp decoders as appropriate. EMF currently has independent binary-record validation and cross-host byte equality, but lacks Windows playback.

## Correctness and compatibility findings

- Projection kernels now use pinned libm transcendentals so UTM outputs agree across native and WASM without destination rounding.
- Registered coordinate bounds enclose hidden nonlinear extrema; forward-only extensions reject inverse operations explicitly.
- Registered model and facet-labeller examples exercise actual callbacks, typed context, malformed output, native-only rejection, replay and retained registries.
- LibraryV1 confidence ribbons no longer emit an invalid zero-width outline through independent-paint conversion. The filled geometry is preserved.
- The legacy binned candidate API retains repeated constant break candidates. More recent guide validation no longer rejects this previously accepted contract.
- Frozen numeric Auto/affine-secondary envelopes remain version 1; explicit temporal capabilities retain their newer version detection.
- Custom-device callbacks obey the smaller of caller and captured-profile output budgets. Host adapters clean up temporary options if a later option setter fails.
- Legacy binding proof packaging now copies the same complete WASM adapter file set as the primary proof builder.

Five Rust integration tests cover exact append/upsert/remove/retention versus batch, source filtering versus coordinate/state zoom, aggregate and derived selection, old immutable captures, rollback, alternate profiles and geographic custom models. Python/WASM independently select and hover real targets, present one view, change source/definition/view, and verify all Presented/Current × Visible/FullDomain × interaction combinations after owner disposal. Four focused save/page tests verify dimensions, explicit filesystem policy, callback errors/budgets and single-page PDF byte preservation.

Math layout intentionally uses supplied-font design metrics. Independent signed glyph metrics explain the observed Cairo/device differences in compound width, dot accents and integral placement; no broad tolerance or reference fixture was weakened. The detailed formulas and source generators are recorded in [math evidence](gg14-math-qualification-2026-09-16.md).

## Validation and remaining gates

The actual legacy Rust/Python/WASM WP09–20 runner and its cross-host comparison pass after the envelope correction. Its final path step initially lacked a copied adapter; after fixing that packaging owner, the resumed Node path proof and comparison pass all 86 operation traces. Logs: `/private/tmp/gg06-18-legacy2.log`, `/private/tmp/gg06-18-legacy-path-resume.log` and `/private/tmp/gg06-18-legacy-path-compare.log`.

Focused all-target Clippy for core/export/text/extensions/Python/WASM passes. Repository isolation checks cover macOS, Linux and wasm32 dependency graphs; license/source checks pass. Repository checks are complete: dependency isolation, licenses/sources, formatting, native Kit examples, workspace all-target compilation/Clippy, strict rustdoc and wasm32 core compile. The initial `mise run check` passed its earlier stages, then exposed feature-dependent enum sizes; boxing internal legend payloads resolved those warnings. `/private/tmp/gg06-18-check-resume2.log` records every remaining prescribed command passing (exit 0). This is resumed completion, not one uninterrupted successful command. Workspace execution also completed through resumed commands (exit 0): `/private/tmp/gg06-18-tests-resume.sh` and `/private/tmp/gg06-18-tests-resume.log`. The earlier `/private/tmp/gg06-18-final-workspace2.log` passed 146 targets before reaching an obsolete portable-envelope test binary compiled while its fix was in flight. The resumed run covers portable and every subsequent core integration target, all other workspace packages, and core doctests. Forty freshly compiled focused tests additionally cover final theme, legend, binned-guide, portable-envelope, integration, math and save/page changes. This records aggregate coverage with targeted revalidation, not an uninterrupted successful `mise run test` invocation.

Fresh primary authors, 34 semantic/scene families, 70-step lifecycle replay and strict declarations pass. The optional broader runner was stopped after 71 completed commands; its remaining accepted scale/D3 matrices were not required to repeat for this assignment. [The precise partial scope](primary-regression-scope-2026-09-16.md) records completed cases and the interrupted WASM callback stage without claiming full cumulative or GG19 acceptance.

The [device matrix](gg17-device-platform-matrix-2026-09-16.md) distinguishes portable implementations from executed platforms. Windows EMF playback is unexecuted here. The independent [Windows verifier](../../tools/reference/windows/verify-emf.ps1) and its [instructions](../../tools/reference/windows/README.md) are ready. GG17 remains open on that named platform evidence, and GG18 retains its GG17 prerequisite. No fresh Linux execution, Windows GPUI product, broad performance certification or GG19 global parity certification is claimed.

Native public-API interaction passes in `/private/tmp/gg18-programmatic-native2.log` (`integration-programmatic-PASS`): exact source selection, derived hover, viewport change, presented inspection and `capture_presented` agree after repaint. The final native image was inspected. Physical pointer automation was not executed successfully and is not claimed.

The [final manifest](phase-2-ggplot-final-wave-2026-09-16.json) fingerprints changed sources, all 240 publications per host and validation logs. Next action: run the prepared Windows EMF verifier and inspect its playback outputs; if successful, record that platform evidence to close GG17 and the dependent GG18 gate. GG19 still requires the separate D3 and global certification prerequisites.
