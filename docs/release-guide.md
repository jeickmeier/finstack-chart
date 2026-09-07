# Local release and developer guide

This repository provides a local 0.1.0 candidate with a shared Rust chart engine, a
macOS GPUI adapter and headless SVG/PDF/PNG publication. Python and browser WASM are
executed proof adapters. All Cargo packages remain `publish = false`. See the
[release evidence index](release-evidence.md) for acceptance and open gates; a source
archive is not a production certification.

## Build and run

Install the tools pinned in [mise.toml](../mise.toml), then from the repository root:

```sh
mise trust
mise install
mise run fmt
mise run check
mise run test
mise exec -- cargo run -p chart-gallery --locked
```

Native macOS builds need the selected full Xcode installation described in
[ADR-001](adr/001-host-dependency-and-toolchain.md). Linux runs the headless core,
text and export packages; the [support matrix](support-matrix.md) records the actual
platforms tested. Do not infer Linux native or screen-reader support from compilation.
To verify native Kit integration, run the existing `kit` examples using ADR-001.

For the actual adapters, install Node 24.14.0, Poppler and wasm-bindgen-cli 0.2.128, then:

```sh
mise run bindings-proof artifacts/release-proof
```

The runner builds and executes the Rust, Python and Node WASM implementations against
the same fixtures. [Binding instructions](../fixtures/bindings/README.md) document
`WASM_BINDGEN` and the Python environment. Compilation alone is not binding acceptance.
No wheel, npm package or browser application distribution is promised.

## Example routes

Each native example is runnable with
`mise exec -- cargo run -p chart-gallery --example NAME --locked` in a graphical session.
Examples use supplied fixture fonts; installed system fonts are not a core dependency.

| Goal | Runnable source and contract |
| --- | --- |
| Basic recipe and layered grammar | [Gallery](../examples/chart-gallery/src/main.rs), [family gallery](../examples/chart-gallery/examples/family_gallery.rs), [grammar contract](statistics-contract.md) |
| Facets, themes, rich text and physical layout | [Composition gallery](../examples/chart-gallery/examples/publication_preview.rs), [composition contract](theme-typography-composition-contract.md) |
| Public custom stat/geom and native painter | [External consumer](../examples/custom-extension/src/lib.rs), [extension contract](extension-contract.md) |
| Gestures, exact inspection and selected targets | [Interaction gallery](../examples/chart-gallery/examples/interaction_gallery.rs), [interaction contract](interaction-contract.md) |
| Linked charts, annotations and controls | [Host tools gallery](../examples/chart-gallery/examples/host_tools_gallery.rs), [host tools contract](host-tools-contract.md) |
| Ordered updates and retention | [Streaming gallery](../examples/chart-gallery/examples/streaming_gallery.rs), [streaming contract](streaming-contract.md) |
| Bounded workers and density rendering | [Scheduling gallery](../examples/chart-gallery/examples/scheduling_gallery.rs), [scheduling contract](scheduling-density-contract.md) |
| Export while data continues | [Live export gallery](../examples/chart-gallery/examples/live_export_gallery.rs), [live export contract](live-export-contract.md) |
| Freeze, resize, recover and dispose | [Hardening gallery](../examples/chart-gallery/examples/hardening_gallery.rs), [WP-21 evidence](evidence/wp-21-completion-2026-09-07.md) |

Read these examples as consumers of the core compiler and scene; they do not define
parallel statistics or data semantics. The [alpha API matrix](alpha-api.md) describes
which Rust and portable surfaces exist. The primary-authoring and Phase 2 plans are
separate future work and do not retroactively certify missing APIs.

## Ownership, interaction and publication

Keep normalized data in `DataStore`. A successful transaction commits all operations
atomically; revision fences, stable row keys, replay outcomes and configured retention
are part of the contract. Keep immutable snapshots as long as readers need them.
Releasing a view does not revoke a caller-owned snapshot. [Data contracts](adr/004-immutable-data-and-transactions.md)
and [streaming](streaming-contract.md) explain capacity and failure semantics.

Durable chart state is distinct from transient gesture previews. Dispatch through the
shared action reducer; use the acknowledged scene for scene-dependent events. Controlled
state requires the owner's matching replacement. Freeze pins semantic data/state while
resize can reproject that exact snapshot. Resume explicitly admits the latest state.
See [actions](state-action-contract.md) and [host tools](host-tools-contract.md).

Publication uses explicit physical dimensions, resources and fonts. Capture a coherent
export job before later updates, then prepare/encode with the bounded export queue.
SVG editable text, outlined text, PDF embedding and PNG density have different output
contracts. Native custom painters need an explicit export lowering. Unsupported effects
fail with structured diagnostics; there is no hidden screenshot fallback. See
[publication](theme-typography-composition-contract.md) and [live export](live-export-contract.md).

## Compatibility and provenance

All packages currently share 0.1.0 and Rust 2024/MSRV 1.97.1. GPUI 0.3.3 and optional
Kit 0.6.0 are exact adopted identities; use `Cargo.lock`, not dated reference versions.
Public Rust enum variants and required struct fields can break exhaustive downstream
matches or literals before 1.0. [CHANGELOG](../CHANGELOG.md) records these additions.
Portable specification/state/action/transaction envelopes have explicit version fences
and reject unknown fields as documented in the [portable contract](portable-contract.md).
A crate patch does not silently change a wire version.

Supplied Noto and Fira fixture fonts retain their license files beside the assets;
[ADR-003](adr/003-font-and-renderer-capability-route.md) records renderer provenance,
font embedding and transformations. The package/source policy is in
[ADR-010](adr/010-package-and-release-policy.md). No repository-wide distribution license
or owner copyright grant has been supplied. Resolve those and registry metadata before
publication. Existing dependency advisories and native accessibility limitations remain
visible in the support/evidence records.

## Reproduce measurements and package locally

Use [ADR-008](adr/008-benchmark-protocol.md) for the exact hardware, intervals, owner
30-minute waiver and interpretation of CPU/GPU/presentation timings. The short native
runner uses temporary floating windows to avoid occlusion and closes them after disposal:

```sh
python3 scripts/performance/run_native.py artifacts/performance-check --stream-seconds 60
```

Do not build other workloads while measuring. Keep raw reports and disclose desktop
activity. A short run cannot establish an unmeasured duration or erase a failed trace.

After checks and evidence review, build a deterministic local source candidate from the
committed tree using [the packaging script](../scripts/package_local_release.py):

```sh
python3 scripts/package_local_release.py artifacts/local-release
```

The archive includes the committed lockfile, manifests, source, fixture notices, scripts
and evidence. It excludes unrelated uncommitted work and build outputs. The manifest
records the exact commit and archive SHA-256. Extraction plus the commands above reproduces
the local build inputs; dependency caches/toolchains are not bundled. This command neither
publishes packages nor creates a remote release.
