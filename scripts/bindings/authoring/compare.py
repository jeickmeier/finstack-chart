"""Compare primary authors, retaining source identity relationships and independent expected values."""
import json
import math
import re
import struct
import sys
from pathlib import Path

root=Path(sys.argv[1])
hosts=('native','python','wasm')
load=lambda host,name:json.loads((root/host/(name+'.json')).read_text())

def identities(value):
    result={}
    if 'datasets' in value:
        for i,d in enumerate(value['datasets']): result[d['version']['dataset']]=f'dataset_{i}'
        layers=value['layers'] or (value.get('panels',[{}])[0].get('layers',[]) if value.get('panels') else [])
        for i,l in enumerate(layers):result[l['id']]=f'layer_{i}'
        for i,t in enumerate(value.get('transforms',[])):result[t['id']]=f'transform_{i}'
        for i,l in enumerate(layers):
            if l.get('color_legend'):result[l['color_legend']['id']]=f'color_{i}'
    if 'data' in value:
        for i,d in enumerate(value['data']):result[d['id']]=f'dataset_{i}'
        for i,l in enumerate(value['definition']['layers']):result[l['id']]=f'layer_{i}'
        if 'epoch' in value:result[value['epoch']]='epoch'
    def epochs(v):
        if isinstance(v,dict):
            for k,x in v.items():
                if k in ('epoch','source_epoch') and isinstance(x,str):result[x]='epoch'
                else:epochs(x)
        elif isinstance(v,list):
            for x in v:epochs(x)
    epochs(value)
    return result

def normalize(v,ids):
    if isinstance(v,dict):return {ids.get(k,k):normalize(x,ids) for k,x in v.items()}
    if isinstance(v,list):return [normalize(x,ids) for x in v]
    if isinstance(v,str):
        if v in ids:return ids[v]
        # Generated group labels explicitly embed the owning layer's opaque ID.
        return re.sub(r'(layer|transform):(\d+)',lambda m:m[1]+':'+ids.get(m[2],m[2]),v)
    return v

def same(a,b,tolerance,location='root'):
    if isinstance(a,dict):
        assert isinstance(b,dict) and a.keys()==b.keys(),(location,a.keys(),b.keys())
        for k in a:same(a[k],b[k],tolerance,location+'/'+k)
    elif isinstance(a,list):
        assert isinstance(b,list) and len(a)==len(b),(location,len(a),len(b))
        for i,(x,y) in enumerate(zip(a,b)):same(x,y,tolerance,f'{location}/{i}')
    elif isinstance(a,(int,float)) and not isinstance(a,bool):
        assert isinstance(b,(int,float)) and math.isfinite(a) and math.isfinite(b) and abs(a-b)<=tolerance,(location,a,b)
    else:assert type(a)==type(b) and a==b,(location,a,b)

for host in hosts:
    initial=load(host,'initial');final=load(host,'final')
    assert initial['layers'][0]['domains']['x']=={'minimum':1,'maximum':3}
    assert initial['layers'][0]['domains']['y']=={'minimum':2,'maximum':4}
    assert final['layers'][0]['domains']['x']=={'minimum':1,'maximum':4}
    assert final['layers'][0]['domains']['y']=={'minimum':2,'maximum':8}
    assert final['definition_revision']=='1' and final['store_revision']=='1'
    assert [key for b in final['datasets'][0]['chunks'] for key in b['keys']]==['1001','1002','1003','1004']
    assert 'Applied' in load(host,'transaction') and 'AlreadyApplied' in load(host,'replay')
    # Validate source/layer references before opaque-identity normalization.
    dataset=initial['datasets'][0]['version']['dataset']
    for layer in initial['layers']:
        for target in layer['targets']:
            assert target['Source']['dataset']==dataset
            assert target['Source']['key'] in ('1001','1002','1003')
    exact=load(host,'exact')['data'][0]['batch']
    assert exact['keys']==['18446744073709551614','18446744073709551615']
    assert exact['columns'][0]['values']['Timestamp']==['9223372036854775806','9223372036854775807']
    assert exact['columns'][1]['values']['UInt64']==exact['keys']
    assert exact['columns'][2]['values']['Int64']==['-9223372036854775808','9223372036854775807']
    assert exact['columns'][3]['validity']==[True,False] and exact['fields'][3]['nullable']
    assert exact['columns'][4]['values']['Categorical']=={'codes':[0,1],'dictionary':['b','a']}
    stats=load(host,'statistics')['layers']
    summary=stats[0]['rows']['Statistical'][0];fit=stats[1]['rows']['Statistical'][0]
    values=lambda row:{v['field']:v['value'] for v in row['values'] if isinstance(v['field'],str)}
    assert summary['count']=='4' and values(summary)['Mean']==5 and values(summary)['Sum']==20
    assert values(fit)['Slope']==2 and values(fit)['Intercept']==0 and fit['members']==['11','12','13','14']
    custom=load(host,'extension')
    rows=custom['transforms'][0]['rows']['Statistical']
    assert [r['members'] for r in rows]==[['201','202'],['203','204']]
    assert [r['count'] for r in rows]==['2','2']
    for row in rows:
        assert next(v['value'] for v in row['values'] if v['field']=={'Custom':'density'})==0.5
    assert len(custom['layers'])==2 and len(custom['transforms'])==1
    png=(root/host/'primary.png').read_bytes();assert struct.unpack_from('>II',png,16)==(533,347)

