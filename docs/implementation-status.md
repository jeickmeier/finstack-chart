# Implementation status

Updated: 6 September 2026. Specification version: 0.1.0.
Bootstrap committed at `fbc9782` (starting commit: `19f4a27`); WP-01 committed at `dfe38e8`.
WP-02 committed at `435e127`; WP-03 at `3a86189`. WP-04 completion is `3a86189`
plus the uncommitted data/schema/transaction/provenance modules, tests and documentation.

## Current handoff

The user assigned WP-04 after WP-03. **WP-04 is DONE for its data acceptance scope.**
Immutable typed/normalized snapshots, versioned schemas, ordered atomic multi-dataset
commits, bounded replay, stable ordinals/categories and provenance are implemented in core.
[The WP-04 report](evidence/wp-04-completion-2026-09-06.md) records FIX-08/09/16 data
portions, independent operation-model results, ownership release and append chunk sharing.
[ADR-004](adr/004-immutable-data-and-transactions.md) defines the storage, schema, revision,
retention and replay contracts for WP-05 and later consumers.

`mise run fmt`, `mise run check` and `mise run test` pass on macOS arm64: 38 core tests
(1 unit, 37 integration) and 1 Rustdoc example. Core remains dependency-free and synchronous.
No dependencies or capability assets changed. Core WASM compilation passes; Python/WASM
runtime/wire encoding, grammar, public chart rendering/export and chart selection behavior
remain unimplemented. FIX-08/09/16 pass only their WP-04 data portions; G1–G4 remain open.

G0 remains passed for initial macOS architecture/capability scope using the
[WP-03 report](evidence/wp-03-completion-2026-09-06.md),
[ADR-003](adr/003-font-and-renderer-capability-route.md) and
[ADR-008](adr/008-benchmark-protocol.md). The six unmaintained dependency advisories from
WP-03 remain open and were not rescanned in this no-dependency-change slice. `block` 0.1.6
still emits its future-compiler warning. WP-08/23 own maintenance/release disposition.
Linux execution, hosted CI and full screen-reader support remain unverified.

**Next:** WP-05 grammar work is READY. WP-06–09 still require their remaining prerequisites.
This assignment stops at WP-04. Time-window retention/queues, interaction removal events
and sustained-load performance remain WP-15/16/18–22. Licensing/public registry names
remain release preparation decisions in ADR-010.

## Work packages

