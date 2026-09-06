# ADR-001: Host identity and initial toolchain

Status: PARTIAL — toolchain/host baseline selected; GPUI and Kit dependency identity OPEN.
Date: 6 September 2026. Owners: WP-01 and WP-03. Requirements: SCP-03, ARC-04.

Use macOS Apple Silicon as the first desktop target, matching the specification and
observed development host. Pin Rust 1.97.1 in `mise.toml` with rustfmt, Clippy and the
browser WASM target; that exact toolchain is installed locally. Set initial `rust-version` to the
same version rather than claiming untested compatibility with older compilers.

The repository has no existing application integration or GPUI dependencies. The
migration plan's `gpui-pre`/Kit snapshot remains dated research. No host package is
selected or fetched by this bootstrap. A floating dependency or two incompatible GPUI
identities would violate ARC-04.

Before completing WP-01, inspect the intended host and current upstream manifests,
choose one GPUI package/source/full revision or exact registry version, and verify
Kit resolves that same identity. Record provenance, compiler requirements and exact
standalone/Kit build commands. Reconcile the toolchain pin if the evidence requires it.
Compiling those adapters does not replace WP-03 native/font/primitive proofs.

Reference benchmark class is Apple Silicon at about 1,200 logical pixels. Actual machine
model/chip/RAM/OS/display scale and PERF timing boundaries remain WP-03/ADR-008 work;
no performance claim follows from this architecture assumption.

See [support matrix](../support-matrix.md) and [status](../implementation-status.md).
