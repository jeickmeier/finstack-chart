from finstack_chart import ShapeStack, ShapeStackConfig, StackSeries, shape_stack, shape_area, bars
config:ShapeStackConfig={'keys':['a','b'],'order':{'Explicit':[1,0]},'offset':'Wiggle','missing':'Gap'}
s=ShapeStack(config);copy=s.copy();values:list[StackSeries]=s.layout([[1.,2.],[3.,4.]])
copy.layout([{'id':9007199254741001}],[[1.,2.]])
shape_area().position(shape_stack(['a','b']).stack_order('InsideOut').stack_offset('Expand').stack_missing('Zero'))
bars().position(shape_stack([0,1]).stack_order({'Explicit':[1,0]}))
s.dispose();copy.dispose()
