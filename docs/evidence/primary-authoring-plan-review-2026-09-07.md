# Primary authoring plan checked against the current library

Date: 7 September 2026. Reviewed clean revision
`c773fcba9a8c846322d08c9c1203fce415e91328`.
HEAD advanced during review to `b631f0e6d7e41722b5774433d616f704234157d7` with
unrelated skill additions only; the reviewed library and plan files did not change.
Scope: the [primary authoring plan](../impl_plans/primary-authoring-api-plan.md),
AUT-01–09 and its fit to the implemented Rust/native/export/binding boundaries.
This checks the plan, not a new implementation of the proposed API.

Follow-up: at the owner's request the plan was updated later on 7 September 2026 to
address APR-01–04 in its ownership/edit/capture/validation contracts and FIX-AUTH01/02/05/06.
The findings below retain their reviewed-revision context. They are addressed at the
planning level; implementation and runtime acceptance remain open.

The overall direction fits the library: retain chart-core's semantics, supply concise
builders, reuse typed runtime operations, and keep host resources outside core.
**Four handoff contracts need tightening before implementation.** They are omissions
or ambiguities in the plan, not demonstrated defects in the existing runtime.

The owner-requested original WP-01–23 completion assumption now matches the ledger's
original-scope completion records. The current release guide describes a local 0.1.0
candidate with publication disabled. Additional parity and primary-authoring work
remain separate. The plan's old `127fe2d` revision is historical context; implementation
should inventory this completed baseline rather than the earlier partial checkout.
This review did not rerun historical full-platform or sustained-performance acceptance.

## Findings

### APR-01 — P1: define a definition-only edit separately from replacing plot data

Requirements: AUT-02/05, DAT-03/04/06, STM-02. Plan evidence: immutable Plot contains
definition and data in section 2, lines 41–47; AP-05 applies immutable Plot edits to a
retained Chart, lines 269–271. It does not specify which embedded data snapshot wins.

Current [ChartView::set_definition](../../crates/gpui-charts/src/view.rs), lines
234–270, validates the candidate definition against **self.source** and keeps current
data. `set_data`, lines 194–232, is a distinct operation with reconciliation.
[Transaction](../../crates/chart-core/src/transaction.rs), lines 79–91, carries explicit
source/expected-version fences for data mutation. This separation is significant.

Counterexample for the proposed API: build a Plot at data revision r0, mount it, append
data through r1, then change its title/theme using a clone of the original Plot. If
applying that Plot also installs its r0 data handle, the edit silently discards the
live append. The current native definition-edit route avoids that behavior.

**Required plan correction:** make ordinary authoring edits replace definition only,
validate them against current data and retain current store/replay/queue ownership.
Define explicit, revision-fenced data replacement separately. If a combined definition/
data edit is supported, specify its candidate validation and atomic commit boundary.
Add r0 Plot → r1 append → title/scale edit → unchanged r1 data/keys/receipts to
FIX-AUTH05, including schema incompatibility and rollback cases. Verdict: **Fail** for
an unambiguous live edit contract; no data-loss bug was reproduced in existing code.

### APR-02 — P2: replace the generic runtime description with an actual ownership map

Requirements: AUT-01/05/06, ARC-02, STM-04/05. Plan evidence: section 2, lines 48–52,
says Chart owns the existing store/reducer/compiler; AP-01, lines 204–216, and AP-06,
lines 280–287, consolidate session ownership without mapping the current owners.

The library has several deliberate lifetimes, not one ready-to-rename runtime:

| Existing owner | Actual responsibility to preserve |
| --- | --- |
| [portable::Session](../../crates/chart-core/src/portable/session.rs), lines 8–15 | Owns DataStore, reducer, compiler, inspector pins, ingestion queue and reconciliation. |
| [ChartView](../../crates/gpui-charts/src/view.rs), lines 121–145 | Owns per-view reducer/compiler/prepared/presented state and host resources; receives immutable source snapshots, not a DataStore. |
| [native Scheduling](../../crates/gpui-charts/src/view/scheduling.rs), lines 14–22, 102–139 | Keeps synchronous admission policy, an executor-owned compiler/cache, active task and latest pending source; the worker returns its compiler. |
| [FigureRequest](../../crates/chart-export/src/request.rs), lines 14–25 | Pins immutable data/state/profile/fonts/registry for deferred publication; no acquisition-time statistical or destination work. |
| [ExportQueue](../../crates/chart-export/src/jobs.rs) | Bounds job/input retention and preserves cancel/drop/outcome lifetimes independently of the view. |

[The live export gallery](../../examples/chart-gallery/examples/live_export_gallery.rs),
lines 23 and 93, owns a DataStore outside ChartView and queues committed snapshots.
The public `ChartInput::new`, `set_data` and `queue_data` support externally managed
sources. Owning all data exclusively inside each new Chart would remove that route or
force copies/duplicate commit authorities. Likewise, one engine does not mean one
mutable Compiler instance shared across a worker and the UI thread.

**Required plan correction:** distinguish default Chart-owned ingestion from an
explicit externally managed snapshot/source route, with one commit authority and
independent per-view state. Specify where shared typed orchestration lives, how the
worker's compiler is loaned/returned, and what remains in the GPUI adapter. Preserve
separate ingestion acceptance, preparation admission and paint acknowledgement.
Move this ownership acceptance into AP-01, with external-source and worker-lifetime
tests, before AP-02 consumes the runtime. Verdict: **Uncertain** until the handoff
chooses these lifetimes; the current library provides reusable machinery for them.

