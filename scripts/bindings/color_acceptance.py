"""CLR-05 independent public color ramp authors and retained publication."""
from pathlib import Path
import json,sys
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(720.,420.).dpi(144)
for name,bg,fg,preset in [('light','#ffffff','#202020','Editorial'),('dark','#18212a','#f0f0f0','Editorial'),('grayscale','#ffffff','#202020','Grayscale')]:
 p=(c.plot(c.Data.columns({'x':[i/20. for i in range(21)]}))
    .title(c.title(f'Perceptual color / {name}').style(c.text_style().color(fg)))
    .x_axis(c.x_axis().scale(c.scale_linear().domain(-.05,1.05)))
    .y_axis(c.y_axis().scale(c.scale_linear().domain(.4,3.6)).visible(False))
    .theme(c.theme().preset(preset).style(c.style().background(bg).panel(bg).foreground(fg))))
 for label,factory,y in [('Lab','Lab',3.),('HCL','Hcl',2.),('Cubehelix','Cubehelix',1.)]:
  scale=c.StandaloneScale('linear',range=['rgba(255, 30, 30, 0.5)','rgba(30, 90, 255, 0.5)'],factory={'kind':factory})
  p=(p.layer(c.points().aes(c.aes().x('x').y(y).color('x').color_scale(label)).size(16.))
    .layer(c.labels().id(label).at(.04,y+.32).text(label).style(c.text_style().color(fg)))
    .scale(c.color_mapped(label,scale)).legend(c.legend().scale(label).title(label)))
 p=p.legend(c.legend().scale('HCL').title('HCL / alpha 0.5')).build()
 wire=p.to_json();assert json.loads(wire)['version']==5
 (out/f'{name}.plot.json').write_text(wire)
 request=output.request(p,options);p.dispose();f=request.prepare();scene=f.scene()
 marks=[i['primitive']['Point']['fill'] for i in scene['items'] if 'Point' in i['primitive'] and i.get('layer') is not None]
 assert len(marks)==63 and all(m['alpha']==128 for m in marks)
 if name=='grayscale':assert all(m['red']==m['green']==m['blue'] for m in marks)
 (out/f'{name}.scene.json').write_text(json.dumps(scene,indent=2))
 for fmt in ['svg','pdf','png']:(out/f'{name}.{fmt}').write_bytes(f.export(fmt))
 f.dispose();request.dispose()
print('PASS CLR-05 Python: three independently authored perceptual/alpha/grayscale figures and retained publication.')
