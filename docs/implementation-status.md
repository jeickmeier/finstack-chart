# Implementation status

Updated: 7 September 2026. Specification version: 0.1.0.
Bootstrap committed at `fbc9782` (starting commit: `19f4a27`); WP-01 committed at `dfe38e8`.
WP-02 committed at `435e127`; WP-03 at `3a86189`; WP-04 at `d0a6c48`.
WP-05 is committed at `3cf1b33`; WP-06 at `f8657fb`; WP-07/08 at `fac148a`.
WP-09/10 are committed at `c5ec829`; WP-11 at `a6fb2ea`; WP-12 at `63dbcc2`.
Original reports retain the revision context from their evidence runs.

## Current handoff

**The owner's original WP-11 through WP-23 assignment is complete, with a separate
commit for every package. Expanded acceptance remains OPEN.** The owner explicitly
waived the 30-minute test; its interrupted trace and numerical failure remain retained.
No package was published and no distribution license was inferred.

WP-23 supplies the [developer/release guide](release-guide.md),
[65-requirement evidence index](release-evidence.md), changelog, package/source policy,
and deterministic unpublished local source-candidate tool. The index explicitly lists
75 additional identifiers from concurrent planning as OPEN and does not certify later
clauses added to original IDs. See [WP-23 evidence](evidence/wp-23-completion-2026-09-07.md).

Final runtime evidence is **219 macOS tests, 213 Linux headless tests**, required
fmt/check/lint/docs/WASM checks and actual Rust/Python/Node WASM fixtures (36 cases,
23 actions, 47 input steps, 70 stream steps, three density cases, 40 live-export steps).
The extracted committed runtime archive passes offline repository/isolation/link checks
and fresh headless all-target compilation. Repeated archives have identical SHA-256.
The final WP-23 archive/manifest is generated under `artifacts/local-release` after commit.

[WP-22](evidence/wp-22-completion-2026-09-07.md) records ten-line p95 hover/frame work
0.092666/11.689291 ms and visible short-stream actual ingest-to-display p95 165.759375 ms.
Exact accounting and concurrent publication drain/release pass. Heavy-dashboard timings,
failed normal-window display traces, allocator high-water and the duration/visibility
limits remain explicit. These results do not certify expanded workloads or current G4.

**Next owner work:** select the separately planned parity/primary-authoring lane and its
acceptance packages. Preserve the open OS accessibility/platform/distribution boundaries,
license metadata and dependency advisories. There is no active implementation package
left in this original assignment; concurrent owner edits remain separate.

## Work packages

