# Implementation status

Updated: 6 September 2026. Specification version: 0.1.0.
Starting commit: `19f4a27`; bootstrap changes are uncommitted.

## Current handoff

The user authorized infrastructure scaffolding and AI setup, excluding library
implementation. Development now uses `mise.toml` for tool versions and the three
tasks `fmt`, `check` and `test`. INF-01–INF-05 are provisioned; validation is recorded in the
[evidence report](evidence/bootstrap-2026-09-06.md). No chart APIs, exporters, native
rendering, bindings, semantic fixtures or benchmarks are implemented.

WP-01 is partial because exact GPUI/Kit dependency selection remains open. WP-02 has
only its infrastructure subset prepared under this explicit assignment; its behavioral
work remains gated on WP-01. The early scaffolding does not waive that prerequisite.
All other work packages remain NOT STARTED. No G0–G4 gate is passed.

**Next:** finish ADR-001 with a verified host-compatible source pair and reproducible
build setup, then complete WP-02's minimal contracts and diagnostic evidence. Library
implementation requires a subsequent task assignment. Licensing/public registry names
remain release preparation decisions in ADR-010; they do not prevent local work.

## Work packages

States: NOT STARTED, READY, IN PROGRESS, IN REVIEW, BLOCKED, DONE. Evidence belongs to
its recorded commit/environment. An unassigned owner means no ongoing agent task.
The final column records outstanding prerequisites/blockers and the next action.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-01 — Project bootstrap and scope ledger | IN PROGRESS | Unassigned | — | SCP-01, SCP-02, SCP-03, ARC-04, QLT-05 | [Infrastructure evidence](evidence/bootstrap-2026-09-06.md); partial only | Finish exact GPUI/Kit dependency selection (ADR-001). |
| WP-02 — Workspace, diagnostics and minimal contracts | IN PROGRESS | Unassigned | — | ARC-01, ARC-02, ARC-03, SCN-01, BND-01, QLT-01, QLT-05 | [Infrastructure evidence](evidence/bootstrap-2026-09-06.md); partial only | After WP-01, implement minimal contracts and meaningful diagnostics tests. |
| WP-03 — Native, font and export capability spike | NOT STARTED | Unassigned | — | ARC-04, LAY-02, LAY-04, SCN-03, GPU-01, GPU-03, EXP-01, EXP-02, QLT-03, QLT-04 | None | Satisfy prerequisites: WP-02. |
| WP-04 — Immutable data, schemas and transactions | NOT STARTED | Unassigned | — | DAT-01, DAT-02, DAT-03, DAT-04, DAT-05, DAT-06, ARC-03, QLT-01 | None | Satisfy prerequisites: WP-02. |
| WP-05 — Grammar compiler and minimal prepared scene | NOT STARTED | Unassigned | — | GRA-01, GRA-02, GRA-03, GRA-04, GRA-06, GRA-08, SCN-01, SCN-02, DAT-06 | None | Satisfy prerequisites: WP-02, WP-04. |
| WP-06 — Foundational scales, ticks and layout | NOT STARTED | Unassigned | — | SCL-01, SCL-02, SCL-04, SCL-05, LAY-01, LAY-02, DAT-05 | None | Satisfy prerequisites: WP-05. |
| WP-07 — Working standalone GPUI vertical slice | NOT STARTED | Unassigned | — | GPU-01, GPU-02, SCN-03, SCN-04, INT-01, INT-03, QLT-01 | None | Satisfy prerequisites: WP-03, WP-06. |
| WP-08 — Headless export and snapshot foundation | NOT STARTED | Unassigned | — | ARC-02, EXP-01, EXP-02, EXP-03, EXP-04, LAY-04, SCN-03 | None | Satisfy prerequisites: WP-03, WP-06. |
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
| G0 | NOT PASSED | Exact dependencies, minimal contracts, actual primitive/font/export proofs and benchmark protocol. |
| G1 | NOT PASSED | Real native chart, headless output, atomic updates, Python/WASM runtime fixtures. |
| G2 | NOT PASSED | Complete alpha grammar/publication/theme/extension evidence. |
| G3 | NOT PASSED | Interaction, corrections/retention, scheduling and coherent live exports. |
| G4 | NOT PASSED | All required FIX/PERF/platform/accessibility/documentation evidence. |

## Evidence updates

For each completed slice record starting/result revision or uncommitted state, assigned
scope/IDs, exact command and working directory, date, OS/architecture/toolchain,
result/counts, retained artifact paths, limitations and next concrete action. Keep
failed or blocked requirements open. Update the [support matrix](support-matrix.md)
when new environments or capabilities are actually exercised.
