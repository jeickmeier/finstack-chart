from collections.abc import Sequence
from typing import Literal, TypeAlias, TypedDict, NotRequired, Self
from . import _Owned, ShapeRegistry
HierarchyJSON: TypeAlias = None | bool | int | float | str | list['HierarchyJSON'] | dict[str, 'HierarchyJSON']
class HierarchyLimits(TypedDict, total=False):
    max_nodes: int
    max_depth: int
    max_work: int
    max_payload_bytes: int
class HierarchyHandle(TypedDict):
    hierarchy: str
    node: str
class _Operation(TypedDict):
    id: str
    version: str
class HierarchyOperation(TypedDict):
    operation: _Operation
    parameters: NotRequired[HierarchyJSON]
class _Constant(TypedDict):
    Constant: float
class _Field(TypedDict):
    Field: str
class _Registered(TypedDict):
    Registered: HierarchyOperation
HierarchyScalar: TypeAlias = _Constant | _Field | _Registered | Literal['Value','Depth']
class _ScalarOrder(TypedDict):
    accessor: HierarchyScalar
    descending: bool
class _ScalarComparator(TypedDict):
    Scalar: _ScalarOrder
class _TextOrder(TypedDict):
    field: str
    descending: bool
class _TextComparator(TypedDict):
    Text: _TextOrder
HierarchyComparator: TypeAlias = _ScalarComparator | _TextComparator | _Registered
class HierarchyPayload(TypedDict):
    key: str
    data: HierarchyJSON
class HierarchyRow(HierarchyPayload):
    parent: str | None
    id: NotRequired[str | None]
    synthetic: NotRequired[bool]
class HierarchyNested(HierarchyPayload):
    children: Sequence[HierarchyNested]
class HierarchyGroup(TypedDict):
    key: str
    label: HierarchyJSON
    value: HierarchyJSON
    children: Sequence[HierarchyGroup]
class StratifyOptions(TypedDict, total=False):
    id_field: str | None
    parent_field: str | None
    path_field: str | None
class _Rows(TypedDict):
    Rows: Sequence[HierarchyRow]
class _Nested(TypedDict):
    Nested: HierarchyNested
class _Node(TypedDict):
    Node: HierarchyPayload
class _Groups(TypedDict):
    root: str
    entries: Sequence[HierarchyGroup]
class _Grouped(TypedDict):
    Grouped: _Groups
class _StratifyInput(TypedDict):
    rows: Sequence[HierarchyPayload]
    options: StratifyOptions
class _Stratify(TypedDict):
    Stratify: _StratifyInput
class _ChildrenInput(TypedDict):
    root: HierarchyPayload
    accessor: HierarchyOperation
class _Children(TypedDict):
    Children: _ChildrenInput
HierarchyInput: TypeAlias = _Rows | _Nested | _Node | _Grouped | _Stratify | _Children
class HierarchyEnvelope(TypedDict):
    version: Literal[1]
    identity: str
    limits: NotRequired[HierarchyLimits]
    input: HierarchyInput
class _Squarify(TypedDict):
    Squarify: float
class _Resquarify(TypedDict):
    Resquarify: float
HierarchyTiler: TypeAlias = Literal['Binary','Dice','Slice','SliceDice','Custom'] | _Squarify | _Resquarify
class _Extent(TypedDict):
    Extent: tuple[float,float]
class _NodeSize(TypedDict):
    NodeSize: tuple[float,float]
class HierarchyTreeOptions(TypedDict, total=False):
    mode: _Extent | _NodeSize
    separation: Literal['Default','Depth'] | _Constant
class HierarchyPartitionOptions(TypedDict, total=False):
    size: tuple[float,float]
    round: bool
    padding: float
class HierarchyPaddingSides(TypedDict, total=False):
    Inner: HierarchyScalar
    Top: HierarchyScalar
    Right: HierarchyScalar
    Bottom: HierarchyScalar
    Left: HierarchyScalar
class HierarchyTreemapOptions(TypedDict, total=False):
    size: tuple[float,float]
    round: bool
    tile: HierarchyTiler
    padding_inner: float
    padding_top: float
    padding_right: float
    padding_bottom: float
    padding_left: float
class HierarchyPackOptions(TypedDict, total=False):
    size: tuple[float,float]
    radius: Literal['Fitted','Explicit']
    padding: float
class _TreeSpec(TypedDict):
    options: HierarchyTreeOptions
    separation: NotRequired[HierarchyOperation | None]
class _Tree(TypedDict):
    Tree: _TreeSpec
class _Cluster(TypedDict):
    Cluster: _TreeSpec
class _Partition(TypedDict):
    Partition: HierarchyPartitionOptions
class _TreemapSpec(TypedDict):
    options: HierarchyTreemapOptions
    history: bool
    padding: NotRequired[HierarchyScalar | None]
    tiler: NotRequired[HierarchyOperation | None]
    padding_sides: NotRequired[HierarchyPaddingSides]
