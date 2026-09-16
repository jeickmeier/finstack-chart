# GG-10, GG-15 and GG-17 dependency decisions — 15 September 2026

Read-only adoption spike for the authorized GG-06–18 program. No dependency, implementation, wire, or acceptance status changes. The implementation plan's GG-10/15/17 requirements remain authoritative; this report identifies viable building blocks and unresolved capabilities, not completed packages.

## Decisions that permit work to start

- **GG-10:** use one shared chart-core model matrix/solver owner; `nalgebra = 0.34.2`, default features disabled and `std` enabled, is a viable numerical dependency candidate. Implement model semantics above it. Do not substitute ordinary least squares for LOESS, GAM, or quantile regression.
- **GG-15:** retain projection, geometry subdivision and clipping as separate responsibilities. `proj4rs = 0.1.10` is a partial portable CRS candidate, not a complete mapproj/sf replacement. Its adoption remains unqualified until the native/WASM experiment and required projection inventory pass. Full PROJ may be an explicit native service, but cannot silently become the sole shared engine.
- **GG-17:** reuse the existing bounded raster rendering in `crates/chart-export/src/encode.rs`; `image = 0.25.10`, with only JPEG/TIFF/BMP features, is a viable encoder candidate. Keep PNG's existing path. Implement PS/EPS and PicTeX as distinct vector device owners; Windows metafiles require a Windows-specific adapter and evidence.

## Verified local spike

Private project `/private/tmp/gg-dependency-spike` pins nalgebra 0.34.2 and image 0.25.10 in its manifest and generated lockfile. Both are cached locally. Licenses from the downloaded manifests are Apache-2.0 and MIT OR Apache-2.0 respectively; minimum Rust versions 1.87 and 1.88 are below workspace 1.97.1. This is direct dependency screening, not a completed transitive license audit.

Commands, run through the workspace's `mise exec`, were:

```
cargo run --offline --manifest-path /private/tmp/gg-dependency-spike/Cargo.toml
cargo check --offline --target wasm32-unknown-unknown --manifest-path /private/tmp/gg-dependency-spike/Cargo.toml
```

**PASS native execution:** SVD solves the independent line `y=1+2x` within 1e-12; JPEG/TIFF/BMP encode 2×2 RGB images and decode dimensions successfully (631/222/70 bytes). Log: `/private/tmp/gg-dependency-native.log`. **PASS WASM compilation**, log `/private/tmp/gg-dependency-wasm.log`. This is not a WASM runtime, independent decoder, color fidelity, uncertainty, performance, or chart export proof. The initial probe had an ambiguous Rust float type; explicit f64 fixed the probe before both passing commands.

The separate `/private/tmp/gg-proj-spike` pins proj4rs 0.1.10 with defaults off and `wasm-strict`. Its offline check **could not resolve the uncached package**; `/private/tmp/gg-proj-spike.log`. Consequently no local native/WASM projection success is claimed. Obtain the pinned dependency and complete this small experiment before an ADR adopts it. No workspace manifests or dependencies were changed.

## GG-10 capability and algorithm matrix

Existing reuse: `grammar/statistics.rs` has stable shifted/scaled simple OLS and grouped preparation/provenance, and `grammar/statistical_types.rs` exposes the fit recipe. This is not a general weighted design-matrix solver or confidence-band engine. Reuse preparation, generated fields, diagnostics, and GG-07 ribbon/path geometry; keep model kernels in chart-core and host adapters thin.

The committed `tools/reference/r/renv.lock` pins **mgcv 1.9-4 and quantreg 6.1**, with R 4.6.1 as the reference runner. Generate offline oracle fixtures once per model matrix. Installation of a lock entry does not establish local availability.

