#!/usr/bin/env python3
"""Deterministic WP-18 replay and independent retained-row oracle (no Rust outputs read).
Run from the repository root. Exact integer event ticks and keys are decimal strings.
"""
from pathlib import Path
import copy,json,random
ROOT=Path(__file__).resolve().parents[2]
base=copy.deepcopy(json.loads((ROOT/'fixtures/interaction/cases.json').read_text())[0])
chart=base['chart']; data=base['data']; origin=9007199254741001; keybase=9007199254743001
layer=chart['definition']['layers'][0]
layer['mappings']['Source']['x']={'Timestamp':{'field':'1','origin':str(origin)}}
chart['definition']['axes']=[]
fields=data['datasets'][0]['batch']['fields']
fields[0]['kind']={'Timestamp':{'unit':'Nanoseconds','timezone':'UTC'}};fields[0]['name']='event_time';fields[1]['name']='value'
def batch(rows):
    return dict(schema_version='1',fields=fields,keys=[str(k) for k,t,y in rows],columns=[dict(values={'Timestamp':[str(t) for k,t,y in rows]},validity=[True]*len(rows),formatted=None),dict(values={'Float64':[y for k,t,y in rows]},validity=[True]*len(rows),formatted=None)])
initial=[(keybase+i,origin+5*i,5.+10*i) for i in range(5)]
data['datasets'][0]['batch']=batch(initial)
bin_stat=dict(operation=dict(id='chart.bin',version='1'),parameters={'Bin':dict(input={'Field':'2'},edges=[0,10,20,30,50],outliers='Exclude',grouping='All',space='Data')})
summary=dict(operation=dict(id='chart.summary',version='1'),parameters={'Summary':dict(input={'Field':'2'},grouping='All',quantiles=[0,.5,1],empty_sum_zero=False,space='Data')})
chart['definition']['transforms']=[dict(id=str(i+10),input={'Dataset':'1'},filters=[],statistic=stat,invalid='Exclude') for i,stat in enumerate([bin_stat,summary])]
rows={k:(t,y,i) for i,(k,t,y) in enumerate(initial)};next_ordinal=len(rows);revision=0;policy='Unbounded';steps=[]
def snapshot(): return [[str(k),str(t),y] for k,(t,y,o) in rows.items()]
def add(name,**kw):
    steps.append(dict(name=name,rows=snapshot(),revision=str(revision),retention=copy.deepcopy(policy),**kw))
def target(key):return dict(epoch='1',layer='1',panel=None,identity={'Source':dict(dataset='1',key=str(key))})
def transaction(name,operations):return dict(version=1,id=name,epoch='1',expected=[dict(dataset='1',revision=str(revision),schema_version='1')],operations=[dict(dataset='1',mutation=op) for op in operations])
def apply(operations):
    global rows,next_ordinal,revision,policy
    old=copy.deepcopy((rows,next_ordinal,policy))
    for op in operations:
        kind,payload=next(iter(op.items()))
        if kind=='SetRetention':policy=copy.deepcopy(payload)
        elif kind=='AdvanceWatermark':policy['EventTime']['watermark']=payload
        elif kind=='RemoveKeys':
            for k in payload:rows.pop(int(k),None)
        else:
            incoming=[(int(k),int(t),float(y)) for k,t,y in zip(payload['keys'],payload['columns'][0]['values']['Timestamp'],payload['columns'][1]['values']['Float64'])]
            if isinstance(policy,dict) and 'EventTime' in policy:
                p=policy['EventTime'];cutoff=int(p['watermark'])-int(p['width'])-int(p['allowed_lateness'])
                incoming=[r for r in incoming if r[1]>=cutoff]
            prior=rows
            if kind=='ReplaceSnapshot':rows={}
            for k,t,y in incoming:
                if k in prior:o=prior[k][2]
                else:o=next_ordinal;next_ordinal+=1
                rows[k]=(t,y,o)
        if isinstance(policy,dict) and 'Count' in policy:
            retained={k for k,v in sorted(rows.items(),key=lambda kv:kv[1][2],reverse=True)[:policy['Count']]};rows={k:v for k,v in rows.items() if k in retained}
        if isinstance(policy,dict) and 'EventTime' in policy:
            p=policy['EventTime'];cutoff=int(p['watermark'])-int(p['width'])-int(p['allowed_lateness']);rows={k:v for k,v in rows.items() if v[0]>=cutoff}
    if (rows,next_ordinal,policy)!=old:revision+=1

def commit(name,operations,**kw):
    tx=transaction(name,operations);apply(operations);add(name,transaction=tx,expected={'Applied':{'store_revision':str(revision)}},**kw);return tx
