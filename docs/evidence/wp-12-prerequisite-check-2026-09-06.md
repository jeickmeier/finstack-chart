# WP-12 prerequisite check — 6 September 2026

Requested scope: WP-12 facets, guides and shared layout (GRA-07/08, SCL-05,
LAY-01/02/03; FIX-06). Checked clean revision `c5ec829`, which commits WP-09/10.
No chart implementation changes were made during this prerequisite check.

The [implementation plan](../impl_plans/gpui-charts-implementation-plan.md#wp-12--facets-guides-and-shared-layout)
requires WP-10 and WP-11. WP-10 has committed implementation and completion evidence.
WP-11 remains READY and unassigned, with no implementation/completion evidence in this checkout.
This is confirmed by source, not only the ledger:

- [AxisScale](../../crates/chart-core/src/layout/types.rs) exposes Auto/Linear/Band/UTC;
  log/symlog/session-time, color-guide metadata and secondary-axis contracts are absent.
- [Geom](../../crates/chart-core/src/grammar/definition.rs) exposes point/line/rule/rectangle;
  required area/ribbon/cell/OHLC families are absent.
- The existing compiler and bounded destination layout are usable foundations, but complete
  guide compatibility and shared/free scale verification depend on WP-11's completed scale
  and geometry contracts.

Independent WP-12 work can define/test facet identities, row partitioning, explicit
broadcast/target rules and empty-panel policy against the existing families. It would be
partial work and would not close WP-12 or G2. Completing WP-11 first expands the current
package assignment, so the owner was asked to choose that sequence or independent WP-12
work before dependent implementation begins.

Verification: `git status --short` (clean), `git log -3 --oneline`, direct reads of the
plan/status/source enums and scoped source/evidence searches. No runtime tests were run;
this check changes documentation only. WP-12 completion remains blocked by the missing
WP-11 prerequisite. Next action: resolve sequencing, then implement the authorized slice.
