"""Primary owned chart authoring. Semantics, validation and execution live in Rust.

Columns accept ordinary Python sequences. Accessors in Data.rows run once during
materialization; no Python callback is retained by a chart or export request.
"""
from __future__ import annotations
import json as _json
from collections.abc import Mapping as _Mapping
from pathlib import Path as _Path
import chart_python as _native

ChartError = _native.ChartError
LegacyChart = _native.Chart

def _encode(value):
    def default(value):
        if isinstance(value, ColorValue): return _decode(value.to_json())
        raise TypeError(f"Unsupported authored value: {type(value).__name__}")
    return _json.dumps(value, default=default, allow_nan=False, separators=(",", ":"))

def _decode(value):
    return _json.loads(value)

class _Owned:
    def __init__(self, inner):
        self._inner = inner
    def dispose(self):
        self._inner.dispose()
    def __enter__(self):
        return self
    def __exit__(self, *_):
        self.dispose()

class ColorValue(_Owned):
    """Owned floating color; channel arithmetic and formatting execute in Rust."""
    @classmethod
    def from_json(cls, value): return cls(_native._Color.from_json(value))
    def to_json(self): return self._inner.to_json()
    def value(self): return _decode(self._inner.value_json())
    def space(self): return self.value()["space"]
    def channels(self): return {key:self.channel(key) for key in self.value()["channels"]}
    def channel(self, name): return self._inner.channel(name)
    def with_channel(self, name, value): return ColorValue(self._inner.with_channel(name,value))
    def copy(self, channels=None):
        result=ColorValue(self._inner.copy())
        try:
            for name,value in (channels or {}).items():
                next_value=result.with_channel(name,value);result.dispose();result=next_value
            return result
        except BaseException:
            result.dispose();raise
    def convert(self, space): return ColorValue(self._inner.convert(space))
    def rgb(self): return self.convert("Rgb")
    def brighter(self, k=None): return ColorValue(self._inner.brighter(k))
    def darker(self, k=None): return ColorValue(self._inner.darker(k))
    def displayable(self): return self._inner.displayable()
    def clamp(self): return ColorValue(self._inner.clamp())
    def format_hex(self): return self._inner.format("formatHex")
    def format_hex8(self): return self._inner.format("formatHex8")
    def format_rgb(self): return self._inner.format("formatRgb")
    def format_hsl(self): return self._inner.format("formatHsl")
    def hex(self): return self.format_hex()
    def __str__(self): return self._inner.format("toString")

def color(css):
    result=_native._Color.parse(css)
    return None if result is None else ColorValue(result)

def _color_constructor(name):
    names={"rgb":("r","g","b"),"hsl":("h","s","l"),"lab":("l","a","b"),"hcl":("h","c","l"),"lch":("l","c","h"),"cubehelix":("h","s","l"),"gray":("l",)}[name]+("opacity",)
    def create(*args, **kwargs):
        if kwargs:
            if not args and set(kwargs)=={"value"}: args=(kwargs.pop("value"),)
            elif set(kwargs)-set(names): raise TypeError("Unknown color constructor channel.")
            else:
                unset=object();values=list(args)+[unset]*max(0,len(names)-len(args))
                for key,value in kwargs.items():
                    index=names.index(key)
                    if values[index] is not unset: raise TypeError("Color channel supplied twice.")
                    values[index]=value
                while values and values[-1] is unset: values.pop()
                if any(v is unset for v in values): raise TypeError("Missing color constructor channel.")
                args=tuple(values)
        if name!="gray" and len(args)==1:
            if isinstance(args[0],ColorValue): return args[0].convert(name)
            if isinstance(args[0],str): return ColorValue(_native._Color.from_css(args[0],name))
        if any(type(v) not in (float,int) for v in args): raise TypeError("Color channels require numbers.")
        return ColorValue(_native._Color(name,list(args)))
    create.__name__=name
    return create
for _color_name in ("rgb","hsl","lab","gray","hcl","lch","cubehelix"):
    globals()[_color_name]=_color_constructor(_color_name)

class Field(_Owned):
    """Opaque field with its original dataset owner."""

class Column(_Owned):
    """Typed source payload with independent null/display metadata."""
    def nullable(self, value=True):
        if type(value) is not bool: raise TypeError("Nullable requires a boolean.")
        return Column(self._inner.nullable(value))
    def validity(self, values):
        values = list(values)
        if any(type(v) is not bool for v in values): raise TypeError("Validity requires booleans.")
        return Column(self._inner.validity(values))
    def formatted(self, values): return Column(self._inner.formatted(list(values)))
    def unit(self, value): return Column(self._inner.unit(value))
    def label(self, value): return Column(self._inner.label(value))

