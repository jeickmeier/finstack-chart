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
  Scale/layout resolution, destination primitives and rendering/export adapters remain
  subsequent work; `PreparedGeometry` explicitly preserves data-space endpoints.

These are WP-02/WP-04/WP-05 foundations for version 0.1.0. Full chart compilation,
rendering/export and wire/binding APIs remain unimplemented. Initial public signatures may evolve with their
first consumers; breaking changes and future schema migrations must be recorded explicitly.
