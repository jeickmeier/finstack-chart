# ADR-025: Row text and portable raster paint

Status: accepted for GG-08 implementation, 15 September 2026.

GG-08 needs source/statistical labels and custom/raster annotations. Fixed figure
annotations are intentionally bounded and cannot represent a data-label population.

A layer retains ordinary prepared point anchors and their source/aggregate/derived
targets. Its optional `text` consumer uses the existing typed value aesthetics,
explicit font resources and shared shaping service. Rotated labels carry one polygon
interaction with their logical label, even when the paint comprises a box and several
runs. The common position stage owns nudging. Text uses pinned ggplot unit conversion
(including 72.27 TeX points per inch), while vector/raster local coordinates use the
existing physical aesthetic units. Definition version 73 identifies these consumers.

One `RasterImage` scene primitive retains positive bounds, bounded row-major RGBA
pixels, and an interpolation flag. Scene version 20 is required only when this
primitive occurs. Core performs no image decoding or I/O; the raster's samples count
against the common path/sample budget. This owner can also serve GG-07 raster geoms.

SVG embeds a PNG produced by the existing encoder, with an explicit image-rendering
policy. The existing resvg 0.48.1 raster-images feature paints PNG output. PDF uses
krilla 0.8.2 `CustomImage` and its interpolation flag; the existing raster-images
feature exposes that API. Native uses the existing GPUI image adapter, including a
one-pixel replicated border to prevent image-atlas bleeding. Nearest native output
uses exact pixel-cell quads. Existing locked versions are unchanged; only optional
feature dependency edges are enabled. No new dependencies enter core.

The interpolation contract is destination image filtering, as for reference raster
annotations. It does not promise identical pixel kernels between a PDF viewer, GPUI,
and resvg. Dimensions, orientation, sRGB bytes, alpha, bounds and the interpolation
selection remain identical across hosts. Mathematical labels remain GG-14 work.
