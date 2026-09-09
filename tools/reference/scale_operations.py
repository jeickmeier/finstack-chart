#!/usr/bin/env python3
"""Translate pinned D3 observations into typed operation inputs; never execute production code.

Expected values are copied from the pinned oracle. Explicit adaptations identify JS coercion,
function identity, integer timestamps and immutable training at the individual operation.
"""
import hashlib,json,re,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
SOURCE=ROOT/'fixtures/parity/d3-scale/cases.json'
OUT=Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'fixtures/parity/d3-scale/operations.json'
UNDEFINED={'kind':'Undefined'}
def number(v):
    return v.get('date',v) if isinstance(v,dict) else v
def value(v):
    if v==UNDEFINED:return {'kind':'Missing'}
    if v is None:return {'kind':'Null'}
    if type(v) is bool:return {'kind':'Boolean','value':v}
    if type(v) is str:return {'kind':'Text','value':v}
    if isinstance(v,list):return {'kind':'Array','value':list(map(value,v))}
    if isinstance(v,dict) and 'date' in v:return {'kind':'Date','value':v['date']}
    if isinstance(v,dict) and 'number' not in v:return {'kind':'Record','value':{k:value(x) for k,x in v.items()}}
    return {'kind':'Number','value':v}
def key(v):
    if v is None:return 'Null'
    if type(v) is bool:return {'Boolean':v}
    if type(v) is str:return {'Text':v}
    return {'Number':v}
def time(v):return str(number(v))
def input_value(v,mode):
    if mode=='Key':return {'Key':key(v)}
    if v is None:return 'Missing'
    return {mode:time(v) if mode=='Time' else number(v)}
def domain(values,mode):return [input_value(v,mode) for v in values]
def exceptional(v):
    if isinstance(v,dict):return v.get('number') in ('NaN','Infinity','-Infinity') or any(exceptional(x) for x in v.values())
    if isinstance(v,list):return any(exceptional(x) for x in v)
    return False