States: NOT STARTED, READY, IN PROGRESS, IN REVIEW, BLOCKED, DONE. Evidence belongs to
its recorded commit/environment. An unassigned owner means no ongoing agent task.
The final column records outstanding prerequisites/blockers and the next action.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-01 — Project bootstrap and scope ledger | DONE | Unassigned | `dfe38e8` | SCP-01, SCP-02, SCP-03, ARC-04, QLT-05 | [Completion evidence](evidence/wp-01-completion-2026-09-06.md); [ADR-001](adr/001-host-dependency-and-toolchain.md) | Bootstrap accepted; WP-03 capability follow-up complete; dependency maintenance/release disposition remain WP-08/23. |
| WP-02 — Workspace, diagnostics and minimal contracts | DONE | Unassigned | `435e127` | ARC-01, ARC-02, ARC-03, SCN-01, BND-01, QLT-01, QLT-05 | [Completion evidence](evidence/wp-02-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Minimal contracts accepted; full scene, data, wire/binding and diagnostic aggregation remain later work. |
| WP-03 — Native, font and export capability spike | DONE | Unassigned | `3a86189` | ARC-04, LAY-02, LAY-04, SCN-03, GPU-01, GPU-03, EXP-01, EXP-02, QLT-03, QLT-04 | [Completion evidence](evidence/wp-03-completion-2026-09-06.md); [ADR-003](adr/003-font-and-renderer-capability-route.md); [ADR-008](adr/008-benchmark-protocol.md) | Actual proof artifacts, native lifecycle/hooks and starting profile inspected; public integration/full fixtures remain later packages. |
| WP-04 — Immutable data, schemas and transactions | DONE | Unassigned | `3a86189` + uncommitted WP-04 slice | DAT-01, DAT-02, DAT-03, DAT-04, DAT-05, DAT-06, ARC-03, QLT-01 | [Completion evidence](evidence/wp-04-completion-2026-09-06.md); [ADR-004](adr/004-immutable-data-and-transactions.md) | Data portions accepted; chart transforms/selection, queued ingestion/time retention and binding runtimes remain later packages. |
| WP-05 — Grammar compiler and minimal prepared scene | READY | Unassigned | — | GRA-01, GRA-02, GRA-03, GRA-04, GRA-06, GRA-08, SCN-01, SCN-02, DAT-06 | None | WP-02/04 prerequisites satisfied; implement the assigned grammar/compiler slice next. |
| WP-06 — Foundational scales, ticks and layout | NOT STARTED | Unassigned | — | SCL-01, SCL-02, SCL-04, SCL-05, LAY-01, LAY-02, DAT-05 | None | Satisfy prerequisites: WP-05. |
| WP-07 — Working standalone GPUI vertical slice | NOT STARTED | Unassigned | — | GPU-01, GPU-02, SCN-03, SCN-04, INT-01, INT-03, QLT-01 | None | WP-03 satisfied; still requires WP-06. |
| WP-08 — Headless export and snapshot foundation | NOT STARTED | Unassigned | — | ARC-02, EXP-01, EXP-02, EXP-03, EXP-04, LAY-04, SCN-03 | None | WP-03 satisfied; still requires WP-06; reassess font/shaping maintenance at production promotion. |
| WP-09 — Portable schema and executable binding proofs | NOT STARTED | Unassigned | — | BND-01, BND-02, BND-03, BND-04, ARC-02, DAT-01, QLT-01 | None | Satisfy prerequisites: WP-04, WP-05, WP-06, WP-07, WP-08. |
| WP-10 — Complete statistical and position semantics | NOT STARTED | Unassigned | — | GRA-03, GRA-04, GRA-05, GRA-08, DAT-05, DAT-06, QLT-02 | None | Satisfy prerequisites: WP-09. |
| WP-11 — Required scale and geometry families | NOT STARTED | Unassigned | — | GRA-06, SCL-01, SCL-02, SCL-03, SCL-04, SCL-05, SCN-01, SCN-03, DAT-05 | None | Satisfy prerequisites: WP-10, WP-07. |
| WP-12 — Facets, guides and shared layout | NOT STARTED | Unassigned | — | GRA-07, GRA-08, SCL-05, LAY-01, LAY-02, LAY-03 | None | Satisfy prerequisites: WP-10, WP-11. |
| WP-13 — Full themes and publication composition | NOT STARTED | Unassigned | — | THM-01, THM-02, THM-03, LAY-02, LAY-03, LAY-04, EXP-01, EXP-02, EXP-04, GPU-03 | None | Satisfy prerequisites: WP-08, WP-12. |
| WP-14 — Extension contracts and alpha API | NOT STARTED | Unassigned | — | SCP-01, SCP-02, ARC-03, GRA-01, GRA-08, SCN-02, SCN-03, INT-06, BND-01, THM-03, QLT-05 | None | Satisfy prerequisites: WP-09, WP-11, WP-12, WP-13. |
| WP-15 — Complete action reducer and state ownership | NOT STARTED | Unassigned | — | INT-01, INT-02, INT-05, INT-06, SCN-04, STM-02, QLT-01 | None | Satisfy prerequisites: WP-07, WP-14. |
| WP-16 — Hit testing, navigation and selection | NOT STARTED | Unassigned | — | INT-03, INT-04, INT-05, INT-06, SCL-01, SCN-04, STM-05 | None | Satisfy prerequisites: WP-11, WP-15. |
| WP-17 — Linked views, editable annotations and host controls | NOT STARTED | Unassigned | — | INT-01, INT-04, INT-05, INT-06, GPU-03, LAY-03, DAT-06 | None | Satisfy prerequisites: WP-13, WP-15, WP-16. |
| WP-18 — Streaming retention and incremental computation | NOT STARTED | Unassigned | — | DAT-03, DAT-04, DAT-06, STM-01, STM-02, STM-03, GRA-08, QLT-01 | None | Satisfy prerequisites: WP-04, WP-10, WP-15. |
| WP-19 — Bounded scheduling, caches and dense representation | NOT STARTED | Unassigned | — | STM-04, STM-05, SCN-04, GPU-02, QLT-04 | None | Satisfy prerequisites: WP-12, WP-16, WP-18. |
| WP-20 — Coherent exports during live interaction | NOT STARTED | Unassigned | — | EXP-03, EXP-04, DAT-06, STM-02, SCN-04, QLT-04 | None | Satisfy prerequisites: WP-13, WP-17, WP-18, WP-19. |
| WP-21 — Correctness, fidelity and supported-platform hardening | NOT STARTED | Unassigned | — | SCP-03, QLT-01, QLT-02, QLT-03, GPU-02, GPU-03, BND-03, BND-04 | None | Satisfy prerequisites: WP-14, WP-17, WP-20. |
| WP-22 — Measured performance and sustained-load release gate | NOT STARTED | Unassigned | — | STM-01, STM-03, STM-04, STM-05, EXP-03, QLT-04 | None | Satisfy prerequisites: WP-19, WP-20. |
| WP-23 — Production documentation and release readiness | NOT STARTED | Unassigned | — | SCP-01, SCP-02, SCP-03, ARC-04, BND-01, QLT-05, QLT-06 | None | Satisfy prerequisites: WP-21, WP-22. |

## Cumulative gates

| Gate | State | Evidence required next |
| --- | --- | --- |
| G0 | PASSED — architecture/capability scope | WP-01/02/03 evidence and ADRs establish the initial macOS route and starting protocol; this does not pass full requirements, FIX/PERF or release support. |
| G1 | NOT PASSED | WP-04 atomic data evidence available; real native chart, headless chart output and Python/WASM runtime fixtures still required. |
| G2 | NOT PASSED | Complete alpha grammar/publication/theme/extension evidence. |
| G3 | NOT PASSED | Interaction, corrections/retention, scheduling and coherent live exports. |
| G4 | NOT PASSED | All required FIX/PERF/platform/accessibility/documentation evidence. |

## Evidence updates

Before ending every task, including reviews and partial or blocked slices, update this
ledger with its outcome and evidence. Record starting/result revision or uncommitted state, assigned
scope/IDs, exact command and working directory, date, OS/architecture/toolchain,
result/counts, retained artifact paths, limitations and next concrete action. Keep
failed or blocked requirements open. Update the [support matrix](support-matrix.md)
when new environments or capabilities are actually exercised.
