# ADR-021: Independent positional guides over retained scales

Status: ACCEPTED for WP-AX01–05. Date: 9 September 2026. Requirements: AXIS-01–07, AUT-01/04,
FIX-19. This conforms to the [axis plan](../impl_plans/d3-axis-parity-plan.md) and
amends the one-guide-per-side restriction in [ADR-005](005-foundational-scales-and-layout.md).

`ScaleId` continues to own positional domain training, layer bindings, calculation-space
meaning and navigation. `GuideId` identifies guide presentation independently. Default
axis guides use the historical identity's numeric value in the separate `GuideId` type;
additional guides receive their own identities and reference an existing positional
scale. Replacing a guide's scale reference preserves its identity and the other guides.
A guide cannot change orientation relative to its scale or use an alternate-unit guide
as its positional source. Existing checked secondary unit conversions remain distinct.

`AxisSpec` remains the compatibility entry point for a positional scale and its default
guide. `GuideStyle` owns its presentation fields once; flattened serialization preserves
the existing field names. The legacy Rust field-access route delegates to that shared
style. `GuideSpec` adds independent identity, scale reference, side and a finite explicit
translation. Definitions and requests carry additional guides separately, so scale
validation and mark projection do not interpret them as new trained scales. Legacy
scale limits remain four; the shared guide budget permits at most 64 total guides and
also applies the existing aggregate text, item and path limits. This is a work bound,
not a tick-count approximation.

Each scale is resolved through the existing scale owner. One common tick resolver reads
that retained scale for default and additional guides; it does not rescan source rows
or train another scale for each guide. Ticks retain original numeric values, exact
integer timestamps with units, or category labels. Secondary conversions consume the
retained semantic number rather than reconstructing it from a painted coordinate.
The renderer uses the existing scene primitives, destination text services, clipping
and publication paths. Explicit guide translation affects the guide; mark mapping keeps
its positional scale's range.

The primary API adds `axis_guide(name, scale_name)`, `PlotBuilder::guide` and immutable
`PlotEditBuilder::guide`, with a separate guide name map and `GuideHandle`. Existing
`AxisHandle` and axis names retain layer-binding/navigation meaning. `Plot::guide_axis`
resolves the positional handle for a named guide. Replacing a named guide preserves its
existing identity. Additional guides require definition/primary envelope version 8;
versions 1–7 remain readable and are emitted when no new capability is retained. Core,
Python and WASM syntax dispatch use the same builders.

The checked provider, independent selection/formatting, geometry, component styling and timed transitions are implemented. WP-AX06 owns cumulative certification. Existing adaptive collision thinning remains the legacy policy.

## Registered positional provider boundary

The AXIS-01 provider boundary is implemented through `CustomScale` registrations in
`ExtensionRegistry`. Each versioned factory resolves one immutable `PositionalScale`
for a positional scale per layout iteration. All its guides and marks consume the
same checked result. The prepared chart retains a copy-on-write registry snapshot;
registering another operation in a later registry cannot change that snapshot.
Factory parameters are bounded JSON values, and descriptor identity/portability is
captured on registration. Providers are trusted native code: core bounds their inputs
and checks their outputs, but cannot preempt arbitrary native callback execution.

The checked adapter captures semantic domain, finite range, optional bandwidth/rounding
and an explicit inverse capability. Numbers, original category strings and exact
integer timestamps with their source unit cross the boundary; prepared ordinals and
relative timestamp coordinates are converted through the layer's own metadata first.
The adapter validates finite mapping/inverse outputs, selected-value counts and types,
and aggregate domain/label bytes. Domain fallback and formatting are independent;
formatters receive the original value, index, complete selected list and tick arguments.
Empty and repeated labels survive the adapter. Optional band-start mappings use shared
centering and band extents for position adjustments. No numeric inverse is inferred
from numeric output. Generic pan/zoom rejects a provider because an inverse alone does
not declare its navigation metric; a factory may explicitly support supplied windows.

