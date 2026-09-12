import finstack_chart as c
args: c.GuideTickArguments = {'count': 2.5, 'specifier': '.1%'}
format: c.GuideFormatter = {'Registered': {'operation': {'id': 'example.guide_format', 'version': 1}, 'parameters': {'mode': 'Indexed'}}}
a: c.Axis = c.x_axis().guide_profile('D3_3_0_0').tick_arguments(args).tick_values([0., .5, 1.]).tick_format(format)
g: c.Guide = c.axis_guide('top', 'x').guide_profile('D3_3_0_0').tick_values([]).tick_values(None).tick_format(None)
geometry: c.GuideGeometry = {'inner': -40., 'outer': 6., 'padding': -3., 'offset': .5, 'labels': 'Preserve', 'overflow': 'Clip', 'clip_ticks': True}
a = a.guide_geometry(geometry).tick_size(-6.).tick_size_inner(0.).tick_size_outer(6.).tick_padding(-3.).tick_offset(None)
g = g.guide_geometry(geometry).tick_size(-6.).tick_size_inner(0.).tick_size_outer(6.).tick_padding(-3.).tick_offset(None)
options: c.LayoutOptions = c.layout_options().device_scale(2.)
components: c.GuideComponents = {'domain': {'color': '#113355', 'width': 2., 'dashes': [4., 2.]}, 'labels': {'font_size': 18.}, 'per_tick': [{'index': 2, 'line': {'visible': False}}]}
a = a.guide_components(components).guide_components(None)
g = g.guide_components(components).guide_components(None)
