"""SP-07: exact piecewise/time navigation, brushing, inspection and linked windows."""
from pathlib import Path
import sys,json
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options=c.export_options(600.,300.).layout(c.layout_options().padding(0))
records=[]
for time in [False,True]:
 base=1700000000000000001
 xs=c.timestamps([base,base+100,base+200,base+550,base+1000],'ns','UTC') if time else [0.,5.,10.,55.,100.]
 scale=c.StandaloneScale('utc',domain=[base,base+200,base+1000],range=[0.,50.,100.],unit='Nanoseconds') if time else c.StandaloneScale('linear',domain=[0.,10.,100.],range=[0.,50.,100.])
 axis=c.scale_calendar(scale) if time else c.scale_numeric(scale)
 data=c.Data.columns({'x':xs,'y':[1.,2.,3.,4.,5.]},keys=[9007199254741001+i for i in range(5)])
 p=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).x_axis(c.x_axis().scale(axis).visible(False)).y_axis(c.y_axis().visible(False)).build()
 wire=p.to_json();chart=p.chart();receiver=c.Plot.from_json(wire).chart();p.dispose()
 frame=chart.present(output,options);receiver.present(output,options).dispose()
 before=frame.scene();old=chart.request(output,options.basis('presented'))
 point=[i['primitive']['Point']['center'] for i in before['items'] if 'Point'in i['primitive']][2]
 hit=chart.inspect(point['x'],point['y'],radius=1.,mode='NearestPoint')['targets']
 assert hit[0]['identity']['Source']['key']=='9007199254741003'
 selected=chart.select_region({'Rectangle':[point['x']-.5,point['y']-.5,1.,1.]})['targets'];assert selected==hit
 zoom=chart.zoom(point['x'],point['y'],2.,axes=['x'])['windows']
 expected={'Timestamp':[str(base+100),str(base+600)]} if time else {'Numeric':[5.,55.]}
 assert zoom['0']==expected,(zoom,expected)
 region=chart.navigate({'Region':{'from':[150.,0.],'to':[450.,300.]}},axes=['x'])['windows'];assert region['0']==expected
 event=chart.set_windows(zoom)['event'];assert event
 changed=chart.present(output,options)
 message=chart.query({'LinkCapture':{'origin':'scale-source','event':event,'axes':['0'],'panel':None,'selection':False}})['message']
 resolved=receiver.query({'LinkResolve':{'message':message,'mappings':[{'source':'0','destination':'0'}],'panel':None,'missing':'Reject'}})
 receiver.act(resolved['action'],origin=resolved['origin'])
 assert receiver.state()['interaction']['windows']==chart.state()['interaction']['windows']
 assert chart.inspect(point['x'],point['y'],radius=1.,mode='NearestPoint')['targets']==hit
 fresh=old.prepare();assert fresh.scene()==before
 fresh.dispose();old.dispose();frame.dispose();changed.dispose();chart.dispose();receiver.dispose();data.dispose();scale.dispose()
 records.append({'case':'nanosecond_piecewise' if time else 'numeric_piecewise','windows':expected,'exact_key':hit[0]['identity']['Source']['key'],'brush_and_link':True,'old_export_retained':True})
out.write_text(json.dumps(records,indent=2)+'\n')
print('PASS SP-07 Python piecewise numeric/time navigation, inspection, brushing, linking and retained exports.')
