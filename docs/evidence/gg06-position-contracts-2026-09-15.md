# GG-06 position contracts

GG2-02/05 and FIX-GG06, implementation on the current working tree. The new
`GgplotStack`, `GgplotDodge`, `GgplotDodge2`, `JitterDodge`, and `Nudge` variants
leave legacy `Stack`, categorical band-relative `Dodge`, and stable-key `Jitter`
unchanged. Layer orientation reuses the existing canonical-axis normalization.

`ggplot_stack()` / `ggplot_fill()` use reference descending group order by default,
separate positive/negative accumulation, `reverse`, and point/text `vjust`.
Interval geometry retains both stack bounds. Fill follows the reference
`sqrt(.Machine$double.eps)` total threshold, including zero/tiny populations.
`ggplot_dodge()` supports independent-axis width and total/single preservation;
`dodge2()` packs variable-width overlapping intervals and supports padding.
Single preservation takes the maximum collision population across existing facet
panels, through preparation-local panel definitions. Padding applies only where
the panel contains overlapping intervals. Missing groups consume no invented row.

`nudge(x, y)` moves coordinate endpoints in calculation units; category units are
steps, projected through the existing axis catalog without inventing identities.
`jitter_dodge(seed)` performs reference dodge then the established keyed jitter:
FNV-1a target/group hash followed by SplitMix64, independently salted x/y draws.
Default horizontal half-width is 0.4 times independent-axis resolution. The selected half-width divides by the maximum collision group count plus two, shared across existing panels. This policy
preserves retained/fresh and reordered-row identity; it is **not R RNG parity**.

Source: `tools/reference/r/position-controls.R` and
`fixtures/parity/ggplot2/position-controls.json`, pinned R 4.6.1 / ggplot2 4.0.3.
Thirty-two position cases and six independent jitter setup cases cover variable/fixed interval widths, total/single, reversal,
signed/zero stack and fill, all three anchors, and unequal facet populations.
The ordinary dodge source warns on overlapping nonaligned interval widths;
recorded numeric results remain the oracle, without suppressing those warnings.

Focused tests: `grammar::ggplot_position::tests` plus
`tests/ggplot_position_controls.rs`. Actual host proof scripts:
`scripts/bindings/ggplot_position_controls.py` and `.cjs`, eight independent
authors per host, immutable roundtrips, and 24 SVG/PDF/PNG outputs each.
Execution results and aggregate acceptance are recorded by the integrating task
in the status ledger; this contract note alone does not close any gate.

Final focused review also checks the reference singleton-stack fast path: default
nonfill/upper-anchor positioning preserves a population with distinct x values,
including negative point coordinates. Source jitter setup fixtures distinguish
maximum collision population from total distinct groups and validate default
resolution scaling as well as explicit displacement. The 32-case position matrix
and six-case jitter setup matrix pass in three unit tests; the integration target
covers four authoring/facet/identity contracts. Runtime/native qualification remains
owned by the integrating task.

Native qualification on 15 September 2026: the position and bin galleries built
with `cargo build -p chart-gallery --example ggplot_position_controls_native
--example ggplot_bin_stat_controls_native --locked`. Build log:
`/private/tmp/gg06-native-build.log`. Six position cases and six bin cases painted
successfully; all panels were visually inspected. Position screenshot:
`/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_19-37-15.png`.
Bin screenshot:
`/var/folders/l1/s1m3_kfn43d77mc45c_3rv_h0000gn/T/codex-shot-2026-09-15_19-38-03.png`.
Paint traces: `/private/tmp/gg06-position-native.log` and
`/private/tmp/gg06-bin-native.log`. Both owned processes were closed with exit 130
following successful inspection. The upstream `block v0.1.6` future-compatibility
notice is unchanged; no project build failure occurred.
