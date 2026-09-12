# WP-H01 hierarchy contract and reference entry

Status: COMPLETE for the contract/reference package; HIR-01–08 production parity is
not certified. Revision: working tree over `a6caa39`, after axis/interpolation acceptance.

[ADR-022](../adr/022-hierarchy-topology-and-layout-history.md) fixes single-core topology,
identity/payload ownership, iterative construction, explicit budgets, numerical adaptations,
resquarify history/invalidation and standalone-versus-chart boundary. ADR-006/012 and the
normative hierarchy inventory were read before this decision.

The existing shared lock pins d3-hierarchy 3.1.2; its tag resolves to source commit
`7bea49efdc1093b28a8d60d2f8717d8a17803564`. No production dependency or version changes.
The [machine inventory](../../fixtures/hierarchy/inventory.json) records all 16 exports,
Node methods, every factory method/default, package ownership and seed case links.
[Controls](../../fixtures/hierarchy/controls.json) execute every factory setter, getter,
reset and owned-array readback, with callable identity and mode switches checked.

The [441-case reference](../../fixtures/hierarchy/reference.json) includes nested/custom/
grouped construction, table/path stratification and structural errors, every node operation,
tree/cluster extent/spacing/custom separation, partition padding/rounding, all six tilers,
all treemap padding controls/custom tiling/ratios, three resquarify histories, hierarchical
packing and standalone sibling/enclosure helpers. Generator assertions independently
check traversal, leaf counts, internal own-value sums, partition bounds and enclosure.

Two runs of `mise exec -- node tools/reference/node/hierarchy.mjs` produced byte-identical
reference, inventory, controls and manifest. Node 24.14.0; supplied UTC/en-US policy;
source hashes, package integrity, license and generator fingerprints are retained in
[manifest](../../fixtures/hierarchy/manifest.json). Numerical tolerances were fixed in the
[fixture README](../../fixtures/hierarchy/README.md) before kernel implementation.
Repository documentation/graph validation passes. No Rust hierarchy test, actual binding,
native artifact or performance claim follows from oracle generation. Next: WP-H02
construction/topology/operations, followed by all standalone layout families and H07/H08.
