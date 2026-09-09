from finstack_chart import (ShapeLineRadial, ShapeAreaRadial, ShapeLink, ShapeLinkRadial, ShapeLineRadialConfig, ShapeAreaRadialConfig, ShapeLinkConfig, ShapeLinkRadialConfig, LinkDatum, point_radial, Path)
line_config: ShapeLineRadialConfig = {'angle': {'Column': 2}, 'radius': {'Constant': 4}, 'defined': [True,False], 'curve': {'kind': 'Cardinal', 'tension': .2}, 'digits': None}
area_config: ShapeAreaRadialConfig = {'start_angle': {'Column':0}, 'end_angle':None, 'inner_radius':{'Constant':2}, 'outer_radius': {'Column':1}}
link_config: ShapeLinkConfig = {'source': 'Target', 'target': {'Constant':[1,2]}, 'curve':{'kind':'BumpY'}}
radial_config: ShapeLinkRadialConfig = {'angle':{'Column':1}, 'radius':{'Column':0}, 'digits':12}
datum: LinkDatum = {'source':[0,10], 'target':[1,30]}
pair: tuple[float,float] = point_radial(1,20)
line = ShapeLineRadial(line_config)
area = ShapeAreaRadial(area_config)
paths: list[Path] = [line.generate([[0,1,2],[2,3,4]]), area.boundary('OuterRadius').generate([[0,1],[2,3]]), ShapeLink(link_config).generate(datum), ShapeLinkRadial(radial_config).generate(datum)]
line.copy().config()
from finstack_chart import (shape_line_radial, shape_area_radial, shape_link, shape_link_horizontal, shape_link_vertical, shape_link_radial, Layer, RadialParameters)
parameters: RadialParameters = {'start_angle':0., 'end_angle':1., 'inner_radius':4., 'outer_radius':20.}
layers: list[Layer] = [shape_line_radial().radial_parameters(parameters).shape_value('Angle',1.).shape_value('Radius',20.), shape_area_radial().curve({'kind':'Basis'}), shape_link({'kind':'BumpX'}), shape_link_horizontal(), shape_link_vertical(), shape_link_radial().radial_parameters(parameters)]
