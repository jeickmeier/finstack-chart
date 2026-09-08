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
    return _json.dumps(value, allow_nan=False, separators=(",", ":"))

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
            else:
                if name in ("geometry", "time_domain"): args = tuple(str(v) if type(v) is int else v for v in args)
                inner = self._inner.set(name, _encode(args))
            return type(self)(inner)
        return apply

class PlotBuilder(_Owned):
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
    def from_json(value): return Plot(_native._Plot.from_json(value))

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
    def prepare(self): return FigureSnapshot(self._inner.prepare())
    def manifest(self): return _decode(self._inner.manifest())

class FigureSnapshot(_Owned):
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

# Each family has its own public type; implementation and validation remain in Rust.
_FAMILIES = {'Aes': 'aes', 'Layer': 'points line area ribbon bars volume ohlc rule rectangle cells histogram', 'Stat': 'identity_stat bin count summary fit custom_stat', 'StatAes': 'stat_aes', 'BinAes': 'bin_aes', 'Position': 'stack dodge jitter', 'Filter': 'filter', 'Transform': 'transform', 'Scale': 'scale_linear scale_log scale_symlog scale_band scale_point scale_utc scale_session', 'Axis': 'x_axis y_axis', 'ColorScale': 'color_discrete color_continuous', 'Legend': 'legend', 'Facet': 'facet_wrap facet_grid', 'Style': 'style', 'Theme': 'theme', 'TextStyle': 'text_style', 'TextRun': 'text_run', 'RichText': 'rich_text', 'Title': 'title', 'Subtitle': 'subtitle', 'Caption': 'caption', 'SourceNote': 'source_note', 'Footnote': 'footnote', 'Labels': 'labels', 'Callout': 'callout', 'PanelLetter': 'panel_letter', 'Inset': 'inset', 'NumberFormat': 'number_format', 'LayoutOptions': 'layout_options', 'RenderOptions': 'render_options', 'StreamOptions': 'stream_options', 'AnnotationEdit': 'annotation_edit', 'Link': 'link'}
_FACTORY_TYPES = {}
for _family, _factories in _FAMILIES.items():
    _class = type(_family, (Component,), {"__module__": __name__})
    globals()[_family] = _class
    for _name in _factories.split(): _FACTORY_TYPES[_name] = _class

def _factory(name, cls):
    def create(*args):
        if name in ("jitter", "custom_stat"): args = tuple(str(v) if type(v) is int else v for v in args)
        return cls(_native._Component(name,_encode(args)))
    create.__name__ = name
    return create
for _name, _class in _FACTORY_TYPES.items():
    if _name != "transform": globals()[_name] = _factory(_name,_class)
def transform(name, stat): return Transform(_native._Component.transform(name,stat._inner))
