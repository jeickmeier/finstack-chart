# SP-03 ordinal, band and point qualification

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 changes.
Requirements: SCL-01/06/07, DAT-02; FIX-20 with FIX-06/07 regression coverage.
SP-03 is COMPLETE for this package scope. The [source hashes](phase-2-categorical-scales/source-hashes.json) identify this
package snapshot; later scale families and complete G-SCALE acceptance remain open.

Native ordinal scales now accept generic typed outputs and stable typed keys.
Explicit training returns a new catalog; lookup cannot mutate it. Empty ranges and
unknown categories remain undefined or return an explicit unknown value. Portable
keys retain exact decimal signed/unsigned integers and timestamps, with no string
coercion; floating NaNs and signed zeros use SameValueZero membership.

D3 bands and points share one prepared spacing kernel with explicit legacy policies.
Alignment, rounding/rangeRound, padding, step, bandwidth, duplicate-domain ordering,
empty/singleton/zero-width/reversed ranges and category windows are implemented.
Existing chart recipes preserve historical spacing. New D3 named axes use definition
version five and the existing guide, projection, category navigation and oriented
dodge paths. Prepared point inspection uses logarithmic search, including stable
first-category ties when large destination origins collapse adjacent centers.

| Evidence | Result |
| --- | --- |
| Pinned reference | All 130 configurations pass: 62 band, 49 point and 19 ordinal. Band/point coordinates, metrics, normalized options, category order, undefined outputs, implicit training and copy isolation match; rounded values and identities compare exactly. [Core log](phase-2-categorical-scales/core.log). |
| Independent contracts | Singleton legacy center 25 versus D3 center 50/width 50, reversed windows/hits, duplicate identity, collapsed-center ties, generic array outputs, exact large integer keys, signed-zero/NaN membership and decimal-key round trips pass. Same core log. |
| Chart/publication | Four band/point and ascending/descending configurations preserve independently calculated positions and missing dodge slots. Definition v5 round trips; v4 rejects. Category append equals fresh batch geometry while the earlier SVG remains byte-identical. Two tests pass. [Export log](phase-2-categorical-scales/export.log). |
| Regression | 13 existing foundational/full-scale tests and 37 authoring/inspection/statistics tests pass, including prior category identity and fixed dodge slots. Total distinct focused macOS tests: 55 (3 new core, 2 export, 50 regression). [Regression log](phase-2-categorical-scales/regressions.log). |
| Linux | Rust 1.97.1 Debian container, offline/read-only source and registry: 30 focused core/scale/statistics tests pass. The final large-coordinate lookup fix passes all three categorical tests again. [Linux](phase-2-categorical-scales/linux.log), [final lookup](phase-2-categorical-scales/linux-final.log). |

Commands: `mise exec -- cargo test -p chart-core --test scale_categorical --test
scales --test full_scales --locked`; core `--test statistics --test authoring --test
inspection`; export `--test scale_categorical`; corresponding offline Linux cargo
invocations. macOS arm64 and Linux aarch64 use Rust 1.97.1 and the committed lockfile.
`mise run check`, formatting and whitespace checks pass, including all-target Clippy,
rustdoc and WASM compilation. [Repository log](phase-2-categorical-scales/check.log).

This scope does not claim actual Python/WASM standalone categorical APIs, native
captures, inspected visual parity, an aggregate Linux gate or measured performance
budgets. SP-07/WP-21/22 retain those obligations. Next: SP-04 distribution and typed
interpolation/color scales, consuming SP-02/03, CLR-04 and WP-IP03/04.
