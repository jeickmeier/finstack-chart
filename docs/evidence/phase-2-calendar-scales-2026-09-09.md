# SP-06 UTC and supplied local calendar qualification

Date: 9 September 2026. Requirements: SCL-04/06/07, DAT-05, BND-01; FIX-20.
Revision: `fab2505` plus retained uncommitted Phase 2 work. The [retained source hashes](phase-2-calendar-scales/source-hashes.json)
identify this package snapshot. SP-06 is COMPLETE for its calendar/time-axis and actual
host resource scope. G-SCALE remains open.

`Calendar` implements explicit/filtered floor, ceil, round, offset and range, automatic
interval selection and nice. UTC and local time use shared Gregorian components;
the legacy UTC implementation consumes that same owner. Supplied local resources
retain zone, owner revision, tzdata identity, bounded UTC coverage and transitions.
The same rules determine geometry and labels, with earlier-fold and forward-gap wall
resolution, elapsed hour/minute offsets and calendar day/month/year offsets.

`TimeScale` and `TimeAxisScale` reuse continuous interpolation and numerical projection.
They retain exact integer origins, piecewise knots, explicit source units, outside
policies and numeric inverse. Native finer-unit ticks/default labels retain precision.
Within the millisecond Date overlap the inverse reproduces D3's weighted absolute
epoch rounding before Date truncation. `TimeFormat` implements explicit locales,
conditional labels, custom directives, padding, ISO weeks and bounded recursive locale
patterns. New calendar axes and time-format-only axes require definition v5.

| Evidence | Result |
| --- | --- |
| Original time inventory | All **10 UTC/local configurations**, integer/Date forward inputs, inverse, ticks/nice and labels pass. Four fractional primitive-number inputs are explicitly outside the integer millisecond API; use a finer source unit. Invalid reference Dates diagnose at the integer result boundary. |
| Original fixed zones | **224 interval configurations and 128 automatic queries** across UTC, New York, Berlin and Lord Howe pass, including leap/month/year and both DST transitions. |
| Expanded pinned oracle | **2,750 filtered interval configurations**, **990 locale/pattern configurations with 16,632 exact labels**, **240 automatic tick/nice/default-label queries**, and **160 mapping configurations with 800 forward and 1,760 exact inverse comparisons** pass. UTC and supplied local UTC are distinct modes; reference count hints include fractional, zero and descending cases. [Corpus](../../fixtures/parity/d3-scale/calendar/cases.json), [manifest](../../fixtures/parity/d3-scale/calendar/manifest.json). Regeneration is byte-identical. |
| Integer/resource boundaries | Seconds/ms/us/ns, large origins, native submillisecond ticks/labels, span limits, checked Date overflow, canonical wire/copy, invalid/unsorted transitions, coverage endpoints, fold/gap resolution, output/work budgets and recursive locale rejection pass. Clamp/omit precede floating conversion. |
| Geometry and navigation | Named calendar axes retain source units, numeric piecewise knots and exact inverse. Piecewise nanosecond zoom preserves the pointer anchor. Visible windows use the common mapping. |
| Publication | Spring/fall New York charts have equal elapsed-hour spacing with skipped/repeated wall hours and distinct offsets. v5 migration, resource revision 42, immutable repeated SVG bytes and large nanosecond geometry pass. Both PNGs visually inspected; matching SVG/PDF are retained. |
| Actual hosts | Python and single-threaded Node WASM independently execute the full expanded corpus with resource revision 7. Both reproduce the Rust spring/fall SVGs byte for byte, retain chart revision 42, exact Python int/WASM bigint timestamps, independent copies, disposal and strict version checks. |
| Repository | `mise run check`, formatting and whitespace checks pass: all-target builds/Clippy, rustdoc, dependency/Markdown checks and WASM compilation. [Log](phase-2-calendar-scales/check.log). |
| Focused regressions | **50 macOS core tests in five suites**, including navigation, legacy scales and layout, plus **two export tests** pass. The same **50 core tests** pass offline in the existing Linux Rust 1.97.1 Debian aarch64 container with read-only source/registry. |

Reference observations exposed descending subsecond/year nice-selection behavior,
month offset rollover and Date inverse rounding. Each fix changed implementation,
without weakening the observed expectations or tolerances. The expanded reference
resource initially covered too little time for 13-year filtered offsets; its coverage
was extended to 1970–2070, while the original fixed-zone corpus remained unchanged.
Date-edge formatting preserves D3's invalid intermediate text for affected week/year
fields. A test adapter initially treated Date-valued inverse positions as plain JSON
numbers; it now applies the reference's Date-to-number input adaptation and checks
invalid Date results as diagnostics. This did not change the reference fixtures.

Commands use the committed lockfile. The focused Rust run is `mise exec -- cargo test
-p chart-core --lib --test scale_calendar --test scales --test scale_numeric --test
layout --locked`; export is `cargo test -p chart-export --test scale_calendar --locked`.
`WASM_BINDGEN=<installed 0.2.128 binary> mise exec -- python scripts/run_time_proofs.py`
builds fresh PyO3 and WASM artifacts, runs reference comparisons and verifies retained
publication equality. Linux repeats the core command with `--offline` in the existing
`rust:1.97.1-bookworm` image. `node tools/reference/node/calendar.mjs <separate-directory>`
regenerates and hashes the pinned independent corpus.

Calendar intervals return checked diagnostics when required intermediate instants
exceed Date/resource coverage; timestamps cannot encode invalid Dates. Projection of
larger native integer epochs is independent from bounded calendar formatting. This
package does not claim process-default timezone discovery, native visual acceptance,
performance measurement, the full public standalone facade/type matrix, aggregate
binding certification or G-SCALE. SP-07 owns the remaining integrated scale proof.

Retained logs: [core](phase-2-calendar-scales/core.log), [export](phase-2-calendar-scales/export.log),
[Linux](phase-2-calendar-scales/linux.log), [runtime runner](phase-2-calendar-scales/runtime.log),
[Python](phase-2-calendar-scales/python.log), [WASM](phase-2-calendar-scales/wasm.log),
[environment](phase-2-calendar-scales/environment.txt). Inspected [spring](phase-2-calendar-scales/artifacts/spring.png)
and [fall](phase-2-calendar-scales/artifacts/fall.png) PNGs are byte-identical to the
fresh final runtime artifacts; their SVG/PDF and portable plots are retained alongside.