def column(values, *, kind=None, timezone="UTC"):
    values = list(values)
    present = [v for v in values if v is not None]
    types = {type(v) for v in present}
    if kind is None:
        if not types:
            raise ValueError("Empty/all-null columns require an explicit kind.")
        if types == {bool}: kind = "bool"
        elif types == {str}: kind = "string"
        elif types <= {int}:
            kind = "uint64" if max(present) > 2**63-1 and min(present) >= 0 else "int64"
        elif types <= {int, float}: kind = "float64"
        else: raise TypeError("Mixed source kinds require explicit materialization.")
    allowed = {"float64": {int,float}, "int64": {int}, "uint64": {int}, "bool": {bool}, "string": {str}, "category": {str}, "s": {int}, "ms": {int}, "us": {int}, "ns": {int}}
    if kind not in allowed or not types <= allowed[kind]:
        raise TypeError(f"Values do not match column kind {kind!r}.")
    if kind == "float64" and any(type(v) is int and abs(v) > 2**53-1 for v in present):
        raise ValueError("Wide exact integers cannot be implicitly converted to Float64.")
    return Column(_native._Column(kind, values, timezone).nullable(len(present) != len(values)))

def categorical(values): return column(values, kind="category")
def timestamps(values, unit="ns", timezone="UTC"): return column(values, kind=unit, timezone=timezone)

class Data(_Owned):
    """One normalized immutable dataset, reusable across plots."""
    @staticmethod
    def columns(columns, *, name="data", keys=None, limits=None, schema_version=None, identity=None):
        if not isinstance(columns, _Mapping): raise TypeError("columns requires a mapping of names to columns.")
        b = _native._Columns().name(name)
        if identity is not None:
            if type(identity) is not int: raise TypeError("Dataset identity requires an exact integer.")
            b = b.identity(identity)
        if limits is not None: b = b.limits(_encode(limits))
        if schema_version is not None: b = b.schema_version(schema_version)
        for key, values in columns.items():
            value = values if isinstance(values, Column) else column(values)
            b = b.column(key, value._inner)
        if keys is not None:
            keys = list(keys)
            if any(type(k) is not int for k in keys): raise TypeError("Row keys must be exact integers.")
            b = b.keys(keys)
        return Data(b.build())
    @staticmethod
    def rows(rows, *, fields=None, name="data", keys=None, **options):
        rows = list(rows)
        if fields is None:
            if not rows: raise ValueError("Empty rows require explicit fields.")
            names = list(rows[0])
            if any(set(row) != set(names) for row in rows): raise ValueError("Rows must have identical named fields.")
            columns = {name: [row[name] for row in rows] for name in names}
        else:
            columns = {name: [accessor(row) for row in rows] for name, accessor in fields.items()}
        if callable(keys): keys = [keys(row) for row in rows]
        return Data.columns(columns, name=name, keys=keys, **options)
    def field(self, name): return Field(self._inner.field(name))
    @property
    def name(self): return self._inner.name()

class Component(_Owned):
    """Persistent syntax adapter for a canonical Rust component builder."""
    def __getattr__(self, name):
        if name.startswith("_"): raise AttributeError(name)
        def apply(*args, **kwargs):
            if kwargs:
                if args: raise TypeError("Use positional values or one set of named options.")
                args = (kwargs,)
            if len(args) == 1 and isinstance(args[0], Component):
                inner = self._inner.with_component(name, args[0]._inner)
            elif len(args) == 1 and isinstance(args[0], Field):
                inner = self._inner.field(name, args[0]._inner)
            elif name == "field_parameter" and len(args) == 2 and isinstance(args[1], Field):
                inner = self._inner.field_parameter(args[0],args[1]._inner)
            elif name == "data" and len(args) == 1 and isinstance(args[0], Data):
                inner = self._inner.data(args[0]._inner)
            elif name == "layer" and len(args) == 2 and all(isinstance(v, Component) for v in args):
                inner = self._inner.theme_layer(*(v._inner for v in args))
            elif name == "candle_volume" and len(args) == 2:
                inner = self._inner.candle_volume(*(v._inner for v in args))
            elif name == "axis" and len(args) == 2 and all(isinstance(v, Component) for v in args):
                inner = self._inner.link_axis(*(v._inner for v in args))
            elif name == "symbol_types" and len(args) == 3 and isinstance(args[0], Field):
                inner=self._inner.symbol_types_field(args[0]._inner,_encode(args[1]),_encode(args[2]))
            elif name == "shape_value" and len(args) == 2:
                target, source = args
                if isinstance(source, Field): inner = self._inner.shape_value_field(_encode(target), source._inner)
                elif isinstance(source, Component): inner = self._inner.shape_value_expression(_encode(target), source._inner)
                else: inner = self._inner.set(name, _encode(args))
            elif name in ("numeric_scale", "value_scale") and len(args) == 3:
                target, source, scale = args
                descriptor = scale.mapped() if isinstance(scale, StandaloneScale) else scale
                if isinstance(source, Field): inner = getattr(self._inner, name+"_field")(_encode(target), source._inner, _encode(descriptor))
                elif isinstance(source, Component): inner = getattr(self._inner, name+"_expression")(_encode(target), source._inner, _encode(descriptor))
                else: inner = self._inner.set(name, _encode((target, source, descriptor)))
            else:
                if name in ("geometry", "time_domain"): args = tuple(str(v) if type(v) is int else v for v in args)
                inner = self._inner.set(name, _encode(args))
            return type(self)(inner)
        return apply

