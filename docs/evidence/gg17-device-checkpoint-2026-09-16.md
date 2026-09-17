# GG17 device implementation checkpoint

This is partial FIX-GG17 evidence, not package acceptance. Parent owns the ledger,
saving/custom-device work and cumulative host builds.

JPEG/TIFF/BMP share existing bounded rasterization. Four focused tests passed in
`/private/tmp/gg17-raster-tests.log`: exact decoded BMP/TIFF pixels and alpha,
row padding, all implemented lossless compression/predictor combinations,
JPEG independent zune decoding, physical metadata and rejected output growth.
The Rust author emitted eight initial raster artifacts at 72/144 DPI under
`/private/tmp/gg17-raster-artifacts`; ImageIO independently decoded all three new
formats. Metadata `/private/tmp/gg17-imageio-metadata.log` reports 360x240 and
720x480; BMP 144 DPI reports 143.993 from integer pixels/metre, as expected.
Decoded TIFF/BMP/JPEG PNGs were visually inspected. Source signatures and actual
22 TIFF calls are committed in device-controls.json. Cairo RLE/LERC produced no
file on this reference host; Quartz warns and ignores non-default compression.
Additional Cairo JPEG/LZMA/Zstd/WebP TIFF compression is implemented and qualified below.

ADR033 records jpeg-encoder0.7.1 and direct tiff0.11.3 adoption. The image0.25.10 candidate was removed from both production and development edges because its workspace-unified optional graph reached wasm-bindgen. JPEG decoding uses independent zune-jpeg0.5.15. Existing PNG rendering
was extracted without changing its rendering/encoding sequence. Actual legacy-byte
and fresh Python/WASM qualification remains the parent integration gate.

PS/EPS now emit real paths, curves, stroke state and supplied-font outlines.
The first ordinary author was decoded by private Ghostscript 10.08.0 and visually
inspected (`/private/tmp/gg17-ps-page.png`). This only qualifies that author, not
all vector features. Source six-color-model paint commands are committed as
postscript-controls.json. Calibrated sRGB, neutral-gray variants, Gray and CMYK
policies are explicit. Default partial-alpha handling rejects unsupported paint;
source-like omission is opt-in. Geometry, gradient, multipage, raster interpolation,
source omission diagnostics and complete device fixtures are still being qualified.

Private verifier source URL:
https://github.com/ArtifexSoftware/ghostpdl-downloads/releases/download/gs10080/ghostscript-10.08.0.tar.xz
SHA256 c20492bc8ebb96c87fa2e52a0926e1cda8cde95d66145e018ac713fed5da38cf.
It is built only under /private/tmp, not linked or adopted by the workspace.
The verifier has its own AGPL license; emitted document code is original.

PicTeX now emits original bounded monochrome paths and safe literal ASCII Computer
Modern device text. Its explicit capability warning retains historical no-fill,
ordinary-linewidth and limited rotation semantics; Unicode/math and rasters do not
silently become literal TeX. Private Tectonic 0.15.0 compiled an actual generated
chart (`/private/tmp/gg17-tex-proof/device.pdf`); the compiled chart was rendered by Poppler and visually inspected in `/private/tmp/gg17-tex-device.png`.
Windows EMF is assigned independently; Windows playback evidence remains required.

Eleven focused device tests passed (`/private/tmp/gg17-devices-multipage-tests.log`),
including pinned JPEG sampling at qualities 0/75/95/100, all lossless TIFF options,
PS gradients/color policies, safe PicTeX text, mixed-dimension/mixed-compression TIFF
pages and cumulative output-budget rejection. TIFF pages render sequentially with
one raster allocation and a 1024-page ceiling. The authors now request 20 files:
14 single-page devices plus two-page PS/PDF/TIFF at both DPIs, retaining page handles
after disposing source figures. Fresh all-host qualification is pending.

