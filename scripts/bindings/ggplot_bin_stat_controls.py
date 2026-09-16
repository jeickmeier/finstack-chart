"""GG06 source-backed bin controls via actual primary Python authoring."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
cases=json.loads((ROOT/'fixtures/parity/ggplot2/bin-stat-controls.json').read_text())['cases'];records=[]
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for i,t in enumerate(cases):
 owned=[]
 try:
  data=c.Data.columns({'x':t['input'],'w':t['weight']});owned.append(data)
  mode=t['mode'];opts={'closed':t['closed'].title(),'pad':t['pad']}
  if mode not in ('explicit','bins'):opts['binwidth']=.75
  if mode in ('center','boundary'):opts[mode]=.25
  stat=c.bin().bins(3).ggplot_bin(opts).bin_weight('w')
  if mode=='explicit':stat=stat.breaks([0.,1.,2.,4.])
  p=c.plot(data).profile('Ggplot2_4_0_3').aes(c.aes().x('x')).layer(c.histogram().stat(stat)).build();owned.append(p)
  wire=p.to_json();assert json.loads(wire)['version']==72
  q=c.Plot.from_json(wire);owned.append(q);assert q.to_json()==wire
  for label,current in [('original',p),('replay',q)]:
   chart=current.chart();owned.append(chart);rows=chart.semantics()['layers'][0]['rows']['Binned'];assert len(rows)==len(t['result'])
   values=[]
   for row,expected in zip(rows,t['result']):
    actual={'xmin':row['start'],'xmax':row['end'],**row['statistics']}
    for key,value in actual.items():
     if value is None:assert expected[key] is None,(t,key)
     else:assert math.isclose(value,expected[key],rel_tol=3e-12,abs_tol=3e-12),(t,key,value,expected[key])
    values.append(actual)
   records.append({'case':i,'state':label,'rows':values})
  selected=t['closed']=='right' and ((t['population']=='ordinary' and not t['pad']) or (mode=='explicit' and t['population'] in ('signed','zero') and t['pad'])) or t['closed']=='left' and t['population']=='boundary' and mode=='explicit' and t['pad']
  if selected:
   options=c.export_options(480.,320.);owned.append(options);request=output.request(q,options);owned.append(request);frame=request.prepare();owned.append(frame)
   for fmt in ('svg','pdf','png'):(out/f'bin-{i}.{fmt}').write_bytes(frame.export(fmt))
 finally:
  for item in reversed(owned):item.dispose()
output.dispose();assert len(records)==200;assert len(list(out.glob('*.svg')))==8
(out/'records.json').write_text(json.dumps(records,allow_nan=False));print('PASS GG06 Python bins: 200 states, 24 publications.')