| Required capability | Implementation decision / remaining algorithm | Independent acceptance anchor |
|---|---|---|
| Weighted LM, intercept/no intercept and typed terms | Weighted design matrix, rank-revealing QR; SVD for explicit rank diagnostics. Center/scale predictors; avoid normal equations as primary solve. Typed intercept, powers, interactions and registered transforms must have explicit supported rows. | Exact polynomial/linear coefficients; zero weights; rank-deficient and high-offset x; pinned `lm` fitted values, residual df and covariance. |
| GLM and link-scale intervals | IRLS using shared weighted solver, step halving, deviance/convergence diagnostics; explicitly enumerate family/link pairs and dispersion treatment. Gaussian, binomial and Poisson do not constitute all R families. | Analytic intercept-only Gaussian/binomial/Poisson; pinned `glm` predictions and normal intervals transformed from link scale; separation and nonconvergence. |
| LOESS | Local polynomial fits with tricube weights, span/degree and robust iteration policies; interpolation surface and influence/df calculations are separate required work. A direct pointwise local fit is not the default interpolated reference algorithm. | Pinned `loess` prediction, SE and df over interior/endpoints/ties, spans, weights, robust family and both surface policies. |
| Automatic GAM | Largest group across panels selects method, below 1,000 LOESS and otherwise GAM. Default GAM needs shrinkage cubic regression splines, basis constraints, REML selection and covariance. Generic cubic interpolation is insufficient. | 999/1,000 observations, unbalanced facets, explicit override; pinned mgcv basis/penalty and REML prediction/SE on nonlinear examples. |
| Quantile regression | Convex weighted check-loss optimization; use a bounded primal-dual/LP solver with objective/KKT diagnostics. Solver dependency still unselected; do not use squared-error fitting or smoothed loss without an explicit different capability. | Intercept weighted quantiles, small hand-enumerated LP optimum; pinned quantreg objective and predictions for several tau values, nonunique/rank-deficient problems. |
| Uncertainty and grids | Shared prediction-grid/orientation/fullrange semantics; t quantiles for LM/LOESS versus link-scale normal intervals for GLM; GAM covariance per reference. | Confidence level monotonicity, zero residual variance, independent distributions and R outputs; filtered versus zoomed data and update-versus-batch. |
| Model options/registration | Enumerate built-in accepted terms/options; registered models validate output schema, resource limits and diagnostics. A hook never substitutes for any built-in row above. | Malformed, nonfinite and wrong-length callback outputs; actual Python/WASM replay and invalidation. |

