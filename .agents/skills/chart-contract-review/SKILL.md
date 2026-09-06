---
name: chart-contract-review
description: Review finstack-chart code or readiness against specified requirements and acceptance evidence. Use for semantic, architecture, binding, renderer or release reviews; return findings without editing unless asked.
---

# Chart contract review

Read [repository rules](../../../AGENTS.md), the relevant
[specification](../../../docs/spec/gpui-charts-specification.md) sections and
[status ledger](../../../docs/implementation-status.md). Paths here resolve from this
skill's installed `.agents/skills/chart-contract-review/` directory.

Scope the review to the requested changes or WP. Map requirements to implementation
and inspect the contracts at affected boundaries; use the implementation plan's
traceability instead of duplicating its inventory. Review is read-only unless the
user requests fixes. Do not launch other agents automatically.

Select discriminating evidence appropriate to the change:

- Data/statistics/scales: independent small expectations, invalid/non-finite/null
  values, large integer IDs/timestamps, source-filter versus zoom, stable keys and
  transaction atomicity; compare incremental outputs with fresh batch computation.
- Interaction/streaming: presented-scene revision, capture/cancel, controlled-state
  staleness, coherent multi-dataset snapshots and progressing bounded scheduling.
- Rendering/export: actual artifacts, physical dimensions, vector marks, font policy
  and destination measurement; a screenshot alone does not certify semantics.
- Bindings: actual Python and WASM runtime behavior, ownership/disposal, large IDs and
  nulls. A portable compile establishes only compilation, not runtime equivalence.
- Infrastructure: inspect Cargo edges and selected identities, runnable scripts and
  documentation links. Empty crates cannot close WP-02/G0 or certify feature support.

Do not run unrelated matrix work for a narrow documentation change. Report each
material finding with severity, requirement ID, file/line evidence, a concrete
counterexample or missing proof, impact and suggested fix. Label assessed requirements
Pass, Fail or Uncertain; distinguish absent evidence from a demonstrated defect.
Never repair a discrepancy by weakening fixtures, tolerances or the specification.
Conclude with exact checks run and open gates, without claiming untested platform or
performance support. Do not modify baselines or status records during a read-only review.
