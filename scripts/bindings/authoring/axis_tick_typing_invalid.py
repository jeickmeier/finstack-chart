import finstack_chart as c
c.x_axis().guide_profile('D3')
c.x_axis().tick_arguments({'count': 'five'})
c.axis_guide('top', 'x').tick_format({'Labels': [3]})
