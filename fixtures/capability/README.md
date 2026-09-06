# WP-03 capability fixture

[scene.svg](scene.svg) is a newly authored, fixed publication scene in a 180 × 120 mm
viewport (72 points/inch). It is an experiment shared by the native and headless examples,
not an alternate chart compiler or a public SVG import API. It exercises the WP-03 subset
of ARC-04, LAY-02/04, SCN-03, GPU-01/03, EXP-01/02 and QLT-03/04. Dimensions match the
FIX-13 physical-size example; this does **not** pass all of FIX-13 or FIX-18.

The shared [resource loader](support.rs) consumes WP-02's resource identity/revision,
byte-budget and provider contracts. Extended primitive/style experiments use `usvg`'s
temporary tree; integration with core `Scene`, `TextMeasurer`, chart layout and diagnostics
is WP-07/08 work. No permanent core API was expanded to accommodate the experiment.

## Exact assets and source

Fonts are copied without modification from the published `cosmic-text` 0.19.0 package,
`fonts/`, repository revision `c24886c2471e5606587c46090cd25dbbf209186b` from its
`.cargo_vcs_info.json`. Source: [cosmic-text fonts at that revision](https://github.com/pop-os/cosmic-text/tree/c24886c2471e5606587c46090cd25dbbf209186b/fonts).
They are fixture resources, not a new Cargo dependency. No system-font directory is loaded.

| Resource ID / revision | Asset / family | SHA-256 | License |
| --- | --- | --- | --- |
| 0 / 0 | `NotoSans-Regular.ttf` / Noto Sans | `2ec33f84606cbaa0a1a944488e14f97faf2f6a25ecdd8354f5358f06da13c7d9` | [SIL OFL 1.1; Google notice](fonts/NotoSans-LICENSE) |
| 1 / 0 | `FiraMono-Medium.ttf` / Fira Mono | `5f9173ce3d05fadef74c7eed06570d54e4f75bd0cd9860726fb2987a7f848292` | [SIL OFL 1.1; Mozilla/Telefonica notice](fonts/FiraMono-LICENSE) |

The loader checks parsed OS/2 permissions before subsetting/outline export and refuses
restricted/unknown permissions, prohibited subsetting or bitmap-only embedding. The
fixture fonts permit this use. No synthesized bold/italic faces are used. Native overlays
load these same bytes into GPUI; publication uses an explicit two-face font database.
The fixture preflight checks representative non-ASCII glyphs and the numeric run; it is
not a general XML/span/fallback validator. PDF conversion also enables the dependency's
missing-glyph check. A U+10FFFF negative probe produces a resource-specific error record.

## Run and inspect

From the repository root:

```sh
mise exec -- cargo run -p chart-export --example capability_export --release --locked -- artifacts/wp-03
mise exec -- python scripts/check_capability_artifacts.py artifacts/wp-03
mise exec -- cargo run -p chart-gallery --example capability_spike --features kit --release --locked
mise exec -- cargo run -p chart-gallery --example capability_spike --features kit --release --locked -- --profile
```

The last command requests 40 additional render notifications, then leaves an interactive
window open; OS/input events may cause extra paints. The timing boundary excludes GPU
presentation. The checker requires Poppler (`pdfinfo`, `pdffonts`, `pdfimages`, `pdftotext`)
and makes independent physical-size, font-policy and vector-structure assertions. It does
not replace visual inspection. Retained outputs and the manual native procedure are in
the [completion report](../../docs/evidence/wp-03-completion-2026-09-06.md).

For desktop automation this run wrapped the executable in a temporary `.app` with bundle
ID `local.finstack.chart-capability`, executable `capability_spike`, name `ChartCapability`
and package type `APPL`. This was only an inspection aid, not product packaging.
