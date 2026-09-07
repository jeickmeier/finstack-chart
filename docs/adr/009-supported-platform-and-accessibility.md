# ADR-009: Verified platform, renderer and accessibility scope

Status: ACCEPTED for the original WP-21 scope, 7 September 2026.
Requirements: SCP-03, GPU-02/03, BND-03/04, QLT-02/03.

The supported native host is macOS Apple Silicon with pinned GPUI/platform 0.3.3.
The actual recorded environment is macOS 26.5.2, arm64, device scale 2 with explicit
Noto Sans/serif fixture resources. No Linux/Windows native or Intel macOS claim follows.
Headless core/text/export executed on macOS arm64 and Linux aarch64 in a local
Debian Bookworm container with Rust 1.97.1. CI additionally targets Ubuntu 24.04 and
macOS 15; configured runners are not evidence that remote jobs ran.

Portable proof adapters execute the original grammar and update/action/export traces
in actual CPython 3.14.6/PyO3 0.29.2 and Node 24.14.0/wasm-bindgen 0.2.128. Node is
WebAssembly runtime evidence, not browser DOM, accessibility or distribution proof.
SVG/PDF/PNG publication remains headless and uses explicitly supplied font resources;
fonts, physical dimensions, vector marks, clips and themes retain the inspected
WP-08/11/12/13/14 artifacts and are replayed by the current binding checks.

Native keyboard inspection, selection and editable annotations, chart summaries,
exact-value table alternatives and button/status exposure have actual WP-16/17
behavior/tree evidence. The OS accessibility inspector reports window focus even
when GPUI internally focuses a chart. VoiceOver speech and complete assistive traversal
are unverified. Applications requiring those capabilities must treat them as an open
integration requirement. We do not claim general screen-reader conformance.

WP-21 fixes frozen destination projection without changing retained semantics, and
adds one coalesced platform frame wake after accepted/failed background preparation.
The latter resolves an actual idle redraw stall reproduced without parent observers
or periodic parent notifications. At most one callback is outstanding per chart; it
retains a weak entity only. This is event-driven demand, not perpetual animation.

The [support matrix](../support-matrix.md) and [WP-21 report](../evidence/wp-21-completion-2026-09-07.md)
are the capability/evidence index. Expanded D3/ggplot2/authoring gates, sustained
performance and packaging do not pass through this decision. Known unmaintained
transitive dependencies and the upstream `block` future-compiler warning remain
release risks; they are not silently waived by passing fixture tests.
