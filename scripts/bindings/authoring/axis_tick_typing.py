import finstack_chart as c
args: c.GuideTickArguments = {'count': 2.5, 'specifier': '.1%'}
format: c.GuideFormatter = {'Registered': {'operation': {'id': 'example.guide_format', 'version': 1}, 'parameters': {'mode': 'Indexed'}}}
a: c.Axis = c.x_axis().guide_profile('D3_3_0_0').tick_arguments(args).tick_values([0., .5, 1.]).tick_format(format)
g: c.Guide = c.axis_guide('top', 'x').guide_profile('D3_3_0_0').tick_values([]).tick_values(None).tick_format(None)
