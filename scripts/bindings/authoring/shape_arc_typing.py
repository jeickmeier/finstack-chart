from finstack_chart import ShapeArc, ShapePie, ArcDatum, ShapeArcConfig, ShapePieConfig, Path, Data, shape_arc, shape_pie, source_expr
config:ShapeArcConfig={'inner_radius':0.,'outer_radius':30.,'start_angle':0.,'end_angle':1.,'corner_radius':2.,'pad_radius':None}
arc=ShapeArc(config)
p:Path=arc.generate()
datum:ArcDatum={'inner_radius':5.,'outer_radius':20.,'start_angle':0.,'end_angle':1.}
arc.centroid(datum)
pie_config:ShapePieConfig={'order':'Input','angles':{'pad_angle':.1}}
ShapePie(pie_config).layout([1.,2.])
ShapePie().layout([{'id':'9007199254741001'}],[1.])
data=Data.columns({'weight':[1.,2.]})
shape_pie().shape_value('PieValue',data.field('weight')).pie_order('ValuesDescending').pie_angles({'end_angle':6.})
shape_arc().shape_value('OuterRadius',source_expr(data.field('weight'))*2.)
p.dispose();arc.dispose()
