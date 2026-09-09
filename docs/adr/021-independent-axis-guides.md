# ADR-021: Independent positional guides over retained scales

Status: ACCEPTED for the WP-AX01 identity migration; provider and complete D3 profile
acceptance remain open. Date: 9 September 2026. Requirements: AXIS-01/07, AUT-01/04,
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

The bounded custom provider boundary, D3 tick selection/formatting profile, independent
inner/outer geometry, component metadata/styling and timed transitions retain their
named work-package obligations. This decision does not mark them implemented or close
G-AXIS. Existing adaptive collision thinning remains the legacy policy pending the
explicit preservation policy in WP-AX02/03.