States: NOT STARTED, READY, IN PROGRESS, IN REVIEW, BLOCKED, DONE. Evidence belongs to
its recorded commit/environment. An unassigned owner means no ongoing agent task.
The final column records outstanding prerequisites/blockers and the next action.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-01 — Project bootstrap and scope ledger | DONE | Unassigned | `dfe38e8` | SCP-01, SCP-02, SCP-03, ARC-04, QLT-05 | [Completion evidence](evidence/wp-01-completion-2026-09-06.md); [ADR-001](adr/001-host-dependency-and-toolchain.md) | Bootstrap accepted; WP-03 capability follow-up complete; dependency maintenance/release disposition remain WP-08/23. |
| WP-02 — Workspace, diagnostics and minimal contracts | DONE | Unassigned | `435e127` | ARC-01, ARC-02, ARC-03, SCN-01, BND-01, QLT-01, QLT-05 | [Completion evidence](evidence/wp-02-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Minimal contracts accepted; full scene, data, wire/binding and diagnostic aggregation remain later work. |
| WP-03 — Native, font and export capability spike | DONE | Unassigned | `3a86189` | ARC-04, LAY-02, LAY-04, SCN-03, GPU-01, GPU-03, EXP-01, EXP-02, QLT-03, QLT-04 | [Completion evidence](evidence/wp-03-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md); [ADR-008](adr/008-benchmark-protocol.md) | Actual proof artifacts, native lifecycle/hooks and starting profile inspected; public integration/full fixtures remain later packages. |
| WP-04 — Immutable data, schemas and transactions | DONE | Unassigned | `d0a6c48` | DAT-01, DAT-02, DAT-03, DAT-04, DAT-05, DAT-06, ARC-03, QLT-01 | [Completion evidence](evidence/wp-04-completion-2026-09-06.md); [ADR-004](adr/004-immutable-data-and-transactions.md) | Data portions accepted; full transform/selection behavior, queued ingestion/time retention and binding runtimes remain later packages. |
| WP-05 — Grammar compiler and minimal prepared scene | DONE | Unassigned | `3cf1b33` | GRA-01, GRA-02, GRA-03, GRA-04, GRA-06, GRA-08, SCN-01, SCN-02, DAT-06 | [Completion evidence](evidence/wp-05-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Known data-space geometry/semantics accepted; scales/layout and destination scene projection are WP-06; complete grammar/extensions remain later packages. |
| WP-06 — Foundational scales, ticks and layout | DONE | Unassigned | `f8657fb` | SCL-01, SCL-02, SCL-04, SCL-05, LAY-01, LAY-02, DAT-05 | [Completion evidence](evidence/wp-06-completion-2026-09-06.md); [ADR-005](adr/005-foundational-scales-and-layout.md) | Foundational scale/scene/text-layout contracts accepted; actual native/export consumers are WP-07/08, full families/typography/shared layout remain WP-11–13. |
| WP-07 — Working standalone GPUI vertical slice | DONE | Unassigned | `fac148a` | GPU-01, GPU-02, SCN-03, SCN-04, INT-01, INT-03, QLT-01 | [Completion evidence](evidence/wp-07-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md) | Standalone line/point/bar, native text, presented-snapshot inspection, resize and disposal accepted for FIX-01/07/18 subsets; complete interaction/indexing/accessibility remain WP-15–21. |
| WP-08 — Headless export and snapshot foundation | DONE | Unassigned | `fac148a` | ARC-02, EXP-01, EXP-02, EXP-03, EXP-04, LAY-04, SCN-03 | [Completion evidence](evidence/wp-08-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md) | Basic headless formats, explicit resources and minimal FIX-13/14 accepted; full composition/live exports remain WP-13/20. |
| WP-09 — Portable schema and executable binding proofs | DONE | Unassigned | `c5ec829` | BND-01, BND-02, BND-03, BND-04, ARC-02, DAT-01, QLT-01 | [Completion evidence](evidence/wp-09-completion-2026-09-06.md); [ADR-006](adr/006-portable-specification-and-binding-proofs.md) | Version 1 subset and actual Rust/Python/WASM FIX-15/16 accepted; extend builtin coverage through WP-10–14 and full parity at WP-21. |
| WP-10 — Complete statistical and position semantics | DONE | Unassigned | `c5ec829` | GRA-03, GRA-04, GRA-05, GRA-08, DAT-05, DAT-06, QLT-02 | [Completion evidence](evidence/wp-10-completion-2026-09-06.md); [contract](statistics-contract.md); [ADR-005](adr/005-foundational-scales-and-layout.md) | Built-in stats/positions and FIX-02–05 accepted; exact full-recompute fallback declared. Extend families in WP-11 and specialized streaming in WP-18. |
| WP-11 — Required scale and geometry families | DONE | Unassigned | `a6fb2ea` | GRA-06, SCL-01, SCL-02, SCL-03, SCL-04, SCL-05, SCN-01, SCN-03, DAT-05 | [Completion evidence](evidence/wp-11-completion-2026-09-07.md); [contract](scale-geometry-contract.md) | Required families and FIX-01/03/07 scope accepted through native/export/actual bindings; proceed to shared layout in WP-12. |
| WP-12 — Facets, guides and shared layout | DONE | Unassigned | `63dbcc2` | GRA-07, GRA-08, SCL-05, LAY-01, LAY-02, LAY-03 | [Completion evidence](evidence/wp-12-completion-2026-09-07.md); [contract](facet-layout-contract.md) | FIX-06 and facet/shared-layout scope accepted through core/native/export/actual bindings; full typography and composition remain WP-13. |
| WP-13 — Full themes and publication composition | DONE | Unassigned | `f6c41c1` | THM-01, THM-02, THM-03, LAY-02, LAY-03, LAY-04, EXP-01, EXP-02, EXP-04, GPU-03 | [Completion evidence](evidence/wp-13-completion-2026-09-07.md); [contract](theme-typography-composition-contract.md) | FIX-12/13 accepted through actual core/native/export/bindings; cumulative G2 remains WP-14. |
| WP-14 — Extension contracts and alpha API | DONE | Unassigned | `1cb9557` | SCP-01, SCP-02, ARC-03, GRA-01, GRA-08, SCN-02, SCN-03, INT-06, BND-01, THM-03, QLT-05 | [Completion evidence](evidence/wp-14-completion-2026-09-07.md); [contract](extension-contract.md); [alpha matrix](alpha-api.md) | FIX-17 and cumulative G2 passed; full reducer/interaction begins WP-15. |
| WP-15 — Complete action reducer and state ownership | DONE | Unassigned | WP-15 completion commit | INT-01, INT-02, INT-05, INT-06, SCN-04, STM-02, QLT-01 | [Completion evidence](evidence/wp-15-completion-2026-09-07.md); [contract](state-action-contract.md); [ADR-007](adr/007-actions-gestures-and-controlled-state.md) | Deterministic action/controlled/gesture/history/lifetime scope accepted through actual native/export/Python/WASM. Input producers and full G3 remain WP-16–20. |
| WP-16 — Hit testing, navigation and selection | DONE | Unassigned | `4148793` | INT-03, INT-04, INT-05, INT-06, SCL-01, SCN-04, STM-05 | [Completion evidence](evidence/wp-16-completion-2026-09-07.md); [contract](interaction-contract.md) | Assigned indexed inspection/navigation/selection acceptance passes; FIX-09/10 remaining portions and G3 stay with WP-17–20. |
| WP-17 — Linked views, editable annotations and host controls | DONE | Unassigned | `6779e44` | INT-01, INT-04, INT-05, INT-06, GPU-03, LAY-03, DAT-06 | [Completion evidence](evidence/wp-17-completion-2026-09-07.md); [contract](host-tools-contract.md) | Original linked/editing/host acceptance passes; native accessibility limitations recorded. Full G3 remains open. |
| WP-18 — Streaming retention and incremental computation | DONE | Unassigned | `30ca2e8` | DAT-03, DAT-04, DAT-06, STM-01, STM-02, STM-03, GRA-08, QLT-01 | [Completion evidence](evidence/wp-18-completion-2026-09-07.md); [contract](streaming-contract.md) | Original queue/retention/incremental/follow acceptance passes; 70-step three-host replay and native lifecycle inspected. Sustained PERF and G3 remain open. |
| WP-19 — Bounded scheduling, caches and dense representation | DONE | Unassigned | `e57246d` | STM-04, STM-05, SCN-04, GPU-02, QLT-04 | [Completion evidence](evidence/wp-19-completion-2026-09-07.md); [contract](scheduling-density-contract.md) | Original bounded-worker/cache/density acceptance passes; actual four-chart progress and three-host dense proofs. Preliminary PERF identifies index/memory bottlenecks; intermittent native redraw question retained. |
| WP-20 — Coherent exports during live interaction | DONE | Unassigned | `9d86ef7` | EXP-03, EXP-04, DAT-06, STM-02, SCN-04, QLT-04 | [Completion evidence](evidence/wp-20-completion-2026-09-07.md); [contract](live-export-contract.md) | Original coherent capture/bounded lifetime acceptance passes; actual 40-step three-host replay and finite native exports during 400 atomic commits. Sustained PERF and native hardening remain open. |
| WP-21 — Correctness, fidelity and supported-platform hardening | DONE (original scope) | Unassigned | `4c099ee` | SCP-03, QLT-01/02/03, GPU-02/03, BND-03/04, FIX-01–18 | [Completion evidence](evidence/wp-21-completion-2026-09-07.md); [ADR-009](adr/009-supported-platform-and-accessibility.md) | Original acceptance passes including fixed native frozen resize/redraw and actual macOS/Linux/headless bindings. Expanded parity/authoring acceptance remains open and requires the separately listed packages/gates. |
| WP-22 — Measured performance and sustained-load release gate | DONE (original scope; owner duration waiver) | Unassigned | `c58f5b2` | STM-01/03/04/05, EXP-03, QLT-04, PERF-01–05 | [Completion evidence](evidence/wp-22-completion-2026-09-07.md); [ADR-008](adr/008-benchmark-protocol.md) | Visible short budgets and exact accounting pass; interrupted/occluded failures and memory limits retained. Thirty-minute duration explicitly owner-waived. Expanded workloads and current G4 stay open. |
| WP-23 — Production documentation and release readiness | DONE (original local candidate scope) | Unassigned | Included in this completion commit | SCP-01/02/03, ARC-04, BND-01, QLT-05/06 | [Completion evidence](evidence/wp-23-completion-2026-09-07.md); [release index](release-evidence.md); [guide](release-guide.md) | All original package handoffs complete; source candidate remains unpublished. Expanded parity/authoring and current production G4 remain OPEN; duration waiver, failed evidence, platform/accessibility/metadata limits are explicit. |

## Cumulative gates

| Gate | State | Evidence required next |
| --- | --- | --- |
| G0 | PASSED — architecture/capability scope | WP-01/02/03 evidence and ADRs establish the initial macOS route and starting protocol; this does not pass full requirements, FIX/PERF or release support. |
| G1 | PASSED — minimal portable core | WP-04–08 foundation/native/headless evidence plus WP-09 actual Python/WASM FIX-15/16 runtime comparison; this does not certify full grammar or production host/distribution products. |
| G2 | PASSED — Cartesian/publication alpha | [Alpha matrix](alpha-api.md) maps complete grammar/facets/themes/publication/extensions and actual portable evidence. G3/G4 retain their remaining scope. |
| G3 | PASSED (original interactive streaming scope) | WP-15–20 interaction/streaming/export plus WP-21 frozen-resize and native redraw regression fixes; actual native and Rust/Python/WASM evidence. Sustained PERF and expanded parity remain separate. |
| G4 | OPEN — local candidate only | All required FIX/PERF/platform/accessibility/documentation evidence. |

## Evidence updates

Before ending every task, including reviews and partial or blocked slices, update this
ledger with its outcome and evidence. Record starting/result revision or uncommitted state, assigned
scope/IDs, exact command and working directory, date, OS/architecture/toolchain,
result/counts, retained artifact paths, limitations and next concrete action. Keep
failed or blocked requirements open. Update the [support matrix](support-matrix.md)
when new environments or capabilities are actually exercised.