class Path(_Owned):
    """Mutable checked standalone path. SVG precision never changes numeric geometry."""
    def __init__(self, digits=None, *, limits=None):
        if digits is not None and type(digits) not in (int, float): raise TypeError("Digits require a number or None.")
        super().__init__(_native._Path(digits, None if limits is None else _encode(limits)))
    @classmethod
    def _wrap(cls, inner):
        result = object.__new__(cls)
        _Owned.__init__(result, inner)
        return result
    @classmethod
    def from_json(cls, request): return cls._wrap(_native._Path.from_json(request))
    def copy(self): return Path._wrap(self._inner.copy())
    def _draw(self, method, values, anticlockwise=False):
        if any(type(v) not in (int,float) for v in values): raise TypeError("Path coordinates require numbers.")
        if type(anticlockwise) is not bool: raise TypeError("Arc direction requires a boolean.")
        self._inner.draw(method, values, anticlockwise)
        return self
    def move_to(self,x,y): return self._draw("moveTo",[x,y])
    def line_to(self,x,y): return self._draw("lineTo",[x,y])
    def quadratic_curve_to(self,cx,cy,x,y): return self._draw("quadraticCurveTo",[cx,cy,x,y])
    def bezier_curve_to(self,cx1,cy1,cx2,cy2,x,y): return self._draw("bezierCurveTo",[cx1,cy1,cx2,cy2,x,y])
    def arc_to(self,x1,y1,x2,y2,r): return self._draw("arcTo",[x1,y1,x2,y2,r])
    def arc(self,x,y,r,a0,a1,anticlockwise=False): return self._draw("arc",[x,y,r,a0,a1],anticlockwise)
    def rect(self,x,y,w,h): return self._draw("rect",[x,y,w,h])
    def close_path(self): return self._draw("closePath",[])
    def apply_batch(self,operations): self._inner.batch(_encode(operations)); return self
    def to_svg(self): return self._inner.to_svg()
    def __str__(self): return self.to_svg()
    def result(self): return _decode(self._inner.result_json())
    def replay(self,sink):
        """Call sink once per owned numeric command; failure retains its accepted prefix."""
        for command in _decode(self._inner.replay_json()): sink(command)

class ShapeRegistry(_Owned):
    """Explicit trusted Rust registrations; constructing a registry installs no code."""
    def __init__(self): super().__init__(_native._ShapeRegistry())
    @classmethod
    def _wrap(cls, inner):
        result=object.__new__(cls); _Owned.__init__(result,inner); return result
    @staticmethod
    def example():
        """Install the external example implementations in a proof-enabled build."""
        if not hasattr(_native._ShapeRegistry, 'example'):
            raise RuntimeError('The extension-proof build feature is required for example registrations.')
        return ShapeRegistry._wrap(_native._ShapeRegistry.example())
    def copy(self): return self._wrap(self._inner.copy())
    def selection(self, selection, family):
        return _decode(self._inner.selection_json(_encode(selection), _encode(family)))

class ShapeLine(_Owned):
    """Reusable checked D3 line controls; generate returns an independent Path."""
    def __init__(self, config=None): super().__init__(_native._ShapeLine(_encode({} if config is None else config)))
    @classmethod
    def _wrap(cls, inner):
        result=object.__new__(cls)
        _Owned.__init__(result,inner)
        return result
    def copy(self): return self._wrap(self._inner.copy())
    def config(self): return _decode(self._inner.config_json())
    def generate(self, rows): return Path._wrap(self._inner.generate(_encode(rows)))
    def generate_registered(self, rows, registry, selection):
        return Path._wrap(self._inner.generate_registered(_encode(rows), registry._inner, _encode(selection)))

