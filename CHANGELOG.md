# Changelog

## Unreleased

- Add initial `chart-core` identities/revisions, structured diagnostics, finite geometry,
  bounded immutable scene construction and synchronous text/resource service contracts.
- Add public contract tests and a Rustdoc example; inspect core/export dependency
  isolation for macOS, Linux and browser-WASM target graphs.

- Add immutable typed/normalized data snapshots, versioned schemas, all seven portable
  column kinds, exact source values and bounded numeric-validity diagnostics.
- Add ordered atomic append/upsert/remove/replace, count retention, expected revision
  fences, bounded FIFO replay, stable ordinals/categories and source/aggregate/derived
  provenance. Shared chunks retain old snapshots without copying row history on append.
- Add schema/epoch/aggregate/derived identities and diagnostic revision/count/sample
  context. Diagnostic enum additions affect exhaustive downstream matches; wire encoding
  is still unimplemented. See ADR-004 for replacement/schema and replay rules.

- Add typed column authoring and a shared staged grammar compiler for heterogeneous
  source/generated layers, identity/explicit-bin statistics and point/line/rule/rectangle
  data-space geometry. Histogram recipes lower to ordinary bin/rectangle layers.
- Add typed generated schemas, endpoint domains, aggregate membership, calculation-space
  metadata, bounded named-transform reuse and minimal viewport/visibility actions.
- Add FIX-01/02 and composite compiler tests plus a public grammar Rustdoc example.
  `PreparedGeometry` explicitly preserves data-space endpoints for later projection.

- Add foundational linear/band/UTC scales, explicit domain and viewport policies, calendar
  ticks, integer-origin time projection and category lookup without a numeric inverse.
- Add named positional scale IDs, categorical source encodings, plot/figure clip policies
  and a shared destination layout route with measured plain axes, a four-pass margin cap,
  controlled no-data/no-space states and exact mark/vertex provenance.
- Add FIX-07 and layout/zoom/font/category counterexamples; record algorithms and tolerances
  in ADR-005. Extend the grammar Rustdoc example through destination scene construction.
- Compatibility: `Layer` gains `scales`/`clip`, `Numeric` and `ValueSpace` gain categorical
  variants, and `SceneStamp` gains `state` for visibility-sensitive snapshots. Struct literals
  and exhaustive downstream matches must adopt these fields/variants. No wire schema exists.

- Add a retained standalone GPUI chart view with native vector paint, supplied-font text
  measurement/shaping, bounded frame caching and caller-owned tooltip elements.
- Add shared presented-snapshot inspection: scatter radius, nearest-x line groups, bar
  containment, keyboard focus, stale action fences and exact source/aggregate targets.
- Replace the exiting gallery shell with line, scatter, histogram and UTC examples; verify
  real hover/keyboard input, zoom, resize, malformed updates, no-space and entity disposal.
- Promote the already locked `ttf-parser` 0.25.1 to the native font preflight bridge.

- Add immutable headless figure capture, point-based publication profiles, explicit font
  resources and SVG/PDF/PNG bytes with diagnostics and reproducibility metadata.
- Add full-font SVG, subset-font PDF and outline modes, physical DPI output, visible/full
  domain policy and a native vector preview using the captured publication layout.
- Adopt resvg/usvg 0.48.1 and krilla 0.8.2 without the legacy font-shaping dependencies
  in the normal export graph; retain historical proof dependencies and open host advisories.
- Add ten export integration tests, inspected artifacts and independent output checks.

These are WP-02/WP-04/WP-05/WP-06/WP-07/WP-08 foundations for version 0.1.0. Full grammar
and wire/binding APIs remain unimplemented. Initial public signatures may evolve with their
first consumers; breaking changes and future schema migrations must be recorded explicitly.
