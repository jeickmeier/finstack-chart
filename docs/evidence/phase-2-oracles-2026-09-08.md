# Phase 2 reference-oracle entry evidence

GG-00 is complete for its entry scope on `fab2505951061eaafe9adb52c86b248ee0dfa6bf`
plus the uncommitted reference workspace. This does not certify implemented ggplot2
capabilities. All 643 public exports remain explicitly OPEN until their owning
packages supply their argument/boundary comparisons and destination evidence.

The [reference workspace](../../tools/reference/README.md) pins eight D3 packages
and a separate R/ggplot2 environment. [renv.lock](../../tools/reference/r/renv.lock)
pins 57 packages; [sources](../../fixtures/parity/ggplot2/sources.json) retain their
archive URLs, SHA-256 hashes and license notices. R 4.6.1 runs from the owner's local
installation. Package libraries and caches are workspace-local. Cairo uses extracted
XQuartz 2.8.5 libraries; no system service was installed. Locale C, UTC, RNG kinds,
seed 1729, Noto Sans identity, external spatial-library versions and execution session
are retained. The runtime environment is Darwin arm64; no Linux oracle run is claimed.

The inventory records all 643 exports, formals/defaults, inherited ggproto members,
226 Rd documentation records and documented computed-field expressions. Every export
has an owning GG package and an explicit capability status. The 32 executed seed
cases retain built layers, trained scales, guide keys, panel parameters, grob layout,
warnings and SVG/PDF/PNG output (96 artifacts). The seeds cover grouping, stages,
weighted bin boundaries, limits versus coordinate zoom, color/size/guide/facet behavior,
distribution summaries, fitted models, 2D statistics, polar/math labels and sf projection.
These are discriminating entry cases, not an exhaustive argument matrix for all exports.

Two generations produced identical structured records and all 96 artifact byte
streams after normalizing only PDF CreationDate to `D:20000101000000Z`.
The normalizer asserts that decoded page streams are unchanged. Expected numeric
values, geometry, fonts and tolerances are never normalized. `pdffonts` verifies
that Cairo's output embeds the supplied Noto Sans face.

Executed commands:

```sh
mise exec -- python3 tools/reference/r/run.py tools/reference/r/inventory.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/generate.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/generate.R target/reference-r/regenerated
mise exec -- python3 tools/reference/r/download_sources.py
mise exec -- cargo test -p chart-core --test reference_inventory --locked
```

The bundled artifact Python ran `tools/reference/r/normalize_pdf.py` on both output
directories. A byte comparison checked `cases.json` and every corresponding artifact.
The offline Rust test independently checks weighted/unweighted bin closure, inferred
versus explicit grouping, two-group boxplot quartiles and OLS values. It requires no
installed R or Node and does not claim Rust implementation parity for those seeds.

The [manifest](../../fixtures/parity/ggplot2/manifest.json) identifies the concrete
generator/lock inputs and retained files. Subsequent reference restoration uses the
lock rather than a fresh latest-version bootstrap. The next GG work package is GG-02;
the concurrently ready path foundation owns WP-P01–04 and its separate FIX-P01–06 proof.
G-GGPLOT, G-PARITY and all D3 capability gates remain open at this entry checkpoint.
