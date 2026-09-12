"""FIX-GG04 checked category-index limit authoring controls."""
import finstack_chart as c
c.x_axis().continuous_limits([0., {'number':'Infinity'}])
c.y_axis().continuous_limits([True, False]).continuous_limits(None)
c.x_axis().continuous_limits([]).continuous_limits([2.]).continuous_limits([-2., 1., 4.])
