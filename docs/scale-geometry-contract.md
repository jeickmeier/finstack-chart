# Built-in scale and geometry contract

WP-11 extends the existing compiler, named scale layout and immutable scene. The
[specification](spec/gpui-charts-specification.md) remains authoritative. The
[family fixtures](../fixtures/families/README.md) and
[completion report](evidence/wp-11-completion-2026-09-07.md) record executed scope.

This document describes the implemented WP-11/0.1.0 behavior. Specification 0.2.0
adds required D3 scale parity under SCL-06–08; the
[scale parity plan](impl_plans/d3-scale-parity-plan.md) tracks missing families and
changes to the positive-only log, tick, category, interpolation and calendar contracts.
Those changes are planned, not implemented or certified by this report.

Specification 0.3.0 additionally requires CHR-01–06 and the
[scale-chromatic parity plan](impl_plans/d3-scale-chromatic-parity-plan.md). Named schemes,
Brewer splines, lookup/analytic/cyclical ramps and their portable operations are planned;
the raw palettes described below retain their existing meaning.

## Positional scales

A nonempty `ChartDefinition.axes` replaces the destination's default axis list. Each
axis has a stable ID, orientation/side, family, optional viewport/range, outside policy
and guide visibility. Empty axes preserve the existing automatic linear/band/UTC route.
Training precedes viewport omission and clipping; a destination omission never reruns
statistics or discards a source row. A closed-session or log-invalid row can therefore
still contribute to another axis's domain.

- Log uses positive values and a finite base greater than one. Its transform is
  `ln(x)/ln(base)`; bases 10 and 2 use the corresponding direct logarithms. The inverse
  is `base^t`. Empty training resolves to `[1, base]`; constants expand by half a
  transformed unit on either side before enabled automatic padding/nice policy.
  Nonpositive points/line vertices are omitted and split runs, or reject for strict
  layers. Interval and area baselines must be valid positive coordinates.
- Symlog is `sign(x) * ln(1 + abs(x)/threshold)`, with finite positive threshold.
  Its inverse is `sign(t) * threshold * expm1(abs(t))`. Checked logarithmic fallbacks
  avoid intermediate overflow when the final value is representable. Nonzero inputs
  collapsing to zero or nonfinite results reject as precision loss. Empty and constant
  policies use the foundational continuous resolver in transformed coordinates.
- Explicit nonconstant domains remain exact in data units and retain direction.
  Automatic baseline/padding/nice never changes an explicit domain. Viewports are
  independent data-space intervals; outside extend/clamp/omit uses the shared resolver.
  Numeric map/invert tests use relative tolerance `1e-12 * max(1, abs(expected))`.
  Nonlinear/secondary guide labels use 12 significant decimal digits; this rounds
  labels only. Prepared values, domains, targets and projection coordinates retain f64.
- Point scales preserve an explicit or retained category order and provide category
  lookup without numeric inversion or a fabricated band width. Default outer padding
  is half a point step; a singleton is centered. Lookup ties choose the earlier category.
  Band scales retain their existing extent behavior and fixed-slot dodge semantics.
- Supplied sessions carry caller identity/revision, explicit timestamp unit, ordered
  non-overlapping positive-duration intervals, and Omit/Nearest/Error closed policy.
  Interior intervals are half-open; the final endpoint is included. Active duration is
  compressed using exact integer offsets, limited to `2^53` ticks. Nearest closed ties
  choose the earlier boundary; inversion at a compressed shared boundary chooses the
  next session start. Original timestamps are retained. Duplicate rounded ticks are
  removed. Labels use existing UTC formatting; no exchange calendar or local zone is supplied.

`AxisScale::Secondary` is a guide-only affine unit conversion over an existing numeric
primary axis: `factor * value + offset`, with finite nonzero factor and finite offset.
Both full domain and viewport must retain distinct finite converted endpoints. A secondary
cannot bind layer coordinates, reference another secondary, change orientation or author
its own viewport/range/outside policy. Its positions come from the primary ticks; each
side can independently thin colliding labels. Independent named axes instead train from
their own bound layers. Both constructs use the same plot clip and projection engine.

## Colors and geometry

`ColorEncoding` binds source categories/numerics, prepared groups or generated numeric
stat fields to a named color scale. Discrete scales use declared palettes and explicit
or retained category order, cycling the palette; unknown explicit-domain/null values
use the declared missing color. Continuous scales use piecewise interpolation of sRGB
bytes and alpha between equally spaced palette stops, rounding to nearest byte, with
explicit numeric domain and clamp/missing-outside policy. They are not perceptually
uniform interpolators. Color domains/palettes are bounded by compile group limits.
Compatible shared IDs must resolve identical legend metadata. Run colors must be
constant within each group; points/cells/bars support row-varying numeric color.
Color never contributes positional domains. Semantic results expose legend entries,
continuous/discrete identity and missing style; guide composition is WP-12.

`Geom::Area` pairs y with an explicit finite calculation-space baseline (`Geom::area()`
uses zero). `Geom::Ribbon` pairs lower y with upper y2 and rejects/excludes lower > upper.
Both reuse line ordering, stable groups and gap handling. Boundary endpoints train
domains and retain one target per observation pair. Single-pair runs paint a vertical
rule; longer runs become closed nonzero-winding filled paths. The same path is validated,
painted natively and encoded as vector SVG/PDF or raster PNG. Source targets follow
boundary vertices; complete interior-region hit testing remains WP-16.

`Geom::Bar` spans y and an explicit y2 baseline at x with a positive destination-unit
width; optional nonnegative validation is independent of other layers. It supports the
existing exact stack/normalize and dodge policies. Existing rectangle intervals provide
band-relative grouped bars and rectangular heatmap cells; histogram remains bin plus
rectangle composition. `Layer::bars`, `volume`, `cells`, `ohlc` and `histogram` are concise
constructors over the common engine. Display widths do not train data-space x domains,
so an author can add axis padding when an edge symbol should be fully visible.

OHLC uses x, y=open, y2=close, low and high source mappings. Eligibility requires
`low <= open <= high` and `low <= close <= high`. Each valid row emits a wick and body
with the same source target; equal open/close paints a horizontal doji rule. Bound values
train price domains. OHLC uses identity positioning to preserve all four values together.
The gallery composes an OHLC layer and an independently scaled `volume` layer: negative
volume is reported/excluded without discarding a valid candle. No financial model or
pricing engine is introduced into core.

## Compatibility and validation boundary

This is an additive pre-1.0 source/wire change: new public enum variants affect exhaustive
matches; new struct fields require updates for Rust literal construction. `axes`, layer
`color` and source low/high fields are optional in portable JSON. Existing fixtures retain
their original numerical tolerances. The same full definitions and fixtures execute in
Rust, Python and single-threaded Node WASM. Native evidence is macOS only. [Facets](facet-layout-contract.md) are now implemented in WP-12; full
themes, extensions, complete interaction/indexing and release-platform gates remain open.