class ShapeArea(_Owned):
    """Independent paired boundaries, defined gaps and inherited boundary generators."""
    def __init__(self, config=None): super().__init__(_native._ShapeArea(_encode({} if config is None else config)))
    @classmethod
    def _wrap(cls, inner):
        result=object.__new__(cls)
        _Owned.__init__(result,inner)
        return result
    def copy(self): return self._wrap(self._inner.copy())
    def config(self): return _decode(self._inner.config_json())
    def generate(self, rows): return Path._wrap(self._inner.generate(_encode(rows)))
    def boundary(self, which): return ShapeLine._wrap(self._inner.boundary(_encode(which)))
    def generate_registered(self, rows, registry, selection):
        return Path._wrap(self._inner.generate_registered(_encode(rows), registry._inner, _encode(selection)))

class ShapeLineRadial(ShapeLine):
    """Owned Rust LineRadial generator with materialized selectors."""
    def __init__(self, config=None): _Owned.__init__(self, _native._ShapeLineRadial(_encode({} if config is None else config)))
class ShapeAreaRadial(ShapeLine):
    """Owned Rust AreaRadial generator with materialized selectors."""
    def __init__(self, config=None): _Owned.__init__(self, _native._ShapeAreaRadial(_encode({} if config is None else config)))
    def boundary(self, which): return ShapeLineRadial._wrap(self._inner.boundary(_encode(which)))
class ShapeLink(ShapeLine):
    """Owned Rust Link generator with materialized selectors."""
    def __init__(self, config=None): _Owned.__init__(self, _native._ShapeLink(_encode({} if config is None else config)))
class ShapeLinkRadial(_Owned):
    """Owned Rust radial link with fixed radial-tangent geometry."""
    def __init__(self, config=None): super().__init__(_native._ShapeLinkRadial(_encode({} if config is None else config)))
    @classmethod
    def _wrap(cls, inner):
        result=object.__new__(cls); _Owned.__init__(result,inner); return result
    def copy(self): return self._wrap(self._inner.copy())
    def config(self): return _decode(self._inner.config_json())
    def generate(self, data): return Path._wrap(self._inner.generate(_encode(data)))
def point_radial(angle, radius):
    """Return Rust pointRadial coordinates as an owned pair."""
    return _native._point_radial(angle, radius)

class ShapeSymbol(_Owned):
    """Owned area/stroke-size symbol generator; all geometry runs in Rust."""
    def __init__(self, config=None): super().__init__(_native._ShapeSymbol(_encode({} if config is None else config)))
    @classmethod
    def _wrap(cls, value):
        obj=cls.__new__(cls); _Owned.__init__(obj,value); return obj
    def copy(self): return ShapeSymbol._wrap(self._inner.copy())
    def config(self): return _decode(self._inner.config_json())
    def generate(self): return Path._wrap(self._inner.generate())
    @staticmethod
    def palettes():
        fill, stroke=_decode(_native._ShapeSymbol.palettes_json())
        return tuple(fill), tuple(stroke)

    def generate_registered(self, registry, selection):
        return Path._wrap(self._inner.generate_registered(registry._inner, _encode(selection)))

class ShapeArc(_Owned):
    """Owned checked sector/annulus generator; constants may replace datum fields."""
    def __init__(self, config=None): super().__init__(_native._ShapeArc(_encode({} if config is None else config)))
    @classmethod
    def _wrap(cls, inner):
        result=object.__new__(cls)
        _Owned.__init__(result,inner)
        return result
    def copy(self): return self._wrap(self._inner.copy())
    def config(self): return _decode(self._inner.config_json())
    def generate(self, datum=None): return Path._wrap(self._inner.generate(_encode({} if datum is None else datum)))
    def centroid(self, datum=None): return _decode(self._inner.centroid_json(_encode({} if datum is None else datum)))

class ShapePie(_Owned):
    """Owned pie layout; materialized values preserve original data and input order."""
    def __init__(self, config=None): super().__init__(_native._ShapePie(_encode({} if config is None else config)))
    @classmethod
    def _wrap(cls, inner):
        result=object.__new__(cls)
        _Owned.__init__(result,inner)
        return result
    def copy(self): return self._wrap(self._inner.copy())
    def config(self): return _decode(self._inner.config_json())
    def layout(self, data, values=None): return _decode(self._inner.layout_json(_encode(data),_encode(data if values is None else values)))

    def layout_registered(self, data, values, registry, selection):
        return _decode(self._inner.layout_registered_json(_encode(data), _encode(values), registry._inner, _encode(selection)))