add('select-source',action={'Select':{'change':'Replace','targets':[target(keybase)]}})
add('pin-source',action={'SetPinned':target(keybase)})
add('pin-current',stream='Pinned',expected={'historical':False,'target':{'target':target(keybase)}})
add('freeze',action={'SetFollow':'FreezePresentation'})
add('queue-capacity',stream={'ConfigureQueue':dict(transactions=1,rows=4,bytes=4096,overload='Backpressure')})
window=dict(field='1',width='15',allowed_lateness='2',watermark=str(origin+20),late='Reject')
policy_ops=[{'SetRetention':{'EventTime':window}}];queued=transaction('retention-queued',policy_ops)
add('queue-policy',stream={'Enqueue':queued},expected='Queued')
add('queue-duplicate',stream={'Enqueue':queued},expected='AlreadyQueued')
add('cannot-reconfigure-accepted-work',stream={'ConfigureQueue':dict(transactions=1,rows=1,bytes=100,overload='DropNewest')},error='CHART_VALIDATION')
blocked=transaction('backpressured', [{'AppendBatch':batch([(keybase+10,origin+25,25.)])}])
add('queue-backpressure',stream={'Enqueue':blocked},expected='Backpressure')
add('queued-not-committed',stream='Status',expected={'store_revision':'0','queue':{'transactions':1,'accepted':'1','backpressured':'1'}})
apply(policy_ops);add('commit-policy',stream='CommitNext',expected={'outcome':{'Applied':{'store_revision':'1','operations':[{'evicted':1}]}},'reconciliation':{'removed_selection':[target(keybase)],'pinned_historical':True}})
add('pin-historical',stream='Pinned',expected={'historical':True,'target':{'target':target(keybase),'cells':[{'value':str(origin)+' Nanoseconds UTC'},{'value':'5'}]}})
add('cannot-select-evicted-frozen-mark',action={'Select':{'change':'Add','targets':[target(keybase)]}},error='CHART_VALIDATION')
add('resume',action='ResumeLatest',present=True)
add('retention-replay',transaction=queued,expected={'AlreadyApplied':{'store_revision':'1'}})
commit('advance-watermark',[{'AdvanceWatermark':str(origin+30)}],present=True)
bad=transaction('atomic-late-reject',[{'AppendBatch':batch([(keybase+11,origin+31,10.)])},{'UpsertByKey':batch([(keybase+4,origin,0.)])}])
add('atomic-late-reject',transaction=bad,expected={'Rejected':{'code':'CHART_VALIDATION'}})
window={**window,'watermark':str(origin+30),'late':'Drop'}
commit('choose-late-drop',[{'SetRetention':{'EventTime':window}}])
commit('future-does-not-advance',[{'AppendBatch':batch([(keybase+12,origin+1000,7.)])}],present=True)
commit('drop-and-accept',[{'UpsertByKey':batch([(keybase+13,origin,1.),(keybase+14,origin+25,29.)])}],present=True)
steps[-1]['expected']['Applied']['operations']=[{'late_dropped':1,'inserted':1,'updated':0}]
commit('count-retention',[{'SetRetention':{'Count':8}}])
add('inspect-window',action={'SetViewport':{'x':[0,40],'y':None}},present=True)
rng=random.Random(180923)
for i in range(40):
    if i%7==6:
        candidates=list(rows);rng.shuffle(candidates);chosen=candidates[:max(1,len(candidates)//2)]
        op={'ReplaceSnapshot':batch([(k,rows[k][0],rows[k][1]) for k in chosen])}
    elif i%4==2 and rows:
        k=rng.choice(list(rows));op={'UpsertByKey':batch([(k,origin+rng.randrange(10,70),float(rng.randrange(0,50)))])}
    elif i%4==3:
        k=rng.choice(list(rows)) if rows else keybase+9999;op={'RemoveKeys':[str(k),str(keybase+9999)]}
    else:
        k=keybase+100+i;op={'AppendBatch':batch([(k,origin+rng.randrange(10,70),float(rng.randrange(0,50)))])}
    commit(f'replay-{i:02}',[op],present=i%8==0)
add('queue-drained',stream='Status',expected={'queue':{'transactions':0,'rows':0,'bytes':0,'committed':'1','failed':'0'}})
add('replay-old-receipt-current-data',transaction=queued,expected={'AlreadyApplied':{'store_revision':'1'}})
add('choose-lossy-overload',stream={'ConfigureQueue':dict(transactions=1,rows=1,bytes=4096,overload='DropNewest')})
add('explicit-drop-newest',stream={'Enqueue':transaction('dropped-newest',[{'AppendBatch':batch([(keybase+900,origin+100,1.),(keybase+901,origin+101,2.)])}])},expected='Dropped')
add('explicit-loss-accounting',stream='Status',expected={'queue':{'transactions':0,'accepted':'0','dropped':'1','dropped_rows':'2'}})
add('no-dropped-commit',stream='CommitNext',expected=None)
add('unpin',action={'SetPinned':None})
add('released-pin',stream='Pinned',expected=None,present=True)
result=dict(name='stream-retention-replay',seed=180923,chart=chart,data=data,steps=steps)
(ROOT/'fixtures/streaming/replay.json').write_text(json.dumps(result,indent=2)+'\n')
print(f'Generated {len(steps)} deterministic replay steps; final revision {revision}.')