The primary selector is `scale_registered(id, version, parameters)`. Under a profile
that projects scales before statistics, select it through `coordinate_scale` to state
that custom mapping happens after statistics. A provider does not silently supply a
statistical transformation. LibraryV1 already projects after statistics. Host
`ExtensionRegistry` is a compatible public alias for the existing registry owner,
with `with_registry` delegating to the same retained registration snapshot.

Definitions and primary envelopes containing a registered positional provider use
version 10. Older capabilities retain their existing versions and serialization.
Loading requires explicit installation of the referenced native code. Native-only
providers reject portable serialization and headless layout; they can still execute
in the native adapter. No interpreter objects, runtime module loading, mandatory I/O,
or duplicate mapping engine enters core. The portable external fold example composes
the shared linear mapper and deliberately has no inverse.

The legacy guide policy still owns automatic thinning and existing formatting options.
Per-guide D3 selection/formatting and preservation are implemented below. Range-derived geometry, components and transitions remain WP-AX03–06. The provider's typed tick arguments and band metadata
are the shared boundary consumed by those packages, not certification of the complete
axis profile.

## Independent tick selection and formatting

`GuideProfile::LibraryV1` remains the default. `D3_3_0_0` opts an individual guide into
shared D3 scale tick/format policies and preserves the complete selected order,
duplicates and empty labels. It never changes scale training. The profile contract
is introduced here; exact D3 geometry/default dimensions remain WP-AX03.

`tick_arguments`, `tick_values` and `tick_format` are independent nullable controls on
both the default axis guide and additional guides. `None` resets only that control.
`Some([])` selects no ticks and bypasses automatic enumeration. Explicit lists are
validated and budgeted before formatting; format callbacks receive the original value,
index and complete selected list, including values for which mapping returns no position.
Numeric defaults call the shared family tick and locale formatter; calendar defaults
use the shared calendar algorithms. Band/point fallback retains domain order without
budget-driven subsampling. Hard count/text limits fail explicitly. Automatic D3 ticks
for the legacy session scale and the new controls on secondary conversion guides are
unsupported; no approximate policy is substituted.

`GuideFormatter` supports exact labels, shared numeric/calendar descriptions and
versioned `CustomGuideFormatter` registrations. Registered descriptors and parameters
are captured in the prepared registry snapshot. Native-only formatters reject portable
serialization and headless execution. Host operation versions pass through the existing
exact-integer converter. No interpreter callback or duplicate formatting engine enters core.

New controls/profile require definition and primary envelope version 11. Earlier
capabilities retain their existing wire versions. `LaidOutChart::guide_snapshots` and
headless `Frame.guides()` expose coherent configuration and semantic ticks, with facet
and inset scope paths and exact timestamp strings. This separate version-1 observation
envelope preserves the historical scene serialization. It supplies selection metadata;
component identities and transitions remain WP-AX04/05.

## Signed destination geometry

WP-AX03 adds optional `GuideGeometry` in definition/primary wire version 13. Optional
inner/outer lengths, padding and offset inherit profile defaults; finite signed values
are valid. D3 defaults are 6/6/3 and offset 0.5 for an absent or unit device scale, zero
for device scale above one. Native supplies `Window::scale_factor`; primary headless
`LayoutOptions::device_scale` supplies an explicit policy independent of raster DPI.
LibraryV1 retains its prior geometry and adaptive thinning when no new controls exist.

Domain paths use resolved range endpoints, including custom/reversed ranges. Band/point
centering consumes the retained band's existing extent and rounding, adjusted for the
profile offset; marks keep the unmodified mapping. Translation is applied once to guide
positions and the orthogonal baseline. Offset does not affect training or data marks.
D3 plain label defaults use size 10, measured by the supplied destination service.

