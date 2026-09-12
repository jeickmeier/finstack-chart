#!/usr/bin/env python3
"""Exercise immutable configuration set/readback/reset equivalents from the pinned inventory."""
import copy,json,pathlib,sys
from requests import reg
ROOT=pathlib.Path(__file__).resolve().parents[2]
inventory=json.loads((ROOT/'fixtures/hierarchy/inventory.json').read_text())
source={'version':1,'identity':'1','input':{'Node':{'key':'1','data':{'value':1}}}}
def default(f):return {f.capitalize():{} if f=='partition' else {'options':{},**({'history':False} if f=='treemap' else {})}}
def configured(f,m):
 s=default(f);v=s[f.capitalize()];o=v if f=='partition' else v['options']
 if m=='size':o['mode' if f in ('tree','cluster') else 'size']={'Extent':[7,9]} if f in ('tree','cluster') else [7,9]
 elif m=='nodeSize':o['mode']={'NodeSize':[7,9]}
 elif m=='separation':v['separation']=reg('DepthSeparation')
 elif m=='round':o['round']=True
 elif m=='radius':o['radius']='Explicit';v['radius']={'Constant':4}
 elif m=='tile':o['tile']='Custom';v['tiler']=reg('EqualTile')
 elif m=='padding' and f!='treemap':o['padding']=4
 elif m=='padding':v['padding']={'Constant':4}
 else:
  side=m.removeprefix('padding');v['padding_sides']={k:{'Constant':4} for k in (['Top','Right','Bottom','Left'] if side=='Outer' else [side])}
 return s
cases=[]
for f,factory in inventory['factories'].items():
 if f=='stratify':continue # Native immutable StratifyOptions, input descriptors and construction fixtures own this adaptation.
 for m in factory['methods']:
  steps=[{'action':'construct','data':source},{'action':'apply','data':{'Sum':{'Field':'value'}}}]
  for label,layout in [('default',default(f)),('set',configured(f,m)),('reset',default(f))]:
   steps.extend([{'action':'apply','data':{'Layout':layout}},{'action':'query','data':'Configuration','out':label}])
  cases.append({'id':f+'.'+m,'steps':[dict(session='h',**s) for s in steps]})
for f in ['Squarify','Resquarify']:
 steps=[{'action':'construct','data':source},{'action':'apply','data':{'Sum':{'Field':'value'}}}]
 for i,r in enumerate([1.618033988749895,3,0,1.618033988749895]):
  steps.extend([{'action':'apply','data':{'Layout':{'Treemap':{'options':{'tile':{f:r}},'history':f=='Resquarify'}}}},{'action':'query','data':'Configuration','out':str(i)}])
 cases.append({'id':f+'.ratio','steps':[dict(session='h',**s) for s in steps]})
# Constructor configuration is an immutable descriptor owned by the caller.
# Consuming it produces topology; changing/restoring selectors creates a new topology.
stratify_rows=[{'key':'1','data':{'id':'r','parentId':None,'custom_id':'r','custom_parent':None,'path':'/r'}},{'key':'2','data':{'id':'a','parentId':'r','custom_id':'A','custom_parent':'r','path':'/r/a'}},{'key':'3','data':{'id':'b','parentId':'a','custom_id':'B','custom_parent':'r','path':'/r/b'}}]
for method,options in [('id',{'id_field':'custom_id','parent_field':'custom_parent'}),('parentId',{'parent_field':'custom_parent'}),('path',{'id_field':None,'parent_field':None,'path_field':'path'})]:
 steps=[]
 for label,config in [('default',{}),('set',options),('reset',{})]:
  steps.extend([{'action':'construct','data':{'version':1,'identity':'1','input':{'Stratify':{'rows':stratify_rows,'options':config}}}},{'action':'query','data':'Nodes','out':label}])
 cases.append({'id':'stratify.'+method,'steps':[dict(session='h',**s) for s in steps]})
def subset(expected,actual):
 if isinstance(expected,dict):
  for k,v in expected.items():subset(v,actual[k])
 else:assert expected==actual,(expected,actual)
if sys.argv[1]=='requests':
 pathlib.Path(sys.argv[2]).write_text(json.dumps(cases));print(len(cases),'set/readback/reset sequences')
else:
 for file in sys.argv[2:]:
  rows=json.loads(pathlib.Path(file).read_text());assert len(rows)==len(cases)
  for c,r in zip(cases,rows):
   assert c['id']==r['id'];v=r['result'];assert 'error' not in v,(c['id'],v)
   if '.ratio' in c['id']:
    assert [v[str(i)]['effective_ratio'] for i in range(4)]==[1.618033988749895,3,1,1.618033988749895];continue
   if c['id'].startswith('stratify.'):
    assert v['default']==v['reset']
    for label in ['default','set','reset']:
     assert len(v[label])==3 and v[label][0]['height']==(1 if label=='set' else 2)
     assert v[label][2]['parent']=={'hierarchy':'1','node':'1' if label=='set' else '2'}
    assert [n['id'] for n in v['set']]==({'stratify.id':['r','A','B'],'stratify.parentId':['r','a','b'],'stratify.path':['/r','/r/a','/r/b']}[c['id']])
    continue
   f,m=c['id'].split('.');assert v['default']['layout']==v['reset']['layout'];subset(configured(f,m),v['set']['layout'])
   d=v['default']['layout'][f.capitalize()];o=d if f=='partition' else d['options'];expected=inventory['factories'][f]['defaults']
   assert (o['mode']=={'Extent':expected['size']} if f in ('tree','cluster') else o['size']==expected['size'])
   if f in ('tree','cluster'):assert o['separation']=='Default' and d['separation'] is None
   if 'round' in expected:assert o['round']==expected['round']
   if f=='pack':assert o['radius']=='Fitted' and d['radius'] is None and o['padding']==0
   if f=='partition':assert o['padding']==expected['padding']
   if f=='treemap':
    assert o['tile']=={'Squarify':1.618033988749895} and d['padding'] is None and d['tiler'] is None
    assert all(o['padding_'+side]==0 for side in ['inner','top','right','bottom','left'])
  print('PASS',file,len(cases),'immutable setter/getter/reset equivalents, pinned defaults and clamped ratios')
