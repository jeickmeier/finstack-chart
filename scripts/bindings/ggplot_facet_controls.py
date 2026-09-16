"""GG12 independent actual Python facet authors and publication replay."""
import json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
levels=lambda values:[{'Text':v} for v in values]
for mode in range(17):
    data=c.Data.columns({'x':[1.,2.,1.,4.,1.,10.,2.],'y':[1.,3.,2.,8.,10.,100.,4.],'r':['B','B','A','A','B','B',None],'nested':['u','u','v','v','w','w','u'],'c':['L','L','R','R','R','R','L'],'z':['z1','z2','z1','z2','z1','z2','z1']})
    b=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points())
    policy={};facet=c.facet_wrap('r').fields(['r','c']).columns(2);annotation=None
    if mode==1:policy.update(drop=False,levels=[levels(['B','A','unused']),levels(['R','L','unused'])])
    elif mode==2:facet=c.facet_grid('r','c').fields(['r','nested','c','z']);policy['row_fields']=2
    elif mode==3:facet=c.facet_grid('r','c').fields(['r','nested','c']);policy.update(row_fields=2,margins=[0,1,2])
    elif mode==4:policy['shrink']=False;facet=c.facet_wrap('c').free_y(True);b=b.layer(c.points().stat(c.summary().x(1.).y('y').summary_helper({'MeanSe':{'mult':1.}})).color('#DC2828'))
    elif mode==5:facet=c.facet_grid('r','c').free_x(True).free_y(True);policy['space']='Free'
    elif mode==6:policy['direction']='Tr'
    elif mode==7:policy['direction']='Bl'
    elif mode in range(8,12):policy['strip_position']=['Top','Bottom','Left','Right'][mode-8]
    elif mode==12:policy.update(axes='All',axis_labels='Margins')
    elif mode==13:policy['labeller']={'variable_names':True,'wrap_width':12,'lookup':{'r':{'B':'Business group B'}}}
    elif mode==14:
        facet=c.facet_grid('r','c');annotation=c.Data.columns({'r':['C'],'y':[9.],'x':[9.]},name='annotation');b=b.layer(c.points().data(annotation).color('#DC2828'))
    elif mode==15:facet=c.facet_grid('r','c');policy.update(as_table=False,switch='Both')
    elif mode==16:facet=c.facet_grid('r','c').fields(['c']).free_x(True);policy.update(row_fields=0,space='FreeX')
    p=b.facet(facet.reference(policy)).build();wire=p.to_json();assert json.loads(wire)['version']>=76
    q=c.Plot.from_json(wire);request=output.request(q,c.export_options(800.,560.).dpi(144).layout(c.layout_options().minimum_plot([0.1,0.1])));frame=request.prepare();scene=frame.scene()
    direct_request=output.request(p,c.export_options(800.,560.).dpi(144).layout(c.layout_options().minimum_plot([0.1,0.1])));direct=direct_request.prepare();assert direct.scene()==scene;direct.dispose();direct_request.dispose()
    (out/f'facet-{mode}.plot.json').write_text(wire);(out/f'facet-{mode}.scene.json').write_text(json.dumps(scene))
    for fmt in ['svg','pdf','png']:(out/f'facet-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();q.dispose();p.dispose();data.dispose()
    if annotation is not None:annotation.dispose()
output.dispose();print('PASS Python GG12 facets: seventeen authors, original/replay equality, 51 publications.')