class _Treemap(TypedDict):
    Treemap: _TreemapSpec
class _PackSpec(TypedDict):
    options: HierarchyPackOptions
    radius: NotRequired[HierarchyScalar | None]
    padding: NotRequired[HierarchyScalar | None]
class _Pack(TypedDict):
    Pack: _PackSpec
HierarchyLayout: TypeAlias = _Tree | _Cluster | _Partition | _Treemap | _Pack
class HierarchyCircle(TypedDict):
    x: float
    y: float
    r: float
class _PointCoordinates(TypedDict):
    x: float
    y: float
class _RectCoordinates(TypedDict):
    x0: float
    y0: float
    x1: float
    y1: float
class _PointGeometry(TypedDict):
    Point: _PointCoordinates
class _RectangleGeometry(TypedDict):
    Rectangle: _RectCoordinates
class _CircleGeometry(TypedDict):
    Circle: HierarchyCircle
HierarchyGeometry: TypeAlias = _PointGeometry | _RectangleGeometry | _CircleGeometry
class HierarchyNodeRecord(TypedDict):
    handle: HierarchyHandle
    data: HierarchyJSON
    id: str | None
    synthetic: bool
    parent: HierarchyHandle | None
    children: list[HierarchyHandle]
    depth: int
    height: int
    value: float | None
    geometry: HierarchyGeometry | None
HierarchyVisitOrder: TypeAlias = Literal['BreadthFirst','PreOrder','PostOrder']
class HierarchyTilingRequest(TypedDict):
    version: Literal[1]
    parent: HierarchyHandle
    bounds: tuple[float,float,float,float]
    tiler: HierarchyTiler
    history: bool
    operation: NotRequired[HierarchyOperation | None]
class HierarchyConfiguration(TypedDict):
    version: Literal[1]
    identity: str
    limits: HierarchyLimits
    layout: HierarchyLayout | None
    effective_ratio: float | None
    history_rows: int
    history_members: int
class _Replace(TypedDict):
    Replace: HierarchyEnvelope
class _Sum(TypedDict):
    Sum: HierarchyScalar
class _Sort(TypedDict):
    Sort: HierarchyComparator
class _Layout(TypedDict):
    Layout: HierarchyLayout
HierarchyChange: TypeAlias = Literal['Count','ResetHistory'] | _Replace | _Sum | _Sort | _Layout
class Hierarchy(_Owned):
    def __init__(self, envelope: HierarchyEnvelope, registry: ShapeRegistry | None = ...) -> None: ...
    @classmethod
    def from_json(cls, value: str, registry: ShapeRegistry | None = ...) -> Hierarchy: ...
    def to_json(self) -> str: ...
    def copy(self) -> Hierarchy: ...
    def copy_subtree(self, node: HierarchyHandle, identity: str | int) -> Hierarchy: ...
    def apply(self, change: HierarchyChange) -> Self: ...
    def query(self, query: HierarchyJSON) -> HierarchyJSON: ...
    def nodes(self) -> list[HierarchyNodeRecord]: ...
    def node(self, node: HierarchyHandle) -> HierarchyNodeRecord: ...
    def ancestors(self, node: HierarchyHandle) -> list[HierarchyHandle]: ...
    def descendants(self, node: HierarchyHandle) -> list[HierarchyHandle]: ...
    def leaves(self, node: HierarchyHandle) -> list[HierarchyHandle]: ...
    def links(self, node: HierarchyHandle) -> list[tuple[HierarchyHandle,HierarchyHandle]]: ...
    def path(self, start: HierarchyHandle, end: HierarchyHandle) -> list[HierarchyHandle]: ...
    def visit(self, root: HierarchyHandle, order: HierarchyVisitOrder = ...) -> list[tuple[HierarchyNodeRecord,int,HierarchyHandle]]: ...
    def find(self, root: HierarchyHandle, field: str, value: HierarchyJSON) -> HierarchyHandle | None: ...
    def find_registered(self, root: HierarchyHandle, predicate: HierarchyOperation) -> HierarchyHandle | None: ...
    def sum(self, accessor: HierarchyScalar) -> Self: ...
    def count(self) -> Self: ...
    def sort(self, comparator: HierarchyComparator) -> Self: ...
    def layout(self, configuration: HierarchyLayout) -> Self: ...
    def replace(self, envelope: HierarchyEnvelope) -> Self: ...
    def reset_history(self) -> Self: ...
    def configuration(self) -> HierarchyConfiguration: ...
    def tile(self, request: HierarchyTilingRequest) -> list[tuple[HierarchyHandle,tuple[float,float,float,float]]]: ...
def pack_siblings(circles: Sequence[HierarchyCircle], limits: HierarchyLimits | None = ...) -> list[HierarchyCircle]: ...
def pack_enclose(circles: Sequence[HierarchyCircle], limits: HierarchyLimits | None = ...) -> HierarchyCircle | None: ...
