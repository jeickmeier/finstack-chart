"""Positive custom protocol facade consumers; constructors retain nominal ownership."""
from finstack_chart import (ShapeRegistry, ShapeOperation, ShapeLine, ShapeArea, ShapeSymbol, ShapePie, ShapeStack, ShapeLineRadial, ShapeAreaRadial, ShapeLink, Data, plot, aes, shape_line, Plot)
op: ShapeOperation = {'operation': {'id':'example.shift_curve','version':'1'}, 'parameters': {'amount':2}}
symbol: ShapeOperation = {'operation': {'id':'example.rectangle_symbol','version':'1'}, 'parameters': {'amount':4}}
compare: ShapeOperation = {'operation': {'id':'example.field_comparator','version':'1'}, 'parameters': {'field':'rank'}}
order: ShapeOperation = {'operation': {'id':'example.first_value_order','version':'1'}, 'parameters': {}}
offset: ShapeOperation = {'operation': {'id':'example.shift_offset','version':'1'}, 'parameters': {'amount':2}}
registry=ShapeRegistry.example()
registry.selection(op,'Curve')
ShapeLine().generate_registered([[0.,1.]],registry,op)
ShapeArea().generate_registered([[0.,1.]],registry,op)
ShapeLineRadial().generate_registered([[0.,1.]],registry,op)
ShapeAreaRadial().generate_registered([[0.,1.]],registry,op)
ShapeLink().generate_registered({'source':[0.,1.],'target':[1.,2.]},registry,op)
ShapeSymbol().generate_registered(registry,symbol)
ShapePie().layout_registered([{'rank':'9007199254740993'}],[1.],registry,compare)
ShapeStack({'keys':['a']}).layout_registered([{'id':'x'}],[[1.]],registry,order,offset)
p=plot(Data.columns({'x':[0.],'y':[1.]})).with_shape_registry(registry).aes(aes().x('x').y('y')).layer(shape_line().shape_protocol('Curve',op)).build()
Plot.from_json(p.to_json(),registry)
