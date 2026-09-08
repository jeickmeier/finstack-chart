"""Primary components for the repository's compiled example extension.

The explicit registry helper requires a native extension built with extension-proof.
It installs already compiled Rust implementations and never evaluates supplied code.
"""
from . import Plot, _native, PlotBuilder, custom_stat, rectangle, stat_aes

def with_extensions(builder):
    return PlotBuilder(builder._inner.with_example_extensions())

def density_histogram(field, edges):
    return custom_stat('example.density_histogram',1,{'edges':list(edges)}).field_parameter('input',field)

def chamfered_bars(native=False):
    operation = 'example.native_bars' if native else 'example.chamfered_bars'
    return rectangle().geometry(operation,1,None).after_stat(stat_aes().x({'Custom':'left'}).y(0.).x2({'Custom':'right'}).y2({'Custom':'density'}))

def load_plot(value):
    return Plot(_native._Plot.from_json_with_example_extensions(value))
