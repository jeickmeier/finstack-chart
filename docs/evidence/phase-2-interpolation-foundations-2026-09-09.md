# WP-IP01–05 interpolation foundations

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 changes.
[Source hashes](phase-2-interpolation-foundations/source-hashes.json) identify the
qualified implementation. Requirements: ITP-01–06/08 and FIX-I01. WP-IP01–05 are
COMPLETE for reference, shared kernels and standalone host scope. WP-IP06/07 and
G-INTERPOLATE remain open.

The pinned d3-interpolate 3.0.1 corpus has 370 scalar, structured, color and zoom
cases. A separate pinned Chromium 151.0.7922.34 corpus has 36 CSS/SVG transform
cases. Both regenerate byte for byte. All 27 exports and factory controls are
inventoried. [ADR-017](../adr/017-shared-interpolation-values.md) defines explicit
typed adaptations, exceptional channels, copy/reuse ownership, bounds and strict
portable descriptors. Hosts forward numerical work to the shared Rust engine.

| Evidence | Result |
| --- | --- |
| Core | 11 interpolation tests plus six color/path regressions pass; two additional descriptor tests pass. Covers independent rounding, spline seams, hue/gamma/alpha, cast, ownership, malformed input and aggregate bounds. [Kernels](phase-2-interpolation-foundations/rust.log), [descriptors](phase-2-interpolation-foundations/descriptors.log). |
| Actual Python/WASM | Both execute 370 plus 36 cases through raw descriptors and public decoded values. Strings/tags/dates/array casts match exactly; numerical operation bounds are declared in the readers. Copies, disposal, configurations and round trips pass. [Python](phase-2-interpolation-foundations/python.json), [WASM](phase-2-interpolation-foundations/wasm.json). |
| Primary API | Complete existing primary runtime/type runner, new positive consumers and six intentional invalid type cases per host pass. [Log](phase-2-interpolation-foundations/primary.log). |
| Repository | `mise run check` and focused core/Python/WASM Clippy pass. [Check](phase-2-interpolation-foundations/check.log), [Clippy](phase-2-interpolation-foundations/clippy.log). |
| Linux | Rust 1.97.1 container with read-only source and networking disabled passes 11 interpolation and six color/path tests. The two later descriptor tests were run on macOS only. [Log](phase-2-interpolation-foundations/linux.log). |

These are focused interpolation gates, not a fresh aggregate macOS/Linux test gate.
No integrated chart scale/axis transition, native window, or publication image is
certified by these standalone results. Paint integration, shared consumer wiring,
registered extension contracts and the full integrated acceptance matrix remain
assigned to the subsequent packages.
