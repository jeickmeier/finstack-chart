# CLR-04 authored colors and shared paint integration

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 changes.
[Source hashes](phase-2-paint/source-hashes.json) identify this qualified snapshot.
CLR-04 is COMPLETE for COL-05, BND-01/03/04, THM-01/02/03 and SCN-03 package scope.
CLR-05 and G-COLOR remain OPEN pending SP-04 integration and measured acceptance.

`Paint` retains either strict legacy byte colors or a versioned floating color value.
CSS and all five color spaces reach constant marks, theme tokens, palettes, annotations,
rich text, candle directions, gradient stops, host/output overrides and backgrounds.
One core lowering boundary produces scene sRGB8. Floating palette endpoints interpolate
before quantization; legacy byte palettes retain their previous behavior. All three
hosts delegate conversion and lowering to Rust. [ADR-016](../adr/016-color-values-and-paint-boundary.md)
records plot definition v4, portable profile v2 and live override v2 compatibility.

The shared exceptional-number codec now preserves negative zero. The D3 oracle's input
and output encoding was strengthened to retain its sign, retaining all 351 case IDs.
Regeneration of cases, manifest and license is byte identical. This fixes information
loss in JSON; no expected tolerance or pre-existing visual baseline was weakened.

| Evidence | Result |
| --- | --- |
| Core | Four paint tests cover every wire location, strict legacy/version validation, wide/exceptional values, deferred palette quantization and color-only cache reuse versus fresh batch. Shared-number/color/interpolation regressions pass. [Core](phase-2-paint/core.log), [numbers](phase-2-paint/numbers.log). |
| Publication | Two new export tests and 16 composition/publication regressions pass. Requests retain authored space after owners drop and produce stable scene/SVG/PDF/PNG bytes; old profile/live versions reject hidden new values. [Log](phase-2-paint/export.log). |
| Actual hosts | Four independently authored figures run in Rust/Python/WASM. Hosts exercise every paint input, malformed descriptors, owned/disposed values, wide binary64 channels, and retained requests. Python/WASM color-only edits preserve store revision; presented/current snapshots remain independent after disposal and current equals fresh batch. [Rust](phase-2-paint/rust.log), [Python](phase-2-paint/python.log), [WASM](phase-2-paint/wasm.log). |
| Runtime/type regression | Complete primary proof runner passes existing cases, 351 color cases, 370 interpolation plus 36 transform cases and the new paint figures. Final fixture layout and live-edit additions were rerun in the targeted hosts. New positive color-as-paint consumers pass mypy and TypeScript. [Primary](phase-2-paint/primary.log), [Python types](phase-2-paint/python-types.log), [TypeScript](phase-2-paint/typescript.log). |
| Visual evidence | All four RGBA channels match exactly across hosts in all four figures. PDF fonts embed the supplied Noto Sans. Final SVG-rendered PNG and rasterized PDF contact sheets were inspected, together with the actual native window. [Checks](phase-2-paint/publication-checks-final.json), [SVG/PNG sheet](phase-2-paint/contact-1.png), [PDF sheet](phase-2-paint/pdf-contact-1.png), [native capture](phase-2-paint/native.png), [native paint log](phase-2-paint/native.log). Individual source definitions, scenes and SVG/PDF/PNG are retained under [publication](phase-2-paint/publication/rust/spaces.svg). |
| Repository/macOS | `mise run check` and `mise run test` pass: 298 tests, zero failures/ignored. Initial test/build processes stalled in `_dyld_start` before execution; only those owned processes were terminated, and unchanged checks passed on retry outside the sandbox. [Check](phase-2-paint/check.log), [tests](phase-2-paint/tests.log), [test sample](phase-2-paint/test-loader-sample.txt), [build sample](phase-2-paint/build-loader-sample.txt). |
| Linux | Actual Rust 1.97.1 container with networking disabled and read-only source/registry passes 14 paint/color/interpolation tests, including descriptors. [Log](phase-2-paint/linux.log). This is a focused run, not aggregate Linux acceptance. |

No full perceptual scale/guide ramp, integrated G-COLOR acceptance, or performance
budget is certified here. Those remain CLR-05/SP-04/WP-22. Next: SP-01's scale
contract, pinned method-level oracle and executable expected-gap report.
