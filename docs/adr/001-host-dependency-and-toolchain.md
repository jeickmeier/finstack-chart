# ADR-001: Host identity and initial toolchain

Status: ACCEPTED for WP-01 dependency setup; native capability evidence remains WP-03.
Date: 6 September 2026. Owner: WP-01. Requirements: SCP-03, ARC-04, QLT-05.

## Selected identities

Use macOS Apple Silicon as the first desktop host. There is no existing application
integration in this repository, so no external application's GPUI types constrain the
selection. Core/export remain independent of the host. Kit remains optional.

| Workspace name | Published package and exact constraint | Source / provenance |
| --- | --- | --- |
| `gpui` | `gpui-pre =0.3.3` | crates.io; Apache-2.0; published metadata identifies Zed revision `5b055fa789a8b8d38ac951a6e0cde272f66b4495`. |
| `gpui_platform` | `gpui-pre-platform =0.3.3` | Same registry, license and Zed revision; its GPUI/platform-family dependencies use exact `=0.3.3` constraints. |
| `gpui-kit` | `gpui-kit =0.6.0` | crates.io; Apache-2.0; published VCS metadata identifies `longbridge/gpui-kit` revision `94a313a72a2513aee2780240cd322d552b2395f0`. |

The exact registry pins are in the [workspace manifest](../../Cargo.toml); the
[lockfile](../../Cargo.lock) fixes transitive versions and package checksums. No Git
source, floating branch, local upstream checkout or patch is needed. Kit's published
`0.3.1` semver constraints admit GPUI 0.3.3; linked builds verify this pairing. The
resolved graph contains one `gpui-pre` identity and no separate `gpui` package.

Use `default-features = false` for direct GPUI and Kit dependencies. The platform
enables `font-kit` and `runtime_shaders`; Kit enables `component` for actual styled
controls. Kit's components still bring their own assets and upstream host features
transitively. None of those packages enter core/export. The standalone `gpui-charts`
dependency closure contains no Kit. The [repository checker](../../scripts/check_repository.py)
enforces these boundaries, exact direct host pins and the single GPUI identity.

The migration document's older snapshot was rechecked against published sources;
it was not treated as an adopted version. A separate `gpui` release or Zed Git checkout
would introduce a different identity from Kit's published dependency line. The selected
registry pair avoids that integration mismatch and supplies a reproducible local baseline.

## Toolchain and reproduction

[mise.toml](../../mise.toml) pins Rust 1.97.1 (rustfmt, Clippy and the WASM target),
Python 3.14.6 and cargo-deny 0.19.8. Workspace `rust-version` remains 1.97.1; the host
packages do not declare an upstream MSRV, and no older compiler support is claimed.
Verified with macOS 26.5.2 arm64, Xcode 26.6 (17F113), macOS SDK 26.5 and mise 2026.8.3.
Use a full Xcode installation selected through `xcode-select`; runtime shader compilation
is the selected native path. Dependencies are fetched during provisioning/build, not
by portable chart logic.

From the repository root, after `mise trust` and `mise install`:

```sh
mise exec -- cargo build -p chart-gallery --example host_bootstrap --locked
mise exec -- cargo build -p chart-gallery --example host_bootstrap --features kit --locked
mise run check
mise run test
```

Both examples compile real GPUI application/window code and link the macOS platform;
the Kit variant inserts a Kit button/root into the same GPUI `Render` contract and
passes `gpui::App` to Kit initialization. They are build proofs, not chart APIs or
visual fixtures. Interactive launch is optional with `cargo run` in place of `cargo build`.
WP-03 must still exercise and inspect primitives, text, fonts, input and accessibility.

CI runs native builds on macOS and limits Linux compilation/tests to core/export plus
portable core compilation. Hosted CI and other native platforms remain unverified.

## Dependency policy and open release risks

[deny.toml](../../deny.toml) checks registry provenance and declared dependency licenses
for the selected macOS graph including Kit. It permits the encountered license set,
with MPL-2.0 allowances restricted to `cbindgen` 0.28.0 (build tool) and `option-ext`
0.2.0 (transitive helper). These upstream sources are used unmodified; retain their
license/source notices when preparing distribution. Local unpublished packages remain
outside that license check because ADR-010 leaves their license undecided.

License/source checks are part of `mise run check`. Run advisories separately with
`mise exec -- cargo deny --locked check advisories`: the recorded scan fails on six
unmaintained transitive dependencies, with no ignored advisory IDs. This failure remains
visible and is not a release pass. The [WP-01 evidence](../evidence/wp-01-completion-2026-09-06.md)
lists each advisory and a future compiler incompatibility in `block` 0.1.6. Track upstream
replacement/upgrade paths during WP-03 and resolve release disposition in WP-23. These
maintenance findings do not prevent WP-01's dependency/link acceptance or authorize G4.

Reference benchmark class remains Apple Silicon at about 1,200 logical pixels. Actual
machine model/chip/RAM/display scale and PERF timing boundaries belong to WP-03/ADR-008;
no performance claim follows from the build evidence.

Sources inspected: published [GPUI manifest](https://docs.rs/crate/gpui-pre/0.3.3/source/Cargo.toml),
[platform manifest](https://docs.rs/crate/gpui-pre-platform/0.3.3/source/Cargo.toml),
[Kit manifest](https://docs.rs/crate/gpui-kit/0.6.0/source/Cargo.toml) and
[Kit facade source](https://docs.rs/crate/gpui-kit/0.6.0/source/src/lib.rs).
See the [support matrix](../support-matrix.md) and [status ledger](../implementation-status.md).
