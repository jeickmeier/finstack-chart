"""FIX-GG06 independently authored position controls through actual Python runtime."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for i in range(8):
 d=c.Data.columns({'x':[1.,1.,2.],'y':[2.,3.,-1.],'g':['a','b','a'],'lo':[.6,.8,1.6],'hi':[1.4,1.2,2.4],'panel':['one','one','two']})
 positions=[c.ggplot_stack().vjust(.5),c.ggplot_fill().reverse(True),c.ggplot_dodge().width(.8).preserve('Single'),c.dodge2().width(.8).padding(.2),c.nudge(.2,-.1),c.jitter_dodge(42).displacement(.2,.1),c.ggplot_stack().reverse(True),c.dodge2().width(.8).reverse(True).preserve('Single')]
 interval=i in [1,3,6,7]
 mapping=c.aes().x(.6).x2(1.4).y('y').y2(0.).group('g') if i in [1,6] else c.aes().x('lo').x2('hi').y('y').y2(0.).group('g') if interval else c.aes().x('x').y('y').group('g')
 builder=c.plot(d).profile('Ggplot2_4_0_3').aes(mapping).layer((c.rectangle() if interval else c.points()).position(positions[i]))
 if i==7:builder=builder.facet(c.facet_wrap('panel'))
 p=builder.build()
 wire=p.to_json(); assert json.loads(wire)['version']>=72
 q=c.Plot.from_json(wire);request=output.request(q,c.export_options(480.,320.));frame=request.prepare()
 (out/f'position-{i}.scene.json').write_text(json.dumps(frame.scene()))
 for fmt in ['svg','pdf','png']:(out/f'position-{i}.{fmt}').write_bytes(frame.export(fmt))
 frame.dispose();request.dispose();q.dispose();p.dispose();d.dispose()
output.dispose();print('PASS Python GG06 positions: 8 authors, roundtrips, 24 publications.')
