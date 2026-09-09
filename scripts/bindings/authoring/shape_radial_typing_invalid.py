from finstack_chart import ShapeLineRadial, ShapeAreaRadial, ShapeLink, ShapeLinkRadial, point_radial
ShapeLineRadial({'x': {'Column': 0}})
ShapeAreaRadial().boundary('X1')
ShapeLinkRadial({'curve': {'kind':'BumpX'}})
ShapeLink().generate([[1,2],[3,4]])
ShapeLink({'source': 'Node'})
point_radial('zero', 10)
