import json
from pathlib import Path
x=json.load(open('fixtures/parity/ggplot2/theme-hierarchy-controls.json'))
def val(v):
 k=v['kind']; c=v.get('class',[]); c=[c] if isinstance(c,str) else c
 if k=='null': return None
 if k=='unit':
  values=v['values'];units=v['units'];values=[values] if not isinstance(values,list) else values;units=[units]*len(values) if isinstance(units,str) else units
  a=[{'value':n,'unit':u} for n,u in zip(values,units)]
  return {'Margin' if any('margin' in z for z in c) else 'Unit':a}
 z=v.get('values')
 if z is None:return 'Missing'
 if 'rel'in c:return {'Relative':z}
 if isinstance(z,list):return {'Vector':[val(dict(kind=k,values=i,**{'class':c})) or 'Missing' for i in z]}
 return {'Bool' if isinstance(z,bool) else 'Text' if isinstance(z,str) else 'Number':z}
def entry(v):
 if v['kind']=='null':return None
 if v['kind']=='element':
  kind=v['class'][0].split('_')[-1].title()
  if kind=='Blank':return 'Blank'
  return {'Element':{'kind':kind,'properties':{k:val(z) for k,z in v['properties'].items() if val(z)is not None}}}
 return {'Value':val(v)}
tree={n:{'classes':t['class'] if isinstance(t['class'],list) else [t['class']],'parents':[] if isinstance(t['parents'],dict) else [t['parents']] if isinstance(t['parents'],str) else t['parents']} for n,t in x['element_tree'].items()}
presets={n:{k:entry(v) for k,v in p['default']['authored']['elements'].items() if entry(v)is not None}for n,p in x['presets'].items()}
Path('crates/chart-core/src/theme/reference_data.json').write_text(json.dumps({'tree':tree,'presets':presets},separators=(',',':'))+'\n')
# Distinct oracle: resolved outputs come from calc_element, not this runtime's resolver.
cases=[]
for n,p in x['presets'].items():
 for variant,v in p.items():
  cases.append({'name':n+'-'+variant,'preset':n,'variant':variant,'theme':{'complete':True,'elements':{k:entry(z) for k,z in v['authored']['elements'].items() if entry(z)is not None}},'expected':{k:entry(z['value']) if 'value'in z else None for k,z in v['resolved'].items()}})
for c in x['inheritance']:
 cases.append({'name':c['name'],'base':{'complete':c['base']['complete'],'elements':{k:entry(z)for k,z in c['base']['elements'].items()if entry(z)is not None}},'patch':{'complete':c['patch']['complete'],'elements':{k:entry(z)for k,z in c['patch']['elements'].items()if entry(z)is not None}},'skip_blank':c['skip_blank'],'expected':{k:entry(z['value'])if'value'in z else None for k,z in c['resolved'].items()},'errors':[k for k,z in c['resolved'].items()if'error'in z]})
Path('fixtures/parity/ggplot2/theme-resolution-vectors.json').write_text(json.dumps(cases,separators=(',',':'))+'\n')
def theme(v):
 return {'complete':v['complete'],'elements':{k:entry(z)for k,z in v['elements'].items()if entry(z)is not None}}
contexts={k:theme(v) for k,v in x['contexts'].items()if isinstance(v,dict)and v.get('kind')=='theme'}
subthemes={n.removeprefix('theme_sub_'):{'authored':{k:entry(v)for k,v in c['authored'].items()},'expected':theme(c['expanded']['value'])}for n,c in x['subthemes'].items()}
Path('fixtures/parity/ggplot2/theme-context-vectors.json').write_text(json.dumps({'contexts':contexts,'subthemes':subthemes},separators=(',',':'))+'\n')
