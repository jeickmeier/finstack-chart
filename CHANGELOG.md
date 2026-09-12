# Changelog

## Unreleased

- Withdraw the optional `gpui-charts-kit` crate. Kit theme mapping is a gallery recipe
  over `ThemePatch` and `ChartInput::from_plot`; standalone GPUI stays Kit-free.

- Add shared UTC and explicitly supplied local calendars, filtered intervals, automatic
  ticks/nice and full locale time formatting. Exact time scales retain integer origins,
  piecewise ranges and finer timestamp units. Calendar axes and time formatting use
  v5; actual Python/WASM proofs preserve timezone revisions across DST publication.

- Add raw D3 numeric tick candidates, log minor-label suppression and inferred numeric
  formatting with complete specifier types and explicit locales. Fixed and significant
  formatting share exact decimal rounding. Axis formatting uses v5 descriptors;
  layout thinning leaves standalone candidates unchanged. Sequential/diverging nice
  preserves the midpoint; quantize nice rebuilds its thresholds.

- Add shared typed continuous, sequential/diverging and distribution scales, with
  quantile/quantize/threshold intervals and native custom sampling. Chart color and
  numeric size/opacity/stroke mappings use the same engine and v5 descriptors;
  quantiles train on eligible post-stat populations independently of viewport.
  Floating alpha now survives after-scale expressions and opacity until final paint.

- Add typed immutable ordinal catalogs and shared D3 band/point spacing with alignment,
  rounding, step/bandwidth, deduplication and exact integer keys. New categorical axes
  use definition v5 while existing recipe spacing remains explicit and unchanged.


- Add compatible Rust numeric scales with piecewise knots, signed power/sqrt,
  negative-domain logarithms, identity, radial mapping, rounding, unknown values and
  explicit nice edits. Named axes and navigation share the numeric mapping;
  definitions carrying these axes use version five. Existing recipe policies remain
  explicit. Full scale ticks/formatting and host qualification remain in progress.

- Add shared floating color values and standalone interpolation across Rust/Python/WASM.
  Authored paint retains color descriptors through layers, palettes, themes, text,
  candles, gradients and publication backgrounds; preparation lowers to existing
  scene bytes. Floating paint first uses version four, preserving byte-only legacy
  versions and palette semantics. Shared numeric transport retains signed zero.

- Add canonical ggplot2 4.0.3 stage/profile provenance, inferred discrete groups,
  horizontal recipes and typed source/stat/bin/post-scale/theme expressions.
  Positional limits/transforms precede statistics under the profile; coordinate
  controls preserve the population. Shared transforms reject conflicting scale
  contexts. Primary Rust/Python/WASM authors share the evaluator and immutable
  capture semantics; new definition capabilities use version three.

- Add checked standalone path authoring and numeric replay, analytic arcs, signed
  rectangles, configurable SVG digits and atomic resource-bounded operations.
  Primary path annotations, native/export rendering and actual Python/WASM adapters
  share the core geometry. Retained path definitions/compositions and scene output
  use explicit version-two capabilities; legacy envelopes remain supported.
- Pin the development-only D3/R reference workspace, complete ggplot2 public inventory
  and reproducible offline seed artifacts for subsequent Phase 2 packages.

- Suppress empty color-guide furniture and honor `legend().untitled()` on primary
  builds and edits in ordinary and faceted charts. Existing absent titles retain
  their generic fallback through `generic_title()`; empty strings explicitly omit
  the title. Python/WASM dispatch and declarations expose both choices.

- Add the primary `Data`/`Plot`/component API and retained `Chart` runtime. Typed rows,
  exact nullable columns, stable handles, definition edits, transactions, retention,
  actions, navigation, linking and registered operations reuse the shared engine.
- Add `Output` and export options with independent Presented/Current capture,
  visible/full projection and interaction policy. Native inputs retain Chart ownership;
  Python and WASM provide ordinary data/components with checked declarations.
- Migrate the gallery, live update/export and native performance consumers, README and
  authoring tutorials. Version 1 wire envelopes and old public expert paths remain
  supported; their planned removal boundary is no earlier than 0.3.0 after a 0.2.0
  migration release. Packages remain unpublished at 0.1.0. See the primary coverage
  register for qualification; this does not add future D3/ggplot2 semantic features.

