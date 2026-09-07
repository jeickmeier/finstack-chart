# FIX-12/13 composition fixtures

The three portable cases share exact populations, layers and figure furniture while
using editorial, terminal or grayscale themes. Each has B/A panels, shared axes,
explicit mapped colors, diamonds, dashed reference rules, panel gradients, rotated
number labels/axis titles, a bold title, an explicitly fallback-shaped Arabic run,
source/caption/notes, coordinate-space annotations, panel letters and a source-sharing
inset in A. `profile.json` is shared by all cases and actual Rust/Python/WASM runners.

The figure is 180 × 120 mm. Expected PNG dimensions are 2126 × 1417 at 300 DPI and
4252 × 2835 at 600 DPI. SVG/PDF remain vector; PDF has three embedded Unicode faces.
`cargo run -p chart-export --example composition_proof --locked` generates all formats
and the exact-outline native preview under `artifacts/wp-13` by default.

## Font provenance

Regular Noto Sans comes from the existing capability fixture and manifest. Additional
faces are OFL-1.1, downloaded 7 September 2026 from the Noto project's official builds:

- [Noto Sans Bold](https://notofonts.github.io/latin-greek-cyrillic/fonts/NotoSans/hinted/ttf/NotoSans-Bold.ttf), 627176 bytes.
- [Noto Sans Arabic Regular](https://notofonts.github.io/arabic/fonts/NotoSansArabic/hinted/ttf/NotoSansArabic-Regular.ttf), 237400 bytes.

`fonts/manifest.json` records SHA-256 and exact descriptor identity/revision. The adjacent
OFL files come from the corresponding official notofonts GitHub repositories. Explicit
font bytes are embedded once in the shared profile for portable host ingestion; Rust
native examples read the same committed font fixture. The Arabic run intentionally
requests the regular primary with an explicit Arabic fallback, so one fallback warning
is expected. It must never be reported as missing glyphs or replaced with an arbitrary
system font. The declared bold face is weight 700; synthetic weight is not permitted.
