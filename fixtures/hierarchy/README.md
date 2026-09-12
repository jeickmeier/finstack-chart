# FIX-H01 hierarchy reference catalog

Pinned development-only oracle: **d3-hierarchy 3.1.2**, ISC, source commit
`7bea49efdc1093b28a8d60d2f8717d8a17803564`. The shared npm lock already selects it;
no production dependency or lock upgrade is introduced. `manifest.json` retains package
integrity, every source hash, generator and artifact hashes and Node environment.

Run `mise exec -- node tools/reference/node/hierarchy.mjs [OUTPUT]` from the repository.
The generator captures 16 exports, every Node/factory method, defaults and control-mode
readback, plus 826 cases (441 entry seeds, 256 asymmetric tree/cluster cases, one topology-history trace and 128 additional packing/enclosure cases) across construction, operations, all five layout families,
six standalone tilers, custom policies, three resquarify histories and packing helpers.
Nested fixture payload names are expected labels, not durable identity. Tests retain
exact node order, topology, labels, values and controls; non-finite reference results use
explicit number tags. Independent traversal/own-value/count/partition/enclosure seeds are
asserted by the generator before writing results.

Initial comparison tolerances, fixed before implementation: tree/cluster/partition and
rectangle tilers use `1e-10 + 1e-12 * abs(expected)` coordinates; packing/enclosure use
`1e-8 + 1e-10 * abs(expected)` coordinates/radii. Integer topology, ordering, metadata,
errors and callback contexts compare exactly. Packing invariants additionally check
non-overlap/containment under scale-aware residual bounds; passing coordinates alone
cannot hide invalid geometry. [ADR-022](../../docs/adr/022-hierarchy-topology-and-layout-history.md)
records checked-input/degenerate adaptations. Final H08 coverage must add actual host,
chart, identity, update, resource and destination evidence; this seed harness is not
production parity certification.

Packing placement retains D3's `1e-6` radius-unit intersection slack; fitted outputs scale
that bound by their radius fitting factor. Enclosure uses its separate relative `1e-9`
weak containment predicate. Independent invariants account for those documented local
bounds, never applying them to tree/rectangle or unrelated geometry comparisons.

Final implementation verdicts and actual runtime/destination evidence are in the
[H08 catalog](../../docs/evidence/phase-2-hierarchy-integration/verdict-catalog.md).
`padding-accessors.json` adds 72 pinned independent cases covering all per-side/outer
accessor controls; its separate manifest preserves the generator/source identity. The
original 826 cases and entry-stage inventory labels remain unchanged. `controls.py`
replays 27 immutable set/readback/reset equivalents against the reference inventory.
