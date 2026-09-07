# Coherent live publication

WP-20 implements EXP-03/04, DAT-06, STM-02, SCN-04 and FIX-14 for the original
owner assignment. See [completion evidence](evidence/wp-20-completion-2026-09-07.md).

`FigureRequest` owns a coherent store snapshot, definition, state, explicit font
resources, extension registry and complete publication profile. Acquisition clones
immutable handles and owned configuration; numeric preparation, layout, shaping and
encoding run later on the caller's executor. Core/export create no threads, perform no
file I/O and hold no GPUI/window/interpreter/browser objects. The host saves returned
owned bytes. A native `ChartView::capture_presented` uses the successfully painted
geometry and actual overlay state, retaining its resource and scene identities.

`InteractionCapture` independently includes selection, hover, focus and gesture preview;
its default is clean committed publication. Durable annotations, hidden layers and
view remain included. Legacy direct `FigureSnapshot::capture` preserves its earlier
all-interaction behavior. Full-domain publication removes navigation through a pure
state projection, including viewport gestures, while preserving explicitly requested
annotation/selection previews. It does not dispatch actions or fabricate revisions.
Original and effective state are both recorded in the output manifest.

Publication rebuilds exact retained data at the chosen physical dimensions and DPI.
Native density reduction does not become publication input. A held request continues
to describe its original data, annotation, theme, font and quality configuration while
the live chart accepts updates, retention, definition changes and later captures.

`ExportQueue` admits two jobs by default, with explicit retained-row and input-byte
budgets. All column chunks, an allowance for retained indexes, font bytes and serialized
configuration are charged conservatively per job, including shared inputs. This is
an input-retention bound, not an RSS or encoder-output bound; layout/scene/raster limits
remain separate. The caller chooses execution order and concurrency. Submission
returns an owned `ExportJob` and a cloneable cancellation handle.

Pending cancellation releases inputs immediately. Running cancellation is checked at
capture, preparation and encoding boundaries; an executing phase cannot be forcibly
interrupted. No partial output is returned. Success, error, cancellation, abandoned
jobs, queue disposal and observer panic unwinding release reservations and retained
inputs. Completed cancellation handles retain metadata/status only. Queue counters
report pending/running jobs, rows/bytes, peak jobs and cumulative outcomes.

The version-1 portable `export_control` envelope supports Begin, Cancel and Status.
Begin chooses Presented (default) or Current data/state, publication overrides and
interaction policy. `export_job` consumes the canonical string job ID and returns
owned bytes. These execute through actual Python and Node WASM adapters on the shared
Rust implementation. Portable execution is synchronous when requested; retaining a
job separately allows mutations before execution. Native hosts can use background
workers. Disposal drops retained jobs, while bytes already returned remain usable.

Manifests include source epoch, all dataset/schema revisions, definition, origin scene,
original/effective state, profile and SHA-256 font identities. They intentionally omit
source and font bytes, so reproducing a figure also requires the original inputs and
registered extension implementation. Revision numbers alone are not content hashes.

The finite native/three-host proofs do not satisfy sustained PERF-03/05. Native frozen
resize behavior and the earlier intermittent redraw observation remain WP-21 hardening
work. Expanded parity/authoring and G4 acceptance remain open.
