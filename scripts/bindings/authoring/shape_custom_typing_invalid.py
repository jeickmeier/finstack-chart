"""Negative callable, family and ownership consumers: three independent static errors."""
from finstack_chart import ShapeRegistry, ShapeOperation, ShapeLine
op: ShapeOperation = {'operation': {'id':'example.shift_curve','version':'1'}, 'parameters': lambda: 2}
registry=ShapeRegistry()
registry.selection(op,'UnknownProtocol')
ShapeLine().generate_registered([[0.,1.]],ShapeLine(),op)
