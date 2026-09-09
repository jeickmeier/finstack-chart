# SP-05 numeric tick and formatting qualification

Date: 9 September 2026. Requirements: SCL-07, LAY-01/02, THM-03; FIX-20.
Revision: `fab2505` plus retained uncommitted Phase 2 work;
[source hashes](phase-2-numeric-format/source-hashes.json) identify this package snapshot.
SP-05 is COMPLETE for shared numeric tick/format and chart integration scope. G-SCALE
and standalone host/native/performance qualification remain open.

Raw candidates and independently prepared labels now cover the numeric, typed continuous,
sequential/diverging and quantize families. Signed logarithmic candidates include minor
values with independently suppressed labels. Numeric axes consume these helpers; layout
retains minor marks while thinning colliding labels. Count hints never replace the
independent hard resource budget. Nice preserves interior knots/diverging midpoints and
rebuilds quantize thresholds. Rank/quantile scales retain their own quantile capabilities.

`NumericFormat` implements the full numeric type/specifier grammar, precision inference,
signs, grouping, alignment, SI and fixed prefixes, with explicit inline locale values.
Fixed/significant decimal rounding shares exact binary-to-decimal arithmetic. Axis
formatters require definition v5, validate before empty publication, reject categories,
and leave legacy formatters and mapped coordinates unchanged.

| Evidence | Result |
| --- | --- |
| Original pinned scale corpus | Exact arrays and strings for **269 numeric configurations**, **1,345 tick queries and 1,345 label queries**. All **200 standalone tick-format cases** pass. **465 additional distribution nice queries** pass; the existing 161 numeric configurations and 805 nice comparisons also remain green. |
| Expanded independent oracle | **192 format/locale configurations, 56 fixed-prefix configurations, 180 log formatter configurations**, plus 90 tick/step boundaries, 24 text cases and seven malformed specifiers. The three formatting matrices cover **11,176 numerical labels**. Four explicit locales, every numeric type, subnormal/extreme inputs, negative zero, decimal ties and missing precision pass. [Corpus](../../fixtures/parity/d3-scale/numeric-format/cases.json), [manifest](../../fixtures/parity/d3-scale/numeric-format/manifest.json). Regeneration is byte-identical. |
| Publication | Explicit currency/decimal locale labels survive v5 round-trip and SVG/PDF/PNG output. Retained SVG bytes reproduce. Wide versus narrow log publication preserves minor marks and thins labels without altering standalone candidates. Formatter-only v5 requirements, invalid empty formatter and category rejection pass. |
| macOS regressions | **84 core tests in 16 suites and 15 export tests in five suites**, all pass. Includes numeric/distribution/category scales, legacy layout, authoring, portable values, paint and shared interpolation. [Core](phase-2-numeric-format/core.log), [export](phase-2-numeric-format/export.log), [final formatter assertions](phase-2-numeric-format/export-current.log). |
| Linux | **33 tests in five core suites** pass in the existing Rust 1.97.1 Debian aarch64 container, offline with read-only source/registry. [Log](phase-2-numeric-format/linux.log). |
| Repository | `mise run check`, formatting and whitespace checks pass: dependency/Markdown checks, builds, all-target Clippy, rustdoc, and WASM compilation. [Log](phase-2-numeric-format/check.log). |
| Visual inspection | Both retained PNGs inspected: correctly placed major labels, minor marks, and deterministic narrow-layout thinning. [Wide](phase-2-numeric-format/artifacts/log-wide.png), [narrow](phase-2-numeric-format/artifacts/log-narrow.png). Matching SVG/PDF artifacts are retained beside them; no native capture is claimed here. |

The first exact comparison found a one-bit libm `exp(1)` difference for base-e ticks;
using the existing correctly rounded exponential implementation fixes it. Expanded
format cases exposed the dependency's optional fixed formatter returning 0.625 at
20 decimals for 1e-25; the shared exact decimal kernel fixes that, with an unchanged
reference expectation. IEEE NaN count propagation, integer infinity text, explicit
empty grouping and undefined fixed prefixes were likewise corrected against the oracle.

The new log-format harness initially used the one-argument D3 constructor (which sets
range), while labeling it as an authored domain. It was corrected to `.domain(...)`
and given an independent domain/tick anchor before qualification. Existing oracle
fixtures and tolerances were unchanged. Initial Rust test issues were corrected:
integer literals, comparing NaN-containing prepared caches instead of immutable
configuration, plain strings instead of categorical inputs, and expecting formatting
validation later than the builder actually performs it. Narrow-layout assertions now
use an explicit two-decimal formatter whose labels actually collide at that width.

Commands: core `mise exec -- cargo test -p chart-core --test scale_ticks_format --test
scale_numeric --test scale_interpolated --test scale_classifier --test scale_categorical
--test scales --test full_scales --test layout --test authoring --test authoring_host
--test portable --test paint_integration --test interpolate_scalar --test interpolate_values
--test interpolate_color --test interpolate_descriptors --locked`; export `--test
scale_ticks_format --test scale_numeric --test scale_categorical --test scale_distributions
--test authoring`; offline Linux `--test scale_ticks_format --test scale_numeric --test
scale_interpolated --test scale_classifier --test layout`. All use the committed lockfile.

These are focused Rust/core/chart/export results, not fresh standalone Python/WASM
scale runtime proof, native acceptance, aggregate Linux certification or a measured
performance claim. SP-06 is next: shared UTC and supplied local-calendar intervals,
time formatting and actual host use of the same timezone revision. SP-07 integrates
all scale families with host, navigation, live-update and native/publication evidence.