for name in ('initial','scene','action','zoom','transaction','replay','final','exact','statistics','extension','extension-scene'):
    values=[]
    for host in hosts:
        value=load(host,name)
        ids=identities(value if name in ('exact','statistics','extension') else load(host,'extension' if name=='extension-scene' else 'initial'))
        ids.update(identities(value))
        values.append(normalize(value,ids))
    for other in values[1:]:same(values[0],other,1e-9 if name in ('scene','zoom','extension-scene') else 1e-12,name)
    print('PASS three primary authors:',name)
# The host-only capture matrices preserve corresponding data/state/policy relationships.
a,b=(load(host,'matrix') for host in ('python','wasm'))
for left,right in zip(a,b):
    for key in ('basis','view','selection'):assert left[key]==right[key]
    same(normalize(left['manifest'],identities(load('python','initial'))),normalize(right['manifest'],identities(load('wasm','initial'))),1e-9,'capture manifest')
print('PASS independent domains, means, OLS, exact timestamp/integer/null/category values, source identity relationships, updates, captures and image sizes')

# All 34 cases are independently authored in each host. Rust additionally compares
# complete semantics and resolved geometry against the pre-existing fixture definitions.
family_values=[load(host,'families') for host in hosts]
assert all(len(v)==34 for v in family_values)
for case in family_values[0]:
    ids=[identities(v[case]) for v in family_values]
    for name in ('families','family-scenes'):
        values=[normalize(load(host,name)[case],mapping) for host,mapping in zip(hosts,ids)]
        for other in values[1:]:same(values[0],other,1e-9 if name=='family-scenes' else 1e-12,case+'/'+name)
print('PASS 34 primary statistic/position/geometry/scale/facet/composition families: full semantic and scene parity')

# Independent input/action expectations already run in each host; compare every state/result.
for name, count in (('runtime-actions',23),('runtime-input',47)):
    values=[load(host,name) for host in hosts]
    assert all(len(value)==count for value in values)
    for value in values[1:]: same(values[0],value,1e-12,name)
    print('PASS primary runtime replay:',name,count)

# Complete retained-stream fixture through typed primary transactions in every host.
import statistics as statistics_oracle
stream_case=json.loads((Path(__file__).resolve().parents[3]/'fixtures/streaming/replay.json').read_text())
traces=[load(host,'runtime-stream') for host in hosts]
for trace in traces:
    assert len(trace)==70
    for item,step in zip(trace,stream_case['steps']):
        assert item['name']==step['name']
        semantic=item['semantics'];assert semantic['store_revision']==step['revision']
        assert semantic['datasets'][0]['retention']==step['retention']
        rows=[]
        for chunk in semantic['datasets'][0]['chunks']:
            rows.extend(map(list,zip(chunk['keys'],chunk['columns'][0]['values']['Timestamp'],chunk['columns'][1]['values']['Float64'])))
        assert rows==step['rows'],step['name']
        edges=[0,10,20,30,50]
        for i,b in enumerate(semantic['transforms'][0]['rows']['Binned']):
            members=sorted([k for k,t,y in rows if edges[i]<=y and (y<edges[i+1] or (i==3 and y==50))],key=int)
            assert b['count']==str(len(members)) and b['target']['Aggregate']['members']==members
            assert b['target']['Aggregate']['input']['revision']==step['revision']
        summaries=semantic['transforms'][1]['rows']['Statistical'];assert len(summaries)==1
        summary=summaries[0];values=[r[2] for r in rows]
        assert summary['count']==str(len(rows)) and summary['members']==sorted([r[0] for r in rows],key=int)
        actual={str(v['field']):v['value'] for v in summary['values']}
        if values:
            expected={'Min':min(values),'Max':max(values),'Sum':math.fsum(values),'Mean':statistics_oracle.mean(values),"{'Quantile': 0}":min(values),"{'Quantile': 1}":statistics_oracle.median(values),"{'Quantile': 2}":max(values)}
            for field,value in expected.items():assert abs(actual[field]-value)<=1e-12,(step['name'],field)
        else:assert all(v is None for v in actual.values())
for other in traces[1:]:
    for a,b in zip(traces[0],other):
        same(a['result'],b['result'],1e-12,a['name']+'/receipt')
        same(a['state'],b['state'],1e-12,a['name']+'/state')
        same(normalize(a['semantics'],identities(a['semantics'])),normalize(b['semantics'],identities(b['semantics'])),1e-12,a['name']+'/semantics')
print('PASS primary three-host 70-step streaming receipts, retained rows, exact bins, independent summaries and historical pins')
