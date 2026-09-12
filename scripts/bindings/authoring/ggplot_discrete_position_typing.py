"""Typed nullable positional scale policy and reset."""
import finstack_chart as c
c.x_axis().discrete_policy({'limits':['Null', {'Text':'NA'}], 'na_translate':True})
c.y_axis().discrete_policy({'levels':[{'Text':'a'}, 'Null'], 'drop':False}).discrete_policy(None)
c.x_axis().discrete_policy({'palette':[2.,4., {'number':'NaN'}]})
