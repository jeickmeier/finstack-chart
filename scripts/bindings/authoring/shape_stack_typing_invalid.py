from finstack_chart import ShapeStack, shape_stack
ShapeStack({'order':'Random'})
ShapeStack({'offset':'Normalize'})
ShapeStack({'missing':'Ignore'})
shape_stack(['a']).stack_order([0])
shape_stack(['a']).stack_offset('Normalized')
shape_stack(['a']).stack_missing(True)
