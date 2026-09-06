---
name: chart-work-package
description: Plan or implement a bounded finstack-chart work package using its specification, prerequisites and evidence ledger. Use for assigned WP work; preserve infrastructure-only and planning-only scope.
---

# Chart work package

Read the repository [rules](../../../AGENTS.md), [status](../../../docs/implementation-status.md),
and assigned section of the [implementation plan](../../../docs/impl_plans/gpui-charts-implementation-plan.md).
Paths here are relative to this skill after installation in `.agents/skills/chart-work-package/`.

1. Identify the user's mode: plan, infrastructure or implementation. Record WP ID,
   requirement/FIX/PERF IDs, owned paths and exclusions. The general handoff never
   expands the current assignment. Inspect live changes before choosing a slice.
2. Read the relevant [normative contracts](../../../docs/spec/gpui-charts-specification.md)
   and ADRs. Check prerequisite behavior and evidence, not directory existence. If
   unavailable, state the missing interface/evidence and continue only independent
   authorized work. Ordinary conforming decisions do not require another approval.
3. Deliver one coherent slice. Keep shared computation in core, host behavior in its
   adapter, and recipes on the common engine. Avoid unimplemented public API forests.
   Infrastructure work stops at tooling/package/documentation readiness.
4. Run the applicable commands in the repository rules. Use independent expectations
   and explicit tolerances; compile-only or zero-test results do not prove behavior.
   When host/export/binding checks matter, require their actual runtime/artifacts.
5. For implementation, update the ledger with scope, commands/environment, evidence,
   limitations and next action. For planning-only work, describe intended evidence
   without marking it run. Do not close a package or cumulative gate from a partial slice.

Report what changed, which requirements were checked, the results and unresolved
interfaces. Record consequential conforming choices in an ADR; report specification
conflicts without weakening required behavior. Use the existing traceability table
rather than duplicating the full project backlog.
