# SP-07 integrated scale acceptance

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 work.
[Source hashes](phase-2-scale-integration/source-hashes.json) identify this snapshot.
SP-07 is COMPLETE; G-SCALE passes for the declared typed FIX-20 scope. This closes
SCL-01–08 integration for these fixtures, not the remaining G-PARITY, WP-21 platform,
WP-22 sustained performance or WP-23 release gates.

The core owns immutable standalone scale construction, queries and changes for all
26 pinned factories. Python and WASM expose owned typed facades and chart adapters;
formatting, interpolation, exact timestamps, training and classification remain in
Rust. [ADR-018](../adr/018-scale-compatibility-and-resources.md) and the
[authoring guide](../authoring-guide.md) document typed keys, explicit ordinal
training, supplied timezone resources, exceptional values, integer Date boundaries,
unsupported inverses and immutable copies. Advanced descriptors remain validated.

| Contract | Recorded evidence |
| --- | --- |
| Complete original scale/helper operations | 661 cases, 19,562 operations and 1,112 explicit typed adaptations pass in Rust and twice per actual host: raw operations and public facade. All 275 expected capability/input diagnostics match. No skipped required cases. The committed operation translation retains the independent D3 expectations. |
| Expanded public calendar coverage | 2,750 interval, 990 locale/format, 240 automatic and 160 mapping configurations pass through both public facades. Exact bigint timestamps, resource revisions, copied/disposed values and retained spring/fall SVGs match Rust. [Python](phase-2-scale-integration/time-python.log), [WASM](phase-2-scale-integration/time-wasm.log). |
| Authoring and guides | Five independently authored figures per host exercise piecewise and power axes, secondary axes, rounded categorical bands/free facets, eligible ordinal/quantile colors, numerical threshold sizes, asymmetric diverging colors/opacity and supplied local time. Typed field and expression source routes pass. |
| Interaction and ownership | Numeric and nanosecond pointer anchoring, brush selection, exact row-key inspection and linked domains pass in both actual hosts. Old prepared requests survive owner disposal. [Python](phase-2-scale-integration/publication/python-interaction.json), [WASM](phase-2-scale-integration/publication/wasm-interaction.json). |
| Population updates | Append, upsert, remove and count retention match fresh batch in single/faceted ordinal and quantile charts: 16 transitions per host, full PNG equality and retained old requests. Independent core assertions cover repeated numerical samples, broadcast layers, exact unsigned keys above 2^53 and figure-wide first-seen category training. |
| Publication | All five Rust/Python/WASM PNGs match exactly in every RGBA channel. Actual SVG renders, PDFs and native charts were inspected. All PDFs embed supplied Noto Sans. [Comparison](phase-2-scale-integration/publication-checks-final.json), [PNG sheet](phase-2-scale-integration/contact-1.png), [PDF sheet](phase-2-scale-integration/pdf-contact-1.png), [native capture](phase-2-scale-integration/native.png), [paint log](phase-2-scale-integration/native.log). |
| Runtime and types | Complete primary runtime proof passes, including existing color/interpolation/paint/stage consumers; mypy and TypeScript accept valid consumers and reject six invalid scale cases each. [Log](phase-2-scale-integration/primary.log). The subsequently added public calendar runner steps were executed separately against the same final binaries and their source artifacts. |
| Repository/platform | `mise run check` passes; `mise run test` passes 343 macOS tests. Focused offline Linux core execution passes 40 tests in the library and seven integration suites; six focused export tests pass. [Check](phase-2-scale-integration/check.log), [macOS](phase-2-scale-integration/macos.log), [Linux](phase-2-scale-integration/linux.log), [export](phase-2-scale-integration/export.log). |

Integration exposed three defects that now have discriminating checks: clamped
piecewise axes used full metadata endpoints rather than the effective knot span;
free facets trained mapped populations independently; and numeric threshold cuts
could be compared as typed categories when a host inferred integral data. The fixes
share the effective clamp, train mapped populations once across eligible figure rows,
and distinguish numeric cuts from explicitly typed integer/category cuts. Default
classifier labels increase precision when distinct adjacent cuts would collide;
exact interval metadata is retained. Existing fixtures/tolerances were preserved.

Release-profile kernel measurements include one warmup and seven samples. Median
piecewise forward lookup is 8.06 / 14.60 / 27.76 ns for 2 / 128 / 4,096 knots;
typed ordinal lookup is 29.60 / 68.00 ns for 1,000 / 100,000 keys; quantile retrain
and preparation is 9.45 microseconds / 2.14 milliseconds for 1,000 / 100,000 samples.
[Raw measurements](phase-2-scale-integration/benchmark.json) include min/max and
operation counts. WASM capacity stays at 3,211,264 bytes across six collected batches
covering 1,080 owned scale handles. This is memory plateau evidence, not a native
allocation count or a PERF budget pass. These workloads feed WP-22.

Commands use the locked Rust graph and an isolated `CARGO_TARGET_DIR` to avoid the
editor's concurrent default-target builds. The complete runner is
`WASM_BINDGEN=<0.2.128 executable> TSC_JS=<installed tsc.js> mise exec -- python scripts/run_primary_authoring_proofs.py <output>`
with task-local mypy on `PYTHONPATH`. Kernel timings use
`cargo run -p chart-core --example scale_benchmark --release --locked`.
[Environment](phase-2-scale-integration/environment.json) records actual versions.
The Linux run uses the existing Rust 1.97.1 Debian bookworm aarch64 image offline,
with read-only source and registry; it is focused Linux evidence.

The retained [SVG inspection probe](phase-2-scale-integration/svg-render-probe/src/main.rs)
loads the same supplied font and aliases embedded `ChartFont-1-0` to Noto Sans in
memory for resvg 0.48.1, which does not load SVG embedded font declarations. Stored
SVGs are unchanged; the probe and lockfile make that inspection adaptation explicit.
Representative retained [numeric SVG](phase-2-scale-integration/publication/rust/numeric.svg)
and [local-time SVG](phase-2-scale-integration/publication/rust/local-time.svg) sit
beside all host definitions, scenes, PNGs and PDFs. Remaining package: CLR-05, then
other Phase 2 lanes in dependency order.
