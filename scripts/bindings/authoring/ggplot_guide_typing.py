"""GG-05 public builder type surface."""
import finstack_chart as c

layer: c.Layer = c.points().legend({'show': True, 'key_glyph': 'Point'})
key: c.Legend = c.legend().aesthetic('Shape').options({'reverse': True, 'ncol': 2})
custom: c.Legend = c.legend().custom({'id': '999', 'bounds': [0, 0, 40, 20], 'paths': []})
axis: c.Axis = c.x_axis().ggplot_axis({'n_dodge': 2, 'check_overlap': True})
guide = c.axis_guide('outer', 'x').ggplot_axis({'logticks': {'expanded': False}})
