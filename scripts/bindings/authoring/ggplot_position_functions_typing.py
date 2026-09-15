"""FIX-GG04 registered primary positional limits."""
import finstack_chart as c
c.x_axis().limits_function({'operation': {'id': 'example.numeric_limits', 'version': '1'}, 'parameters': {'mode': 'identity'}})
c.y_axis().scale(c.scale_sqrt()).limits_function({'operation': {'id': 'example.numeric_limits', 'version': '1'}, 'parameters': {'mode': 'single'}})
c.x_axis().scale(c.scale_binned({'transform': 'Sqrt'})).limits_function({'operation': {'id': 'example.numeric_limits', 'version': '1'}, 'parameters': {'mode': 'fixed'}})
c.x_axis().missing_value(5.)
c.y_axis().scale(c.scale_sqrt()).missing_value(-1.)
c.x_axis().missing_value(None)
c.x_axis().missing_value({'number': 'NaN'})

c.x_axis().numeric_limits([None, 5.])
c.y_axis().numeric_limits(({'number': '-Infinity'}, {'number': 'Infinity'}))
c.x_axis().numeric_limits(None)
