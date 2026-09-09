#!/usr/bin/env python3
"""All pinned FIX-20 operations through the public Python facade and the strict raw transport."""
import json,math,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(Path(sys.argv[1]).resolve()),str(ROOT/'packages/python')]
import chart_python as native
import finstack_chart as c
out=Path(sys.argv[2]);out.parent.mkdir(parents=True,exist_ok=True)
corpus=json.loads((ROOT/'fixtures/parity/d3-scale/operations.json').read_text())
dump=lambda v:json.dumps(v,allow_nan=False,separators=(',',':'))
def n(v):
    if isinstance(v,dict):return {'NaN':math.nan,'Infinity':math.inf,'-Infinity':-math.inf,'-0':-0.0}[v['number']]
    return float(v)
def en(v):
    if math.isnan(v):return {'number':'NaN'}
    if math.isinf(v):return {'number':'Infinity' if v>0 else '-Infinity'}
    if v==0 and math.copysign(1,v)<0:return {'number':'-0'}
    return v
def k(v):
    if v=='Null':return None
    kind,payload=next(iter(v.items()))
    if kind=='Number':return n(payload)
    if kind in ('Integer','Unsigned','Timestamp'):return c.ScaleKey(kind,int(payload))
    return payload
def ek(v):
    if v is c.MISSING:return None
    if v is None:return 'Null'
    if isinstance(v,c.ScaleKey):return {v.kind:str(v.value) if v.kind in ('Integer','Unsigned','Timestamp') else v.value}
    if type(v) is bool:return {'Boolean':v}
    if type(v) is str:return {'Text':v}
    if type(v) is int:return {'Integer':str(v)}
    return {'Number':en(v)}
def v(x):
    kind=x['kind'];p=x.get('value')
    if kind=='Missing':return c.MISSING
    if kind=='Null':return None
    if kind=='Number':return n(p)
    if kind in ('Text','Boolean'):return p
    if kind=='Array':return list(map(v,p))
    if kind=='Record':return {key:v(a) for key,a in p.items()}
    if kind=='Date':return c.date_value(n(p))
    if kind=='Color':return c.ColorValue.from_json(dump({'version':1,'value':p}))
    raise AssertionError(kind)
def ev(x):
    if x is c.MISSING:return {'kind':'Missing'}
    if x is None:return {'kind':'Null'}
    if type(x) is bool:return {'kind':'Boolean','value':x}
    if type(x) is str:return {'kind':'Text','value':x}
    if isinstance(x,(float,int)):return {'kind':'Number','value':en(x)}
    if isinstance(x,c.InterpolationDate):return {'kind':'Date','value':en(x.milliseconds)}
    if isinstance(x,list):return {'kind':'Array','value':list(map(ev,x))}
    if isinstance(x,dict):return {'kind':'Record','value':{key:ev(a) for key,a in x.items()}}
    if isinstance(x,c.ColorValue):return {'kind':'Color','value':x.value()}
    raise AssertionError(type(x))
def inp(x):
    if x=='Missing':return c.MISSING
    kind,p=next(iter(x.items()))
    if kind=='Key':return k(p)
    if kind=='Time':return int(p) if '.' not in p else float(p)
    return n(p)
def einp(x,mode):return {mode:ek(x) if mode=='Key' else str(x) if mode=='Time' else en(x)}
def opts(o):
    r=dict(o)
    if 'domain' in r:r['domain']=list(map(inp,r['domain']))
    if 'range' in r:r['range']=list(map(v,r['range']))
    if 'unknown' in r:r['unknown']=v(r['unknown'])
    if 'interpolator' in r:r['interpolator']=c.Interpolator.from_json(dump({'version':1,'spec':r['interpolator']}))
    return r
def close(a,e,exact=False):
    if type(a) in (int,float) and type(e) in (int,float):return a==e or not exact and abs(a-e)<=1e-12+1e-12*abs(e)
    if isinstance(a,list) and isinstance(e,list):return len(a)==len(e) and all(close(x,y,exact) for x,y in zip(a,e))
    if isinstance(a,dict) and isinstance(e,dict):return a.keys()==e.keys() and all(close(a[k],e[k],exact) for k in a)
    return type(a)==type(e) and a==e
def change(s,query,public):
    if not public:return s.change(dump(query))
    kind,p=next(iter(query.items()))
    if kind=='Configure':return s.configure(**opts(p))
    if kind=='Reconfigure':return s.reconfigure(p)
    if kind=='Train':return s.train(list(map(k,p)))
    if kind=='Nice':return s.nice(n(p))
    if kind=='NiceTime':return s.nice(n(p['Count'])) if 'Count' in p else s.nice(interval=p['Interval'])
    raise AssertionError(kind)
