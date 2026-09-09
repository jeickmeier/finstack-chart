#!/usr/bin/env python3
"""SP-06 executable Python calendar, exact integer and retained chart proof."""
import json,sys,math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(Path(sys.argv[1]).resolve()),str(ROOT/'packages/python')]
import chart_python as native
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
artifacts=Path(sys.argv[3])
corpus=json.loads((ROOT/'fixtures/parity/d3-scale/calendar/cases.json').read_text())
number=lambda x:{'kind':'Number','value':x}
def spec(z,domain):
 r=z['resource'];ts=r['transitions'];zone='Utc' if z['mode']=='Utc' else {'Local':{'version':1,'zone':z['zone'],'revision':'7','tzdata':z['tzdata'],'coverage':{'start':str(r['start']),'end':str(r['end'])},'initial_offset_seconds':ts[0]['offset_seconds'],'transitions':[{'at_millis':str(t['at']),'offset_seconds':t['offset_seconds']} for t in ts[1:]]}}
 return {'domain':list(map(str,domain)),'unit':'Milliseconds','zone':zone,'range':[number(0),number(1)],'factory':{'kind':'Value'},'clamp':False,'unknown':{'kind':'Missing'}}
def every(name,k):return {'unit':{'Week':'Sunday' if name=='Sunday' else 'Monday'} if name in ('Sunday','Monday') else name,'step':k}
# SP-07 public facade replay uses the same independent corpus, without host math.
class PublicTime:
 def __init__(self,spec):self.inner=c.StandaloneScale.from_spec({'Time':json.loads(spec)})
 @classmethod
 def owned(cls,inner):result=object.__new__(cls);result.inner=inner;return result
 @classmethod
 def from_json(cls,wire):return cls.owned(c.StandaloneScale.from_json(wire))
 def to_json(self):return self.inner.to_json()
 def copy(self):return self.owned(self.inner.copy())
 def dispose(self):self.inner.dispose()
 def map_json(self,value):return json.dumps(self.inner.map_value(value))
 def invert(self,value):return self.inner.invert(value)
 def floor(self,value,interval):return self.inner.floor(value,json.loads(interval))
 def ceil(self,value,interval):return self.inner.ceil(value,json.loads(interval))
 def round(self,value,interval):return self.inner.round_time(value,json.loads(interval))
 def offset(self,value,interval,steps):return self.inner.offset(value,json.loads(interval),steps)
 def format(self,value,fmt):return self.inner.format(value,**json.loads(fmt))
 def ticks(self,selection,budget):
  q=json.loads(selection);return list(self.inner.ticks(q.get('Count',10),interval=q.get('Interval'),budget=budget))
 def nice(self,selection):
  q=json.loads(selection);return self.owned(self.inner.nice(q.get('Count',10),interval=q.get('Interval')))
Time=PublicTime if '--public' in sys.argv[4:] else native._TimeScale
def domain(scale):
 spec=json.loads(scale.to_json())['spec'];return spec.get('Time',spec)['domain']
counts={'intervals':0,'formats':0,'automatic':0,'mapping':0}
for z in corpus['zones']:
 s=Time(json.dumps(spec(z,[1704067200000,1704153600000])))
 copied=s.copy();wire=s.to_json();assert Time.from_json(wire).to_json()==wire
 for q in z['intervals']:
  counts['intervals']+=1;interval=json.dumps(every(q['name'],q['every']))
  for method in ('floor','ceil','round'):assert getattr(s,method)(q['value'],interval)==q[method]['value'],(z['zone'],method,q)
  for row in q['offset']:assert s.offset(q['value'],interval,row['step'])==row['result']['value']
 locales={l['id']:l['value'] for l in z['locales']}
 for q in z['formats']:
  counts['formats']+=1;fmt=json.dumps({'pattern':q['pattern'],'locale':locales[q['locale']]})
  assert [s.format(v,fmt) for v in q['values']]==q['result']['value']
 for q in z['automatic']:
  counts['automatic']+=1;a=Time(json.dumps(spec(z,q['domain'])));selection=json.dumps({'Count':q['count']});ticks=a.ticks(selection,10000)
  assert ticks==q['ticks']['value'];assert list(map(int,domain(a.nice(selection))))==q['nice']['value']
  assert [a.format(v,json.dumps({'pattern':None,'locale':locales['en-US']})) for v in ticks]==q['labels']['value'];a.dispose()
 for q in z['mapping']:
  counts['mapping']+=1;d=spec(z,q['domain']);d.update(range=list(map(number,q['range'])),clamp=q['clamp'],factory={'kind':'Round' if q['round'] else 'Value'});a=Time(json.dumps(d))
  for v,e in zip(q['values'],q['outputs']):assert math.isclose(json.loads(a.map_json(v))['value'],e,rel_tol=1e-12,abs_tol=1e-10)
  assert [a.invert(p) for p in q['positions']]==q['inverse'];a.dispose()
 s.dispose();assert copied.to_json()==wire;copied.dispose()
 try:s.to_json()
 except c.ChartError:pass
 else:raise AssertionError('Disposed time scale remained usable')
# Host integers outside the JS safe-number range remain exact on native operations and wire.
d=spec(corpus['zones'][0],[1700000000000000001,1700000000000000011]);d['unit']='Nanoseconds';a=Time(json.dumps(d));assert a.invert(.3)==1700000000000000004;assert json.loads(a.map_json(1700000000000000004))['value']==.3
bad=json.loads(a.to_json());bad['version']=2
try:Time.from_json(json.dumps(bad))
except c.ChartError:pass
else:raise AssertionError('Future scale version accepted')
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
for name in ('spring','fall'):
 p=c.Plot.from_json((artifacts/f'{name}.plot.json').read_text());wire=json.loads(p.to_json());axis=next(a for a in wire['definition']['axes'] if 'Calendar' in a['scale']);assert axis['scale']['Calendar']['spec']['zone']['Local']['revision']=='42'
 request=output.request(p,c.export_options(900.,300.));frame=request.prepare();svg=frame.export('svg');assert svg==(artifacts/f'{name}.svg').read_bytes();p.dispose();assert request.prepare().export('svg')==svg
 (out/f'{name}.svg').write_bytes(svg);frame.dispose();request.dispose()
(out/'checks.json').write_text(json.dumps({'counts':counts,'resource_revision':'7','chart_revision':'42','exact_big_integer':True,'copied_and_disposed':True},indent=2)+'\n')
print('PASS SP-06 Python',counts,'two retained publication fixtures and exact integer lifecycle')
