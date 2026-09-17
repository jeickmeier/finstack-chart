# GG15 mapproj oracle readiness — 16 September 2026

Pinned R 4.6.1, mapproj 1.2.12 and maps 3.4.3 were verified in the isolated temporary R library. The capture covers **41 documented projection names, 426 independent calls and one stateful reuse case**. All 41 standard parameterizations were recognized; results retain per-point finite/NA/NaN/infinity classes, projection error flags, warnings, resolved orientation and returned bounds. Parameter failures remain failures in the fixture.

The generator is `tools/reference/r/mapproj-controls.R`; numeric evidence is `fixtures/parity/ggplot2/mapproj-controls.json`. It covers every documented parameter, including independent boundary perturbations for both parameters of two-parameter methods. Every method has explicit standard, tilted and dateline-rotated orientation, data-dependent default orientation, absent parameters and invalid orientation length; parameterized methods add zero/negative/90/180 values and wrong arities. Inputs cover interior points, poles, dateline sides, the Mercator 80-degree boundary, invalid latitude and missing values. This is a bounded acceptance matrix, not a proof over every possible parameter.

Commands:

```sh
CHART_REFERENCE_R_LIBRARY=/private/tmp/finstack-chart-tools/r-library CHART_REFERENCE_R_WORK=/private/tmp/finstack-chart-tools/r-work mise exec -- python3 tools/reference/r/run.py tools/reference/r/mapproj-controls.R
```

Installation log: `/private/tmp/gg15-install.log`; capture log: `/private/tmp/gg15-mapproj-capture.log`. Private oracle binaries only; no runtime dependency, workspace manifest or geometry change.

## Exact candidate inventory boundary

The installed proj4rs 0.1.10 source registry contains nine potentially related kernels for eleven mapproj rows. **None is certified as a mapproj alias. Thirty documented rows have no matching kernel.** The default-features-off Mercator spike includes neither `aeqd` nor `esri`; `aeqd` adds the optional geodesic dependency and `esri` admits `cea`. Native/WASM Mercator portability already passed in the parent spike; that does not qualify other kernels or parameter conventions.

| mapproj method | Parameters | Candidate | Feature | Status / convention |
|---|---|---|---|---|
| mercator | none | merc | base | Sphere must be explicit; source excludes points above its 80-degree cap. |
| sinusoidal | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| cylequalarea | lat0 | cea | esri | lat0 maps conceptually to lat_ts; source requires one parameter, candidate defaults scale to k0. |
| cylindrical | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| rectangular | lat0 | eqc | base | lat0 maps to lat_ts; candidate defaults0, source requires one parameter; candidate forces sphere. |
| gall | lat0 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| mollweide | none | moll | base | Candidate forces sphere; scaling/normalization and boundary equivalence remain unqualified. |
| gilbert | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| azequidistant | none | aeqd | aeqd | Source standard is north-polar; candidate lat_0 defaults0; explicit90 required before comparison. |
| azequalarea | none | laea | base | Source standard is north-polar; candidate lat_0 defaults0; explicit90 required before comparison. |
| gnomonic | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| perspective | dist | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| orthographic | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| stereographic | none | stere | base | Source standard is north-polar; candidate supports polar/equatorial/oblique and ellipsoid; k0 normalization unqualified. |
| laue | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| fisheye | n | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| newyorker | r | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| conic | lat0 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| simpleconic | lat0, lat1 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| lambert | lat0, lat1 | lcc | base | Source requires lat0,lat1; candidate lat_1/lat_2 defaults differ; origin and spherical convention unqualified. |
| albers | lat0, lat1 | aea | base | Source requires lat0,lat1; candidate defaults both0 and rejects degenerate cone; origin/normalization unqualified. |
| bonne | lat0 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| polyconic | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| aitoff | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| lagrange | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| bicentric | lon0 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| elliptic | lon0 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| globular | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| vandergrinten | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| eisenlohr | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| guyou | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| square | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| tetra | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| hex | none | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| harrison | dist, angle | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| trapezoidal | lat0, lat1 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| lune | lat, angle | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| mecca | lat0 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| homing | lat0 | missing | base | No matching projection kernel in the pinned proj4rs registry; similarly named projections are not substitutes. |
| sp_mercator | none | merc | base | Source spheroid constants must be identified; no assumed WGS84 equivalence. |
| sp_albers | lat0, lat1 | aea | base | Two parallels plus exact reference spheroid required; no assumed WGS84 equivalence. |

## Adapter contracts still open

mapproj accepts degrees, rotates a spherical overlay using a three-element orientation and emits reference projection units. Its omitted orientation is `[90, 0, mean(longitude range)]`, which changes with input; empty projection names reuse the previous projection and parameters. A pure immutable runtime must capture this resolved identity explicitly rather than inherit R global state. proj4rs takes radians for geographic coordinates and supports explicit ellipsoid/radius, units, axis and projection origins; these are not the same orientation API. All adapters must qualify normalization, central meridian, latitude origin, radius/ellipsoid, clipping/unprojectable status and antimeridian behavior before claiming equivalence.

The mapproj manual explicitly says spheroid methods are not meaningful for tilted orientations. Captured tilted results for those methods are evidence of observable output, not endorsement of geodetic correctness. The reference spheroid constants remain to be identified from pinned package source; neither WGS84 nor spherical fallback is assumed.

Missing kernels require original qualified implementations or an explicit service boundary. `geos` is geostationary satellite view, not the documented mapproj `perspective`/`harrison`; `sterea` is not a substitute for the periodic conformal methods. `eqc` does not implement central cylindrical, and `lcc` does not implement central/simple conic. No missing row is silently mapped to one of these.

GG15 remains open: geometry/datum/CRS resources, sf runtime identity and geographic fixtures, all missing projection implementations, numeric candidate comparisons beyond the already-qualified Mercator portability anchor, native+WASM integration and actual publication/interaction acceptance.
