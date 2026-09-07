# Implementation status

Updated: 7 September 2026. Specification version: 0.1.0.
Bootstrap committed at `fbc9782` (starting commit: `19f4a27`); WP-01 committed at `dfe38e8`.
WP-02 committed at `435e127`; WP-03 at `3a86189`; WP-04 at `d0a6c48`.
WP-05 is committed at `3cf1b33`; WP-06 at `f8657fb`; WP-07/08 at `fac148a`.
WP-09/10 are committed at `c5ec829`. Their original reports retain the revision context
from the time the evidence was produced.

## Current handoff

**Owner assignment: complete WP-11 through WP-23 in prerequisite order and commit after
each completed package. WP-11 is DONE; WP-12 is next.** Starting revision was `c5ec829`;
this handoff belongs to the WP-11 completion commit. The prior WP-12 sequencing question
is resolved and its prerequisite-check report is retained as historical evidence.

WP-11 adds checked log/symlog/point/color/supplied-session scales, portable named axis
policies and separate secondary-unit guides; area/ribbon vector fills, interval bars,
heatmaps and validated OHLC with independently validated volume use the shared engine.
[The report](evidence/wp-11-completion-2026-09-07.md) records scope, commands, native and
export captures, precision, runtime fixtures and remaining gates. The
[scale/geometry contract](scale-geometry-contract.md) and
[ADR-005](adr/005-foundational-scales-and-layout.md) record consequential choices.

`mise run fmt`, `mise run check`, `mise run test` and actual `bindings-proof` pass locally.
**133 tests pass** (117 core, 13 export, 1 native conversion and 2 Rustdoc examples).
Twenty-four cases run in each of Rust, Python and actual Node WebAssembly, including 12
new family fixtures. Semantics agree within 1e-12, scenes within 1e-10 points, and SVG
bytes match exactly. All 12 native family views, representative PNGs and an actual vector
PDF render were visually inspected. No new dependency package/version was introduced.

WP-09/10 are committed at `c5ec829` and retain their accepted scopes. G0/G1 remain passed
at recorded scopes; G2–G4 remain open. Existing dependency advisories and the `block`
future-compiler warning remain unresolved. Linux/full accessibility/release packaging
and performance remain unverified. **Next: WP-12 facets, guides and shared layout.**

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
| WP-11 — Required scale and geometry families | DONE | Unassigned | WP-11 completion commit | GRA-06, SCL-01, SCL-02, SCL-03, SCL-04, SCL-05, SCN-01, SCN-03, DAT-05 | [Completion evidence](evidence/wp-11-completion-2026-09-07.md); [contract](scale-geometry-contract.md) | Required families and FIX-01/03/07 scope accepted through native/export/actual bindings; proceed to shared layout in WP-12. |
| WP-12 — Facets, guides and shared layout | READY | Codex | — | GRA-07, GRA-08, SCL-05, LAY-01, LAY-02, LAY-03 | [Prerequisite check](evidence/wp-12-prerequisite-check-2026-09-06.md) | WP-10/11 complete; implement panel scopes, shared/free alignment and compatible guides. Prior prerequisite check is historical. |
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
| G1 | PASSED — minimal portable core | WP-04–08 foundation/native/headless evidence plus WP-09 actual Python/WASM FIX-15/16 runtime comparison; this does not certify full grammar or production host/distribution products. |
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