def query(s,q,mode,public):
    if not public:return json.loads(s.query(dump(q)))
    name,p=(q,None) if isinstance(q,str) else next(iter(q.items()))
    if name=='Spec':return s.spec()
    if name=='Domain':return [einp(x,mode) for x in s.domain()]
    if name=='Range':return list(map(ev,s.range()))
    if name=='Map':
        raw=s.map_value(inp(p));assert close(ev(s.map(inp(p))),raw);return raw
    if name=='Invert':
        result=s.invert(None if p is None else n(p));return {'Time':str(result)} if mode=='Time' else {'Value':ev(result)}
    if name=='InvertExtent':
        r=s.invert_extent(v(p));return {'found':r['found'],'lower':ek(r['lower']),'upper':ek(r['upper'])}
    if name=='Ticks':return [en(x) for x in s.ticks(n(p['count']),budget=p['budget'])]
    if name=='TimeTicks':
        sel=p['selection'];r=s.ticks(n(sel['Count']),budget=p['budget']) if 'Count' in sel else s.ticks(interval=sel['Interval'],budget=p['budget']);return [{'Time':str(x)} for x in r]
    if name=='Format':return s.format(n(p['value']),count=n(p['count']),specifier=p['specifier'],locale=p['locale'])
    if name=='TimeFormat':return s.format(int(p['value']),**p['format'])
    if name=='Thresholds':return [None if x is None else en(x) for x in s.thresholds()]
    if name=='Quantiles':return [None if x is None else en(x) for x in s.quantiles(n(p))]
    if name=='Step':return en(s.step())
    if name=='Bandwidth':return en(s.bandwidth())
    raise AssertionError(name)
records=[];failures=[]
for public in (False,True):
    counts={'cases':0,'operations':0,'adaptations':0,'diagnostics':0,'public':public}
    for case in corpus['cases']:
        counts['cases']+=1;scale=None
        try:scale=c.StandaloneScale(case['family'],**opts(case['options'])) if public else native._Scale.create(dump(case['family']),dump(case['options']))
        except (c.ChartError,ValueError,TypeError) as error:
            if 'setup_error' not in case:failures.append((public,case['id'],'setup',str(error)))
            else:counts['diagnostics']+=1
            continue
        if 'setup_error' in case:failures.append((public,case['id'],'setup','expected error'));continue
        copied=scale.copy();assert copied.to_json()==scale.to_json();copied.dispose()
        restored=c.StandaloneScale.from_json(scale.to_json()) if public else native._Scale.from_json(scale.to_json());assert restored.to_json()==scale.to_json();restored.dispose()
        for op in case['operations']:
            counts['operations']+=1;counts['adaptations']+='adaptation' in op
            before=scale.to_json();selected=scale;changed=None;copy=None
            try:
                if op.get('copy'):copy=scale.copy();selected=copy
                if 'change' in op:changed=change(selected,op['change'],public);selected=changed
                actual=query(selected,op['query'],case['mode'],public)
                if 'select' in op:
                    for part in op['select'].split('/')[1:]:actual=actual[part]
                if 'diagnostic' in op or not close(actual,op.get('expected'),op.get('exact',False)):failures.append((public,case['id'],op['name'],actual,op.get('expected',op.get('diagnostic'))))
                if op.get('persist'):scale=changed;changed=None
            except (c.ChartError,ValueError,TypeError) as error:
                allowed=op.get('diagnostic',op.get('allowed_diagnostic',[]));code=getattr(error,'code',None)
                if code in allowed or public and code is None and op.get('diagnostic') and op.get('adaptation'):counts['diagnostics']+=1
                else:failures.append((public,case['id'],op['name'],str(error)))
            finally:
                if changed:changed.dispose()
                if copy:copy.dispose()
            if not op.get('persist'):assert scale.to_json()==before
        scale.dispose()
        try:scale.to_json();raise AssertionError('disposed scale still usable')
        except c.ChartError as error:assert error.code=='CHART_DISPOSED_HANDLE'
    records.append(counts)
assert not failures, json.dumps(failures[:30],default=str,indent=2)+f'\n{len(failures)} failures'
assert all(r['cases']==661 and r['operations']==19562 and r['adaptations']==1112 for r in records)
out.write_text(json.dumps(records,indent=2)+'\n')
print('PASS Python FIX-20 raw/public',records)
