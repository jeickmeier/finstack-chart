# ADR-021: Independent positional guides over retained scales

Status: ACCEPTED for WP-AX01 and WP-AX02. Complete D3 geometry, components and transitions remain open. Date: 9 September 2026. Requirements: AXIS-01/07, AUT-01/04,
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

The checked provider and independent selection/formatting policy are implemented. Independent inner/outer geometry, component metadata/styling and timed transitions retain their WP-AX03–06 obligations. G-AXIS remains open. Existing adaptive collision thinning remains the legacy policy.

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
