"""FIX-S09: actual projected path budgets across panels/insets; exact boundary acceptance."""
from pathlib import Path
import sys,math
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for radial in [False,True]:
 for faceted in [False,True]:
  for inset in [False,True]:
   labels=['A','B']if faceted else['A'];count=2 if radial else 1;d=c.Data.columns({'a':[i*math.pi/2 for label in labels for i in range(count)],'facet':c.categorical([label for label in labels for i in range(count)])})
   layer=c.shape_line_radial().shape_value('Angle',d.field('a')).shape_value('Radius',10.)if radial else c.shape_arc().shape_value('EndAngle',math.pi/2)
   draft=c.plot(d).layer(layer)
   if faceted:draft=draft.facet(c.facet_wrap('facet'))
   if inset:
    view=c.inset().id('repeat').rectangle(.6,.1,.3,.3).layer(layer).guides(False)
    if faceted:view=view.panel({'values':[{'Text':'A'}]})
    draft=draft.inset(view)
   p=draft.build();required=(4 if radial else 5)*(len(labels)+int(inset))
   for maximum in [required-1,required]:
    request=output.request(p,c.export_options(400.,200.).dpi(72).layout(c.layout_options().max_vertices(maximum)))
    try:f=request.prepare()
    except c.ChartError as e:assert maximum<required and e.code=='CHART_RESOURCE_LIMIT',(radial,faceted,inset,maximum,e)
    else:assert maximum==required;f.dispose()
    request.dispose()
   p.dispose();d.dispose()
print('PASS Python projected shape work: exact single/facet/inset limits for atomic arcs and radial runs, with no budget reset per panel.')
