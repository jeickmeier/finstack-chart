"""Standalone interpolation syntax and ownership; numerical execution is in Rust."""
from __future__ import annotations
import json
import math
import re
from collections.abc import Mapping
from typing import Generic, TypeVar
from . import _native, _Owned, ColorValue, ShapeRegistry

class MissingValue:
    """Explicit missing interpolation value, distinct from None/null."""
    __slots__ = ()
    def __repr__(self): return "MISSING"

MISSING = MissingValue()

def _number(value):
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise TypeError("Interpolation numbers require int or float.")
    value = float(value)
    if math.isnan(value): return {"number": "NaN"}
    if math.isinf(value): return {"number": "Infinity" if value > 0 else "-Infinity"}
    if value == 0 and math.copysign(1, value) < 0: return {"number": "-0"}
    return value

def _un_number(value):
    if isinstance(value, dict):
        return {"NaN": float("nan"), "Infinity": float("inf"), "-Infinity": -float("inf"), "-0": -0.0}[value["number"]]
    return float(value)

def _json(value): return json.dumps(value, allow_nan=False, separators=(",", ":"))

class InterpolationDate:
    """Core-normalized epoch milliseconds, including the full Date range and invalid dates."""
    __slots__ = ("_milliseconds",)
    def __init__(self, milliseconds):
        value=json.loads(_native._Interpolator.date_value(float(milliseconds)))
        self._milliseconds=_un_number(value["value"])
    @classmethod
    def _from_core(cls, value):
        result=object.__new__(cls);result._milliseconds=_un_number(value);return result
    @property
    def milliseconds(self): return self._milliseconds
    @property
    def valid(self): return not math.isnan(self._milliseconds)

def date_value(milliseconds): return InterpolationDate(milliseconds)

class NumericArray:
    """Owned immutable numeric-array values with a core-applied destination cast."""
    __slots__ = ("_kind", "_values")
    def __init__(self, kind, values):
        value=json.loads(_native._Interpolator.normalize_value(_json({"kind":"NumericArray","value":{"element":kind,"values":[_number(v) for v in values]}})))
        self._kind=value["value"]["element"]
        self._values=tuple(_un_number(v) for v in value["value"]["values"])
    @classmethod
    def _from_core(cls, value):
        result=object.__new__(cls);result._kind=value["element"];result._values=tuple(_un_number(v) for v in value["values"]);return result
    @property
    def kind(self): return self._kind
    @property
    def values(self): return self._values

def numeric_array(kind, values): return NumericArray(kind, values)

def _pack(value, depth=0, budget=None):
    if budget is None: budget=[0,0]
    budget[0]+=1
    if depth>32 or budget[0]>200_000: raise ValueError("Interpolation value nesting/node budget exceeded.")
    if isinstance(value, MissingValue): return {"kind":"Missing"}
    if value is None: return {"kind":"Null"}
    if isinstance(value, bool): return {"kind":"Boolean","value":value}
    if isinstance(value, (int,float)): return {"kind":"Number","value":_number(value)}
    if isinstance(value, str):
        budget[1]+=len(value.encode("utf-8"))
        if budget[1]>4*1024*1024: raise ValueError("Interpolation text budget exceeded.")
        return {"kind":"Text","value":value}
    if isinstance(value, ColorValue): return {"kind":"Color","value":value.value()}
    if isinstance(value, InterpolationDate): return {"kind":"Date","value":_number(value.milliseconds)}
    if isinstance(value, NumericArray):
        budget[0]+=len(value.values)
        if budget[0]>200_000: raise ValueError("Interpolation node budget exceeded.")
        return {"kind":"NumericArray","value":{"element":value.kind,"values":[_number(v) for v in value.values]}}
    if isinstance(value, (list,tuple)):
        return {"kind":"Array","value":[_pack(v,depth+1,budget) for v in value]}
    if isinstance(value, Mapping):
        if any(not isinstance(k,str) for k in value): raise TypeError("Interpolation records need string keys.")
        budget[1]+=sum(len(k.encode("utf-8")) for k in value)
        if budget[1]>4*1024*1024: raise ValueError("Interpolation record key budget exceeded.")
        return {"kind":"Record","value":{k:_pack(v,depth+1,budget) for k,v in value.items()}}
    raise TypeError("Unsupported interpolation value; use typed values without callbacks.")

def _unpack(value):
    kind=value["kind"]
    if kind=="Missing": return MISSING
    if kind=="Null": return None
    if kind in ("Boolean","Text"): return value["value"]
    if kind=="Number": return _un_number(value["value"])
    if kind=="Date": return InterpolationDate._from_core(value["value"])
    if kind=="Color": return ColorValue.from_json(_json({"version":1,"value":value["value"]}))
    if kind=="NumericArray": return NumericArray._from_core(value["value"])
    if kind=="Array": return [_unpack(v) for v in value["value"]]
    if kind=="Record": return {k:_unpack(v) for k,v in value["value"].items()}
    raise ValueError("Unknown core interpolation result kind.")

