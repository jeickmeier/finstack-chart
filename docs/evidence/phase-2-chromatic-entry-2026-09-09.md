# CP-01 complete chromatic reference entry

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 continuation.
CP-01 is COMPLETE for its contract/reference scope. CHR-01/03/05/06 implementation
and G-CHROMATIC remain open; this inventory is not an implementation pass.

[ADR-019](../adr/019-chromatic-catalog-and-evaluation.md) records table/evaluator
ownership, finite outside/reversal behavior, typed IDs, versioned descriptor and
plot migration, and the shared interpolation/color boundary. The existing oracle
lock pins d3-scale-chromatic 3.1.0, d3-interpolate 3.0.1, d3-color 3.1.0 and d3-scale
4.0.2. The [manifest](../../fixtures/parity/d3-scale-chromatic/manifest.json) records
source-file, lockfile, generator and corpus hashes plus actual Node identity. D3 and
ColorBrewer notices are retained with both reference and adapted production tables.

The corpus covers all **76 exports**, **218 actual scheme arrays**, **38 interpolators**
and **160,666 sample rows**. Every ramp has the complete i/4096 grid; Brewer basis
knots and lookup i/256 jumps include adjacent binary64 sides. Analytic families add
independently located output-byte transitions, finite outside inputs, very large
magnitudes, NaN and infinities. Original CSS and exact RGBA are stored together;
invalid reference output stays explicit. Typed direct non-finite input diagnoses.

`mise exec -- node tools/reference/node/chromatic.mjs <output>` independently
regenerates cases, manifest and license byte for byte. The Rust inventory reader
checks every export's disposition, array lengths/channels, sample total and the
independent three-color Blues anchor. Two inventory tests pass. Repeated macOS
loader stalls required running a copy of the unchanged compiled test binary with
`DYLD_SHARED_REGION=private`; its results are retained. This is inventory evidence,
not numerical chromatic parity.

[Generation](phase-2-chromatic-entry/generate.log),
[regeneration](phase-2-chromatic-entry/regenerate.log),
[reader](phase-2-chromatic-entry/rust.log),
[expected gaps](phase-2-chromatic-entry/gaps.json), and
[source hashes](phase-2-chromatic-entry/source-hashes.json) identify the entry snapshot.
CP-02 owns the exact static tables; CP-03 owns evaluator math; actual public host,
chart, guide, update, visual and measured qualification remain CP-04/05.
