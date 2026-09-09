# SP-01 scale contract and reference measurement

Date: 9 September 2026. Revision: `fab2505` plus uncommitted Phase 2 changes.
SP-01 is COMPLETE for SCL-06/07/08, ARC-04, BND-01 and QLT-02 entry scope.
[ADR-018](../adr/018-scale-compatibility-and-resources.md) freezes compatibility,
method adaptations, descriptor migration and supplied timezone/interpolation boundaries.
This stage measures gaps; G-SCALE remains OPEN.

The pinned d3-scale 4.0.2 source resolves to commit
`83555bd759c7314420bd4240642beda5e258db9e`. The shared npm lock and manifest retain
all eight participating packages, integrity, source digests and licenses. The complete
inventory has 26 factories and both helper exports, inherited methods and default
getter results. Generated observations include 461 configured scales, 200 numeric
formatting cases, and 32 local-calendar records across four explicit zones. Local rules
cover 2020–2030, retain Node tzdata/ICU identities and include Lord Howe's half-hour DST.
All 13 fixture/license files regenerate byte identically into a separate directory.
Independent anchors verify unequal knots, constant domains, inverse clamping,
negative-domain logs, count-four ticks and aligned singleton bands.

The executable offline Rust reader reports all 311 constructor/method/helper
positions and every case, with explicit equivalent-result or typed-configuration
dispositions. It currently measures the existing finite two-endpoint linear API;
other families and operations remain explicitly missing/unqualified. Its 176 passing
observations, 63 differences, 707 missing observations and six reference diagnostics
are a measurement snapshot, not six diagnostics accepted as parity and not a feature
pass. For example, domain [1,10], range [0,100], clamp enabled: reference invert(200)
is 10 while the legacy API returns 19. Legacy policy will remain explicit.

| Command/evidence | Result |
| --- | --- |
| `mise exec -- node tools/reference/node/scale.mjs` | 461 + 200 cases and four zone resources generated. [Log](phase-2-scale-reference/generate.log). |
| Same command with `target/scale-reference-regenerated` | All 13 files byte equal, including local resources. [Log](phase-2-scale-reference/regenerate.log). |
| `mise exec -- cargo run -p chart-core --example scale_gap_report --locked -- target/scale-gap-report.json` | Offline measurement completed. [Log](phase-2-scale-reference/rust.log), [complete gaps](phase-2-scale-reference/gaps.json). |
| Focused example Clippy, rustfmt and repository checks | Pass. [Clippy](phase-2-scale-reference/clippy.log), [repository](phase-2-scale-reference/repository.log). |
| [Source hashes](phase-2-scale-reference/source-hashes.json), [source tag](phase-2-scale-reference/source-tag.txt) | Retain the exact fixture/tool/contract snapshot and peeled source identity. |

No production scale behavior or public wire reader changed. No new Python/WASM,
native/publication, Linux or performance scale acceptance is claimed. Next: SP-02
continuous mapping and numeric families through named axes, preserving existing FIX-07
behavior under the explicit legacy policy.
