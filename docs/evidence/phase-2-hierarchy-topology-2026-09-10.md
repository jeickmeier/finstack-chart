# WP-H02 topology and node operations

Status: COMPLETE for the standalone core topology package. HIR-01/02/FIX-H01-A/B;
portable/registered/chart proofs remain H07/H08. Revision: working tree over `a6caa39`.

One immutable indexed owner serves keyed/nested/custom-child/grouped/table/path inputs.
Exact hierarchy/node occurrence IDs serialize as decimal strings and remain distinct from
parent labels and source payloads. Nodes retain shared Arc payloads, ordered children,
depth/height and optional explicit aggregation. Duplicate unreferenced leaf labels and
anonymous leaves match D3; ambiguous/missing parents, multiple/no roots, disconnected
cycles and duplicate occurrence keys reject. Escaped slash/path imputation and redundant
root removal use the pinned algorithm. Grouped root payload is null/derived; explicit
ordered entries retain key/value pairs and separate child topology, the JSON adaptation
of Map/group input. It does not infer arbitrary object property order.

Breadth-first iteration/visitation, pre/post visitation, ancestors/descendants/leaves,
find, links, shortest paths, own-value sum, leaf count, stable sibling sort and subtree
copy are implemented. Copies use a distinct owner and share immutable payloads, retaining
values while resetting depth/parent context. Callbacks receive checked node/index/root
contexts. Sort failures and invalid sums leave prior snapshots unchanged.

`cargo test -p chart-core --test hierarchy_topology --locked`: **3 tests pass**, including
31 pinned constructor/operation cases, exact callback/traversal results, custom-child and
grouped inputs, large/scoped IDs, shared payloads, atomic failures, a 20,000-node flat chain
and 20,001-node native nested construction. Invalid deep native inputs release iteratively;
an unbounded child iterator stops at the configured budget. Payload nesting/bytes,
node/depth and construction/layout work are bounded. Float-valued aggregates compare
exact numerical values despite JSON integer-versus-float spelling; topology and keys are
exact. Strict core/test Clippy and formatting pass; [logs](phase-2-hierarchy-topology/)
are retained. No binding/native/layout algorithm claim follows from these core tests.

Next: WP-H03 tidy tree/cluster, then weighted rectangle/circle families, H07 and H08.
