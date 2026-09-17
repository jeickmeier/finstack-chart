# ADR 033: Additional bounded raster publication codecs

Status: adopted for the GG17 raster implementation slice; package acceptance pending.

JPEG uses jpeg-encoder 0.7.1 ((MIT OR Apache-2.0) AND IJG), with std and no SIMD, after native encoding and wasm32 compilation in an isolated project. BMP is an original bounded RGB24 BITMAPINFOHEADER encoder.
TIFF uses tiff 0.11.3 (MIT), the same codec underneath image's TIFF feature, directly
so checked compression, predictor, straight-alpha and physical-resolution tags are
available without inventing metadata. The initial image 0.25.10 candidate was rejected: workspace feature unification creates optional graph paths to browser host objects, violating the mandatory repository dependency boundary. It is absent from both chart-export production and dev dependencies. The original codec portability spike compiled native and wasm32; the replacement JPEG was independently re-spiked. This does not certify actual
chart-export host execution.

All three devices reuse existing bounded scene rasterization. JPEG/BMP flatten
straight alpha over an explicit opaque matte; TIFF retains unassociated RGBA.
A bounded seekable writer rejects output growth before allocation. Pixel and output
budgets remain inherited from PublicationProfile. Original SVG/PDF/PNG encoders and
default serialized options retain their contracts; absent raster options are omitted.

The direct TIFF API is preferable to patching TIFF offsets after encoding. BMP's fixed RGB24 BITMAPINFOHEADER density fields, bottom-up BGR rows and four-byte padding are emitted directly with checked file lengths. No native GUI, filesystem,
interpreter or browser dependency is introduced by encoding.

Focused codec tests qualify decoding, DPI, alpha, row padding, compression/predictor
and budget errors. Actual host artifacts, independent platform decode, saving rules,
remaining device rows and cumulative acceptance are separate GG17 work.

JPEG attribution: this software is based in part on the work of the Independent JPEG Group. The exact IJG notice is retained in docs/licenses/jpeg-encoder-0.7.1-IJG.txt.

## Extended TIFF strip codecs (2026-09-16)

The same TIFF directory encoder writes the additional standard compression tags;
there is no second IFD serializer. XZ/LZMA2 uses `lzma-rust2 =0.16.2` (Apache-2.0,
std/encoder/xz only), Zstandard uses `ruzstd =0.8.2` (MIT, std only, real Fastest
compression level), and lossless WebP uses `image-webp =0.2.4` (MIT OR Apache-2.0).
JPEG strips reuse the existing JPEG owner and flatten alpha against the explicit
matte. This preserves the existing four TIFF codecs and their bytes.

Private native encode/roundtrip and wasm32 compile experiments passed in
`/private/tmp/gg17-codecs-{native,wasm}.log`. Runtime integration and independent
libtiff decoding are separate gates. Ruzstd's infallible compression API unwraps
writer errors; its output sink therefore bounds retention, consumes excess output,
and returns a ResourceLimit diagnostic afterward rather than panicking. Codec
workspace buffers remain bounded by the existing raster pixel limit. No threading,
filesystem, browser or interpreter object enters the publication encoder.
