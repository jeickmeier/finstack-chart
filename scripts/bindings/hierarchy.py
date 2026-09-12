#!/usr/bin/env python3
"""Execute every public hierarchy request through the actual Python extension."""
import json,pathlib,sys
if len(sys.argv)==4:sys.path[:0]=[str(pathlib.Path(__file__).resolve().parents[2]/"packages/python"),str(pathlib.Path(sys.argv[3]).resolve())]
from finstack_chart import Hierarchy,ShapeRegistry,ChartError,pack_siblings,pack_enclose
registry=ShapeRegistry.example()
def run(steps):
    sessions={};out={}
    try:
        for step in steps:
            name=step['session'];data=step.get('data');action=step['action'];result=None
            if action=='construct':sessions[name]=Hierarchy(data,registry)
            elif action=='apply':sessions[name].apply(data)
            elif action=='query':result=sessions[name].query(data)
            elif action=='tile':result=sessions[name].tile(data)
            elif action=='packing':result=(pack_siblings if data['siblings'] else pack_enclose)(data['circles'],data.get('limits'))
            elif action=='snapshot':result=json.loads(sessions[name].to_json())
            elif action=='restore':
                old=sessions[name];sessions[name]=Hierarchy.from_json(old.to_json(),registry);old.dispose()
            elif action=='clone':sessions[step['target']]=sessions[name].copy()
            elif action=='copy_subtree':sessions[step['target']]=sessions[name].copy_subtree(data['node'],data['identity'])
            else:raise AssertionError(action)
            if 'out' in step:out[step['out']]=result
        return out
    except ChartError as e:return {'error':e.message,'code':e.code}
    finally:
        for session in sessions.values():session.dispose()
try:
    cases=json.loads(pathlib.Path(sys.argv[1]).read_text())
    output=[{'id':c['id'],'result':run(c['steps'])} for c in cases]
    pathlib.Path(sys.argv[2]).write_text(json.dumps(output,separators=(',',':')))
    print(f'{len(output)} Python hierarchy sequences executed')
finally:registry.dispose()