The automatic threshold and GLM/LOESS interval distinctions follow the [ggplot2 smoothing contract](https://ggplot2.tidyverse.org/reference/geom_smooth.html). LOESS uses local polynomial fitting, default span 0.75/degree 2, and distinguishes Gaussian and robust symmetric families; implementation must cover the applicable controls, not borrow LOWESS defaults. [R LOESS documentation](https://stat.ethz.ch/R-manual/R-devel/library/stats/html/loess.html).

The default GAM basis dimension is 10, with knots distributed over ordered covariates and a natural cubic basis; `cs` modifies the penalty to shrink the smooth toward zero. These facts do not supply a complete REML implementation. [mgcv cubic spline documentation](https://stat.ethz.ch/R-manual/R-devel/library/mgcv/html/smooth.construct.cr.smooth.spec.html). Quantile regression solver/method distinctions remain part of the numerical contract. [quantreg documentation](https://search.r-project.org/CRAN/refmans/quantreg/html/rq.html).

## GG-15 projection and geometry matrix

Pinned references: **sf 1.1-2, mapproj 1.2.12, maps 3.4.3**. Record sf's actual GDAL/GEOS/PROJ versions and grids alongside newly generated fixtures, not just the R package versions. Shared geometry work depends on GG-13; there is no complete GIS/projection owner in the present workspace.

| Required capability | Candidate / explicit gap | Acceptance anchor |
|---|---|---|
| Geographic point/line/polygon, multi-geometries, holes, collections | Typed owned geometry and validity checks; reuse scene paths only after topology is established. A CRS library does not implement this layer. | Empty/invalid rings, mixed types, nested holes, map joins and provenance. |
| Common projection kernels | proj4rs advertises Mercator, transverse Mercator, LCC, Albers, LAEA, stereographic, Mollweide and related kernels. Parameter/ellipsoid/orientation equivalence is untested. | Independent origin/axis anchors and pinned sf/mapproj numerical fixtures; forward/inverse residuals. |
| Full mapproj methods | See exhaustive method names below. Absence from proj4rs's documented module list means **unverified**, not silently supported by similar names. Missing kernels need a separate implementation or qualified service. | Every name, defaults, parameter boundaries, orientation and unprojectable points against mapproj. |
| CRS parsing, axis order, units, default CRS and mixed CRS | proj4rs uses radians and has no default WKT support. EPSG definitions are optional. Adapter must explicitly distinguish authority axis order from plotting longitude/latitude order. | EPSG/CRS84 order examples; overlays in default CRS; transformed limits and mixed layers. |
| Datum/grid resources | proj4rs documents experimental NTv2 grids only; broader grid formats, operation pipelines and resource selection remain gaps. Disable default multithreading; supply immutable resource bytes explicitly. | Known grid-shift anchors; missing/wrong grids; deterministic resource identity; no implicit I/O. |
| Dateline, subdivision, clipping and graticules | GG-13 shared protocol plus geographic seam splitting; projection library alone cannot preserve holes or choose labels. | Crossing ±180°, polar horizon, clipped holes, high-resolution exports, selection and bounded subdivision. |
| sf label/stat extraction and map/border joins | Shared geometry statistics and typed map records; not an R object adapter. | Known centroids/interior points, disconnected geometries and join-key misses. |

Complete mapproj 1.2.12 method checklist: mercator, sinusoidal, cylequalarea, cylindrical, rectangular, gall, mollweide, gilbert, azequidistant, azequalarea, gnomonic, perspective, orthographic, stereographic, laue, fisheye, newyorker, conic, simpleconic, lambert, albers, bonne, polyconic, aitoff, lagrange, bicentric, elliptic, globular, vandergrinten, eisenlohr, guyou, square, tetra, hex, harrison, trapezoidal, lune, mecca, homing, sp_mercator, sp_albers. Each needs its parameter/orientation contract; matching a name alone is insufficient. [mapproj 1.2.12 manual](https://cran.r-project.org/web/packages/mapproj/mapproj.pdf).

Candidate limitations and license MIT OR Apache-2.0 are documented by [proj4rs](https://docs.rs/proj4rs/latest/proj4rs/index.html) and its [projection module inventory](https://docs.rs/proj4rs/latest/proj4rs/projections/index.html). A native PROJ adapter can cover additional capabilities only with explicit host/resource declarations and corresponding WASM decisions; it cannot close unsupported shared rows.

## GG-17 complete device matrix

The platform columns below are **proposed destinations**, not implemented or certified support. Existing SVG/PDF/PNG behavior and current bindings retain their established evidence; the isolated WASM codec check does not make the full chart-export crate a browser device.

| Device | Native/Python decision | WASM decision | Acceptance and remaining work |
|---|---|---|---|
| SVG | Preserve existing vector owner | Preserve scene/SVG baseline | Fonts, clipping, physical bounds and retained vectors. |
| PDF | Preserve existing krilla owner | No new availability claim | Independent PDF decode, fonts, multipage contract. |
| PNG | Preserve existing renderer/encoder | No new availability claim | Density metadata, alpha and dimensions. |
| JPEG | image codec candidate after shared rasterization | Codec compiles; host route unimplemented | Explicit alpha flattening/background, quality, DPI, independent lossy decoding. |
| TIFF | image codec candidate | Codec compiles; host route unimplemented | Alpha convention, compression/options, physical resolution and multi-page support must be checked separately. |
| BMP | image codec candidate | Codec compiles; host route unimplemented | Bit depth, row order/padding, color/alpha and resolution metadata. |
| PS | New direct vector emitter over shared scene paths | Not required baseline; no runtime claim | Pages, bounding boxes, fills/dashes, fonts, color models; independent Ghostscript inspection. |
| EPS | Shared PS owner with EPS-specific document policy | Same as PS | Single-page EPS header/bounds and retained vectors; no renamed raster file. |
| TeX/PicTeX | New explicit restricted device | No availability claim | Compile with independent TeX; historical device restrictions must be represented. |
| Windows metafile | Windows-only adapter and Windows runner needed | Not applicable to Windows device | Actual metafile playback/inspection; distinguish WMF/EMF output identities and device options. |
| Custom device | Bounded encoded-artifact callback, host registry and diagnostics | Explicit registration only if supported | Malformed output, byte budgets, exceptions, repeated capture and stale resources. |

`ggsave` supplies the device names, units, DPI aliases, 50-inch size guard, filename inference and page numbering. Host adapters should implement those explicit policies around existing Output/ExportOptions/FigureRequest and coherent capture; no hidden current plot or implicit directory creation. [ggsave contract](https://ggplot2.tidyverse.org/reference/ggsave.html).

PicTeX is deprecated but still a documented route: it lacks color and plotmath/font metrics, ignores ordinary widths, and permits multiple plot environments. A generic SVG-in-TeX wrapper is not this device. [R PicTeX](https://stat.ethz.ch/R-manual/R-devel/library/grDevices/html/pictex.html). PS includes page/color/font policies distinct from EPS; transparent backgrounds and semi-transparent primitives need explicit fidelity treatment. [R PostScript](https://stat.ethz.ch/R-manual/R-devel/library/grDevices/html/postscript.html). Windows metafile evidence belongs on its declared host. [R Windows devices](https://stat.ethz.ch/R-manual/R-devel/library/grDevices/html/windows.html).

## Fast qualification order

Run matrix/codec micro-proofs first, then frozen independent oracle fixtures, then one focused actual-host matrix per package. Reuse compiled host modules until shared implementation changes. Reserve a complete cumulative run for integration milestones. Numeric fixtures need values/diagnostics, not an export for every parameter combination; export a deliberately selected boundary/geometry matrix. This saves repeated builds and inspections without calling compile-only or uninspected output acceptance. Root owns ledger updates and the adoption ADRs after the remaining experiments pass.
