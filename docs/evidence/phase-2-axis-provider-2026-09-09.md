# AXIS-01 registered positional provider qualification

Revision: `51f2edafdf5ec65121966dedcb1d5ffa838f80de` plus the retained changes identified
in [the source snapshot](phase-2-axis-provider/source-sha256.json). The provider
implementation began over the earlier Phase 2 work; the revision above captured
that work during this task. Scope: AXIS-01/07, FIX-19, AUT-01/04. This extends the
[372-case reference and identity entry](phase-2-axis-entry-2026-09-09.md).

The checked provider is implemented and passes its focused acceptance. Full Linux regression validation passes. Aggregate macOS validation remains incomplete after prolonged compiler waits; runtime suites and core doctests passed before interruption. WP-AX01 remains open for the explicit D3 guide
profile, which is the next integration step with WP-AX02. G-AXIS is not passed.

## Contract and behavior

[ADR-021](../adr/021-independent-axis-guides.md) defines a single retained positional
mapping for marks and all its guides. Native factories register a captured versioned
identity, validate bounded JSON parameters and receive post-stat extent, exact semantic
space, requested range/window/outside policy and work limits. Optional inversion is
independent of numeric output. Metadata, ticks, labels, band extents, mapping results
and inverse values pass through the shared checked adapter. Prepared charts retain
registry snapshots; later registration does not mutate old requests.

The primary Rust/Python/WASM selector is `scale_registered`; the host registry gains
`ExtensionRegistry`/`with_registry` aliases without changing existing shape-registry
ownership. Provider-bearing definition/primary envelopes require version 10 and explicit
registration on load. Native-only implementations reject serialization and headless
publication while remaining executable in native layout. In ggplot2's pre-stat profile,
`coordinate_scale` explicitly selects post-stat projection; implicit provider use rejects.
LibraryV1 already projects after statistics. Generic provider pan/zoom rejects unless a
future provider navigation contract supplies its coordinate metric. Exact supported
windows remain inputs to the factory.

## Evidence

- [Focused core tests](phase-2-axis-provider/core-tests.log): six provider tests,
  five independent-guide tests and ten grammar-stage regressions pass on macOS.
  [Linux](phase-2-axis-provider/linux-tests.log) passes the same 21 tests offline.
- Provider cases independently check `100 + 40*abs(x)` for `x=-2,+2`, one factory
  resolution per scale per layout iteration despite additional guides, retained
  domains and optional inverses, full formatter callback context, empty/repeated
  labels in the checked adapter, exact nanosecond values above 2^53, different layer
  timestamp origins and category catalogs, band centering/extents, nonfinite/type
  errors, count/text limits, registry snapshots and portability rejection.
- Independently authored [Rust](../../crates/chart-export/examples/axis_provider_proof.rs),
  [Python](../../scripts/bindings/axis_providers.py) and
  [WASM](../../scripts/bindings/axis_providers.cjs) proofs execute the same external
  native registration. Actual macOS/Linux Python and Node/WASM check positions, exact
  source keys, version-10 round trips, missing versions, invalid parameters, native-only
  rejection, copied/disposed registries and requests surviving plot disposal.
- All four executions produce byte-identical SVG/PDF/PNG files. The
  [hash comparison](phase-2-axis-provider/publication-comparison.json) records each
  format/host. The native screenshot, PNG, external SVG render and Poppler PDF render were inspected:
  paired folded points align with the top guide, the lower guide is translated by
  10 horizontally and 30 vertically, and labels fit without clipping/collisions.
  [Native capture](phase-2-axis-provider/native.png),
  [PNG](phase-2-axis-provider/rust/provider.png), [SVG render](phase-2-axis-provider/svg.png),
  [PDF render](phase-2-axis-provider/pdf.png). The [inspection record](phase-2-axis-provider/inspection.json)
  retains artifact and external-renderer hashes.
  Actual native instrumentation records three paints and one layout.
- Strict TypeScript positive/negative cases and mypy positive cases pass; mypy rejects
  exactly three invalid version/parameter/registry inputs. Existing independent-guide
  and all-five-protocol registered-shape runtime proofs pass again in macOS Python
  and Node/WASM. These protect shared registry and legacy version-8 behavior.
- The regular primary-authoring proof runner now includes these provider/identity cases
  and typed consumers. Its new provider section was executed through the focused
  commands above; this is not a claim that every unrelated primary proof was rerun.

## Remaining boundaries

The explicit D3 guide profile, per-guide selection/formatter controls, preservation
policy, range-derived geometry/caps/offset, portable component styles and transitions
remain the next axis packages. Existing adaptive thinning is still the legacy guide
policy; checked-adapter label preservation is not a claim that legacy layout retains
all ticks. Providers are trusted native code and cannot be preempted; only explicit
installed portable code can run through host adapters. There is no browser/interpreter
object or duplicate mapping algorithm in core. The external example folds through the
existing linear scale. No broader D3, ggplot2, Linux native or release gate is inferred.

The initial aggregate macOS run was interrupted after compiler processes waited with
minimal CPU progress. A two-job retry in that build directory did not clear the waits.
Validation now uses the same source and commands in the native build directory that
produced the inspected example. The full Linux headless regression suite passes 425 tests/doctests in 84 result
blocks, with none failed or ignored; its [complete log](phase-2-axis-provider/linux-full-tests.log)
is retained. Interrupted logs remain retained; no failed/interrupted run is counted
as passing. A test-only Clippy slice-construction warning was corrected.

The native-target aggregate macOS run was interrupted after 18 minutes, with runtime suites and core doctests passed and adapter doctests still pending. The final repository run was interrupted after more than 10 minutes during all-target checking. Both logs are retained as incomplete evidence. The source hashes matched the retained provider snapshot immediately before the next axis implementation began. Focused provider acceptance stands; aggregate macOS/repository qualification remains an open gate and will be rerun for the integrated axis source.


## Integrated WP-AX01 closure

The [WP-AX01/02 report](phase-2-axis-ticks-2026-09-09.md) supersedes this report's
open profile-entry status. The per-guide compatibility profile and independent tick
controls are implemented and actual host/provider regressions pass. WP-AX01 is complete
for its contract/reference/provider scope. The integrated report records the final
430-test macOS/Linux regression evidence and exact repository-check limits; the older
interrupted runs above remain historical evidence. Full geometry and G-AXIS stay open.
