"""Owned standalone scale syntax. All preparation, arithmetic and defaults are in Rust."""
from __future__ import annotations
import json
from dataclasses import dataclass
from . import _native, _Owned
from ._interpolation import MISSING, MissingValue, Interpolator, _pack, _unpack, _number, _un_number, _json

@dataclass(frozen=True)
class ScaleKey:
    """Explicit category identity when native types cannot distinguish a key kind."""
    kind: str
    value: object = None

def _key(value):
    if isinstance(value, ScaleKey):
        if value.kind == "Null": return "Null"
        return {value.kind: str(value.value) if value.kind in ("Integer", "Unsigned", "Timestamp") else _number(value.value) if value.kind == "Number" else value.value}
    if value is None: return "Null"
    if type(value) is bool: return {"Boolean": value}
    if type(value) is int: return {"Integer": str(value)}
    if type(value) is float: return {"Number": _number(value)}
    if type(value) is str: return {"Text": value}
    raise TypeError("Scale categories require null, bool, int, float, str or ScaleKey.")

def _un_key(key):
    if key == "Null": return None
    kind, value = next(iter(key.items()))
    if kind == "Number": return _un_number(value)
    if kind == "Integer": return int(value)
    if kind in ("Unsigned", "Timestamp"): return ScaleKey(kind, int(value))
    return value

def _exact(value):
    if type(value) is not int: raise TypeError("Time scales require exact Python int timestamps.")
    return str(value)

def _input(value, mode):
    if isinstance(value, MissingValue) or value is None and mode != "Key": return "Missing"
    return {mode: _key(value) if mode == "Key" else _exact(value) if mode == "Time" else _number(value)}

def _un_input(value):
    if value == "Missing": return MISSING
    kind, payload = next(iter(value.items()))
    if kind == "Value": return _unpack(payload)
    return _un_key(payload) if kind == "Key" else int(payload) if kind == "Time" else _un_number(payload)

def _options(options, mode):
    result = dict(options)
    if "domain" in result: result["domain"] = [_input(v, mode) for v in result["domain"]]
    if "range" in result: result["range"] = [_pack(v) for v in result["range"]]
    if "unknown" in result: result["unknown"] = _pack(result["unknown"])
    if "interpolator" in result:
        i = result["interpolator"]
        if not isinstance(i, Interpolator): raise TypeError("Scale interpolator requires an owned Interpolator.")
        result["interpolator"] = json.loads(i.to_json())["spec"]
    if "factory" in result:
        f = result["factory"]
        result["factory"] = f._descriptor() if hasattr(f, "_descriptor") else f
    return result

def _selection(count, interval):
    return {"Interval": interval} if interval is not None else {"Count": _number(count)}

class StandaloneScale(_Owned):
    """Immutable scale. configure, nice and train return independent owned scales."""
    def __init__(self, family="linear", **options):
        mode = "Key" if family in ("ordinal", "band", "point", "threshold") else "Time" if family in ("utc", "local") else "Number"
        super().__init__(_native._Scale.create(_json(family), _json(_options(options, mode))))
        self._read_mode()
    def _read_mode(self):
        family = next(iter(self.spec()))
        self._mode = "Key" if family in ("Ordinal", "Band", "Point", "Threshold") else "Time" if family == "Time" else "Number"
    @classmethod
    def _wrap(cls, inner):
        s = object.__new__(cls); _Owned.__init__(s, inner); s._read_mode(); return s
    @classmethod
    def from_json(cls, text): return cls._wrap(_native._Scale.from_json(text))
    @classmethod
    def from_spec(cls, spec): return cls._wrap(_native._Scale(_json(spec)))
    def to_json(self): return self._inner.to_json()
    def copy(self): return type(self)._wrap(self._inner.copy())
    def _query(self, value): return json.loads(self._inner.query(_json(value)))
    def _change(self, value): return type(self)._wrap(self._inner.change(_json(value)))
    def spec(self): return self._query("Spec")
    def mapped(self, training="Authored"): return self._query({"Mapped": training})
    def configure(self, **options): return self._change({"Configure": _options(options, self._mode)})
    def reconfigure(self, spec): return self._change({"Reconfigure": spec})
    def domain(self): return tuple(_un_input(v) for v in self._query("Domain"))
    def range(self): return tuple(_unpack(v) for v in self._query("Range"))
    def map(self, value=MISSING): return _unpack(self._query({"Map": _input(value, self._mode)}))
    def map_value(self, value=MISSING): return self._query({"Map": _input(value, self._mode)})
    def invert(self, value): return _un_input(self._query({"Invert": _number(value)}))
    def invert_extent(self, value):
        result = self._query({"InvertExtent": _pack(value)})
        return {"found": result["found"], "lower": MISSING if result["lower"] is None else _un_key(result["lower"]), "upper": MISSING if result["upper"] is None else _un_key(result["upper"])}
    def ticks(self, count=10, *, interval=None, budget=10000):
        if self._mode == "Time": return tuple(_un_input(v) for v in self._query({"TimeTicks": {"selection": _selection(count, interval), "budget": budget}}))
        if interval is not None: raise TypeError("Calendar intervals require a time scale.")
        return tuple(_un_number(v) for v in self._query({"Ticks": {"count": _number(count), "budget": budget}}))
    def format(self, value, *, count=10, specifier=None, pattern=None, locale=None):
        if self._mode == "Time":
            if specifier is not None: raise TypeError("Time scales use pattern, not numeric specifier.")
            return self._query({"TimeFormat": {"value": _exact(value), "format": {"pattern": pattern, "locale": locale or {}}}})
        if pattern is not None: raise TypeError("Numeric scales use specifier, not time pattern.")
        return self._query({"Format": {"value": _number(value), "count": _number(count), "specifier": specifier, "locale": locale or {}}})
    def nice(self, count=10, *, interval=None):
        if self._mode == "Time": return self._change({"NiceTime": _selection(count, interval)})
        if interval is not None: raise TypeError("Calendar intervals require a time scale.")
        return self._change({"Nice": _number(count)})
    def train(self, values): return self._change({"Train": [_key(v) for v in values]})
    def thresholds(self): return tuple(None if v is None else _un_number(v) for v in self._query("Thresholds"))
    def quantiles(self, count): return tuple(None if v is None else _un_number(v) for v in self._query({"Quantiles": _number(count)}))
    def step(self): return _un_number(self._query("Step"))
    def bandwidth(self): return _un_number(self._query("Bandwidth"))
    def extent(self, value):
        v = self._query({"Extent": _key(value)}); return None if v is None else (v["start"], v["end"])
    def center(self, value):
        v = self._query({"Center": _key(value)}); return None if v is None else _un_number(v)
    def _calendar(self, method, value, interval, **extra): return _un_input(self._query({method: {"value": _exact(value), "interval": interval, **extra}}))
    def floor(self, value, interval): return self._calendar("Floor", value, interval)
    def ceil(self, value, interval): return self._calendar("Ceil", value, interval)
    def round_time(self, value, interval): return self._calendar("RoundTime", value, interval)
    def offset(self, value, interval, steps=1): return self._calendar("Offset", value, interval, steps=_number(steps))

__all__ = ["ScaleKey", "StandaloneScale"]