def tagged_family(name):return re.sub(r'(?<!^)(?=[A-Z])','_',name.removeprefix('scale')).lower().replace('time','local')
def build_case(c):
    family=tagged_family(c['factory']);mode='Key' if family in ('ordinal','band','point','threshold') else 'Time' if family in ('utc','local') else 'Number'
    cfg=c['config'];o={}
    for method,args in cfg.items():
        v=args[0]
        if method=='domain':o['domain']=domain(v,mode)
        elif method in ('range','rangeRound'):
            o['range']=list(map(value,v))
            if method=='rangeRound':o['round']=True
        elif method=='unknown':o['unknown']=value(v)
        elif method=='interpolate':o['factory']={'kind':v['name'].removeprefix('interpolate') or 'Value'}
        elif method=='interpolator':
            assert v['name']=='interpolateRgb.gamma(2)(red,blue)'
            o['interpolator']={'operation':'Between','factory':{'kind':'Rgb','gamma':2},'a':value('red'),'b':value('blue')}
        else:o[{'paddingInner':'padding_inner','paddingOuter':'padding_outer'}.get(method,method)]=v
    if family=='local':o['zone']={'Local':{'version':1,'zone':'UTC','revision':'1','tzdata':'fixed-utc','coverage':{'start':'-8640000000000000','end':'8640000000000000'},'initial_offset_seconds':0,'transitions':[]}}
    out={'id':c['id'],'family':family,'mode':mode,'options':o,'source_config':cfg,'operations':[]}
    if 'error' in c['setup']:
        out['setup_error']=c['setup']['error'];return out
    ops=out['operations']
    def add(name,query,expected=None,**kw):
        exact=name.startswith(('domain','copy/','nice/','ticks/','labels/')) or query=='Spec' or family in ('ordinal','quantile','quantize','threshold') or name.startswith('map/') and o.get('round',False)
        row={'name':name,'query':query,'exact':exact,**kw}
        if 'diagnostic' not in kw:row['expected']=expected
        if exceptional(expected):row['allowed_diagnostic']=['NumericalDomain']
        ops.append(row);return row
    for i,(v,result) in enumerate(zip(c['inputs'],c['output'])):
        x=input_value(v,mode);extra={}
        if mode=='Time' and isinstance(number(v),(int,float)) and number(v)!=int(number(v)):
            add(f'map/{i}',{'Map':x},diagnostic=['Validation'],adaptation='Fractional millisecond primitives are outside the exact integer timestamp API; source-unit precision is tested separately.');continue
        if family.startswith('diverging') and v is None:
            x={'Number':0};extra['adaptation']='D3 null-to-zero coercion is represented as explicit numeric zero; typed missing is tested independently.'
        if family=='threshold' and v is None:x='Missing'
        if family=='ordinal':extra.update(change={'Train':[key(v)]},persist=True,adaptation='Explicit immutable training precedes lookup; resolved map never mutates the catalog.')
        if 'error' in result:add(f'map/{i}',{'Map':x},diagnostic=['NumericalDomain','Validation'],**extra)
        else:add(f'map/{i}',{'Map':x},value(result['value']),**extra)
    getters=c['getters']
    add('domain','Domain',domain(getters['domain']['value'],mode))
    if 'error' in getters['range']:add('range','Range',diagnostic=['NumericalDomain','Validation'])
    else:add('range','Range',list(map(value,getters['range']['value'])))
    kind='Time' if mode=='Time' else 'Ordinal' if family=='ordinal' else 'Band' if family=='band' else 'Point' if family=='point' else 'Threshold' if family=='threshold' else 'Classifier' if family in ('quantile','quantize') else 'Interpolated' if family.startswith(('sequential','diverging')) else 'Continuous' if 'factory' in o else 'Numeric'
    root='/'+kind
    norm='Diverging' if family.startswith('diverging') else 'Quantile' if family=='sequential_quantile' else 'Sequential'
    for name,result in getters.items():
        if name in ('domain','range'):continue
        if name in ('step','bandwidth'):
            add(name,name.title(),result['value']);continue
        if name=='unknown':
            if family=='ordinal':
                expected='Implicit' if 'unknown' not in o else {'Explicit':o['unknown']}
            elif family in ('quantile','quantize','threshold'):expected=o.get('unknown')
            else:expected=o.get('unknown',{'kind':'Missing'})
            add(name,'Spec',expected,select=root+'/unknown',adaptation='Typed unknown policy/value replaces the reference implicit symbol or accidental quantize function getter.');continue
        if name in ('interpolate','interpolator'):
            if name=='interpolator':
                expected={'Interpolate':o['interpolator']} if 'interpolator' in o else None
                if 'range' in o:
                    vals=o['range'];f={'kind':'Value'}
                    expected={'Interpolate':{'operation':'Piecewise','factory':f,'values':vals}} if family.startswith('diverging') else {'Interpolate':{'operation':'Between','factory':f,'a':vals[0],'b':vals[1]}}
                if expected is None:expected='Identity'
                add(name,'Spec',expected,select=root+'/output',adaptation='Serializable built-in interpolation descriptor replaces executable function identity.')
            elif kind=='Numeric':add(name,'Spec',o.get('round',False),select=root+'/round',adaptation='Numeric kernel factory is represented by its round flag; mapping is independently compared.')
            else:add(name,'Spec',o.get('factory',{'kind':'Round' if o.get('round') else 'Value'}),select=root+'/factory',adaptation='Serializable built-in factory descriptor replaces executable function identity.')
            continue
        field={'paddingInner':'padding_inner','paddingOuter':'padding_outer'}.get(name,name)
        if name in ('base','exponent','constant'):
            fam={'base':'Log','exponent':'Pow','constant':'Symlog'}[name]
            pointer=root+(f'/normalization/{norm}' if kind=='Interpolated' else '')+f'/family/{fam}/{name}'
        elif kind in ('Band','Point'):
            if name=='padding':field='padding_inner' if kind=='Band' else 'padding'
            pointer=root+'/spec/'+field
        elif kind=='Interpolated':pointer=root+f'/normalization/{norm}/'+field
        else:pointer=root+'/'+field
        add(name,'Spec',result['value'],select=pointer)
    queries=c['queries']
    for i,q in enumerate(queries.get('invert',[])):
        v=q['input']
        if v is None:
            add(f'invert/{i}',{'Invert':None},diagnostic=['Validation'],adaptation='The typed inverse rejects implicit JavaScript null coercion.');continue
        query={'Invert':number(v)}
        if kind=='Continuous' and any(x.get('kind')!='Number' for x in o.get('range',[])):
            add(f'invert/{i}',query,diagnostic=['UnsupportedCapability'],adaptation='Non-numeric output ranges have no continuous inverse; D3 returns NaN.');continue
        expected=q.get('value')
        if 'error' in q or mode=='Time' and exceptional(expected):add(f'invert/{i}',query,diagnostic=['NumericalDomain','Validation'])
        else:add(f'invert/{i}',query,{'Time':time(expected)} if mode=='Time' else {'Value':value(expected)})
    for method in ('ticks','labels','nice'):
        for j,q in enumerate(queries.get(method,[])):
            n=q['count'];selection={'Count':n}
            if method=='ticks':query={'TimeTicks':{'selection':selection,'budget':10000}} if mode=='Time' else {'Ticks':{'count':n,'budget':10000}}
            elif method=='nice':query='Domain'
            else:query=None
            change={'NiceTime':selection} if mode=='Time' else {'Nice':n}
            if 'error' in q:
                # A label observation evaluates ticks before formatting; reproduce that route.
                if method=='labels':query={'Ticks':{'count':n,'budget':10000}}
                add(f'{method}/{j}',query,diagnostic=['NumericalDomain','Validation','ResourceLimit'],**({'change':change} if method=='nice' else {}));continue
            if method=='labels':
                for k,(v,label) in enumerate(q['value']):
                    query={'TimeFormat':{'value':time(v),'format':{}}} if mode=='Time' else {'Format':{'value':number(v),'count':n,'specifier':None,'locale':{}}}
                    add(f'labels/{j}/{k}',query,label)
            else:
                expected=domain(q['value'],mode) if method=='nice' or mode=='Time' else q['value']
                add(f'{method}/{j}',query,expected,**({'change':change} if method=='nice' else {}))
    for j,q in enumerate(queries.get('invertExtent',[])):
        e=q['value'];found=q['input'] in getters['range']['value']
        expected={'found':found,'lower':None if e[0]==UNDEFINED else key(e[0]),'upper':None if e[1]==UNDEFINED else key(e[1])}
        add(f'invertExtent/{j}',{'InvertExtent':value(q['input'])},expected,adaptation='Typed found flag distinguishes absent range membership from unbounded interval endpoints.')
    for method in ('thresholds','quantiles'):
        if method not in queries:continue
        rows=queries[method]
        if isinstance(rows,list):
            for j,q in enumerate(rows):add(f'{method}/{j}',{'Quantiles':q['count']},[None if v==UNDEFINED else v for v in q['value']])
        else:add(method,'Thresholds',[None if v==UNDEFINED else v for v in rows['value']])
    add('copy/reconfigure','Domain',domain(c['copy']['copyDomain'],mode),change={'Configure':{'domain':[]}},copy=True)
    add('copy/original','Domain',domain(c['copy']['after'],mode))
    return out

