#!/usr/bin/env python3
"""Public request and independent oracle checks for per-side treemap accessors."""
import json,pathlib,sys
from compare import near
ROOT=pathlib.Path(__file__).resolve().parents[2]
cases=json.loads((ROOT/'fixtures/hierarchy/padding-accessors.json').read_text())['cases']
if sys.argv[1]=='requests':
    results=[]
    for c in cases:
        actions=[('construct',c['input'],None),('apply',{'Sum':{'Field':'value'}},None),('apply',{'Layout':c['layout']},None),('query','Nodes','nodes'),('query','Configuration','configuration'),('snapshot',None,'snapshot'),('restore',None,None),('snapshot',None,'roundtrip')]
        results.append({'id':c['id'],'steps':[{'session':'h','action':a,'data':d,**({'out':o} if o else {})} for a,d,o in actions]})
    pathlib.Path(sys.argv[2]).write_text(json.dumps(results,separators=(',',':')))
    print(f'{len(results)} per-side padding request sequences')
else:
    for file in sys.argv[2:]:
        results=json.loads(pathlib.Path(file).read_text());assert len(results)==len(cases)==72
        for c,r in zip(cases,results):
            assert c['id']==r['id'];r=r['result'];nodes=r['nodes']
            actual=[{'key':n['handle']['node'],'parent':n['parent']['node'] if n['parent'] else None,'children':[v['node'] for v in n['children']],'value':n['value'],'geometry':n['geometry']} for n in nodes]
            near(actual,c['expected'],1e-10,1e-12,c['id'])
            assert r['configuration']['layout']['Treemap']['padding_sides']==c['layout']['Treemap']['padding_sides']
            assert r['snapshot']==r['roundtrip']
        print(f'PASS {file}: 72 per-side/depth/field/registered accessor cases, readback and exact snapshot restoration')
