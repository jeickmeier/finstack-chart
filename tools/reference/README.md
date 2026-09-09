# Phase 2 development references

These tools generate offline fixtures. They are not production dependencies and are
never invoked by ordinary Rust tests. ADR-014 owns shared layout/metadata conventions.

`node/package-lock.json` pins all eight requested D3 modules and their transitive
sources. `node/manifest.json` records the executed Node version, complete export
lists, source-file SHA-256 values and the lock digest. From the repository root:

```sh
mise exec -- npm --prefix tools/reference/node ci --ignore-scripts --no-audit --no-fund
TZ=UTC mise exec -- node tools/reference/node/provenance.mjs
TZ=UTC mise exec -- node tools/reference/node/path.mjs
```

R 4.6.1 is supplied by the local machine. `r/run.py` resolves its R home, isolates
packages/caches/font configuration in `target/reference-r`, and fixes locale/timezone.
The accepted ggplot2 4.0.3 environment is `r/renv.lock`; the initial `bootstrap.R`
provisioned it, while subsequent restoration uses that lock. Source archives and
licenses for all 57 locked packages are identified by `fixtures/parity/ggplot2/sources.json`.
The runtime manifest records the font and external-library identities. This macOS
execution extracted XQuartz 2.8.5 libraries into
`target/reference-downloads/xquartz-expanded/XQuartzComponent.pkg/Payload/opt/X11/lib`
for Cairo; it installed no system services. Use the exact corresponding libraries
when reproducing this device profile. The owner-installed R is used instead of the
earlier temporary R extraction.

```sh
mise exec -- python3 tools/reference/r/run.py tools/reference/r/restore.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/inventory.R
mise exec -- python3 tools/reference/r/run.py tools/reference/r/generate.R
mise exec -- python3 tools/reference/r/download_sources.py
```

Normalize PDF creation timestamps with `r/normalize_pdf.py` using Python with the
pypdf version in the runtime manifest. The script verifies unchanged decoded page
content streams. No geometric values, fonts, text or tolerance are normalized.
Generate into a temporary directory, normalize that directory, and compare all files
against the retained set before accepting an update. SVG/PNG and numeric records
already regenerate exactly; PDF bytes become deterministic after this sole metadata
normalization. The font is the repository's Noto Sans; `pdffonts` confirms its actual
embedding in Cairo PDF output.

The ggplot2 inventory covers 643 exports, signatures/defaults, inherited methods,
documentation links and documented generated-field expressions. Its 32 executed
seed cases span grouping/stages/bins/aesthetics/guides/facets/distributions/models/2D
statistics/coordinates/math/sf and include built-layer, scale, guide, panel and grob
records plus 96 SVG/PDF/PNG artifacts. Required capabilities remain OPEN; the entry
oracle is not an implementation verdict. Each delivering package adds its complete
argument/boundary cases and compares the shared Rust engine with these references.

## Scale reference entry (SP-01)

Run `mise exec -- node tools/reference/node/scale.mjs [output-directory]` to regenerate
all factory/method/default observations and numeric formatting cases. The generator
launches one fresh Node process for each explicit timezone; `local-time.json` retains
UTC, New York, Berlin and Lord Howe rules and calendar observations. Node and tzdata
identity, source digests, transitive npm integrity and licenses are in the manifest.
No machine-default timezone is adopted.

`mise exec -- cargo run -p chart-core --example scale_gap_report --locked -- target/scale-gap-report.json`
reads the committed corpus offline and measures the current two-endpoint linear API.
Every other method/case is explicitly unqualified or missing. Runner success reports
measurement completion; only its individual Pass observations match, and it does not
close G-SCALE. SP-02–07 extend the executable comparator as capabilities arrive.