def main():
    source=json.loads(SOURCE.read_text());cases=list(map(build_case,source['cases']))
    for i,q in enumerate(source['format_cases']):
        specifier=None if q['specifier']==UNDEFINED else q['specifier']
        inputs=[q['start'],(q['start']+q['stop'])/2,q['stop'],{'number':'-0'}]
        operations=[]
        for j,v in enumerate(inputs):
            op={'name':f'format/{j}','query':{'Format':{'value':v,'count':q['count'],'specifier':specifier,'locale':{}}}}
            if 'error' in q['result']:op['diagnostic']=['NumericalDomain','Validation']
            else:op['expected']=q['result']['value'][j]
            operations.append(op)
        cases.append({'id':f'helper-tickFormat/{i}','family':'linear','mode':'Number','options':{'domain':[{'Number':q['start']},{'Number':q['stop']}]},'operations':operations})
    for c in cases:
        for op in c['operations']:
            for field in ('diagnostic','allowed_diagnostic'):
                if field in op:op[field]=['CHART_'+re.sub(r'(?<!^)(?=[A-Z])','_',code).upper() for code in op[field]]
    OUT.parent.mkdir(parents=True,exist_ok=True)
    result={'version':1,'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),'generator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'cases':cases}
    OUT.write_text(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
    print('PASS translated',len(cases),'cases',sum(len(c['operations']) for c in cases),'operations',sum('adaptation' in o for c in cases for o in c['operations']),'explicit adaptations')
if __name__=='__main__':main()
