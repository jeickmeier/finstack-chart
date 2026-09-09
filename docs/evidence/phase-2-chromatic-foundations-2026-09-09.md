# CP-02/03 exact catalog and evaluator foundations

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 continuation.
[Source hashes](phase-2-chromatic-foundations/source-hashes.json) identify this snapshot.
CP-02 and CP-03 are COMPLETE for native core scope. Actual Python/WASM catalog APIs,
chart integration and final G-CHROMATIC certification remain CP-04/05.

The core exposes checked scheme/interpolator IDs and metadata, all 218 separately
authored tables and all 38 prepared evaluators. The extraction script reads literal
packed tables from pinned upstream source, independently of the executable oracle's
returned samples. Every discrete array, order, alpha byte, supported size, reversal
and owned copy passes. Invalid IDs and sizes diagnose rather than synthesizing data.
Brewer ramps reuse the shared RGB basis; lookup ramps retain all 256 step entries;
polynomial, long-hue Cubehelix and cyclic recipes use their pinned formulas and
individual outside behavior. Non-finite direct parameters diagnose.

All **160,666 reference sample rows** pass with exact RGBA. Tests include both sides
of lookup/basis knots, analytic byte transitions, large finite parameters, reverse
composition and explicit non-finite inputs. A further **27,434 trigonometric anchors**
match the pinned Node/V8 oracle bit for bit, including exceptional and signed-zero
states. The new fixed shared color trigonometry removes platform variation without
adding host math or weakening byte/channel/format expectations.

The initial native libc and libm results differed by one byte at 11 sampled boundary
rows on Linux and seven on macOS. Source comparison isolated sine/cosine rounding.
The [pinned V8 fdlibm source](https://github.com/v8/v8/blob/13.6.233/src/base/ieee754.cc)
and a retained stand-alone C++ probe showed that the Node arm64 reference uses fused
operations. The explicit Rust `mul_add` sequence reproduces that behavior on every
host, rather than depending on a compiler's contraction setting. The original
unfused port already matched sampled colors but failed the added exact helper
anchors; the final fused implementation passes both. The safe binary64 argument
reducer retains its source notice and checked indexing; V8/Sun licenses are retained.

Nineteen macOS tests pass across the library, chromatic, color and color-interpolation
suites. The offline Linux run adds two inventory tests: 21 total. The existing 351
color cases and all applicable color interpolation cases pass unchanged.
`mise run check` passes, including repository/Markdown/dependency checks, all-target
Clippy/builds, rustdoc and WASM compilation. This is not fresh binding execution.

Commands: `mise exec -- cargo test -p chart-core --lib --test chromatic --test
color_parity --test interpolate_color --locked`; Linux repeats that command plus
`--test reference_inventory --offline` in Rust 1.97.1 Debian bookworm aarch64 with
read-only source/registry. The development oracle generators are
`tools/reference/node/chromatic.mjs` and `chromatic-trig.mjs`; both regenerate into
a separate directory byte-identically. The diagnostic C++ probe uses
`clang++ -std=c++20 -O2 -ffp-contract=fast`; the unfused comparison uses `off`.

[Core log](phase-2-chromatic-foundations/core.log),
[Linux](phase-2-chromatic-foundations/linux.log),
[repository check](phase-2-chromatic-foundations/check.log),
[probe source](phase-2-chromatic-foundations/chromatic-v8-probe.cc), and
[probe extraction](phase-2-chromatic-foundations/chromatic-v8-probe.py) retain the
validation boundary. No named chart/native/publication or performance pass is
claimed. Next: CP-04 portable catalog and chart/guide integration, then CP-05.
