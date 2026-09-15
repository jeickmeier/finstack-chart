# ADR-024: sampled gradient paint for continuous guides

Status: accepted design; GG-05 implementation qualification is recorded in the status ledger.

GG2-04 requires a continuous colorbar to consume the same mapping as marks. GG-04
already retains the reference's complete default 300-sample color ramp. Painting
adjacent two-stop rectangles produced visible antialias seams in SVG and PNG;
retaining the numbers alone was insufficient publication evidence.

A single `SampledGradientRectangle` carries colors in increasing destination-axis
order. By default, color `i` lies at `(i + 0.5) / n`, with padded endpoint colors and sRGB
interpolation between samples. This matches the default raster colorbar's sample
and key coordinates. SVG/PDF use one gradient and one rectangle; PNG uses the
common SVG renderer. The native adapter creates one owned image with a
one-pixel-wide/high interior ramp and duplicated edge pixels. It maps only the
interior into the bar, supplying BGRA samples to GPUI's linear image filtering.
The explicit border prevents bilinear reads from adjacent atlas textures. No palette
computation or data training moves into a renderer.

The primitive requires at least two samples and charges the complete sample
payload against the existing aggregate path/paint work budget before scene
cloning. A scene containing this primitive requires scene wire version 17. Older
scene capabilities retain their earlier minimum version. The scene remains owned
and immutable, with no external image resource or I/O requirement in core.

The native adapter directly pins `image` 0.25.10 with default features disabled to
construct GPUI's owned image buffer. That exact package was already locked through
GPUI; the dependency does not add a package or a decoder requirement to core or
export. The lock change adds only the native adapter's dependency edge.

Authored raster sampling stays on the mapped scale in one optional
`GgplotColorbarOptions` carrier. It does not multiply each numeric/temporal guide
selection variant. The core palette owner evaluates `nbin` only when keys demand
samples, bounded by the interpolation work limit. Zero uses unique limits; one
uses the lower-limit sample. Single-color bars lower to the existing rectangle,
preserving the sampled-gradient primitive's two-color minimum. Fractional counts
retain their authored value for raster key coordinates. Only plots retaining this
carrier require definition version 64; the default guide path stays unchanged.

Raster presentation uses the same carrier for direction, reversal and selected
endpoint-tick controls. Nondefault presentation requires definition version 65.
Core reverses guide samples independently of mark mapping and resolves keys from
the resulting domain. Ticks do not depend on label visibility. Non-finite key
coordinates produce no ink. Horizontal guide allocation includes measured label
overhang and leaves the configured minimum plot span; constrained frames report
layout pressure through the existing solver.

Non-raster display requires definition version 66. Gradient display defaults to
15 samples with endpoint stops and unpadded keys. Scene version 18 adds
`SampledGradientMode::Endpoints` and `Steps`; the omitted `CellCenters` default
preserves version-17 serialization. A constant gradient domain lowers to solid
paint while its complete prepared sample vector remains available.

Rectangle display retains the actual sample count as equal-width cells, while
keys retain the authored count's padding. Separate antialiased rectangle paints
reproduced lighter page-colored seams inside a constant-color PNG bar. `Steps`
therefore stores the original cell colors once and emits coincident gradient
stops at shared boundaries. SVG/PDF paint one rectangle; PNG consumes that SVG.
The native adapter prepares adjacent quads from the same cell intervals. Both
stops per cell count against scene paint-work validation before cloning. This
representation preserves hard boundaries without a palette or stop calculation
being independently implemented in each headless encoder.

Colorbar alpha uses the same option carrier and requires definition version 67
when explicitly retained. The shared palette owner replaces sampled paint alpha
after resolving missing-paint policy, using the existing reference alpha encoder;
it leaves mark mapping and keys unchanged. Omission preserves palette alpha.
Missing paint retains its missing identity, while an explicitly transparent color
can become opaque. Alpha validation is eager even for hidden guides; finite values
outside zero to one and NaN reject, while infinities retain the reference's zero
coverage behavior. Renderers consume the resulting color bytes without applying
another alpha override. Qualification remains recorded separately in the ledger.

Even stepped guides retain directed transformed intervals and resolved colors in
`ColorLegend::colorsteps`. Explicit stepped guides retain the existing interval
palette batch instead of discarding it or evaluating callbacks again. Default
binned guides reuse the trained bin palette. The shared cut parser retains ordinal
key positions alongside the raw numeric candidates, before missing cuts are removed.
Layout consumes those positions so duplicate and unsorted cuts retain source order;
infinite edge intervals retain their missing-color cells. Zero-interval explicit
guides retain build-time labels and callbacks but reject at layout; automatic empty
guides remain hidden. Cells reuse the measured
bar, label and tick painter and the existing seam-free `Steps` primitive. Reversal
changes guide order independently of mark mapping. Nonuniform spacing and limit-label
controls need their own reference and qualification evidence.

This decision supplies default and authored continuous colorbar display. Remaining
layout/composition controls, full stepped guides and complete legend component identity
remain GG-05 work. Device pixel certification retains the WP-21 boundary; source
key/decor and native/export inspections have distinct evidence in the ledger.
