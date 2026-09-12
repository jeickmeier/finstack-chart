# Support and evidence matrix

Updated 8 September 2026. [Release evidence](release-evidence.md), [WP-21 evidence](evidence/wp-21-completion-2026-09-07.md)
and [ADR-009](adr/009-supported-platform-and-accessibility.md) define the original
implemented scope. Expanded parity and production release gates remain open.

| Surface | Executed evidence | Limits |
| --- | --- | --- |
| Rust core/text/export on macOS | Rust 1.97.1, macOS 26.5.2 arm64; 176 core tests, 33 export tests, four Rustdoc cases; actual portable fixtures | [WP-22 measured results](evidence/wp-22-completion-2026-09-07.md); owner waived 30-minute duration |
| Rust core/text/export on Linux | Local Debian Bookworm aarch64 container, Rust 1.97.1; 213 tests, all-target checks/docs, full native headless fixture runner and cross-host oracle comparison | No Linux native GPUI; x86 remote CI is configured, not locally executed |
| Native GPUI | macOS Apple Silicon, GPUI/platform 0.3.3; original families, themes, composition, controls, input/editing, streaming/export and inspected frozen resize/recovery | Other desktop hosts and full assistive conformance unverified |
| Optional Kit | Opt-in gallery `kit` feature with Kit 0.6.0 and the same GPUI identity; linked build/Clippy and inspected controls | No library crate and no independent chart semantics |
| Python headless proof | CPython 3.14.6, PyO3 0.29.2; actual 36 cases, 23 actions, 47 input steps, 70 stream steps, three density cases, 40 live-export steps | Proof adapter; no wheel/notebook distribution certification |
| WebAssembly proof | wasm32 core/export compile; actual Node 24.14.0 / wasm-bindgen 0.2.128 executes the same fixtures, exact identities/times and memory-growth/disposal tests | No browser viewer/DOM/accessibility or package distribution claim |
| SVG/PDF/PNG | Original primitives, explicit fonts, physical sizes, clips, vector publication, themes/composition and coherent live capture; shared host outputs compared | Supplied fonts required; browser SVG font policy remains consumer-specific |
| Accessibility | Actual native keyboard inspection/selection/editing, chart summaries, exact data-table alternatives, status/buttons and annotation values; WP-17 tree artifacts | OS inspector reports window focus; VoiceOver speech and full traversal unverified |
| Streaming and scheduling | Atomic corrections/removals/retention; bounded preparation; exact raw lookup and declared visual reduction; fixed native idle redraw and coherent exports with released resources | Visible short PERF-03 p95 165.759 ms; full duration not claimed, interrupted display failure retained |
| Primary authoring | [AP qualification](evidence/primary-authoring-completion-2026-09-08.md): 252 macOS / 246 Linux tests, 34 three-host component families, 23 action + 47 input + 70 stream steps, actual native/Kit and publication checks | Final native performance qualification pending; existing platform/assistive and distribution limits remain |
| Additional parity | Separately planned D3 and ggplot2 work | Not certified by original WP-21 or primary authoring |

The [alpha API matrix](alpha-api.md) identifies original supported families and recipes.
The [portable contract](portable-contract.md), [streaming contract](streaming-contract.md),
[scheduling/density contract](scheduling-density-contract.md),
[host tools contract](host-tools-contract.md) and [live-export contract](live-export-contract.md)
explain data ownership, state and host-specific behavior. Actual portable runtime checks
are a separate `bindings-proof` task; zero-test packages and compilation do not pass them.

`mise run check` builds the native/Kit variants on macOS and isolates core/text/export on
Linux. The CI workflow runs those matrices plus actual Rust/Python/Node WASM fixture
comparisons with pinned CLI versions. CI configuration does not imply a remote CI pass.

Six previously recorded unmaintained transitive packages and the `block` 0.1.6 future
compiler warning remain unresolved release risks. No package has been published.