`Preserve`, `HideLabels` and `ThinTicks` distinguish semantic tick preservation from
adaptive presentation. Empty/repeated labels retain ticks. Explicit figure-cell overflow
and tick-grid clipping are independent; all primitives still obey the outer scene bounds.
Finite/work/path validation precedes destination callbacks and immutable failed layouts
preserve prior scenes. Shared facets and resize consume the same geometry kernel.
The [geometry evidence](../evidence/phase-2-axis-geometry-2026-09-09.md) records the pinned
reference replay and inspected native/publication examples.

## Portable components and typography

WP-AX04 adds `GuideComponents` in definition/primary wire version 14. Domain, default
tick lines and default labels have independent optional overrides. A bounded `per_tick`
list addresses original selected indices before hiding/thinning; duplicate indices reject.
Missing values inherit the whole-guide component, profile and destination theme. Line
widths/dashes reuse the scene's validated stroke/dash contract. Authored color inputs
resolve at the common paint boundary. Label size, optional `RichRun` typography and
rotation use the existing supplied-font/shaping service; no destination reimplements
styles. Per-tick font resources, fallback, weight and run scale inherit through the
same rich-run contract. Global guide visibility still controls margin/painting; hiding
a component does not change scale training or selected values.

D3-profile or explicitly configured guides retain `SceneItem::guide` metadata. It carries
independent guide identity, panel/inset scope, side, domain/line/label role, original
selection index, logical label and value-plus-occurrence identity. Equal signed zeros
share an occurrence counter. Decorative items keep empty data-target lists. Scenes with
these roles use version 14; unconfigured LibraryV1 scenes omit the optional field and
retain their earlier version/output. Hand-authored Rust scene items add `guide: None`.

SVG text output groups consecutive components into addressable `axis` and `tick`
groups with logical values, occurrence/index and labels. Domain paths, tick `line`
elements and labels carry classes/roles. Rich runs use positioned outline groups.
Outline SVG keeps the same structure and logical metadata: it takes positioned glyph
fragments from the already retained usvg tree, using usvg's pinned XML reader, and feeds
them through the same component writer. Empty/transparent text may have no glyph path
but retains an explicit logical group. This adds no parser/renderer dependency and no
second shaping route. PDF/native/PNG consume the same resolved scene styles. External
CSS edits affect only the exported SVG. Full selection, including hidden components,
remains observable in the coherent guide snapshots.

The [component evidence](../evidence/phase-2-axis-components-2026-09-09.md) records
three-host versioned style/typography proofs and inspected native/vector/high-DPI output.

## Timed guides and exact displayed capture

Core `GuideTransitionPlan` compiles bounded enter/update/exit joins keyed by the new
raw scale projection, including duplicate first-match behavior and retained lifecycle
IDs. `LayoutGuideTransition` applies sampled geometry and opacity to existing scene
primitives, retaining facet/inset scopes and empty decoration targets. Shared scalar
and transform interpolators own numeric sampling. Host clocks remain outside core.
Interruption starts existing nodes from the displayed sample; new nodes use the prior
target scale. Side replacement and incompatible resources use immediate replacement.
Final primitives equal fresh target layout; lifecycle IDs survive final presentation.

Native opts in with a duration, uses GPUI reduced motion, cancels on disposal/frozen
interaction, and acknowledges only painted frames. Portable FigureTransition owns its
inputs and samples explicit fractions without relayout. Optional Scene v15 animation
metadata adds tick lifecycle identity and continuous opacity; static scene versions and
primary authoring envelope versions do not change. Native resolves continuous primitive
alpha, while SVG/PDF use tick group opacity; overlap raster compositing is renderer
specific and no pixel-identical cross-renderer claim follows from semantic parity.

Displayed capture freezes the acknowledged scene at its current dimensions, including
exits and fractional opacity, and retains fonts/extensions after the view is disposed.
One scene unit equals one publication point. Reflow, full-domain or different-size
capture must use the existing input-based Presented/Current policy. Host
`acknowledge_frame` checks definition/source/view/visibility/annotation identity before
admitting an explicit sample, preventing stale handles from replacing presented truth.