class ShapeStack(_Owned):
    """Owned stack layout; materialized rows follow configured series key order."""
    def __init__(self, config=None): super().__init__(_native._ShapeStack(_encode({} if config is None else config)))
    @classmethod
    def _wrap(cls, inner):
        result=object.__new__(cls)
        _Owned.__init__(result,inner)
        return result
    def copy(self): return self._wrap(self._inner.copy())
    def config(self): return _decode(self._inner.config_json())
    def layout(self, data, values=None):
        from ._interpolation import _un_number
        result=_decode(self._inner.layout_json(_encode(data),_encode(data if values is None else values)))
        for series in result:
            for point in series['points']:
                point['y0']=_un_number(point['y0']);point['y1']=_un_number(point['y1'])
        return result

    def layout_registered(self, data, values, registry, order=None, offset=None):
        from ._interpolation import _un_number
        result=_decode(self._inner.layout_registered_json(_encode(data), _encode(values), registry._inner, _encode(order), _encode(offset)))
        for series in result:
            for point in series['points']:
                point['y0']=_un_number(point['y0']);point['y1']=_un_number(point['y1'])
        return result


def path(): return Path()
def path_round(digits=3): return Path(digits)

class VectorPath(Component):
    def transform(self, matrix, *, max_error=0.01, max_commands=1000000):
        return VectorPath(self._inner.set("transform",_encode([list(matrix),max_error,max_commands])))
def vector_path(id, path): return VectorPath(path._inner.annotation(id))

# Generic registry name; the established shape name retains the same owned identity.
ExtensionRegistry = ShapeRegistry

class PlotBuilder(_Owned):
    def with_registry(self, registry): return self.with_shape_registry(registry)
    def with_shape_registry(self, registry): return type(self)(self._inner.with_shape_registry(registry._inner))
    def __getattr__(self, name):
        if name.startswith("_"): raise AttributeError(name)
        def apply(*args, **kwargs):
            if kwargs:
                if args: raise TypeError("Use positional values or named options.")
                args = (kwargs,)
            if len(args) == 1 and isinstance(args[0], Component): inner = self._inner.with_component(name, args[0]._inner)
            elif name == "data" and len(args) == 1 and isinstance(args[0], Data): inner = self._inner.dataset(args[0]._inner)
            elif name == "layer" and len(args) == 2: inner = self._inner.layer(args[0],args[1]._inner)
            else: inner = self._inner.set(name, _encode(args))
            return type(self)(inner)
        return apply
    def build(self): return Plot(self._inner.build())

class PlotEdit(PlotBuilder): pass

def plot(data): return PlotBuilder(_native._Draft(data._inner))

class Plot(_Owned):
    def edit(self): return PlotEdit(self._inner.edit())
    def chart(self): return Chart(self)
    def to_json(self): return self._inner.to_json()
    @staticmethod
    def from_json(value, registry=None):
        return Plot(_native._Plot.from_json(value) if registry is None else _native._Plot.from_json_with_registry(value,registry._inner))

class ExportOptions(_Owned):
    def __getattr__(self, name):
        if name.startswith("_"): raise AttributeError(name)
        def apply(*args):
            if name == "layout": inner = self._inner.layout(args[0]._inner)
            else: inner = self._inner.set(name, _encode(args))
            return ExportOptions(inner)
        return apply

def export_options(width, height, unit="pt"): return ExportOptions(_native._ExportOptions(width,height,unit))

class FigureRequest(_Owned):
    def with_hierarchy_history(self, previous): return FigureRequest(self._inner.with_hierarchy_history(previous._inner))
    def prepare(self): return FigureSnapshot(self._inner.prepare())
    def manifest(self): return _decode(self._inner.manifest())

class FigureTransition(_Owned):
    def sample(self, fraction): return FigureSnapshot(self._inner.sample(fraction))

class FigureSnapshot(_Owned):
    def guide_transition(self, previous): return FigureTransition(self._inner.guide_transition(previous._inner))
    def presentation(self): return _decode(self._inner.presentation())
    def guides(self): return _decode(self._inner.guides())
    def scene(self): return _decode(self._inner.scene())
    def manifest(self): return _decode(self._inner.manifest())
    def export(self, format): return self._inner.export(format)
    def save(self, path, format): _Path(path).write_bytes(self.export(format))

