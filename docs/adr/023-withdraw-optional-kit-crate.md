# ADR-023: Withdraw the optional Kit adapter crate

Date: 11 September 2026. Status: ACCEPTED. Requirements: SCP-01, ARC-01, ARC-04,
AUT-06, AP-06.

## Decision

Remove `gpui-charts-kit` from the workspace. Kit is not a library surface. Hosts that
use GPUI Kit map semantic tokens into `ThemePatch` and mount with `ChartInput::from_plot`.
The composition gallery keeps that recipe behind its optional `kit` feature. Do not
fold Kit types into `gpui-charts`.

Standalone native charts, headless export and language bindings never depended on the
crate. The former A-KIT row is withdrawn; [E4](../evidence/primary-authoring-completion-2026-09-08.md)
remains historical gallery evidence, not a first-party adapter contract.

## Why

The crate was a 71-line theme/control wrapper. The migration plan's cards, legends,
menus and toolbars were never built. A dedicated package implied a fourth library crate
and leaked Kit into workspace membership, checker allow-lists and authority tables
without adding semantics.

ADR-001's isolation still holds: one `gpui-pre` identity, Kit opt-in only through
`chart-gallery`, and no Kit in `chart-core`, `chart-export` or `gpui-charts`.

## Alternatives

Keeping the crate preserves a named helper at the cost of a workspace member for a
gallery recipe. Merging it into `gpui-charts` would place Kit types next to the
standalone host and violate the repository checker.

## Consequences

Update ARC-01, the authoring register, gallery `kit` feature and
`scripts/check_repository.py`. Historical evidence snapshots may still name the
removed paths.