At that intermediate checkpoint, advanced TIFF, final PicTeX rendering and
multipage inspection remained open; the final codec checkpoint below supersedes
those rows. Windows EMF playback and complete vector mask capability remain
explicit qualification boundaries. Cairo RLE/LERC source unavailability and
Quartz ignored compression remain recorded platform outcomes, not implementations.

## Final codec checkpoint

All 28 Rust artifacts were produced (`/private/tmp/gg17-advanced-author.log`).
The same TIFF directory owner now accepts JPEG, XZ/LZMA2, Zstandard and lossless
WebP strips; no second IFD container was introduced. Fifteen focused device tests
pass (`/private/tmp/gg17-advanced-tests.log`, including three independently owned
EMF tests). Alpha, exact decoded lossless pixels, quality sampling, mixed page
sizes/compressions and bounded output rejection are exercised.

Independent libtiff 4.7.1 decoded JPEG/XZ/Zstd and both pages of the LZW multipage
file. `tiffcmp -t` compares every XZ/Zstd pixel to the original LZW raster; reported
differences are only codec/predictor/strip tags. Installed libtiff lacks WebP, so
private official libwebp 1.6.0 was built and its WebPDecodeRGBA used against the
extracted standard TIFF strip. Both 360x240 and 720x480 pixel arrays match libtiff's
independently decoded LZW arrays exactly (`/private/tmp/gg17-webp-independent.log`).
The verifier source archive is the official webmproject libwebp-1.6.0.tar.gz;
SHA256 e4ab7009bf0629fd11982d4c2aa83964cf244cffba7347ecd39019a9e38c4564.
It is not a workspace runtime dependency. Multipage PS decoded through Ghostscript;
Poppler confirms PDF has two360x240pt pages. Final shared host proof remains parent
owned. Agent A adds EMF to each DPI, bringing the author set to30 artifacts.

Remaining platform qualification: Windows GDI EMF playback. Source-unavailable
Cairo RLE/LERC and Quartz ignored compression remain explicit platform outcomes.
Existing SVG/PDF/PNG legacy-byte and all-host gates are tracked by the parent.

## EMF bounded device and external Windows gate

`devices_metafile.rs` emits enhanced metafile records directly from the shared immutable publication tree: supplied-font outlines, retained filled paths, shared stroke expansion with dashes/caps/joins, per-item clipping and opaque top-down BGRA raster records. Both `emf` and the source-compatible `wmf` alias select enhanced metafile, not classic WMF. Partial alpha rejects by default; explicit `OmitTranslucent` produces a diagnostic count. Gradient/pattern paint, masks/filters, non-RGB color conversion and interpolated alpha rasters produce explicit ExportFidelity errors. No implicit page rasterization or native-font substitution occurs. One page is emitted per file; save's numbered page policy owns batches.

Three independent focused tests pass (`/private/tmp/gg17-emf-tests1.log`; final narrow rerun `/private/tmp/gg17-emf-tests2.log`): header/signature/record lengths/count/EOF and resource budgets, explicit alpha policy, and independent raster offsets/top-down BGRA decoding. The three actual-host device authors include72/144DPI EMF (30files total per host); fresh shared host comparison is owned by integration. This is binary/contract evidence, not Windows visual certification.

`tools/reference/windows/verify-emf.ps1` validates record framing independently and invokes actual Windows `GetEnhMetaFile`/`PlayEnhMetaFile`, saves a PNG, and optionally compares it against an independently exported expected PNG without modifying that expectation. It logs OS, dimensions, byte/record counts and RGB mean absolute error. **The Windows harness has not been executed in this macOS environment.** No LibreOffice, ImageMagick EMF delegate or PowerShell executable was found locally. Actual Windows GDI playback and visual inspection remain an explicit external gate.

Source contracts: [R Windows devices](https://stat.ethz.ch/R-manual/R-devel/library/grDevices/html/windows.html) specifies enhanced metafile even for.wmf and one plot/file; [Microsoft MS-EMF header](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-emf/de081cd7-351f-4cc2-830b-d03fb55e89ab) defines the binary header. Device limitations are not renamed as complete Windows certification.