class Output(_Owned):
    def __init__(self, font): super().__init__(_native._Output(bytes(font)))
    def primary_font(self): return _decode(self._inner.primary_font())
    def register_font(self, font): return _decode(self._inner.register_font(bytes(font)))
    def request(self, source, options):
        if isinstance(source, Chart): return source.request(self, options)
        return FigureRequest(self._inner.request(source._inner, options._inner))
    def export(self, source, format, options): return self.request(source, options).prepare().export(format)
    def save(self, source, path, format, options): _Path(path).write_bytes(self.export(source, format, options))

class ExportJob(_Owned):
    def cancel(self): return self._inner.cancel()
    def run(self): return self._inner.run()

class ExportQueue(_Owned):
    def __init__(self, *, max_jobs=2, max_input_bytes=536870912, max_rows=2000000): super().__init__(_native._ExportQueue(max_jobs,max_input_bytes,max_rows))
    def submit(self, request, format): return ExportJob(self._inner.submit(request._inner,format))
    def metrics(self): return _decode(self._inner.metrics())

class Transaction(_Owned): pass
class Updates(_Owned):
    def id(self, value): return Updates(self._inner.id(value))
    def append(self, target, data): return Updates(self._inner.data("append", _dataset(target), data._inner))
    def upsert(self, target, data): return Updates(self._inner.data("upsert", _dataset(target), data._inner))
    def replace(self, target, data): return Updates(self._inner.data("replace", _dataset(target), data._inner))
    def remove(self, target, keys): return Updates(self._inner.remove(_dataset(target),list(keys)))
    def retain_count(self, target, count): return Updates(self._inner.retain_count(_dataset(target), count))
    def retention(self, target, policy): return Updates(self._inner.retention(_dataset(target),_encode(policy)))
    def retain_event_time(self, target, field, width, watermark, *, allowed_lateness=0, late="Reject"):
        values = (width,watermark,allowed_lateness)
        if any(type(v) is not int for v in values): raise TypeError("Event time requires exact integer ticks.")
        return Updates(self._inner.retain_event_time(_dataset(target),field,_encode(dict(width=str(width),watermark=str(watermark),allowed_lateness=str(allowed_lateness),late=late))))
    def watermark(self, target, ticks): return Updates(self._inner.watermark(_dataset(target),ticks))
    def reset_categories(self, target, field): return Updates(self._inner.reset_categories(_dataset(target),field))
    def build(self): return Transaction(self._inner.build())

def _dataset(value): return value.name if isinstance(value, Data) else value

class Editor(_Owned):
    def original(self): return _decode(self._inner.original())
    def preview(self, dx, dy): return _decode(self._inner.preview(dx,dy))
    def nudge(self, *, horizontal=True, forward=True, steps=1): return _decode(self._inner.nudge(horizontal,forward,steps))

