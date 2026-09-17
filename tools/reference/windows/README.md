# Windows enhanced-metafile qualification

This verifier is development evidence only and is never linked to chart-core/export.
Run the actual device authors first, then on Windows PowerShell5.1:

```powershell
.\verify-emf.ps1 -InputFile device-144.emf -OutputPng device-144-windows.png -ExpectedPng device-144.png
```

The script checks framing and signature, uses real Windows GDI playback and records
a JSON result. The optional RGB mean absolute error threshold defaults to2/255;
it is an explicit raster-comparison policy, not exact pixel equivalence between
independent antialiasers. Inspect the resulting image for clipping, dashes, curves,
glyph outlines and colors. Repeat at72DPI and with opaque rasters, multipart holes
and source-alpha omission examples. Archive OS/build, input hashes, JSON and images.
The script never modifies expected images. A successful binary parser on another
platform cannot substitute for Windows playback. The.wmf alias is enhanced EMF,
consistent with R win.metafile, and does not claim classic WMF support.
