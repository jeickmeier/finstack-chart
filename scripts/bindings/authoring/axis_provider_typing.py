from typing import assert_type
from finstack_chart import ExtensionRegistry, ShapeRegistry, Scale, PlotBuilder, scale_registered, plot, Data, x_axis
registry = ExtensionRegistry.example()
assert_type(registry.copy(), ShapeRegistry)
assert_type(scale_registered('example.fold', 1, {'limit': 10.}), Scale)
assert_type(plot(Data.columns({'x': [1.]})).with_registry(registry), PlotBuilder)
x_axis().coordinate_scale(scale_registered('example.fold', 1, {'limit': 10.}))
