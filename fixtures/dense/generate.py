"""Independent deterministic dense preview inputs; no renderer-derived expected prices."""
import copy,json
from pathlib import Path
root=Path(__file__).resolve().parents[2]
base=json.loads((root/'fixtures/streaming/replay.json').read_text())
origin=9007199254743001
cases=[]
def build(name,rows,geom,mappings,options):
 chart=copy.deepcopy(base['chart']);definition=chart['definition'];definition['transforms']=[]
 layer=definition['layers'][0];layer['geom']=geom;layer['mappings']={'Source':mappings}
 fields=[{'id':str(i+1),'name':n,'kind':'Float64','nullable':True,'unit':None,'label':None} for i,n in enumerate(['x','open','high','low','close','volume'])]
 data={'version':1,'epoch':'1','datasets':[{'id':'1','batch':{'schema_version':'1','fields':fields,'keys':[str(origin+i) for i in range(len(rows))],'columns':[{'values':{'Float64':[r[i] if r[i] is not None else 0 for r in rows]},'validity':[r[i] is not None for r in rows]} for i in range(6)]}}]}
 cases.append({'name':name,'chart':chart,'data':data,'request':{'version':1,'options':options},'rows':rows})
def aes(y='2',**kw):return dict(x={'Field':'1'},y={'Field':y},**kw)
options={'line_bucket_width':8.,'candle_bucket_width':12.,'candle_volume':{'1':'6'},'max_columns':4096}
rows=[[i,None if i in (200,400,600,800) else (90 if i==333 else i%17),None,None,None,None] for i in range(1000)]
build('gapped-lines',rows,{'Line':{'order':'X','connect_gaps':False}},aes(),dict(options,candle_volume={}))
rows=[]
for i in range(256):
 o=100+i%13;c=o+(i%5-2);rows.append([i,o,max(o,c)+3+i%3,min(o,c)-2-i%2,c,None if i%7==0 else i%19+1])
build('supplied-candles',rows,{'Ohlc':{'width':5.}},aes(y2={'Field':'5'},low={'Field':'4'},high={'Field':'3'}),options)
cases[-1]['chart']['definition']['layers'][0]['candle_colors']={'up':{'red':20,'green':140,'blue':80,'alpha':255},'down':{'red':190,'green':50,'blue':50,'alpha':255}}
gray=copy.deepcopy(cases[-1]);gray['name']='grayscale-candles';gray['chart']['definition']['theme']={'version':1,'named':'Grayscale','plot':{'stroke_width':2.5,'dashes':[3.,2.]},'layers':{}};cases.append(gray)
(root/'fixtures/dense/cases.json').write_text(json.dumps(cases,indent=2)+'\n')
