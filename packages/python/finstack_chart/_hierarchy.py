"""Owned hierarchy operations; topology and numerical layouts execute in Rust."""
from . import _native, _Owned, _encode, _decode, ShapeRegistry

class Hierarchy(_Owned):
    """Versioned, keyed input with atomic operations and independent layout history.

    Identity fields in descriptors are canonical decimal strings. Queries return
    owned dictionaries; changing them never changes the retained Rust snapshot.
    """
    def __init__(self, envelope, registry=None):
        super().__init__(_native._Hierarchy(_encode(envelope)) if registry is None else _native._Hierarchy.registered(_encode(envelope), registry._inner))
    @classmethod
    def _wrap(cls, inner):
        value=cls.__new__(cls);_Owned.__init__(value,inner);return value
    @classmethod
    def from_json(cls, value, registry=None):
        registry=ShapeRegistry() if registry is None else registry
        return cls._wrap(_native._Hierarchy.from_snapshot(value,registry._inner))
    def to_json(self): return self._inner.to_json()
    def copy(self): return self._wrap(self._inner.copy())
    def copy_subtree(self,node,identity): return self._wrap(self._inner.copy_subtree(_encode(node),_encode(str(identity))))
    def apply(self,change): self._inner.apply(_encode(change));return self
    def query(self,query): return _decode(self._inner.query(_encode(query)))
    def nodes(self): return self.query("Nodes")
    def node(self,node): return self.query({"Node":node})
    def ancestors(self,node): return self.query({"Ancestors":node})
    def descendants(self,node): return self.query({"Descendants":node})
    def leaves(self,node): return self.query({"Leaves":node})
    def links(self,node): return self.query({"Links":node})
    def path(self,start,end): return self.query({"Path":{"start":start,"end":end}})
    def visit(self,root,order="BreadthFirst"): return self.query({"Visit":{"root":root,"order":order}})
    def find(self,root,field,value): return self.query({"Find":{"root":root,"field":field,"value":value}})
    def find_registered(self,root,predicate): return self.query({"FindRegistered":{"root":root,"predicate":predicate}})
    def sum(self,accessor): return self.apply({"Sum":accessor})
    def count(self): return self.apply("Count")
    def sort(self,comparator): return self.apply({"Sort":comparator})
    def layout(self,configuration): return self.apply({"Layout":configuration})
    def replace(self,envelope): return self.apply({"Replace":envelope})
    def reset_history(self): return self.apply("ResetHistory")
    def configuration(self): return self.query("Configuration")
    def tile(self,request): return _decode(self._inner.tile(_encode(request)))

def pack_siblings(circles,limits=None):
    request={"version":1,"siblings":True,"circles":circles}
    if limits is not None:request["limits"]=limits
    return _decode(_native._Hierarchy.packing(_encode(request)))

def pack_enclose(circles,limits=None):
    request={"version":1,"siblings":False,"circles":circles}
    if limits is not None:request["limits"]=limits
    return _decode(_native._Hierarchy.packing(_encode(request)))