_T = TypeVar("_T")
class Interpolator(_Owned, Generic[_T]):
    """Owned compiled operation; samples remain independent of later calls and disposal."""
    @classmethod
    def from_json(cls, value, registry=None):
        return cls(_native._Interpolator.from_json(value) if registry is None else _native._Interpolator.from_json_registered(value, registry._inner))
    def to_json(self): return self._inner.to_json()
    def copy(self): return Interpolator(self._inner.copy())
    def sample(self, t): return _unpack(json.loads(self._inner.sample_json(t)))
    def __call__(self, t): return self.sample(t)
    def sample_value(self, t): return json.loads(self._inner.sample_json(t))
    def sample_color(self, t): return ColorValue(self._inner.sample_color(t))
    def sample_transform(self, t): return json.loads(self._inner.sample_transform_json(t))
    def quantize(self, count): return _unpack(json.loads(self._inner.quantize_json(count)))
    @property
    def duration(self): return self._inner.duration_ms()
    @property
    def scheduling_duration(self): return self._inner.scheduling_duration_ms()

class _Factory:
    __slots__=("_name","_options")
    def __init__(self, name, options=None): self._name=name;self._options=options or {}
    def __call__(self, *args):
        budget=[0,0]
        return Interpolator(_native._Interpolator(self._name,_json([_pack(v,0,budget) for v in args]),_json(self._options)))
    def gamma(self, value):
        _native._Interpolator.factory(self._name,float(value))
        return _Factory(self._name,{"gamma":_number(value)})
    def rho(self, value):
        if self._name!="interpolateZoom": raise TypeError("Only zoom has a rho configuration.")
        return _Factory(self._name,{"rho":_number(value)})
    def _descriptor(self):
        if "rho" in self._options: raise TypeError("Zoom is not a binary value factory for piecewise.")
        gamma=self._options.get("gamma")
        return json.loads(_native._Interpolator.factory(self._name,None if gamma is None else _un_number(gamma)))

class RegisteredInterpolationFactory:
    """An installed Rust factory selection with an independently owned registry snapshot."""
    __slots__ = ("_registry", "_factory")
    def __init__(self, registry, operation_id, version, parameters=None):
        if not isinstance(registry, ShapeRegistry): raise TypeError("Expected a ShapeRegistry.")
        if type(version) not in (int, str) or not re.fullmatch(r"[0-9]+", str(version)):
            raise TypeError("Factory version requires an exact nonnegative integer.")
        self._factory = json.loads(registry._inner.interpolation_factory_json(
            _json({"id": operation_id, "version": str(version)}), _json({} if parameters is None else parameters)))
        self._registry = registry.copy()
    def _descriptor(self):
        self._registry._inner.interpolation_factory_json(_json(self._factory["registration"]["operation"]), _json(self._factory["registration"]["parameters"]))
        return json.loads(_json(self._factory))
    def _compile(self, spec): return Interpolator(_native._Interpolator.from_spec(_json(spec), self._registry._inner))
    def __call__(self, a, b):
        budget=[0,0]
        return self._compile({"operation":"Between", "factory":self._descriptor(), "a":_pack(a,0,budget), "b":_pack(b,0,budget)})
    def copy(self):
        result=object.__new__(type(self));result._factory=self._descriptor();result._registry=self._registry.copy();return result
    def dispose(self): self._registry.dispose()
    def __enter__(self): self._descriptor();return self
    def __exit__(self, *args): self.dispose()

def registered_interpolation(registry, operation_id, version, parameters=None):
    return RegisteredInterpolationFactory(registry, operation_id, version, parameters)

def piecewise(*args):
    if len(args)==1: factory=interpolate;values=args[0]
    elif len(args)==2: factory,values=args
    else: raise TypeError("piecewise expects values or a built-in factory and values.")
    if isinstance(factory,RegisteredInterpolationFactory):
        budget=[0,0]
        return factory._compile({"operation":"Piecewise", "factory":factory._descriptor(), "values":[_pack(v,0,budget) for v in values]})
    if not isinstance(factory,_Factory): raise TypeError("Portable piecewise needs a registered built-in factory.")
    return _Factory("piecewise",{"factory":factory._descriptor()})(values)

def quantize(interpolator, count):
    if not isinstance(interpolator,Interpolator): raise TypeError("quantize requires an owned Interpolator.")
    return interpolator.quantize(count)

__all__=["RegisteredInterpolationFactory","registered_interpolation","MissingValue","MISSING","InterpolationDate","NumericArray","Interpolator","date_value","numeric_array","piecewise","quantize"]
for _name in ["interpolate","interpolateArray","interpolateBasis","interpolateBasisClosed","interpolateDate","interpolateDiscrete","interpolateHue","interpolateNumber","interpolateNumberArray","interpolateObject","interpolateRound","interpolateString","interpolateTransformCss","interpolateTransformSvg","interpolateZoom","interpolateRgb","interpolateRgbBasis","interpolateRgbBasisClosed","interpolateHsl","interpolateHslLong","interpolateLab","interpolateHcl","interpolateHclLong","interpolateCubehelix","interpolateCubehelixLong"]:
    _snake=re.sub(r"(?<!^)(?=[A-Z])","_",_name).lower()
    globals()[_snake]=_Factory(_name);__all__.append(_snake)

# Named color catalogs use the same owned interpolation handle and value decoder.
def chromatic_catalog(): return json.loads(_native._Interpolator.chromatic_catalog())
def chromatic_scheme(name, size=None, reverse=False):
    return _unpack(json.loads(_native._Interpolator.chromatic_scheme(_json({"version":1,"id":name,"size":size,"reverse":reverse}))))
def chromatic(name, reverse=False): return Interpolator(_native._Interpolator.chromatic(name, reverse))
__all__ += ["chromatic_catalog", "chromatic_scheme", "chromatic"]
