from finstack_chart import ShapeSymbol, ShapeSymbolConfig, SymbolKind, Path, Data, shape_symbol, source_expr, StandaloneScale
config:ShapeSymbolConfig={'kind':'Star','size':64.,'digits':3.}
s=ShapeSymbol(config);p:Path=s.copy().generate();fill:tuple[SymbolKind,...]=ShapeSymbol.palettes()[0]
d=Data.columns({'type':['a','b'],'area':[1.,2.]})
shape_symbol().symbol_types(d.field('type'),['a','b'],['Circle','Plus']).shape_value('AreaSize',source_expr(d.field('area'))*64.).symbol_title('Kind').symbol_size_guide('Area',[1.,2.]).symbol_paint('Auto')
shape_symbol().numeric_scale('AreaSize',d.field('area'),StandaloneScale('linear',domain=[0.,2.],range=[16.,256.])).symbol_size_guide('Area',[0.,1.,2.])
p.dispose();s.dispose()