- Complete original WP-16–23 indexed interaction, exact source lookup, linked host tools,
  annotation editing, atomic retention/streaming, bounded native preparation/density,
  coherent live export and native hardening. Frozen resize keeps the exact captured
  semantics; background completions demand a bounded presentation frame.
- Consolidate inspection identity/index storage while preserving exact and duplicate-x
  results. Add release-mode CPU/GPU/actual-presentation benchmarks and measured evidence.
  The owner waived the 30-minute run; retained interrupted/occluded traces are not passing
  sustained evidence. See WP-22 for numerical budgets, memory and visibility limits.
- Add an original-requirement release index, developer/example routes, source provenance
  and deterministic unpublished local source packaging. Current expanded parity/authoring
  and production G4 remain open; no license or registry distribution is implied.


- Add the shared revision-fenced action reducer, distinct interaction state, explicit scene
  acknowledgment, pinned gesture previews/cancellation, bounded annotation undo/redo,
  controlled replacements, linked sequence tracking and freeze/resume ownership. Native,
  Python and WASM dispatch use the same core; saved state excludes ephemeral previews.

- Add typed facet wrap/grid, shared/free named scales, stable panel targets and aligned guides.
- Add explicit theme cascade, rich/rotated/tabular typography, publication furniture/insets,
  optional Kit theme integration and physical SVG/PDF/PNG preview.
- Add registered custom stat/geom APIs with checked generated schemas, targets and interaction
  metadata; host-owned native painters report explicit export failure. Add Cartesian capability
  APIs, custom guides, candle direction colors and compile-fail stage-accessor examples.
- Freeze the documented alpha boundary and complete 36 actual Rust/Python/WASM fixture cases.
  Public enum/struct additions require downstream exhaustive-match/literal updates. Optional
  portable fields preserve existing defaults; native callback code is never serialized.

- Add portable log/symlog/point/color/supplied-session scales and guide-only secondary unit axes.
- Add area/ribbon filled paths across native/SVG/PDF/PNG, interval bars, validated OHLC and
  independently validated volume, and rectangular heatmap recipes over the shared compiler.
- Extend actual Rust/Python/WASM proofs with 12 family fixtures and a native family gallery.
  New enum variants and Rust struct fields affect exhaustive matches/literal construction;
  portable axes/color/low/high fields are optional. See the scale/geometry contract.

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
  context. Diagnostic enum additions affect exhaustive downstream matches; the initial delivery preceded the later WP-09 wire encoding. See ADR-004 for replacement/schema and replay rules.

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

- Add strict version 1 chart/data/transaction/action/state/profile envelopes, exact decimal
  64-bit encodings, native-accessor rejection and bounded input decoding in the shared core.
- Add actual PyO3 0.29.2 and wasm-bindgen 0.2.128 proof adapters, owned copies, deterministic
  disposal and shared publication sessions; Python Rust-only work detaches the interpreter.
- Add the executable `bindings-proof` task with Rust/Python/WASM semantic, scene, export,
  large-integer/time and lifetime fixtures. Minimal G1 now passes; full G4 portability remains open.
- Core gains Serde/serde_json dependencies, without host libraries or I/O. Wire spellings and
  operation versions are explicit compatibility contracts; see ADR-006. No old wire format
  existed to migrate, and future schema changes require a version or explicit migration.

These are WP-02/WP-04/WP-05/WP-06/WP-07/WP-08/WP-09 foundations for version 0.1.0.
Full grammar, complete portable coverage and production host/distribution APIs remain later work. Initial public signatures may evolve with their
first consumers; breaking changes and future schema migrations must be recorded explicitly.

- WP-10: add version-one count, automatic 30-bin, exact grouped summary/quantile and
  intercept OLS operations, typed generated fields/schemas and exact model memberships.
- Add explicit mixed-sign stack/normalize, band-relative fixed-slot dodge and stable
  seeded data/display jitter; normalized domains are dimensionless and incompatible
  additive encodings reject. Declare exact full-recompute capabilities for every builtin.
- Add independent FIX-02–05, malformed/numeric/reorder/update tests and twelve cases in
  the real Rust/Python/WASM proof. Semantic results include schema and operation records.
- Compatibility: new `StatParameters`, `Mappings`, `Position`, `PreparedRows`, `OutputSchema`
  and `GeneratedKind` variants require exhaustive consumers to update. `Position` now owns
  explicit group-order vectors and is no longer `Copy`/`Eq`; integer seeds/group keys use
  canonical decimal wire strings. New builtin registrations retain envelope version one.