### APR-03 — P2: preserve both current and presented export bases

Requirements: AUT-06/09, EXP-03/04. Plan evidence: section 3, lines 117–119, and
AP-05, line 271, describe live capture only as acknowledged/presented state. AP-06
lists visible/full-domain and interaction policies but not the independent data basis.

[The implemented export protocol](../../crates/chart-export/src/portable/live_export.rs)
has `Basis::Presented` and `Basis::Current`, lines 5–10 and 79–102. Current captures
the session definition/source/state without an origin scene; Presented captures the
retained displayed figure and rejects when none exists. `full_domain`, lines 104–110,
is a separate choice. [The live export contract](../live-export-contract.md) also
distinguishes default clean committed publication from legacy direct capture's
all-interaction behavior.

Counterexample: a worker is still preparing revision r1 while r0 is displayed. The
application can already save either the displayed r0 chart or current committed r1
data. Selecting full-domain does not choose r1; it changes navigation projection.

**Required plan correction:** add typed Presented/Current capture basis alongside
visible/full-domain and interaction inclusion. Keep FigureRequest acquisition cheap,
reuse ExportQueue/ExportJob for deferred work, and preserve the legacy versus new
interaction defaults explicitly. FIX-AUTH06 must assert both data revisions and the
origin-scene manifest, no-presentation behavior, full-domain orthogonality and resource
release. Verdict: **Fail** for explicit coverage of an existing capture option.

### APR-04 — P2: separate native execution validation from portable validation

Requirements: AUT-03/08, ARC-03, BND-01. Plan evidence: AP-01, lines 204–216, extracts
shared construction from the existing sessions; AP-02 build validates operations,
while AP-03/04 are responsible for native/registered extension support.

[Session::with_extensions](../../crates/chart-core/src/portable/session.rs), lines
28–41, calls `validate_portable` before construction and eagerly prepares.
[ExtensionRegistry::validate_portable](../../crates/chart-core/src/grammar/extensions.rs),
lines 208–233, explicitly rejects native-only stat/geom implementations.
Conversely, [ChartInput::with_extensions](../../crates/gpui-charts/src/view.rs), lines
61–71, executes the supplied registry through Compiler without imposing that portable
restriction. The extension tests exercise native-only versus portable/export behavior.

Counterexample: extracting the current Session constructor intact and making every
new Chart use it would reject a native-only custom painter that the native API already
supports. Eager preparation would also violate the plan's build-versus-execute split
if that constructor were reused for `.build()`.

**Required plan correction:** factor shared structural/schema/registry validation from
execution and destination/serialization capability checks. Keep portable validation on
wire import/export or portable destinations; retain the supplied registry through
Plot/Chart and capture lifetimes without serializing implementations. Add native-only
success, portable rejection, unknown/version-mismatch rejection and no-stat-execution
during structural build to AP-01/02 evidence. Reuse existing validator code rather than
adding a parallel validator. Verdict: **Uncertain** until extraction boundaries are
specified; this is a refactor hazard, not a current extension failure.

## What already aligns

- **Pass, architecture fit:** current Cargo metadata has chart-core as the shared
  semantic owner; native/export both use core and chart-text. A new façade crate is
  unnecessary. A typed core API can sit below the existing JSON adapter.
- **Pass, reuse direction:** immutable snapshots, typed source/generated mappings,
  statistics/cache reuse, source reconciliation, ingestion bounds, scheduler admission,
  dense rendering, linked/edit actions and publication requests already exist. AP work
  should expose and reorganize them, not reimplement the original algorithms.
- **Pass, scope separation:** the plan keeps additional D3/GG algorithms with their
  existing owners and does not treat the original-WP completion as full parity.
- **Pass, compatibility direction:** staged forwarding and preserved wire/default
  semantics fit the local 0.1.0 candidate. Package publication remains outside scope.
- **Uncertain, proposed ergonomics:** Data/Plot/Chart builders, generated identity and
  name resolution, broad host-native builders and the new authoring examples still
  need implementation. Their absence is expected and is not a finding against a plan.

Keep AP-00–09. Resolve APR-01/02/04 in the AP-00/01 handoff and APR-03 in the AP-06
capture contract. Add the counterexamples to those packages' acceptance instead of
adding another runtime, release process or broad implementation phase. No change to
numerical defaults, baseline fixtures or the original-WP completion premise is needed.

## Checks run

Working directory `/Users/jeickmeier/Projects/finstack-chart`; Darwin arm64;
Rust 1.97.1, committed lockfile.

- `mise exec -- cargo metadata --no-deps --format-version 1 --locked`: succeeded;
  nine workspace packages inspected. This inventories direct edges, not a full
  target-isolation or runtime proof.
- `mise exec -- cargo test -p chart-core --test streaming --test scheduling --test stage_cache --locked`:
  **16 passed**, zero failed (nine streaming, four scheduling, three cache).
- `mise exec -- cargo test -p chart-export --test live_export --test extensions --locked`:
  **9 passed**, zero failed (five live export, four extension).

The 25 tests validate existing contracts involved in the proposed refactor. They do
not validate unimplemented Plot/Chart behavior. No new Python/WASM runtime, native
window, artifact inspection, Linux execution or sustained performance measurement
was performed. Plan and implementation were left unchanged; this report and the
mandatory review ledger entry are the only repository changes.
