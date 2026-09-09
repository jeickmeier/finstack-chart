# WP-AX01 reference entry in progress

Base revision: `fab2505951061eaafe9adb52c86b248ee0dfa6bf` plus the retained Phase 2
working tree. Requirements AXIS-01/07 and FIX-19. WP-14 is accepted; the axis plan
explicitly permits independent contract work after WP-14. This entry proceeded
while the separate WP-S04 macOS runtime qualification waited at the system loader.
It does not close WP-S04 or any axis gate.

The pinned d3-axis 3.0.0 inventory contains four factories and ten public methods.
The new [reference harness](../../tools/reference/node/axis.mjs) invokes actual D3
inside the already installed Chromium 151.0.7922.34 browser. It records 372 cases
at device scales 1/2, plus shared top/bottom and same-side translated guides, exact
DOM identities during rerenders, and replacement of only one guide's scale.
The harness records DOM component attributes, exact semantic values, label strings,
order, getter defaults and returned-array ownership. Independent checks assert eleven
ticks for domain [0,1] with count 10, default offsets 0.5/0 at device scales 1/2,
and preserved shared-scale references after one guide replacement.

All 16 generated corpus/manifest/inventory/license files repeated byte for byte.
[The corpus](../../fixtures/axes/) pins source hashes, browser binary identity,
generator and shared dependency lock. Pinned d3-selection 3.0.0 and d3-transition
3.0.1 plus their dependencies are development-only DOM harness support. The exact
previous shared lock remains archived and hash-verified for historical oracle
provenance. Existing oracle values and production dependencies were unchanged.

Production guide/scale identity migration, the bounded provider interface, primary
handles/name maps/navigation, actual Rust/Python/WASM behavior, timed transitions
and native/publication qualification are not implemented by this entry. WP-AX01
and G-AXIS remain open. Next: implement the minimal independent guide identity and
scale-reference contract, preserving the existing named-axis and navigation meaning.

## Identity migration in progress

[ADR-021](../adr/021-independent-axis-guides.md) records the implemented identity and
compatibility boundary. Core now resolves additional top/bottom and same-side translated
guides over retained positional scales, with original semantic tick values and separate
guide names/handles. Rust primary builders, immutable edits, version-8 definition/plot
round trips and common Python/WASM syntax dispatch are implemented. The first Linux run
passes four guide/legacy-wire tests plus 24 existing layout/scale tests. New primary API
coverage and actual host qualification are running. The provider boundary remains open;
this is partial WP-AX01 evidence and does not close a gate.

The completed identity slice passes five core guide tests plus 39 existing authoring,
host-dispatch, layout and scale tests on Linux. Actual Linux Python and Node/WASM pass
shared top/bottom positions, two translated same-side guides, exact keys, named scale
navigation, immutable scale replacement with stable guide names/IDs, version-8 round
trips, missing-scale rejection and retained PNG output. Navigation queries remain
read-only; the harness explicitly applies their returned windows through `set_windows`.
Strict positive mypy/TypeScript consumers and the full macOS repository check pass.
The 404-test macOS workspace run also includes these five core guide tests. Runtime
provider callbacks, dedicated macOS Python/native guide proofs and full axis parity
remain open. Source/runtime hashes and logs are retained in the entry directory.

With WP-S04 accepted, WP-S07 custom-shape registrations are the next ready shape slice;
this bounded axis identity entry remains partial WP-AX01 work, with provider integration
and the remaining axis packages still required.

The subsequent WP-S08 source snapshot also passes the unchanged `axis_guides.py`
proof in macOS Python 3.14.6. The [runtime identity](phase-2-axis-entry/identity-python-macos.json)
and log retain that dedicated check. The new `chart-gallery --example axis_guides`
also rendered in the actual native app. [The inspected capture](phase-2-axis-entry/native/guides.png)
shows top/lower values aligned with the three source points and the third guide
translated right/down without clipping or collisions; three paints and one layout,
source hashes and the binary identity are retained alongside it. This qualifies the
legacy-profile identity slice. Provider integration and the complete D3 profile
remain open; these additional host results do not close WP-AX01.
