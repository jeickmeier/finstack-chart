from finstack_chart import ShapeLine, ShapeArea, ShapeLineConfig, ShapeAreaConfig, CurveSpec, Path, shape_line, shape_area
curve: CurveSpec = {'kind':'CatmullRom','alpha':0.5}
line: ShapeLineConfig = {'curve':curve,'defined':[True,False],'x':{'Column':0},'y':{'Constant':1},'digits':None}
area: ShapeAreaConfig = {'curve':curve,'x0':{'Column':0},'y0':{'Constant':0},'x1':None,'y1':{'Column':1}}
p:Path=ShapeLine(line).generate([[1.,2.],[2.,3.]])
ShapeArea(area).boundary('X1').copy().generate([[1.,2.]])
shape_line().curve(curve)
shape_area().curve({'kind':'Natural'})
p.dispose()
