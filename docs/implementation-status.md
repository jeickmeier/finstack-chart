# Implementation status

Updated: 6 September 2026. Specification version: 0.1.0.
Bootstrap committed at `fbc9782` (starting commit: `19f4a27`); WP-01 committed at `dfe38e8`.
WP-02 completion: `dfe38e8` plus uncommitted core contracts/tests, target-isolation
checks and documentation updates. No dependency/lockfile change.

## Current handoff

The user assigned WP-02 after WP-01. **WP-02 is DONE for its minimal-contract scope:**
core now provides typed identities/revisions, contextual diagnostics, finite geometry,
bounded immutable scene construction and synchronous text/resource interfaces.
The [completion evidence](evidence/wp-02-completion-2026-09-06.md) records 21 passing
integration tests and one passing Rustdoc example, strict repository/build/lint/docs
checks and implemented-core WASM compilation. Core remains dependency-free. Target graph
isolation and a WASM-only forbidden-dependency negative probe pass.

WP-01's pinned host identities and compatible build examples remain verified; Kit stays
optional and core/export remain isolated. [ADR-002](adr/002-minimal-core-contracts.md)
defines the WP-03/WP-04 handoff. Both packages are now READY; later packages remain
NOT STARTED. Chart compilation, data transactions, rendering/export, wire schema, binding
runtimes and benchmarks remain unimplemented. No complete canonical FIX or G0–G4 gate
is passed by these foundations.

**Open dependency risks:** the WP-01 cargo-deny report contains six unmaintained
transitive packages; no advisory was ignored and the scan was not repeated in WP-02.
Cargo still reports a future compiler
incompatibility in `block` 0.1.6. ADR-001 and the completion evidence retain the exact
findings for upstream follow-up in WP-03 and release disposition in WP-23. Hosted CI,
Linux execution and native visual/runtime capabilities remain unverified.

**Next:** assign WP-03's native/font/export capability spike using the minimal scene and
service contracts. WP-04 data work also has its prerequisites satisfied. This task stops
at WP-02. Licensing/public registry names remain release preparation decisions
in ADR-010; they do not prevent local work.

## Work packages

States: NOT STARTED, READY, IN PROGRESS, IN REVIEW, BLOCKED, DONE. Evidence belongs to
its recorded commit/environment. An unassigned owner means no ongoing agent task.
The final column records outstanding prerequisites/blockers and the next action.

| Package | State | Owner | Commit/PR | Requirement IDs | Evidence | Open work / next action |
| --- | --- | --- | --- | --- | --- | --- |
| WP-01 — Project bootstrap and scope ledger | DONE | Unassigned | `dfe38e8` | SCP-01, SCP-02, SCP-03, ARC-04, QLT-05 | [Completion evidence](evidence/wp-01-completion-2026-09-06.md); [ADR-001](adr/001-host-dependency-and-toolchain.md) | Bootstrap acceptance complete; native capabilities and dependency maintenance follow-up remain WP-03/WP-23 work. |
| WP-02 — Workspace, diagnostics and minimal contracts | DONE | Unassigned | `dfe38e8` + uncommitted WP-02 slice | ARC-01, ARC-02, ARC-03, SCN-01, BND-01, QLT-01, QLT-05 | [Completion evidence](evidence/wp-02-completion-2026-09-06.md); [ADR-002](adr/002-minimal-core-contracts.md) | Minimal contracts accepted; full scene, data, wire/binding and diagnostic aggregation remain later work. |
| WP-03 — Native, font and export capability spike | READY | Unassigned | — | ARC-04, LAY-02, LAY-04, SCN-03, GPU-01, GPU-03, EXP-01, EXP-02, QLT-03, QLT-04 | None | WP-02 prerequisite satisfied; implement and inspect actual native/font/export proofs and benchmark protocol. |
| WP-04 — Immutable data, schemas and transactions | READY | Unassigned | — | DAT-01, DAT-02, DAT-03, DAT-04, DAT-05, DAT-06, ARC-03, QLT-01 | None | WP-02 prerequisite satisfied; implement snapshots, schemas, stable keys and atomic transactions. |
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
| G0 | NOT PASSED | Dependency identities/builds and minimal contracts established; actual primitive/font/export proofs and benchmark protocol still required. |
| G1 | NOT PASSED | Real native chart, headless output, atomic updates, Python/WASM runtime fixtures. |
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
