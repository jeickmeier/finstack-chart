# Canonical fixtures and resources

FIX-01–FIX-18 are defined in the [specification](../docs/spec/gpui-charts-specification.md).
No fixture execution or chart output exists at bootstrap. Add small cases beside their
independent expected semantic results as the owning work package is implemented.

For each imported or generated reference asset, record fixture ID, origin URL and exact
revision, license/notices, generation command/tool version, relevant semantic settings
and any transformations. Font assets also require a hash and embedding permissions.
Routine Rust tests consume stored cases without requiring R, JavaScript or downloads.

Keep semantic expectations distinct from renderer snapshots. A visual baseline update
needs an explained change and actual inspection; never replace it solely to pass CI.
Use operation-specific tolerances. Do not copy implementations into expected-value
calculations. Generated local outputs belong under ignored `artifacts/`; retained release
evidence must have a durable linked location in the status ledger.