class Chart(_Owned):
    def __init__(self, plot): super().__init__(_native._Runtime(plot._inner))
    def external_view(self):
        view = object.__new__(Chart)
        _Owned.__init__(view,self._inner.external_view())
        return view
    def accept_from(self, source): return _decode(self._inner.accept_from(source._inner))
    def _command(self, name, expected=None, **options): return _decode(self._inner.command(name,_encode(options),expected))
    def _named_query(self, name, options, *, gesture=False, stamp=None): return _decode(self._inner.named_query(name,_encode(options),gesture,None if stamp is None else _encode(stamp)))
    def layer_visible(self, layer, visible, *, expected=None): return self._command("layer_visible",expected,layer=layer,visible=visible)
    def legend_visible(self, visible, *, expected=None): return self._command("legend_visible",expected,visible=visible)
    def follow(self, mode="FollowLatest", *, expected=None): return self._command("follow",expected,mode=mode)
    def freeze(self, *, expected=None): return self.follow("FreezePresentation",expected=expected)
    def resume(self, *, expected=None): return self._command("resume",expected)
    def reset(self, *, expected=None): return self._command("reset",expected)
    def undo(self, *, expected=None): return self._command("undo",expected)
    def redo(self, *, expected=None): return self._command("redo",expected)
    def clear_inspection(self, *, expected=None): return self._command("clear_inspection",expected)
    def select(self, targets, change="Replace", *, expected=None): return self._command("select",expected,targets=targets,change=change)
    def hover(self, targets, *, expected=None): return self._command("hover",expected,targets=targets)
    def focus(self, target=None, *, expected=None): return self._command("focus",expected,target=target)
    def pin(self, target=None, *, expected=None): return self._command("pin",expected,target=target)
    def set_annotation(self, annotation, *, expected=None): return self._command("annotation",expected,annotation=annotation)
    def remove_annotation(self, id, *, expected=None): return self._command("remove_annotation",expected,id=id)
    def set_windows(self, windows, *, expected=None): return self._command("windows",expected,windows=windows)
    def editor(self, options): return Editor(self._inner.editor(options._inner))
    def describe(self, offset=0, limit=64, **fences): return self.query({"Describe":dict(offset=offset,limit=limit)},**fences)
    def inspect(self, x, y, *, radius=10, max_grouped=32, mode="Auto", **fences): return self.query({"Inspect":dict(point=[x,y],radius=radius,max_grouped=max_grouped,mode=mode)},**fences)
    def select_region(self, region, *, limit=4096, **fences): return self.query({"Select":dict(region=region,limit=limit)},**fences)
    def select_series(self, layer, *, panel=None, limit=4096, **fences): return self._named_query("Series",dict(layer=layer,panel=panel,limit=limit),**fences)
    def navigate(self, action, *, axes=("x","y"), panel=None, boundary="ClampToDomain", **fences): return self._named_query("Navigate",dict(axes=list(axes),panel=panel,action=action,boundary=boundary),**fences)
    def zoom(self, x, y, factor, **options): return self.navigate({"Zoom":dict(anchor=[x,y],factor=factor)},**options)
    def pan(self, dx, dy, **options): return self.navigate({"Pan":dict(dx=dx,dy=dy)},**options)
    def range(self, axis, window, *, panel=None, **fences): return self._named_query("SetRange",dict(axis=axis,window=window,panel=panel),**fences)
    def revisions(self): return {k:int(v) for k,v in _decode(self._inner.revisions()).items()}
    def semantics(self): return _decode(self._inner.semantics())
    def state(self): return _decode(self._inner.state())
    def apply_plot(self, plot, expected): return self._inner.apply_plot(plot._inner,expected)
    def restore_state(self, state, expected): self._inner.restore_state(_encode(state),expected)
    def act(self, action, *, origin="Programmatic", expected=None): return _decode(self._inner.act(_encode(action),_encode(origin),expected))
    def query(self, operation, *, gesture=False, stamp=None): return _decode(self._inner.query(_encode(operation),gesture,None if stamp is None else _encode(stamp)))
    def request(self, output, options): return FigureRequest(self._inner.request(output._inner,options._inner))
    def acknowledge_frame(self, frame): self._inner.acknowledge_frame(frame._inner)
    def present(self, output, options): return FigureSnapshot(self._inner.present(output._inner,options._inner))
    def stream(self, options): self._inner.stream(options._inner); return self
    def stream_status(self): return _decode(self._inner.stream_status())
    def pinned(self): return _decode(self._inner.pinned())
    def queue_status(self): return _decode(self._inner.queue_status())
    def commit_next(self): return _decode(self._inner.commit_next())
    def reset_epoch(self): return _decode(self._inner.reset_epoch())
    def transaction(self): return Updates(self._inner.transaction())
    def commit(self, transaction): return _decode(self._inner.commit(transaction._inner))
    def enqueue(self, transaction): return _decode(self._inner.enqueue(transaction._inner))
    def dense(self, frame, options): return _decode(self._inner.dense(frame._inner,options._inner))
    def link_capture(self, component, event): return _decode(self._inner.link_capture(component._inner,_encode(event)))
    def link_resolve(self, component, message): return _decode(self._inner.link_resolve(component._inner,_encode(message)))

class _Expression(Component):
    def __add__(self, other): return self.add(other)
    def __sub__(self, other): return self.sub(other)
    def __mul__(self, other): return self.mul(other)
    def __truediv__(self, other): return self.div(other)
    def __pow__(self, other): return self.pow(other)
    def __neg__(self): return self.negate()

