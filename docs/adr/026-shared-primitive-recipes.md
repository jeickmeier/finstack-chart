# ADR-026: Primitive recipes and retained statistical aesthetics

Status: accepted for GG-07 implementation, 15 September 2026.

Primitive families use one optional typed `BuiltinRecipe` on an ordinary layer.
Extra source/generated channels reuse the existing mapping and expression readers.
Interval bounds and tile/column widths resolve before positioning; compound marks
are emitted afterward so stacking and nudging apply once to each statistical row.
Every component retains the original row or aggregate target. Reference equations
use the resolved axes without becoming observations that train those axes.

Physical curves use one core implementation of the grid control polygon and
rational X-spline, checked against pinned R paths. All destinations consume that
same projected geometry. Arrows and interval components use shared stroke, units,
clipping and path budgets. Independent middle/box/point styles are retained on
interval recipes. Optional line ends and joins use one bounded filled stroke
outline in core, including dashed runs, with a miter limit of ten. The same-winding
outline is painted once so overlapping regions do not multiply alpha. Absent
controls preserve existing primitives. These controls are data, not host callbacks.
The two optional stroke selectors increase the aligned resolved `Style` from 56
to 64 bytes, still one cache line; `Color` remains four bytes. The explicit style
size regression records this intentional cost separately from serialized legacy
compatibility. Shared dash lengths use deterministic `libm` so native and WASM
construct the same portable outline endpoints.

StatSum extends the count operation with opt-in joint positions and explicit
partition inputs. The original group continues to own proportion normalization.
Source aesthetic fields and evaluated numeric expressions are retained only when
the statistical partition proves them identical. Non-injective expressions such
as `abs(x)` partition by the evaluated value, so `-1` and `1` can share one count.
Numeric and categorical scale readers and guide training consume these retained
values; no representative source row is substituted for aggregate provenance.
Older statistics serialize no retained metadata when it is empty. Rust producers
that construct `StatisticalRow` directly initialize the new fields to `All` and
an empty vector unless their operation proves retention; the registered extension
example is updated accordingly.

Definition envelope 74 identifies recipe selectors/channels and joint count
options or explicit line-end/join styles. Scene envelope 21 identifies nondefault path fill rules or raster cell
interaction bounds. Empty raster cell metadata preserves the GG-08 whole-image
interaction contract and scene version 20. One row-grid image keeps its source
cell targets and hit rectangles without adding invisible paint primitives.

Python and WASM forward recipe configuration and owned field/expression inputs to
canonical Rust builders. Neither adapter computes geometry or statistics. The
existing runtime ownership, portable replay and explicit font/resource boundaries
remain in force. This work introduces no runtime dependency versions.
