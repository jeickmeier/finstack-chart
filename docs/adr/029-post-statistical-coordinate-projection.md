# ADR-029: shared post-statistical coordinate projection

Status: accepted for GG13 implementation; acceptance evidence remains in the status ledger.

Coordinate views must change presentation without changing fitted models, statistical
populations or positional scale training. Native paint, publication, inspection and
navigation must consume the same geometry and preserve typed scale identity.

An optional definition-level `CoordinateSpec` selects Cartesian, transformed, polar or
radial policies. One resolved core map reads the existing positional scale calculation
spaces and retained domains. Runtime axis windows take precedence over authored view
limits. Calendar and session coordinates reuse their existing exact timestamp readers.
Definition capability 78 identifies this contract; earlier definitions remain unchanged.

Paths use bounded adaptive subdivision through that map. A transform must establish
monotonicity on the relevant interval before its enclosure can bound destination error.
Shared clipping lowers sectors and annuli into portable paths before any painter or
hit test runs. Physical glyph, line and arrow dimensions remain destination units.
Polar and radial clipping retain the distinct pinned reference policies. No renderer
implements its own coordinate equations.

Guide values are selected by the existing typed guide engine over a separate view of
the resolved axes; mark scales are retained. One guide owner then projects the selected
values and lays out radial domains, ticks, labels and theme grids. Static radial guide
snapshots remain available. Radial guide animation explicitly rejects because its
existing straight-domain transition model cannot represent arcs; Cartesian, flipped
and transformed transitions reuse the projected positions and physical panel range.

Images use bounded inverse sampling with premultiplied-alpha interpolation. Sampling
density is the larger of two samples per destination unit and the explicit device
scale, avoiding multiplication of two existing pixel densities. Invalid sectors and
holes remain transparent and have no hit coverage. Scene capability 22 adds bounded
per-target raster coverage while retaining the original target array. Compound custom
interaction regions reuse the shared path containment owner.

The same inverse supports inspection and navigation. Singular and genuinely overlapping
branches remain unavailable or ambiguous rather than selecting an arbitrary value.
Non-affine rectangular pan/region gestures and opaque native paint lack a conforming
projection contract and reject explicitly. Reference dotplots retain physical circles
under Cartesian/flip coordinates; the source-disclaimed nonlinear combination rejects.
No GIS engine, implicit resource lookup or destination-specific model is introduced.
