"""Typed materialized shape descriptors; numerical behavior is owned by Rust."""
from collections.abc import Sequence, Mapping
from typing import Literal, TypedDict, NotRequired, TypeAlias
class _Column(TypedDict):
    Column: int
class _Constant(TypedDict):
    Constant: float
ShapeCoordinate: TypeAlias = _Column | _Constant
class _PlainCurve(TypedDict):
    kind: Literal['Linear', 'LinearClosed', 'Basis', 'BasisOpen', 'BasisClosed', 'BumpX', 'BumpY', 'MonotoneX', 'MonotoneY', 'Natural', 'Step', 'StepBefore', 'StepAfter']
class _BundleCurve(TypedDict):
    kind: Literal['Bundle']
    beta: NotRequired[float]
class _CardinalCurve(TypedDict):
    kind: Literal['Cardinal', 'CardinalOpen', 'CardinalClosed']
    tension: NotRequired[float]
class _CatmullCurve(TypedDict):
    kind: Literal['CatmullRom', 'CatmullRomOpen', 'CatmullRomClosed']
    alpha: NotRequired[float]
CurveSpec: TypeAlias = _PlainCurve | _BundleCurve | _CardinalCurve | _CatmullCurve
class ShapeLimits(TypedDict, total=False):
    max_points: int
    path: Mapping[str, object]
class _ShapeCommon(TypedDict, total=False):
    curve: CurveSpec
    defined: bool | Sequence[bool]
    digits: float | None
    limits: ShapeLimits
class ShapeLineConfig(_ShapeCommon, total=False):
    x: ShapeCoordinate
    y: ShapeCoordinate
class ShapeAreaConfig(_ShapeCommon, total=False):
    x0: ShapeCoordinate
    y0: ShapeCoordinate
    x1: ShapeCoordinate | None
    y1: ShapeCoordinate | None

class ArcDatum(TypedDict, total=False):
    inner_radius: float
    outer_radius: float
    start_angle: float
    end_angle: float
    pad_angle: float
class ShapeArcConfig(TypedDict, total=False):
    inner_radius: float | None
    outer_radius: float | None
    start_angle: float | None
    end_angle: float | None
    pad_angle: float | None
    pad_radius: float | None
    corner_radius: float
    digits: float | None
    limits: ShapeLimits
class PieAngles(TypedDict, total=False):
    start_angle: float
    end_angle: float
    pad_angle: float
class ShapePieConfig(TypedDict, total=False):
    value: float | None
    order: Literal['ValuesDescending','ValuesAscending','Input']
    angles: PieAngles
    limits: ShapeLimits
class PieSlice(TypedDict):
    data: object
    index: int
    value: float
    start_angle: float
    end_angle: float
    pad_angle: float

class ArcParameters(TypedDict):
    datum: ArcDatum
    corner_radius: float
    pad_radius: float | None

class _GgplotShape(TypedDict):
    Ggplot: int
SymbolKind: TypeAlias = Literal['Circle','Cross','Diamond','Square','Star','Triangle','Wye','Plus','Times','X','Asterisk','Diamond2','Square2','Triangle2'] | _GgplotShape
class ShapeSymbolConfig(TypedDict, total=False):
    kind: SymbolKind
    size: float
    digits: float | None
    limits: ShapeLimits

class _StackPermutation(TypedDict):
    Explicit: Sequence[int]
StackOrder: TypeAlias = Literal['None','Reverse','Ascending','Descending','Appearance','InsideOut'] | _StackPermutation
StackOffset: TypeAlias = Literal['None','Expand','Diverging','Silhouette','Wiggle']
StackMissing: TypeAlias = Literal['Gap','Zero','Error']
class StackLimits(TypedDict, total=False):
    max_series: int
    max_cells: int
    max_work: int
class ShapeStackConfig(TypedDict, total=False):
    keys: Sequence[str]
    order: StackOrder
    offset: StackOffset
    missing: StackMissing
    value: float | None
    limits: StackLimits
class StackPoint(TypedDict):
    data: object
    y0: float
    y1: float
class StackSeries(TypedDict):
    key: str
    index: int
    points: list[StackPoint]

class ShapeLineRadialConfig(_ShapeCommon, total=False):
    angle: ShapeCoordinate
    radius: ShapeCoordinate
class ShapeAreaRadialConfig(_ShapeCommon, total=False):
    angle: ShapeCoordinate
    radius: ShapeCoordinate
    start_angle: ShapeCoordinate
    end_angle: ShapeCoordinate | None
    inner_radius: ShapeCoordinate
    outer_radius: ShapeCoordinate | None
RadialBoundary: TypeAlias = Literal['StartAngle','EndAngle','InnerRadius','OuterRadius']
class _LinkConstant(TypedDict):
    Constant: Sequence[float]
LinkEndpoint: TypeAlias = Literal['Source','Target'] | _LinkConstant
class LinkDatum(TypedDict):
    source: Sequence[float]
    target: Sequence[float]
class _LinkCommon(TypedDict, total=False):
    source: LinkEndpoint
    target: LinkEndpoint
    digits: float | None
    limits: ShapeLimits
class ShapeLinkConfig(_LinkCommon, total=False):
    x: ShapeCoordinate
    y: ShapeCoordinate
    curve: CurveSpec
class ShapeLinkRadialConfig(_LinkCommon, total=False):
    angle: ShapeCoordinate
    radius: ShapeCoordinate

class RadialParameters(TypedDict, total=False):
    start_angle: float
    end_angle: float | None
    inner_radius: float
    outer_radius: float | None

ShapeJSON: TypeAlias = None | bool | int | float | str | Sequence['ShapeJSON'] | dict[str, 'ShapeJSON']
class ShapeOperationId(TypedDict):
    id: str
    version: str | int
class ShapeOperation(TypedDict):
    operation: ShapeOperationId
    parameters: ShapeJSON
ShapeFamily: TypeAlias = Literal['Curve','Symbol','PieComparator','StackOrder','StackOffset']
