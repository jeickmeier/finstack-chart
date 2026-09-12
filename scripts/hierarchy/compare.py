#!/usr/bin/env python3
"""FIX-H01 standalone oracle comparison, with fixed family-specific tolerances."""
import json,math,pathlib,sys
ROOT=pathlib.Path(__file__).resolve().parents[2]
reference=json.loads((ROOT/'fixtures/hierarchy/reference.json').read_text())['cases']
def near(a,b,absolute,relative,path=''):
    if isinstance(a,(float,int)) and not isinstance(a,bool) and isinstance(b,(float,int)) and not isinstance(b,bool):
        assert math.isfinite(a) and math.isfinite(b) and abs(a-b)<=absolute+relative*abs(b),(path,a,b)
    elif isinstance(a,list):
        assert isinstance(b,list) and len(a)==len(b),(path,len(a),len(b) if isinstance(b,list) else b)
        for i,(x,y) in enumerate(zip(a,b)):near(x,y,absolute,relative,f'{path}[{i}]')
    elif isinstance(a,dict):
        assert isinstance(b,dict) and a.keys()==b.keys(),(path,a.keys(),b.keys() if isinstance(b,dict) else b)
        for k in a:near(a[k],b[k],absolute,relative,f'{path}.{k}')
    else:assert a==b,(path,a,b)
def records(nodes):
    lookup={n['handle']['node']:n for n in nodes}
    def name(h):
        if h is None:return None
        n=lookup[h['node']];return n['data'].get('name',n['id']) if isinstance(n['data'],dict) else n['id']
    result=[]
    for n in nodes:
        r={'name':n['data'].get('name') if isinstance(n['data'],dict) else None,'parent':name(n['parent']),'children':[name(c) for c in n['children']],'depth':n['depth'],'height':n['height']}
        if n['id'] is not None:r['id']=n['id']
        if n['value'] is not None:r['value']=n['value']
        if n['geometry'] is not None:r.update(next(iter(n['geometry'].values())))
        result.append(r)
    return result
def exceptional(v):return isinstance(v,dict) and ('number' in v or any(exceptional(x) for x in v.values())) or isinstance(v,list) and any(exceptional(x) for x in v)
def check(c,result):
    expected=c['expected'];op=c['op'];tol=(1e-8,1e-10) if op.startswith('pack') else (1e-10,1e-12)
    if isinstance(expected,dict) and 'error' in expected:
        assert 'error' in result,(c['id'],result)
        if op=='stratify':assert result['error']==expected['error']
        assert result['code']=='CHART_VALIDATION'
        return
    assert 'error' not in result,(c['id'],result)
    if 'snapshot'in result:near(result['snapshot'],result['roundtrip'],0,0,'roundtrip')
    if op in ('packSiblings','packEnclose'):near(result['result'],expected,*tol,c['id']);return
    if op in ('history','historyTopology'):
        for i,state in enumerate(expected):near(records(result[f'step{i}']),state,*tol,f'{c["id"]}/{i}')
        assert all(n['geometry'] is None and n['value'] is None for n in result['original'])
        return
    if op=='operations':
        lookup={n['handle']['node']:n['data']['name'] for n in result['sum']}
        names=lambda hs:[lookup[h['node']] for h in hs]
        actual={k:records(result[k]) for k in ('sum','count','copy','sort')}
        for k in ('before','after','each'):actual[k]=[[n['data']['name'],i,h==result['sum'][0]['handle']] for n,i,h in result[k]]
        for k in ('iterator','ancestors','leaves','path','selfPath'):actual[k]=names(result[k])
        actual['links']=[names(pair) for pair in result['links']]
        actual['find']=lookup[result['find']['node']];actual['noMatch']=result['noMatch'] is None
        # Pointer sharing is checked in native topology tests; host records must be owned.
        assert result['copy'][0]['data']==result['sum'][1]['data']
        near(actual,{k:v for k,v in expected.items() if k!='copyShared'},*tol,c['id']);return
    if c['id']=='ordered-grouped':
        nodes=result['nodes'];actual=[]
        for n in nodes:
            data=n['data'];children=len(n['children'])
            if n['synthetic']:assert data is None;data=[None,'Map']
            elif children:data=[data[0],'Map']
            actual.append({'data':data,'children':children,'depth':n['depth'],'height':n['height']})
        near(actual,expected,0,0,c['id']);return
    actual=records(result['nodes'])
    if op=='tile':
        rectangles={h['node']:r for h,r in result['tiles']}
        for raw,record in zip(result['nodes'],actual):
            if raw['handle']['node'] in rectangles:record.update(zip(('x0','y0','x1','y1'),rectangles[raw['handle']['node']]))
    if op=='pack' and exceptional(expected):
        size=c['config'].get('size',[1,1]);assert not c['config'].get('radius')
        for n in actual:assert [n['x'],n['y'],n['r']]==[size[0]/2,size[1]/2,0]
        near([{k:v for k,v in n.items() if k not in ('x','y','r')} for n in actual],[{k:v for k,v in n.items() if k not in ('x','y','r')} for n in expected],*tol,c['id']);return
    near(actual,expected,*tol,c['id'])
def main():
    baseline=None
    for name in sys.argv[1:]:
        rows=json.loads(pathlib.Path(name).read_text());assert len(rows)==len(reference)==826
        for c,row in zip(reference,rows):assert row['id']==c['id'];check(c,row['result'])
        if baseline is not None:
            for c,a,b in zip(reference,baseline,rows):near(a,b,*( (1e-8,1e-10) if c['op'].startswith('pack') else (1e-10,1e-12) ),c['id'])
        else:baseline=rows
        print(f'PASS: {name}: all 826 oracle cases, error profiles and exact owned snapshot roundtrips')
if __name__=='__main__':main()