# Each family has its own public type; implementation and validation remain in Rust.
_FAMILIES = {'SourceExpression': 'source_expr', 'StatExpression': 'stat_expr', 'BinExpression': 'bin_expr', 'ScaleExpression': 'after_scale_expr from_theme', 'ScaleAes': 'scale_aes', 'Aes': 'aes', 'Layer': 'blank points line area ribbon hierarchy hierarchy_tree hierarchy_cluster hierarchy_icicle hierarchy_sunburst hierarchy_treemap hierarchy_pack shape_line shape_area shape_line_radial shape_area_radial shape_link shape_link_horizontal shape_link_vertical shape_link_radial shape_arc shape_pie shape_symbol bars volume ohlc rule rectangle cells histogram', 'Stat': 'identity_stat bin count summary fit custom_stat', 'StatAes': 'stat_aes', 'BinAes': 'bin_aes', 'Position': 'stack shape_stack dodge jitter', 'Filter': 'filter', 'Transform': 'transform', 'Scale': 'scale_linear scale_binned scale_reverse scale_sqrt scale_transform scale_log scale_symlog scale_band scale_point scale_utc scale_date scale_duration scale_session', 'Axis': 'x_axis y_axis xlim ylim', 'Guide': 'axis_guide', 'ColorScale': 'color_discrete color_continuous', 'Legend': 'legend', 'Facet': 'facet_wrap facet_grid', 'Style': 'style', 'Theme': 'theme', 'TextStyle': 'text_style', 'TextRun': 'text_run', 'RichText': 'rich_text', 'Title': 'title', 'Subtitle': 'subtitle', 'Caption': 'caption', 'SourceNote': 'source_note', 'Footnote': 'footnote', 'Labels': 'labels', 'Callout': 'callout', 'PanelLetter': 'panel_letter', 'Inset': 'inset', 'NumberFormat': 'number_format', 'LayoutOptions': 'layout_options', 'RenderOptions': 'render_options', 'StreamOptions': 'stream_options', 'AnnotationEdit': 'annotation_edit', 'Link': 'link'}
_FACTORY_TYPES = {}
for _family, _factories in _FAMILIES.items():
    _class = type(_family, (_Expression if _family.endswith("Expression") else Component,), {"__module__": __name__})
    globals()[_family] = _class
    for _name in _factories.split(): _FACTORY_TYPES[_name] = _class

def _factory(name, cls):
    def create(*args):
        if name == "filter" and len(args)==1 and isinstance(args[0],Component):
            return cls(_native._Component("filter", "[0]").with_component("expression", args[0]._inner))
        if name == "source_expr" and len(args)==1 and isinstance(args[0],Field):
            return cls(_native._Component.source_expression(args[0]._inner))
        if name in ("jitter", "custom_stat"): args = tuple(str(v) if type(v) is int else v for v in args)
        return cls(_native._Component(name,_encode(args)))
    create.__name__ = name
    return create
for _name, _class in _FACTORY_TYPES.items():
    if _name != "transform": globals()[_name] = _factory(_name,_class)
def transform(name, stat): return Transform(_native._Component.transform(name,stat._inner))

# Standalone interpolation shares these owned handle/error conventions.
from . import _interpolation as _interpolation_api
for _name in _interpolation_api.__all__:
    globals()[_name] = getattr(_interpolation_api, _name)
from ._scales import ScaleKey, StandaloneScale

def _scale_payload(spec, kind):
    if isinstance(spec, StandaloneScale):
        descriptor = spec.spec()
        if kind not in descriptor: raise TypeError(f"This chart constructor requires a {kind} scale descriptor.")
        return descriptor[kind]
    return spec
def scale_numeric(spec): return Scale(_native._Component("scale_numeric", _encode([_scale_payload(spec, "Numeric")])))
def scale_registered(name, version, parameters): return Scale(_native._Component("scale_registered", _encode([name, version, parameters])))
def scale_calendar(spec): return Scale(_native._Component("scale_calendar", _encode([_scale_payload(spec, "Time")])))
def scale_band_d3(spec): return Scale(_native._Component("scale_band_d3", _encode([spec])))
def scale_point_d3(spec): return Scale(_native._Component("scale_point_d3", _encode([spec])))
def color_mapped(name, scale, training="Authored"):
    if isinstance(scale, StandaloneScale): descriptor = scale.mapped(training)
    else:
        if training != "Authored": raise TypeError("Set training on the supplied mapped descriptor.")
        descriptor = scale
    return ColorScale(_native._Component("color_mapped", _encode([name, descriptor])))

from ._shape import CurveSpec, ShapeCoordinate, ShapeLimits, ShapeLineConfig, ShapeAreaConfig

from ._shape import ArcDatum, ShapeArcConfig, PieAngles, ShapePieConfig, PieSlice

from ._shape import ArcParameters

from ._shape import SymbolKind, ShapeSymbolConfig

from ._shape import ShapeStackConfig, StackOrder, StackOffset, StackMissing, StackLimits, StackPoint, StackSeries

from ._shape import ShapeLineRadialConfig, ShapeAreaRadialConfig, ShapeLinkConfig, ShapeLinkRadialConfig, RadialBoundary, LinkEndpoint, LinkDatum

from ._shape import RadialParameters

from ._shape import ShapeOperationId, ShapeOperation, ShapeFamily

from ._hierarchy import Hierarchy as Hierarchy, pack_siblings as pack_siblings, pack_enclose as pack_enclose
