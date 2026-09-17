# ADR-030: shared two-dimensional statistics

Status: accepted for GG11 implementation; package acceptance remains in the status ledger.

Rectangular and hexagonal bins, density surfaces, contours, filled bands and ellipses
must run in the same filtered/statistical pipeline as existing chart statistics.
Host adapters must not independently aggregate, fit, interpolate or assemble topology.

One typed `SpatialSpec` feeds the existing statistic schema, scale-population planner
and generated-row stage. Algorithms own their source-specific weight, missing-value,
normalization and failure policies. Coordinate zoom never changes these populations.
Density estimation and ellipse calculations reuse the existing bounded numerical
owners; source fixtures remain development-only R dependencies.

Contour threshold selection is shared across the eligible panel population. A bounded
grid normalization step handles rotated rectangular inputs before contour extraction.
Line contours and filled isobands retain piece/subgroup identities, missing cells,
saddles and holes. Filled bands lower through the existing polygon recipe with the
shared even-odd fill rule. This is statistical grid topology, not a general GIS engine.

Generated rows retain exact source membership through shared group storage. Their
typed fields distinguish counts, normalized counts, densities, thresholds, interval
endpoints and midpoints. Filled interval levels are ordered categories; numerical
endpoints remain separately inspectable. Existing palette and guide owners resolve
their colors. Automatic continuous spatial fill uses a colorbar; surface legend keys
use their actual rectangle/polygon paint. Explicit authored scales remain authoritative.

Public layer/stat factories lower to these same operations and existing line, tile,
polygon and hexagon recipes. Width conversion occurs once in the owning geometry
path, preserving destination units across native and publication. Definition
capability 79 identifies spatial operations or hexagon recipes. No runtime numerical
dependency or parallel host compiler is introduced. Actual host replay, publication,
native selection and aggregate checks are required before closing the package.
